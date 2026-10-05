//! Class definitions and their strict early errors (15.7).

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
        let heritage = if self.eat("extends") {
            let expression = self.expression_with_in(17, true)?;
            if matches!(
                expression.kind,
                ExprKind::Unary(..) | ExprKind::Update { .. }
            ) {
                return Err(early(
                    expression.span,
                    "class heritage requires a left-hand-side expression",
                ));
            }
            Some(expression)
        } else {
            None
        };
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
                    if matches!(prefix.as_str(), "get" | "set") && self.class_method_name_ahead() {
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
                    } else if prefix == "async"
                        && !self.current().newline
                        && (self.at("*") || self.class_method_name_ahead())
                    {
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
                if kind != PropertyKind::Method {
                    return Err(self.error("class accessor requires parameters"));
                }
                if matches!(&property_name, PropertyName::Literal(Literal::String(name)) if name == &JsString::from("constructor") || (is_static && name == &JsString::from("prototype")))
                {
                    return Err(early(name_span, "invalid class field name"));
                }
                let initializer = if self.eat("=") {
                    Some(Rc::new(self.class_field_initializer()?))
                } else {
                    None
                };
                if !(self.eat(";") || self.at("}") || self.current().newline) {
                    return Err(self.error("expected class field terminator"));
                }
                elements.push(ClassElement::Field {
                    is_static,
                    name: property_name,
                    initializer,
                    span: Span::new(element_start, self.tokens[self.index - 1].span.end),
                });
                continue;
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
            let value =
                self.method_definition(method_start, kind, is_constructor && heritage.is_some())?;
            if is_constructor {
                let ExprKind::Function(function) = value.kind else {
                    unreachable!("method syntax")
                };
                constructor = Some(function);
            } else {
                elements.push(ClassElement::Method {
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
        let default_constructor = constructor.is_none();
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
            heritage,
            default_constructor,
            constructor,
            elements,
            source,
        }))
    }

    fn class_field_initializer(&mut self) -> Result<Expr, Diagnostic> {
        // ContainsArguments crosses arrows and computed method names, while
        // ordinary functions/method bodies establish their own boundary (15.7.2).
        let previous = (
            self.allow_new_target,
            self.allow_super_property,
            self.allow_super_call,
            self.allow_arguments,
        );
        self.allow_new_target = true;
        self.allow_super_property = true;
        self.allow_super_call = false;
        self.allow_arguments = false;
        let result = self.expression_with_in(2, true);
        (
            self.allow_new_target,
            self.allow_super_property,
            self.allow_super_call,
            self.allow_arguments,
        ) = previous;
        result
    }

    fn class_method_name_ahead(&self) -> bool {
        let start = self.index;
        let end = match &self.current().kind {
            Kind::Word(_) | Kind::Literal(_) => start + 1,
            Kind::Punct("#") => return true,
            Kind::Punct("[") => {
                let mut depth = 0usize;
                let mut end = None;
                for (index, token) in self.tokens.iter().enumerate().skip(start) {
                    match token.kind {
                        Kind::Punct("[") => depth += 1,
                        Kind::Punct("]") => {
                            depth -= 1;
                            if depth == 0 {
                                end = Some(index + 1);
                                break;
                            }
                        }
                        Kind::Eof => break,
                        _ => {}
                    }
                }
                let Some(end) = end else { return false };
                end
            }
            _ => return false,
        };
        self.tokens
            .get(end)
            .is_some_and(|token| token.kind == Kind::Punct("("))
    }
}

fn early_unsupported(span: Span, message: &str) -> Diagnostic {
    Diagnostic::new(DiagnosticKind::Unsupported, span, message)
}

pub(super) fn validate_class(class: &Class) -> Result<(), Diagnostic> {
    if let Some(heritage) = &class.heritage {
        validate_expr(heritage, true)?;
    }
    if let Some(name) = &class.name {
        validate_binding_name(&name.name, name.span, true)?;
    }
    function::validate_method(&class.constructor, true)?;
    for element in &class.elements {
        if let PropertyName::Computed(key) = element.name() {
            validate_expr(key, true)?;
        }
        match element {
            ClassElement::Method { property, .. } => {
                let ExprKind::Function(method) = &property.value.kind else {
                    unreachable!("method syntax")
                };
                function::validate_method(method, true)?;
            }
            ClassElement::Field {
                initializer: Some(initializer),
                ..
            } => validate_expr(initializer, true)?,
            ClassElement::Field {
                initializer: None, ..
            } => {}
        }
    }
    Ok(())
}
