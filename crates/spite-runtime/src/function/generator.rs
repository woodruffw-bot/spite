//! Generator objects, suspension and abrupt resumption (15.5, 27.3, 27.5).

use super::{
    Builtin,
    async_execution::{AsyncBodyOutcome, SourceResume},
    ordinary::OrdinaryInvocation,
};
use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{
        DataDescriptor,
        generator::{GeneratorStart, GeneratorStatus},
    },
};
use spite_core::{JsString, Span, WellKnownSymbol};

#[derive(Debug)]
pub(crate) struct GeneratorIntrinsics {
    pub constructor: ObjectHandle,
    pub function_prototype: ObjectHandle,
    pub prototype: ObjectHandle,
}

impl GeneratorIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [&self.constructor, &self.function_prototype, &self.prototype].into_iter()
    }
}

impl Realm {
    pub(super) fn generator_intrinsics(
        &mut self,
        kind: spite_parser::ast::FunctionKind,
        function_constructor: &ObjectHandle,
        function_prototype: &ObjectHandle,
        iterator_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<GeneratorIntrinsics, Error> {
        let (constructor_kind, function_tag, instance_tag, methods) = match kind {
            spite_parser::ast::FunctionKind::Generator => (
                Builtin::GeneratorFunction,
                "GeneratorFunction",
                "Generator",
                [
                    Builtin::GeneratorNext,
                    Builtin::GeneratorReturn,
                    Builtin::GeneratorThrow,
                ],
            ),
            spite_parser::ast::FunctionKind::AsyncGenerator => (
                Builtin::AsyncGeneratorFunction,
                "AsyncGeneratorFunction",
                "AsyncGenerator",
                [
                    Builtin::AsyncGeneratorNext,
                    Builtin::AsyncGeneratorReturn,
                    Builtin::AsyncGeneratorThrow,
                ],
            ),
            _ => unreachable!("generator intrinsic kind"),
        };
        let constructor = self.new_builtin(function_constructor, constructor_kind, span)?;
        let generator_function =
            self.object_work(span, |objects, _| objects.create(Some(function_prototype)))?;
        let prototype =
            self.object_work(span, |objects, _| objects.create(Some(iterator_prototype)))?;
        self.define_property_or_throw(
            &constructor,
            JsString::from("prototype"),
            DataDescriptor {
                value: Some(Value::Object(generator_function.clone())),
                writable: Some(false),
                enumerable: Some(false),
                configurable: Some(false),
            }
            .into(),
            span,
        )?;
        self.define_builtin_property(
            &generator_function,
            "constructor",
            Value::Object(constructor.clone()),
            false,
            span,
        )?;
        self.define_builtin_property(
            &generator_function,
            "prototype",
            Value::Object(prototype.clone()),
            false,
            span,
        )?;
        self.define_builtin_property(
            &prototype,
            "constructor",
            Value::Object(generator_function.clone()),
            false,
            span,
        )?;
        for (object, tag) in [
            (&generator_function, function_tag),
            (&prototype, instance_tag),
        ] {
            self.define_property_or_throw(
                object,
                WellKnownSymbol::ToStringTag.symbol(),
                DataDescriptor {
                    value: Some(Value::String(JsString::from(tag))),
                    writable: Some(false),
                    enumerable: Some(false),
                    configurable: Some(true),
                }
                .into(),
                span,
            )?;
        }
        for (name, builtin) in ["next", "return", "throw"].into_iter().zip(methods) {
            let method = self.new_builtin(function_prototype, builtin, span)?;
            self.define_builtin_property(&prototype, name, Value::Object(method), true, span)?;
        }
        Ok(GeneratorIntrinsics {
            constructor,
            function_prototype: generator_function,
            prototype,
        })
    }

    pub(super) fn define_generator_function_prototype(
        &mut self,
        function: &ObjectHandle,
        kind: spite_parser::ast::FunctionKind,
        span: Span,
    ) -> Result<(), Error> {
        let intrinsics = self.intrinsics.as_ref().expect("initialized");
        let intrinsic = if kind == spite_parser::ast::FunctionKind::AsyncGenerator {
            &intrinsics.async_generator
        } else {
            &intrinsics.generator
        }
        .prototype
        .clone();
        let prototype = self.object_work(span, |objects, _| objects.create(Some(&intrinsic)))?;
        self.define_property_or_throw(
            function,
            JsString::from("prototype"),
            DataDescriptor {
                value: Some(Value::Object(prototype)),
                writable: Some(true),
                enumerable: Some(false),
                configurable: Some(false),
            }
            .into(),
            span,
        )
    }

    pub(super) fn call_generator_ordinary(
        &mut self,
        invocation: OrdinaryInvocation,
        span: Span,
    ) -> Result<Value, Error> {
        let asynchronous = invocation.code.kind == spite_parser::ast::FunctionKind::AsyncGenerator;
        let callee = invocation.callee.clone();
        let base = self.scopes.len();
        let (value, constructor) = self.with_ordinary_call(invocation, span, |realm, body| {
            // EvaluateGeneratorBody performs all declaration/parameter setup at
            // the call. Only the initialized body waits for the first next().
            let root = realm.prepare_function_body(body, span)?;
            if asynchronous {
                // 15.6.2 performs FDI before creating an async-generator instance.
                return Err(Self::unsupported(
                    span,
                    "async generator instances are not implemented",
                ));
            }
            let prototype = realm.get_property(&callee, &JsString::from("prototype"), span)?;
            let prototype = match prototype {
                Value::Object(prototype) => prototype,
                _ => {
                    realm.check_function_realm(&callee, span)?;
                    realm
                        .intrinsics
                        .as_ref()
                        .expect("initialized")
                        .generator
                        .prototype
                        .clone()
                }
            };
            let context = realm.capture_source_execution_context(
                base,
                span,
                "generator context allocation failed",
            )?;
            let start = GeneratorStart {
                body: body.clone(),
                callee: callee.clone(),
                context,
                root,
                span,
            };
            realm
                .object_work(span, |objects, budget| {
                    objects.create_generator(&prototype, start, budget)
                })
                .map(Value::Object)
        })?;
        debug_assert!(constructor.is_none());
        Ok(value)
    }

    pub(super) fn reject_async_generator_receiver(&mut self, span: Span) -> Result<Value, Error> {
        // Until instance creation is implemented, every exposed receiver lacks
        // async-generator slots. AsyncGeneratorValidate rejects a fresh Promise.
        let capability = self.intrinsic_promise_capability(span)?;
        let error = Self::exception(
            ExceptionKind::TypeError,
            span,
            "receiver is not a source async generator",
        );
        let reason = self.language_exception_value(error)?;
        self.call(capability.reject, Value::Undefined, vec![reason], span)?;
        Ok(capability.promise)
    }

    pub(super) fn generator_resume(
        &mut self,
        receiver: Value,
        kind: Builtin,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let invalid = || {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "receiver is not a source generator",
            )
        };
        let Value::Object(generator) = receiver else {
            return Err(invalid());
        };
        let status = self
            .object_work(span, |objects, _| objects.generator_status(&generator))?
            .ok_or_else(invalid)?;
        if status == GeneratorStatus::Executing {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "generator is already executing",
            ));
        }
        if status != GeneratorStatus::SuspendedYield
            && matches!(kind, Builtin::GeneratorReturn | Builtin::GeneratorThrow)
        {
            self.objects
                .complete_generator(&generator)
                .expect("validated generator");
            return if matches!(kind, Builtin::GeneratorThrow) {
                Err(Error::Thrown(value))
            } else {
                self.iterator_result(value, true, span)
            };
        }
        if status == GeneratorStatus::Completed {
            return self.iterator_result(Value::Undefined, true, span);
        }
        let outcome = if status == GeneratorStatus::SuspendedStart {
            let GeneratorStart {
                body,
                context,
                root,
                callee,
                span: body_span,
            } = self.object_work(span, |objects, _| objects.take_generator_start(&generator))?;
            self.with_source_execution_context(context, |realm| {
                realm.start_generator_execution(body, root, callee, body_span, generator.clone())
            })
        } else {
            let mut data = self.object_work(span, |objects, _| {
                objects.take_generator_execution(&generator)
            })?;
            let context = data.context.take().expect("yielded generator context");
            let resumed = match kind {
                Builtin::GeneratorNext => SourceResume::Value(value),
                Builtin::GeneratorReturn => SourceResume::Return(value),
                Builtin::GeneratorThrow => SourceResume::Throw(Error::Thrown(value)),
                _ => unreachable!("generator resume operation"),
            };
            self.with_source_execution_context(context, |realm| {
                realm.resume_generator_execution(*data, generator.clone(), resumed)
            })
        };
        let completion = match outcome {
            Ok(AsyncBodyOutcome::Yielded(result)) => return Ok(result),
            Ok(AsyncBodyOutcome::Suspended) => unreachable!("synchronous generator cannot await"),
            Ok(AsyncBodyOutcome::Complete(value)) => Ok(value),
            Err(error) => Err(error),
        };
        // Completing and releasing captures must work even after budget exhaustion.
        self.objects
            .complete_generator(&generator)
            .expect("executing generator");
        self.iterator_result(completion?, true, span)
    }
}
