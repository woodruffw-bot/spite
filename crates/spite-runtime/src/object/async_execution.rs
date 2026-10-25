//! Traced source async-function state between Await jobs (27.7.5.3).

use super::{Budget, Error, Objects};
use crate::{Value, environment::EnvironmentHandle, function::PromiseCapability};
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
}

#[derive(Debug)]
pub(crate) struct AsyncExecutionData {
    pub capability: PromiseCapability,
    pub body: ArrowBody,
    pub index: usize,
    pub pending: AsyncAwaitAction,
    pub root_scope: Option<EnvironmentHandle>,
    pub context: Option<AsyncExecutionContext>,
    pub span: Span,
}

impl AsyncExecutionData {
    pub(super) fn values(&self) -> impl Iterator<Item = &Value> {
        self.capability.values()
    }

    fn environments(&self) -> impl Iterator<Item = &EnvironmentHandle> {
        self.root_scope.iter().chain(
            self.context
                .iter()
                .flat_map(|context| context.scopes.iter().chain(&context.variable)),
        )
    }

    pub(super) fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        self.values()
            .flat_map(Trace::trace)
            .chain(self.environments().map(|environment| Some(&environment.0)))
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
