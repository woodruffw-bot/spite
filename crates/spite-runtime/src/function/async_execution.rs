//! Direct source Await statements and returns (Await, sec-asyncblockstart).

mod binding;
mod block;
mod completion;
mod conditional;
mod expression;
mod for_in;
mod for_loop;
mod for_of;
mod iteration;
mod iteration_head;
mod labelled;
mod protected;
mod source;
mod switch;
mod with;

use source::{active_statement, child_location};

use super::{
    PromiseCapability, ScriptFunction, async_native::AsyncResumeKind, ordinary::OrdinaryInvocation,
};
use crate::{
    Completion, CompletionKind, Error, ObjectHandle, Realm, Value,
    object::{
        async_execution::{AsyncAwaitAction, AsyncExecutionContext, AsyncExecutionData},
        async_expression::ExprRoot,
    },
};
use spite_core::Span;
use spite_parser::ast::{ArrowBody, StatementKind};

pub(super) enum AsyncBodyOutcome {
    Complete(Value),
    Suspended,
}

enum Step {
    Continue,
    Complete(Completion),
    Finish(Completion),
    Await(Value, AsyncAwaitAction, Span),
}

enum FrameStep {
    Continue,
    Complete(Result<Completion, Error>),
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
            expression: None,
            index: 0,
            statement: None,
            next: None,
            frames: Vec::new(),
            normal: None,
            pending: AsyncAwaitAction::Discard,
            root_scope,
            var_target: None,
            context: None,
            span,
        };
        self.drive_async_execution(data, None, None, base)
    }

    fn drive_async_execution(
        &mut self,
        mut data: AsyncExecutionData,
        mut state: Option<ObjectHandle>,
        resumed: Option<Result<Value, Error>>,
        base: usize,
    ) -> Result<AsyncBodyOutcome, Error> {
        let mut next_step = resumed.map(|completion| match completion {
            Err(error) => Err(error),
            Ok(value) => {
                let action = data.pending;
                self.complete_async_value(&mut data, action, value)
            }
        });
        loop {
            let step = next_step
                .take()
                .unwrap_or_else(|| self.async_execution_step(&mut data));
            let completion = match step {
                Ok(Step::Continue) => continue,
                Ok(Step::Finish(completion)) => {
                    return self.finish_async_execution(data, Ok(completion), base);
                }
                Ok(Step::Complete(completion)) => Ok(completion),
                Err(error) => Err(error),
                Ok(Step::Await(value, action, span)) => {
                    data.pending = action;
                    data.context = match self.capture_async_execution_context(base, span) {
                        Ok(context) => Some(context),
                        Err(error) => return self.finish_async_execution(data, Err(error), base),
                    };
                    let handle = match self.object_work(span, |objects, budget| {
                        objects.store_async_execution(state.as_ref(), Box::new(data), budget)
                    }) {
                        Ok(handle) => handle,
                        Err(error) => return Err(self.abort_async_execution(base, error)),
                    };
                    match self.await_native_value(&handle, AsyncResumeKind::Execution, value, span)
                    {
                        Ok(()) => return Ok(AsyncBodyOutcome::Suspended),
                        Err(error) => {
                            let mut restored = self
                                .objects
                                .take_async_execution(&handle)
                                .map_err(Error::InvalidObject)?
                                .expect("active source await");
                            restored.context = None;
                            data = *restored;
                            state = Some(handle);
                            next_step = Some(Err(error));
                            continue;
                        }
                    }
                }
            };
            match self.handle_async_completion(&mut data, completion) {
                Ok(None) => (),
                Ok(Some(completion)) => {
                    return self.finish_async_execution(data, Ok(completion), base);
                }
                Err(error) => return self.finish_async_execution(data, Err(error), base),
            }
        }
    }

    fn async_execution_step(&mut self, data: &mut AsyncExecutionData) -> Result<Step, Error> {
        match &data.body {
            ArrowBody::Expression(expression) => {
                debug_assert_eq!(data.index, 0);
                data.index += 1;
                if expression::has_supported_await(expression) {
                    self.start_async_expression(data, ExprRoot::Arrow, AsyncAwaitAction::Return)
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
            ArrowBody::Block(_) => self.async_block_step(data),
        }
    }

    fn async_source_statement(&mut self, data: &mut AsyncExecutionData) -> Result<Step, Error> {
        let statement = active_statement(&data.body, data.statement.as_ref());
        if matches!(statement.kind, StatementKind::Block(_)) {
            let span = statement.span;
            self.tick(span)?;
            return self.enter_async_block(data, span);
        }
        if matches!(statement.kind, StatementKind::Labelled { .. }) {
            let span = statement.span;
            self.tick(span)?;
            return self.enter_async_label(data, span);
        }
        if matches!(statement.kind, StatementKind::For { .. }) {
            let span = statement.span;
            self.tick(span)?;
            return self.enter_async_for(data, span);
        }
        if matches!(statement.kind, StatementKind::ForOf { awaited: true, .. }) {
            self.tick(statement.span)?;
            return Err(Self::unsupported(
                statement.span,
                "for-await-of execution is not implemented",
            ));
        }
        if matches!(
            statement.kind,
            StatementKind::ForOf { .. } | StatementKind::ForIn { .. }
        ) {
            let span = statement.span;
            self.tick(span)?;
            return self.enter_async_iteration(data, span);
        }
        if matches!(
            statement.kind,
            StatementKind::While { .. } | StatementKind::DoWhile { .. }
        ) {
            let span = statement.span;
            self.tick(span)?;
            return self.enter_async_loop(data, span);
        }
        if matches!(statement.kind, StatementKind::Try { .. }) {
            let span = statement.span;
            self.tick(span)?;
            return self.enter_async_try(data, span);
        }
        if let StatementKind::With { object, .. } = &statement.kind {
            self.tick(statement.span)?;
            if expression::has_supported_await(object) {
                return self.start_async_expression(
                    data,
                    ExprRoot::WithObject,
                    AsyncAwaitAction::WithObject,
                );
            }
            let value = self.expression(object)?;
            return self.enter_async_with(data, value);
        }
        if let StatementKind::Switch { discriminant, .. } = &statement.kind {
            self.tick(statement.span)?;
            if expression::has_supported_await(discriminant) {
                return self.start_async_expression(
                    data,
                    ExprRoot::SwitchDiscriminant,
                    AsyncAwaitAction::SwitchDiscriminant,
                );
            }
            let input = self.expression(discriminant)?;
            return self.enter_async_switch(data, input);
        }
        if let StatementKind::If { test, .. } = &statement.kind {
            self.tick(statement.span)?;
            if expression::has_supported_await(test) {
                return self.start_async_expression(
                    data,
                    ExprRoot::IfTest,
                    AsyncAwaitAction::IfTest,
                );
            }
            let value = self.expression(test)?;
            return self.finish_async_if(data, value);
        }
        if matches!(
            statement.kind,
            StatementKind::Lexical { .. } | StatementKind::Using(_) | StatementKind::Var(_)
        ) {
            self.tick(statement.span)?;
            return self.async_binding_step(data, 0, None);
        }
        let action = match &statement.kind {
            StatementKind::Expression(expression)
                if expression::has_supported_await(expression) =>
            {
                Some(AsyncAwaitAction::Discard)
            }
            StatementKind::Return(Some(expression))
                if expression::has_supported_await(expression) =>
            {
                Some(AsyncAwaitAction::Return)
            }
            StatementKind::Throw(expression) if expression::has_supported_await(expression) => {
                Some(AsyncAwaitAction::Throw)
            }
            _ => None,
        };
        if let Some(action) = action {
            self.tick(statement.span)?;
            return self.start_async_expression(data, ExprRoot::Statement, action);
        }
        self.statement(statement).map(Step::Complete)
    }

    fn complete_async_value(
        &mut self,
        data: &mut AsyncExecutionData,
        action: AsyncAwaitAction,
        value: Value,
    ) -> Result<Step, Error> {
        match action {
            AsyncAwaitAction::Discard => Ok(Step::Complete(Completion::normal(Some(value)))),
            AsyncAwaitAction::Return => Ok(Step::Complete(Completion {
                kind: CompletionKind::Return,
                value: Some(value),
                target: None,
            })),
            AsyncAwaitAction::Throw => Err(Error::Thrown(value)),
            AsyncAwaitAction::WithObject => self.enter_async_with(data, value),
            AsyncAwaitAction::SwitchDiscriminant => self.enter_async_switch(data, value),
            AsyncAwaitAction::SwitchSelector => self.finish_async_switch_selector(data, value),
            AsyncAwaitAction::IfTest => self.finish_async_if(data, value),
            AsyncAwaitAction::LoopTest => self.finish_async_loop_test(data, value),
            AsyncAwaitAction::ForHeader => self.finish_async_for_header(data, value),
            AsyncAwaitAction::IterationSource => self.finish_async_iteration_source(data, value),
            AsyncAwaitAction::Binding { element } => {
                self.async_binding_step(data, element, Some(value))
            }
            AsyncAwaitAction::Expression => self.resume_async_expression(data, Some(value)),
        }
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
        debug_assert!(data.frames.is_empty());
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
