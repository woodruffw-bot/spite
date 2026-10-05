//! Destructuring binding syntax and strict validation (14.3.3, 14.15.1).

use super::*;

impl Parser {
    pub(super) fn binding_pattern(&mut self) -> Result<BindingPattern, Diagnostic> {
        self.enter()?;
        let result = self.binding_pattern_inner();
        self.depth -= 1;
        result
    }

    fn binding_pattern_inner(&mut self) -> Result<BindingPattern, Diagnostic> {
        let start = self.current().span.start;
        let kind = if self.eat("{") {
            let mut properties = Vec::new();
            let mut rest = None;
            while !self.at("}") {
                if self.eat("...") {
                    rest = Some(Box::new(self.binding_identifier()?));
                    break;
                }
                let token = self.bump();
                let key = self.object_property_name(token.clone())?;
                let element = if self.eat(":") {
                    self.binding_element()?
                } else {
                    let pattern = self.binding_identifier_token(token)?;
                    let initializer = self.binding_initializer()?;
                    BindingElement {
                        pattern,
                        initializer,
                    }
                };
                properties.push(BindingProperty { key, element });
                if !self.eat(",") {
                    break;
                }
            }
            self.expect("}")?;
            BindingPatternKind::Object { properties, rest }
        } else if self.eat("[") {
            let mut elements = Vec::new();
            let mut rest = None;
            while !self.at("]") {
                if self.eat(",") {
                    elements.push(None);
                    continue;
                }
                if self.eat("...") {
                    rest = Some(Box::new(self.binding_pattern()?));
                    break;
                }
                elements.push(Some(self.binding_element()?));
                if !self.eat(",") {
                    break;
                }
            }
            self.expect("]")?;
            BindingPatternKind::Array { elements, rest }
        } else {
            return self.binding_identifier();
        };
        Ok(BindingPattern {
            kind,
            span: Span::new(start, self.tokens[self.index - 1].span.end),
        })
    }

    fn binding_identifier(&mut self) -> Result<BindingPattern, Diagnostic> {
        let token = self.bump();
        self.binding_identifier_token(token)
    }

    fn binding_identifier_token(&self, token: Token) -> Result<BindingPattern, Diagnostic> {
        let Kind::Word(name) = token.kind else {
            return Err(early(token.span, "expected binding identifier"));
        };
        if reserved(&name) || (name == "await" && !self.allow_await_identifier) {
            return Err(early(token.span, "invalid binding identifier"));
        }
        Ok(BindingPattern {
            kind: BindingPatternKind::Identifier(name),
            span: token.span,
        })
    }

    fn binding_element(&mut self) -> Result<BindingElement, Diagnostic> {
        let pattern = self.binding_pattern()?;
        let initializer = self.binding_initializer()?;
        Ok(BindingElement {
            pattern,
            initializer,
        })
    }

    fn binding_initializer(&mut self) -> Result<Option<Expr>, Diagnostic> {
        if self.eat("=") {
            Ok(Some(self.expression_with_in(2, true)?))
        } else {
            Ok(None)
        }
    }
}

pub(super) fn validate_pattern(pattern: &BindingPattern, strict: bool) -> Result<(), Diagnostic> {
    match &pattern.kind {
        BindingPatternKind::Identifier(name) => validate_binding_name(name, pattern.span, strict)?,
        BindingPatternKind::Object { properties, rest } => {
            for property in properties {
                if let PropertyName::Computed(expression) = &property.key {
                    validate_expr(expression, strict)?;
                }
                validate_element(&property.element, strict)?;
            }
            if let Some(rest) = rest {
                validate_pattern(rest, strict)?;
            }
        }
        BindingPatternKind::Array { elements, rest } => {
            for element in elements.iter().flatten() {
                validate_element(element, strict)?;
            }
            if let Some(rest) = rest {
                validate_pattern(rest, strict)?;
            }
        }
    }
    Ok(())
}

pub(super) fn validate_element(element: &BindingElement, strict: bool) -> Result<(), Diagnostic> {
    validate_pattern(&element.pattern, strict)?;
    if let Some(expression) = &element.initializer {
        validate_expr(expression, strict)?;
    }
    Ok(())
}
