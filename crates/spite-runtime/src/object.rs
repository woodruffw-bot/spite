//! Ordinary property storage and function call metadata.
//!
//! This layer implements own string/symbol data and accessor properties.
//! Additional exotic internal methods are separate increments. Mapped arguments
//! synchronize indexed properties with traced parameter environments.
//! Handles are unrooted and checked by the owning heap, not by these records.

use crate::{Value, function::Callable};
use spite_bigint::BigInt;
use spite_core::{JsString, JsSymbol, PropertyKey, PropertyKeyRef};
use spite_heap::{Handle, Trace};
use std::fmt;

#[cfg(test)]
mod accessor_tests;
mod arguments;
mod array;
#[cfg(test)]
mod symbol_tests;
use arguments::ParameterMap;
mod descriptor;
mod iterator;
mod map;
use iterator::IteratorState;
use iterator::MapIterator;
use iterator::SetIterator;
pub(crate) use iterator::{
    ArrayIterationKind, ArrayIterator, CallbackIterator, CallbackKind, ConcatIterable,
    HelperStatus, IteratorHelper, IteratorWrapper, LimitKind, StringIterator,
};
pub(crate) use map::CollectionKey;
mod private;
mod set;
pub(crate) use set::SetData;
mod entry;
mod store;
mod weak_set;
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

#[derive(Clone, Debug)]
pub(crate) enum PrimitiveData {
    Boolean(bool),
    Number(f64),
    String(JsString),
    Symbol(JsSymbol),
    BigInt(BigInt),
}

/// Stored object properties, prototype, extensibility, and internal-slot metadata.
///
/// Lookup is linear. String keys compare UTF-16 units; symbols compare identity.
/// An optional property quota bounds storage; callers account for lookup and enumeration work when
/// integrating these records into an evaluator.
#[derive(Debug)]
pub struct OrdinaryObject {
    prototype: Option<Handle>,
    extensible: bool,
    properties: Vec<(PropertyKey, Property)>,
    private_elements: Vec<(crate::private::PrivateName, private::PrivateElement)>,
    max_properties: Option<usize>,
    callable: Option<Callable>,
    constructible: bool,
    primitive_data: Option<PrimitiveData>,
    error_data: bool,
    raw_json: bool,
    map: Option<Box<map::MapData>>,
    set: Option<Box<SetData>>,
    weak_set: Option<Box<weak_set::WeakSetData>>,
    immutable_prototype: bool,
    // Presence of [[ParameterMap]], including the empty unmapped form.
    arguments: bool,
    parameter_map: Option<ParameterMap>,
    array: bool,
    iterator: Option<IteratorState>,
}

impl OrdinaryObject {
    /// Creates an extensible object with no own properties.
    ///
    /// The caller must supply a prototype from the intended owning heap.
    pub fn new(prototype: Option<Handle>, max_properties: usize) -> Self {
        Self::with_property_limit(prototype, Some(max_properties))
    }

