//! Expression heads and continuations use separate native parser frames.

use super::*;

impl Parser {
    #[inline(never)]
    pub(super) fn expression_head(&mut self, minimum: u8) -> Result<Expr, Diagnostic> {
        if minimum <= 2 && self.at("yield") && !matches!(self.yield_context, YieldContext::None) {
            return self.yield_expression();
        }
        if minimum <= 10 && self.allow_in && self.at("#") {
            let name = self.private_identifier()?;
            self.use_private_identifier(&name)?;
            self.expect("in")?;
            let value = self.expression(11)?;
            let span = Span::new(name.span.start, value.span.end);
            return self.make_expr(
                ExprKind::PrivateIn {
                    name,
                    value: Box::new(value),
                },
                span,
            );
        }
        if minimum <= 2 {
            let cover = self
                .pattern_cover()?
                .filter(|cover| self.tokens[cover.end].kind == Kind::Punct("="));
            if let Some(cover) = cover {
                let pattern = self.consume_pattern_cover(&cover)?;
                self.expect("=")?;
                let value = self.expression(2)?;
                let span = Span::new(pattern.span.start, value.span.end);
                return self.make_expr(
                    ExprKind::DestructuringAssign {
                        pattern: Box::new(pattern),
                        value: Box::new(value),
                    },
                    span,
                );
            }
            if let Some(arrow) = self.arrow_expression()? {
                return Ok(arrow);
            }
        }
        self.prefix()
    }

    #[inline(never)]
    pub(super) fn expression_tail(
        &mut self,
        mut left: Expr,
        minimum: u8,
    ) -> Result<Expr, Diagnostic> {
        loop {
            // YieldExpression is an AssignmentExpression; only an outer comma
            // may extend it without parentheses.
            if matches!(left.kind, ExprKind::Yield { .. }) && !self.at(",") {
                break;
            }
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
                let property = self.member_property_name()?;
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
            if op == BinaryOp::Exponentiate
                && matches!(left.kind, ExprKind::Unary(..) | ExprKind::Await(_))
            {
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
            left = self.make_binary_expr(op, left, right, span)?;
        }
        Ok(left)
    }
}
