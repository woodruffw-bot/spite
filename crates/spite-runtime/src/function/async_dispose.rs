//! Native asynchronous DisposeResources continuations (sec-disposeresources).

use super::PromiseCapability;
use crate::{
    Error, ObjectHandle, Realm, Value,
    object::{DisposableResource, DisposableStackKind, DisposalHint},
};
use spite_core::{JsString, Span};

impl Realm {
    fn intrinsic_promise_capability(&mut self, span: Span) -> Result<PromiseCapability, Error> {
        let constructor = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .global_metadata
            .promise
            .clone();
        self.new_promise_capability(Value::Object(constructor), span)
    }

    pub(super) fn async_stack_dispose(&mut self, this: Value, span: Span) -> Result<Value, Error> {
        // This built-in is async: even receiver validation rejects a fresh
        // intrinsic Promise, whereas host failures bypass JavaScript rejection.
        let capability = self.intrinsic_promise_capability(span)?;
        let result = capability.promise.clone();
        let completion = (|| {
            let Value::Object(stack) = this else {
                return Err(Self::disposable_stack_brand_error(span));
            };
            let Some(disposed) = self.object_work(span, |objects, _| {
                objects.disposable_stack_state(&stack, DisposableStackKind::Async)
            })?
            else {
                return Err(Self::disposable_stack_brand_error(span));
            };
            if disposed {
                self.call(
                    capability.resolve.clone(),
                    Value::Undefined,
                    vec![Value::Undefined],
                    span,
                )?;
                return Ok(());
            }
            let state = self.object_work(span, |objects, budget| {
                objects.create_async_disposal(&stack, capability.clone(), budget)
            })?;
            self.continue_async_disposal(&state, span)
        })();
        if let Err(error) = completion {
            let reason = self.language_exception_value(error)?;
            self.call(capability.reject, Value::Undefined, vec![reason], span)?;
        }
        Ok(result)
    }

    fn record_async_disposal_error(
        &mut self,
        state: &ObjectHandle,
        error: Value,
        span: Span,
    ) -> Result<(), Error> {
        let previous = self.object_work(span, |objects, _| {
            Ok(objects.async_disposal_mut(state)?.error.take())
        })?;
        let error = if let Some(suppressed) = previous {
            self.create_intrinsic_suppressed_error(error, suppressed, span)?
        } else {
            error
        };
        self.object_work(span, |objects, budget| {
            budget.value(&error)?;
            if let Value::Object(object) = &error {
                objects.inspect(object)?;
            }
            objects.async_disposal_mut(state)?.error = Some(error);
            Ok(())
        })
    }

    fn call_async_disposal_resource(
        &mut self,
        resource: DisposableResource,
        span: Span,
    ) -> Result<Value, Error> {
        let arguments = resource.argument.into_iter().collect();
        if resource.hint != DisposalHint::AsyncFromSync {
            return self.call(resource.method, resource.value, arguments, span);
        }
        // GetDisposeMethod's unobservable sync fallback creates a Promise
        // before calling the disposer. Its return value is deliberately ignored.
        let capability = self.intrinsic_promise_capability(span)?;
        match self.call(resource.method, resource.value, arguments, span) {
            Ok(_) => {
                self.call(
                    capability.resolve,
                    Value::Undefined,
                    vec![Value::Undefined],
                    span,
                )?;
            }
            Err(error) => {
                let reason = self.language_exception_value(error)?;
                self.call(capability.reject, Value::Undefined, vec![reason], span)?;
            }
        }
        Ok(capability.promise)
    }

    // Returns true when a reaction owns the continuation; false when
    // PromiseResolve itself threw before suspension and the loop can continue.
    fn await_async_disposal(
        &mut self,
        state: &ObjectHandle,
        value: Value,
        span: Span,
    ) -> Result<bool, Error> {
        self.object_work(span, |objects, _| {
            objects.async_disposal_mut(state)?.has_awaited = true;
            Ok(())
        })?;
        let constructor = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .global_metadata
            .promise
            .clone();
        let promise = match self.promise_resolve(Value::Object(constructor), value, span) {
            Ok(Value::Object(promise)) => promise,
            Ok(_) => unreachable!("intrinsic PromiseResolve returns a Promise"),
            Err(error) => {
                let reason = self.language_exception_value(error)?;
                self.record_async_disposal_error(state, reason, span)?;
                return Ok(false);
            }
        };
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .function_prototype
            .clone();
        let mut handlers = [Value::Undefined, Value::Undefined];
        for (slot, fulfilled) in handlers.iter_mut().zip([true, false]) {
            let handler = self.object_work(span, |objects, _| {
                objects.create_async_dispose_resume(&prototype, state, fulfilled)
            })?;
            self.define_builtin_property(
                &handler,
                "name",
                Value::String(JsString::from("")),
                false,
                span,
            )?;
            self.define_builtin_property(&handler, "length", Value::Number(1.0), false, span)?;
            *slot = Value::Object(handler);
        }
        let [fulfilled, rejected] = handlers;
        self.perform_promise_then(&promise, fulfilled, rejected, None, span)?;
        Ok(true)
    }

    fn continue_async_disposal(&mut self, state: &ObjectHandle, span: Span) -> Result<(), Error> {
        loop {
            let resource = self.object_work(span, |objects, _| {
                Ok(objects.async_disposal_mut(state)?.resources.pop())
            })?;
            if let Some(resource) = resource {
                debug_assert_ne!(resource.hint, DisposalHint::Sync);
                if matches!(resource.method, Value::Undefined) {
                    self.object_work(span, |objects, _| {
                        objects.async_disposal_mut(state)?.needs_await = true;
                        Ok(())
                    })?;
                    continue;
                }
                match self.call_async_disposal_resource(resource, span) {
                    Ok(value) => {
                        if self.await_async_disposal(state, value, span)? {
                            return Ok(());
                        }
                    }
                    Err(error) => {
                        let reason = self.language_exception_value(error)?;
                        self.record_async_disposal_error(state, reason, span)?;
                    }
                }
                continue;
            }
            let needs_await = self.object_work(span, |objects, _| {
                let data = objects.async_disposal_mut(state)?;
                Ok(data.needs_await && !data.has_awaited)
            })?;
            if needs_await && self.await_async_disposal(state, Value::Undefined, span)? {
                return Ok(());
            }
            let (capability, error) =
                self.object_work(span, |objects, _| objects.finish_async_disposal(state))?;
            let (method, value) = if let Some(error) = error {
                (capability.reject, error)
            } else {
                (capability.resolve, Value::Undefined)
            };
            self.call(method, Value::Undefined, vec![value], span)?;
            return Ok(());
        }
    }

    pub(super) fn resume_async_disposal(
        &mut self,
        state: ObjectHandle,
        fulfilled: bool,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        if !fulfilled {
            self.record_async_disposal_error(&state, value, span)?;
        }
        self.continue_async_disposal(&state, span)?;
        Ok(Value::Undefined)
    }
}
