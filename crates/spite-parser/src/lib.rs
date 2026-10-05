//! Parsing and early-error validation for the implemented ECMAScript subset.

mod array;
mod arrow;
mod assignment_pattern;
pub mod ast;
mod binding;
mod construction;
mod dynamic_function;
mod function;
mod iteration;
pub mod json;
mod lexer;
mod object;
mod optional_chain;
mod template;

pub use dynamic_function::parse_dynamic_function;

use ast::*;
use lexer::{Kind, Lexer, Token};
use spite_core::{Diagnostic, DiagnosticKind, JsString, Span};
use std::collections::BTreeSet;

/// Maximum recursive syntax depth accepted by this parser.
pub const MAX_DEPTH: usize = 64;

/// Parses a Script with an opted-in maximum UTF-8 source length.
pub fn parse_script_with_source_limit(
    source: &str,
    max_bytes: usize,
) -> Result<Script, Diagnostic> {
    if source.len() > max_bytes {
        return Err(Diagnostic::new(
            DiagnosticKind::Limit,
            Span::new(0, source.len()),
            "source size limit exceeded",
        ));
    }
    parse_script(source)
}

/// Parses a Script and validates implemented early errors before returning it.
///
/// This is not yet a complete ECMAScript parser. Unsupported features produce
/// [`DiagnosticKind::Unsupported`] where they can be recognized.
pub fn parse_script(source: &str) -> Result<Script, Diagnostic> {
    let mut parser = Parser::new(source)?;
    let mut statements = Vec::new();
    while parser.current().kind != Kind::Eof {
        statements.push(parser.statement(true)?);
    }
    let strict = has_use_strict(&statements, source);
    if strict {
        reject_legacy_tokens(&parser.tokens)?;
    }
    validate_scope(
        &statements,
        strict,
        ControlContext::default(),
        &mut Vec::new(),
        ScopeKind::Variable,
    )?;
    Ok(Script { statements, strict })
}

// ECMA-262 14.1: only unescaped, unparenthesized string directives count.
fn has_use_strict(statements: &[Statement], source: &str) -> bool {
    statements
        .iter()
        .take_while(|s| {
            matches!(
                s.kind,
                StatementKind::Expression(Expr {
                    kind: ExprKind::Literal(Literal::String(_)),
                    ..
                })
            )
        })
        .any(|s| {
            let StatementKind::Expression(expr) = &s.kind else {
                return false;
            };
            matches!(
                &source[expr.span.start..expr.span.end],
                "\"use strict\"" | "'use strict'"
            )
        })
}

fn reject_legacy_tokens(tokens: &[Token]) -> Result<(), Diagnostic> {
    // Strictness also applies to earlier directives and nested function code.
    if let Some(token) = tokens.iter().find(|token| token.legacy) {
        return Err(early(
            token.span,
            "legacy numeric literals and escapes are forbidden in strict code",
        ));
    }
    Ok(())
}

struct Parser {
    source: std::rc::Rc<str>,
    tokens: Vec<Token>,
    index: usize,
    depth: usize,
    allow_in: bool,
    allow_return: bool,
    allow_new_target: bool,
}

impl Parser {
    fn new(source: &str) -> Result<Self, Diagnostic> {
        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();
        loop {
            let token = lexer.next()?;
            let done = token.kind == Kind::Eof;
            tokens.push(token);
            if done {
                break;
            }
        }
        Ok(Self {
            source: std::rc::Rc::from(source),
            tokens,
            index: 0,
            depth: 0,
            allow_in: true,
            allow_return: false,
            allow_new_target: false,
        })
    }

