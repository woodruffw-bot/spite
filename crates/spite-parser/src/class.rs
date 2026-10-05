//! Base class definitions and their strict early errors (15.7).

use super::*;
use std::rc::Rc;

impl Parser {
    pub(super) fn class_definition(&mut self, require_name: bool) -> Result<Rc<Class>, Diagnostic> {
        self.enter()?;
        let result = self.class_definition_inner(require_name);
        self.depth -= 1;
        result
    }

    fn class_definition_inner(&mut self, require_name: bool) -> Result<Rc<Class>, Diagnostic> {
        let first = self.index;
        let start = self.current().span.start;
        self.expect("class")?;
        let name = if self.at("{") || self.at("extends") {
            if require_name {
                return Err(self.error("class declaration requires a name"));
            }
            None
        } else {
            let token = self.bump();
            let Kind::Word(name) = token.kind else {
                return Err(early(token.span, "expected a class binding identifier"));
            };
            if reserved(&name) {
                return Err(early(token.span, "invalid class binding identifier"));
            }
            validate_binding_name(&name, token.span, true)?;
            Some(FunctionName {
                name,
                span: token.span,
            })
        };
        if self.at("extends") {
            return Err(self.unsupported("class heritage is not implemented"));
        }
        self.expect("{")?;
        let mut constructor = None;
        let mut elements = Vec::new();
        while !self.at("}") {
            if self.eat(";") {
                continue;
            }
            if self.current().kind == Kind::Eof {
                return Err(self.error("unterminated class body"));
            }
            let element_start = self.current().span.start;
            let mut token = self.bump();
            let is_static = !token.escaped
                && matches!(&token.kind, Kind::Word(name) if name == "static")
                && !self.at("(")
                && !self.at(";")
                && !self.at("}")
                && !self.at("=");
            if is_static {
                if self.at("{") {
                    return Err(self.unsupported("class static blocks are not implemented"));
                }
                token = self.bump();
            }
            if token.kind == Kind::Punct("#") {
                return Err(early_unsupported(
                    token.span,
                    "private class elements are not implemented",
                ));
            }
            if token.kind == Kind::Punct("*") {
                return Err(early_unsupported(
                    token.span,
                    "generator methods are not implemented",
                ));
            }
            let method_start = token.span.start;
            let mut kind = PropertyKind::Method;
            if !token.escaped && !self.at("(") {
                if let Kind::Word(prefix) = &token.kind {
                    if matches!(prefix.as_str(), "get" | "set")
                        && !self.at(";")
                        && !self.at("}")
                        && !self.at("=")
                    {
                        kind = if prefix == "get" {
                            PropertyKind::Getter
                        } else {
                            PropertyKind::Setter
                        };
                        token = self.bump();
                        if token.kind == Kind::Punct("#") {
                            return Err(early_unsupported(
                                token.span,
                                "private class elements are not implemented",
                            ));
                        }
                    } else if prefix == "async" && !self.current().newline {
                        return Err(early_unsupported(
                            token.span,
                            "async methods are not implemented",
                        ));
                    }
                }
            }
            let name_span = token.span;
            let property_name = self.object_property_name(token)?;
            if !self.at("(") {
                return Err(self.unsupported("class fields are not implemented"));
            }
            let is_constructor = !is_static
                && matches!(&property_name, PropertyName::Literal(Literal::String(name)) if name == &JsString::from("constructor"));
            if is_constructor && (kind != PropertyKind::Method || constructor.is_some()) {
                return Err(early(name_span, "invalid or duplicate class constructor"));
            }
            if is_static
                && matches!(&property_name, PropertyName::Literal(Literal::String(name)) if name == &JsString::from("prototype"))
            {
                return Err(early(
                    name_span,
                    "static class method cannot be named prototype",
                ));
            }
            let value = self.object_method(method_start, kind)?;
            if is_constructor {
                let ExprKind::Function(function) = value.kind else {
                    unreachable!("method syntax")
                };
                constructor = Some(function);
            } else {
                elements.push(ClassElement {
                    is_static,
                    property: ObjectProperty {
                        name: property_name,
                        span: Span::new(element_start, value.span.end),
                        value,
                        kind,
                    },
                });
            }
        }
        self.expect("}")?;
        // All ClassDefinition code, including computed names and nested code,
        // is strict, independently of the surrounding Script's strictness.
        reject_legacy_tokens(&self.tokens[first..self.index])?;
        let source = FunctionSource {
            text: self.source.clone(),
            span: Span::new(start, self.tokens[self.index - 1].span.end),
        };
        let mut constructor = constructor.unwrap_or_else(|| {
            Rc::new(Function {
                name: None,
                parameters: Rc::from([]),
                body: FunctionBody {
                    statements: Rc::from([]),
                    strict: false,
                },
                source: source.clone(),
            })
        });
        Rc::make_mut(&mut constructor).source = source.clone();
        Ok(Rc::new(Class {
            name,
            constructor,
            elements,
            source,
        }))
    }
}

fn early_unsupported(span: Span, message: &str) -> Diagnostic {
    Diagnostic::new(DiagnosticKind::Unsupported, span, message)
}

pub(super) fn validate_class(class: &Class) -> Result<(), Diagnostic> {
    if let Some(name) = &class.name {
        validate_binding_name(&name.name, name.span, true)?;
    }
    function::validate_method(&class.constructor, true)?;
    for element in &class.elements {
        if let PropertyName::Computed(key) = &element.property.name {
            validate_expr(key, true)?;
        }
        let ExprKind::Function(method) = &element.property.value.kind else {
            unreachable!("method syntax")
        };
        function::validate_method(method, true)?;
    }
    Ok(())
}
