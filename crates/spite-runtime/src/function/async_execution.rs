//! Direct source Await statements and returns (Await, sec-asyncblockstart).

use super::{
    PromiseCapability, ScriptFunction, async_native::AsyncResumeKind, ordinary::OrdinaryInvocation,
};
use crate::{
    Completion, CompletionKind, Error, ObjectHandle, Realm, Value,
    object::async_execution::{AsyncAwaitAction, AsyncExecutionContext, AsyncExecutionData},
};
use spite_core::Span;
use spite_parser::ast::{ArrowBody, Expr, ExprKind, StatementKind};

pub(super) enum AsyncBodyOutcome {
    Complete(Value),
    Suspended,
}

enum Step {
    Continue,
    Complete(Completion),
    Await(Value, AsyncAwaitAction, Span),
}

impl Realm {
    pub(super) fn call_async_arrow(
        &mut self,
        code: ScriptFunction,
        arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        self.call_async_body(span, |realm, capability, base| {
            realm.with_arrow_call(code, arguments, span, |realm, body| {
                realm.start_async_execution(body, capability.clone(), base, span)
            })
        })
    }

    pub(super) fn call_async_ordinary(
        &mut self,
        invocation: OrdinaryInvocation,
        span: Span,
    ) -> Result<Value, Error> {
        self.call_async_body(span, |realm, capability, base| {
            realm
                .with_ordinary_call(invocation, span, |realm, body| {
                    realm.start_async_execution(
                        ArrowBody::Block(body.clone()),
                        capability.clone(),
                        base,
                        span,
                    )
                })
                .map(|(outcome, constructor)| {
                    debug_assert!(constructor.is_none());
                    outcome
                })
        })
    }

    fn start_async_execution(
        &mut self,
        body: ArrowBody,
        capability: PromiseCapability,
        base: usize,
        span: Span,
    ) -> Result<AsyncBodyOutcome, Error> {
        let root_scope = match &body {
            ArrowBody::Block(body) => Some(self.prepare_function_body(body, span)?),
            ArrowBody::Expression(_) => None,
        };
        let data = AsyncExecutionData {
            capability,
            body,
            index: 0,
            pending: AsyncAwaitAction::Discard,
            root_scope,
            context: None,
            span,
        };
        self.drive_async_execution(data, None, None, base)
    }

    fn drive_async_execution(
        &mut self,
        mut data: AsyncExecutionData,
        state: Option<ObjectHandle>,
        resumed: Option<Result<Value, Error>>,
        base: usize,
    ) -> Result<AsyncBodyOutcome, Error> {
        let mut next_step = None;
        if let Some(completion) = resumed {
            match completion {
                Err(error) => return self.finish_async_execution(data, Err(error), base),
                Ok(value) if matches!(data.pending, AsyncAwaitAction::Return) => {
                    return self.finish_async_execution(
                        data,
                        Ok(Completion {
                            kind: CompletionKind::Return,
                            value: Some(value),
                            target: None,
                        }),
                        base,
                    );
                }
                Ok(value) => {
                    if let AsyncAwaitAction::Binding { statement, element } = data.pending {
                        next_step =
                            Some(self.async_binding_step(&data, statement, element, Some(value)));
                    }
                }
            }
        }
        loop {
            let step = next_step
                .take()
                .unwrap_or_else(|| self.async_execution_step(&mut data));
            match step {
                Ok(Step::Continue) => (),
                Ok(Step::Complete(completion)) => {
                    return self.finish_async_execution(data, Ok(completion), base);
                }
                Err(error) => return self.finish_async_execution(data, Err(error), base),
                Ok(Step::Await(value, action, span)) => {
                    data.pending = action;
                    data.context = match self.capture_async_execution_context(base, span) {
                        Ok(context) => Some(context),
                        Err(error) => return self.finish_async_execution(data, Err(error), base),
                    };
                    let state = match self.object_work(span, |objects, budget| {
                        objects.store_async_execution(state.as_ref(), Box::new(data), budget)
                    }) {
                        Ok(state) => state,
                        Err(error) => return Err(self.abort_async_execution(base, error)),
                    };
                    return match self.await_native_value(
                        &state,
                        AsyncResumeKind::Execution,
                        value,
                        span,
                    ) {
                        Ok(()) => Ok(AsyncBodyOutcome::Suspended),
                        Err(error) => {
                            let mut data = self
                                .objects
                                .take_async_execution(&state)
                                .map_err(Error::InvalidObject)?
                                .expect("active source await");
                            data.context = None;
                            self.finish_async_execution(*data, Err(error), base)
                        }
                    };
                }
            }
        }
    }

