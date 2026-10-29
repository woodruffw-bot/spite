//! Direct source Await statements and returns (Await, sec-asyncblockstart).

mod binding;
mod binding_object;
mod binding_pattern;
mod block;
mod class_definition;
mod completion;
mod conditional;
mod delegation;
mod expression;
mod for_await;
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
        async_execution::{AsyncAwaitAction, AsyncExecutionData, SourceFunctionMode},
        async_expression::ExprRoot,
    },
};
use spite_core::Span;
use spite_parser::ast::{ArrowBody, StatementKind};

pub(super) enum AsyncBodyOutcome {
    Complete(Value),
    Suspended,
    // Sync generators carry their iterator result; async generators carry the
    // awaited value so CompleteStep creates its result after dequeuing.
    Yielded(Value),
}

pub(super) enum SourceResume {
    Value(Value),
    Throw(Error),
    Return(Value),
}

enum Step {
    Continue,
    Complete(Completion),
    Finish(Completion),
    Await(Value, AsyncAwaitAction, Span),
    Yield(Value, AsyncAwaitAction, Span),
}

enum FrameStep {
    Continue,
    Complete(Result<Completion, Error>),
}

fn new_execution_data(
    body: ArrowBody,
    function: SourceFunctionMode,
    root_scope: Option<crate::environment::EnvironmentHandle>,
    span: Span,
) -> AsyncExecutionData {
    AsyncExecutionData {
        function,
        body,
        expression: None,
        pattern_binding: None,
        class_context: None,
        class_definition: None,
        expression_classes: Vec::new(),
        index: 0,
        statement: None,
        next: None,
        frames: Vec::new(),
        normal: None,
        pending: AsyncAwaitAction::Discard,
        root_scope,
        var_target: None,
        iteration_value: None,
        context: None,
        span,
    }
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
        let data = new_execution_data(
            body,
            SourceFunctionMode::Async(capability),
            root_scope,
            span,
        );
        self.drive_async_execution(data, None, None, base)
    }

    pub(super) fn start_generator_execution(
        &mut self,
        body: spite_parser::ast::FunctionBody,
        root: crate::environment::EnvironmentHandle,
        function: SourceFunctionMode,
        span: Span,
        state: ObjectHandle,
    ) -> Result<AsyncBodyOutcome, Error> {
        let data = new_execution_data(ArrowBody::Block(body), function, Some(root), span);
        self.drive_async_execution(data, Some(state), None, 1)
    }

    fn with_module_context<T>(
        &mut self,
        root: crate::environment::EnvironmentHandle,
        execute: impl FnOnce(&mut Self) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let context = crate::object::source_context::SourceExecutionContext {
            scopes: vec![
                self.scopes.first().expect("global environment").clone(),
                root.clone(),
            ],
            variable: Some(root.clone()),
            strict: true,
            module: match &self
                .objects
                .environment(&root)
                .map_err(Error::InvalidObject)?
                .this
            {
                Some(crate::environment::ThisBinding::Module(record)) => Some(record.clone()),
                _ => unreachable!("Module environment"),
            },
        };
        self.with_source_execution_context(context, execute)
    }

    pub(crate) fn instantiate_module_body(
        &mut self,
        body: &spite_parser::ast::FunctionBody,
        root: crate::environment::EnvironmentHandle,
        default_export: Option<usize>,
        record: &ObjectHandle,
        span: Span,
    ) -> Result<(), Error> {
        self.with_module_context(root.clone(), |realm| {
            realm.instantiate(body.statements().iter(), false, true)?;
            realm.prepare_module_default(body, &root, default_export)?;
            realm.object_work(span, |objects, budget| {
                objects.store_module_environment(record, root, budget)
            })
        })
    }

    pub(crate) fn execute_async_module_body(
        &mut self,
        body: spite_parser::ast::FunctionBody,
        root: crate::environment::EnvironmentHandle,
        default_export: Option<usize>,
        record: &ObjectHandle,
        span: Span,
    ) -> Result<(), Error> {
        self.with_module_context(root.clone(), |realm| {
            realm
                .call_async_body(span, |realm, capability, _| {
                    let Value::Object(promise) = &capability.promise else {
                        unreachable!("intrinsic module body Promise")
                    };
                    realm.react_to_native_promise(
                        promise,
                        record,
                        super::async_native::AsyncResumeKind::ModuleExecution,
                        span,
                    )?;
                    let data = new_execution_data(
                        ArrowBody::Block(body),
                        SourceFunctionMode::Module {
                            capability: capability.clone(),
                            default_export,
                            record: record.clone(),
                        },
                        Some(root),
                        span,
                    );
                    realm.drive_async_execution(data, None, None, 1)
                })
                .map(|_| ())
        })
    }

    pub(crate) fn execute_sync_module_body(
        &mut self,
        body: spite_parser::ast::FunctionBody,
        root: crate::environment::EnvironmentHandle,
        default_export: Option<usize>,
        span: Span,
    ) -> Result<(), Error> {
        self.with_module_context(root.clone(), |realm| {
            let data = new_execution_data(
                ArrowBody::Block(body),
                SourceFunctionMode::SyncModule { default_export },
                Some(root),
                span,
            );
            match realm.drive_async_execution(data, None, None, 1)? {
                AsyncBodyOutcome::Complete(_) => Ok(()),
                AsyncBodyOutcome::Suspended | AsyncBodyOutcome::Yielded(_) => {
                    unreachable!("Module HasTLA is false")
                }
            }
        })
    }

    pub(super) fn resume_generator_execution(
        &mut self,
        data: AsyncExecutionData,
        state: ObjectHandle,
        resumed: SourceResume,
    ) -> Result<AsyncBodyOutcome, Error> {
        self.drive_async_execution(data, Some(state), Some(resumed), 1)
    }

    fn drive_async_execution(
        &mut self,
        mut data: AsyncExecutionData,
        mut state: Option<ObjectHandle>,
        resumed: Option<SourceResume>,
        base: usize,
    ) -> Result<AsyncBodyOutcome, Error> {
        let mut next_step = resumed.map(|completion| {
            if !matches!(data.pending, AsyncAwaitAction::DelegationResult(_))
                && data
                    .expression
                    .as_ref()
                    .is_some_and(|state| state.delegation.is_some())
            {
                return self.resume_yield_delegation(&mut data, completion);
            }
            match completion {
                SourceResume::Throw(error) => Err(error),
                SourceResume::Return(value) => Ok(Step::Complete(Completion {
                    kind: CompletionKind::Return,
                    value: Some(value),
                    target: None,
                })),
                SourceResume::Value(value) => {
                    let action = data.pending;
                    self.complete_async_value(&mut data, action, value)
                }
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
                Ok(Step::Yield(result, action, span)) => {
                    debug_assert!(data.function.generator().is_some());
                    data.pending = action;
                    data.context = match self.capture_source_execution_context(
                        base,
                        span,
                        "generator context allocation failed",
                    ) {
                        Ok(context) => Some(context),
                        Err(error) => return self.finish_async_execution(data, Err(error), base),
                    };
                    let generator = state.as_ref().expect("source generator state");
                    match self.object_work(span, |objects, budget| {
                        if data.function.is_async_generator() {
                            objects
                                .store_async_execution(Some(generator), Box::new(data), budget)
                                .map(|_| ())
                        } else {
                            objects.store_generator_execution(generator, Box::new(data), budget)
                        }
                    }) {
                        Ok(()) => return Ok(AsyncBodyOutcome::Yielded(result)),
                        Err(error) => return Err(self.abort_async_execution(base, error)),
                    }
                }
                Ok(Step::Await(value, action, span)) => {
                    data.pending = action;
                    data.context = match self.capture_source_execution_context(
                        base,
                        span,
                        "async context allocation failed",
                    ) {
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
        if matches!(
            data.frames.last(),
            Some(crate::object::async_execution::AsyncStatementFrame::CatchBinding { .. })
        ) {
            let Some(crate::object::async_execution::AsyncStatementFrame::CatchBinding {
                environment,
                value,
            }) = data.frames.pop()
            else {
                unreachable!("pending catch binding")
            };
            return self.start_async_pattern_binding(
                data,
                crate::object::async_binding::BindingRoot::Catch,
                value,
                Some(environment),
            );
        }
        if data.pattern_binding.is_some() {
            return self.advance_async_pattern_binding(data);
        }
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

    fn async_module_default_step(&mut self, data: &mut AsyncExecutionData) -> Result<Step, Error> {
        let statement = active_statement(&data.body, data.statement.as_ref());
        self.tick(statement.span)?;
        let value = match &statement.kind {
            StatementKind::Function(_) => return Ok(Step::Complete(Completion::normal(None))),
            StatementKind::Class(class) => {
                if class_definition::has_class_await(class) {
                    return self.start_class_heritage(data);
                }
                self.class_definition(class, Some(spite_core::JsString::from("default").into()))?
            }
            StatementKind::Expression(expression)
                if expression::has_supported_await(expression) =>
            {
                return self.start_async_expression(
                    data,
                    ExprRoot::Statement,
                    AsyncAwaitAction::ModuleDefault,
                );
            }
            StatementKind::Expression(expression) => {
                self.named_expression(expression, spite_core::JsString::from("default"))?
            }
            _ => unreachable!("synthetic default declaration"),
        };
        self.complete_async_value(data, AsyncAwaitAction::ModuleDefault, value)
    }

    fn async_source_statement(&mut self, data: &mut AsyncExecutionData) -> Result<Step, Error> {
        if let Some(index) = data.function.module_default()
            && data
                .statement
                .as_ref()
                .is_some_and(|location| location.root == index && location.children.is_empty())
        {
            return self.async_module_default_step(data);
        }
        let statement = active_statement(&data.body, data.statement.as_ref());
        if let StatementKind::Class(class) = &statement.kind
            && class_definition::has_class_await(class)
        {
            self.tick(statement.span)?;
            return self.start_class_heritage(data);
        }
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
                if data.function.is_async_generator()
                    || expression::has_supported_await(expression) =>
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
            AsyncAwaitAction::ClassHeritage => self.finish_class_heritage(data, value),
            AsyncAwaitAction::ClassElement { element } => {
                self.finish_async_class_key(data, element, value)
            }
            AsyncAwaitAction::ModuleDefault => {
                let environment = data.root_scope.as_ref().expect("module environment");
                let span = active_statement(&data.body, data.statement.as_ref()).span;
                self.initialize_module_default(environment, value, span)?;
                Ok(Step::Complete(Completion::normal(None)))
            }
            AsyncAwaitAction::Return if data.function.is_async_generator() => {
                // ReturnStatement awaits its expression in async generators
                // before finalizers run (14.10, GetGeneratorKind).
                Ok(Step::Await(
                    value,
                    AsyncAwaitAction::AsyncGeneratorReturn,
                    data.span,
                ))
            }
            AsyncAwaitAction::DelegationResult(operation) => {
                self.finish_yield_delegation(data, value, operation)
            }
            AsyncAwaitAction::AsyncGeneratorYield(span) => {
                Ok(Step::Yield(value, AsyncAwaitAction::Expression, span))
            }
            AsyncAwaitAction::Return | AsyncAwaitAction::AsyncGeneratorReturn => {
                Ok(Step::Complete(Completion {
                    kind: CompletionKind::Return,
                    value: Some(value),
                    target: None,
                }))
            }
            AsyncAwaitAction::Throw => Err(Error::Thrown(value)),
            AsyncAwaitAction::WithObject => self.enter_async_with(data, value),
            AsyncAwaitAction::SwitchDiscriminant => self.enter_async_switch(data, value),
            AsyncAwaitAction::SwitchSelector => self.finish_async_switch_selector(data, value),
            AsyncAwaitAction::IfTest => self.finish_async_if(data, value),
            AsyncAwaitAction::LoopTest => self.finish_async_loop_test(data, value),
            AsyncAwaitAction::ForAwaitNext => self.finish_async_for_await_next(data, value),
            AsyncAwaitAction::ForAwaitClose => self
                .finish_async_for_await_close(data, Some(Ok(value)))
                .map(Step::Complete),
            AsyncAwaitAction::ForHeader => self.finish_async_for_header(data, value),
            AsyncAwaitAction::BindingDefault => self.resume_async_binding_default(data, value),
            AsyncAwaitAction::BindingKey => self.resume_async_binding_key(data, value),
            AsyncAwaitAction::IterationBinding => {
                data.iteration_value = None;
                Ok(Step::Continue)
            }
            AsyncAwaitAction::IterationSource => self.finish_async_iteration_source(data, value),
            AsyncAwaitAction::Binding { element } => {
                self.async_binding_step(data, element, Some(value))
            }
            AsyncAwaitAction::Expression => self.resume_async_expression(data, Some(value)),
        }
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
        if let Some(record) = data.function.module_record()
            && self
                .objects
                .module_execution(record)
                .map_err(Error::InvalidObject)?
                .is_some_and(|execution| {
                    execution.status == crate::object::ModuleExecutionStatus::Aborted
                })
        {
            for environment in &data
                .context
                .as_ref()
                .expect("suspended source context")
                .scopes[1..]
            {
                self.objects
                    .take_environment_disposable_resources(environment)
                    .map_err(Error::InvalidObject)?;
            }
            return Ok(Value::Undefined);
        }
        let context = data.context.take().expect("suspended source context");
        let asynchronous_generator = data.function.is_async_generator();
        let module = data.function.module_record().cloned();
        let capability = data.function.promise().cloned();
        let completion = if fulfilled {
            SourceResume::Value(value)
        } else {
            SourceResume::Throw(Error::Thrown(value))
        };
        let outcome = self.with_source_execution_context(context, |realm| {
            realm.drive_async_execution(*data, Some(state.clone()), Some(completion), 1)
        });
        if asynchronous_generator {
            return self.finish_async_generator_execution(state, outcome, span);
        }
        let capability = capability.expect("async source capability");
        let result = match outcome {
            Ok(AsyncBodyOutcome::Yielded(_)) => unreachable!("async body cannot yield"),
            Ok(AsyncBodyOutcome::Suspended) => Ok(Value::Undefined),
            Ok(AsyncBodyOutcome::Complete(value)) => {
                self.call(capability.resolve, Value::Undefined, vec![value], span)
            }
            Err(error) => self.language_exception_value(error).and_then(|reason| {
                self.call(capability.reject, Value::Undefined, vec![reason], span)
            }),
        };
        if let (Some(record), Err(error)) = (&module, &result)
            && !error.is_language_exception()
        {
            self.abort_module_evaluation(record, error)?;
        }
        result?;
        Ok(Value::Undefined)
    }
}