    /// Creates an extensible object with an optional own-property limit.
    pub fn with_property_limit(prototype: Option<Handle>, max_properties: Option<usize>) -> Self {
        Self {
            prototype,
            extensible: true,
            properties: Vec::new(),
            private_elements: Vec::new(),
            max_properties,
            callable: None,
            constructible: false,
            primitive_data: None,
            error_data: false,
            raw_json: false,
            map: None,
            set: None,
            weak_set: None,
            immutable_prototype: false,
            arguments: false,
            parameter_map: None,
            array: false,
            iterator: None,
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

    /// Returns whether the object has a [[Construct]] internal method.
    pub fn is_constructor(&self) -> bool {
        self.constructible
    }

    /// Returns whether the object has the ErrorData internal slot.
    pub fn is_error(&self) -> bool {
        self.error_data
    }

    pub(crate) fn is_raw_json(&self) -> bool {
        self.raw_json
    }

    pub(crate) fn is_map(&self) -> bool {
        self.map.is_some()
    }

    pub(crate) fn is_set(&self) -> bool {
        self.set.is_some()
    }

    /// Returns whether this record has Array exotic internal methods.
    pub fn is_array(&self) -> bool {
        self.array
    }

    pub(crate) fn array_iterator(&self) -> Option<&ArrayIterator> {
        match &self.iterator {
            Some(IteratorState::Array(state)) => Some(state),
            _ => None,
        }
    }

    pub(crate) fn map_iterator(&self) -> Option<&MapIterator> {
        match &self.iterator {
            Some(IteratorState::Map(state)) => Some(state),
            _ => None,
        }
    }

    pub(crate) fn set_iterator(&self) -> Option<&SetIterator> {
        match &self.iterator {
            Some(IteratorState::Set(state)) => Some(state),
            _ => None,
        }
    }

    pub(crate) fn string_iterator(&self) -> Option<&StringIterator> {
        match &self.iterator {
            Some(IteratorState::String(state)) => Some(state),
            _ => None,
        }
    }

    pub(crate) fn iterator_wrapper(&self) -> Option<&IteratorWrapper> {
        match &self.iterator {
            Some(IteratorState::Wrapper(state)) => Some(state),
            _ => None,
        }
    }

    pub(crate) fn iterator_helper(&self) -> Option<&IteratorHelper> {
        match &self.iterator {
            Some(IteratorState::Helper(state)) => Some(state),
            _ => None,
        }
    }

    pub(crate) fn is_arguments(&self) -> bool {
        self.arguments
    }

    pub(crate) fn boolean_data(&self) -> Option<bool> {
        match &self.primitive_data {
            Some(PrimitiveData::Boolean(value)) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn number_data(&self) -> Option<f64> {
        match &self.primitive_data {
            Some(PrimitiveData::Number(value)) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn callable(&self) -> Option<&Callable> {
        self.callable.as_ref()
    }

    pub(crate) fn string_data(&self) -> Option<&JsString> {
        match &self.primitive_data {
            Some(PrimitiveData::String(value)) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn symbol_data(&self) -> Option<&JsSymbol> {
        match &self.primitive_data {
            Some(PrimitiveData::Symbol(value)) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn bigint_data(&self) -> Option<&BigInt> {
        match &self.primitive_data {
            Some(PrimitiveData::BigInt(value)) => Some(value),
            _ => None,
        }
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
    pub fn own_property<'key>(&self, key: impl Into<PropertyKeyRef<'key>>) -> Option<&Property> {
        let key = key.into();
        self.properties
            .iter()
            .find(|(k, _)| PropertyKeyRef::from(k) == key)
            .map(|(_, p)| p)
    }

    /// Applies an ordinary descriptor (ECMA-262 10.1.6.3).
    ///
    /// Descriptor rejection or capacity failure leaves this record unchanged.
    /// New attributes default to undefined/false; omitted fields are preserved
    /// on existing properties. Heap callers must validate getter/setter handles.
    pub fn define_own_property(
        &mut self,
        key: impl Into<PropertyKey>,
        descriptor: impl Into<PropertyDescriptor>,
    ) -> Result<bool, PropertyLimit> {
        let key = key.into();
        let descriptor = descriptor.into().normalize();
        if let Some((_, current)) = self.properties.iter_mut().find(|(k, _)| *k == key) {
            return Ok(current.apply(descriptor));
        }
        if !self.extensible {
            return Ok(false);
        }
        if self
            .max_properties
            .is_some_and(|limit| self.properties.len() >= limit)
        {
            return Err(PropertyLimit);
        }
        self.properties.try_reserve(1).map_err(|_| PropertyLimit)?;
        self.properties.push((key, descriptor.complete()));
        Ok(true)
    }

    /// Implements OrdinaryDelete (10.1.10.1), without strict-mode throw handling.
    pub fn delete<'key>(&mut self, key: impl Into<PropertyKeyRef<'key>>) -> bool {
        let key = key.into();
        let Some(index) = self
            .properties
            .iter()
            .position(|(k, _)| PropertyKeyRef::from(k) == key)
        else {
            return true;
        };
        if !self.properties[index].1.configurable() {
            return false;
        }
        self.properties.remove(index);
        true
    }

    /// Returns all own keys in OrdinaryOwnPropertyKeys order (10.1.11.1).
    ///
    /// Array indices precede other strings and sort numerically. Other strings
    /// retain creation order, followed by symbols in their creation order.
    /// Non-enumerable properties are included in each group.
    pub fn own_keys(&self) -> Vec<PropertyKey> {
        let mut indices = Vec::new();
        let mut strings = Vec::new();
        let mut symbols = Vec::new();
        for (key, _) in &self.properties {
            if let Some(index) = array_index(key) {
                indices.push((index, key));
            } else if matches!(key, PropertyKey::Symbol(_)) {
                symbols.push(key);
            } else {
                strings.push(key);
            }
        }
        indices.sort_unstable_by_key(|(index, _)| *index);
        indices
            .into_iter()
            .map(|(_, key)| key)
            .chain(strings)
            .chain(symbols)
            .cloned()
            .collect()
    }
}

impl Trace for OrdinaryObject {
    fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        let bound = self.callable.as_ref().and_then(|callable| match callable {
            Callable::Builtin(_)
            | Callable::Arrow(_)
            | Callable::Ordinary(_)
            | Callable::Method(_)
            | Callable::ClassConstructor(_) => None,
            Callable::Bound(bound) => Some(bound),
        });
        let capture = match self.callable.as_ref() {
            Some(Callable::Bound(bound)) => Some(&bound.target),
            Some(Callable::Arrow(function)) | Some(Callable::Ordinary(function)) => {
                Some(&function.environment.0)
            }
            Some(Callable::Method(method)) => Some(&method.code.environment.0),
            Some(Callable::ClassConstructor(class)) => Some(&class.method.code.environment.0),
            Some(Callable::Builtin(_)) | None => None,
        };
        let home_object = match &self.callable {
            Some(Callable::Method(method)) => Some(&method.home_object),
            Some(Callable::ClassConstructor(class)) => Some(&class.method.home_object),
            _ => None,
        };
        std::iter::once(self.prototype.as_ref())
            .chain(self.primitive_data.iter().map(|_| None))
            .chain(self.iterator.iter().flat_map(IteratorState::trace))
            .chain(self.error_data.then_some(None))
            .chain(self.raw_json.then_some(None))
            .chain(
                self.private_elements
                    .iter()
                    .flat_map(|(_, element)| element.trace()),
            )
            .chain(self.map.iter().flat_map(|data| data.trace()))
            .chain(self.set.iter().flat_map(|data| data.trace()))
            .chain(self.weak_set.iter().flat_map(|data| data.trace()))
            .chain(std::iter::once(capture))
            .chain(home_object.map(Some))
            .chain(self.callable.iter().flat_map(|callable| {
                let fields = match callable {
                    Callable::ClassConstructor(class) => class.elements.fields.as_ref(),
                    _ => &[],
                };
                fields
                    .iter()
                    .filter_map(|field| field.initializer.as_ref())
                    .flat_map(|initializer| {
                        [
                            Some(&initializer.environment.0),
                            Some(&initializer.home_object),
                        ]
                    })
            }))
            .chain(self.callable.iter().flat_map(|callable| {
                let methods = match callable {
                    Callable::ClassConstructor(class) => class.elements.private_methods.as_ref(),
                    _ => &[],
                };
                methods.iter().flat_map(|method| method.kind.trace())
            }))
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
fn array_index<'key>(key: impl Into<PropertyKeyRef<'key>>) -> Option<u32> {
    let key = key.into().as_string()?;
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
