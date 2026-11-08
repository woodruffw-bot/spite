//! ToObject and prototype selection for the exposed primitive wrapper kinds.

use crate::{Error, ObjectHandle, Realm, Value};
use spite_core::Span;

#[cfg(test)]
mod tests;

impl Realm {
    pub(crate) fn primitive_prototype(&self, value: &Value) -> Option<ObjectHandle> {
        let intrinsics = self.record.intrinsics.as_ref().expect("initialized realm");
        match value {
            Value::Boolean(_) => Some(intrinsics.boolean.prototype.clone()),
            Value::Number(_) => Some(intrinsics.number.prototype.clone()),
            Value::String(_) => Some(intrinsics.string.prototype.clone()),
            Value::Symbol(_) => Some(intrinsics.symbol.prototype.clone()),
            Value::BigInt(_) => Some(intrinsics.bigint.prototype.clone()),
            Value::Undefined | Value::Null | Value::Object(_) => None,
        }
    }

    pub(crate) fn box_primitive(&mut self, value: Value, span: Span) -> Result<Value, Error> {
        Self::require_object_coercible(&value, span)?;
        if matches!(value, Value::Object(_)) {
            return Ok(value);
        }
        let prototype = self
            .primitive_prototype(&value)
            .expect("coercible primitive has an intrinsic prototype");
        self.object_work(span, |objects, budget| match value {
            Value::Boolean(value) => objects.create_boolean(&prototype, value),
            Value::Number(value) => objects.create_number(&prototype, value),
            Value::String(value) => objects.create_string(&prototype, value, budget),
            Value::Symbol(value) => objects.create_symbol(&prototype, value),
            Value::BigInt(value) => objects.create_bigint(&prototype, value),
            Value::Undefined | Value::Null | Value::Object(_) => {
                unreachable!("coercible primitive after object fast path")
            }
        })
        .map(Value::Object)
    }
}
