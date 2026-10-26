//! Retained Source Text Module records (16.2.1).

use super::{Budget, Error, Objects};
use crate::{Value, environment::EnvironmentHandle};
use spite_heap::{Handle, Trace};
use spite_parser::ast::Module;

#[derive(Debug)]
pub(super) struct ModuleData {
    source: Module,
    environment: Option<EnvironmentHandle>,
    evaluation: Option<Value>,
    namespace: Option<Value>,
    dependencies: Option<Vec<Handle>>,
}

impl ModuleData {
    pub(super) fn values(&self) -> impl Iterator<Item = &Value> {
        self.evaluation.iter().chain(self.namespace.iter())
    }

    pub(super) fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        self.environment
            .iter()
            .map(|environment| Some(&environment.0))
            .chain(self.values().flat_map(Trace::trace))
            .chain(self.dependencies.iter().flatten().map(Some))
    }
}

impl Objects {
    pub(crate) fn module_dependencies(&self, record: &Handle) -> Result<Option<&[Handle]>, Error> {
        Ok(self
            .inspect(record)?
            .module_record
            .as_ref()
            .ok_or(Error::WrongKind)?
            .dependencies
            .as_deref())
    }
    pub(crate) fn install_module_graph(
        &mut self,
        graph: Vec<(Handle, Vec<Handle>)>,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        for (record, dependencies) in &graph {
            budget.charge(dependencies.len().checked_add(1).ok_or(Error::WorkLimit)?)?;
            if self.module_dependencies(record)?.is_some() {
                return Err(Error::WrongKind);
            };
            for dependency in dependencies {
                self.module_source(dependency)?;
            }
        }
        for (record, dependencies) in graph {
            self.object_mut(&record)?
                .module_record
                .as_mut()
                .ok_or(Error::WrongKind)?
                .dependencies = Some(dependencies);
        }
        Ok(())
    }

    pub(crate) fn module_environment(
        &self,
        record: &Handle,
    ) -> Result<Option<EnvironmentHandle>, Error> {
        Ok(self
            .inspect(record)?
            .module_record
            .as_ref()
            .ok_or(Error::WrongKind)?
            .environment
            .clone())
    }
    pub(crate) fn module_namespace(&self, record: &Handle) -> Result<Option<Value>, Error> {
        Ok(self
            .inspect(record)?
            .module_record
            .as_ref()
            .ok_or(Error::WrongKind)?
            .namespace
            .clone())
    }
    pub(crate) fn store_module_namespace(
        &mut self,
        record: &Handle,
        namespace: Handle,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        budget.charge(1)?;
        if !self.inspect(&namespace)?.is_module_namespace() {
            return Err(Error::WrongKind);
        };
        self.object_mut(record)?
            .module_record
            .as_mut()
            .ok_or(Error::WrongKind)?
            .namespace = Some(Value::Object(namespace));
        Ok(())
    }

    pub(crate) fn create_module_record(
        &mut self,
        source: Module,
        budget: &mut Budget,
    ) -> Result<Handle, Error> {
        budget.charge(source.statements().len())?;
        budget.charge(source.imports().len())?;
        budget.charge(source.exports().len())?;
        let record = self.create(None)?;
        self.object_mut(&record)?.module_record = Some(Box::new(ModuleData {
            source,
            environment: None,
            evaluation: None,
            namespace: None,
            dependencies: None,
        }));
        Ok(record)
    }

    pub(crate) fn module_source(&self, record: &Handle) -> Result<&Module, Error> {
        Ok(&self
            .inspect(record)?
            .module_record
            .as_ref()
            .ok_or(Error::WrongKind)?
            .source)
    }

    pub(crate) fn module_evaluation(&self, record: &Handle) -> Result<Option<Value>, Error> {
        Ok(self
            .inspect(record)?
            .module_record
            .as_ref()
            .ok_or(Error::WrongKind)?
            .evaluation
            .clone())
    }

    pub(crate) fn store_module_environment(
        &mut self,
        record: &Handle,
        environment: EnvironmentHandle,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        budget.charge(1)?;
        self.environment(&environment)?;
        self.object_mut(record)?
            .module_record
            .as_mut()
            .ok_or(Error::WrongKind)?
            .environment = Some(environment);
        Ok(())
    }

    pub(crate) fn begin_module_evaluation(
        &mut self,
        record: &Handle,
        promise: Value,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        budget.value(&promise)?;
        let Value::Object(handle) = &promise else {
            return Err(Error::WrongKind);
        };
        if self.inspect(handle)?.promise_state().is_none() {
            return Err(Error::WrongKind);
        };
        let data = self
            .object_mut(record)?
            .module_record
            .as_mut()
            .ok_or(Error::WrongKind)?;
        if data.evaluation.is_some() || data.environment.is_none() {
            return Err(Error::WrongKind);
        };
        data.evaluation = Some(promise);
        Ok(())
    }
}
