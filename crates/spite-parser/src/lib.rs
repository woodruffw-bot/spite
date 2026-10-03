//! Parsing and early-error validation for the implemented ECMAScript subset.

pub mod ast;
mod lexer;

use ast::*;
use lexer::{Kind, Lexer, Token};
use spite_core::{Diagnostic, DiagnosticKind, Span};
use std::collections::BTreeSet;

/// Maximum source size accepted by this parser, in UTF-8 bytes.
pub const MAX_SOURCE_BYTES: usize = 1024 * 1024;
/// Maximum recursive syntax depth accepted by this parser.
pub const MAX_DEPTH: usize = 64;

/// Parses a Script and validates implemented early errors before returning it.
///
/// This is not yet a complete ECMAScript parser. Unsupported features produce
/// [`DiagnosticKind::Unsupported`] where they can be recognized.
pub fn parse_script(source: &str) -> Result<Script, Diagnostic> {
    if source.len() > MAX_SOURCE_BYTES {
        return Err(Diagnostic::new(
            DiagnosticKind::Limit,
            Span::new(0, source.len()),
            "source size limit exceeded",
        ));
    }
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
    let mut parser = Parser {
        tokens,
        index: 0,
        depth: 0,
    };
    let mut statements = Vec::new();
    while parser.current().kind != Kind::Eof {
        statements.push(parser.statement(true)?);
    }
    let strict = statements
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
        });
    validate_scope(&statements, strict)?;
    Ok(Script { statements, strict })
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
    depth: usize,
}

