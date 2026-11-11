//! Parser state, grammar entry points and token consumption.

use crate::ast::{
    ArrayElement, ArrowBody, BinaryOp, BinaryStep, BindingElement, BindingPatternKind,
    ChainStepKind, ClassElement, Expr, ExprKind, Label, Literal, ObjectElement, Parameter,
    PropertyName, Statement, StatementKind,
};
use crate::lexer::{Goal, Kind, Lexer, Token};
use crate::{MAX_DEPTH, MAX_EXPRESSION_DEPTH, early, reserved};
use crate::{arrow, assignment_pattern, checkpoint, cover, private, source};
use spite_core::{Diagnostic, DiagnosticKind, JsString, Span};
use spite_regexp as regexp;

#[derive(Clone, Copy)]
pub(crate) enum AwaitContext {
    None,
    Parameters,
    Body,
}

#[derive(Clone, Copy)]
pub(crate) enum YieldContext {
    None,
    Parameters,
    Body,
}

pub(crate) struct Parser {
    pub(crate) source: std::rc::Rc<source::SourceText>,
    pub(crate) lexer: Lexer,
    pub(crate) template_braces: Vec<usize>,
    pub(crate) computed_class_name: Option<(usize, usize, PropertyName)>,
    pub(crate) arrow_heads: std::collections::BTreeMap<cover::Key, Option<arrow::Head>>,
    pub(crate) pattern_covers:
        std::collections::BTreeMap<cover::Key, Option<assignment_pattern::Cover>>,
    pub(crate) probing_cover: bool,
    pub(crate) probe_error: Option<Diagnostic>,
    pub(crate) scan_error: Option<Diagnostic>,
    pub(crate) lookahead_error: Option<Diagnostic>,
    pub(crate) tokens: Vec<Token>,
    pub(crate) scan_checkpoints: Vec<checkpoint::ScanCheckpoint>,
    pub(crate) index: usize,
    pub(crate) depth: usize,
    pub(crate) allow_in: bool,
    pub(crate) allow_return: bool,
    pub(crate) allow_new_target: bool,
    pub(crate) allow_super_property: bool,
    pub(crate) allow_super_call: bool,
    pub(crate) allow_arguments: bool,
    pub(crate) allow_await_identifier: bool,
    pub(crate) module_goal: bool,
    pub(crate) await_context: AwaitContext,
    pub(crate) yield_context: YieldContext,
    pub(crate) private_scopes: Vec<private::PrivateScope>,
}

impl Parser {
    pub(crate) fn new(source: &str) -> Result<Self, Diagnostic> {
        Self::from_source(std::rc::Rc::new(source::SourceText::from_str(source)))
    }

    pub(crate) fn from_source(source: std::rc::Rc<source::SourceText>) -> Result<Self, Diagnostic> {
        let mut lexer = Lexer::new(source.clone());
        let checkpoint = checkpoint::ScanCheckpoint::new(&lexer, &[]);
        let token = lexer.next(Goal::HashbangOrRegExp)?;
        let template_braces = if matches!(token.kind, Kind::Template { tail: false, .. }) {
            vec![0]
        } else {
            Vec::new()
        };
        let tokens = vec![token];
        Ok(Self {
            source,
            lexer,
            template_braces,
            computed_class_name: None,
            arrow_heads: Default::default(),
            pattern_covers: Default::default(),
            probing_cover: false,
            probe_error: None,
            scan_error: None,
            lookahead_error: None,
            tokens,
            scan_checkpoints: vec![checkpoint],
            index: 0,
            depth: 0,
            allow_in: true,
            allow_return: false,
            allow_new_target: false,
            allow_super_property: false,
            allow_super_call: false,
            allow_arguments: true,
            allow_await_identifier: true,
            module_goal: false,
            await_context: AwaitContext::None,
            yield_context: YieldContext::None,
            private_scopes: Vec::new(),
        })
    }

    pub(crate) fn await_identifier_allowed(&self) -> bool {
        self.allow_await_identifier && !self.module_goal
    }

    pub(crate) fn current(&self) -> &Token {
        &self.tokens[self.index]
    }

