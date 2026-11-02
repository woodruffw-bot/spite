//! Dispatch one invocation before the outer loop performs any tail transfer.

use super::{
    Builtin, CallOutcome, Callable, TailCall, async_native, iterator, method, ordinary, proxy,
};
use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::Span;

fn forward_call(function: Value, this: Value, arguments: Vec<Value>, span: Span) -> CallOutcome {
    CallOutcome::Tail(Box::new(TailCall {
        function,
        this,
        arguments,
        span,
    }))
}

impl Realm {
    pub(super) fn call_step(
        &mut self,
        function: Value,
        this: Value,
        arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<CallOutcome, Error> {
        // 13.3.6.2: callers evaluate arguments before entering this callable check.
        let (callable, proxy, creation_realm) = if let Value::Object(object) = &function {
            self.object_work(span, |objects, budget| {
                // Capability is immutable, including for revoked Proxies.
                if !objects.inspect(object)?.is_callable() {
                    return Ok((None, None, None));
                }
                let proxy = objects.proxy_state(object, budget)?;
                if proxy.is_some() {
                    return Ok((None, proxy, None));
                }
                let callable = objects
                    .inspect(object)?
                    .callable()
                    .map(|callable| callable.copy_with_budget(budget))
                    .transpose()?;
                let creation_realm = objects.inspect(object)?.function_realm().cloned();
                Ok((callable, None, creation_realm))
            })?
        } else {
            (None, None, None)
        };
        if let Some(proxy) = proxy {
            match self.proxy_call(proxy, &this, arguments.as_slice(), span)? {
                proxy::ProxyInvocation::Forward(target) => {
                    return Ok(forward_call(
                        Value::Object(target),
                        this,
                        arguments.collect(),
                        span,
                    ));
                }
                proxy::ProxyInvocation::Complete(value) => return Ok(CallOutcome::Value(value)),
            }
        }

        let Some(callable) = callable else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "value is not callable",
            ));
        };
        if let Callable::Bound(bound) = callable {
            let count = bound
                .arguments
                .len()
                .checked_add(arguments.len())
                .ok_or_else(|| Error::Limit {
                    span,
                    message: "call argument limit exceeded".into(),
                })?;
            self.check_argument_count(count, span)?;
            let mut values = bound.arguments;
            values.extend(arguments);
            return Ok(forward_call(
                Value::Object(bound.target),
                bound.this,
                values,
                span,
            ));
        }
        if let Some(realm) = creation_realm {
            self.with_function_realm(&realm, span, |realm| {
                realm.call_concrete(function, callable, this, arguments, span)
            })
        } else {
            self.call_concrete(function, callable, this, arguments, span)
        }
    }

    fn call_concrete(
        &mut self,
        function: Value,
        callable: Callable,
        this: Value,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<CallOutcome, Error> {
        let builtin = match callable {
            Callable::Builtin(builtin) => builtin,
            Callable::PromiseCapabilityExecutor(capability) => {
                return self
                    .promise_capability_executor(capability, arguments, span)
                    .map(CallOutcome::Value);
            }
            Callable::PromiseResolvingFunction { cell, resolve } => {
                return self
                    .promise_resolving_function(
                        cell,
                        resolve,
                        arguments.next().unwrap_or(Value::Undefined),
                        span,
                    )
                    .map(CallOutcome::Value);
            }
            Callable::PromiseFinally { cell, kind } => {
                return self
                    .promise_finally_callback(
                        cell,
                        kind,
                        arguments.next().unwrap_or(Value::Undefined),
                        span,
                    )
                    .map(CallOutcome::Value);
            }
            Callable::AsyncFromSyncUnwrap { done } => {
                return self
                    .iterator_result(arguments.next().unwrap_or(Value::Undefined), done, span)
                    .map(CallOutcome::Value);
            }
            Callable::AsyncFromSyncClose(iterator) => {
                let record = iterator::IteratorRecord::uninitialized(iterator);
                return Err(self.iterator_close_error(
                    &record,
                    Error::Thrown(arguments.next().unwrap_or(Value::Undefined)),
                    span,
                ));
            }
            Callable::AsyncResume {
                state,
                kind,
                fulfilled,
            } => {
                let value = arguments.next().unwrap_or(Value::Undefined);
                return match kind {
                    async_native::AsyncResumeKind::DisposeResources => {
                        self.resume_async_disposal(state, fulfilled, value, span)
                    }
                    async_native::AsyncResumeKind::ArrayFromAsync => {
                        self.resume_array_from_async(state, fulfilled, value, span)
                    }
                    async_native::AsyncResumeKind::Execution => {
                        self.resume_async_execution(state, fulfilled, value, span)
                    }
                    async_native::AsyncResumeKind::ModuleExecution => {
                        self.resume_module_execution(state, fulfilled, value, span)
                    }
                    async_native::AsyncResumeKind::ModuleNamespace => {
                        self.module_namespace_record(&state, span)
                    }
                    async_native::AsyncResumeKind::ModuleImportLoad => {
                        self.resume_module_import(state, fulfilled, value, span)
                    }
                    async_native::AsyncResumeKind::AsyncGeneratorReturn => {
                        self.resume_async_generator_return(state, fulfilled, value, span)
                    }
                    async_native::AsyncResumeKind::AsyncGeneratorResumeReturn => {
                        self.resume_async_generator_return_resumption(state, fulfilled, value, span)
                    }
                }
                .map(CallOutcome::Value);
            }
            Callable::PromiseAggregateElement { fulfilled, .. } => {
                let Value::Object(function) = function else {
                    unreachable!("resolve element is callable")
                };
                return self
                    .promise_aggregate_element(
                        function,
                        fulfilled,
                        arguments.next().unwrap_or(Value::Undefined),
                        span,
                    )
                    .map(CallOutcome::Value);
            }
            Callable::ProxyRevoker(_) => {
                let Value::Object(revoker) = function else {
                    unreachable!("revoker is callable")
                };
                self.object_work(span, |objects, budget| {
                    objects.revoke_proxy_from(&revoker, budget)
                })?;
                return Ok(CallOutcome::Value(Value::Undefined));
            }
            Callable::ClassConstructor(_) => {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "class constructor requires new",
                ));
            }
            Callable::Arrow(arrow) => {
                if arrow.kind == spite_parser::ast::FunctionKind::Async {
                    return self
                        .call_async_arrow(arrow, arguments, span)
                        .map(CallOutcome::Value);
                }
                return self.call_arrow(arrow, arguments, span);
            }
            Callable::Ordinary(code) => {
                let Value::Object(callee) = function else {
                    unreachable!("callable object")
                };
                if matches!(
                    code.kind,
                    spite_parser::ast::FunctionKind::Generator
                        | spite_parser::ast::FunctionKind::AsyncGenerator
                ) {
                    return self
                        .call_generator_ordinary(
                            ordinary::OrdinaryInvocation {
                                code,
                                callee,
                                this,
                                context: Default::default(),
                                arguments,
                            },
                            span,
                        )
                        .map(CallOutcome::Value);
                }
                if code.kind == spite_parser::ast::FunctionKind::Async {
                    return self
                        .call_async_ordinary(
                            ordinary::OrdinaryInvocation {
                                code,
                                callee,
                                this,
                                context: Default::default(),
                                arguments,
                            },
                            span,
                        )
                        .map(CallOutcome::Value);
                }
                return self.call_ordinary_outcome(
                    code,
                    callee,
                    this,
                    Default::default(),
                    arguments,
                    span,
                );
            }
            Callable::Method(method) => {
                let Value::Object(callee) = function else {
                    unreachable!("callable object")
                };
                let method::MethodFunction { code, home_object } = *method;
                let context = crate::environment::FunctionContext {
                    new_target: None,
                    home_object: Some(home_object),
                    derived_constructor: None,
                    class_field_initializer: false,
                };
                if matches!(
                    code.kind,
                    spite_parser::ast::FunctionKind::Generator
                        | spite_parser::ast::FunctionKind::AsyncGenerator
                ) {
                    return self
                        .call_generator_ordinary(
                            ordinary::OrdinaryInvocation {
                                code,
                                callee,
                                this,
                                context,
                                arguments,
                            },
                            span,
                        )
                        .map(CallOutcome::Value);
                }
                if code.kind == spite_parser::ast::FunctionKind::Async {
                    return self
                        .call_async_ordinary(
                            ordinary::OrdinaryInvocation {
                                code,
                                callee,
                                this,
                                context,
                                arguments,
                            },
                            span,
                        )
                        .map(CallOutcome::Value);
                }
                return self.call_ordinary_outcome(code, callee, this, context, arguments, span);
            }
            Callable::Bound(_) => unreachable!("bound calls forward before concrete dispatch"),
        };
        let result = match builtin {
            // Eval compiles and executes more Script code. Do not retain
            // the large native-algorithm dispatch frame during re-entry.
            Builtin::Eval => {
                self.perform_eval(arguments.next().unwrap_or(Value::Undefined), false, span)
            }
            Builtin::EvalScript => {
                self.host_eval_script(arguments.next().unwrap_or(Value::Undefined), span)
            }
            // Source generators can synchronously reenter JavaScript.
            // Keep the large native-algorithm frame out of that recursion.
            Builtin::GeneratorNext | Builtin::GeneratorReturn | Builtin::GeneratorThrow => self
                .generator_resume(
                    this,
                    builtin,
                    arguments.next().unwrap_or(Value::Undefined),
                    span,
                ),
            Builtin::AsyncGeneratorNext
            | Builtin::AsyncGeneratorReturn
            | Builtin::AsyncGeneratorThrow => self.async_generator_resume(
                this,
                builtin,
                arguments.next().unwrap_or(Value::Undefined),
                span,
            ),
            Builtin::FunctionCall => {
                // 20.2.3.3 is a tail call. Transfer ownership of the receiver and
                // advance the argument iterator without Rust stack recursion.
                let receiver = arguments.next().unwrap_or(Value::Undefined);
                return Ok(forward_call(this, receiver, arguments.collect(), span));
            }
            Builtin::FunctionBind => {
                return self
                    .bind_function(this, arguments, span)
                    .map(CallOutcome::Value);
            }
            Builtin::FunctionApply => {
                // 20.2.3.1 checks the target before inspecting the argument list.
                if !self.is_callable(&this, span)? {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "apply requires a callable receiver",
                    ));
                }
                let this_argument = arguments.next().unwrap_or(Value::Undefined);
                let array_like = arguments.next().unwrap_or(Value::Undefined);
                let values = if matches!(array_like, Value::Undefined | Value::Null) {
                    Vec::new()
                } else {
                    self.argument_list_from_array_like(array_like, span)?
                };
                return Ok(forward_call(this, this_argument, values, span));
            }
            Builtin::ReflectApply => {
                // 28.1.1: check the target before CreateListFromArrayLike.
                // Transfer to Call as a tail call, as for Function.apply.
                let target = arguments.next().unwrap_or(Value::Undefined);
                if !self.is_callable(&target, span)? {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "Reflect.apply requires a callable target",
                    ));
                }
                let receiver = arguments.next().unwrap_or(Value::Undefined);
                let list = arguments.next().unwrap_or(Value::Undefined);
                let values = self.argument_list_from_array_like(list, span)?;
                return Ok(forward_call(target, receiver, values, span));
            }
            builtin => self.call_builtin(builtin, this, arguments, span),
        }?;
        self.check_string(&result, span)?;
        Ok(CallOutcome::Value(result))
    }
}