    fn current(&self) -> &Token {
        &self.tokens[self.index]
    }
    // Grammar terminals cannot contain Unicode escapes (ECMA-262 5.1.5.1).
    fn at(&self, text: &str) -> bool {
        match &self.current().kind {
            Kind::Word(s) => !self.current().escaped && s == text,
            Kind::Punct(s) => *s == text,
            _ => false,
        }
    }
    fn bump(&mut self) -> Token {
        let token = self.current().clone();
        if token.kind != Kind::Eof {
            self.index += 1;
        }
        token
    }
    fn eat(&mut self, text: &str) -> bool {
        if self.at(text) {
            self.bump();
            true
        } else {
            false
        }
    }
    fn error(&self, message: &str) -> Diagnostic {
        Diagnostic::new(DiagnosticKind::Syntax, self.current().span, message)
    }
    fn unsupported(&self, message: &str) -> Diagnostic {
        Diagnostic::new(DiagnosticKind::Unsupported, self.current().span, message)
    }
    fn expect(&mut self, text: &str) -> Result<(), Diagnostic> {
        if self.eat(text) {
            Ok(())
        } else {
            Err(self.error(&format!("expected {text}")))
        }
    }
    fn enter(&mut self) -> Result<(), Diagnostic> {
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
    fn semicolon(&mut self) -> Result<(), Diagnostic> {
        if self.eat(";")
            || self.at("}")
            || self.current().kind == Kind::Eof
            || self.current().newline
        {
            Ok(())
        } else if matches!(self.current().kind, Kind::Punct("(" | "[" | "." | "?.")) {
            Err(self.unsupported("calls and property access are not implemented"))
        } else {
            Err(self.error("expected a semicolon or line terminator"))
        }
    }
    fn statement(&mut self, allow_declaration: bool) -> Result<Statement, Diagnostic> {
        self.enter()?;
        let result = self.statement_inner(allow_declaration);
        self.depth -= 1;
        result
    }
    fn statement_inner(&mut self, allow_declaration: bool) -> Result<Statement, Diagnostic> {
        let start = self.current().span.start;
        if matches!(&self.current().kind, Kind::Word(name) if !reserved(name))
            && self
                .tokens
                .get(self.index + 1)
                .is_some_and(|t| t.kind == Kind::Punct(":"))
        {
            let label = self.label_identifier()?;
            self.expect(":")?;
            // ECMA-262 14.13.1: labelled functions require the excluded Annex B
            // extension even in non-strict code.
            if self.at("function") {
                return Err(self.error("labelled functions are not allowed"));
            }
            let body = Box::new(self.statement(false)?);
            let span = Span::new(start, body.span.end);
            return Ok(Statement {
                kind: StatementKind::Labelled { label, body },
                span,
            });
        }
        // ECMA-262 14.5 forbids an ExpressionStatement starting with `let [`,
        // but permits `let` as an IdentifierReference in non-strict code. In a
        // Statement position, ASI can separate it from a following name or `{`.
        if !allow_declaration
            && self.at("let")
            && self
                .tokens
                .get(self.index + 1)
                .is_some_and(|t| t.kind == Kind::Punct("["))
        {
            return Err(self.error("expression statement cannot start with let ["));
        }
        let lexical = self.at("const")
            || (allow_declaration
                && self.at("let")
                && self
                    .tokens
                    .get(self.index + 1)
                    .is_some_and(|t| matches!(t.kind, Kind::Word(_) | Kind::Punct("[" | "{"))));
        // 14.5's ExpressionStatement lookahead cannot turn declarations into
        // statements, even when the declaration's body is not implemented yet.
        if !allow_declaration && (self.at("class") || self.at_async_function()) {
            return Err(self.error("declaration requires a statement list"));
        }
        let kind = if self.eat(";") {
            StatementKind::Empty
        } else if self.eat("{") {
            let mut body = Vec::new();
            while !self.at("}") {
                if self.current().kind == Kind::Eof {
                    return Err(self.error("unterminated block"));
                }
                body.push(self.statement(true)?);
            }
            self.expect("}")?;
            StatementKind::Block(body)
        } else if self.at("function") {
            if !allow_declaration {
                return Err(self.error("function declaration requires a statement list"));
            }
            StatementKind::Function(self.ordinary_function(true)?)
        } else if self.eat("var") {
            let bindings = self.binding_list(false, false, false)?;
            self.semicolon()?;
            StatementKind::Var(bindings)
        } else if lexical {
            if !allow_declaration {
                return Err(self.error("lexical declaration requires a statement list"));
            }
            let (mutable, bindings) = self.lexical_bindings(false)?;
            self.semicolon()?;
            StatementKind::Lexical { mutable, bindings }
        } else if self.eat("try") {
            // ECMA-262 14.15 requires blocks, not arbitrary statements.
            let body = self.required_block()?;
            let handler = if self.at("catch") {
                let start = self.bump().span.start;
                let parameter = if self.eat("(") {
                    let parameter = self.binding_pattern()?;
                    self.expect(")")?;
                    Some(parameter)
                } else {
                    None
                };
                let body = self.required_block()?;
                let span = Span::new(start, body.span.end);
                Some(Box::new(CatchClause {
                    parameter,
                    body,
                    span,
                }))
            } else {
                None
            };
            let finalizer = if self.eat("finally") {
                Some(self.required_block()?)
            } else {
                None
            };
            if handler.is_none() && finalizer.is_none() {
                return Err(self.error("expected catch or finally"));
            }
            StatementKind::Try {
                body,
                handler,
                finalizer,
            }
        } else if self.eat("switch") {
            self.expect("(")?;
            let discriminant = self.expression(1)?;
            self.expect(")")?;
            self.expect("{")?;
            let mut clauses = Vec::new();
            let mut has_default = false;
            while !self.at("}") {
                let start = self.current().span.start;
                let test = if self.eat("case") {
                    Some(self.expression(1)?)
                } else if self.at("default") {
                    if has_default {
                        return Err(self.error("duplicate default clause"));
                    }
                    self.bump();
                    has_default = true;
                    None
                } else {
                    return Err(self.error("expected case, default, or }"));
                };
                self.expect(":")?;
                let mut statements = Vec::new();
                while !self.at("case") && !self.at("default") && !self.at("}") {
                    if self.current().kind == Kind::Eof {
                        return Err(self.error("unterminated switch"));
                    }
                    statements.push(self.statement(true)?);
                }
                let end = self.tokens[self.index - 1].span.end;
                clauses.push(SwitchClause {
                    test,
                    statements,
                    span: Span::new(start, end),
                });
            }
            self.expect("}")?;
            StatementKind::Switch {
                discriminant,
                clauses,
            }
        } else if self.eat("if") {
            self.expect("(")?;
            let test = self.expression(1)?;
            self.expect(")")?;
            let consequent = Box::new(self.statement(false)?);
            let alternate = if self.eat("else") {
                Some(Box::new(self.statement(false)?))
            } else {
                None
            };
            StatementKind::If {
                test,
                consequent,
                alternate,
            }
        } else if self.eat("for") {
            let header = self.for_header()?;
            let body = Box::new(self.statement(false)?);
            header.with_body(body)
        } else if self.eat("while") {
            self.expect("(")?;
            let test = self.expression(1)?;
            self.expect(")")?;
            let body = Box::new(self.statement(false)?);
            StatementKind::While { test, body }
        } else if self.eat("with") {
            self.expect("(")?;
            let object = self.expression_with_in(1, true)?;
            self.expect(")")?;
            let body = Box::new(self.statement(false)?);
            StatementKind::With { object, body }
        } else if self.eat("do") {
            let body = Box::new(self.statement(false)?);
            self.expect("while")?;
            self.expect("(")?;
            let test = self.expression(1)?;
            self.expect(")")?;
            // ECMA-262 12.10.1 allows ASI after the closing parenthesis of
            // do-while even without a line terminator before the next token.
            self.eat(";");
            StatementKind::DoWhile { body, test }
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
            if !self.allow_return {
                return Err(self.error("return outside a function body"));
            }
            self.bump();
            // ECMA-262 14.10: a line terminator ends a bare return.
            let value = if self.current().newline
                || self.at(";")
                || self.at("}")
                || self.current().kind == Kind::Eof
            {
                None
            } else {
                Some(self.expression_with_in(1, true)?)
            };
            self.semicolon()?;
            StatementKind::Return(value)
        } else if self.eat("throw") {
            if self.current().newline {
                return Err(self.error("line terminator after throw"));
            }
            let expr = self.expression(1)?;
            self.semicolon()?;
            StatementKind::Throw(expr)
        } else {
            if let Kind::Word(word) = &self.current().kind {
                if !self.current().escaped && matches!(word.as_str(), "class" | "import" | "export")
                {
                    return Err(self.unsupported("statement is not implemented"));
                }
            }
            let expr = self.expression(1)?;
            self.semicolon()?;
            StatementKind::Expression(expr)
        };
        let end = self.tokens[self.index.saturating_sub(1)].span.end;
        Ok(Statement {
            kind,
            span: Span::new(start, end),
        })
    }

    fn required_block(&mut self) -> Result<Box<Statement>, Diagnostic> {
        if !self.at("{") {
            return Err(self.error("expected {"));
        }
        Ok(Box::new(self.statement(false)?))
    }

    fn lexical_bindings(
        &mut self,
        for_header: bool,
    ) -> Result<(bool, Vec<BindingElement>), Diagnostic> {
        let mutable = self.eat("let");
        if !mutable {
            self.expect("const")?;
        }
        Ok((mutable, self.binding_list(!mutable, for_header, true)?))
    }

    fn binding_list(
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

    fn label_identifier(&mut self) -> Result<Label, Diagnostic> {
        let token = self.bump();
        match token.kind {
            Kind::Word(name) if !reserved(&name) => Ok(Label {
                name,
                span: token.span,
            }),
            _ => Err(early(token.span, "invalid label identifier")),
        }
    }

    fn make_expr(&self, kind: ExprKind, span: Span) -> Result<Expr, Diagnostic> {
        let depth = 1 + match &kind {
            ExprKind::Unary(_, e) | ExprKind::Parenthesized(e) => e.depth,
            ExprKind::Update { argument, .. } => argument.depth,
            ExprKind::Function(function) => function
                .parameters
                .iter()
                .map(Parameter::expression_depth)
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
            ExprKind::Member(base, name) => base.depth.max(match name {
                PropertyName::Computed(key) => key.depth,
                PropertyName::Literal(_) => 0,
            }),
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
                    ChainStepKind::Property(PropertyName::Literal(_)) => 0,
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
                            PropertyName::Literal(_) => 0,
                        };
                        key_depth.max(property.value.depth)
                    }
                })
                .max()
                .unwrap_or(0),
            _ => 0,
        };
        if depth > MAX_DEPTH {
            return Err(Diagnostic::new(
                DiagnosticKind::Limit,
                span,
                "expression depth limit exceeded",
            ));
        }
        Ok(Expr { kind, span, depth })
    }

    fn expression(&mut self, minimum: u8) -> Result<Expr, Diagnostic> {
        self.enter()?;
        let result = self.expression_inner(minimum);
        self.depth -= 1;
        result
    }

    fn expression_with_in(&mut self, minimum: u8, allow_in: bool) -> Result<Expr, Diagnostic> {
        let previous = self.allow_in;
        self.allow_in = allow_in;
        let result = self.expression(minimum);
        self.allow_in = previous;
        result
    }
    fn expression_inner(&mut self, minimum: u8) -> Result<Expr, Diagnostic> {
        let mut left = if minimum <= 2 {
            if self.pattern_cover_end().is_some_and(|end| {
                self.tokens
                    .get(end + 1)
                    .is_some_and(|token| token.kind == Kind::Punct("="))
            }) {
                let pattern = self.assignment_pattern()?;
                self.expect("=")?;
                let value = self.expression(2)?;
                let span = Span::new(pattern.span.start, value.span.end);
                self.make_expr(
                    ExprKind::DestructuringAssign {
                        pattern: Box::new(pattern),
                        value: Box::new(value),
                    },
                    span,
                )?
            } else {
                match self.arrow_expression()? {
                    Some(arrow) => arrow,
                    None => self.prefix()?,
                }
            }
        } else {
            self.prefix()?
        };
        loop {
            if minimum <= 17 && self.at("(") {
                if !member_base(&left) {
                    if self.current().newline {
                        break;
                    }
                    return Err(self.error("call requires a left-hand-side expression"));
                }
                let arguments = self.arguments()?;
                let span = Span::new(left.span.start, self.tokens[self.index - 1].span.end);
                if matches!(left.kind, ExprKind::OptionalChain { .. }) {
                    left = self.append_chain_step(
                        left,
                        ChainStep {
                            optional: false,
                            kind: ChainStepKind::Call(arguments),
                            span,
                        },
                    )?;
                    continue;
                }
                left = self.make_expr(
                    ExprKind::Call {
                        callee: Box::new(left),
                        arguments,
                    },
                    span,
                )?;
                continue;
            }
            if minimum <= 18 && (self.at(".") || self.at("[")) {
                if !member_base(&left) {
                    // 12.10.1: a completed UpdateExpression cannot continue as
                    // a MemberExpression. A line break can therefore allow ASI.
                    if self.current().newline {
                        break;
                    }
                    return Err(self.error("property access requires a left-hand-side expression"));
                }
                let property = if self.eat(".") {
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
                };
                let span = Span::new(left.span.start, self.tokens[self.index - 1].span.end);
                left = if matches!(left.kind, ExprKind::OptionalChain { .. }) {
                    self.append_chain_step(
                        left,
                        ChainStep {
                            optional: false,
                            kind: ChainStepKind::Property(property),
                            span,
                        },
                    )?
                } else {
                    self.make_expr(ExprKind::Member(Box::new(left), property), span)?
                };
                continue;
            }
            if minimum <= 18
                && matches!(
                    self.current().kind,
                    Kind::Template {
                        continuation: false,
                        ..
                    }
                )
            {
                if !member_base(&left) {
                    if self.current().newline {
                        break;
                    }
                    return Err(self.error("tagged template requires a left-hand-side expression"));
                }
                if matches!(left.kind, ExprKind::OptionalChain { .. }) {
                    return Err(self.error("optional chains cannot be tagged templates"));
                }
                let token = self.bump();
                let Kind::Template { element, tail, .. } = token.kind else {
                    unreachable!("template head")
                };
                left = self.template_literal(element, tail, token.span, Some(left))?;
                continue;
            }
            if minimum <= 18 && self.at("?.") {
                if !member_base(&left) {
                    if self.current().newline {
                        break;
                    }
                    return Err(self.error("optional chain requires a left-hand-side expression"));
                }
                left = self.optional_chain_step(left)?;
                continue;
            }
            // ECMA-262 13.4: a postfix update cannot cross a line terminator.
            if minimum <= 16 && !self.current().newline && (self.at("++") || self.at("--")) {
                if !assignment_target(&left) {
                    return Err(early(left.span, "invalid update target"));
                }
                let token = self.bump();
                let op = if token.kind == Kind::Punct("++") {
                    UpdateOp::Increment
                } else {
                    UpdateOp::Decrement
                };
                let span = Span::new(left.span.start, token.span.end);
                left = self.make_expr(
                    ExprKind::Update {
                        op,
                        argument: Box::new(left),
                        prefix: false,
                    },
                    span,
                )?;
                continue;
            }
            let assignment_op = compound_assignment(&self.current().kind);
            if minimum <= 2 && (self.at("=") || assignment_op.is_some()) {
                self.bump();
                if !assignment_target(&left) {
                    return Err(early(left.span, "invalid assignment target"));
                }
                let right = self.expression(2)?;
                let span = Span::new(left.span.start, right.span.end);
                let kind = if let Some(op) = assignment_op {
                    ExprKind::CompoundAssign(op, Box::new(left), Box::new(right))
                } else {
                    ExprKind::Assign(Box::new(left), Box::new(right))
                };
                left = self.make_expr(kind, span)?;
                continue;
            }
            if minimum <= 3 && self.eat("?") {
                let yes = self.expression_with_in(2, true)?;
                self.expect(":")?;
                let no = self.expression(2)?;
                let span = Span::new(left.span.start, no.span.end);
                left = self.make_expr(
                    ExprKind::Conditional(Box::new(left), Box::new(yes), Box::new(no)),
                    span,
                )?;
                continue;
            }
            let operator = if self.allow_in && self.at("in") {
                Some((BinaryOp::In, 10))
            } else if self.at("instanceof") {
                Some((BinaryOp::Instanceof, 10))
            } else {
                binary(&self.current().kind)
            };
            let Some((op, power)) = operator else {
                break;
            };
            if power < minimum {
                break;
            }
            self.bump();
            if op == BinaryOp::Exponentiate && matches!(left.kind, ExprKind::Unary(..)) {
                return Err(self.error("unparenthesized unary expression before exponentiation"));
            }
            let right = self.expression(if op == BinaryOp::Exponentiate {
                power
            } else {
                power + 1
            })?;
            if forbidden_nullish_mix(op, &left) || forbidden_nullish_mix(op, &right) {
                return Err(self.error("parentheses required when mixing ?? with && or ||"));
            }
            let span = Span::new(left.span.start, right.span.end);
            left = self.make_expr(ExprKind::Binary(op, Box::new(left), Box::new(right)), span)?;
        }
        Ok(left)
    }
    fn at_async_function(&self) -> bool {
        self.at("async")
            && self.tokens.get(self.index + 1).is_some_and(|token| {
                !token.newline
                    && !token.escaped
                    && matches!(&token.kind, Kind::Word(name) if name == "function")
            })
    }

    fn prefix(&mut self) -> Result<Expr, Diagnostic> {
        if self.at_async_function() {
            return Err(self.unsupported("async functions are not implemented"));
        }
        if self.at("function") {
            return self.function_expression();
        }
        if self.at("new") {
            return self.new_expression();
        }
        let token = self.bump();
        let span = token.span;
        if matches!(token.kind, Kind::Punct("++" | "--")) {
            let argument = self.expression(15)?;
            if !assignment_target(&argument) {
                return Err(early(argument.span, "invalid update target"));
            }
            let op = if token.kind == Kind::Punct("++") {
                UpdateOp::Increment
            } else {
                UpdateOp::Decrement
            };
            let span = Span::new(span.start, argument.span.end);
            return self.make_expr(
                ExprKind::Update {
                    op,
                    argument: Box::new(argument),
                    prefix: true,
                },
                span,
            );
        }
        let op = match &token.kind {
            Kind::Punct("+") => Some(UnaryOp::Plus),
            Kind::Punct("-") => Some(UnaryOp::Minus),
            Kind::Punct("!") => Some(UnaryOp::Not),
            Kind::Punct("~") => Some(UnaryOp::BitNot),
            Kind::Word(s) if !token.escaped && s == "void" => Some(UnaryOp::Void),
            Kind::Word(s) if !token.escaped && s == "typeof" => Some(UnaryOp::Typeof),
            Kind::Word(s) if !token.escaped && s == "delete" => Some(UnaryOp::Delete),
            _ => None,
        };
        if let Some(op) = op {
            let right = self.expression(15)?;
            let span = Span::new(span.start, right.span.end);
            return self.make_expr(ExprKind::Unary(op, Box::new(right)), span);
        }
        match token.kind {
            Kind::Template {
                element,
                tail,
                continuation: false,
            } => self.template_literal(element, tail, span, None),
            Kind::Literal(lit) => self.make_expr(ExprKind::Literal(lit), span),
            Kind::Word(name) if !token.escaped && name == "this" => {
                self.make_expr(ExprKind::This, span)
            }
            Kind::Punct("{") => self.object_literal(span.start),
            Kind::Punct("[") => self.array_literal(span.start),
            Kind::Word(name) if !reserved(&name) => {
                self.make_expr(ExprKind::Identifier(name), span)
            }
            Kind::Punct("(") => {
                let expr = self.expression_with_in(1, true)?;
                self.expect(")")?;
                let span = Span::new(span.start, self.tokens[self.index - 1].span.end);
                self.make_expr(ExprKind::Parenthesized(Box::new(expr)), span)
            }
            Kind::Word(_) if token.escaped => Err(Diagnostic::new(
                DiagnosticKind::Syntax,
                span,
                "reserved word cannot be an identifier",
            )),
            Kind::Word(name) if matches!(name.as_str(), "catch" | "finally") => Err(
                Diagnostic::new(DiagnosticKind::Syntax, span, "unexpected catch or finally"),
            ),
            Kind::Word(name) if name == "instanceof" => {
                Err(early(span, "unexpected binary operator"))
            }
            Kind::Word(_) | Kind::Punct("/") => Err(Diagnostic::new(
                DiagnosticKind::Unsupported,
                span,
                "expression form is not implemented",
            )),
            _ => Err(Diagnostic::new(
                DiagnosticKind::Syntax,
                span,
                "expected an expression",
            )),
        }
    }
}