    fn async_execution_step(&mut self, data: &mut AsyncExecutionData) -> Result<Step, Error> {
        match &data.body {
            ArrowBody::Expression(expression) => {
                debug_assert_eq!(data.index, 0);
                data.index += 1;
                if let Some((argument, span)) = direct_await(expression) {
                    self.tick(expression.span)?;
                    self.expression(argument)
                        .map(|value| Step::Await(value, AsyncAwaitAction::Return, span))
                } else {
                    self.expression(expression).map(|value| {
                        Step::Complete(Completion {
                            kind: CompletionKind::Return,
                            value: Some(value),
                            target: None,
                        })
                    })
                }
            }
            ArrowBody::Block(body) => {
                let Some(statement) = body.statements().get(data.index) else {
                    return Ok(Step::Complete(Completion::normal(None)));
                };
                data.index += 1;
                if matches!(
                    statement.kind,
                    StatementKind::Lexical { .. } | StatementKind::Using(_)
                ) {
                    self.tick(statement.span)?;
                    return self.async_binding_step(data, data.index - 1, 0, None);
                }
                let awaiting = match &statement.kind {
                    StatementKind::Expression(expression) => direct_await(expression)
                        .map(|(argument, span)| (argument, span, AsyncAwaitAction::Discard)),
                    StatementKind::Return(Some(expression)) => direct_await(expression)
                        .map(|(argument, span)| (argument, span, AsyncAwaitAction::Return)),
                    _ => None,
                };
                if let Some((argument, span, action)) = awaiting {
                    // Match expression/statement entry work before evaluating the
                    // operand. Nested Await forms retain their explicit gap.
                    self.tick(statement.span)?;
                    self.tick(span)?;
                    return self
                        .expression(argument)
                        .map(|value| Step::Await(value, action, span));
                }
                let completion = self.statement(statement)?;
                Ok(if completion.kind == CompletionKind::Normal {
                    Step::Continue
                } else {
                    Step::Complete(completion)
                })
            }
        }
    }

    fn async_binding_step(
        &mut self,
        data: &AsyncExecutionData,
        statement: usize,
        first: usize,
        mut resumed: Option<Value>,
    ) -> Result<Step, Error> {
        let ArrowBody::Block(body) = &data.body else {
            unreachable!("block declaration");
        };
        let (bindings, using) = match &body.statements()[statement].kind {
            StatementKind::Lexical { bindings, .. } => (bindings, false),
            StatementKind::Using(bindings) => (bindings, true),
            _ => unreachable!("declaration continuation"),
        };
        let environment = data.root_scope.as_ref().expect("body lexical scope");
        for (element, binding) in bindings.iter().enumerate().skip(first) {
            if let Some(value) = resumed.take() {
                if using {
                    // AddDisposableResource still precedes binding initialization.
                    self.initialize_using_value(&binding.pattern, value, environment)?;
                } else {
                    self.initialize_pattern(&binding.pattern, value, environment)?;
                }
            } else if let Some((argument, span)) =
                binding.initializer.as_ref().and_then(direct_await)
            {
                self.tick(binding.pattern.span)?;
                self.tick(span)?;
                // AwaitExpression is not an anonymous function definition: its
                // fulfilled function values do not receive binding-name inference.
                let value = self.expression(argument)?;
                return Ok(Step::Await(
                    value,
                    AsyncAwaitAction::Binding { statement, element },
                    span,
                ));
            } else if using {
                self.initialize_using_bindings(std::slice::from_ref(binding))?;
            } else {
                self.initialize_bindings(std::slice::from_ref(binding))?;
            }
        }
        Ok(Step::Continue)
    }

