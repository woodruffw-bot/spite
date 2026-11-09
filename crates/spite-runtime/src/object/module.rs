//! Retained Source Text and JSON Synthetic Module records (16.2.1).

use super::{Budget, Error, Objects};
use crate::{Value, environment::EnvironmentHandle};
use spite_core::JsString;
use spite_heap::{Handle, Trace};
use spite_parser::ast::{LocalExportEntry, Module, ModuleEntries, ModuleRequest};

mod evaluation;
pub(crate) use evaluation::{ModuleExecution, ModuleExecutionStatus};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ModuleLinkState {
    Unlinked,
    Linking,
    Linked,
}

#[derive(Debug)]
enum ModuleSource {
    SourceText(Module),
    Json(Value),
}

impl ModuleSource {
    fn json_value(&self) -> Option<&Value> {
        match self {
            Self::SourceText(_) => None,
            Self::Json(value) => Some(value),
        }
    }
}

#[derive(Debug)]
pub(super) struct ModuleData {
    source: ModuleSource,
    realm: EnvironmentHandle,
    environment: Option<EnvironmentHandle>,
    evaluation: Option<Value>,
    namespace: Option<Value>,
    import_meta: Option<Value>,
    dependencies: Option<Vec<Handle>>,
    link_state: ModuleLinkState,
    execution: Option<Box<ModuleExecution>>,
}

impl ModuleData {
    pub(super) fn values(&self) -> impl Iterator<Item = &Value> {
        self.source.json_value().into_iter().chain(
            self.evaluation
                .iter()
                .chain(self.namespace.iter())
                .chain(self.import_meta.iter())
                .chain(
                    self.execution
                        .iter()
                        .flat_map(|execution| execution.values()),
                ),
        )
    }

    pub(super) fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        self.environment
            .iter()
            .map(|environment| Some(&environment.0))
            .chain(std::iter::once(Some(&self.realm.0)))
            .chain(self.values().flat_map(Trace::trace))
            .chain(self.dependencies.iter().flatten().map(Some))
            .chain(
                self.execution
                    .iter()
                    .flat_map(|execution| execution.async_parents.iter())
                    .map(Some),
            )
            .chain(
                self.execution
                    .iter()
                    .filter_map(|execution| execution.cycle_root.as_ref())
                    .map(Some),
            )
    }
}

impl Objects {
    pub(crate) fn module_import_meta(
        &mut self,
        record: &Handle,
        budget: &mut Budget,
    ) -> Result<Value, Error> {
        budget.charge(1)?;
        let data = self
            .inspect(record)?
            .module_record
            .as_ref()
            .ok_or(Error::WrongKind)?;
        if let Some(value) = &data.import_meta {
            return Ok(value.clone());
        }
        // 13.3.12: default HostGetImportMetaProperties is empty and
        // HostFinalizeImportMeta performs no further operations.
        let value = Value::Object(self.create(None)?);
        self.object_mut(record)?
            .module_record
            .as_mut()
            .ok_or(Error::WrongKind)?
            .import_meta = Some(value.clone());
        Ok(value)
    }