    // Scanner and computed-name lookahead failures are reported at each public
    // parsing boundary, even if the syntactic parser accepts the EOF sentinel
    // or produces a secondary diagnostic. Keeping ordinary cursor operations
    // infallible avoids growing every recursive grammar frame with error paths.
    pub(crate) fn finish<T>(&self, result: Result<T, Diagnostic>) -> Result<T, Diagnostic> {
        match &self.lookahead_error {
            Some(error) => Err(error.clone()),
            None => result,
        }
    }

    // Cache only the lookahead demanded by the syntactic grammar. Consumed
    // tokens remain available for exact spans and strict legacy-token checks.
    pub(crate) fn token_at(&mut self, index: usize) -> Option<&Token> {
        while index >= self.tokens.len() {
            if self
                .tokens
                .last()
                .is_some_and(|token| token.kind == Kind::Eof)
            {
                return None;
            }
            let _ = self.cache_next_token(false);
        }
        self.tokens.get(index)
    }

    // The parser tracks template substitution braces. Grammar probes choose
    // the RegExp goal at primary expressions; the lexer has no brace state.
    pub(crate) fn scan_token(&mut self, regexp: bool) -> Result<Token, Diagnostic> {
        let goal = match (regexp, self.template_braces.last() == Some(&0)) {
            (false, false) => Goal::Div,
            (false, true) => Goal::TemplateTail,
            (true, false) => Goal::RegExp,
            (true, true) => Goal::RegExpOrTemplateTail,
        };
        let token = self.lexer.next(goal)?;
        match &token.kind {
            Kind::Template {
                tail, continuation, ..
            } => {
                if *continuation {
                    self.template_braces.pop();
                }
                if !tail {
                    if self.template_braces.len() >= MAX_EXPRESSION_DEPTH {
                        return Err(Diagnostic::new(
                            DiagnosticKind::Limit,
                            token.span,
                            "template nesting limit exceeded",
                        ));
                    }
                    self.template_braces.push(0);
                }
            }
            Kind::Punct(punct) => {
                if let Some(depth) = self.template_braces.last_mut() {
                    if *punct == "{" {
                        *depth += 1;
                    } else if *punct == "}" {
                        *depth -= 1;
                    }
                }
            }
            _ => {}
        }
        Ok(token)
    }
    // Grammar terminals cannot contain Unicode escapes (ECMA-262 5.1.5.1).
    pub(crate) fn at(&self, text: &str) -> bool {
        match &self.current().kind {
            Kind::Word(s) => !self.current().escaped && s == text,
            Kind::Punct(s) => *s == text,
            _ => false,
        }
    }
    pub(crate) fn bump(&mut self) -> Token {
        let token = self.current().clone();
        if token.kind != Kind::Eof {
            let _ = self.token_at(self.index + 1);
            self.index += 1;
        }
        token
    }
    pub(crate) fn eat(&mut self, text: &str) -> bool {
        if self.at(text) {
            self.bump();
            true
        } else {
            false
        }
    }
    pub(crate) fn error(&self, message: &str) -> Diagnostic {
        Diagnostic::new(DiagnosticKind::Syntax, self.current().span, message)
    }
    pub(crate) fn unsupported(&self, message: &str) -> Diagnostic {
        Diagnostic::new(DiagnosticKind::Unsupported, self.current().span, message)
    }
    pub(crate) fn expect(&mut self, text: &str) -> Result<(), Diagnostic> {
        if self.eat(text) {
            Ok(())
        } else {
            Err(self.error(&format!("expected {text}")))
        }
    }
    pub(crate) fn enter(&mut self) -> Result<(), Diagnostic> {
        if self.depth >= MAX_DEPTH {
            return Err(Diagnostic::new(
                DiagnosticKind::Limit,
                self.current().span,
                "syntax nesting limit exceeded",
            ));
        }
        self.depth += 1;
        Ok(())
    }
    pub(crate) fn semicolon(&mut self) -> Result<(), Diagnostic> {
        if self.eat(";")
            || self.at("}")
            || self.current().kind == Kind::Eof
            || self.current().newline
        {
            Ok(())
        } else {
            Err(self.error("expected a semicolon or line terminator"))
        }
    }
    pub(crate) fn statement(&mut self, allow_declaration: bool) -> Result<Statement, Diagnostic> {
        self.enter()?;
        let result = self.statement_inner(allow_declaration);
        self.depth -= 1;
        result
    }
    pub(crate) fn statement_inner(
        &mut self,
        allow_declaration: bool,
    ) -> Result<Statement, Diagnostic> {
        let start = self.current().span.start;
        if matches!(&self.current().kind, Kind::Word(name) if !reserved(name))
            && self
                .token_at(self.index + 1)
                .is_some_and(|t| t.kind == Kind::Punct(":"))
        {
            return self.labelled_statement(start);
        }
        // ECMA-262 14.5 forbids an ExpressionStatement starting with `let [`,
        // but permits `let` as an IdentifierReference in non-strict code. In a
        // Statement position, ASI can separate it from a following name or `{`.
        if !allow_declaration
            && self.at("let")
            && self
                .token_at(self.index + 1)
                .is_some_and(|t| t.kind == Kind::Punct("["))
        {
            return Err(self.error("expression statement cannot start with let ["));
        }
        let lexical = self.at("const") || (allow_declaration && self.at_let_declaration());
        // 14.5's ExpressionStatement lookahead cannot turn declarations into
        // statements, even when the declaration's body is not implemented yet.
        if !allow_declaration && (self.at("class") || self.at_async_function()) {
            return Err(self.error("declaration requires a statement list"));
        }
        let kind = if self.eat(";") {
            StatementKind::Empty
        } else if self.eat("{") {
            return self.block_statement(start);
        } else if self.at("function") || self.at_async_function() {
            if !allow_declaration {
                return Err(self.error("function declaration requires a statement list"));
            }
            StatementKind::Function(self.ordinary_function(true)?)
        } else if self.at("class") {
            StatementKind::Class(self.class_definition(true)?)
        } else if self.eat("var") {
            let bindings = self.binding_list(false, false, false)?;
            self.semicolon()?;
            StatementKind::Var(bindings)
        } else if self.at_using_declaration(false) {
            if !allow_declaration {
                return Err(self.error("using declaration requires a statement list"));
            }
            let bindings = self.using_bindings(false)?;
            self.semicolon()?;
            StatementKind::Using(bindings)
        } else if lexical {
            if !allow_declaration {
                return Err(self.error("lexical declaration requires a statement list"));
            }
            let (mutable, bindings) = self.lexical_bindings(false)?;
            self.semicolon()?;
            StatementKind::Lexical { mutable, bindings }
        } else if self.eat("try") {
            return self.try_statement(start);
        } else if self.eat("switch") {
            return self.switch_statement(start);
        } else if self.eat("if") {
            return self.if_statement(start);
        } else if self.eat("for") {
            return self.for_statement(start);
        } else if self.eat("while") {
            return self.while_statement(start);
        } else if self.eat("with") {
            return self.with_statement(start);
        } else if self.eat("do") {
            return self.do_while_statement(start);
        } else if self.at("break") || self.at("continue") {
            let is_break = self.eat("break");
            if !is_break {
                self.expect("continue")?;
            }
            // ECMA-262 14.8/14.9: a label cannot follow a line terminator.
            let target = if !self.current().newline && matches!(self.current().kind, Kind::Word(_))
            {
                Some(self.label_identifier()?)
            } else {
                None
            };
            self.semicolon()?;
            if is_break {
                StatementKind::Break(target)
            } else {
                StatementKind::Continue(target)
            }
        } else if self.eat("debugger") {
            self.semicolon()?;
            StatementKind::Debugger
        } else if self.at("return") {
            return self.return_statement(start);
        } else if self.eat("throw") {
            return self.throw_statement(start);
        } else {
            return self.expression_statement(start);
        };
        Ok(self.finish_statement(start, kind))
    }

