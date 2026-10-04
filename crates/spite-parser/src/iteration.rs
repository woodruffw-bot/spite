//! Three-clause and synchronous for-of loop headers (14.7.4–5).

use crate::*;

pub(super) enum ForHeader {
    ThreeClause {
        initializer: Option<ForInitializer>,
        test: Option<Expr>,
        update: Option<Expr>,
    },
    Of {
        binding: ForOfBinding,
        iterable: Expr,
    },
}

impl ForHeader {
    pub(super) fn with_body(self, body: Box<Statement>) -> StatementKind {
        match self {
            Self::ThreeClause {
                initializer,
                test,
                update,
            } => StatementKind::For {
                initializer,
                test,
                update,
                body,
            },
            Self::Of { binding, iterable } => StatementKind::ForOf {
                binding,
                iterable,
                body,
            },
        }
    }
}

impl Parser {
    // Finish the header before recursively parsing the body. Large temporary AST
    // values must not remain on native frames at every nested statement level.
    pub(super) fn for_header(&mut self) -> Result<Box<ForHeader>, Diagnostic> {
        if self.at("await") {
            return Err(self.unsupported("for-await-of is not implemented"));
        }
        self.expect("(")?;
        let header_start = self.current().clone();
        let initializer = if self.at(";") {
            None
        } else if self.eat("var") {
            Some(ForInitializer::Var(self.binding_list(false, true, false)?))
        } else if self.at("const")
            || (self.at("let")
                && self
                    .tokens
                    .get(self.index + 1)
                    .is_some_and(|t| matches!(t.kind, Kind::Word(_) | Kind::Punct("[" | "{"))))
        {
            let (mutable, bindings) = self.lexical_bindings(true)?;
            Some(ForInitializer::Lexical { mutable, bindings })
        } else {
            Some(ForInitializer::Expression(
                self.expression_with_in(1, false)?,
            ))
        };
        if self.at("in") {
            return Err(self.unsupported("for-in is not implemented"));
        }
        if self.eat("of") {
            let binding = self.for_of_binding(initializer, &header_start)?;
            let iterable = self.expression_with_in(2, true)?;
            self.expect(")")?;
            Ok(Box::new(ForHeader::Of { binding, iterable }))
        } else {
            // ECMA-262 12.10.1: ASI never supplies either header semicolon.
            self.expect(";")?;
            let test = if self.at(";") {
                None
            } else {
                Some(self.expression(1)?)
            };
            self.expect(";")?;
            let update = if self.at(")") {
                None
            } else {
                Some(self.expression(1)?)
            };
            self.expect(")")?;
            Ok(Box::new(ForHeader::ThreeClause {
                initializer,
                test,
                update,
            }))
        }
    }

    fn for_of_binding(
        &self,
        initializer: Option<ForInitializer>,
        header_start: &Token,
    ) -> Result<ForOfBinding, Diagnostic> {
        match initializer.expect("of follows a parsed header") {
            ForInitializer::Expression(expression) => {
                if matches!(expression.kind, ExprKind::Array(_) | ExprKind::Object(_)) {
                    return Err(self.unsupported("assignment patterns are not implemented"));
                }
                // 14.7.5 grammar excludes leading let and the bare async-of form.
                if (matches!(&header_start.kind, Kind::Word(name) if name == "let")
                    && !header_start.escaped)
                    || (matches!(&expression.kind, ExprKind::Identifier(name) if name == "async")
                        && !header_start.escaped)
                    || !assignment_target(&expression)
                {
                    return Err(early(expression.span, "invalid for-of assignment target"));
                }
                Ok(ForOfBinding::Assignment(expression))
            }
            declaration => {
                let (mutable, mut bindings) = match declaration {
                    ForInitializer::Var(bindings) => (None, bindings),
                    ForInitializer::Lexical { mutable, bindings } => (Some(mutable), bindings),
                    ForInitializer::Expression(_) => unreachable!(),
                };
                if bindings.len() != 1 {
                    return Err(early(
                        bindings[1].span,
                        "for-of requires one binding without an initializer",
                    ));
                }
                if let Some(initializer) = &bindings[0].initializer {
                    return Err(early(
                        initializer.span,
                        "for-of requires one binding without an initializer",
                    ));
                }
                let binding = bindings.pop().expect("one binding");
                Ok(match mutable {
                    Some(mutable) => ForOfBinding::Lexical { mutable, binding },
                    None => ForOfBinding::Var(binding),
                })
            }
        }
    }
}
