//! Initial iterator prototype graph and IteratorResult creation (27.1, 7.4.16).

use super::Builtin;
use crate::{Error, ObjectHandle, Realm, Value, object::DataDescriptor};
use spite_core::{JsString, Span, WellKnownSymbol};

#[derive(Debug)]
pub(crate) struct IteratorIntrinsics {
    pub prototype: ObjectHandle,
    pub array_prototype: ObjectHandle,
    identity: ObjectHandle,
    array_next: ObjectHandle,
}

impl IteratorIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [
            &self.prototype,
            &self.array_prototype,
            &self.identity,
            &self.array_next,
        ]
        .into_iter()
    }
}

impl Realm {
    pub(super) fn iterator_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<IteratorIntrinsics, Error> {
        let prototype =
            self.object_work(span, |objects, _| objects.create(Some(object_prototype)))?;
        let array_prototype =
            self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
        let identity = self.new_builtin(function_prototype, Builtin::IteratorIdentity, span)?;
        let array_next = self.new_builtin(function_prototype, Builtin::ArrayIteratorNext, span)?;
        self.define_builtin_property(
            &array_prototype,
            "next",
            Value::Object(array_next.clone()),
            true,
            span,
        )?;
        self.object_work(span, |objects, budget| {
            objects.define(
                &prototype,
                WellKnownSymbol::Iterator.symbol(),
                DataDescriptor {
                    value: Some(Value::Object(identity.clone())),
                    writable: Some(true),
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                budget,
            )?;
            objects.define(
                &array_prototype,
                WellKnownSymbol::ToStringTag.symbol(),
                DataDescriptor {
                    value: Some(Value::String(JsString::from("Array Iterator"))),
                    writable: Some(false),
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                budget,
            )?;
            Ok(())
        })?;
        Ok(IteratorIntrinsics {
            prototype,
            array_prototype,
            identity,
            array_next,
        })
    }

    pub(crate) fn iterator_result(
        &mut self,
        value: Value,
        done: bool,
        span: Span,
    ) -> Result<Value, Error> {
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .object_prototype
            .clone();
        self.object_work(span, |objects, budget| {
            let result = objects.create(Some(&prototype))?;
            for (key, value) in [("value", value), ("done", Value::Boolean(done))] {
                objects.define(
                    &result,
                    key,
                    DataDescriptor {
                        value: Some(value),
                        writable: Some(true),
                        enumerable: Some(true),
                        configurable: Some(true),
                    },
                    budget,
                )?;
            }
            Ok(Value::Object(result))
        })
    }
}
