//! Object data properties and ordinary method/accessor definitions (13.2.5, 15.4).

use super::*;
use std::rc::Rc;

impl Parser {
    pub(super) fn object_literal(&mut self, start: usize) -> Result<Expr, Diagnostic> {
        let mut properties = Vec::new();
        let mut prototype_seen = false;
        while !self.at("}") {
            if self.eat("...") {
                properties.push(ObjectElement::Spread(self.expression_with_in(2, true)?));
                if !self.eat(",") {
                    break;
                }
                continue;
            }
            if self.at("*") {
                return Err(self.unsupported("generator methods are not implemented"));
            }
            let token = self.bump();
            let property_start = token.span.start;
            let shorthand = match &token.kind {
                Kind::Word(name) => Some(name.clone()),
                _ => None,
            };
            let mut name = self.object_property_name(token.clone())?;
            let (kind, value) = if self.eat(":") {
                let prototype = matches!(&name, PropertyName::Literal(Literal::String(name)) if name == &JsString::from("__proto__"));
                if prototype && prototype_seen {
                    return Err(early(
                        token.span,
                        "duplicate prototype setter in object literal",
                    ));
                }
                prototype_seen |= prototype;
                (
                    if prototype {
                        PropertyKind::Prototype
                    } else {
                        PropertyKind::Data
                    },
                    self.expression_with_in(2, true)?,
                )
            } else if self.at("(") {
                (
                    PropertyKind::Method,
                    self.object_method(property_start, PropertyKind::Method)?,
                )
            } else {
                if self.at("=") {
                    return Err(
                        self.error("initialized shorthand is not allowed in an object literal")
                    );
                }
                if !token.escaped
                    && !self.at(",")
                    && !self.at("}")
                    && shorthand
                        .as_deref()
                        .is_some_and(|name| matches!(name, "get" | "set"))
                {
                    let kind = if shorthand.as_deref() == Some("get") {
                        PropertyKind::Getter
                    } else {
                        PropertyKind::Setter
                    };
                    let name_token = self.bump();
                    name = self.object_property_name(name_token)?;
                    (kind, self.object_method(property_start, kind)?)
                } else {
                    if !token.escaped
                        && shorthand.as_deref() == Some("async")
                        && !self.at(",")
                        && !self.at("}")
                    {
                        return Err(self.unsupported("async methods are not implemented"));
                    }
                    let Some(identifier) = shorthand.filter(|name| !reserved(name)) else {
                        return Err(early(token.span, "expected a colon after property name"));
                    };
                    (
                        PropertyKind::Shorthand,
                        self.make_expr(ExprKind::Identifier(identifier), token.span)?,
                    )
                }
            };
            let span = Span::new(property_start, value.span.end);
            properties.push(ObjectElement::Property(ObjectProperty {
                name,
                value,
                kind,
                span,
            }));
            if !self.eat(",") {
                break;
            }
        }
        self.expect("}")?;
        let end = self.tokens[self.index - 1].span.end;
        self.make_expr(ExprKind::Object(properties), Span::new(start, end))
    }

    fn object_property_name(&mut self, token: Token) -> Result<PropertyName, Diagnostic> {
        Ok(match token.kind {
            Kind::Word(name) => {
                PropertyName::Literal(Literal::String(JsString::from(name.as_str())))
            }
            Kind::Literal(Literal::Null) => {
                PropertyName::Literal(Literal::String(JsString::from("null")))
            }
            Kind::Literal(Literal::Boolean(value)) => {
                PropertyName::Literal(Literal::String(JsString::from(if value {
                    "true"
                } else {
                    "false"
                })))
            }
            Kind::Literal(literal) => PropertyName::Literal(literal),
            Kind::Punct("[") => {
                let key = self.expression_with_in(2, true)?;
                self.expect("]")?;
                PropertyName::Computed(Box::new(key))
            }
            _ => return Err(early(token.span, "expected an object property name")),
        })
    }

    fn object_method(&mut self, start: usize, kind: PropertyKind) -> Result<Expr, Diagnostic> {
        self.enter()?;
        let previous = self.allow_new_target;
        self.allow_new_target = true;
        let result = (|| {
            let parameters = match kind {
                PropertyKind::Getter => {
                    self.expect("(")?;
                    self.expect(")")?;
                    Rc::from([])
                }
                PropertyKind::Setter => {
                    self.expect("(")?;
                    if self.at("...") {
                        return Err(self.error("a setter cannot have a rest parameter"));
                    }
                    let parameter =
                        self.formal_parameter("invalid setter parameter identifier", true)?;
                    // PropertySetParameterList is a single FormalParameter: unlike
                    // FormalParameters, it does not permit a trailing comma.
                    self.expect(")")?;
                    Rc::from([parameter])
                }
                PropertyKind::Method => {
                    self.formal_parameters("invalid method parameter identifier")?
                }
                _ => unreachable!("method property kind"),
            };
            let body = self.function_body()?;
            let span = Span::new(start, self.tokens[self.index - 1].span.end);
            let source = FunctionSource {
                text: self.source.clone(),
                span,
            };
            self.make_expr(
                ExprKind::Function(Rc::new(Function {
                    name: None,
                    parameters,
                    body,
                    source,
                })),
                span,
            )
        })();
        self.allow_new_target = previous;
        self.depth -= 1;
        result
    }
}
