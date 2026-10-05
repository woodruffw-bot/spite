//! Three-clause, for-in, and synchronous for-of loop headers (14.7.4–5).

use crate::*;

pub(super) enum ForHeader {
    ThreeClause {
        initializer: Option<ForInitializer>,
        test: Option<Expr>,
        update: Option<Expr>,
    },
    Of {
        binding: ForBinding,
        iterable: Expr,
    },
    In {
        binding: ForBinding,
        object: Expr,
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
            Self::In { binding, object } => StatementKind::ForIn {
                binding,
                object,
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
                    .is_some_and(|t| match &t.kind {
                        Kind::Word(name) => !reserved(name),
                        Kind::Punct("[" | "{") => true,
                        _ => false,
                    }))
        {
            let (mutable, bindings) = self.lexical_bindings(true)?;
            Some(ForInitializer::Lexical { mutable, bindings })
        } else {
            Some(ForInitializer::Expression(
                self.expression_with_in(1, false)?,
            ))
        };
        let is_of = self.eat("of");
        if is_of || self.eat("in") {
            let binding = self.iteration_binding(initializer, &header_start, is_of)?;
            let expression = self.expression_with_in(if is_of { 2 } else { 1 }, true)?;
            self.expect(")")?;
            Ok(Box::new(if is_of {
                ForHeader::Of {
                    binding,
                    iterable: expression,
                }
            } else {
                ForHeader::In {
                    binding,
                    object: expression,
                }
            }))
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

    fn iteration_binding(
        &self,
        initializer: Option<ForInitializer>,
        header_start: &Token,
        is_of: bool,
    ) -> Result<ForBinding, Diagnostic> {
        let invalid_target = if is_of {
            "invalid for-of assignment target"
        } else {
            "invalid for-in assignment target"
        };
        let invalid_binding = if is_of {
            "for-of requires one binding without an initializer"
        } else {
            "for-in requires one binding without an initializer"
        };
        match initializer.expect("in/of follows a parsed header") {
            ForInitializer::Expression(expression) => {
                if matches!(expression.kind, ExprKind::Array(_) | ExprKind::Object(_)) {
                    return Err(self.unsupported("assignment patterns are not implemented"));
                }
                // 14.7.5 grammar excludes leading let and the bare async-of form.
                if (is_of
                    && !header_start.escaped
                    && (matches!(&header_start.kind, Kind::Word(name) if name == "let")
                        || matches!(&expression.kind, ExprKind::Identifier(name) if name == "async")))
                    || !assignment_target(&expression)
                {
                    return Err(early(expression.span, invalid_target));
                }
                Ok(ForBinding::Assignment(expression))
            }
            ForInitializer::Var(mut bindings) => {
                if bindings.len() != 1 {
                    return Err(early(bindings[1].pattern.span, invalid_binding));
                }
                if let Some(initializer) = &bindings[0].initializer {
                    if is_of
                        || !matches!(bindings[0].pattern.kind, BindingPatternKind::Identifier(_))
                    {
                        return Err(early(initializer.span, invalid_binding));
                    }
                }
                let binding = bindings.pop().expect("one binding");
                Ok(ForBinding::Var(binding))
            }
            ForInitializer::Lexical {
                mutable,
                mut bindings,
            } => {
                if bindings.len() != 1 {
                    return Err(early(bindings[1].pattern.span, invalid_binding));
                }
                if let Some(initializer) = &bindings[0].initializer {
                    return Err(early(initializer.span, invalid_binding));
                }
                let binding = bindings.pop().expect("one binding").pattern;
                Ok(ForBinding::Lexical { mutable, binding })
            }
        }
    }
}
