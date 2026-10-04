//! Heap context for ordinary internal methods with bounded prototype traversal.

use super::entry::Entry;
use super::{DataDescriptor, DescriptorKind, OrdinaryObject, Property, PropertyDescriptor};
use crate::function::{BoundFunction, Builtin, Callable, ScriptFunction};
use crate::{
    Value,
    environment::{BindingState, Environment, EnvironmentHandle},
};
use spite_core::JsString;
use spite_heap::{Collection, Handle, Heap};
use std::collections::BTreeMap;
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
    /// A supplied function handle does not refer to a callable object.
    NotCallable,
    /// A valid handle refers to a different kind of runtime heap entry.
    WrongKind,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Heap(error) => error.fmt(f),
            Self::WorkLimit => f.write_str("object work limit exceeded"),
            Self::PropertyLimit => f.write_str("object property limit exceeded"),
            Self::NotCallable => f.write_str("function handle must be callable"),
            Self::WrongKind => f.write_str("heap entry has the wrong kind"),
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

    pub(crate) fn charge(&mut self, work: usize) -> Result<(), Error> {
        self.remaining = self.remaining.checked_sub(work).ok_or(Error::WorkLimit)?;
        Ok(())
    }

    pub(super) fn lookup(&mut self, object: &OrdinaryObject, key: &JsString) -> Result<(), Error> {
        let comparisons = key
            .len()
            .checked_add(1)
            .and_then(|units| units.checked_mul(object.property_count()))
            .and_then(|units| units.checked_add(1))
            .ok_or(Error::WorkLimit)?;
        self.charge(comparisons)
    }

    pub(crate) fn value(&mut self, value: &Value) -> Result<(), Error> {
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

/// The next step of OrdinaryGet, performed after releasing storage borrows.
#[derive(Clone, Debug, PartialEq)]
pub enum GetAction {
    /// Return the supplied data value (or undefined for an absent getter).
    Value(Value),
    /// Call this getter with the original receiver and no arguments.
    Call(Handle),
}

/// The next step of OrdinarySet, performed after releasing storage borrows.
#[derive(Clone, Debug, PartialEq)]
pub enum SetAction {
    /// The write completed or was rejected, without invoking a setter.
    Done(bool),
    /// Call this setter with the original receiver and assigned value, then return true.
    Call(Handle),
}

/// An ordinary-object heap that validates prototypes and prevents their cycles.
///
/// Allocation and internal methods never trigger collection. Returned handles
/// are unrooted: retain a `Root` or include each live host/interpreter handle in
/// the explicit collection roots. Accessor calls are returned as actions for
/// the evaluator; storage never executes JavaScript under a heap borrow.
#[derive(Debug)]
pub struct Objects {
    heap: Heap<Entry>,
    max_properties: usize,
    roots: Vec<Weak<Handle>>,
}

impl Objects {
    /// Creates an empty heap with slot and per-object property limits.
    pub fn new(max_entries: usize, max_properties: usize) -> Self {
        Self {
            heap: Heap::new(max_entries),
            max_properties,
            roots: Vec::new(),
        }
    }

    /// Allocates an extensible object after validating its prototype.
    pub fn create(&mut self, prototype: Option<&Handle>) -> Result<Handle, Error> {
        if let Some(prototype) = prototype {
            self.inspect(prototype)?;
        }
        Ok(self.heap.insert(Entry::Object(OrdinaryObject::new(
            prototype.cloned(),
            self.max_properties,
        )))?)
    }

    pub(crate) fn create_builtin(
        &mut self,
        prototype: &Handle,
        builtin: Builtin,
    ) -> Result<Handle, Error> {
        self.inspect(prototype)?;
        let mut object = OrdinaryObject::new(Some(prototype.clone()), self.max_properties);
        object.callable = Some(Callable::Builtin(builtin));
        Ok(self.heap.insert(Entry::Object(object))?)
    }

    pub(crate) fn create_arrow(
        &mut self,
        prototype: &Handle,
        arrow: ScriptFunction,
    ) -> Result<Handle, Error> {
        self.inspect(prototype)?;
        self.environment(&arrow.environment)?;
        let mut object = OrdinaryObject::new(Some(prototype.clone()), self.max_properties);
        object.callable = Some(Callable::Arrow(arrow));
        Ok(self.heap.insert(Entry::Object(object))?)
    }

    pub(crate) fn create_ordinary_function(
        &mut self,
        prototype: &Handle,
        function: ScriptFunction,
    ) -> Result<Handle, Error> {
        self.inspect(prototype)?;
        self.environment(&function.environment)?;
        let mut object = OrdinaryObject::new(Some(prototype.clone()), self.max_properties);
        object.callable = Some(Callable::Ordinary(function));
        object.constructible = true;
        Ok(self.heap.insert(Entry::Object(object))?)
    }

    pub(crate) fn create_bound(
        &mut self,
        bound: BoundFunction,
        budget: &mut Budget,
    ) -> Result<Handle, Error> {
        budget.charge(1)?;
        let target = self.inspect(&bound.target)?;
        if !target.is_callable() {
            return Err(Error::NotCallable);
        }
        let prototype = target.prototype().cloned();
        let constructible = target.is_constructor();
        for value in std::iter::once(&bound.this).chain(&bound.arguments) {
            budget.charge(1)?;
            if let Value::Object(handle) = value {
                self.inspect(handle)?;
            }
        }
        let mut object = OrdinaryObject::new(prototype, self.max_properties);
        object.callable = Some(Callable::Bound(bound));
        object.constructible = constructible;
        Ok(self.heap.insert(Entry::Object(object))?)
    }

    pub(crate) fn create_object_prototype(&mut self) -> Result<Handle, Error> {
        let mut object = OrdinaryObject::new(None, self.max_properties);
        object.immutable_prototype = true;
        Ok(self.heap.insert(Entry::Object(object))?)
    }

    /// Borrows a record for host inspection without traversing prototypes.
    pub fn inspect(&self, object: &Handle) -> Result<&OrdinaryObject, Error> {
        match self.heap.get(object)? {
            Entry::Object(object) => Ok(object),
            Entry::Environment(_) => Err(Error::WrongKind),
        }
    }

    pub(super) fn object_mut(&mut self, object: &Handle) -> Result<&mut OrdinaryObject, Error> {
        match self.heap.get_mut(object)? {
            Entry::Object(object) => Ok(object),
            Entry::Environment(_) => Err(Error::WrongKind),
        }
    }

    pub(crate) fn create_arguments(&mut self, prototype: &Handle) -> Result<Handle, Error> {
        self.inspect(prototype)?;
        let mut object = OrdinaryObject::new(Some(prototype.clone()), self.max_properties);
        object.arguments = true;
        Ok(self.heap.insert(Entry::Object(object))?)
    }

    pub(crate) fn create_function_environment(
        &mut self,
        outer: EnvironmentHandle,
        bindings: BTreeMap<String, BindingState>,
        this: Value,
        new_target: Option<Handle>,
        budget: &mut Budget,
    ) -> Result<EnvironmentHandle, Error> {
        budget.value(&this)?;
        if let Value::Object(handle) = &this {
            self.inspect(handle)?;
        }
        if let Some(target) = &new_target {
            budget.charge(1)?;
            if !self.inspect(target)?.is_constructor() {
                return Err(Error::WrongKind);
            }
        }
        let environment = self.create_environment(Some(outer), bindings, budget)?;
        let record = self.environment_mut(&environment)?;
        record.this = Some(this);
        record.new_target = new_target;
        Ok(environment)
    }

    pub(crate) fn create_environment(
        &mut self,
        outer: Option<EnvironmentHandle>,
        bindings: BTreeMap<String, BindingState>,
        budget: &mut Budget,
    ) -> Result<EnvironmentHandle, Error> {
        if let Some(outer) = &outer {
            self.environment(outer)?;
        }
        for binding in bindings.values() {
            budget.charge(1)?;
            if let Some(Value::Object(handle)) = &binding.value {
                self.inspect(handle)?;
            }
        }
        Ok(EnvironmentHandle(self.heap.insert(Entry::Environment(
            Environment {
                outer,
                bindings,
                this: None,
                new_target: None,
            },
        ))?))
    }

    pub(crate) fn environment(&self, handle: &EnvironmentHandle) -> Result<&Environment, Error> {
        match self.heap.get(&handle.0)? {
            Entry::Environment(environment) => Ok(environment),
            Entry::Object(_) => Err(Error::WrongKind),
        }
    }

    pub(crate) fn environment_mut(
        &mut self,
        handle: &EnvironmentHandle,
    ) -> Result<&mut Environment, Error> {
        match self.heap.get_mut(&handle.0)? {
            Entry::Environment(environment) => Ok(environment),
            Entry::Object(_) => Err(Error::WrongKind),
        }
    }

    /// Retains an object across collections, validating ownership and liveness.
    ///
    /// Repeated roots of the same object share a token. Expired registry entries
    /// are reused, so the registry cannot exceed the heap's slot capacity. Work
    /// exhaustion creates no root and leaves existing root lifetimes unchanged.
    pub fn root(&mut self, object: &Handle, budget: &mut Budget) -> Result<Root, Error> {
        self.inspect(object)?;
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
        self.object_mut(object)?.prevent_extensions();
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
        let current = self.inspect(object)?;
        if let Some(prototype) = prototype {
            self.inspect(prototype)?;
        }
        if current.prototype() == prototype {
            return Ok(true);
        }
        if !current.is_extensible() || current.immutable_prototype {
            return Ok(false);
        }
        let mut next = prototype;
        while let Some(parent) = next {
            budget.charge(1)?;
            if parent == object {
                return Ok(false);
            }
            next = self.inspect(parent)?.prototype();
        }
        self.object_mut(object)?.prototype = prototype.cloned();
        Ok(true)
    }

    /// Applies an own descriptor, validating value edges and accessor callability.
    pub fn define(
        &mut self,
        object: &Handle,
        key: JsString,
        descriptor: impl Into<PropertyDescriptor>,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        let mut descriptor = descriptor.into().normalize();
        budget.lookup(self.inspect(object)?, &key)?;
        let mapping = self.mapped_target(object, &key, budget)?;
        // Prepare copies and charge all work before changing either the ordinary
        // descriptor or its aliased environment cell (10.4.4.2).
        let mapped_value = if mapping.is_some() {
            if let DescriptorKind::Data {
                value: Some(value), ..
            } = &descriptor.kind
            {
                budget.value(value)?;
                Some(value.clone())
            } else {
                None
            }
        } else {
            None
        };
        let detach = mapping.is_some()
            && matches!(
                descriptor.kind,
                DescriptorKind::Accessor { .. }
                    | DescriptorKind::Data {
                        writable: Some(false),
                        ..
                    }
            );
        if mapping.is_some() {
            if let DescriptorKind::Data {
                value: None,
                writable: Some(false),
            } = &descriptor.kind
            {
                let value = self
                    .mapped_value(object, &key, budget)?
                    .expect("existing mapping");
                budget.value(value)?;
                descriptor.kind = DescriptorKind::Data {
                    value: Some(value.clone()),
                    writable: Some(false),
                };
            }
        }
        match &descriptor.kind {
            DescriptorKind::Data {
                value: Some(value), ..
            } => {
                if let Value::Object(handle) = value {
                    self.inspect(handle)?;
                }
                budget.value(value)?;
            }
            DescriptorKind::Accessor { get, set } => {
                for handle in [get, set].into_iter().flatten().flatten() {
                    budget.charge(1)?;
                    if !self.inspect(handle)?.is_callable() {
                        return Err(Error::NotCallable);
                    }
                }
            }
            _ => {}
        }
        let allowed = self
            .object_mut(object)?
            .define_own_property(key, descriptor)
            .map_err(|_| Error::PropertyLimit)?;
        if allowed {
            if let Some((environment, name, index)) = mapping {
                if let Some(value) = mapped_value {
                    self.environment_mut(&environment)?
                        .bindings
                        .get_mut(&name)
                        .ok_or(Error::WrongKind)?
                        .value = Some(value);
                }
                if detach {
                    let record = self.object_mut(object)?;
                    let map = record.parameter_map.as_mut().expect("existing map");
                    map.names.remove(&index);
                    if map.names.is_empty() {
                        record.parameter_map = None;
                    }
                }
            }
        }
        Ok(allowed)
    }

    /// Checks own property presence without copying its value or visiting prototypes.
    pub fn has_own(
        &self,
        object: &Handle,
        key: &JsString,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        let record = self.inspect(object)?;
        budget.lookup(record, key)?;
        Ok(record.own_property(key).is_some())
    }

    /// Copies an own complete descriptor with bounded key scans and value-copy work.
    pub fn get_own(
        &self,
        object: &Handle,
        key: &JsString,
        budget: &mut Budget,
    ) -> Result<Option<Property>, Error> {
        let record = self.inspect(object)?;
        budget.lookup(record, key)?;
        match record.own_property(key) {
            Some(Property::Data(data)) => {
                let value = self
                    .mapped_value(object, key, budget)?
                    .unwrap_or(&data.value);
                budget.value(value)?;
                Ok(Some(Property::Data(super::DataProperty {
                    value: value.clone(),
                    writable: data.writable,
                    enumerable: data.enumerable,
                    configurable: data.configurable,
                })))
            }
            Some(property) => {
                budget.charge(2)?;
                Ok(Some(property.clone()))
            }
            None => Ok(None),
        }
    }

    /// Finds a data result or getter call, searching prototypes iteratively.
    pub fn get(
        &self,
        object: &Handle,
        key: &JsString,
        budget: &mut Budget,
    ) -> Result<GetAction, Error> {
        match self.find(object, key, budget)? {
            Some((owner, Property::Data(property))) => {
                let value = self
                    .mapped_value(&owner, key, budget)?
                    .unwrap_or(&property.value);
                budget.value(value)?;
                Ok(GetAction::Value(value.clone()))
            }
            Some((_, Property::Accessor(property))) => {
                budget.charge(1)?;
                Ok(match &property.get {
                    Some(getter) => GetAction::Call(getter.clone()),
                    None => GetAction::Value(Value::Undefined),
                })
            }
            None => Ok(GetAction::Value(Value::Undefined)),
        }
    }

    /// Implements OrdinaryHasProperty, including properties whose value is undefined.
    pub fn has(&self, object: &Handle, key: &JsString, budget: &mut Budget) -> Result<bool, Error> {
        Ok(self.find(object, key, budget)?.is_some())
    }

    /// Applies ordinary data writes or returns a setter call with an explicit receiver.
    ///
    /// `None` represents a primitive receiver. Setter calls preserve the actual
    /// receiver in the evaluator. A completed rejection leaves strict-mode error
    /// handling to the evaluator (10.1.9.2).
    pub fn set(
        &mut self,
        object: &Handle,
        key: JsString,
        value: Value,
        receiver: Option<&Handle>,
        budget: &mut Budget,
    ) -> Result<SetAction, Error> {
        if let Some(receiver) = receiver {
            self.inspect(receiver)?;
        }
        match self.find(object, &key, budget)? {
            Some((_, Property::Accessor(property))) => {
                budget.charge(1)?;
                return Ok(match &property.set {
                    Some(setter) => SetAction::Call(setter.clone()),
                    None => SetAction::Done(false),
                });
            }
            Some((_, Property::Data(property))) if !property.writable => {
                return Ok(SetAction::Done(false));
            }
            _ => {}
        }
        let Some(receiver) = receiver else {
            return Ok(SetAction::Done(false));
        };
        let destination = self.inspect(receiver)?;
        budget.lookup(destination, &key)?;
        let descriptor = if let Some(property) = destination.own_property(&key) {
            if !matches!(property, Property::Data(data) if data.writable) {
                return Ok(SetAction::Done(false));
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
            .map(SetAction::Done)
    }

    /// Deletes only an own property, with OrdinaryDelete's Boolean result.
    pub fn delete(
        &mut self,
        object: &Handle,
        key: &JsString,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        let record = self.inspect(object)?;
        // Deletion may shift the remaining vector entries after the key scan.
        budget.charge(record.property_count())?;
        budget.lookup(record, key)?;
        let index = super::array_index(key);
        let record = self.object_mut(object)?;
        let deleted = record.delete(key);
        if deleted {
            if let (Some(map), Some(index)) = (&mut record.parameter_map, index) {
                map.names.remove(&index);
                if map.names.is_empty() {
                    record.parameter_map = None;
                }
            }
        }
        Ok(deleted)
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
    ) -> Result<Option<(Handle, &Property)>, Error> {
        let mut next = Some(object);
        while let Some(handle) = next {
            let record = self.inspect(handle)?;
            budget.lookup(record, key)?;
            if let Some(property) = record.own_property(key) {
                return Ok(Some((handle.clone(), property)));
            }
            next = record.prototype();
        }
        Ok(None)
    }
}
