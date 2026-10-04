//! Ordinary property storage and function call metadata.
//!
//! This layer implements own string-keyed data and accessor properties. Symbols
//! and additional exotic internal methods are separate increments. Mapped arguments
//! synchronize indexed properties with traced parameter environments.
//! Handles are unrooted and checked by the owning heap, not by these records.

use crate::{Value, function::Callable};
use spite_core::JsString;
use spite_heap::{Handle, Trace};
use std::fmt;

#[cfg(test)]
mod accessor_tests;
mod arguments;
use arguments::ParameterMap;
mod descriptor;
mod entry;
mod store;
pub use descriptor::{
    AccessorProperty, DataDescriptor, DataProperty, DescriptorKind, Property, PropertyDescriptor,
};
pub use store::{Budget, Error, GetAction, Objects, Root, SetAction};

/// The host-configured property capacity was exhausted.
///
/// This is a host failure, not a JavaScript exception or descriptor rejection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PropertyLimit;

impl fmt::Display for PropertyLimit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("object property limit exceeded")
    }
}
impl std::error::Error for PropertyLimit {}

/// An ordinary object's prototype, extensibility, and ordered properties.
///
/// Lookup is linear and keys compare exact UTF-16 code units. The property limit
/// bounds storage; callers must account for lookup and enumeration work when
/// integrating these records into an evaluator.
#[derive(Debug)]
pub struct OrdinaryObject {
    prototype: Option<Handle>,
    extensible: bool,
    properties: Vec<(JsString, Property)>,
    max_properties: usize,
    callable: Option<Callable>,
    immutable_prototype: bool,
    // Presence of [[ParameterMap]], including the empty unmapped form.
    arguments: bool,
    parameter_map: Option<ParameterMap>,
}

impl OrdinaryObject {
    /// Creates an extensible object with no own properties.
    ///
    /// The caller must supply a prototype from the intended owning heap.
    pub fn new(prototype: Option<Handle>, max_properties: usize) -> Self {
        Self {
            prototype,
            extensible: true,
            properties: Vec::new(),
            max_properties,
            callable: None,
            immutable_prototype: false,
            arguments: false,
            parameter_map: None,
        }
    }

    /// Returns the unrooted prototype handle, or `None` for a null prototype.
    pub fn prototype(&self) -> Option<&Handle> {
        self.prototype.as_ref()
    }

    /// Returns whether the object has a [[Call]] internal method.
    pub fn is_callable(&self) -> bool {
        self.callable.is_some()
    }

    pub(crate) fn is_arguments(&self) -> bool {
        self.arguments
    }

    pub(crate) fn callable(&self) -> Option<&Callable> {
        self.callable.as_ref()
    }

    /// Returns whether new own properties may be created.
    pub fn is_extensible(&self) -> bool {
        self.extensible
    }

    /// Permanently disallows creation of new own properties.
    pub fn prevent_extensions(&mut self) {
        self.extensible = false;
    }

    /// Returns the number of own properties, including non-enumerable properties.
    pub fn property_count(&self) -> usize {
        self.properties.len()
    }

    /// Looks up a stored own property without consulting the prototype.
    /// Use [`Objects::get_own`] to observe current mapped-argument values.
    pub fn own_property(&self, key: &JsString) -> Option<&Property> {
        self.properties
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, p)| p)
    }

    /// Applies an ordinary descriptor (ECMA-262 10.1.6.3).
    ///
    /// Descriptor rejection or capacity failure leaves this record unchanged.
    /// New attributes default to undefined/false; omitted fields are preserved
    /// on existing properties. Heap callers must validate getter/setter handles.
    pub fn define_own_property(
        &mut self,
        key: JsString,
        descriptor: impl Into<PropertyDescriptor>,
    ) -> Result<bool, PropertyLimit> {
        let descriptor = descriptor.into().normalize();
        if let Some((_, current)) = self.properties.iter_mut().find(|(k, _)| *k == key) {
            return Ok(current.apply(descriptor));
        }
        if !self.extensible {
            return Ok(false);
        }
        if self.properties.len() >= self.max_properties {
            return Err(PropertyLimit);
        }
        self.properties.push((key, descriptor.complete()));
        Ok(true)
    }

    /// Implements OrdinaryDelete (10.1.10.1), without strict-mode throw handling.
    pub fn delete(&mut self, key: &JsString) -> bool {
        let Some(index) = self.properties.iter().position(|(k, _)| k == key) else {
            return true;
        };
        if !self.properties[index].1.configurable() {
            return false;
        }
        self.properties.remove(index);
        true
    }

    /// Returns all own string keys in OrdinaryOwnPropertyKeys order (10.1.11.1).
    ///
    /// Array indices precede other strings and sort numerically. Other strings
    /// retain creation order, including non-enumerable properties.
    pub fn own_keys(&self) -> Vec<JsString> {
        let mut indices = Vec::new();
        let mut strings = Vec::new();
        for (key, _) in &self.properties {
            if let Some(index) = array_index(key) {
                indices.push((index, key));
            } else {
                strings.push(key);
            }
        }
        indices.sort_unstable_by_key(|(index, _)| *index);
        indices
            .into_iter()
            .map(|(_, key)| key)
            .chain(strings)
            .cloned()
            .collect()
    }
}

impl Trace for OrdinaryObject {
    fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        let bound = self.callable.as_ref().and_then(|callable| match callable {
            Callable::Builtin(_) | Callable::Arrow(_) | Callable::Ordinary(_) => None,
            Callable::Bound(bound) => Some(bound),
        });
        let capture = match self.callable.as_ref() {
            Some(Callable::Bound(bound)) => Some(&bound.target),
            Some(Callable::Arrow(function)) | Some(Callable::Ordinary(function)) => {
                Some(&function.environment.0)
            }
            Some(Callable::Builtin(_)) | None => None,
        };
        std::iter::once(self.prototype.as_ref())
            .chain(std::iter::once(capture))
            .chain(self.parameter_map.iter().flat_map(|map| {
                std::iter::once(Some(&map.environment.0)).chain(map.names.values().map(|_| None))
            }))
            .chain(bound.into_iter().flat_map(|bound| {
                std::iter::once(&bound.this)
                    .chain(&bound.arguments)
                    .flat_map(Value::trace)
            }))
            .chain(self.properties.iter().flat_map(|(_, property)| {
                let (first, second) = match property {
                    Property::Data(data) => (data.value.trace().next().flatten(), None),
                    Property::Accessor(accessor) => {
                        (accessor.get.as_ref(), Some(accessor.set.as_ref()))
                    }
                };
                std::iter::once(first).chain(second)
            }))
    }
}

// Array indices are canonical decimal strings in [0, 2^32 - 2]. Checking code
// units directly preserves non-ASCII and lone-surrogate property names.
fn array_index(key: &JsString) -> Option<u32> {
    let units = key.code_units();
    if units.is_empty() || units.len() > 10 || (units.len() > 1 && units[0] == u16::from(b'0')) {
        return None;
    }
    let mut index = 0u32;
    for &unit in units {
        let digit = unit.checked_sub(u16::from(b'0'))?;
        if digit > 9 {
            return None;
        }
        index = index.checked_mul(10)?.checked_add(u32::from(digit))?;
    }
    (index != u32::MAX).then_some(index)
}
