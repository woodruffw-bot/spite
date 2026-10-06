//! Supplemental destructuring assignment grammar and early errors (13.15.5).

use super::*;

pub(super) type Cover = std::rc::Rc<cover::Probe<AssignmentPattern>>;

impl Parser {
    // The supplemental AssignmentPattern grammar selects lexical goals in
    // computed keys/defaults; ordinary literals keep their own grammar.
    pub(super) fn pattern_cover(&mut self) -> Result<Option<Cover>, Diagnostic> {
        if !matches!(self.current().kind, Kind::Punct("{" | "[")) {
            return Ok(None);
        }
        let key = self.cover_key(self.index);
        if let Some(cover) = self.pattern_covers.get(&key) {
            return Ok(cover.clone());
        }
        let probe = self.probe_cover(Self::assignment_pattern);
        let cover = match probe {
            Ok(cover) => Some(cover),
            Err(error) if error.kind == DiagnosticKind::Syntax => None,
            Err(error) => return Err(error),
        };
        self.pattern_covers.insert(key, cover.clone());
        Ok(cover)
    }

    #[inline(never)]
    pub(super) fn consume_pattern_cover(
        &mut self,
        cover: &Cover,
    ) -> Result<AssignmentPattern, Diagnostic> {
        self.consume_cover(cover)?;
        Ok(cover.value.clone())
    }

    pub(super) fn assignment_pattern(&mut self) -> Result<AssignmentPattern, Diagnostic> {
        self.enter()?;
        let result = self.assignment_pattern_inner();
        self.depth -= 1;
        result
    }

    fn assignment_pattern_inner(&mut self) -> Result<AssignmentPattern, Diagnostic> {
        let start = self.current().span.start;
        let kind = if self.eat("{") {
            let mut properties = Vec::new();
            let mut rest = None;
            while !self.at("}") {
                if self.eat("...") {
                    let target = self.destructuring_target()?;
                    let reference = match target {
                        AssignmentTarget::Reference(reference) => reference,
                        AssignmentTarget::Pattern(pattern) => {
                            self.defer_cover_error(early(
                                pattern.span,
                                "object assignment rest requires a reference",
                            ))?;
                            Box::new(
                                self.make_expr(ExprKind::Literal(Literal::Null), pattern.span)?,
                            )
                        }
                    };
                    rest = Some(reference);
                    if self.probe_rest_continuation("}")? {
                        continue;
                    }
                    break;
                }
                let token = self.bump();
                let key = self.object_property_name(token.clone())?;
                let element = if self.probing_cover && self.at("(") {
                    let error = if matches!(token.kind, Kind::Word(_)) {
                        self.error("expected }")
                    } else {
                        early(token.span, "expected assignment identifier")
                    };
                    self.defer_cover_error(error)?;
                    self.object_method(token.span.start, PropertyKind::Method)?;
                    AssignmentElement {
                        target: AssignmentTarget::Reference(Box::new(
                            self.make_expr(ExprKind::Literal(Literal::Null), token.span)?,
                        )),
                        initializer: None,
                    }
                } else if self.probing_cover
                    && !token.escaped
                    && matches!(&token.kind, Kind::Word(name) if name == "get" || name == "set")
                    && !self.at(":")
                    && !self.at("=")
                    && !self.at(",")
                    && !self.at("}")
                {
                    self.defer_cover_error(self.error("expected }"))?;
                    let kind = if matches!(&token.kind, Kind::Word(name) if name == "get") {
                        PropertyKind::Getter
                    } else {
                        PropertyKind::Setter
                    };
                    let name = self.bump();
                    self.object_property_name(name)?;
                    self.object_method(token.span.start, kind)?;
                    AssignmentElement {
                        target: AssignmentTarget::Reference(Box::new(
                            self.make_expr(ExprKind::Literal(Literal::Null), token.span)?,
                        )),
                        initializer: None,
                    }
                } else if self.eat(":") {
                    self.assignment_element()?
                } else {
                    let Kind::Word(name) = token.kind else {
                        return Err(early(token.span, "expected assignment identifier"));
                    };
                    if reserved(&name) {
                        self.defer_cover_error(early(token.span, "invalid assignment identifier"))?;
                    }
                    let target = AssignmentTarget::Reference(Box::new(
                        self.make_expr(ExprKind::Identifier(name), token.span)?,
                    ));
                    AssignmentElement {
                        target,
                        initializer: self.assignment_initializer()?,
                    }
                };
                properties.push(AssignmentProperty { key, element });
                if !self.eat(",") {
                    break;
                }
            }
            self.expect("}")?;
            AssignmentPatternKind::Object { properties, rest }
        } else {
            self.expect("[")?;
            let mut elements = Vec::new();
            let mut rest = None;
            while !self.at("]") {
                if self.eat(",") {
                    elements.push(None);
                    continue;
                }
                if self.eat("...") {
                    rest = Some(self.destructuring_target()?);
                    if self.probe_rest_continuation("]")? {
                        continue;
                    }
                    break;
                }
                elements.push(Some(self.assignment_element()?));
                if !self.eat(",") {
                    break;
                }
            }
            self.expect("]")?;
            AssignmentPatternKind::Array { elements, rest }
        };
        Ok(AssignmentPattern {
            kind,
            span: Span::new(start, self.tokens[self.index - 1].span.end),
        })
    }

