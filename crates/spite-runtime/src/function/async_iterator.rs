//! Async iterator prototype identity and awaited disposal (27.1.4).

use super::Builtin;
use crate::{Error, ObjectHandle, Realm, Value};
use spite_core::{JsString, Span, WellKnownSymbol};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub(crate) struct AsyncIteratorIntrinsics {
    pub prototype: ObjectHandle,
    identity: ObjectHandle,
    dispose: ObjectHandle,
}

impl AsyncIteratorIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [&self.prototype, &self.identity, &self.dispose].into_iter()
    }
}

impl Realm {
    pub(super) fn async_iterator_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<AsyncIteratorIntrinsics, Error> {
        let prototype =
            self.object_work(span, |objects, _| objects.create(Some(object_prototype)))?;
        let identity =
            self.new_builtin(function_prototype, Builtin::AsyncIteratorIdentity, span)?;
        let dispose = self.new_builtin(function_prototype, Builtin::AsyncIteratorDispose, span)?;
        for (symbol, value) in [
            (WellKnownSymbol::AsyncIterator, &identity),
            (WellKnownSymbol::AsyncDispose, &dispose),
        ] {
            self.define_property_or_throw(
                &prototype,
                symbol.symbol(),
                crate::object::DataDescriptor {
                    value: Some(Value::Object(value.clone())),
                    writable: Some(true),
                    enumerable: Some(false),
                    configurable: Some(true),
                }
                .into(),
                span,
            )?;
        }
        Ok(AsyncIteratorIntrinsics {
            prototype,
            identity,
            dispose,
        })
    }

    pub(super) fn async_iterator_dispose(
        &mut self,
        this: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let capability = self.intrinsic_promise_capability(span)?;
        let result = capability.promise.clone();
        let completion = (|| {
            let Some(method) = self.get_method(&this, &JsString::from("return"), span)? else {
                self.call(
                    capability.resolve.clone(),
                    Value::Undefined,
                    vec![Value::Undefined],
                    span,
                )?;
                return Ok(());
            };
            let value = self.call(method, this, Vec::new(), span)?;
            let constructor = self
                .intrinsics
                .as_ref()
                .expect("initialized")
                .global_metadata
                .promise
                .clone();
            let promise = self.promise_resolve(Value::Object(constructor), value, span)?;
            let Value::Object(promise) = promise else {
                unreachable!("intrinsic PromiseResolve returns a Promise");
            };
            let prototype = self
                .intrinsics
                .as_ref()
                .expect("initialized")
                .function_prototype
                .clone();
            let fulfilled =
                self.new_builtin(&prototype, Builtin::AsyncIteratorDisposeFulfilled, span)?;
            self.perform_promise_then(
                &promise,
                Value::Object(fulfilled),
                Value::Undefined,
                Some(capability.clone()),
                span,
            )?;
            Ok(())
        })();
        if let Err(error) = completion {
            let reason = self.language_exception_value(error)?;
            self.call(capability.reject, Value::Undefined, vec![reason], span)?;
        }
        Ok(result)
    }
}