    pub(crate) fn finish_statement(&self, start: usize, kind: StatementKind) -> Statement {
        let end = self.tokens[self.index.saturating_sub(1)].span.end;
        Statement {
            kind,
            span: Span::new(start, end),
        }
    }

    pub(crate) fn required_block(&mut self) -> Result<Box<Statement>, Diagnostic> {
        if !self.at("{") {
            return Err(self.error("expected {"));
        }
        Ok(Box::new(self.statement(false)?))
    }

    pub(crate) fn lexical_bindings(
        &mut self,
        for_header: bool,
    ) -> Result<(bool, Vec<BindingElement>), Diagnostic> {
        let mutable = self.eat("let");
        if !mutable {
            self.expect("const")?;
        }
        Ok((mutable, self.binding_list(!mutable, for_header, true)?))
    }

    pub(crate) fn binding_list(
        &mut self,
        require_initializer: bool,
        for_header: bool,
        lexical: bool,
    ) -> Result<Vec<BindingElement>, Diagnostic> {
        let mut bindings = Vec::new();
        loop {
            let pattern = self.binding_pattern()?;
            if lexical {
                for (name, span) in pattern.bound_names() {
                    if name == "let" {
                        return Err(early(span, "invalid binding identifier"));
                    }
                }
            }
            let initializer = if self.eat("=") {
                Some(self.expression_with_in(2, !for_header)?)
            } else {
                None
            };
            if initializer.is_none() && !(for_header && (self.at("of") || self.at("in"))) {
                if !matches!(pattern.kind, BindingPatternKind::Identifier(_)) {
                    return Err(self.error("binding pattern requires an initializer"));
                }
                if require_initializer {
                    return Err(self.error("const requires an initializer"));
                }
            }
            bindings.push(BindingElement {
                pattern,
                initializer,
            });
            if !self.eat(",") {
                break;
            }
        }
        Ok(bindings)
    }

