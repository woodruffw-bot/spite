//! Promise.all ordered aggregation and guarded resolve elements (27.2.4.1).

use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value};
use spite_core::{JsString, Span, WellKnownSymbol};

impl Realm {
    pub(in crate::function) fn promise_all(
        &mut self,
        constructor: Value,
        iterable: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let capability = self.new_promise_capability(constructor.clone(), span)?;
        let result = (|| -> Result<(), Error> {
            let resolve =
                self.get_property_value(&constructor, &JsString::from("resolve"), span)?;
            if !self.is_callable(&resolve, span)? {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "Promise constructor resolve is not callable",
                ));
            }
            let method = self
                .get_method(&iterable, &WellKnownSymbol::Iterator.symbol(), span)?
                .ok_or_else(|| {
                    Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "Promise.all input is not iterable",
                    )
                })?;
            let mut iterator = self.get_iterator_from_method(iterable, method, span)?;
            let state = self.object_work(span, |objects, _| {
                objects.create_promise_all_state(capability.clone())
            })?;
            loop {
                self.tick(span)?;
                let Some(value) = self.iterator_step_value(&mut iterator, span)? else {
                    let ready = self.object_work(span, |objects, budget| {
                        objects.finish_promise_all_remaining(&state, budget)
                    })?;
                    if let Some((resolve, len)) = ready {
                        self.resolve_promise_all_list(&state, resolve, len, span)?;
                    }
                    return Ok(());
                };
                let subscribe = (|| {
                    let index = self.object_work(span, |objects, budget| {
                        objects.append_promise_all_slot(&state, budget)
                    })?;
                    let promise =
                        self.call(resolve.clone(), constructor.clone(), vec![value], span)?;
                    let prototype = self
                        .intrinsics
                        .as_ref()
                        .expect("initialized")
                        .function_prototype
                        .clone();
                    let function = self.object_work(span, |objects, budget| {
                        objects
                            .create_promise_all_resolve_element(&prototype, &state, index, budget)
                    })?;
                    self.define_builtin_property(
                        &function,
                        "length",
                        Value::Number(1.0),
                        false,
                        span,
                    )?;
                    self.define_builtin_property(
                        &function,
                        "name",
                        Value::String(JsString::from("")),
                        false,
                        span,
                    )?;
                    let then = self.get_property_value(&promise, &JsString::from("then"), span)?;
                    self.call(
                        then,
                        promise,
                        vec![Value::Object(function), capability.reject.clone()],
                        span,
                    )
                })();
                if let Err(error) = subscribe {
                    return Err(self.iterator_close_error(&iterator, error, span));
                }
            }
        })();
        if let Err(error) = result {
            let reason = self.language_exception_value(error)?;
            self.call(capability.reject, Value::Undefined, vec![reason], span)?;
        }
        Ok(capability.promise)
    }

    pub(in crate::function) fn promise_all_resolve_element(
        &mut self,
        function: ObjectHandle,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let ready = self.object_work(span, |objects, budget| {
            objects.fulfill_promise_all_element(&function, value, budget)
        })?;
        if let Some((state, resolve, len)) = ready {
            self.resolve_promise_all_list(&state, resolve, len, span)
        } else {
            Ok(Value::Undefined)
        }
    }

    fn resolve_promise_all_list(
        &mut self,
        state: &ObjectHandle,
        resolve: Value,
        len: usize,
        span: Span,
    ) -> Result<Value, Error> {
        // Internal lists never consult Array.prototype or the global Array.
        // Copy one element at a time instead of allocating a temporary Vec.
        let array = self.create_intrinsic_array(len as u64, span)?;
        for index in 0..len {
            let value = self.object_work(span, |objects, budget| {
                objects.promise_all_value(state, index, budget)
            })?;
            self.create_array_element(&array, index as u64, value, span)?;
        }
        self.call(resolve, Value::Undefined, vec![Value::Object(array)], span)
    }
}
