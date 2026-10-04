//! Boolean constructor, prototype methods, and Boolean wrapper objects (20.3).

use super::Builtin;
use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value, object::DataDescriptor};
use spite_core::{JsString, Span};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub(crate) struct BooleanIntrinsics {
    pub constructor: ObjectHandle,
    pub prototype: ObjectHandle,
    to_string: ObjectHandle,
    value_of: ObjectHandle,
}

impl BooleanIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [
            &self.constructor,
            &self.prototype,
            &self.to_string,
            &self.value_of,
        ]
        .into_iter()
    }
}

impl Realm {
    pub(super) fn boolean_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<BooleanIntrinsics, Error> {
        let constructor = self.new_builtin(function_prototype, Builtin::Boolean, span)?;
        let prototype = self.object_work(span, |objects, _| {
            objects.create_boolean(object_prototype, false)
        })?;
        let to_string = self.new_builtin(function_prototype, Builtin::BooleanToString, span)?;
        let value_of = self.new_builtin(function_prototype, Builtin::BooleanValueOf, span)?;
        self.object_work(span, |objects, budget| {
            objects.define(
                &constructor,
                JsString::from("prototype"),
                DataDescriptor {
                    value: Some(Value::Object(prototype.clone())),
                    writable: Some(false),
                    enumerable: Some(false),
                    configurable: Some(false),
                },
                budget,
            )
        })?;
        for (name, value) in [
            ("constructor", &constructor),
            ("toString", &to_string),
            ("valueOf", &value_of),
        ] {
            self.define_builtin_property(
                &prototype,
                name,
                Value::Object(value.clone()),
                true,
                span,
            )?;
        }
        Ok(BooleanIntrinsics {
            constructor,
            prototype,
            to_string,
            value_of,
        })
    }

    pub(super) fn this_boolean_value(&mut self, value: &Value, span: Span) -> Result<bool, Error> {
        let value = match value {
            Value::Boolean(value) => Some(*value),
            Value::Object(object) => self.object_work(span, |objects, _| {
                Ok(objects.inspect(object)?.boolean_data())
            })?,
            _ => None,
        };
        value.ok_or_else(|| {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "receiver does not contain a Boolean value",
            )
        })
    }
}