fn assignment_name(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Identifier(name) => Some(name),
        ExprKind::Parenthesized(inner) => assignment_name(inner),
        _ => None,
    }
}

fn assignment_target(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Identifier(_) | ExprKind::Member(..) => true,
        ExprKind::Parenthesized(inner) => assignment_target(inner),
        _ => false,
    }
}

fn member_base(expr: &Expr) -> bool {
    matches!(
        expr.kind,
        ExprKind::Identifier(_)
            | ExprKind::This
            | ExprKind::NewTarget
            | ExprKind::Literal(_)
            | ExprKind::Object(_)
            | ExprKind::Array(_)
            | ExprKind::Template { .. }
            | ExprKind::TaggedTemplate { .. }
            | ExprKind::OptionalChain { .. }
            | ExprKind::Parenthesized(_)
            | ExprKind::Member(..)
            | ExprKind::Call { .. }
            | ExprKind::New {
                arguments: Some(_),
                ..
            }
            | ExprKind::Function(_)
    )
}

fn binary(kind: &Kind) -> Option<(BinaryOp, u8)> {
    use BinaryOp::*;
    let Kind::Punct(p) = kind else {
        return None;
    };
    Some(match *p {
        "," => (Comma, 1),
        "??" => (Nullish, 4),
        "||" => (Or, 4),
        "&&" => (And, 5),
        "|" => (BitOr, 6),
        "^" => (BitXor, 7),
        "&" => (BitAnd, 8),
        "==" => (Equal, 9),
        "!=" => (NotEqual, 9),
        "===" => (StrictEqual, 9),
        "!==" => (StrictNotEqual, 9),
        "<" => (Less, 10),
        "<=" => (LessEqual, 10),
        ">" => (Greater, 10),
        ">=" => (GreaterEqual, 10),
        "<<" => (LeftShift, 11),
        ">>" => (RightShift, 11),
        ">>>" => (UnsignedRightShift, 11),
        "+" => (Add, 12),
        "-" => (Subtract, 12),
        "*" => (Multiply, 13),
        "/" => (Divide, 13),
        "%" => (Remainder, 13),
        "**" => (Exponentiate, 14),
        _ => return None,
    })
}