    fn destructuring_target(&mut self) -> Result<AssignmentTarget, Diagnostic> {
        if let Some(cover) = self.pattern_cover()? {
            // A literal followed by member/call/template syntax is part of a
            // reference expression rather than a nested AssignmentPattern.
            if !matches!(
                self.tokens.get(cover.end).map(|t| &t.kind),
                Some(Kind::Punct("." | "[" | "(" | "?.")) | Some(Kind::Template { .. })
            ) {
                return self
                    .consume_pattern_cover(&cover)
                    .map(|pattern| AssignmentTarget::Pattern(Box::new(pattern)));
            }
        }
        let reference = self.expression_with_in(3, true)?;
        if !assignment_target(&reference) {
            self.defer_cover_error(early(
                reference.span,
                "invalid destructuring assignment target",
            ))?;
        }
        Ok(AssignmentTarget::Reference(Box::new(reference)))
    }

    fn assignment_element(&mut self) -> Result<AssignmentElement, Diagnostic> {
        let target = self.destructuring_target()?;
        Ok(AssignmentElement {
            target,
            initializer: self.assignment_initializer()?,
        })
    }

    fn assignment_initializer(&mut self) -> Result<Option<Expr>, Diagnostic> {
        if self.eat("=") {
            Ok(Some(self.expression_with_in(2, true)?))
        } else {
            Ok(None)
        }
    }
}

pub(super) fn validate_pattern(
    pattern: &AssignmentPattern,
    strict: bool,
) -> Result<(), Diagnostic> {
    match &pattern.kind {
        AssignmentPatternKind::Object { properties, rest } => {
            for property in properties {
                if let PropertyName::Computed(key) = &property.key {
                    validate_expr(key, strict)?;
                }
                validate_element(&property.element, strict)?;
            }
            if let Some(rest) = rest {
                validate_reference(rest, strict)?;
            }
        }
        AssignmentPatternKind::Array { elements, rest } => {
            for element in elements.iter().flatten() {
                validate_element(element, strict)?;
            }
            if let Some(rest) = rest {
                validate_target(rest, strict)?;
            }
        }
    }
    Ok(())
}

fn validate_element(element: &AssignmentElement, strict: bool) -> Result<(), Diagnostic> {
    validate_target(&element.target, strict)?;
    if let Some(initializer) = &element.initializer {
        validate_expr(initializer, strict)?;
    }
    Ok(())
}

fn validate_target(target: &AssignmentTarget, strict: bool) -> Result<(), Diagnostic> {
    match target {
        AssignmentTarget::Reference(reference) => validate_reference(reference, strict),
        AssignmentTarget::Pattern(pattern) => validate_pattern(pattern, strict),
    }
}

fn validate_reference(reference: &Expr, strict: bool) -> Result<(), Diagnostic> {
    if strict
        && assignment_name(reference)
            .is_some_and(|name| strict_reserved(name) || matches!(name, "eval" | "arguments"))
    {
        return Err(early(reference.span, "invalid assignment in strict mode"));
    }
    validate_expr(reference, strict)
}
