//! ToObject and prototype selection for the exposed primitive wrapper kinds.

use crate::{Error, ObjectHandle, Realm, Value};
use spite_core::Span;

#[cfg(test)]
mod tests;

impl Realm {
    pub(crate) fn primitive_prototype(&self, value: &Value) -> Option<ObjectHandle> {
        let intrinsics = self.intrinsics.as_ref().expect("initialized realm");
        match value {
            Value::Boolean(_) => Some(intrinsics.boolean.prototype.clone()),
            Value::Number(_) => Some(intrinsics.number.prototype.clone()),
            _ => None,
        }
    }

    pub(crate) fn box_primitive(&mut self, value: Value, span: Span) -> Result<Value, Error> {
        Self::require_object_coercible(&value, span)?;
        if matches!(value, Value::Object(_)) {
            return Ok(value);
        }
        let Some(prototype) = self.primitive_prototype(&value) else {
            return Err(Self::unsupported(
                span,
                "this primitive wrapper type is not implemented",
            ));
        };
        self.object_work(span, |objects, _| match value {
            Value::Boolean(value) => objects.create_boolean(&prototype, value),
            Value::Number(value) => objects.create_number(&prototype, value),
            _ => unreachable!("primitive with an implemented prototype"),
        })
        .map(Value::Object)
    }
}
