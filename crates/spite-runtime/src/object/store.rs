//! Heap context for ordinary internal methods with bounded prototype traversal.

use super::{DataDescriptor, DataProperty, OrdinaryObject};
use crate::Value;
use spite_core::JsString;
use spite_heap::{Collection, Handle, Heap};
use std::{
    fmt,
    rc::{Rc, Weak},
};

/// A host-held root that keeps an object reachable until its last clone is dropped.
///
/// Store unrooted handles in object fields, never roots: roots belong to host and
/// interpreter lifetimes outside the heap graph. The token owns only a handle;
/// it neither owns object data nor prevents the owning heap from being dropped.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Root {
    handle: Rc<Handle>,
}

impl Root {
    /// Borrows the rooted object's handle.
    pub fn handle(&self) -> &Handle {
        &self.handle
    }
}

/// A host failure from ordinary-object storage or work accounting.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// An invalid handle or exhausted heap capacity.
    Heap(spite_heap::Error),
    /// The operation exhausted its work budget.
    WorkLimit,
    /// A permitted new property would exceed the object's capacity.
    PropertyLimit,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Heap(error) => error.fmt(f),
            Self::WorkLimit => f.write_str("object work limit exceeded"),
            Self::PropertyLimit => f.write_str("object property limit exceeded"),
        }
    }
}
impl std::error::Error for Error {}
impl From<spite_heap::Error> for Error {
    fn from(error: spite_heap::Error) -> Self {
        Self::Heap(error)
    }
}

/// Work remaining for prototype hops, property scans, and value copies.
#[derive(Debug)]
pub struct Budget {
    remaining: usize,
}

impl Budget {
    /// Creates an object-operation budget.
    pub fn new(work: usize) -> Self {
        Self { remaining: work }
    }

    /// Returns the unconsumed work units.
    pub fn remaining_work(&self) -> usize {
        self.remaining
    }

    fn charge(&mut self, work: usize) -> Result<(), Error> {
        self.remaining = self.remaining.checked_sub(work).ok_or(Error::WorkLimit)?;
        Ok(())
    }

    fn lookup(&mut self, object: &OrdinaryObject, key: &JsString) -> Result<(), Error> {
        let comparisons = key
            .len()
            .checked_add(1)
            .and_then(|units| units.checked_mul(object.property_count()))
            .and_then(|units| units.checked_add(1))
            .ok_or(Error::WorkLimit)?;
        self.charge(comparisons)
    }

    fn value(&mut self, value: &Value) -> Result<(), Error> {
        let size = match value {
            Value::String(value) => value.len(),
            Value::BigInt(value) => value.bit_length().div_ceil(32),
            Value::Undefined
            | Value::Null
            | Value::Boolean(_)
            | Value::Number(_)
            | Value::Object(_) => 1,
        };
        self.charge(size)
    }
}

/// An ordinary-object heap that validates prototypes and prevents their cycles.
///
/// Allocation and internal methods never trigger collection. Returned handles
/// are unrooted: retain a `Root` or include each live host/interpreter handle in
/// the explicit collection roots. Only data properties are implemented.
#[derive(Debug)]
pub struct Objects {
    heap: Heap<OrdinaryObject>,
    max_properties: usize,
    roots: Vec<Weak<Handle>>,
}

impl Objects {
    /// Creates an empty heap with slot and per-object property limits.
    pub fn new(max_objects: usize, max_properties: usize) -> Self {
        Self {
            heap: Heap::new(max_objects),
            max_properties,
            roots: Vec::new(),
        }
    }

    /// Allocates an extensible object after validating its prototype.
    pub fn create(&mut self, prototype: Option<&Handle>) -> Result<Handle, Error> {
        if let Some(prototype) = prototype {
            self.heap.get(prototype)?;
        }
        Ok(self
            .heap
            .insert(OrdinaryObject::new(prototype.cloned(), self.max_properties))?)
    }

    /// Borrows a record for host inspection without traversing prototypes.
    pub fn inspect(&self, object: &Handle) -> Result<&OrdinaryObject, Error> {
        Ok(self.heap.get(object)?)
    }

    /// Retains an object across collections, validating ownership and liveness.
    ///
    /// Repeated roots of the same object share a token. Expired registry entries
    /// are reused, so the registry cannot exceed the heap's slot capacity. Work
    /// exhaustion creates no root and leaves existing root lifetimes unchanged.
    pub fn root(&mut self, object: &Handle, budget: &mut Budget) -> Result<Root, Error> {
        self.heap.get(object)?;
        budget.charge(self.roots.len().checked_add(1).ok_or(Error::WorkLimit)?)?;
        let mut empty = None;
        for (index, entry) in self.roots.iter().enumerate() {
            if let Some(handle) = entry.upgrade() {
                if handle.as_ref() == object {
                    return Ok(Root { handle });
                }
            } else if empty.is_none() {
                empty = Some(index);
            }
        }
        let handle = Rc::new(object.clone());
        let entry = Rc::downgrade(&handle);
        if let Some(index) = empty {
            self.roots[index] = entry;
        } else {
            self.roots.push(entry);
        }
        Ok(Root { handle })
    }

    /// Disallows new own properties while preserving existing property attributes.
    pub fn prevent_extensions(&mut self, object: &Handle) -> Result<(), Error> {
        self.heap.get_mut(object)?.prevent_extensions();
        Ok(())
    }

