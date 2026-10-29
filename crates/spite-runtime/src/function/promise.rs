//! Promise construction, resolution, capabilities and protocol dispatch (27.2).

use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value, object::PromiseReaction};
use spite_core::{JsString, Span, WellKnownSymbol};

mod aggregate;
pub(crate) use aggregate::PromiseAggregateKind;
mod race;
#[cfg(test)]
mod reaction_tests;

/// The outcome of one explicitly scheduled Promise job.
#[derive(Debug)]
pub enum PromiseJobOutcome {
    /// No job was queued; the Realm was not initialized or changed.
    NoJob,
    /// The selected job completed normally.
    Completed,
    /// The started job reported an abrupt completion. Object values are unrooted
    /// after return; retain them with Realm::root_value before collection.
    Reported(Error),
}

#[derive(Clone, Debug)]
pub(crate) enum PromiseJob {
    Thenable {
        promise: ObjectHandle,
        thenable: ObjectHandle,
        then: ObjectHandle,
    },
    Reaction {
        reaction: PromiseReaction,
        argument: Value,
    },
}

impl PromiseJob {
    pub(crate) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        let roots = match self {
            Self::Thenable {
                promise,
                thenable,
                then,
            } => [Some(promise), Some(thenable), Some(then), None, None],
            Self::Reaction { reaction, argument } => {
                fn object(value: &Value) -> Option<&ObjectHandle> {
                    if let Value::Object(value) = value {
                        Some(value)
                    } else {
                        None
                    }
                }
                let capability = reaction.capability.as_ref();
                [
                    capability.and_then(|capability| object(&capability.promise)),
                    capability.and_then(|capability| object(&capability.resolve)),
                    capability.and_then(|capability| object(&capability.reject)),
                    object(&reaction.handler),
                    object(argument),
                ]
            }
        };
        roots.into_iter().flatten()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PromiseCapability {
    pub promise: Value,
    pub resolve: Value,
    pub reject: Value,
}

impl PromiseCapability {
    pub(crate) fn values(&self) -> impl Iterator<Item = &Value> {
        [&self.promise, &self.resolve, &self.reject].into_iter()
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum PromiseFinallyKind {
    Then,
    Catch,
    ValueThunk,
    Thrower,
}

impl Realm {
    pub(crate) fn intrinsic_promise_capability(
        &mut self,
        span: Span,
    ) -> Result<PromiseCapability, Error> {
        let constructor = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .global_metadata
            .promise
            .clone();
        self.new_promise_capability(Value::Object(constructor), span)
    }

    fn promise_species_constructor(
        &mut self,
        promise: &ObjectHandle,
        span: Span,
    ) -> Result<Value, Error> {
        let default = Value::Object(
            self.intrinsics
                .as_ref()
                .expect("initialized")
                .global_metadata
                .promise
                .clone(),
        );
        let constructor = self.get_property(promise, &JsString::from("constructor"), span)?;
        Ok(if matches!(constructor, Value::Undefined) {
            default
        } else {
            let Value::Object(object) = constructor else {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "Promise constructor property is not an object",
                ));
            };
            let species = self.get_property(&object, &WellKnownSymbol::Species.symbol(), span)?;
            if matches!(species, Value::Undefined | Value::Null) {
                default
            } else {
                if !self.is_constructor(&species, span)? {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "Promise species is not a constructor",
                    ));
                }
                species
            }
        })
    }

    // Promise.prototype.finally (27.2.5.3) is generic over object receivers.
    // Its wrappers capture the species now and invoke the receiver's live then.
    pub(super) fn promise_finally(
        &mut self,
        this: Value,
        on_finally: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(object) = &this else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Promise.finally requires an object receiver",
            ));
        };
        let constructor = self.promise_species_constructor(object, span)?;
        let (fulfill, reject) = if self.is_callable(&on_finally, span)? {
            let cell = self.object_work(span, |objects, _| objects.create(None))?;
            self.define_builtin_property(&cell, "constructor", constructor, true, span)?;
            self.define_builtin_property(&cell, "callback", on_finally, true, span)?;
            (
                self.create_finally_callback(&cell, PromiseFinallyKind::Then, span)?,
                self.create_finally_callback(&cell, PromiseFinallyKind::Catch, span)?,
            )
        } else {
            (on_finally.clone(), on_finally)
        };
        let then = self.get_property_value(&this, &JsString::from("then"), span)?;
        self.call(then, this, vec![fulfill, reject], span)
    }

    fn create_finally_callback(
        &mut self,
        cell: &ObjectHandle,
        kind: PromiseFinallyKind,
        span: Span,
    ) -> Result<Value, Error> {
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .function_prototype
            .clone();
        let function = self.object_work(span, |objects, _| {
            objects.create_promise_finally_function(&prototype, cell, kind)
        })?;
        let length = match kind {
            PromiseFinallyKind::Then | PromiseFinallyKind::Catch => 1.0,
            PromiseFinallyKind::ValueThunk | PromiseFinallyKind::Thrower => 0.0,
        };
        self.define_builtin_property(&function, "length", Value::Number(length), false, span)?;
        self.define_builtin_property(
            &function,
            "name",
            Value::String(JsString::from("")),
            false,
            span,
        )?;
        Ok(Value::Object(function))
    }

    pub(super) fn promise_finally_callback(
        &mut self,
        cell: ObjectHandle,
        kind: PromiseFinallyKind,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let continuation = match kind {
            PromiseFinallyKind::ValueThunk => {
                return self.get_property(&cell, &JsString::from("value"), span);
            }
            PromiseFinallyKind::Thrower => {
                return Err(Error::Thrown(self.get_property(
                    &cell,
                    &JsString::from("value"),
                    span,
                )?));
            }
            PromiseFinallyKind::Then => PromiseFinallyKind::ValueThunk,
            PromiseFinallyKind::Catch => PromiseFinallyKind::Thrower,
        };
        let callback = self.get_property(&cell, &JsString::from("callback"), span)?;
        let constructor = self.get_property(&cell, &JsString::from("constructor"), span)?;
        let result = self.call(callback, Value::Undefined, Vec::new(), span)?;
        let promise = self.promise_resolve(constructor, result, span)?;
        let capture = self.object_work(span, |objects, _| objects.create(None))?;
        self.define_builtin_property(&capture, "value", value, true, span)?;
        let continuation = self.create_finally_callback(&capture, continuation, span)?;
        let then = self.get_property_value(&promise, &JsString::from("then"), span)?;
        self.call(then, promise, vec![continuation], span)
    }

    pub(super) fn promise_then(
        &mut self,
        this: Value,
        on_fulfilled: Value,
        on_rejected: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(promise) = this else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Promise.then requires a Promise",
            ));
        };
        if !self.object_work(span, |objects, _| {
            Ok(objects.inspect(&promise)?.promise_state().is_some())
        })? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Promise.then requires a Promise",
            ));
        }
        // Species hooks may settle the parent before reaction registration.
        let constructor = self.promise_species_constructor(&promise, span)?;
        let capability = self.new_promise_capability(constructor, span)?;
        self.perform_promise_then(&promise, on_fulfilled, on_rejected, Some(capability), span)
    }

    // PerformPromiseThen (27.2.5.4.1) also serves specification-internal
    // continuations. Without a capability it creates no derived Promise and
    // performs no visible then, constructor or species lookup.
    pub(super) fn perform_promise_then(
        &mut self,
        promise: &ObjectHandle,
        on_fulfilled: Value,
        on_rejected: Value,
        capability: Option<PromiseCapability>,
        span: Span,
    ) -> Result<Value, Error> {
        let result = capability
            .as_ref()
            .map_or(Value::Undefined, |capability| capability.promise.clone());
        let fulfill = if self.is_callable(&on_fulfilled, span)? {
            on_fulfilled
        } else {
            Value::Undefined
        };
        let reject = if self.is_callable(&on_rejected, span)? {
            on_rejected
        } else {
            Value::Undefined
        };
        let fulfill = PromiseReaction {
            capability: capability.clone(),
            handler: fulfill,
            fulfill: true,
        };
        let reject = PromiseReaction {
            capability,
            handler: reject,
            fulfill: false,
        };
        if let Some((reaction, argument)) = self.object_work(span, |objects, budget| {
            objects.add_promise_reactions(promise, fulfill, reject, budget)
        })? {
            self.enqueue_promise_job(PromiseJob::Reaction { reaction, argument }, span)?;
        }
        Ok(result)
    }

    fn settle_promise(
        &mut self,
        promise: &ObjectHandle,
        fulfilled: bool,
        value: Value,
        span: Span,
    ) -> Result<(), Error> {
        let reactions = self.object_work(span, |objects, budget| {
            objects.settle_promise(promise, fulfilled, value, budget)
        })?;
        for reaction in reactions {
            let argument = self.object_work(span, |objects, budget| {
                objects.promise_result(promise, budget)
            })?;
            self.enqueue_promise_job(PromiseJob::Reaction { reaction, argument }, span)?;
        }
        Ok(())
    }

    fn enqueue_promise_job(&mut self, job: PromiseJob, span: Span) -> Result<(), Error> {
        let moved = if self.promise_jobs.len() == self.promise_jobs.capacity() {
            self.promise_jobs.len()
        } else {
            0
        };
        self.object_work(span, |_, budget| {
            budget.charge(
                moved
                    .checked_add(1)
                    .ok_or(crate::object::Error::WorkLimit)?,
            )
        })?;
        self.promise_jobs.try_reserve(1).map_err(|_| Error::Limit {
            span,
            message: "Promise job capacity exceeded".into(),
        })?;
        self.promise_jobs.push_back(job);
        Ok(())
    }

    pub(super) fn promise_constructor(
        &mut self,
        new_target: ObjectHandle,
        executor: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // Promise (27.2.3.1) checks callability before observing the prototype.
        if !self.is_callable(&executor, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Promise executor is not callable",
            ));
        }
        let prototype = self.get_property(&new_target, &JsString::from("prototype"), span)?;
        let prototype = if let Value::Object(prototype) = prototype {
            prototype
        } else {
            self.check_function_realm(&new_target, span)?;
            self.intrinsics
                .as_ref()
                .expect("initialized")
                .global_metadata
                .promise_prototype
                .clone()
        };
        let promise = self.object_work(span, |objects, _| objects.create_promise(&prototype))?;
        let (resolve, reject) = self.promise_resolving_functions(&promise, span)?;
        if let Err(error) = self.call(
            executor,
            Value::Undefined,
            vec![resolve, reject.clone()],
            span,
        ) {
            let value = self.language_exception_value(error)?;
            self.call(reject, Value::Undefined, vec![value], span)?;
        }
        Ok(Value::Object(promise))
    }

    fn promise_resolving_functions(
        &mut self,
        promise: &ObjectHandle,
        span: Span,
    ) -> Result<(Value, Value), Error> {
        let cell = self.object_work(span, |objects, _| objects.create(None))?;
        self.define_builtin_property(&cell, "promise", Value::Object(promise.clone()), true, span)?;
        self.define_builtin_property(&cell, "alreadyResolved", Value::Boolean(false), true, span)?;
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .function_prototype
            .clone();
        let mut functions = Vec::new();
        for resolve in [true, false] {
            let function = self.object_work(span, |objects, _| {
                objects.create_promise_resolving_function(&prototype, &cell, resolve)
            })?;
            self.define_builtin_property(&function, "length", Value::Number(1.0), false, span)?;
            self.define_builtin_property(
                &function,
                "name",
                Value::String(JsString::from("")),
                false,
                span,
            )?;
            functions.push(Value::Object(function));
        }
        let reject = functions.pop().expect("reject");
        let resolve = functions.pop().expect("resolve");
        Ok((resolve, reject))
    }

    pub(super) fn promise_resolving_function(
        &mut self,
        cell: ObjectHandle,
        resolve: bool,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let done = self.get_property(&cell, &JsString::from("alreadyResolved"), span)?;
        if matches!(done, Value::Boolean(true)) {
            return Ok(Value::Undefined);
        }
        // Resolve and reject share this record, including while then is read.
        self.define_builtin_property(&cell, "alreadyResolved", Value::Boolean(true), true, span)?;
        let Value::Object(promise) = self.get_property(&cell, &JsString::from("promise"), span)?
        else {
            unreachable!("private resolving record")
        };
        if !resolve {
            self.settle_promise(&promise, false, value, span)?;
            return Ok(Value::Undefined);
        }
        if value.same_value(&Value::Object(promise.clone())) {
            let error = Self::exception(
                ExceptionKind::TypeError,
                span,
                "Promise cannot resolve itself",
            );
            let reason = self.language_exception_value(error)?;
            self.settle_promise(&promise, false, reason, span)?;
            return Ok(Value::Undefined);
        }
        let Value::Object(thenable) = value else {
            self.settle_promise(&promise, true, value, span)?;
            return Ok(Value::Undefined);
        };
        let then = match self.get_property(&thenable, &JsString::from("then"), span) {
            Ok(then) => then,
            Err(error) => {
                let reason = self.language_exception_value(error)?;
                self.settle_promise(&promise, false, reason, span)?;
                return Ok(Value::Undefined);
            }
        };
        if !self.is_callable(&then, span)? {
            self.settle_promise(&promise, true, Value::Object(thenable), span)?;
            return Ok(Value::Undefined);
        }
        let Value::Object(then) = then else {
            unreachable!("callable object")
        };
        self.enqueue_promise_job(
            PromiseJob::Thenable {
                promise,
                thenable,
                then,
            },
            span,
        )?;
        Ok(Value::Undefined)
    }

    pub(super) fn promise_resolve(
        &mut self,
        constructor: Value,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        if !matches!(constructor, Value::Object(_)) {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Promise.resolve requires an object receiver",
            ));
        }
        if let Value::Object(object) = &value {
            let branded = self.object_work(span, |objects, _| {
                Ok(objects.inspect(object)?.promise_state().is_some())
            })?;
            if branded {
                let actual = self.get_property(object, &JsString::from("constructor"), span)?;
                if actual.same_value(&constructor) {
                    return Ok(value);
                }
            }
        }
        let capability = self.new_promise_capability(constructor, span)?;
        self.call(capability.resolve, Value::Undefined, vec![value], span)?;
        Ok(capability.promise)
    }

    /// Runs one queued Promise job at an idle host turn, without collecting or
    /// clearing WeakRef kept objects. Empty queues are inert. Preflight errors
    /// preserve the queue; started jobs report errors through PromiseJobOutcome.
    /// All calls within the job share one optional work allowance. The host
    /// controls subsequent jobs and explicit collection checkpoints.
    pub fn run_next_promise_job(&mut self) -> Result<PromiseJobOutcome, Error> {
        let span = Span::new(0, 0);
        if self.call_depth != 0 || self.evaluation_depth != 0 || self.job_active {
            return Err(Error::Host {
                span,
                message: "cannot run a Promise job during active execution".into(),
            });
        }
        if self.promise_jobs.is_empty() {
            return Ok(PromiseJobOutcome::NoJob);
        }
        let caller_work = self.remaining_steps;
        self.remaining_steps = self.limits.max_steps;
        // Allocation never collects. Temporarily owning the front job avoids
        // copying arbitrary reaction arguments; rollback cannot allocate.
        let job = self.promise_jobs.pop_front().expect("queued job");
        let prepared = (|| -> Result<_, Error> {
            self.object_work(span, |_, budget| budget.charge(job.roots().count() + 1))?;
            let count = if matches!(job, PromiseJob::Thenable { .. }) {
                2
            } else {
                1
            };
            self.check_argument_count(count, span)?;
            let mut arguments = Vec::new();
            arguments
                .try_reserve_exact(count)
                .map_err(|_| Error::Limit {
                    span,
                    message: "Promise job argument capacity exceeded".into(),
                })?;
            let reject = if let PromiseJob::Thenable { promise, .. } = &job {
                let (resolve, reject) = self.promise_resolving_functions(promise, span)?;
                arguments.push(resolve);
                arguments.push(reject.clone());
                Some(reject)
            } else {
                None
            };
            Ok((reject, arguments))
        })();
        let (reject, mut arguments) = match prepared {
            Ok(prepared) => prepared,
            Err(error) => {
                self.promise_jobs.push_front(job);
                self.remaining_steps = caller_work;
                return Err(error);
            }
        };
        let global = self
            .scopes
            .first()
            .expect("queued Promise has a Realm")
            .clone();
        let caller_variable = self.variable_environment.replace(global);
        let caller_strict = std::mem::replace(&mut self.strict, false);
        let caller_scope_depth = self.scopes.len();
        self.job_active = true;
        let result = (|| -> Result<(), Error> {
            match job {
                PromiseJob::Thenable { thenable, then, .. } => {
                    if let Err(error) = self.call(
                        Value::Object(then),
                        Value::Object(thenable),
                        arguments,
                        span,
                    ) {
                        let reason = self.language_exception_value(error)?;
                        self.call(
                            reject.expect("thenable reject function"),
                            Value::Undefined,
                            vec![reason],
                            span,
                        )?;
                    }
                }
                PromiseJob::Reaction { reaction, argument } => {
                    let handler = reaction.handler.clone();
                    let completion = if matches!(reaction.handler, Value::Undefined) {
                        if reaction.fulfill {
                            Ok(argument)
                        } else {
                            Err(Error::Thrown(argument))
                        }
                    } else {
                        arguments.push(argument);
                        self.call(reaction.handler, Value::Undefined, arguments, span)
                    };
                    let Some(capability) = reaction.capability else {
                        // Specification-internal handlers consume their own
                        // language completions. Host failures still reach the
                        // job outcome and never become JavaScript rejections.
                        if let Err(error) = &completion
                            && !error.is_language_exception()
                        {
                            self.abort_native_module_reaction(&handler, error)?;
                        }
                        return completion.map(|_| ());
                    };
                    let (method, value) = match completion {
                        Ok(value) => (capability.resolve, value),
                        Err(error) => (capability.reject, self.language_exception_value(error)?),
                    };
                    self.call(method, Value::Undefined, vec![value], span)?;
                }
            }
            Ok(())
        })();
        self.job_active = false;
        self.strict = caller_strict;
        self.variable_environment = caller_variable;
        self.remaining_steps = caller_work;
        debug_assert_eq!(self.scopes.len(), caller_scope_depth);
        debug_assert_eq!((self.call_depth, self.evaluation_depth), (0, 0));
        Ok(match result {
            Ok(()) => PromiseJobOutcome::Completed,
            Err(error) => PromiseJobOutcome::Reported(error),
        })
    }

    // Promise.prototype.catch: Invoke(this, "then", [undefined, onRejected]).
    // No Promise brand or callback validation is performed (27.2.5.1).
    pub(super) fn promise_catch(
        &mut self,
        this: Value,
        on_rejected: Value,
        span: Span,
    ) -> Result<Value, Error> {
        Self::require_object_coercible(&this, span)?;
        let then = self.get_property_value(&this, &JsString::from("then"), span)?;
        self.call(then, this, vec![Value::Undefined, on_rejected], span)
    }

    // NewPromiseCapability (27.2.1.5) uses the supplied constructor without
    // branding its result. Its executor captures a private traced Record.
    pub(super) fn new_promise_capability(
        &mut self,
        constructor: Value,
        span: Span,
    ) -> Result<PromiseCapability, Error> {
        if !self.is_constructor(&constructor, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Promise capability requires a constructor",
            ));
        }
        let capability = self.object_work(span, |objects, _| objects.create(None))?;
        for key in ["resolve", "reject"] {
            self.define_builtin_property(&capability, key, Value::Undefined, true, span)?;
        }
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .function_prototype
            .clone();
        let executor = self.object_work(span, |objects, _| {
            objects.create_promise_capability_executor(&prototype, &capability)
        })?;
        self.define_builtin_property(&executor, "length", Value::Number(2.0), false, span)?;
        self.define_builtin_property(
            &executor,
            "name",
            Value::String(JsString::from("")),
            false,
            span,
        )?;
        let promise = self.construct(constructor, vec![Value::Object(executor)], span)?;
        let resolve = self.get_property(&capability, &JsString::from("resolve"), span)?;
        let reject = self.get_property(&capability, &JsString::from("reject"), span)?;
        if !self.is_callable(&resolve, span)? || !self.is_callable(&reject, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Promise capability functions must be callable",
            ));
        }
        Ok(PromiseCapability {
            promise,
            resolve,
            reject,
        })
    }

    pub(super) fn promise_capability_executor(
        &mut self,
        capability: ObjectHandle,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        // Check both existing fields before changing either. In particular a
        // prior call with undefined/undefined does not prevent a later call.
        let resolve = self.get_property(&capability, &JsString::from("resolve"), span)?;
        let reject = self.get_property(&capability, &JsString::from("reject"), span)?;
        if !matches!(resolve, Value::Undefined) || !matches!(reject, Value::Undefined) {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Promise capability executor was already initialized",
            ));
        }
        self.define_builtin_property(
            &capability,
            "resolve",
            arguments.next().unwrap_or(Value::Undefined),
            true,
            span,
        )?;
        self.define_builtin_property(
            &capability,
            "reject",
            arguments.next().unwrap_or(Value::Undefined),
            true,
            span,
        )?;
        Ok(Value::Undefined)
    }

    pub(super) fn promise_with_resolvers(
        &mut self,
        constructor: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let capability = self.new_promise_capability(constructor, span)?;
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .object_prototype
            .clone();
        let object = self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
        // CreateDataPropertyOrThrow gives all three fields writable/enumerable/
        // configurable attributes and bypasses inherited setters.
        for (key, value) in [
            ("promise", capability.promise),
            ("resolve", capability.resolve),
            ("reject", capability.reject),
        ] {
            self.define_property_or_throw(
                &object,
                JsString::from(key),
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

    pub(super) fn promise_reject(
        &mut self,
        constructor: Value,
        reason: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let capability = self.new_promise_capability(constructor, span)?;
        self.call(capability.reject, Value::Undefined, vec![reason], span)?;
        Ok(capability.promise)
    }

    pub(super) fn promise_try(
        &mut self,
        constructor: Value,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        // Edition 17 sec-promise.try checks only the receiver's type before
        // invoking the callback. Capability construction follows that Call.
        if !matches!(constructor, Value::Object(_)) {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Promise.try requires an object receiver",
            ));
        }
        let callback = arguments.next().unwrap_or(Value::Undefined);
        match self.call(callback, Value::Undefined, arguments.collect(), span) {
            Ok(value) => self.promise_resolve(constructor, value, span),
            Err(error) => {
                let value = self.language_exception_value(error)?;
                let capability = self.new_promise_capability(constructor, span)?;
                self.call(capability.reject, Value::Undefined, vec![value], span)?;
                Ok(capability.promise)
            }
        }
    }
}
