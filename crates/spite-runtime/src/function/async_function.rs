//! Async function intrinsics and Promise body completion (27.7, 15.8).

use super::{Builtin, PromiseCapability, async_execution::AsyncBodyOutcome};
use crate::{Error, ObjectHandle, Realm, Value, object::DataDescriptor};
use spite_core::{JsString, Span, WellKnownSymbol};

#[derive(Debug)]
pub(crate) struct AsyncFunctionIntrinsics {
    pub constructor: ObjectHandle,
    pub prototype: ObjectHandle,
}

impl AsyncFunctionIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [&self.constructor, &self.prototype].into_iter()
    }
}

impl Realm {
    pub(super) fn async_function_intrinsics(
        &mut self,
        function_constructor: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<AsyncFunctionIntrinsics, Error> {
        let constructor = self.new_builtin(function_constructor, Builtin::AsyncFunction, span)?;
        let prototype =
            self.object_work(span, |objects, _| objects.create(Some(function_prototype)))?;
        self.define_property_or_throw(
            &constructor,
            JsString::from("prototype"),
            DataDescriptor {
                value: Some(Value::Object(prototype.clone())),
                writable: Some(false),
                enumerable: Some(false),
                configurable: Some(false),
            }
            .into(),
            span,
        )?;
        self.define_builtin_property(
            &prototype,
            "constructor",
            Value::Object(constructor.clone()),
            false,
            span,
        )?;
        self.define_property_or_throw(
            &prototype,
            WellKnownSymbol::ToStringTag.symbol(),
            DataDescriptor {
                value: Some(Value::String(JsString::from("AsyncFunction"))),
                writable: Some(false),
                enumerable: Some(false),
                configurable: Some(true),
            }
            .into(),
            span,
        )?;
        Ok(AsyncFunctionIntrinsics {
            constructor,
            prototype,
        })
    }

    pub(super) fn call_async_body(
        &mut self,
        span: Span,
        execute: impl FnOnce(&mut Self, &PromiseCapability, usize) -> Result<AsyncBodyOutcome, Error>,
    ) -> Result<Value, Error> {
        // Async function/concise-body evaluation creates a fresh Promise before
        // declaration instantiation. Public Promise bindings/methods are unused.
        let capability = self.intrinsic_promise_capability(span)?;
        let base = self.scopes.len();
        match execute(self, &capability, base) {
            Ok(AsyncBodyOutcome::Yielded(_)) => unreachable!("async body cannot yield"),
            Ok(AsyncBodyOutcome::Suspended) => (),
            Ok(AsyncBodyOutcome::Complete(value)) => {
                self.call(capability.resolve, Value::Undefined, vec![value], span)?;
            }
            Err(error) => {
                let reason = self.language_exception_value(error)?;
                self.call(capability.reject, Value::Undefined, vec![reason], span)?;
            }
        }
        Ok(capability.promise)
    }
}
