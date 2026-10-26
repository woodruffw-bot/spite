//! Unary value operations shared by synchronous and resumed expression evaluation.

use crate::{Error, ExceptionKind, Realm, Reference, Value, value::to_uint32};
use spite_bigint::BigInt;
use spite_core::{JsString, Span};
use spite_parser::ast::{BinaryOp, UnaryOp, UpdateOp};

impl Realm {
    pub(crate) fn update_reference(
        &mut self,
        mut reference: Reference<'_>,
        op: UpdateOp,
        prefix: bool,
        span: Span,
        argument_span: Span,
    ) -> Result<Value, Error> {
        // ECMA-262 13.4.2–13.4.5: GetValue and ToNumeric precede
        // PutValue. Postfix returns the numeric old value, not its input.
        let old = self.get(&mut reference, argument_span)?;
        let old = self.numeric(old, argument_span)?;
        let one = if matches!(old, Value::BigInt(_)) {
            Value::BigInt(BigInt::from(1))
        } else {
            Value::Number(1.0)
        };
        let operation = match op {
            UpdateOp::Increment => BinaryOp::Add,
            UpdateOp::Decrement => BinaryOp::Subtract,
        };
        let new = self.binary(operation, old.clone(), one, span)?;
        self.put(reference, new.clone(), span)?;
        Ok(if prefix { new } else { old })
    }

    pub(crate) fn delete_reference(
        &mut self,
        reference: Reference<'_>,
        span: Span,
    ) -> Result<bool, Error> {
        let deleted = match reference {
            Reference::Value(_) => true,
            Reference::Private { .. } => {
                unreachable!("parser rejects private deletion")
            }
            Reference::SuperProperty { .. } => {
                // 13.5.1.2: fail before ToObject or ToPropertyKey.
                return Err(Self::exception(
                    ExceptionKind::ReferenceError,
                    span,
                    "cannot delete a super property",
                ));
            }
            Reference::Property { base, mut key } => {
                Self::require_object_coercible(&base, span)?;
                let key = self.reference_key(&mut key, span)?;
                let deleted = self.delete_property_value(&base, &key, span)?;
                if !deleted && self.strict {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "property is not configurable",
                    ));
                }
                deleted
            }
            Reference::Lexical(environment, name) => {
                let bindings = &mut self
                    .objects
                    .environment_mut(&environment)
                    .expect("resolved environment")
                    .bindings;
                if bindings
                    .get(name.as_ref())
                    .is_none_or(|binding| binding.deletable)
                {
                    bindings.remove(name.as_ref());
                    true
                } else {
                    false
                }
            }
            Reference::ObjectBinding { object, name } => self.delete_property_value(
                &Value::Object(object),
                &JsString::from(name.as_ref()),
                span,
            )?,
            Reference::Global { key, .. } => {
                let object = Value::Object(self.global_object());
                self.delete_property_value(&object, &key, span)?
            }
            Reference::Unresolvable(_) => true,
            Reference::UnsupportedGlobal(name) => {
                return Err(Self::unsupported(
                    span,
                    format!("{name} is not implemented"),
                ));
            }
        };
        Ok(deleted)
    }

    pub(crate) fn unary_value(
        &mut self,
        op: UnaryOp,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        Ok(match op {
            UnaryOp::Plus => Value::Number(self.number(value, span)?),
            UnaryOp::Minus => match self.numeric(value, span)? {
                Value::BigInt(value) => {
                    Value::BigInt(self.integer_work(span, |budget| value.neg(budget))?)
                }
                Value::Number(value) => Value::Number(-value),
                _ => unreachable!("ToNumeric returns a numeric value"),
            },
            UnaryOp::Not => Value::Boolean(!value.to_boolean()),
            UnaryOp::BitNot => match self.numeric(value, span)? {
                Value::BigInt(value) => {
                    Value::BigInt(self.integer_work(span, |budget| value.bitnot(budget))?)
                }
                Value::Number(value) => Value::Number((!(to_uint32(value) as i32)) as f64),
                _ => unreachable!("ToNumeric returns a numeric value"),
            },
            UnaryOp::Void => Value::Undefined,
            UnaryOp::Typeof => Value::String(JsString::from(if self.is_callable(&value, span)? {
                "function"
            } else {
                value.type_name()
            })),
            UnaryOp::Delete => Value::Boolean(true),
        })
    }
}
