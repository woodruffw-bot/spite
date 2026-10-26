//! Persistent evaluation status, errors, cycle roots and top-level capability.

use super::*;
use crate::function::PromiseCapability;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ModuleExecutionStatus {
    Evaluating,
    Evaluated,
    Aborted,
}

#[derive(Debug)]
pub(crate) struct ModuleExecution {
    pub status: ModuleExecutionStatus,
    pub cycle_root: Option<Handle>,
    pub error: Option<Value>,
    // Only host failures enter this slot; owning language values use error.
    pub host_failure: Option<crate::Error>,
    pub capability: Option<PromiseCapability>,
}

impl ModuleExecution {
    pub(super) fn values(&self) -> impl Iterator<Item = &Value> {
        self.error
            .iter()
            .chain(self.capability.iter().flat_map(PromiseCapability::values))
    }
}

impl Objects {
    pub(crate) fn module_execution(
        &self,
        record: &Handle,
    ) -> Result<Option<&ModuleExecution>, Error> {
        Ok(self
            .inspect(record)?
            .module_record
            .as_ref()
            .ok_or(Error::WrongKind)?
            .execution
            .as_deref())
    }
    pub(crate) fn module_execution_mut(
        &mut self,
        record: &Handle,
    ) -> Result<&mut ModuleExecution, Error> {
        self.object_mut(record)?
            .module_record
            .as_mut()
            .ok_or(Error::WrongKind)?
            .execution
            .as_deref_mut()
            .ok_or(Error::WrongKind)
    }
    pub(crate) fn start_module_execution(
        &mut self,
        record: &Handle,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        budget.charge(1)?;
        let data = self
            .object_mut(record)?
            .module_record
            .as_mut()
            .ok_or(Error::WrongKind)?;
        if data.execution.is_some() || data.environment.is_none() {
            return Err(Error::WrongKind);
        }
        data.execution = Some(Box::new(ModuleExecution {
            status: ModuleExecutionStatus::Evaluating,
            cycle_root: None,
            error: None,
            host_failure: None,
            capability: None,
        }));
        Ok(())
    }
    pub(crate) fn store_module_top_level_capability(
        &mut self,
        record: &Handle,
        capability: PromiseCapability,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        budget.charge(2)?;
        let data = self
            .object_mut(record)?
            .module_record
            .as_mut()
            .ok_or(Error::WrongKind)?;
        if data.evaluation.is_some() {
            return Err(Error::WrongKind);
        }
        if data.environment.is_none() {
            return Err(Error::WrongKind);
        }
        let execution = data.execution.get_or_insert_with(|| {
            Box::new(ModuleExecution {
                status: ModuleExecutionStatus::Evaluating,
                cycle_root: None,
                error: None,
                host_failure: None,
                capability: None,
            })
        });
        data.evaluation = Some(capability.promise.clone());
        execution.capability = Some(capability);
        Ok(())
    }
}
