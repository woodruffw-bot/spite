//! Module namespace export names and links (10.4.6).

use super::{Budget, Error, Objects, OrdinaryObject, PropertyKey, PropertyKeyRef};
use crate::environment::EnvironmentHandle;
use spite_core::JsString;
use spite_heap::Handle;

#[derive(Debug)]
pub(super) struct NamespaceData {
    pub record: Handle,
    exports: Vec<(JsString, String)>,
}

impl NamespaceData {
    fn binding(&self, key: PropertyKeyRef<'_>, budget: &mut Budget) -> Result<Option<&str>, Error> {
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
            .map(|index| self.exports[index].1.as_str()))
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
        mut exports: Vec<(JsString, String)>,
        budget: &mut Budget,
    ) -> Result<Handle, Error> {
        self.module_source(record)?;
        for (name, local) in &exports {
            budget.charge(name.len())?;
            budget.charge(local.len() + 1)?;
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
    ) -> Result<Option<(EnvironmentHandle, String)>, Error> {
        let Some(data) = &self.inspect(object)?.module_namespace else {
            return Ok(None);
        };
        let Some(local) = data.binding(key, budget)? else {
            return Ok(None);
        };
        budget.charge(local.len() + 1)?;
        let environment = self
            .module_environment(&data.record)?
            .ok_or(Error::WrongKind)?;
        Ok(Some((environment, local.to_owned())))
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