    pub(crate) fn label_identifier(&mut self) -> Result<Label, Diagnostic> {
        let token = self.bump();
        match token.kind {
            Kind::Word(name)
                if !reserved(&name)
                    && (name != "await" || self.await_identifier_allowed())
                    && (name != "yield" || matches!(self.yield_context, YieldContext::None)) =>
            {
                Ok(Label {
                    name,
                    span: token.span,
                })
            }
            _ => Err(early(token.span, "invalid label identifier")),
        }
    }

    pub(crate) fn make_expr(&self, kind: ExprKind, span: Span) -> Result<Expr, Diagnostic> {
        if !self.allow_arguments
            && matches!(&kind, ExprKind::Identifier(name) if name == "arguments")
        {
            return Err(early(
                span,
                "arguments is not allowed in class initialization",
            ));
        }
        if self.module_goal && matches!(&kind, ExprKind::Identifier(name) if name == "await") {
            return Err(early(span, "await cannot be an identifier in Modules"));
        }
        if !self.allow_await_identifier
            && matches!(&kind, ExprKind::Identifier(name) if name == "await")
        {
            return Err(early(
                span,
                match self.await_context {
                    AwaitContext::None => "await is not allowed in a static initialization block",
                    AwaitContext::Parameters => "await is not allowed in async function parameters",
                    AwaitContext::Body => "await cannot be an identifier in an async function",
                },
            ));
        }
        if !matches!(self.yield_context, YieldContext::None)
            && matches!(&kind, ExprKind::Identifier(name) if name == "yield")
        {
            return Err(early(
                span,
                "yield cannot be an identifier in a generator context",
            ));
        }
        let depth = 1 + match &kind {
            ExprKind::Unary(_, e) | ExprKind::Await(e) | ExprKind::Parenthesized(e) => e.depth,
            ExprKind::PrivateIn { value, .. } => value.depth,
            ExprKind::ImportCall { specifier, options } => specifier
                .depth
                .max(options.as_ref().map_or(0, |options| options.depth)),
            ExprKind::Yield { argument, .. } => {
                argument.as_ref().map_or(0, |argument| argument.depth)
            }
            ExprKind::Update { argument, .. } => argument.depth,
            ExprKind::Function(function) => function
                .parameters
                .iter()
                .map(Parameter::expression_depth)
                .max()
                .unwrap_or(0),
            ExprKind::Class(class) => class
                .elements
                .iter()
                .map(|element| {
                    let key = match element.name() {
                        Some(PropertyName::Computed(key)) => key.depth,
                        Some(PropertyName::Literal(_) | PropertyName::Private(_)) | None => 0,
                    };
                    key.max(match element {
                        ClassElement::Method { property, .. } => property.value.depth,
                        ClassElement::Field { initializer, .. } => initializer
                            .as_ref()
                            .map_or(0, |expression| expression.depth),
                        ClassElement::StaticBlock { .. } => 0,
                    })
                })
                .chain(
                    class
                        .constructor
                        .parameters
                        .iter()
                        .map(Parameter::expression_depth),
                )
                .chain(class.heritage.iter().map(|heritage| heritage.depth))
                .max()
                .unwrap_or(0),
            ExprKind::Arrow {
                parameters, body, ..
            } => parameters
                .iter()
                .map(Parameter::expression_depth)
                .max()
                .unwrap_or(0)
                .max(match body {
                    ArrowBody::Expression(body) => body.depth,
                    // Statement nesting is bounded while parsing the body.
                    ArrowBody::Block(_) => 0,
                }),
            ExprKind::Binary(_, a, b)
            | ExprKind::Assign(a, b)
            | ExprKind::CompoundAssign(_, a, b) => a.depth.max(b.depth),
            ExprKind::DestructuringAssign { pattern, value } => {
                pattern.expression_depth().max(value.depth)
            }
            ExprKind::BinaryChain { head, steps } => head
                .depth
                .max(steps.iter().map(|step| step.right.depth).max().unwrap_or(0)),
            ExprKind::Member(base, name) => base.depth.max(match name {
                PropertyName::Computed(key) => key.depth,
                PropertyName::Literal(_) | PropertyName::Private(_) => 0,
            }),
            ExprKind::SuperProperty(name) => match name {
                PropertyName::Computed(key) => key.depth,
                PropertyName::Literal(_) | PropertyName::Private(_) => 0,
            },
            ExprKind::SuperCall(arguments) => arguments
                .iter()
                .map(|argument| argument.expression().depth)
                .max()
                .unwrap_or(0),
            ExprKind::Call { callee, arguments } => arguments
                .iter()
                .map(|argument| argument.expression().depth)
                .max()
                .unwrap_or(0)
                .max(callee.depth),
            ExprKind::New { callee, arguments } => arguments
                .iter()
                .flatten()
                .map(|argument| argument.expression().depth)
                .max()
                .unwrap_or(0)
                .max(callee.depth),
            ExprKind::Conditional(a, b, c) => a.depth.max(b.depth).max(c.depth),
            ExprKind::OptionalChain { base, steps } => steps
                .iter()
                .map(|step| match &step.kind {
                    ChainStepKind::Property(PropertyName::Computed(expression)) => expression.depth,
                    ChainStepKind::Property(
                        PropertyName::Literal(_) | PropertyName::Private(_),
                    ) => 0,
                    ChainStepKind::Call(arguments) => arguments
                        .iter()
                        .map(|argument| argument.expression().depth)
                        .max()
                        .unwrap_or(0),
                })
                .max()
                .unwrap_or(0)
                .max(base.depth),
            ExprKind::TaggedTemplate {
                tag, substitutions, ..
            } => substitutions
                .iter()
                .map(|expression| expression.depth)
                .max()
                .unwrap_or(0)
                .max(tag.depth),
            ExprKind::Template { substitutions, .. } => {
                substitutions.iter().map(|e| e.depth).max().unwrap_or(0)
            }
            ExprKind::Array(elements) => elements
                .iter()
                .filter_map(ArrayElement::expression)
                .map(|e| e.depth)
                .max()
                .unwrap_or(0),
            ExprKind::Object(properties) => properties
                .iter()
                .map(|element| match element {
                    ObjectElement::Spread(expression) => expression.depth,
                    ObjectElement::Property(property) => {
                        let key_depth = match &property.name {
                            PropertyName::Computed(key) => key.depth,
                            PropertyName::Literal(_) | PropertyName::Private(_) => 0,
                        };
                        key_depth.max(property.value.depth)
                    }
                })
                .max()
                .unwrap_or(0),
            _ => 0,
        };
        if depth > MAX_EXPRESSION_DEPTH {
            return Err(Diagnostic::new(
                DiagnosticKind::Limit,
                span,
                "expression depth limit exceeded",
            ));
        }
        Ok(Expr { kind, span, depth })
    }

