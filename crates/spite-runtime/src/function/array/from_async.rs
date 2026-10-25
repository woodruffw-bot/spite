//! Array.fromAsync's sequential awaits and AsyncIteratorClose (23.1.2.2).

use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    function::async_native::AsyncResumeKind,
    object::array_from_async::{AsyncArrayData, AsyncArrayInput, AsyncArrayPhase},
};
use spite_core::{JsString, Span, WellKnownSymbol};

enum Step {
    Await(AsyncArrayPhase, Value),
    Advance,
    Finish(Value),
}

impl Realm {
    pub(crate) fn array_from_async(
        &mut self,
        receiver: Value,
        mut items: Value,
        mapper: Value,
        this_arg: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let capability = self.intrinsic_promise_capability(span)?;
        let promise = capability.promise.clone();
        let completion = (|| {
            if !matches!(mapper, Value::Undefined) && !self.is_callable(&mapper, span)? {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "Array mapper is not callable",
                ));
            }
            let async_method =
                self.get_method(&items, &WellKnownSymbol::AsyncIterator.symbol(), span)?;
            let iterator = if let Some(method) = async_method {
                Some(self.get_iterator_from_method(
                    std::mem::replace(&mut items, Value::Undefined),
                    method,
                    span,
                )?)
            } else if let Some(method) =
                self.get_method(&items, &WellKnownSymbol::Iterator.symbol(), span)?
            {
                let record = self.get_iterator_from_method(
                    std::mem::replace(&mut items, Value::Undefined),
                    method,
                    span,
                )?;
                Some(self.create_async_from_sync_iterator(record, span)?)
            } else {
                None
            };
            // Acquire and cache next before construction, unlike Array.from.
            let (input, length) = if let Some(iterator) = iterator {
                (AsyncArrayInput::Iterator(iterator.into_wrapper()), None)
            } else {
                let Value::Object(object) = self.box_primitive(items, span)? else {
                    unreachable!("ToObject");
                };
                let length = self.length_of_array_like(&object, span)?;
                (AsyncArrayInput::ArrayLike { object, length }, Some(length))
            };
            let array = self.array_from_result(receiver, length, span)?;
            let data = AsyncArrayData {
                capability: capability.clone(),
                array,
                input,
                index: 0,
                mapper,
                this_arg,
                phase: AsyncArrayPhase::Next,
                error: None,
            };
            let state = self.object_work(span, |objects, budget| {
                objects.create_array_from_async(data, budget)
            })?;
            self.drive_array_from_async(&state, None, span)
        })();
        if let Err(error) = completion {
            let reason = self.language_exception_value(error)?;
            self.call(capability.reject, Value::Undefined, vec![reason], span)?;
        }
        Ok(promise)
    }

    fn drive_array_from_async(
        &mut self,
        state: &ObjectHandle,
        mut resumed: Option<Result<Value, Error>>,
        span: Span,
    ) -> Result<(), Error> {
        loop {
            let step = if let Some(completion) = resumed.take() {
                self.resume_array_from_async_step(state, completion, span)
            } else {
                self.advance_array_from_async(state, span)
            };
            let step = match step {
                Ok(step) => step,
                Err(error) => return self.finish_array_from_async(state, Err(error), span),
            };
            match step {
                Step::Advance => (),
                Step::Finish(value) => return self.finish_array_from_async(state, Ok(value), span),
                Step::Await(phase, value) => {
                    if let Err(error) = self.object_work(span, |objects, _| {
                        objects.array_from_async_mut(state)?.phase = phase;
                        Ok(())
                    }) {
                        return self.finish_array_from_async(state, Err(error), span);
                    }
                    match self.await_native_value(
                        state,
                        AsyncResumeKind::ArrayFromAsync,
                        value,
                        span,
                    ) {
                        Ok(()) => return Ok(()),
                        Err(error) if error.is_language_exception() => resumed = Some(Err(error)),
                        Err(error) => return self.finish_array_from_async(state, Err(error), span),
                    }
                }
            }
        }
    }

    fn advance_array_from_async(
        &mut self,
        state: &ObjectHandle,
        span: Span,
    ) -> Result<Step, Error> {
        self.tick(span)?;
        let (source, next, index, length) = self.object_work(span, |objects, budget| {
            let data = objects.array_from_async_mut(state)?;
            Ok(match &data.input {
                AsyncArrayInput::Iterator(record) => {
                    budget.value(&record.next)?;
                    (
                        record.iterator.clone(),
                        Some(record.next.clone()),
                        data.index,
                        None,
                    )
                }
                AsyncArrayInput::ArrayLike { object, length } => {
                    (object.clone(), None, data.index, Some(*length))
                }
            })
        })?;
        if let Some(next) = next {
            if index >= 9_007_199_254_740_991 {
                return self.close_array_from_async(
                    state,
                    Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "Array.fromAsync exceeds the maximum safe integer length",
                    ),
                    span,
                );
            }
            // Step errors do not close; the sync adapter separately implements
            // AsyncFromSyncIteratorContinuation's rejected-value cleanup.
            let value = self.call(next, Value::Object(source), Vec::new(), span)?;
            Ok(Step::Await(AsyncArrayPhase::Next, value))
        } else if index == length.expect("array-like length") {
            self.complete_array_from_async(state, span)
        } else {
            let value =
                self.get_property(&source, &JsString::from(index.to_string().as_str()), span)?;
            Ok(Step::Await(AsyncArrayPhase::Item, value))
        }
    }

    fn resume_array_from_async_step(
        &mut self,
        state: &ObjectHandle,
        completion: Result<Value, Error>,
        span: Span,
    ) -> Result<Step, Error> {
        let phase = self.object_work(span, |objects, _| {
            Ok(objects.array_from_async_mut(state)?.phase)
        })?;
        match phase {
            AsyncArrayPhase::Close => {
                if let Err(error) = completion
                    && !error.is_language_exception()
                {
                    return Err(error);
                }
                let error = self.object_work(span, |objects, _| {
                    Ok(objects
                        .array_from_async_mut(state)?
                        .error
                        .take()
                        .expect("closing error"))
                })?;
                Err(Error::Thrown(error))
            }
            AsyncArrayPhase::Next => {
                let Value::Object(result) = completion? else {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "async iterator result is not an object",
                    ));
                };
                if self
                    .get_property(&result, &JsString::from("done"), span)?
                    .to_boolean()
                {
                    return self.complete_array_from_async(state, span);
                }
                // Async iterator values pass through without an extra Await.
                let value = self.get_property(&result, &JsString::from("value"), span)?;
                self.map_array_from_async(state, value, span)
            }
            AsyncArrayPhase::Item => self.map_array_from_async(state, completion?, span),
            AsyncArrayPhase::Map => match completion {
                Ok(value) => self.store_array_from_async(state, value, span),
                Err(error) => self.close_array_from_async(state, error, span),
            },
        }
    }

    fn map_array_from_async(
        &mut self,
        state: &ObjectHandle,
        value: Value,
        span: Span,
    ) -> Result<Step, Error> {
        let mapping = self.object_work(span, |objects, budget| {
            let data = objects.array_from_async_mut(state)?;
            if matches!(data.mapper, Value::Undefined) {
                return Ok(None);
            }
            budget.value(&data.mapper)?;
            budget.value(&data.this_arg)?;
            Ok(Some((
                data.mapper.clone(),
                data.this_arg.clone(),
                data.index,
            )))
        })?;
        let Some((mapper, this_arg, index)) = mapping else {
            return self.store_array_from_async(state, value, span);
        };
        match self.call(
            mapper,
            this_arg,
            vec![value, Value::Number(index as f64)],
            span,
        ) {
            Ok(value) => Ok(Step::Await(AsyncArrayPhase::Map, value)),
            Err(error) => self.close_array_from_async(state, error, span),
        }
    }

    fn store_array_from_async(
        &mut self,
        state: &ObjectHandle,
        value: Value,
        span: Span,
    ) -> Result<Step, Error> {
        let (array, index) = self.object_work(span, |objects, _| {
            let data = objects.array_from_async_mut(state)?;
            Ok((data.array.clone(), data.index))
        })?;
        if let Err(error) = self.create_array_element(&array, index, value, span) {
            return self.close_array_from_async(state, error, span);
        }
        self.object_work(span, |objects, _| {
            objects.array_from_async_mut(state)?.index += 1;
            Ok(())
        })?;
        Ok(Step::Advance)
    }

    fn complete_array_from_async(
        &mut self,
        state: &ObjectHandle,
        span: Span,
    ) -> Result<Step, Error> {
        let (array, index) = self.object_work(span, |objects, _| {
            let data = objects.array_from_async_mut(state)?;
            Ok((data.array.clone(), data.index))
        })?;
        self.finish_array_from(array, index, span).map(Step::Finish)
    }

    fn close_array_from_async(
        &mut self,
        state: &ObjectHandle,
        error: Error,
        span: Span,
    ) -> Result<Step, Error> {
        if !error.is_language_exception() {
            return Err(error);
        }
        let source = self.object_work(span, |objects, _| {
            Ok(match &objects.array_from_async_mut(state)?.input {
                AsyncArrayInput::Iterator(record) => Some(record.iterator.clone()),
                AsyncArrayInput::ArrayLike { .. } => None,
            })
        })?;
        let Some(source) = source else {
            return Err(error);
        };
        let reason = self.language_exception_value(error)?;
        self.object_work(span, |objects, budget| {
            budget.value(&reason)?;
            objects.array_from_async_mut(state)?.error = Some(reason);
            Ok(())
        })?;
        let close = (|| {
            let Some(method) = self.get_method(
                &Value::Object(source.clone()),
                &JsString::from("return"),
                span,
            )?
            else {
                return Ok(None);
            };
            self.call(method, Value::Object(source), Vec::new(), span)
                .map(Some)
        })();
        match close {
            Ok(Some(value)) => Ok(Step::Await(AsyncArrayPhase::Close, value)),
            Err(error) if !error.is_language_exception() => Err(error),
            // With a throw completion, the original reason wins over every
            // language failure from return lookup, invocation, or Await.
            Ok(None) | Err(_) => {
                let error = self.object_work(span, |objects, _| {
                    Ok(objects
                        .array_from_async_mut(state)?
                        .error
                        .take()
                        .expect("closing error"))
                })?;
                Err(Error::Thrown(error))
            }
        }
    }

    fn finish_array_from_async(
        &mut self,
        state: &ObjectHandle,
        completion: Result<Value, Error>,
        span: Span,
    ) -> Result<(), Error> {
        // Release captures even after a host allowance is exhausted.
        let data = self
            .objects
            .take_array_from_async(state)
            .map_err(Error::InvalidObject)?;
        let (method, value) = match completion {
            Ok(value) => (data.capability.resolve, value),
            Err(error) => (
                data.capability.reject,
                self.language_exception_value(error)?,
            ),
        };
        self.call(method, Value::Undefined, vec![value], span)?;
        Ok(())
    }

    pub(in crate::function) fn resume_array_from_async(
        &mut self,
        state: ObjectHandle,
        fulfilled: bool,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        self.drive_array_from_async(
            &state,
            Some(if fulfilled {
                Ok(value)
            } else {
                Err(Error::Thrown(value))
            }),
            span,
        )?;
        Ok(Value::Undefined)
    }
}
