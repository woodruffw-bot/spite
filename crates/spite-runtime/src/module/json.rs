//! JSON default-export Synthetic Modules (ParseJSONModule).

use super::*;
use spite_core::JsString;

impl Realm {
    /// Parses JSON into a retained Synthetic Module with one default export.
    ///
    /// Parsing uses intrinsic JSON semantics, independent of the public JSON
    /// object. Linking initializes the export to undefined; evaluation publishes
    /// the parsed value. The host still chooses supported import attributes and
    /// resolves module requests. No loader or new attribute key is enabled.
    pub fn create_json_module(&mut self, source: &str) -> Result<ModuleHandle, Error> {
        let span = Span::new(0, 0);
        if self
            .limits
            .max_source_bytes
            .is_some_and(|limit| source.len() > limit)
        {
            return Err(Error::Limit {
                span,
                message: "JSON module source exceeds the host byte limit".into(),
            });
        }
        self.initialize_realm()?;
        if !self.job_active {
            self.remaining_steps = self.limits.max_steps;
        }
        self.object_work(span, |_, budget| budget.charge(source.len()))?;
        let value = self.json_parse_text(&JsString::from(source), Value::Undefined, span)?;
        let root = self.object_work(span, |objects, budget| {
            let record = objects.create_json_module_record(value, budget)?;
            objects.root(&record, budget)
        })?;
        Ok(ModuleHandle { root })
    }

    pub(super) fn initialize_json_module_record(
        &mut self,
        record: &ObjectHandle,
        span: Span,
    ) -> Result<bool, Error> {
        if self
            .objects
            .module_json_value(record)
            .map_err(Error::InvalidObject)?
            .is_none()
        {
            return Ok(false);
        }
        let global = self
            .objects
            .module_realm(record)
            .map_err(Error::InvalidObject)?
            .clone();
        let mut bindings = BTreeMap::new();
        bindings.insert(
            "default".into(),
            BindingState {
                value: Some(Value::Undefined),
                mutable: true,
                deletable: false,
                strict: true,
            },
        );
        self.object_work(span, |objects, budget| {
            budget.charge("default".len() + 1)?;
            let environment = objects.create_environment(Some(global), bindings, budget)?;
            objects.environment_mut(&environment)?.this = Some(ThisBinding::Module(record.clone()));
            objects.store_module_environment(record, environment, budget)
        })?;
        Ok(true)
    }

    pub(super) fn execute_json_module_record(
        &mut self,
        record: &ObjectHandle,
        span: Span,
    ) -> Result<bool, Error> {
        if self
            .objects
            .module_json_value(record)
            .map_err(Error::InvalidObject)?
            .is_none()
        {
            return Ok(false);
        }
        self.object_work(span, |objects, budget| {
            let value = objects.module_json_value(record)?.expect("JSON module");
            budget.value(value)?;
            let value = value.clone();
            let environment = objects
                .module_environment(record)?
                .expect("linked JSON module");
            objects
                .environment_mut(&environment)?
                .bindings
                .get_mut("default")
                .expect("JSON default binding")
                .value = Some(value);
            Ok(())
        })?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests;
