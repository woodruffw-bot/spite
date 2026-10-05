//! Class definitions and their strict early errors (15.7).

use super::*;
use std::rc::Rc;

impl Parser {
    pub(super) fn class_definition(&mut self, require_name: bool) -> Result<Rc<Class>, Diagnostic> {
        self.enter()?;
        let private_depth = self.private_scopes.len();
        let result = self.class_definition_inner(require_name);
        let result = if self.private_scopes.len() > private_depth {
            let scope = self.private_scopes.pop().expect("class private scope");
            match result {
                Ok(class) => self.finish_private_scope(scope).map(|()| class),
                Err(error) => Err(error),
            }
        } else {
            result
        };
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
            if reserved(&name) || (name == "await" && !self.allow_await_identifier) {
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
        // ClassHeritage uses the enclosing private scope; only ClassBody adds
        // the class's names (AllPrivateIdentifiersValid, 16.1.1).
        self.private_scopes.push(Default::default());
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
                    let body = self.class_static_block()?;
                    elements.push(ClassElement::StaticBlock {
                        body,
                        span: Span::new(element_start, self.tokens[self.index - 1].span.end),
                    });
                    continue;
                }
                token = self.bump();
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
            let mut name_span = token.span;
            let property_name = if token.kind == Kind::Punct("#") {
                let name = self.private_identifier_after_hash(token)?;
                name_span = name.span;
                PropertyName::Private(name)
            } else {
                self.object_property_name(token)?
            };
            if !self.at("(") {
                if kind != PropertyKind::Method {
                    return Err(self.error("class accessor requires parameters"));
                }
                if let PropertyName::Private(name) = &property_name {
                    self.declare_private_identifier(name, is_static, PropertyKind::Data)?;
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
            if let PropertyName::Private(name) = &property_name {
                self.declare_private_identifier(name, is_static, kind)?;
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

    fn class_static_block(&mut self) -> Result<FunctionBody, Diagnostic> {
        self.enter()?;
        let previous = (
            self.allow_return,
            self.allow_in,
            self.allow_new_target,
            self.allow_super_property,
            self.allow_super_call,
            self.allow_arguments,
            self.allow_await_identifier,
        );
        self.allow_return = false;
        self.allow_in = true;
        self.allow_new_target = true;
        self.allow_super_property = true;
        self.allow_super_call = false;
        self.allow_arguments = false;
        self.allow_await_identifier = false;
        let result = (|| {
            self.expect("{")?;
            let mut statements = Vec::new();
            while !self.at("}") {
                if self.current().kind == Kind::Eof {
                    return Err(self.error("unterminated static initialization block"));
                }
                statements.push(self.statement(true)?);
            }
            self.expect("}")?;
            let strict = has_use_strict(&statements, self.source.lexical_text());
            Ok(FunctionBody {
                statements: statements.into(),
                strict,
            })
        })();
        (
            self.allow_return,
            self.allow_in,
            self.allow_new_target,
            self.allow_super_property,
            self.allow_super_call,
            self.allow_arguments,
            self.allow_await_identifier,
        ) = previous;
        self.depth -= 1;
        result
    }

    fn class_method_name_ahead(&mut self) -> bool {
        let start = self.index;
        let end = match &self.current().kind {
            Kind::Word(_) | Kind::Literal(_) => start + 1,
            Kind::Punct("#") => start + 2,
            Kind::Punct("[") => {
                let mut depth = 0usize;
                let mut end = None;
                let mut index = start;
                while let Some(token) = self.token_at(index) {
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
                    index += 1;
                }
                let Some(end) = end else { return false };
                end
            }
            _ => return false,
        };
        self.token_at(end)
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
        if let Some(PropertyName::Computed(key)) = element.name() {
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
            ClassElement::StaticBlock { body, .. } => {
                function::validate_body(body, true, &BTreeSet::new())?
            }
        }
    }
    Ok(())
}
