//! Traced native continuation state for asynchronous resource disposal.

use super::{Budget, DisposableResource, Error, Objects};
use crate::{Value, function::PromiseCapability};
use spite_heap::{Handle, Trace};

#[derive(Debug)]
pub(crate) struct AsyncDisposeData {
    pub resources: Vec<DisposableResource>,
    pub capability: PromiseCapability,
    pub error: Option<Value>,
    pub needs_await: bool,
    pub has_awaited: bool,
}

impl AsyncDisposeData {
    pub(super) fn values(&self) -> impl Iterator<Item = &Value> {
        self.capability
            .values()
            .chain(&self.error)
            .chain(self.resources.iter().flat_map(DisposableResource::values))
    }

    pub(super) fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        std::iter::once(None).chain(self.values().flat_map(Trace::trace))
    }
}

impl Objects {
    pub(crate) fn create_async_disposal(
        &mut self,
        stack: &Handle,
        capability: PromiseCapability,
        budget: &mut Budget,
    ) -> Result<Handle, Error> {
        for value in capability.values() {
            if let Value::Object(object) = value {
                self.inspect(object)?;
            }
            budget.value(value)?;
        }
        let resources = self.take_disposable_resources(stack, budget)?;
        let state = self.create(None)?;
        self.object_mut(&state)?.async_disposal = Some(Box::new(AsyncDisposeData {
            resources,
            capability,
            error: None,
            needs_await: false,
            has_awaited: false,
        }));
        Ok(state)
    }

    pub(crate) fn async_disposal_mut(
        &mut self,
        state: &Handle,
    ) -> Result<&mut AsyncDisposeData, Error> {
        self.object_mut(state)?
            .async_disposal
            .as_deref_mut()
            .ok_or(Error::WrongKind)
    }

    pub(crate) fn finish_async_disposal(
        &mut self,
        state: &Handle,
    ) -> Result<(PromiseCapability, Option<Value>), Error> {
        let data = self
            .object_mut(state)?
            .async_disposal
            .take()
            .ok_or(Error::WrongKind)?;
        debug_assert!(data.resources.is_empty());
        Ok((data.capability, data.error))
    }
}
