//! Array construction, identification, and length conversion (23.1, 10.4.2.4).

mod access;
mod callback;
mod find;
mod literal;
mod mutation;
mod string;

use super::Builtin;
use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{DataDescriptor, DescriptorKind, PropertyDescriptor},
    value::to_uint32,
};
use spite_core::{JsString, Span};

#[derive(Debug)]
pub(crate) struct ArrayIntrinsics {
    pub constructor: ObjectHandle,
    pub prototype: ObjectHandle,
    is_array: ObjectHandle,
    methods: Vec<ObjectHandle>,
}

impl ArrayIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [&self.constructor, &self.prototype, &self.is_array]
            .into_iter()
            .chain(self.methods.iter())
    }
}

impl Realm {
    pub(super) fn array_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<ArrayIntrinsics, Error> {
        let constructor = self.new_builtin(function_prototype, Builtin::Array, span)?;
        // 23.1.3: Array.prototype is itself an Array exotic object.
        let prototype = self.object_work(span, |objects, budget| {
            objects.create_array(Some(object_prototype), 0, budget)
        })?;
        let is_array = self.new_builtin(function_prototype, Builtin::ArrayIsArray, span)?;
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
        self.define_builtin_property(
            &prototype,
            "constructor",
            Value::Object(constructor.clone()),
            true,
            span,
        )?;
        self.define_builtin_property(
            &constructor,
            "isArray",
            Value::Object(is_array.clone()),
            true,
            span,
        )?;
        let mut methods = Vec::new();
        for builtin in [
            Builtin::ArrayJoin,
            Builtin::ArrayToString,
            Builtin::ArrayAt,
            Builtin::ArrayPush,
            Builtin::ArrayPop,
            Builtin::ArrayForEach,
            Builtin::ArrayEvery,
            Builtin::ArraySome,
            Builtin::ArrayFind,
            Builtin::ArrayFindIndex,
            Builtin::ArrayFindLast,
            Builtin::ArrayFindLastIndex,
        ] {
            let method = self.new_builtin(function_prototype, builtin, span)?;
            self.define_builtin_property(
                &prototype,
                builtin.initial_name(),
                Value::Object(method.clone()),
                true,
                span,
            )?;
            methods.push(method);
        }
        Ok(ArrayIntrinsics {
            constructor,
            prototype,
            is_array,
            methods,
        })
    }

    pub(super) fn array_constructor(
        &mut self,
        new_target: Option<ObjectHandle>,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        // 23.1.1.1: GetPrototypeFromConstructor precedes length validation.
        let target = new_target.unwrap_or_else(|| {
            self.intrinsics
                .as_ref()
                .expect("initialized")
                .array
                .constructor
                .clone()
        });
        let prototype = self.get_property(&target, &JsString::from("prototype"), span)?;
        let prototype = match prototype {
            Value::Object(prototype) => prototype,
            _ => self
                .intrinsics
                .as_ref()
                .expect("initialized")
                .array
                .prototype
                .clone(),
        };
        let count = arguments.len();
        let length = u32::try_from(if count == 1 { 0 } else { count }).map_err(|_| {
            Self::exception(ExceptionKind::RangeError, span, "invalid Array length")
        })?;
        let array = self.object_work(span, |objects, budget| {
            objects.create_array(Some(&prototype), length, budget)
        })?;
        if count == 1 {
            let value = arguments.next().expect("one argument");
            let length = if let Value::Number(number) = value {
                let length = to_uint32(number);
                if f64::from(length) != number {
                    return Err(Self::exception(
                        ExceptionKind::RangeError,
                        span,
                        "invalid Array length",
                    ));
                }
                length
            } else {
                self.create_array_element(&array, 0, value, span)?;
                1
            };
            // A fresh Array has its own writable length; this normalized Number
            // cannot invoke user code. Inherited setters are never consulted.
            self.object_work(span, |objects, budget| {
                objects.define(
                    &array,
                    JsString::from("length"),
                    DataDescriptor {
                        value: Some(Value::Number(f64::from(length))),
                        ..Default::default()
                    },
                    budget,
                )
            })?;
        } else {
            for (index, value) in arguments.enumerate() {
                self.create_array_element(&array, index as u32, value, span)?;
            }
        }
        Ok(Value::Object(array))
    }

    fn create_array_element(
        &mut self,
        array: &ObjectHandle,
        index: u32,
        value: Value,
        span: Span,
    ) -> Result<(), Error> {
        self.define_property_or_throw(
            array,
            JsString::from(index.to_string().as_str()),
            DataDescriptor {
                value: Some(value),
                writable: Some(true),
                enumerable: Some(true),
                configurable: Some(true),
            }
            .into(),
            span,
        )
    }

    pub(super) fn array_is_array(
        &mut self,
        value: Option<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        // 7.2.2 / 23.1.2.3: identity does not depend on a public prototype.
        // Proxy target traversal joins IsArray when Proxy objects are exposed.
        let result = if let Some(Value::Object(object)) = value {
            self.object_work(span, |objects, _| Ok(objects.inspect(&object)?.is_array()))?
        } else {
            false
        };
        Ok(Value::Boolean(result))
    }

    pub(crate) fn convert_array_length(
        &mut self,
        object: &ObjectHandle,
        key: &JsString,
        descriptor: &mut PropertyDescriptor,
        span: Span,
    ) -> Result<(), Error> {
        if key.code_units() != [0x6c, 0x65, 0x6e, 0x67, 0x74, 0x68] {
            return Ok(());
        }
        let DescriptorKind::Data {
            value: Some(value), ..
        } = &mut descriptor.kind
        else {
            return Ok(());
        };
        if !self.object_work(span, |objects, _| Ok(objects.inspect(object)?.is_array()))? {
            return Ok(());
        }
        self.object_work(span, |_, budget| budget.value(value))?;
        // Both conversions use the original value. They can call user code
        // twice and mutate the array; storage reads its current length only
        // after both complete. ToUint32 may produce a different first Number.
        let new_length = to_uint32(self.number(value.clone(), span)?);
        let number_length = self.number(std::mem::replace(value, Value::Undefined), span)?;
        if f64::from(new_length) != number_length {
            return Err(Self::exception(
                ExceptionKind::RangeError,
                span,
                "invalid Array length",
            ));
        }
        *value = Value::Number(f64::from(new_length));
        Ok(())
    }
}

#[cfg(test)]
mod tests;