    /// Applies OrdinarySetPrototypeOf (10.1.2.1), returning false on rejection.
    ///
    /// A same-prototype request succeeds even for a non-extensible object.
    /// Budget exhaustion or a cycle leaves the existing prototype unchanged.
    pub fn set_prototype(
        &mut self,
        object: &Handle,
        prototype: Option<&Handle>,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        budget.charge(1)?;
        let current = self.heap.get(object)?;
        if let Some(prototype) = prototype {
            self.heap.get(prototype)?;
        }
        if current.prototype() == prototype {
            return Ok(true);
        }
        if !current.is_extensible() {
            return Ok(false);
        }
        let mut next = prototype;
        while let Some(parent) = next {
            budget.charge(1)?;
            if parent == object {
                return Ok(false);
            }
            next = self.heap.get(parent)?.prototype();
        }
        self.heap.get_mut(object)?.prototype = prototype.cloned();
        Ok(true)
    }

    /// Applies an own data descriptor without consulting inherited properties.
    pub fn define(
        &mut self,
        object: &Handle,
        key: JsString,
        descriptor: DataDescriptor,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        budget.lookup(self.heap.get(object)?, &key)?;
        if let Some(value) = &descriptor.value {
            if let Value::Object(handle) = value {
                self.heap.get(handle)?;
            }
            budget.value(value)?;
        }
        self.heap
            .get_mut(object)?
            .define_own_property(key, descriptor)
            .map_err(|_| Error::PropertyLimit)
    }

    /// Checks own property presence without copying its value or visiting prototypes.
    pub fn has_own(
        &self,
        object: &Handle,
        key: &JsString,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        let record = self.heap.get(object)?;
        budget.lookup(record, key)?;
        Ok(record.own_property(key).is_some())
    }

    /// Copies an own data descriptor with bounded key scans and value-copy work.
    pub fn get_own(
        &self,
        object: &Handle,
        key: &JsString,
        budget: &mut Budget,
    ) -> Result<Option<DataProperty>, Error> {
        let record = self.heap.get(object)?;
        budget.lookup(record, key)?;
        match record.own_property(key) {
            Some(property) => {
                budget.value(&property.value)?;
                Ok(Some(property.clone()))
            }
            None => Ok(None),
        }
    }

    /// Implements OrdinaryGet for data properties, searching prototypes iteratively.
    pub fn get(
        &self,
        object: &Handle,
        key: &JsString,
        budget: &mut Budget,
    ) -> Result<Value, Error> {
        match self.find(object, key, budget)? {
            Some(property) => {
                budget.value(&property.value)?;
                Ok(property.value.clone())
            }
            None => Ok(Value::Undefined),
        }
    }

    /// Implements OrdinaryHasProperty, including properties whose value is undefined.
    pub fn has(&self, object: &Handle, key: &JsString, budget: &mut Budget) -> Result<bool, Error> {
        Ok(self.find(object, key, budget)?.is_some())
    }

    /// Implements OrdinarySet for data properties with an explicit receiver.
    ///
    /// `None` represents any primitive receiver. Inherited writable data properties
    /// write to the receiver; inherited non-writable properties reject the write.
    /// A rejection returns false, leaving strict-mode throw handling to the caller.
    pub fn set(
        &mut self,
        object: &Handle,
        key: JsString,
        value: Value,
        receiver: Option<&Handle>,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        if let Some(receiver) = receiver {
            self.heap.get(receiver)?;
        }
        if self
            .find(object, &key, budget)?
            .is_some_and(|property| !property.writable)
        {
            return Ok(false);
        }
        let Some(receiver) = receiver else {
            return Ok(false);
        };
        let destination = self.heap.get(receiver)?;
        budget.lookup(destination, &key)?;
        let descriptor = if let Some(property) = destination.own_property(&key) {
            if !property.writable {
                return Ok(false);
            }
            DataDescriptor {
                value: Some(value),
                ..Default::default()
            }
        } else {
            DataDescriptor {
                value: Some(value),
                writable: Some(true),
                enumerable: Some(true),
                configurable: Some(true),
            }
        };
        self.define(receiver, key, descriptor, budget)
    }

    /// Deletes only an own property, with OrdinaryDelete's Boolean result.
    pub fn delete(
        &mut self,
        object: &Handle,
        key: &JsString,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        let record = self.heap.get(object)?;
        // Deletion may shift the remaining vector entries after the key scan.
        budget.charge(record.property_count())?;
        budget.lookup(record, key)?;
        Ok(self.heap.get_mut(object)?.delete(key))
    }

    /// Collects from live `Root` tokens and additional caller-supplied handles.
    ///
    /// Registry scans consume the same collection budget. Failure occurs before
    /// sweeping, including when the registry scan itself exceeds the budget.
    pub fn collect<'a>(
        &mut self,
        roots: impl IntoIterator<Item = &'a Handle>,
        max_work: usize,
    ) -> Result<Collection, spite_heap::Error> {
        let remaining = max_work
            .checked_sub(self.roots.len())
            .ok_or(spite_heap::Error::Limit)?;
        let retained: Vec<_> = self.roots.iter().filter_map(Weak::upgrade).collect();
        let mut registered = retained.iter();
        let mut supplied = roots.into_iter();
        let combined = std::iter::from_fn(|| match registered.next() {
            Some(root) => Some(root.as_ref()),
            None => supplied.next(),
        });
        let mut result = self.heap.collect(combined, remaining)?;
        result.work_used += self.roots.len();
        Ok(result)
    }

    fn find(
        &self,
        object: &Handle,
        key: &JsString,
        budget: &mut Budget,
    ) -> Result<Option<&DataProperty>, Error> {
        let mut next = Some(object);
        while let Some(handle) = next {
            let record = self.heap.get(handle)?;
            budget.lookup(record, key)?;
            if let Some(property) = record.own_property(key) {
                return Ok(Some(property));
            }
            next = record.prototype();
        }
        Ok(None)
    }
}
