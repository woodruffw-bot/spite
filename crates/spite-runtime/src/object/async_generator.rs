//! Async-generator requests and retained start contexts (27.6.3).

use super::{Budget, Error, Objects, generator::GeneratorStart};
use crate::{Value, function::PromiseCapability};
use spite_core::Span;
use spite_heap::{Handle, Trace};
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AsyncGeneratorStatus {
    SuspendedStart,
    SuspendedYield,
    Executing,
    DrainingQueue,
    Completed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RequestKind {
    Next,
    Return,
    Throw,
}

#[derive(Clone, Debug)]
pub(crate) struct AsyncGeneratorRequest {
    pub kind: RequestKind,
    pub value: Value,
    pub capability: PromiseCapability,
    pub span: Span,
}

impl AsyncGeneratorRequest {
    fn values(&self) -> impl Iterator<Item = &Value> {
        std::iter::once(&self.value).chain(self.capability.values())
    }
}

#[derive(Debug)]
pub(crate) struct AsyncGeneratorData {
    pub status: AsyncGeneratorStatus,
    pub start: Option<GeneratorStart>,
    pub requests: VecDeque<AsyncGeneratorRequest>,
}

impl AsyncGeneratorData {
    pub(super) fn values(&self) -> impl Iterator<Item = &Value> {
        self.requests.iter().flat_map(AsyncGeneratorRequest::values)
    }
    pub(super) fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        self.values()
            .flat_map(Trace::trace)
            .chain(self.start.iter().flat_map(|start| {
                std::iter::once(Some(&start.callee))
                    .chain(start.environments().map(|environment| Some(&environment.0)))
            }))
    }
}

impl Objects {
    pub(crate) fn create_async_generator(
        &mut self,
        prototype: &Handle,
        start: GeneratorStart,
        budget: &mut Budget,
    ) -> Result<Handle, Error> {
        budget.charge(1)?;
        self.inspect(&start.callee)?;
        for environment in start.environments() {
            self.environment(environment)?;
        }
        let object = self.create(Some(prototype))?;
        self.object_mut(&object)?.async_generator = Some(Box::new(AsyncGeneratorData {
            status: AsyncGeneratorStatus::SuspendedStart,
            start: Some(start),
            requests: VecDeque::new(),
        }));
        Ok(object)
    }
    pub(crate) fn async_generator_status(
        &self,
        generator: &Handle,
    ) -> Result<Option<AsyncGeneratorStatus>, Error> {
        Ok(self
            .inspect(generator)?
            .async_generator
            .as_ref()
            .map(|data| data.status))
    }
    pub(crate) fn set_async_generator_status(
        &mut self,
        generator: &Handle,
        status: AsyncGeneratorStatus,
    ) -> Result<(), Error> {
        let data = self
            .object_mut(generator)?
            .async_generator
            .as_deref_mut()
            .ok_or(Error::WrongKind)?;
        data.status = status;
        if matches!(
            status,
            AsyncGeneratorStatus::DrainingQueue | AsyncGeneratorStatus::Completed
        ) {
            data.start = None;
        }
        Ok(())
    }
    pub(crate) fn take_async_generator_start(
        &mut self,
        generator: &Handle,
    ) -> Result<GeneratorStart, Error> {
        let data = self
            .object_mut(generator)?
            .async_generator
            .as_deref_mut()
            .ok_or(Error::WrongKind)?;
        if data.status != AsyncGeneratorStatus::SuspendedStart {
            return Err(Error::WrongKind);
        }
        data.status = AsyncGeneratorStatus::Executing;
        data.start.take().ok_or(Error::WrongKind)
    }
    pub(crate) fn enqueue_async_generator(
        &mut self,
        generator: &Handle,
        request: AsyncGeneratorRequest,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        budget.charge(1)?;
        for value in request.values() {
            budget.value(value)?;
            if let Value::Object(object) = value {
                self.inspect(object)?;
            }
        }
        let data = self
            .object_mut(generator)?
            .async_generator
            .as_deref_mut()
            .ok_or(Error::WrongKind)?;
        if data.requests.len() == data.requests.capacity() {
            budget.charge(data.requests.len())?;
        }
        data.requests
            .try_reserve(1)
            .map_err(|_| spite_heap::Error::Capacity)?;
        data.requests.push_back(request);
        Ok(())
    }
    pub(crate) fn async_generator_front(
        &self,
        generator: &Handle,
    ) -> Result<Option<AsyncGeneratorRequest>, Error> {
        Ok(self
            .inspect(generator)?
            .async_generator
            .as_deref()
            .ok_or(Error::WrongKind)?
            .requests
            .front()
            .cloned())
    }
    pub(crate) fn pop_async_generator_request(
        &mut self,
        generator: &Handle,
    ) -> Result<AsyncGeneratorRequest, Error> {
        self.object_mut(generator)?
            .async_generator
            .as_deref_mut()
            .ok_or(Error::WrongKind)?
            .requests
            .pop_front()
            .ok_or(Error::WrongKind)
    }
    pub(crate) fn abort_async_generator(&mut self, generator: &Handle) -> Result<(), Error> {
        let object = self.object_mut(generator)?;
        let data = object
            .async_generator
            .as_deref_mut()
            .ok_or(Error::WrongKind)?;
        data.status = AsyncGeneratorStatus::Completed;
        data.start = None;
        data.requests.clear();
        Ok(())
    }
}
