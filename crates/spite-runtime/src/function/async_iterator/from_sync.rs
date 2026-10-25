//! Async-from-Sync method forwarding and awaited IteratorResult values (27.1.4.5).

use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    function::{PromiseCapability, iterator::IteratorRecord},
};
use spite_core::{JsString, Span};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Member {
    Next,
    Return,
    Throw,
}

impl Member {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Next => "next",
            Self::Return => "return",
            Self::Throw => "throw",
        }
    }
}

impl Realm {
    pub(crate) fn create_async_from_sync_iterator(
        &mut self,
        record: IteratorRecord,
        span: Span,
    ) -> Result<IteratorRecord, Error> {
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .async_iterator
            .from_sync_prototype
            .clone();
        let state = record.into_wrapper();
        let wrapper = self.object_work(span, |objects, budget| {
            budget.value(&state.next)?;
            objects.create_async_from_sync_iterator(&prototype, state)
        })?;
        self.get_iterator_direct(wrapper, span)
    }

    pub(in crate::function) fn async_from_sync_method(
        &mut self,
        member: Member,
        this: Value,
        value: Option<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let capability = self.intrinsic_promise_capability(span)?;
        let result = capability.promise.clone();
        let completion = (|| {
            let Value::Object(wrapper) = this else {
                return Err(Self::async_from_sync_brand_error(span));
            };
            let (iterator, next) = self
                .object_work(span, |objects, budget| {
                    let Some(state) = objects.inspect(&wrapper)?.async_from_sync_iterator() else {
                        return Ok(None);
                    };
                    let next = if member == Member::Next {
                        budget.value(&state.next)?;
                        Some(state.next.clone())
                    } else {
                        None
                    };
                    Ok(Some((state.iterator.clone(), next)))
                })?
                .ok_or_else(|| Self::async_from_sync_brand_error(span))?;
            let receiver = Value::Object(iterator.clone());
            let method = if member == Member::Next {
                next.expect("cached next")
            } else if let Some(method) =
                self.get_method(&receiver, &JsString::from(member.name()), span)?
            {
                method
            } else if member == Member::Return {
                // Missing return does not await an argument embedded in the result.
                let result = self.iterator_result(value.unwrap_or(Value::Undefined), true, span)?;
                self.call(
                    capability.resolve.clone(),
                    Value::Undefined,
                    vec![result],
                    span,
                )?;
                return Ok(());
            } else {
                self.iterator_close_direct(iterator, span)?;
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "synchronous iterator has no throw method",
                ));
            };
            let result = self.call(method, receiver, value.into_iter().collect(), span)?;
            let Value::Object(result) = result else {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "synchronous iterator result is not an object",
                ));
            };
            self.async_from_sync_continuation(
                result,
                capability.clone(),
                iterator,
                member != Member::Return,
                span,
            )
        })();
        if let Err(error) = completion {
            let reason = self.language_exception_value(error)?;
            self.call(capability.reject, Value::Undefined, vec![reason], span)?;
        }
        Ok(result)
    }

    fn async_from_sync_continuation(
        &mut self,
        result: ObjectHandle,
        capability: PromiseCapability,
        iterator: ObjectHandle,
        close_on_rejection: bool,
        span: Span,
    ) -> Result<(), Error> {
        let done = self
            .get_property(&result, &JsString::from("done"), span)?
            .to_boolean();
        let value = self.get_property(&result, &JsString::from("value"), span)?;
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
                return Err(if !done && close_on_rejection {
                    self.iterator_close_error(&IteratorRecord::uninitialized(iterator), error, span)
                } else {
                    error
                });
            }
        };
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .function_prototype
            .clone();
        let fulfilled = self.object_work(span, |objects, _| {
            objects.create_async_from_sync_unwrap(&prototype, done)
        })?;
        let rejected = if !done && close_on_rejection {
            Some(self.object_work(span, |objects, _| {
                objects.create_async_from_sync_close(&prototype, &iterator)
            })?)
        } else {
            None
        };
        for callback in std::iter::once(&fulfilled).chain(&rejected) {
            self.define_builtin_property(
                callback,
                "name",
                Value::String(JsString::from("")),
                false,
                span,
            )?;
            self.define_builtin_property(callback, "length", Value::Number(1.0), false, span)?;
        }
        self.perform_promise_then(
            &promise,
            Value::Object(fulfilled),
            rejected.map_or(Value::Undefined, Value::Object),
            Some(capability),
            span,
        )?;
        Ok(())
    }

    fn async_from_sync_brand_error(span: Span) -> Error {
        Self::exception(
            ExceptionKind::TypeError,
            span,
            "receiver is not an Async-from-Sync iterator",
        )
    }
}

#[cfg(test)]
mod tests;
