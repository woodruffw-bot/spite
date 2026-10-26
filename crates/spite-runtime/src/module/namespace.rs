//! GetModuleNamespace and Module namespace exotic methods (10.4.6, 16.2.1).

use super::*;
use crate::object::{DataDescriptor, DataProperty, DescriptorKind, Property, PropertyDescriptor};
use spite_core::{JsString, PropertyKeyRef, WellKnownSymbol};

impl Realm {
    /// Returns a record's unique namespace, preparing bindings without body entry.
    ///
    /// Export values remain live; reading a lexical export before initialization
    /// throws ReferenceError. The returned object is unrooted. The Module handle
    /// retains it; use [`Self::root_value`] if it must outlive that handle.
    pub fn module_namespace(&mut self, module: &ModuleHandle) -> Result<Value, Error> {
        let record = module.root.handle();
        if self
            .objects
            .module_link_state(record)
            .map_err(Error::InvalidObject)?
            == crate::object::ModuleLinkState::Linked
            && let Some(namespace) = self
                .objects
                .module_namespace(record)
                .map_err(Error::InvalidObject)?
        {
            return Ok(namespace);
        }
        if !self.job_active {
            self.remaining_steps = self.limits.max_steps;
        }
        self.link_module_record(record, Span::new(0, 0))?;
        self.module_namespace_record(record, Span::new(0, 0))
    }

    pub(crate) fn module_namespace_record(
        &mut self,
        record: &ObjectHandle,
        span: Span,
    ) -> Result<Value, Error> {
        if let Some(namespace) = self
            .objects
            .module_namespace(record)
            .map_err(Error::InvalidObject)?
        {
            return Ok(namespace);
        };
        let catalog = self.module_catalog(record, span)?;
        let index = catalog.positions[record];
        let exports = self.object_work(span, |_, budget| {
            let names = catalog.export_names(index, budget)?;
            let mut exports = Vec::new();
            exports
                .try_reserve_exact(names.len())
                .map_err(|_| spite_heap::Error::Capacity)?;
            for name in names {
                if let super::resolution::ExportResolution::Binding(binding) =
                    catalog.resolve(index, &name, budget)?
                {
                    exports.push((name, binding));
                }
            }
            Ok(exports)
        })?;
        for (name, _) in &exports {
            self.check_string(&Value::String(name.clone()), span)?;
        }
        let namespace = self.object_work(span, |objects, budget| {
            objects.create_module_namespace(record, exports, budget)
        })?;
        self.object_work(span, |objects, budget| {
            objects.define(
                &namespace,
                WellKnownSymbol::ToStringTag.symbol(),
                DataDescriptor {
                    value: Some(Value::String(JsString::from("Module"))),
                    writable: Some(false),
                    enumerable: Some(false),
                    configurable: Some(false),
                },
                budget,
            )
        })?;
        self.object_work(span, |objects, budget| {
            objects.prevent_extensions(&namespace)?;
            objects.store_module_namespace(record, namespace.clone(), budget)
        })?;
        Ok(Value::Object(namespace))
    }

    pub(crate) fn namespace_export_property(
        &mut self,
        object: &ObjectHandle,
        key: PropertyKeyRef<'_>,
        span: Span,
    ) -> Result<Option<Property>, Error> {
        // Ordinary objects take no extra evaluator step for this brand check.
        if !self
            .objects
            .inspect(object)
            .map_err(Error::InvalidObject)?
            .is_module_namespace()
        {
            return Ok(None);
        };
        let Some(binding) = self.object_work(span, |objects, budget| {
            objects.namespace_binding(object, key, budget)
        })?
        else {
            return Ok(None);
        };
        let value = self.read_module_binding(&binding, span)?;
        Ok(Some(Property::Data(DataProperty {
            value,
            writable: true,
            enumerable: true,
            configurable: false,
        })))
    }

    pub(crate) fn namespace_define_property(
        &mut self,
        object: &ObjectHandle,
        key: PropertyKeyRef<'_>,
        descriptor: &PropertyDescriptor,
        span: Span,
    ) -> Result<Option<bool>, Error> {
        if key.as_string().is_none()
            || !self
                .objects
                .inspect(object)
                .map_err(Error::InvalidObject)?
                .is_module_namespace()
        {
            return Ok(None);
        };
        let Some(Property::Data(current)) = self.namespace_export_property(object, key, span)?
        else {
            return Ok(Some(false));
        };
        // GetOwnProperty precedes descriptor rejection, including its TDZ check.
        if descriptor.configurable == Some(true) || descriptor.enumerable == Some(false) {
            return Ok(Some(false));
        };
        let allowed = match &descriptor.kind {
            DescriptorKind::Accessor {
                get: None,
                set: None,
            } => true,
            DescriptorKind::Accessor { .. } => false,
            DescriptorKind::Data { value, writable } => {
                *writable != Some(false)
                    && value
                        .as_ref()
                        .is_none_or(|value| value.same_value(&current.value))
            }
            DescriptorKind::Generic => true,
        };
        Ok(Some(allowed))
    }
}

#[cfg(test)]
mod tests;
