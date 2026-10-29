//! Traced source async-function state between Await jobs (27.7.5.3).

use super::{Budget, Error, Objects, source_context::SourceExecutionContext};
use crate::{
    Completion, Reference, Value,
    environment::EnvironmentHandle,
    function::{IteratorRecord, PromiseCapability},
};
use spite_core::{JsSymbol, Span};
use spite_heap::{Handle, Trace};
use spite_parser::ast::ArrowBody;

#[derive(Clone, Copy, Debug)]
pub(crate) enum AsyncAwaitAction {
    Discard,
    Return,
    AsyncGeneratorReturn,
    AsyncGeneratorYield(Span),
    ModuleDefault,
    ClassHeritage,
    ClassElement { element: usize },
    DelegationResult(super::async_expression::DelegationOperation),
    Throw,
    Expression,
    IfTest,
    WithObject,
    SwitchDiscriminant,
    SwitchSelector,
    LoopTest,
    ForHeader,
    ForAwaitNext,
    ForAwaitClose,
    IterationSource,
    IterationBinding,
    BindingDefault,
    BindingKey,
    Binding { element: usize },
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum StatementChild {
    Block(usize),
    IfConsequent,
    IfAlternate,
    TryBody,
    CatchBody,
    FinallyBody,
    LoopBody,
    ForOfBody,
    ForBody,
    ForInBody,
    LabelBody,
    WithBody,
    SwitchBody { clause: usize, statement: usize },
}

#[derive(Debug)]
pub(crate) struct StatementLocation {
    pub root: usize,
    pub children: Vec<StatementChild>,
}

#[derive(Debug)]
pub(crate) struct AsyncBlockCursor {
    pub location: StatementLocation,
    pub index: usize,
    pub environment: EnvironmentHandle,
    pub span: Span,
    pub normal: Option<Value>,
}

#[derive(Debug)]
pub(crate) struct AsyncTryCursor {
    pub location: StatementLocation,
    pub phase: AsyncTryPhase,
    pub pending: Option<Result<Completion, crate::Error>>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum AsyncTryPhase {
    Body,
    Catch,
    Finally,
}

#[derive(Debug)]
pub(crate) struct AsyncLoopCursor {
    pub location: StatementLocation,
    pub phase: AsyncLoopPhase,
    pub value: Value,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum AsyncLoopPhase {
    Test,
    Body,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum AsyncForPhase {
    Initialize,
    Test,
    Body,
    Update,
}

#[derive(Debug)]
pub(crate) struct AsyncForCursor {
    pub location: StatementLocation,
    pub phase: AsyncForPhase,
    pub environment: Option<EnvironmentHandle>,
    pub disposable: bool,
    pub value: Value,
}

#[derive(Debug)]
pub(crate) struct AsyncIterationHead {
    pub location: StatementLocation,
    pub environment: Option<EnvironmentHandle>,
}

#[derive(Debug)]
pub(crate) struct AsyncForOfCursor {
    pub location: StatementLocation,
    pub iterator: IteratorRecord,
    pub value: Value,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum AsyncForAwaitPhase {
    Next,
    Body,
    Close,
}

#[derive(Debug)]
pub(crate) struct AsyncForAwaitCursor {
    pub location: StatementLocation,
    pub iterator: IteratorRecord,
    pub value: Value,
    pub phase: AsyncForAwaitPhase,
    pub pending: Option<Result<Completion, crate::Error>>,
}

#[derive(Debug)]
pub(crate) struct AsyncForInCursor {
    pub location: StatementLocation,
    pub iterator: crate::for_in::ForInIterator,
    pub value: Value,
}

#[derive(Debug)]
pub(crate) struct AsyncSwitchCursor {
    pub location: StatementLocation,
    pub environment: EnvironmentHandle,
    pub clause: usize,
    pub statement: usize,
    pub default: Option<usize>,
    pub input: Option<Value>,
    pub value: Value,
}

#[derive(Debug)]
pub(crate) enum AsyncStatementFrame {
    Block(AsyncBlockCursor),
    Switch(AsyncSwitchCursor),
    With(EnvironmentHandle),
    If,
    CatchBinding {
        environment: EnvironmentHandle,
        value: Value,
    },
    Try(AsyncTryCursor),
    Scope {
        environment: EnvironmentHandle,
        span: Span,
        disposable: bool,
    },
    Loop(AsyncLoopCursor),
    For(AsyncForCursor),
    IterationHead(AsyncIterationHead),
    ForOf(AsyncForOfCursor),
    ForAwait(AsyncForAwaitCursor),
    ForIn(AsyncForInCursor),
    Label(StatementLocation),
}

impl AsyncStatementFrame {
    fn environment(&self) -> Option<&EnvironmentHandle> {
        match self {
            Self::Block(block) => Some(&block.environment),
            Self::Switch(cursor) => Some(&cursor.environment),
            Self::With(environment) => Some(environment),
            Self::CatchBinding { environment, .. } => Some(environment),
            Self::If => None,
            Self::Try(_) => None,
            Self::Scope { environment, .. } => Some(environment),
            Self::Loop(_) => None,
            Self::For(cursor) => cursor.environment.as_ref(),
            Self::IterationHead(head) => head.environment.as_ref(),
            Self::ForOf(_) | Self::ForIn(_) | Self::ForAwait(_) => None,
            Self::Label(_) => None,
        }
    }

    fn values(&self) -> impl Iterator<Item = &Value> {
        let value = match self {
            Self::Block(block) => block.normal.as_ref(),
            Self::Switch(cursor) => Some(&cursor.value),
            Self::With(_) => None,
            Self::CatchBinding { value, .. } => Some(value),
            Self::If => None,
            Self::Try(cursor) => match cursor.pending.as_ref() {
                Some(Ok(completion)) => completion.value.as_ref(),
                Some(Err(crate::Error::Thrown(value))) => Some(value),
                _ => None,
            },
            Self::Scope { .. } => None,
            Self::Loop(cursor) => Some(&cursor.value),
            Self::For(cursor) => Some(&cursor.value),
            Self::IterationHead(_) => None,
            Self::ForOf(cursor) => Some(&cursor.value),
            Self::ForAwait(cursor) => Some(&cursor.value),
            Self::ForIn(cursor) => Some(&cursor.value),
            Self::Label(_) => None,
        };
        let next = match self {
            Self::ForOf(cursor) => Some(cursor.iterator.next_value()),
            Self::ForAwait(cursor) => Some(cursor.iterator.next_value()),
            Self::Switch(cursor) => cursor.input.as_ref(),
            _ => None,
        };
        let pending = match self {
            Self::ForAwait(cursor) => match cursor.pending.as_ref() {
                Some(Ok(completion)) => completion.value.as_ref(),
                Some(Err(crate::Error::Thrown(value))) => Some(value),
                _ => None,
            },
            _ => None,
        };
        value.into_iter().chain(next).chain(pending)
    }

    fn object(&self) -> Option<&Handle> {
        match self {
            Self::ForOf(cursor) => Some(cursor.iterator.iterator_handle()),
            Self::ForAwait(cursor) => Some(cursor.iterator.iterator_handle()),
            Self::ForIn(cursor) => cursor.iterator.object_handle(),
            Self::Block(_)
            | Self::Switch(_)
            | Self::With(_)
            | Self::If
            | Self::CatchBinding { .. }
            | Self::Try(_)
            | Self::Scope { .. }
            | Self::Loop(_)
            | Self::For(_)
            | Self::IterationHead(_)
            | Self::Label(_) => None,
        }
    }
}

#[derive(Debug)]
pub(crate) enum SourceFunctionMode {
    Async(PromiseCapability),
    Module {
        capability: PromiseCapability,
        default_export: Option<usize>,
        record: Handle,
    },
    SyncModule {
        default_export: Option<usize>,
    },
    Generator(Handle),
    AsyncGenerator(Handle),
}

impl SourceFunctionMode {
    pub(crate) fn module_record(&self) -> Option<&Handle> {
        match self {
            Self::Module { record, .. } => Some(record),
            _ => None,
        }
    }
    pub(crate) fn module_default(&self) -> Option<usize> {
        match self {
            Self::Module { default_export, .. } | Self::SyncModule { default_export } => {
                *default_export
            }
            _ => None,
        }
    }

    pub(crate) fn is_async_generator(&self) -> bool {
        matches!(self, Self::AsyncGenerator(_))
    }
    pub(crate) fn promise(&self) -> Option<&PromiseCapability> {
        match self {
            Self::Async(capability) | Self::Module { capability, .. } => Some(capability),
            Self::Generator(_) | Self::AsyncGenerator(_) | Self::SyncModule { .. } => None,
        }
    }
    pub(crate) fn generator(&self) -> Option<&Handle> {
        match self {
            Self::Generator(callee) | Self::AsyncGenerator(callee) => Some(callee),
            Self::Async(_) | Self::Module { .. } | Self::SyncModule { .. } => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct AsyncExecutionData {
    pub function: SourceFunctionMode,
    pub body: ArrowBody,
    pub expression: Option<super::async_expression::AsyncExpressionData>,
    pub pattern_binding: Option<super::async_binding::AsyncPatternBinding>,
    pub class_context: Option<crate::function::ClassDefinitionContext>,
    pub class_definition: Option<crate::function::ClassDefinitionState>,
    pub expression_classes: Vec<super::async_expression::AsyncClassExpression>,
    pub assignment_patterns: Vec<super::async_assignment::AsyncAssignmentPattern>,
    pub index: usize,
    pub statement: Option<StatementLocation>,
    pub next: Option<StatementLocation>,
    pub frames: Vec<AsyncStatementFrame>,
    pub normal: Option<Value>,
    pub pending: AsyncAwaitAction,
    pub root_scope: Option<EnvironmentHandle>,
    pub var_target: Option<Reference<'static>>,
    pub iteration_value: Option<Value>,
    pub context: Option<SourceExecutionContext>,
    pub span: Span,
}

impl AsyncExecutionData {
    fn pattern_cursors(&self) -> impl Iterator<Item = &super::async_binding::BindingCursor> {
        self.pattern_binding
            .iter()
            .flat_map(|state| &state.cursors)
            .chain(
                self.assignment_patterns
                    .iter()
                    .flat_map(|state| &state.cursors),
            )
    }

    pub(super) fn symbols(&self) -> impl Iterator<Item = &JsSymbol> {
        self.expression
            .iter()
            .flat_map(|expression| expression.symbols())
            .chain(
                self.expression_classes
                    .iter()
                    .flat_map(|state| state.symbols()),
            )
            .chain(
                self.class_definition
                    .iter()
                    .flat_map(|state| state.symbols()),
            )
            .chain(self.var_target.iter().flat_map(Reference::symbols))
            .chain(self.pattern_cursors().flat_map(|cursor| cursor.symbols()))
    }

    pub(super) fn values(&self) -> impl Iterator<Item = &Value> {
        self.function
            .promise()
            .into_iter()
            .flat_map(PromiseCapability::values)
            .chain(&self.normal)
            .chain(&self.iteration_value)
            .chain(self.assignment_patterns.iter().map(|state| &state.original))
            .chain(self.pattern_cursors().flat_map(|cursor| cursor.values()))
            .chain(self.var_target.iter().flat_map(Reference::values))
            .chain(self.frames.iter().flat_map(AsyncStatementFrame::values))
            .chain(
                self.expression
                    .iter()
                    .flat_map(|expression| expression.values()),
            )
    }

    fn objects(&self) -> impl Iterator<Item = &Handle> {
        self.function
            .generator()
            .into_iter()
            .chain(self.function.module_record())
            .chain(
                self.expression_classes
                    .iter()
                    .flat_map(|state| state.objects()),
            )
            .chain(
                self.class_definition
                    .iter()
                    .flat_map(|state| state.objects()),
            )
            .chain(
                self.context
                    .iter()
                    .filter_map(|context| context.module.as_ref()),
            )
            .chain(self.frames.iter().filter_map(AsyncStatementFrame::object))
            .chain(self.pattern_cursors().flat_map(|cursor| cursor.objects()))
            .chain(self.var_target.iter().filter_map(Reference::object))
            .chain(
                self.expression
                    .iter()
                    .flat_map(|expression| expression.objects()),
            )
    }

    fn environments(&self) -> impl Iterator<Item = &EnvironmentHandle> {
        self.expression
            .iter()
            .flat_map(|expression| expression.environments())
            .chain(
                self.expression_classes
                    .iter()
                    .flat_map(|state| state.environments()),
            )
            .chain(
                self.class_definition
                    .iter()
                    .flat_map(|state| state.environments()),
            )
            .chain(self.root_scope.iter())
            .chain(
                self.class_context
                    .iter()
                    .map(|context| &context.environment),
            )
            .chain(
                self.pattern_binding
                    .iter()
                    .flat_map(|state| state.environment.iter()),
            )
            .chain(
                self.pattern_cursors().filter_map(|cursor| {
                    cursor.reference.as_ref().and_then(Reference::environment)
                }),
            )
            .chain(self.var_target.iter().filter_map(Reference::environment))
            .chain(
                self.frames
                    .iter()
                    .filter_map(AsyncStatementFrame::environment),
            )
            .chain(
                self.context
                    .iter()
                    .flat_map(|context| context.scopes.iter().chain(&context.variable)),
            )
    }

    pub(super) fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        self.values()
            .flat_map(Trace::trace)
            .chain(self.environments().map(|environment| Some(&environment.0)))
            .chain(self.objects().map(Some))
    }
}

impl Objects {
    pub(crate) fn async_execution_module(&self, state: &Handle) -> Result<Option<Handle>, Error> {
        Ok(self
            .inspect(state)?
            .async_execution
            .as_ref()
            .and_then(|data| data.function.module_record())
            .cloned())
    }

    pub(crate) fn store_async_execution(
        &mut self,
        state: Option<&Handle>,
        data: Box<AsyncExecutionData>,
        budget: &mut Budget,
    ) -> Result<Handle, Error> {
        self.validate_source_execution_data(&data, budget)?;
        let state = if let Some(state) = state {
            if self.inspect(state)?.async_execution.is_some() {
                return Err(Error::WrongKind);
            }
            state.clone()
        } else {
            self.create(None)?
        };
        self.object_mut(&state)?.async_execution = Some(data);
        Ok(state)
    }

    pub(crate) fn validate_source_execution_data(
        &self,
        data: &AsyncExecutionData,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        debug_assert!(data.context.is_some());
        for value in data.values() {
            budget.value(value)?;
            if let Value::Object(object) = value {
                self.inspect(object)?;
            }
        }
        for environment in data.environments() {
            self.environment(environment)?;
        }
        for object in data.objects() {
            self.inspect(object)?;
        }
        Ok(())
    }

    pub(crate) fn take_async_execution(
        &mut self,
        state: &Handle,
    ) -> Result<Option<Box<AsyncExecutionData>>, Error> {
        Ok(self.object_mut(state)?.async_execution.take())
    }
}