    fn capture_async_execution_context(
        &mut self,
        base: usize,
        span: Span,
    ) -> Result<AsyncExecutionContext, Error> {
        let count = self.scopes.len() - base + 1;
        self.object_work(span, |_, budget| budget.charge(count))?;
        let mut scopes = Vec::new();
        scopes.try_reserve(count).map_err(|_| Error::Limit {
            span,
            message: "async context allocation failed".into(),
        })?;
        scopes.push(self.scopes.first().expect("global environment").clone());
        scopes.extend(self.scopes[base..].iter().cloned());
        Ok(AsyncExecutionContext {
            scopes,
            variable: self.variable_environment.clone(),
            strict: self.strict,
        })
    }

    fn abort_async_execution(&mut self, base: usize, error: Error) -> Error {
        // Registration prepaid resource release. Host aborts clear captures even
        // at budget zero and never invoke JavaScript cleanup.
        for environment in &self.scopes[base..] {
            self.objects
                .take_environment_disposable_resources(environment)
                .expect("active async scope");
        }
        error
    }

    fn finish_async_execution(
        &mut self,
        data: AsyncExecutionData,
        completion: Result<Completion, Error>,
        base: usize,
    ) -> Result<AsyncBodyOutcome, Error> {
        let completion = match completion {
            Err(error) if !error.is_language_exception() => {
                return Err(self.abort_async_execution(base, error));
            }
            completion => completion,
        };
        let completion = if let Some(environment) = &data.root_scope {
            self.dispose_environment_resources(environment, completion, data.span)?
        } else {
            completion?
        };
        Ok(AsyncBodyOutcome::Complete(match completion.kind {
            CompletionKind::Return => completion.value.expect("return value"),
            CompletionKind::Normal => Value::Undefined,
            CompletionKind::Break | CompletionKind::Continue => {
                unreachable!("validated function control target")
            }
        }))
    }

    pub(super) fn resume_async_execution(
        &mut self,
        state: ObjectHandle,
        fulfilled: bool,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let Some(mut data) = self
            .objects
            .take_async_execution(&state)
            .map_err(Error::InvalidObject)?
        else {
            // A host-aborted await may still own an already registered reaction.
            return Ok(Value::Undefined);
        };
        let context = data.context.take().expect("suspended source context");
        let capability = data.capability.clone();
        let caller = AsyncExecutionContext {
            scopes: std::mem::replace(&mut self.scopes, context.scopes),
            variable: std::mem::replace(&mut self.variable_environment, context.variable),
            strict: std::mem::replace(&mut self.strict, context.strict),
        };
        let completion = if fulfilled {
            Ok(value)
        } else {
            Err(Error::Thrown(value))
        };
        let outcome = self.drive_async_execution(*data, Some(state), Some(completion), 1);
        self.scopes = caller.scopes;
        self.variable_environment = caller.variable;
        self.strict = caller.strict;
        match outcome {
            Ok(AsyncBodyOutcome::Suspended) => (),
            Ok(AsyncBodyOutcome::Complete(value)) => {
                self.call(capability.resolve, Value::Undefined, vec![value], span)?;
            }
            Err(error) => {
                let reason = self.language_exception_value(error)?;
                self.call(capability.reject, Value::Undefined, vec![reason], span)?;
            }
        }
        Ok(Value::Undefined)
    }
}

fn direct_await(mut expression: &Expr) -> Option<(&Expr, Span)> {
    while let ExprKind::Parenthesized(inner) = &expression.kind {
        expression = inner;
    }
    if let ExprKind::Await(argument) = &expression.kind {
        Some((argument, expression.span))
    } else {
        None
    }
}
