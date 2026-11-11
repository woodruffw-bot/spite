//! Prefix operators and primary-expression parsing use separate native frames.

use super::{
    AwaitContext, Diagnostic, DiagnosticKind, Expr, ExprKind, Kind, Literal, Parser, Span, Token,
    UnaryOp, UpdateOp, assignment_target, early, reserved,
};

impl Parser {
    #[inline(never)]
    pub(super) fn prefix(&mut self) -> Result<Expr, Diagnostic> {
        if matches!(
            self.current().kind,
            Kind::RegExp { .. } | Kind::Punct("/" | "/=")
        ) {
            return self.regexp_primary();
        }
        if self.at("import")
            && !self.current().escaped
            && self
                .token_at(self.index + 1)
                .is_some_and(|token| token.kind == Kind::Punct("("))
        {
            return self.import_call();
        }
        if self.at("class") {
            let class = self.class_definition(false)?;
            let span = class.source.span;
            return self.make_expr(ExprKind::Class(class), span);
        }
        if self.at("await") && matches!(self.await_context, AwaitContext::Body) {
            return self.await_expression();
        }
        if self.at("function") || self.at_async_function() {
            return self.function_expression();
        }
        if self.at("new") {
            return self.new_expression();
        }
        self.prefix_operator_or_primary()
    }

    #[inline(never)]
    fn regexp_primary(&mut self) -> Result<Expr, Diagnostic> {
        let Some(diagnostic) = self.regexp_diagnostic() else {
            let Token {
                kind: Kind::RegExp { body, flags },
                span,
                ..
            } = self.bump()
            else {
                unreachable!("validated RegExp token");
            };
            return self.make_expr(ExprKind::Literal(Literal::RegExp { body, flags }), span);
        };
        if !self.probing_cover
            || !matches!(self.current().kind, Kind::RegExp { .. })
            || !matches!(
                diagnostic.kind,
                DiagnosticKind::Syntax | DiagnosticKind::Unsupported
            )
        {
            return Err(diagnostic);
        }
        // A cover probe only needs the scanned literal boundary. This
        // temporary value cannot escape: selecting the head reports the
        // deferred diagnostic before exposing its AST.
        if self.probe_error.is_none() {
            self.probe_error = Some(diagnostic);
        }
        let token = self.bump();
        self.make_expr(ExprKind::Literal(Literal::Null), token.span)
    }

    #[inline(never)]
    fn await_expression(&mut self) -> Result<Expr, Diagnostic> {
        let start = self.bump().span.start;
        if !self.current().newline && self.at_using_declaration(false) {
            return Err(self.unsupported("await using declarations are not implemented"));
        }
        // AwaitExpression consumes UnaryExpression, before multiplication,
        // conditionals and assignment (13.3.9, 13.5, 13.6).
        let argument = self.expression(15)?;
        let span = Span::new(start, argument.span.end);
        self.make_expr(ExprKind::Await(Box::new(argument)), span)
    }

    #[inline(never)]
    fn prefix_operator_or_primary(&mut self) -> Result<Expr, Diagnostic> {
        let token = self.bump();
        let span = token.span;
        if matches!(token.kind, Kind::Punct("++" | "--")) {
            return self.prefix_update(token);
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
            return self.unary_expression(span, op);
        }
        self.primary_token(token)
    }

    #[inline(never)]
    fn prefix_update(&mut self, token: Token) -> Result<Expr, Diagnostic> {
        let span = token.span;
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
        self.make_expr(
            ExprKind::Update {
                op,
                argument: Box::new(argument),
                prefix: true,
            },
            span,
        )
    }

    #[inline(never)]
    fn unary_expression(&mut self, span: Span, op: UnaryOp) -> Result<Expr, Diagnostic> {
        let right = self.expression(15)?;
        let span = Span::new(span.start, right.span.end);
        self.make_expr(ExprKind::Unary(op, Box::new(right)), span)
    }

    #[inline(never)]
    fn primary_token(&mut self, token: Token) -> Result<Expr, Diagnostic> {
        let span = token.span;
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
            Kind::Word(name) if name == "super" => {
                // 19.2.1.1 and 15.2/15.4: arrows inherit the enclosing method
                // context, but ordinary functions and indirect eval do not.
                if self.at("(") && self.allow_super_call {
                    let arguments = self.arguments()?;
                    let span = Span::new(span.start, self.tokens[self.index - 1].span.end);
                    self.make_expr(ExprKind::SuperCall(arguments), span)
                } else if self.at(".") || self.at("[") {
                    if self.allow_super_property {
                        if self.at(".")
                            && self
                                .token_at(self.index + 1)
                                .is_some_and(|token| token.kind == Kind::Punct("#"))
                        {
                            return Err(early(
                                self.tokens[self.index + 1].span,
                                "super cannot access private properties",
                            ));
                        }
                        let name = self.member_property_name()?;
                        let span = Span::new(span.start, self.tokens[self.index - 1].span.end);
                        self.make_expr(ExprKind::SuperProperty(name), span)
                    } else {
                        Err(early(span, "super property access outside a method"))
                    }
                } else {
                    Err(early(span, "super call requires a derived constructor"))
                }
            }
            Kind::Word(name) if !token.escaped && name == "import" && self.at(".") => {
                if !self.module_goal {
                    return Err(early(span, "import.meta is only allowed in Modules"));
                }
                self.bump();
                if !self.at("meta") || self.current().escaped {
                    return Err(self.error("expected meta after import dot"));
                }
                let end = self.bump().span.end;
                self.make_expr(ExprKind::ImportMeta, Span::new(span.start, end))
            }
            Kind::Word(_) => Err(early(span, "reserved word cannot begin an expression")),
            _ => Err(Diagnostic::new(
                DiagnosticKind::Syntax,
                span,
                "expected an expression",
            )),
        }
    }
}