fn compound_assignment(kind: &Kind) -> Option<BinaryOp> {
    use BinaryOp::*;
    let Kind::Punct(p) = kind else {
        return None;
    };
    Some(match *p {
        "+=" => Add,
        "-=" => Subtract,
        "*=" => Multiply,
        "/=" => Divide,
        "%=" => Remainder,
        "**=" => Exponentiate,
        "<<=" => LeftShift,
        ">>=" => RightShift,
        ">>>=" => UnsignedRightShift,
        "&=" => BitAnd,
        "^=" => BitXor,
        "|=" => BitOr,
        "&&=" => And,
        "||=" => Or,
        "??=" => Nullish,
        _ => return None,
    })
}

fn forbidden_nullish_mix(op: BinaryOp, expr: &Expr) -> bool {
    let ExprKind::Binary(other, ..) = expr.kind else {
        return false;
    };
    (op == BinaryOp::Nullish && matches!(other, BinaryOp::And | BinaryOp::Or))
        || (other == BinaryOp::Nullish && matches!(op, BinaryOp::And | BinaryOp::Or))
}

// ReservedWord StringValues other than the context-sensitive yield and await.
// https://262.ecma-international.org/17.0/#sec-identifiers-static-semantics-early-errors
fn reserved(name: &str) -> bool {
    matches!(
        name,
        "null"
            | "true"
            | "false"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "new"
            | "return"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
    )
}
fn strict_reserved(name: &str) -> bool {
    matches!(
        name,
        "implements"
            | "interface"
            | "let"
            | "package"
            | "private"
            | "protected"
            | "public"
            | "static"
            | "yield"
    )
}
fn early(span: Span, message: &str) -> Diagnostic {
    Diagnostic::new(DiagnosticKind::Syntax, span, message)
}