    pub(crate) fn module_link_state(&self, record: &Handle) -> Result<ModuleLinkState, Error> {
        Ok(self
            .inspect(record)?
            .module_record
            .as_ref()
            .ok_or(Error::WrongKind)?
            .link_state)
    }
    pub(crate) fn begin_module_link(
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
        if data.link_state != ModuleLinkState::Unlinked {
            return Err(Error::WrongKind);
        };
        data.link_state = ModuleLinkState::Linking;
        Ok(())
    }
    pub(crate) fn finish_module_links(
        &mut self,
        records: &[Handle],
        budget: &mut Budget,
    ) -> Result<(), Error> {
        budget.charge(records.len())?;
        for record in records {
            if self.module_link_state(record)? != ModuleLinkState::Linking
                || self.module_environment(record)?.is_none()
            {
                return Err(Error::WrongKind);
            };
        }
        for record in records {
            self.object_mut(record)?
                .module_record
                .as_mut()
                .ok_or(Error::WrongKind)?
                .link_state = ModuleLinkState::Linked;
        }
        Ok(())
    }
    pub(crate) fn rollback_module_links(&mut self, records: &[Handle]) -> Result<(), Error> {
        for record in records {
            let data = self
                .object_mut(record)?
                .module_record
                .as_mut()
                .ok_or(Error::WrongKind)?;
            if data.link_state == ModuleLinkState::Linking {
                data.link_state = ModuleLinkState::Unlinked;
                data.environment = None;
            }
        }
        Ok(())
    }
    pub(crate) fn store_module_imports(
        &mut self,
        environment: &EnvironmentHandle,
        imports: std::collections::BTreeMap<String, crate::module::ModuleBinding>,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        budget.charge(imports.len())?;
        for binding in imports.values() {
            self.module_realm(&binding.module)?;
        }
        self.environment_mut(environment)?.imports = imports;
        Ok(())
    }

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
                self.module_realm(dependency)?;
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
        self.create_module_record_source(ModuleSource::SourceText(source))
    }

    pub(crate) fn create_json_module_record(
        &mut self,
        value: Value,
        budget: &mut Budget,
    ) -> Result<Handle, Error> {
        budget.value(&value)?;
        self.create_module_record_source(ModuleSource::Json(value))
    }

    fn create_module_record_source(&mut self, source: ModuleSource) -> Result<Handle, Error> {
        // Source Text Module [[Realm]] is fixed at ParseModule, before linking
        // has created its environment (16.2.1.6). The factory hint is checked.
        let realm = self.active_function_realm()?.ok_or(Error::WrongKind)?;
        let record = self.create(None)?;
        self.object_mut(&record)?.module_record = Some(Box::new(ModuleData {
            source,
            realm,
            environment: None,
            evaluation: None,
            namespace: None,
            import_meta: None,
            dependencies: None,
            link_state: ModuleLinkState::Unlinked,
            execution: None,
        }));
        Ok(record)
    }

    pub(crate) fn module_realm(&self, record: &Handle) -> Result<&EnvironmentHandle, Error> {
        Ok(&self
            .inspect(record)?
            .module_record
            .as_ref()
            .ok_or(Error::WrongKind)?
            .realm)
    }

    pub(crate) fn module_source(&self, record: &Handle) -> Result<&Module, Error> {
        let source = &self
            .inspect(record)?
            .module_record
            .as_ref()
            .ok_or(Error::WrongKind)?
            .source;
        match source {
            ModuleSource::SourceText(source) => Ok(source),
            ModuleSource::Json(_) => Err(Error::WrongKind),
        }
    }

    pub(crate) fn module_json_value(&self, record: &Handle) -> Result<Option<&Value>, Error> {
        Ok(self
            .inspect(record)?
            .module_record
            .as_ref()
            .ok_or(Error::WrongKind)?
            .source
            .json_value())
    }

    pub(crate) fn module_entries(&self, record: &Handle) -> Result<ModuleEntries, Error> {
        if self.module_json_value(record)?.is_some() {
            return Ok(ModuleEntries {
                requested_modules: Vec::new(),
                import_entries: Vec::new(),
                local_export_entries: vec![LocalExportEntry {
                    local: JsString::from("default"),
                    exported: JsString::from("default"),
                }],
                indirect_export_entries: Vec::new(),
                star_export_entries: Vec::new(),
            });
        }
        Ok(self.module_source(record)?.entries())
    }

    pub(crate) fn module_declaration_count(&self, record: &Handle) -> Result<usize, Error> {
        if self.module_json_value(record)?.is_some() {
            return Ok(1);
        }
        let source = self.module_source(record)?;
        Ok(source.imports().len() + source.exports().len())
    }

    pub(crate) fn module_requests(&self, record: &Handle) -> Result<Vec<ModuleRequest>, Error> {
        if self.module_json_value(record)?.is_some() {
            return Ok(Vec::new());
        }
        Ok(self.module_source(record)?.requested_modules())
    }

    pub(crate) fn module_has_top_level_await(&self, record: &Handle) -> Result<bool, Error> {
        if self.module_json_value(record)?.is_some() {
            return Ok(false);
        }
        Ok(self.module_source(record)?.has_top_level_await())
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
        let data = self
            .object_mut(record)?
            .module_record
            .as_mut()
            .ok_or(Error::WrongKind)?;
        data.environment = Some(environment);
        Ok(())
    }
}
