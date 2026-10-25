//! Async iterator prototype identity and awaited disposal (27.1.4).

use super::Builtin;
use crate::{Error, ObjectHandle, Realm, Value};
use spite_core::{JsString, Span, WellKnownSymbol};

#[cfg(test)]
mod tests;

mod from_sync;
pub(crate) use from_sync::Member as AsyncFromSyncMember;

#[derive(Debug)]
pub(crate) struct AsyncIteratorIntrinsics {
    pub prototype: ObjectHandle,
    identity: ObjectHandle,
    dispose: ObjectHandle,
    from_sync_prototype: ObjectHandle,
    from_sync_methods: [ObjectHandle; 3],
}

impl AsyncIteratorIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [
            &self.prototype,
            &self.identity,
            &self.dispose,
            &self.from_sync_prototype,
        ]
        .into_iter()
        .chain(&self.from_sync_methods)
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
        let from_sync_prototype =
            self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
        let mut methods = [None, None, None];
        for (slot, member) in methods.iter_mut().zip([
            AsyncFromSyncMember::Next,
            AsyncFromSyncMember::Return,
            AsyncFromSyncMember::Throw,
        ]) {
            let method = self.new_builtin(
                function_prototype,
                Builtin::AsyncFromSyncMember(member),
                span,
            )?;
            self.define_builtin_property(
                &from_sync_prototype,
                member.name(),
                Value::Object(method.clone()),
                true,
                span,
            )?;
            *slot = Some(method);
        }
        Ok(AsyncIteratorIntrinsics {
            prototype,
            identity,
            dispose,
            from_sync_prototype,
            from_sync_methods: methods.map(|method| method.expect("from-sync method")),
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
