//! Disposable resources and their strong captures (sec-disposablestack-objects).

use super::{Budget, Error, Objects};
use crate::Value;
use spite_heap::Handle;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DisposableStackKind {
    Sync,
    Async,
}

impl DisposableStackKind {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Sync => "DisposableStack",
            Self::Async => "AsyncDisposableStack",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DisposalHint {
    Sync,
    Async,
    AsyncFromSync,
}

#[derive(Debug)]
pub(crate) struct DisposableResource {
    pub hint: DisposalHint,
    pub value: Value,
    pub method: Value,
    // Adopt's unobservable closure is represented by its captured argument.
    pub argument: Option<Value>,
}

impl DisposableResource {
    pub(in crate::object) fn values(&self) -> impl Iterator<Item = &Value> {
        [&self.value, &self.method]
            .into_iter()
            .chain(&self.argument)
    }
}

#[derive(Debug)]
pub(super) struct DisposableStackData {
    pub(super) kind: DisposableStackKind,
    pub(super) disposed: bool,
    // A private heap record gives the List stable identity across reentrant move.
    resources: Handle,
}

#[derive(Debug, Default)]
pub(super) struct DisposableResourceList {
    pub(super) entries: Vec<DisposableResource>,
}

impl DisposableStackData {
    pub(super) fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        std::iter::once(Some(&self.resources))
    }
}

impl Objects {
    fn create_disposable_resource_list(&mut self) -> Result<Handle, Error> {
        let resources = self.create(None)?;
        self.object_mut(&resources)?.disposable_resources = Some(Box::default());
        Ok(resources)
    }

    pub(crate) fn push_environment_disposable_resource(
        &mut self,
        environment: &crate::environment::EnvironmentHandle,
        resource: DisposableResource,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        let resources =
            if let Some(resources) = &self.environment(environment)?.disposable_resources {
                resources.clone()
            } else {
                let resources = self.create_disposable_resource_list()?;
                self.environment_mut(environment)?.disposable_resources = Some(resources.clone());
                resources
            };
        self.push_disposable_resource(&resources, resource, budget)
    }

    pub(crate) fn take_environment_disposable_resources(
        &mut self,
        environment: &crate::environment::EnvironmentHandle,
    ) -> Result<Vec<DisposableResource>, Error> {
        // Registration prepays release. Scope exit must discard captures even
        // after a host work abort, without allocating or consuming more work.
        let Some(resources) = self
            .environment_mut(environment)?
            .disposable_resources
            .take()
        else {
            return Ok(Vec::new());
        };
        let list = self
            .object_mut(&resources)?
            .disposable_resources
            .as_mut()
            .ok_or(Error::WrongKind)?;
        Ok(std::mem::take(&mut list.entries))
    }

    pub(crate) fn create_disposable_stack(
        &mut self,
        prototype: &Handle,
        kind: DisposableStackKind,
    ) -> Result<Handle, Error> {
        self.inspect(prototype)?;
        let resources = self.create_disposable_resource_list()?;
        let stack = self.create(Some(prototype))?;
        self.object_mut(&stack)?.disposable_stack = Some(Box::new(DisposableStackData {
            kind,
            disposed: false,
            resources,
        }));
        Ok(stack)
    }

    pub(crate) fn disposable_resource_list(&self, stack: &Handle) -> Result<Handle, Error> {
        Ok(self
            .inspect(stack)?
            .disposable_stack
            .as_ref()
            .ok_or(Error::WrongKind)?
            .resources
            .clone())
    }

    pub(crate) fn disposable_stack_state(
        &self,
        stack: &Handle,
        kind: DisposableStackKind,
    ) -> Result<Option<bool>, Error> {
        Ok(self
            .inspect(stack)?
            .disposable_stack
            .as_ref()
            .filter(|data| data.kind == kind)
            .map(|data| data.disposed))
    }

    pub(crate) fn push_disposable_resource(
        &mut self,
        resources: &Handle,
        resource: DisposableResource,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        // Charge storage and eventual release here; taking the list is constant work.
        budget.charge(4)?;
        for value in resource.values() {
            if let Value::Object(object) = value {
                self.inspect(object)?;
            }
            budget.value(value)?;
        }
        let resources = self
            .object_mut(resources)?
            .disposable_resources
            .as_mut()
            .ok_or(Error::WrongKind)?;
        let resources = &mut resources.entries;
        if resources.len() == resources.capacity() {
            budget.charge(resources.len())?;
        }
        resources
            .try_reserve(1)
            .map_err(|_| spite_heap::Error::Capacity)?;
        resources.push(resource);
        Ok(())
    }

    pub(crate) fn take_disposable_resources(
        &mut self,
        stack: &Handle,
        budget: &mut Budget,
    ) -> Result<Vec<DisposableResource>, Error> {
        budget.charge(1)?;
        let data = self
            .object_mut(stack)?
            .disposable_stack
            .as_mut()
            .ok_or(Error::WrongKind)?;
        data.disposed = true;
        let resources = data.resources.clone();
        let resources = self
            .object_mut(&resources)?
            .disposable_resources
            .as_mut()
            .ok_or(Error::WrongKind)?;
        Ok(std::mem::take(&mut resources.entries))
    }

    pub(crate) fn move_disposable_resources(
        &mut self,
        source: &Handle,
        target: &Handle,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        let kind = self
            .inspect(source)?
            .disposable_stack
            .as_ref()
            .ok_or(Error::WrongKind)?
            .kind;
        if self.disposable_stack_state(target, kind)? != Some(false) {
            return Err(Error::WrongKind);
        }
        budget.charge(1)?;
        let replacement = self.disposable_resource_list(target)?;
        let source_data = self
            .object_mut(source)?
            .disposable_stack
            .as_mut()
            .ok_or(Error::WrongKind)?;
        source_data.disposed = true;
        let resources = std::mem::replace(&mut source_data.resources, replacement);
        self.object_mut(target)?
            .disposable_stack
            .as_mut()
            .ok_or(Error::WrongKind)?
            .resources = resources;
        Ok(())
    }
}
