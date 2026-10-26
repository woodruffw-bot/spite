//! GetModuleNamespace and Module namespace exotic methods (10.4.6, 16.2.1).

use super::*;
use crate::{
    Reference,
    object::{DataDescriptor, DataProperty, DescriptorKind, Property, PropertyDescriptor},
};
use spite_core::{JsString, PropertyKeyRef, WellKnownSymbol};

impl Realm {
    /// Returns a record's unique namespace, preparing bindings without body entry.
    ///
    /// Export values remain live; reading a lexical export before initialization
    /// throws ReferenceError. The returned object is unrooted. The Module handle
    /// retains it; use [`Self::root_value`] if it must outlive that handle.
    pub fn module_namespace(&mut self, module: &ModuleHandle) -> Result<Value, Error> {
        let record = module.root.handle();
        if let Some(namespace) = self
            .objects
            .module_namespace(record)
            .map_err(Error::InvalidObject)?
        {
            return Ok(namespace);
        };
        if !self.job_active {
            self.remaining_steps = self.limits.max_steps;
        }
        let source = self
            .objects
            .module_source(record)
            .map_err(Error::InvalidObject)?
            .clone();
        self.prepare_module_record(&source, record)?;
        let span = Span::new(0, 0);
        let entries = source.entries().local_export_entries;
        let mut exports = Vec::new();
        exports
            .try_reserve_exact(entries.len())
            .map_err(|_| Error::Limit {
                span,
                message: "module namespace capacity exceeded".into(),
            })?;
        for entry in entries {
            self.check_string(&Value::String(entry.exported.clone()), span)?;
            exports.push((
                entry.exported,
                String::from_utf16(entry.local.code_units()).expect("local module binding"),
            ));
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
        let Some((environment, name)) = self.object_work(span, |objects, budget| {
            objects.namespace_binding(object, key, budget)
        })?
        else {
            return Ok(None);
        };
        let value = self.get(&mut Reference::Lexical(environment, name.into()), span)?;
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
