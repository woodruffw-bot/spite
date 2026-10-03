//! Ordinary data-property storage, before objects are exposed to JavaScript.
//!
//! This layer implements own string-keyed data properties. Accessors, Symbols,
//! and exotic internal methods are separate increments.
//! Handles are unrooted and checked by the owning heap, not by these records.

use crate::Value;
use spite_core::JsString;
use spite_heap::{Handle, Trace};
use std::fmt;

mod store;
pub use store::{Budget, Error, Objects, Root};

/// A complete ordinary data property.
#[derive(Clone, Debug, PartialEq)]
pub struct DataProperty {
    /// The stored value.
    pub value: Value,
    /// Whether assignment may change the value.
    pub writable: bool,
    /// Whether the property participates in enumerable-key operations.
    pub enumerable: bool,
    /// Whether the property may be deleted or reconfigured.
    pub configurable: bool,
}

/// A partial data or generic descriptor; omitted fields remain unspecified.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DataDescriptor {
    /// A replacement value, if supplied.
    pub value: Option<Value>,
    /// A replacement writable attribute, if supplied.
    pub writable: Option<bool>,
    /// A replacement enumerable attribute, if supplied.
    pub enumerable: Option<bool>,
    /// A replacement configurable attribute, if supplied.
    pub configurable: Option<bool>,
}

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

/// An ordinary object's prototype, extensibility, and ordered data properties.
///
/// Lookup is linear and keys compare exact UTF-16 code units. The property limit
/// bounds storage; callers must account for lookup and enumeration work when
/// integrating these records into an evaluator.
#[derive(Debug)]
pub struct OrdinaryObject {
    prototype: Option<Handle>,
    extensible: bool,
    properties: Vec<(JsString, DataProperty)>,
    max_properties: usize,
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
        }
    }

    /// Returns the unrooted prototype handle, or `None` for a null prototype.
    pub fn prototype(&self) -> Option<&Handle> {
        self.prototype.as_ref()
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

    /// Looks up an own property without consulting the prototype.
    pub fn own_property(&self, key: &JsString) -> Option<&DataProperty> {
        self.properties
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, p)| p)
    }

    /// Applies an ordinary data descriptor (ECMA-262 10.1.6.3).
    ///
    /// `false` means descriptor validation rejected the change. A capacity error
    /// occurs only when a permitted new property would exceed the host limit.
    /// Neither failure changes the object. New properties default to undefined
    /// and false attributes; existing properties preserve omitted fields.
    pub fn define_own_property(
        &mut self,
        key: JsString,
        descriptor: DataDescriptor,
    ) -> Result<bool, PropertyLimit> {
        let Some((_, current)) = self.properties.iter_mut().find(|(k, _)| *k == key) else {
            if !self.extensible {
                return Ok(false);
            }
            if self.properties.len() >= self.max_properties {
                return Err(PropertyLimit);
            }
            self.properties.push((
                key,
                DataProperty {
                    value: descriptor.value.unwrap_or(Value::Undefined),
                    writable: descriptor.writable.unwrap_or(false),
                    enumerable: descriptor.enumerable.unwrap_or(false),
                    configurable: descriptor.configurable.unwrap_or(false),
                },
            ));
            return Ok(true);
        };
        if !current.configurable {
            if descriptor.configurable == Some(true)
                || descriptor
                    .enumerable
                    .is_some_and(|v| v != current.enumerable)
            {
                return Ok(false);
            }
            if !current.writable {
                if descriptor.writable == Some(true) {
                    return Ok(false);
                }
                if let Some(value) = &descriptor.value {
                    // Return immediately, including on success. Edition 17
                    // preserves the existing value's distinguishable NaN bits.
                    return Ok(value.same_value(&current.value));
                }
            }
        }
        if let Some(value) = descriptor.value {
            current.value = value;
        }
        if let Some(writable) = descriptor.writable {
            current.writable = writable;
        }
        if let Some(enumerable) = descriptor.enumerable {
            current.enumerable = enumerable;
        }
        if let Some(configurable) = descriptor.configurable {
            current.configurable = configurable;
        }
        Ok(true)
    }

    /// Implements OrdinaryDelete (10.1.10.1), without strict-mode throw handling.
    pub fn delete(&mut self, key: &JsString) -> bool {
        let Some(index) = self.properties.iter().position(|(k, _)| k == key) else {
            return true;
        };
        if !self.properties[index].1.configurable {
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
        std::iter::once(self.prototype.as_ref()).chain(
            self.properties
                .iter()
                .flat_map(|(_, property)| property.value.trace()),
        )
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