impl Parser {
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
        } else if matches!(
            self.current().kind,
            Kind::Punct(
                "(" | "["
                    | "."
                    | "?."
                    | "++"
                    | "--"
                    | "+="
                    | "-="
                    | "*="
                    | "/="
                    | "%="
                    | "**="
                    | "&&="
                    | "||="
                    | "??="
                    | "&="
                    | "|="
                    | "^="
                    | "<<="
                    | ">>="
                    | ">>>="
            )
        ) {
            Err(self.unsupported(
                "calls, properties, updates, and compound assignment are not implemented",
            ))
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
        let lexical = self.at("const")
            || (self.at("let")
                && self
                    .tokens
                    .get(self.index + 1)
                    .is_some_and(|t| matches!(t.kind, Kind::Word(_) | Kind::Punct("[" | "{"))));
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
        } else if lexical {
            if !allow_declaration {
                return Err(self.error("lexical declaration requires a statement list"));
            }
            let mutable = self.eat("let");
            if !mutable {
                self.expect("const")?;
            }
            let mut bindings = Vec::new();
            loop {
                if self.at("[") || self.at("{") {
                    return Err(self.unsupported("binding patterns are not implemented"));
                }
                let token = self.bump();
                let Kind::Word(name) = token.kind else {
                    return Err(Diagnostic::new(
                        DiagnosticKind::Syntax,
                        token.span,
                        "expected binding identifier",
                    ));
                };
                if reserved(&name) || name == "let" {
                    return Err(Diagnostic::new(
                        DiagnosticKind::Syntax,
                        token.span,
                        "invalid lexical binding identifier",
                    ));
                }
                let initializer = if self.eat("=") {
                    Some(self.expression(2)?)
                } else {
                    None
                };
                if !mutable && initializer.is_none() {
                    return Err(self.error("const requires an initializer"));
                }
                bindings.push(Binding {
                    name,
                    span: token.span,
                    initializer,
                });
                if !self.eat(",") {
                    break;
                }
            }
            self.semicolon()?;
            StatementKind::Lexical { mutable, bindings }
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
        } else if self.eat("while") {
            self.expect("(")?;
            let test = self.expression(1)?;
            self.expect(")")?;
            let body = Box::new(self.statement(false)?);
            StatementKind::While { test, body }
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
        } else if self.eat("throw") {
            if self.current().newline {
                return Err(self.error("line terminator after throw"));
            }
            let expr = self.expression(1)?;
            self.semicolon()?;
            StatementKind::Throw(expr)
        } else {
            if let Kind::Word(word) = &self.current().kind {
                if !self.current().escaped
                    && matches!(
                        word.as_str(),
                        "var"
                            | "function"
                            | "class"
                            | "return"
                            | "for"
                            | "switch"
                            | "try"
                            | "with"
                            | "break"
                            | "continue"
                            | "debugger"
                            | "import"
                            | "export"
                    )
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

    fn make_expr(&self, kind: ExprKind, span: Span) -> Result<Expr, Diagnostic> {
        let depth = 1 + match &kind {
            ExprKind::Unary(_, e) | ExprKind::Parenthesized(e) | ExprKind::Assign(_, e) => e.depth,
            ExprKind::Binary(_, a, b) => a.depth.max(b.depth),
            ExprKind::Conditional(a, b, c) => a.depth.max(b.depth).max(c.depth),
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
    fn expression_inner(&mut self, minimum: u8) -> Result<Expr, Diagnostic> {
        let mut left = self.prefix()?;
        loop {
            if minimum <= 2 && self.eat("=") {
                let Some(name) = assignment_name(&left) else {
                    return Err(self.error("invalid assignment target"));
                };
                let name = name.to_owned();
                let right = self.expression(2)?;
                let span = Span::new(left.span.start, right.span.end);
                left = self.make_expr(ExprKind::Assign(name, Box::new(right)), span)?;
                continue;
            }
            if minimum <= 3 && self.eat("?") {
                let yes = self.expression(2)?;
                self.expect(":")?;
                let no = self.expression(2)?;
                let span = Span::new(left.span.start, no.span.end);
                left = self.make_expr(
                    ExprKind::Conditional(Box::new(left), Box::new(yes), Box::new(no)),
                    span,
                )?;
                continue;
            }
            let Some((op, power)) = binary(&self.current().kind) else {
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
        // These tokens continue an expression even across a newline. ASI cannot
        // make a call or property access into a separate expression statement.
        if matches!(self.current().kind, Kind::Punct("(" | "[" | "." | "?.")) {
            return Err(self.unsupported("calls and property access are not implemented"));
        }
        Ok(left)
    }
    fn prefix(&mut self) -> Result<Expr, Diagnostic> {
        let token = self.bump();
        let span = token.span;
        let op = match &token.kind {
            Kind::Punct("+") => Some(UnaryOp::Plus),
            Kind::Punct("-") => Some(UnaryOp::Minus),
            Kind::Punct("!") => Some(UnaryOp::Not),
            Kind::Punct("~") => Some(UnaryOp::BitNot),
            Kind::Word(s) if !token.escaped && s == "void" => Some(UnaryOp::Void),
            Kind::Word(s) if !token.escaped && s == "typeof" => Some(UnaryOp::Typeof),
            _ => None,
        };
        if let Some(op) = op {
            let right = self.expression(15)?;
            let span = Span::new(span.start, right.span.end);
            return self.make_expr(ExprKind::Unary(op, Box::new(right)), span);
        }
        match token.kind {
            Kind::Literal(lit) => self.make_expr(ExprKind::Literal(lit), span),
            Kind::Word(name) if !reserved(&name) => {
                self.make_expr(ExprKind::Identifier(name), span)
            }
            Kind::Punct("(") => {
                let expr = self.expression(1)?;
                self.expect(")")?;
                let span = Span::new(span.start, self.tokens[self.index - 1].span.end);
                self.make_expr(ExprKind::Parenthesized(Box::new(expr)), span)
            }
            Kind::Word(_) if token.escaped => Err(Diagnostic::new(
                DiagnosticKind::Syntax,
                span,
                "reserved word cannot be an identifier",
            )),
            Kind::Word(_) | Kind::Punct("[" | "{" | "/" | "++" | "--") => Err(Diagnostic::new(
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

fn validate_scope(statements: &[Statement], strict: bool) -> Result<(), Diagnostic> {
    let mut names = BTreeSet::new();
    for statement in statements {
        if let StatementKind::Lexical { bindings, .. } = &statement.kind {
            for binding in bindings {
                if !names.insert(&binding.name) {
                    return Err(early(binding.span, "duplicate lexical binding"));
                }
                if strict
                    && (strict_reserved(&binding.name)
                        || matches!(binding.name.as_str(), "eval" | "arguments"))
                {
                    return Err(early(binding.span, "invalid binding in strict mode"));
                }
            }
        }
        validate_statement(statement, strict)?;
    }
    Ok(())
}
fn validate_statement(statement: &Statement, strict: bool) -> Result<(), Diagnostic> {
    match &statement.kind {
        StatementKind::Expression(expr) | StatementKind::Throw(expr) => {
            validate_expr(expr, strict)?
        }
        StatementKind::Lexical { bindings, .. } => {
            for binding in bindings {
                if let Some(expr) = &binding.initializer {
                    validate_expr(expr, strict)?;
                }
            }
        }
        StatementKind::Block(body) => validate_scope(body, strict)?,
        StatementKind::While { test, body } | StatementKind::DoWhile { test, body } => {
            validate_expr(test, strict)?;
            validate_statement(body, strict)?;
        }
        StatementKind::If {
            test,
            consequent,
            alternate,
        } => {
            validate_expr(test, strict)?;
            validate_statement(consequent, strict)?;
            if let Some(alternate) = alternate {
                validate_statement(alternate, strict)?;
            }
        }
        StatementKind::Empty => {}
    }
    Ok(())
}
fn validate_expr(expr: &Expr, strict: bool) -> Result<(), Diagnostic> {
    match &expr.kind {
        ExprKind::Identifier(name) if strict && strict_reserved(name) => {
            return Err(early(expr.span, "reserved identifier in strict mode"));
        }
        ExprKind::Assign(name, value) => {
            if strict && (strict_reserved(name) || matches!(name.as_str(), "eval" | "arguments")) {
                return Err(early(expr.span, "invalid assignment in strict mode"));
            }
            validate_expr(value, strict)?;
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
