//! Async-generator request serialization and completion (27.6.3).

use super::{
    Builtin,
    async_execution::{AsyncBodyOutcome, SourceResume},
    async_native::AsyncResumeKind,
};
use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{
        async_execution::SourceFunctionMode,
        async_generator::{AsyncGeneratorRequest, AsyncGeneratorStatus as Status, RequestKind},
        generator::GeneratorStart,
    },
};
use spite_core::Span;

impl Realm {
    pub(super) fn async_generator_resume(
        &mut self,
        receiver: Value,
        kind: Builtin,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let capability = self.intrinsic_promise_capability(span)?;
        let generator = if let Value::Object(object) = receiver {
            self.object_work(span, |objects, _| objects.async_generator_status(&object))?
                .map(|status| (object, status))
        } else {
            None
        };
        let Some((generator, status)) = generator else {
            let error = Self::exception(
                ExceptionKind::TypeError,
                span,
                "receiver is not a source async generator",
            );
            let reason = self.language_exception_value(error)?;
            self.call(capability.reject, Value::Undefined, vec![reason], span)?;
            return Ok(capability.promise);
        };
        let kind = match kind {
            Builtin::AsyncGeneratorNext => RequestKind::Next,
            Builtin::AsyncGeneratorReturn => RequestKind::Return,
            Builtin::AsyncGeneratorThrow => RequestKind::Throw,
            _ => unreachable!("async-generator method"),
        };
        if kind == RequestKind::Next && status == Status::Completed {
            let result = self.iterator_result(Value::Undefined, true, span)?;
            self.call(capability.resolve, Value::Undefined, vec![result], span)?;
            return Ok(capability.promise);
        }
        if kind == RequestKind::Throw
            && matches!(status, Status::SuspendedStart | Status::Completed)
        {
            self.objects
                .set_async_generator_status(&generator, Status::Completed)
                .map_err(Error::InvalidObject)?;
            self.call(capability.reject, Value::Undefined, vec![value], span)?;
            return Ok(capability.promise);
        }
        let promise = capability.promise.clone();
        self.object_work(span, |objects, budget| {
            objects.enqueue_async_generator(
                &generator,
                AsyncGeneratorRequest {
                    kind,
                    value,
                    capability,
                    span,
                },
                budget,
            )
        })?;
        let result = if kind == RequestKind::Return
            && matches!(status, Status::SuspendedStart | Status::Completed)
        {
            self.objects
                .set_async_generator_status(&generator, Status::DrainingQueue)
                .map_err(Error::InvalidObject)?;
            self.drain_async_generator(&generator, span)
        } else if matches!(status, Status::SuspendedStart | Status::SuspendedYield) {
            self.drive_async_generator(&generator, span)
        } else {
            Ok(())
        };
        if let Err(error) = result {
            return Err(self.abort_async_generator_execution(&generator, error));
        }
        Ok(promise)
    }

    fn drive_async_generator(&mut self, generator: &ObjectHandle, span: Span) -> Result<(), Error> {
        loop {
            let status = self
                .objects
                .async_generator_status(generator)
                .map_err(Error::InvalidObject)?
                .expect("source async generator");
            let outcome = if status == Status::SuspendedStart {
                let GeneratorStart {
                    body,
                    callee,
                    context,
                    root,
                    span: body_span,
                } = self.object_work(span, |objects, _| {
                    objects.take_async_generator_start(generator)
                })?;
                self.with_source_execution_context(context, |realm| {
                    realm.start_generator_execution(
                        body,
                        root,
                        SourceFunctionMode::AsyncGenerator(callee),
                        body_span,
                        generator.clone(),
                    )
                })
            } else {
                debug_assert!(matches!(status, Status::SuspendedYield | Status::Executing));
                let request = self
                    .objects
                    .async_generator_front(generator)
                    .map_err(Error::InvalidObject)?
                    .expect("queued async-generator request");
                self.objects
                    .set_async_generator_status(generator, Status::Executing)
                    .map_err(Error::InvalidObject)?;
                let completion = match request.kind {
                    RequestKind::Next => SourceResume::Value(request.value),
                    RequestKind::Throw => SourceResume::Throw(Error::Thrown(request.value)),
                    RequestKind::Return => {
                        // AsyncGeneratorUnwrapYieldResumption awaits a return
                        // argument before injecting it into catch/finally.
                        match self.await_native_value(
                            generator,
                            AsyncResumeKind::AsyncGeneratorResumeReturn,
                            request.value,
                            request.span,
                        ) {
                            Ok(()) => return Ok(()),
                            Err(error) if error.is_language_exception() => {
                                SourceResume::Throw(error)
                            }
                            Err(error) => return Err(error),
                        }
                    }
                };
                self.resume_async_generator_source(generator, completion)
            };
            if !self.handle_async_generator_outcome(generator, outcome, span)? {
                return Ok(());
            }
        }
    }

    fn resume_async_generator_source(
        &mut self,
        generator: &ObjectHandle,
        completion: SourceResume,
    ) -> Result<AsyncBodyOutcome, Error> {
        let mut data = self
            .objects
            .take_async_execution(generator)
            .map_err(Error::InvalidObject)?
            .expect("suspended async-generator execution");
        let context = data.context.take().expect("async-generator source context");
        self.with_source_execution_context(context, |realm| {
            realm.resume_generator_execution(*data, generator.clone(), completion)
        })
    }

