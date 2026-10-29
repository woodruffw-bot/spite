//! Traced host import capabilities and their eventual target Module record.

use super::*;
use crate::function::PromiseCapability;

#[derive(Debug)]
pub(crate) struct ModuleImportData {
    pub capability: PromiseCapability,
    pub module: Option<Handle>,
}

impl ModuleImportData {
    pub(super) fn values(&self) -> impl Iterator<Item = &Value> {
        self.capability.values()
    }

    pub(super) fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        self.values()
            .flat_map(Trace::trace)
            .chain(self.module.iter().map(Some))
    }
}

impl Objects {
    pub(crate) fn create_module_import(
        &mut self,
        capability: PromiseCapability,
        budget: &mut Budget,
    ) -> Result<Handle, Error> {
        for value in capability.values() {
            budget.value(value)?;
        }
        let state = self.create(None)?;
        self.object_mut(&state)?.module_import = Some(Box::new(ModuleImportData {
            capability,
            module: None,
        }));
        Ok(state)
    }

    pub(crate) fn set_module_import_target(
        &mut self,
        state: &Handle,
        module: &Handle,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        budget.charge(1)?;
        self.module_source(module)?;
        let data = self
            .object_mut(state)?
            .module_import
            .as_mut()
            .ok_or(Error::WrongKind)?;
        if data.module.is_some() {
            return Err(Error::WrongKind);
        }
        data.module = Some(module.clone());
        Ok(())
    }

    pub(crate) fn take_module_import(
        &mut self,
        state: &Handle,
        budget: &mut Budget,
    ) -> Result<ModuleImportData, Error> {
        budget.charge(1)?;
        self.object_mut(state)?
            .module_import
            .take()
            .map(|data| *data)
            .ok_or(Error::WrongKind)
    }
}
