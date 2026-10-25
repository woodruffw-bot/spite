//! Traced source async-function state between Await jobs (27.7.5.3).

use super::{Budget, Error, Objects};
use crate::{
    Completion, Value,
    environment::EnvironmentHandle,
    function::{IteratorRecord, PromiseCapability},
};
use spite_core::Span;
use spite_heap::{Handle, Trace};
use spite_parser::ast::ArrowBody;

#[derive(Debug)]
pub(crate) struct AsyncExecutionContext {
    pub scopes: Vec<EnvironmentHandle>,
    pub variable: Option<EnvironmentHandle>,
    pub strict: bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum AsyncAwaitAction {
    Discard,
    Return,
    Throw,
    Expression,
    IfTest,
    LoopTest,
    ForOfSource,
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

#[derive(Debug)]
pub(crate) struct AsyncForOfHead {
    pub location: StatementLocation,
    pub environment: Option<EnvironmentHandle>,
}

#[derive(Debug)]
pub(crate) struct AsyncForOfCursor {
    pub location: StatementLocation,
    pub iterator: IteratorRecord,
    pub value: Value,
}

#[derive(Debug)]
pub(crate) enum AsyncStatementFrame {
    Block(AsyncBlockCursor),
    If,
    Try(AsyncTryCursor),
    Scope {
        environment: EnvironmentHandle,
        span: Span,
        disposable: bool,
    },
    Loop(AsyncLoopCursor),
    ForOfHead(AsyncForOfHead),
    ForOf(AsyncForOfCursor),
}

impl AsyncStatementFrame {
    fn environment(&self) -> Option<&EnvironmentHandle> {
        match self {
            Self::Block(block) => Some(&block.environment),
            Self::If => None,
            Self::Try(_) => None,
            Self::Scope { environment, .. } => Some(environment),
            Self::Loop(_) => None,
            Self::ForOfHead(head) => head.environment.as_ref(),
            Self::ForOf(_) => None,
        }
    }

    fn values(&self) -> impl Iterator<Item = &Value> {
        let value = match self {
            Self::Block(block) => block.normal.as_ref(),
            Self::If => None,
            Self::Try(cursor) => match cursor.pending.as_ref() {
                Some(Ok(completion)) => completion.value.as_ref(),
                Some(Err(crate::Error::Thrown(value))) => Some(value),
                _ => None,
            },
            Self::Scope { .. } => None,
            Self::Loop(cursor) => Some(&cursor.value),
            Self::ForOfHead(_) => None,
            Self::ForOf(cursor) => Some(&cursor.value),
        };
        let next = match self {
            Self::ForOf(cursor) => Some(cursor.iterator.next_value()),
            _ => None,
        };
        value.into_iter().chain(next)
    }

    fn iterator(&self) -> Option<&Handle> {
        match self {
            Self::ForOf(cursor) => Some(cursor.iterator.iterator_handle()),
            Self::Block(_)
            | Self::If
            | Self::Try(_)
            | Self::Scope { .. }
            | Self::Loop(_)
            | Self::ForOfHead(_) => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct AsyncExecutionData {
    pub capability: PromiseCapability,
    pub body: ArrowBody,
    pub expression: Option<super::async_expression::AsyncExpressionData>,
    pub index: usize,
    pub statement: Option<StatementLocation>,
    pub next: Option<StatementLocation>,
    pub frames: Vec<AsyncStatementFrame>,
    pub normal: Option<Value>,
    pub pending: AsyncAwaitAction,
    pub root_scope: Option<EnvironmentHandle>,
    pub var_target: Option<EnvironmentHandle>,
    pub context: Option<AsyncExecutionContext>,
    pub span: Span,
}

impl AsyncExecutionData {
    pub(super) fn values(&self) -> impl Iterator<Item = &Value> {
        self.capability
            .values()
            .chain(&self.normal)
            .chain(self.frames.iter().flat_map(AsyncStatementFrame::values))
            .chain(
                self.expression
                    .iter()
                    .flat_map(|expression| expression.values()),
            )
    }

    fn iterators(&self) -> impl Iterator<Item = &Handle> {
        self.frames.iter().filter_map(AsyncStatementFrame::iterator)
    }

    fn environments(&self) -> impl Iterator<Item = &EnvironmentHandle> {
        self.root_scope
            .iter()
            .chain(&self.var_target)
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
            .chain(self.iterators().map(Some))
    }
}

impl Objects {
    pub(crate) fn store_async_execution(
        &mut self,
        state: Option<&Handle>,
        data: Box<AsyncExecutionData>,
        budget: &mut Budget,
    ) -> Result<Handle, Error> {
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
        for iterator in data.iterators() {
            self.inspect(iterator)?;
        }
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

    pub(crate) fn take_async_execution(
        &mut self,
        state: &Handle,
    ) -> Result<Option<Box<AsyncExecutionData>>, Error> {
        Ok(self.object_mut(state)?.async_execution.take())
    }
}