    // true means that a queued request can resume the newly yielded body.
    fn handle_async_generator_outcome(
        &mut self,
        generator: &ObjectHandle,
        outcome: Result<AsyncBodyOutcome, Error>,
        span: Span,
    ) -> Result<bool, Error> {
        match outcome {
            Ok(AsyncBodyOutcome::Suspended) => Ok(false),
            Ok(AsyncBodyOutcome::Yielded(value)) => {
                // Keep Executing during Resolve's observable then lookup.
                // Reentrant methods enqueue instead of entering the body twice.
                self.complete_async_generator_step(generator, Ok(value), false, span)?;
                if self
                    .objects
                    .async_generator_front(generator)
                    .map_err(Error::InvalidObject)?
                    .is_some()
                {
                    Ok(true)
                } else {
                    self.objects
                        .set_async_generator_status(generator, Status::SuspendedYield)
                        .map_err(Error::InvalidObject)?;
                    Ok(false)
                }
            }
            outcome => {
                let completion = match outcome {
                    Ok(AsyncBodyOutcome::Complete(value)) => Ok(value),
                    Err(error) if error.is_language_exception() => {
                        Err(self.language_exception_value(error)?)
                    }
                    Err(error) => return Err(error),
                    _ => unreachable!("completed async-generator body"),
                };
                self.objects
                    .set_async_generator_status(generator, Status::DrainingQueue)
                    .map_err(Error::InvalidObject)?;
                self.complete_async_generator_step(generator, completion, true, span)?;
                self.drain_async_generator(generator, span)?;
                Ok(false)
            }
        }
    }

    fn complete_async_generator_step(
        &mut self,
        generator: &ObjectHandle,
        completion: Result<Value, Value>,
        done: bool,
        span: Span,
    ) -> Result<(), Error> {
        let request = self.object_work(span, |objects, _| {
            objects.pop_async_generator_request(generator)
        })?;
        // Remove the request before Resolve/Reject can call user code.
        match completion {
            Ok(value) => {
                let result = self.iterator_result(value, done, request.span)?;
                self.call(
                    request.capability.resolve,
                    Value::Undefined,
                    vec![result],
                    request.span,
                )?;
            }
            Err(reason) => {
                self.call(
                    request.capability.reject,
                    Value::Undefined,
                    vec![reason],
                    request.span,
                )?;
            }
        }
        Ok(())
    }

    fn drain_async_generator(&mut self, generator: &ObjectHandle, span: Span) -> Result<(), Error> {
        while let Some(request) = self
            .objects
            .async_generator_front(generator)
            .map_err(Error::InvalidObject)?
        {
            let completion = match request.kind {
                RequestKind::Next => Ok(Value::Undefined),
                RequestKind::Throw => Err(request.value),
                RequestKind::Return => match self.await_native_value(
                    generator,
                    AsyncResumeKind::AsyncGeneratorReturn,
                    request.value,
                    request.span,
                ) {
                    Ok(()) => return Ok(()),
                    Err(error) if error.is_language_exception() => {
                        Err(self.language_exception_value(error)?)
                    }
                    Err(error) => return Err(error),
                },
            };
            self.complete_async_generator_step(generator, completion, true, span)?;
        }
        self.objects
            .set_async_generator_status(generator, Status::Completed)
            .map_err(Error::InvalidObject)
    }

    pub(super) fn resume_async_generator_return(
        &mut self,
        generator: ObjectHandle,
        fulfilled: bool,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        if self
            .objects
            .async_generator_status(&generator)
            .map_err(Error::InvalidObject)?
            != Some(Status::DrainingQueue)
        {
            return Ok(Value::Undefined);
        }
        let result = (|| {
            self.complete_async_generator_step(
                &generator,
                if fulfilled { Ok(value) } else { Err(value) },
                true,
                span,
            )?;
            self.drain_async_generator(&generator, span)
        })();
        if let Err(error) = result {
            return Err(self.abort_async_generator_execution(&generator, error));
        }
        Ok(Value::Undefined)
    }

    pub(super) fn resume_async_generator_return_resumption(
        &mut self,
        generator: ObjectHandle,
        fulfilled: bool,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        if self
            .objects
            .async_generator_status(&generator)
            .map_err(Error::InvalidObject)?
            != Some(Status::Executing)
        {
            return Ok(Value::Undefined);
        }
        let completion = if fulfilled {
            SourceResume::Return(value)
        } else {
            SourceResume::Throw(Error::Thrown(value))
        };
        let outcome = self.resume_async_generator_source(&generator, completion);
        self.finish_async_generator_execution(generator, outcome, span)
    }

    pub(super) fn finish_async_generator_execution(
        &mut self,
        generator: ObjectHandle,
        outcome: Result<AsyncBodyOutcome, Error>,
        span: Span,
    ) -> Result<Value, Error> {
        let result = self
            .handle_async_generator_outcome(&generator, outcome, span)
            .and_then(|resume| {
                if resume {
                    self.drive_async_generator(&generator, span)
                } else {
                    Ok(())
                }
            });
        if let Err(error) = result {
            return Err(self.abort_async_generator_execution(&generator, error));
        }
        Ok(Value::Undefined)
    }

    fn abort_async_generator_execution(&mut self, generator: &ObjectHandle, error: Error) -> Error {
        // Host aborts release prepaid resource records without executing disposal
        // or settling queued Promises. Already registered reactions become inert.
        if let Some(data) = self
            .objects
            .take_async_execution(generator)
            .expect("async-generator object")
            && let Some(context) = &data.context
        {
            for environment in context.scopes.iter().skip(1) {
                self.objects
                    .take_environment_disposable_resources(environment)
                    .expect("captured async-generator scope");
            }
        }
        self.objects
            .abort_async_generator(generator)
            .expect("async-generator object");
        error
    }
}
