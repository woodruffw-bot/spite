//! Module namespace export names and links (10.4.6).

use super::{Budget, Error, Objects, OrdinaryObject, PropertyKey, PropertyKeyRef};
use crate::module::ModuleBinding;
use spite_core::JsString;
use spite_heap::Handle;

#[derive(Debug)]
pub(super) struct NamespaceData {
    pub record: Handle,
    exports: Vec<(JsString, ModuleBinding)>,
}

impl NamespaceData {
    pub(super) fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        std::iter::once(Some(&self.record)).chain(
            self.exports
                .iter()
                .map(|(_, binding)| Some(&binding.module)),
        )
    }

    fn binding(
        &self,
        key: PropertyKeyRef<'_>,
        budget: &mut Budget,
    ) -> Result<Option<&ModuleBinding>, Error> {
        let Some(key) = key.as_string() else {
            return Ok(None);
        };
        let comparisons = self.exports.len().max(1).ilog2() as usize + 1;
        budget.charge(
            key.len()
                .checked_add(1)
                .and_then(|length| length.checked_mul(comparisons))
                .ok_or(Error::WorkLimit)?,
        )?;
        Ok(self
            .exports
            .binary_search_by(|(name, _)| name.cmp(key))
            .ok()
            .map(|index| &self.exports[index].1))
    }
}

impl OrdinaryObject {
    pub(crate) fn is_module_namespace(&self) -> bool {
        self.module_namespace.is_some()
    }
    pub(super) fn namespace_has_export(
        &self,
        key: PropertyKeyRef<'_>,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        match &self.module_namespace {
            Some(data) => Ok(data.binding(key, budget)?.is_some()),
            None => Ok(false),
        }
    }
}

impl Objects {
    pub(crate) fn create_module_namespace(
        &mut self,
        record: &Handle,
        mut exports: Vec<(JsString, ModuleBinding)>,
        budget: &mut Budget,
    ) -> Result<Handle, Error> {
        self.module_source(record)?;
        for (name, local) in &exports {
            budget.charge(name.len())?;
            budget.charge(local.name.as_ref().map_or(1, |name| name.len() + 1))?;
        }
        let sorting = exports
            .len()
            .checked_mul(exports.len().max(1).ilog2() as usize + 1)
            .ok_or(Error::WorkLimit)?;
        budget.charge(sorting)?;
        exports.sort_by(|(a, _), (b, _)| a.cmp(b));
        let object = self.create(None)?;
        let count = exports.len().checked_add(1).ok_or(Error::PropertyLimit)?;
        if self
            .inspect(&object)?
            .max_properties
            .is_some_and(|limit| count > limit)
        {
            return Err(Error::PropertyLimit);
        };
        self.object_mut(&object)?.module_namespace = Some(Box::new(NamespaceData {
            record: record.clone(),
            exports,
        }));
        Ok(object)
    }

    pub(crate) fn namespace_binding(
        &self,
        object: &Handle,
        key: PropertyKeyRef<'_>,
        budget: &mut Budget,
    ) -> Result<Option<ModuleBinding>, Error> {
        let Some(data) = &self.inspect(object)?.module_namespace else {
            return Ok(None);
        };
        let Some(binding) = data.binding(key, budget)? else {
            return Ok(None);
        };
        budget.charge(1)?;
        Ok(Some(binding.clone()))
    }

    pub(super) fn namespace_own_keys(
        &self,
        object: &Handle,
        budget: &mut Budget,
    ) -> Result<Vec<PropertyKey>, Error> {
        let object = self.inspect(object)?;
        let data = object.module_namespace.as_ref().ok_or(Error::WrongKind)?;
        let count = data
            .exports
            .len()
            .checked_add(object.properties.len())
            .ok_or(Error::WorkLimit)?;
        budget.charge(count)?;
        let mut keys = Vec::new();
        keys.try_reserve_exact(count)
            .map_err(|_| spite_heap::Error::Capacity)?;
        for (name, _) in &data.exports {
            budget.charge(name.len())?;
            keys.push(name.clone().into());
        }
        for (key, _) in &object.properties {
            if key.as_symbol().is_some() {
                keys.push(key.clone());
            }
        }
        Ok(keys)
    }
}