    pub(crate) fn member_property_name(&mut self) -> Result<PropertyName, Diagnostic> {
        Ok(if self.eat(".") {
            if self.at("#") {
                let name = self.private_identifier()?;
                self.use_private_identifier(&name)?;
                return Ok(PropertyName::Private(name));
            }
            let token = self.bump();
            let name = match token.kind {
                Kind::Word(name) => JsString::from(name.as_str()),
                Kind::Literal(Literal::Null) => JsString::from("null"),
                Kind::Literal(Literal::Boolean(value)) => {
                    JsString::from(if value { "true" } else { "false" })
                }
                _ => {
                    return Err(early(token.span, "expected an identifier name after dot"));
                }
            };
            PropertyName::Literal(Literal::String(name))
        } else {
            self.expect("[")?;
            let key = self.expression_with_in(1, true)?;
            self.expect("]")?;
            PropertyName::Computed(Box::new(key))
        })
    }

    pub(crate) fn expression(&mut self, minimum: u8) -> Result<Expr, Diagnostic> {
        self.enter()?;
        let result = self.expression_inner(minimum);
        self.depth -= 1;
        result
    }

    pub(crate) fn expression_with_in(
        &mut self,
        minimum: u8,
        allow_in: bool,
    ) -> Result<Expr, Diagnostic> {
        let previous = self.allow_in;
        self.allow_in = allow_in;
        let result = self.expression(minimum);
        self.allow_in = previous;
        result
    }
    pub(crate) fn expression_inner(&mut self, minimum: u8) -> Result<Expr, Diagnostic> {
        let left = self.expression_head(minimum)?;
        self.expression_tail(left, minimum)
    }