fn validate_binding_names<'a>(
    bindings: &'a [BindingElement],
    strict: bool,
    names: &mut BTreeSet<&'a str>,
) -> Result<(), Diagnostic> {
    for binding in bindings {
        for (name, span) in binding.pattern.bound_names() {
            if !names.insert(name) {
                return Err(early(span, "duplicate lexical binding"));
            }
        }
        binding::validate_pattern(&binding.pattern, strict)?;
    }
    Ok(())
}

fn validate_binding_name(name: &str, span: Span, strict: bool) -> Result<(), Diagnostic> {
    if strict && (strict_reserved(name) || matches!(name, "eval" | "arguments")) {
        return Err(early(span, "invalid binding in strict mode"));
    }
    Ok(())
}

fn validate_var_bindings(bindings: &[BindingElement], strict: bool) -> Result<(), Diagnostic> {
    for binding in bindings {
        binding::validate_element(binding, strict)?;
    }
    Ok(())
}

// Function bodies start with a fresh context; class static blocks will too.
#[derive(Clone, Copy, Default)]
struct ControlContext {
    in_iteration: bool,
    in_breakable: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum ScopeKind {
    Variable,
    Block,
}

// LexicallyDeclaredNames of a block includes its function declarations (8.2.6).
fn block_lexical_names(statement: &Statement) -> impl Iterator<Item = (&str, Span)> {
    let bindings = match &statement.kind {
        StatementKind::Lexical { bindings, .. } => bindings.as_slice(),
        _ => &[],
    };
    let function_name = match &statement.kind {
        StatementKind::Function(function) => function.name.as_ref(),
        _ => None,
    };
    bindings
        .iter()
        .flat_map(|b| b.pattern.bound_names())
        .chain(function_name.map(|name| (name.name.as_str(), name.span)))
}

fn validate_scope<'a>(
    statements: impl IntoIterator<Item = &'a Statement>,
    strict: bool,
    control: ControlContext,
    labels: &mut Vec<(&'a str, bool)>,
    scope_kind: ScopeKind,
) -> Result<(), Diagnostic> {
    let mut names = BTreeSet::new();
    let mut var_names = BTreeSet::new();
    let mut declarations = Vec::new();
    for statement in statements {
        if scope_kind == ScopeKind::Block || !matches!(statement.kind, StatementKind::Function(_)) {
            for (name, span) in block_lexical_names(statement) {
                if !names.insert(name) {
                    return Err(early(span, "duplicate lexical binding"));
                }
                validate_binding_name(name, span, strict)?;
                if var_names.contains(name) {
                    return Err(early(span, "lexical declaration conflicts with var"));
                }
            }
        } else if let StatementKind::Function(function) = &statement.kind {
            // 8.2.8: direct functions are var-scoped only in Scripts/function bodies.
            let name = function.name.as_ref().expect("named declaration");
            if names.contains(name.name.as_str()) {
                return Err(early(
                    name.span,
                    "function declaration conflicts with lexical binding",
                ));
            }
            var_names.insert(name.name.as_str());
        }
        // ECMA-262 14.2.1, 14.12.1, 16.1.1: vars in nested statements
        // cannot conflict with lexical names of an enclosing statement list.
        declarations.clear();
        statement.collect_var_declarations(&mut declarations);
        for binding in &declarations {
            if names.contains(binding.name) {
                return Err(early(
                    binding.span,
                    "var declaration conflicts with lexical binding",
                ));
            }
            var_names.insert(binding.name);
        }
        validate_statement(statement, strict, control, labels)?;
    }
    Ok(())
}
fn validate_statement<'a>(
    statement: &'a Statement,
    strict: bool,
    control: ControlContext,
    labels: &mut Vec<(&'a str, bool)>,
) -> Result<(), Diagnostic> {
    match &statement.kind {
        StatementKind::Function(function) => function::validate_function(function, strict)?,
        StatementKind::Var(bindings) => validate_var_bindings(bindings, strict)?,
        StatementKind::Expression(expr)
        | StatementKind::Throw(expr)
        | StatementKind::Return(Some(expr)) => validate_expr(expr, strict)?,
        StatementKind::Lexical { bindings, .. } => {
            for binding in bindings {
                binding::validate_element(binding, strict)?;
            }
        }
        StatementKind::Block(body) => {
            validate_scope(body, strict, control, labels, ScopeKind::Block)?
        }
        StatementKind::With { object, body } => {
            // ECMA-262 14.11.1: this grammar is available only in non-strict code.
            if strict {
                return Err(early(
                    statement.span,
                    "with statements are not allowed in strict mode",
                ));
            }
            validate_expr(object, strict)?;
            validate_statement(body, strict, control, labels)?;
        }
        StatementKind::Try {
            body,
            handler,
            finalizer,
        } => {
            validate_statement(body, strict, control, labels)?;
            if let Some(handler) = handler {
                if let Some(parameter) = &handler.parameter {
                    binding::validate_pattern(parameter, strict)?;
                    let names = parameter.bound_names();
                    let mut seen = BTreeSet::new();
                    for &(name, span) in &names {
                        if !seen.insert(name) {
                            return Err(early(span, "duplicate catch binding"));
                        }
                    }
                    let StatementKind::Block(statements) = &handler.body.kind else {
                        unreachable!("catch requires a block");
                    };
                    // ECMA-262 14.15.1. The Annex B exception allowing var to
                    // redeclare a simple catch parameter is not enabled by this host.
                    for statement in statements {
                        for (name, span) in block_lexical_names(statement) {
                            if seen.contains(name) {
                                return Err(early(
                                    span,
                                    "catch parameter conflicts with lexical declaration",
                                ));
                            }
                        }
                    }
                    let mut declarations = Vec::new();
                    handler.body.collect_var_declarations(&mut declarations);
                    for binding in declarations {
                        if seen.contains(binding.name) {
                            return Err(early(
                                binding.span,
                                "catch parameter conflicts with var declaration",
                            ));
                        }
                    }
                }
                validate_statement(&handler.body, strict, control, labels)?;
            }
            if let Some(finalizer) = finalizer {
                validate_statement(finalizer, strict, control, labels)?;
            }
        }
        StatementKind::While { test, body } | StatementKind::DoWhile { test, body } => {
            validate_expr(test, strict)?;
            validate_statement(
                body,
                strict,
                ControlContext {
                    in_iteration: true,
                    in_breakable: true,
                },
                labels,
            )?;
        }
        StatementKind::For {
            initializer,
            test,
            update,
            body,
        } => {
            if let Some(initializer) = initializer {
                match initializer {
                    ForInitializer::Expression(expr) => validate_expr(expr, strict)?,
                    ForInitializer::Var(bindings) => validate_var_bindings(bindings, strict)?,
                    ForInitializer::Lexical { bindings, .. } => {
                        let mut names = BTreeSet::new();
                        validate_binding_names(bindings, strict, &mut names)?;
                        let mut declarations = Vec::new();
                        body.collect_var_declarations(&mut declarations);
                        // ECMA-262 14.7.4.1: lexical header names cannot be vars in the body.
                        for binding in declarations {
                            if names.contains(binding.name) {
                                return Err(early(
                                    binding.span,
                                    "var declaration conflicts with lexical for binding",
                                ));
                            }
                        }
                        for binding in bindings {
                            if let Some(expr) = &binding.initializer {
                                validate_expr(expr, strict)?;
                            }
                        }
                    }
                }
            }
            for expression in [test, update].into_iter().flatten() {
                validate_expr(expression, strict)?;
            }
            validate_statement(
                body,
                strict,
                ControlContext {
                    in_iteration: true,
                    in_breakable: true,
                },
                labels,
            )?;
        }
        StatementKind::ForOf {
            binding,
            iterable,
            body,
        }
        | StatementKind::ForIn {
            binding,
            object: iterable,
            body,
        } => {
            match binding {
                ForBinding::Assignment(target) => {
                    if strict
                        && assignment_name(target).is_some_and(|name| {
                            strict_reserved(name) || matches!(name, "eval" | "arguments")
                        })
                    {
                        return Err(early(target.span, "invalid assignment in strict mode"));
                    }
                    validate_expr(target, strict)?;
                }
                ForBinding::Pattern(pattern) => {
                    assignment_pattern::validate_pattern(pattern, strict)?
                }
                ForBinding::Var(binding) => {
                    binding::validate_pattern(&binding.pattern, strict)?;
                    if let Some(initializer) = &binding.initializer {
                        // Annex B's initialized for-in extension is not enabled. Keep its
                        // non-strict extension separate from core early errors.
                        return Err(if strict {
                            early(
                                initializer.span,
                                "for-in var initializer is forbidden in strict mode",
                            )
                        } else {
                            Diagnostic::new(
                                DiagnosticKind::Unsupported,
                                initializer.span,
                                "initialized for-in var declarations require Annex B",
                            )
                        });
                    }
                }
                ForBinding::Lexical { binding, .. } => {
                    binding::validate_pattern(binding, strict)?;
                    let mut names = BTreeSet::new();
                    for (name, span) in binding.bound_names() {
                        if !names.insert(name) {
                            return Err(early(span, "duplicate lexical binding"));
                        }
                    }
                    let mut declarations = Vec::new();
                    body.collect_var_declarations(&mut declarations);
                    for declaration in declarations {
                        if names.contains(declaration.name) {
                            return Err(early(
                                declaration.span,
                                "var declaration conflicts with lexical for binding",
                            ));
                        }
                    }
                }
            }
            validate_expr(iterable, strict)?;
            validate_statement(
                body,
                strict,
                ControlContext {
                    in_iteration: true,
                    in_breakable: true,
                },
                labels,
            )?;
        }
        StatementKind::Switch {
            discriminant,
            clauses,
        } => {
            validate_expr(discriminant, strict)?;
            for clause in clauses {
                if let Some(test) = &clause.test {
                    validate_expr(test, strict)?;
                }
            }
            // ECMA-262 14.12.1: all clauses form one scope for duplicate names.
            // Switch permits break while preserving the enclosing loop context.
            validate_scope(
                clauses.iter().flat_map(|clause| &clause.statements),
                strict,
                ControlContext {
                    in_breakable: true,
                    ..control
                },
                labels,
                ScopeKind::Block,
            )?;
        }
        // ECMA-262 14.8.1/14.9.1 distinguish iterations from breakable statements.
        StatementKind::Break(None) if !control.in_breakable => {
            return Err(early(
                statement.span,
                "break requires an enclosing loop or switch",
            ));
        }
        StatementKind::Continue(None) if !control.in_iteration => {
            return Err(early(statement.span, "continue requires an enclosing loop"));
        }
        StatementKind::If {
            test,
            consequent,
            alternate,
        } => {
            validate_expr(test, strict)?;
            validate_statement(consequent, strict, control, labels)?;
            if let Some(alternate) = alternate {
                validate_statement(alternate, strict, control, labels)?;
            }
        }
        StatementKind::Labelled { label, body } => {
            validate_label(label, strict)?;
            // ECMA-262 8.3.1: duplicate labels are forbidden only while active.
            if labels.iter().any(|(name, _)| *name == label.name) {
                return Err(early(label.span, "duplicate label"));
            }
            labels.push((&label.name, labels_iteration(body)));
            let result = validate_statement(body, strict, control, labels);
            labels.pop();
            result?;
        }
        StatementKind::Break(Some(target)) | StatementKind::Continue(Some(target)) => {
            validate_label(target, strict)?;
            // ECMA-262 8.3.2/8.3.3: continue needs an iteration target, not just
            // a label enclosing an iteration somewhere in its subtree.
            let Some((_, is_iteration)) = labels.iter().find(|(name, _)| *name == target.name)
            else {
                return Err(early(target.span, "undefined label"));
            };
            if matches!(statement.kind, StatementKind::Continue(_)) && !is_iteration {
                return Err(early(
                    target.span,
                    "continue target is not an iteration label",
                ));
            }
        }
        StatementKind::Empty
        | StatementKind::Debugger
        | StatementKind::Break(None)
        | StatementKind::Continue(None)
        | StatementKind::Return(None) => {}
    }
    Ok(())
}
fn validate_label(label: &Label, strict: bool) -> Result<(), Diagnostic> {
    if strict && strict_reserved(&label.name) {
        return Err(early(label.span, "reserved label in strict mode"));
    }
    Ok(())
}

fn labels_iteration(mut statement: &Statement) -> bool {
    while let StatementKind::Labelled { body, .. } = &statement.kind {
        statement = body;
    }
    matches!(
        statement.kind,
        StatementKind::While { .. }
            | StatementKind::DoWhile { .. }
            | StatementKind::For { .. }
            | StatementKind::ForOf { .. }
            | StatementKind::ForIn { .. }
    )
}

fn validate_expr(expr: &Expr, strict: bool) -> Result<(), Diagnostic> {
    match &expr.kind {
        ExprKind::Function(function) => function::validate_function(function, strict)?,
        ExprKind::Arrow {
            parameters, body, ..
        } => {
            let own_strict = matches!(body, ArrowBody::Block(body) if body.is_strict());
            let names =
                function::validate_parameters(parameters, strict, own_strict, true, expr.span)?;
            let strict = strict || own_strict;
            match body {
                ArrowBody::Expression(body) => validate_expr(body, strict)?,
                ArrowBody::Block(body) => function::validate_body(body, strict, &names)?,
            }
        }

        ExprKind::Array(elements) => {
            for element in elements.iter().filter_map(ArrayElement::expression) {
                validate_expr(element, strict)?;
            }
        }
        ExprKind::Object(properties) => {
            for element in properties {
                let property = match element {
                    ObjectElement::Spread(expression) => {
                        validate_expr(expression, strict)?;
                        continue;
                    }
                    ObjectElement::Property(property) => property,
                };
                if let PropertyName::Computed(key) = &property.name {
                    validate_expr(key, strict)?;
                }
                if matches!(
                    property.kind,
                    PropertyKind::Method | PropertyKind::Getter | PropertyKind::Setter
                ) {
                    let ExprKind::Function(function) = &property.value.kind else {
                        unreachable!("method syntax");
                    };
                    function::validate_method(function, strict)?;
                } else {
                    validate_expr(&property.value, strict)?;
                }
            }
        }
        ExprKind::OptionalChain { base, steps } => {
            validate_expr(base, strict)?;
            for step in steps {
                match &step.kind {
                    ChainStepKind::Property(PropertyName::Computed(expression)) => {
                        validate_expr(expression, strict)?
                    }
                    ChainStepKind::Property(PropertyName::Literal(_)) => {}
                    ChainStepKind::Call(arguments) => {
                        for argument in arguments {
                            validate_expr(argument.expression(), strict)?;
                        }
                    }
                }
            }
        }
        ExprKind::TaggedTemplate {
            tag, substitutions, ..
        } => {
            validate_expr(tag, strict)?;
            for expression in substitutions {
                validate_expr(expression, strict)?;
            }
        }
        ExprKind::Template { substitutions, .. } => {
            for expression in substitutions {
                validate_expr(expression, strict)?;
            }
        }
        ExprKind::Identifier(name) if strict && strict_reserved(name) => {
            return Err(early(expr.span, "reserved identifier in strict mode"));
        }
        ExprKind::Assign(target, value) | ExprKind::CompoundAssign(_, target, value) => {
            if strict
                && assignment_name(target).is_some_and(|name| {
                    strict_reserved(name) || matches!(name, "eval" | "arguments")
                })
            {
                return Err(early(expr.span, "invalid assignment in strict mode"));
            }
            validate_expr(target, strict)?;
            validate_expr(value, strict)?;
        }
        ExprKind::DestructuringAssign { pattern, value } => {
            assignment_pattern::validate_pattern(pattern, strict)?;
            validate_expr(value, strict)?;
        }
        ExprKind::Member(base, name) => {
            validate_expr(base, strict)?;
            if let PropertyName::Computed(key) = name {
                validate_expr(key, strict)?;
            }
        }
        ExprKind::Call { callee, arguments } => {
            validate_expr(callee, strict)?;
            for argument in arguments {
                validate_expr(argument.expression(), strict)?;
            }
        }
        ExprKind::New { callee, arguments } => {
            validate_expr(callee, strict)?;
            for argument in arguments.iter().flatten() {
                validate_expr(argument.expression(), strict)?;
            }
        }
        ExprKind::Update { argument, .. } => {
            if strict
                && assignment_name(argument).is_some_and(|name| {
                    strict_reserved(name) || matches!(name, "eval" | "arguments")
                })
            {
                return Err(early(argument.span, "invalid update target in strict mode"));
            }
            validate_expr(argument, strict)?;
        }
        ExprKind::Unary(UnaryOp::Delete, e) if strict && assignment_name(e).is_some() => {
            // ECMA-262 13.5.1.1 also rejects parenthesized identifier references.
            return Err(early(e.span, "cannot delete an identifier in strict mode"));
        }
        ExprKind::Unary(_, e) | ExprKind::Parenthesized(e) => validate_expr(e, strict)?,
        ExprKind::Binary(_, a, b) => {
            validate_expr(a, strict)?;
            validate_expr(b, strict)?;
        }
        ExprKind::Conditional(a, b, c) => {
            validate_expr(a, strict)?;
            validate_expr(b, strict)?;
            validate_expr(c, strict)?;
        }
        _ => {}
    }
    Ok(())
}
