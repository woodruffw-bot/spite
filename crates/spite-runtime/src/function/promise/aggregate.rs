//! Promise.all, allSettled and any aggregation with shared per-input guards.

use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value};
use spite_core::{JsString, Span, WellKnownSymbol};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PromiseAggregateKind {
    All,
    AllSettled,
    Any,
}

impl Realm {
    pub(in crate::function) fn promise_aggregate(
        &mut self,
        kind: PromiseAggregateKind,
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
                        "Promise input is not iterable",
                    )
                })?;
            let mut iterator = self.get_iterator_from_method(iterable, method, span)?;
            let state = self.object_work(span, |objects, _| {
                objects.create_promise_aggregate_state(capability.clone(), kind)
            })?;
            loop {
                self.tick(span)?;
                let Some(value) = self.iterator_step_value(&mut iterator, span)? else {
                    let ready = self.object_work(span, |objects, budget| {
                        objects.finish_promise_aggregate_remaining(&state, budget)
                    })?;
                    if let Some((resolve, len)) = ready {
                        self.settle_promise_aggregate_list(&state, resolve, len, kind, span)?;
                    }
                    return Ok(());
                };
                let subscribe = (|| {
                    let index = self.object_work(span, |objects, budget| {
                        objects.append_promise_aggregate_slot(&state, budget)
                    })?;
                    let promise =
                        self.call(resolve.clone(), constructor.clone(), vec![value], span)?;
                    let fulfilled = if kind == PromiseAggregateKind::Any {
                        capability.resolve.clone()
                    } else {
                        self.create_promise_aggregate_callback(&state, index, true, span)?
                    };
                    let rejected = if kind != PromiseAggregateKind::All {
                        self.create_promise_aggregate_callback(&state, index, false, span)?
                    } else {
                        capability.reject.clone()
                    };
                    self.object_work(span, |objects, budget| {
                        objects.increment_promise_aggregate_remaining(&state, budget)
                    })?;
                    let then = self.get_property_value(&promise, &JsString::from("then"), span)?;
                    self.call(then, promise, vec![fulfilled, rejected], span)
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

    fn create_promise_aggregate_callback(
        &mut self,
        state: &ObjectHandle,
        index: usize,
        fulfilled: bool,
        span: Span,
    ) -> Result<Value, Error> {
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .function_prototype
            .clone();
        let function = self.object_work(span, |objects, _| {
            objects.create_promise_aggregate_element(&prototype, state, index, fulfilled)
        })?;
        self.define_builtin_property(&function, "length", Value::Number(1.0), false, span)?;
        self.define_builtin_property(
            &function,
            "name",
            Value::String(JsString::from("")),
            false,
            span,
        )?;
        Ok(Value::Object(function))
    }

    pub(in crate::function) fn promise_aggregate_element(
        &mut self,
        function: ObjectHandle,
        fulfilled: bool,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let started = self.object_work(span, |objects, budget| {
            objects.begin_promise_aggregate_element(&function, budget)
        })?;
        let Some((state, index, kind)) = started else {
            return Ok(Value::Undefined);
        };
        let value = if kind == PromiseAggregateKind::AllSettled {
            self.promise_settlement_snapshot(fulfilled, value, span)?
        } else {
            value
        };
        let ready = self.object_work(span, |objects, budget| {
            objects.store_promise_aggregate_element(&state, index, value, budget)
        })?;
        if let Some((resolve, len)) = ready {
            self.settle_promise_aggregate_list(&state, resolve, len, kind, span)
        } else {
            Ok(Value::Undefined)
        }
    }

    fn promise_settlement_snapshot(
        &mut self,
        fulfilled: bool,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .object_prototype
            .clone();
        let object = self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
        for (name, value) in [
            (
                "status",
                Value::String(JsString::from(if fulfilled {
                    "fulfilled"
                } else {
                    "rejected"
                })),
            ),
            (if fulfilled { "value" } else { "reason" }, value),
        ] {
            self.define_property_or_throw(
                &object,
                JsString::from(name),
                crate::object::DataDescriptor {
                    value: Some(value),
                    writable: Some(true),
                    enumerable: Some(true),
                    configurable: Some(true),
                }
                .into(),
                span,
            )?;
        }
        Ok(Value::Object(object))
    }

    fn settle_promise_aggregate_list(
        &mut self,
        state: &ObjectHandle,
        callback: Value,
        len: usize,
        kind: PromiseAggregateKind,
        span: Span,
    ) -> Result<Value, Error> {
        let error = if kind == PromiseAggregateKind::Any {
            Some(self.create_intrinsic_aggregate_error(span)?)
        } else {
            None
        };
        // Internal lists never consult Array.prototype or the global Array.
        // Copy one element at a time instead of allocating a temporary Vec.
        let array = self.create_intrinsic_array(len as u64, span)?;
        for index in 0..len {
            let value = self.object_work(span, |objects, budget| {
                objects.promise_aggregate_value(state, index, budget)
            })?;
            self.create_array_element(&array, index as u64, value, span)?;
        }
        let result = if let Some(error) = error {
            self.define_builtin_property(&error, "errors", Value::Object(array), true, span)?;
            Value::Object(error)
        } else {
            Value::Object(array)
        };
        self.call(callback, Value::Undefined, vec![result], span)
    }
}