    pub(crate) fn make_binary_expr(
        &self,
        op: BinaryOp,
        left: Expr,
        right: Expr,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let Expr {
            kind,
            span: left_span,
            depth,
        } = left;
        let step = BinaryStep { op, right, span };
        let kind = match kind {
            ExprKind::Binary(previous, head, right) => ExprKind::BinaryChain {
                head,
                steps: vec![
                    BinaryStep {
                        op: previous,
                        right: *right,
                        span: left_span,
                    },
                    step,
                ],
            },
            ExprKind::BinaryChain { head, mut steps } => {
                // Existing chain depth is already cached. Extending a flat chain
                // must not rescan all earlier operands and become quadratic.
                let depth = depth.max(1 + step.right.depth);
                if depth > MAX_EXPRESSION_DEPTH {
                    return Err(Diagnostic::new(
                        DiagnosticKind::Limit,
                        span,
                        "expression depth limit exceeded",
                    ));
                }
                steps.push(step);
                return Ok(Expr {
                    kind: ExprKind::BinaryChain { head, steps },
                    span,
                    depth,
                });
            }
            kind => ExprKind::Binary(
                op,
                Box::new(Expr {
                    kind,
                    span: left_span,
                    depth,
                }),
                Box::new(step.right),
            ),
        };
        self.make_expr(kind, span)
    }
    pub(crate) fn at_async_function(&mut self) -> bool {
        self.at("async")
            && self.token_at(self.index + 1).is_some_and(|token| {
                !token.newline
                    && !token.escaped
                    && matches!(&token.kind, Kind::Word(name) if name == "function")
            })
    }

    // Literal early errors use the same Pattern grammar as construction. Native
    // evaluation instantiates the RegExp; unsupported matching remains a runtime
    // boundary, just as it is for an equivalent constructor-created instance.
    pub(crate) fn regexp_diagnostic(&mut self) -> Option<Diagnostic> {
        let span = self.current().span;
        if self
            .lookahead_error
            .as_ref()
            .is_some_and(|error| error.span.start >= span.start)
        {
            // Div-goal cover lookahead may have scanned literal contents as
            // JavaScript. That diagnostic belongs to the wrong lexical goal.
            self.lookahead_error = None;
            self.scan_error = None;
        }
        if !matches!(self.current().kind, Kind::RegExp { .. })
            && let Err(error) = self.rescan_regexp()
        {
            return Some(error);
        }
        let Token {
            kind: Kind::RegExp { body, flags },
            span,
            ..
        } = self.current()
        else {
            unreachable!("RegExp goal at a primary-expression solidus");
        };
        regexp::validate_regexp_pattern(body, flags, *span).err()
    }
}
