//! Closed Runtime Symbol-field inventory; census and marking are distinct modes.

use super::super::{
    CollectionKey, OrdinaryObject, PrimitiveData, Property,
    entry::Entry,
    finalization_registry::WeakTarget,
    iterator::{HelperClosure, IteratorState},
    private::PrivateElement,
};
use crate::{
    Value,
    environment::{Environment, ThisBinding},
    function::Callable,
};
use spite_core::{JsSymbol, WeakJsSymbol};
use spite_heap::Error;
use std::rc::Rc;

#[derive(Clone, Copy)]
pub(super) enum Mode {
    Physical,
    Logical,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum SharedKind {
    Fields,
    PrivateMethods,
}

pub(super) fn symbol(value: &Value) -> Option<&JsSymbol> {
    match value {
        Value::Symbol(value) => Some(value),
        Value::Undefined
        | Value::Null
        | Value::Boolean(_)
        | Value::Number(_)
        | Value::BigInt(_)
        | Value::String(_)
        | Value::Object(_) => None,
    }
}

fn collection_symbol(key: &CollectionKey) -> Option<&JsSymbol> {
    match key {
        CollectionKey::Symbol(value) => Some(value),
        CollectionKey::Undefined
        | CollectionKey::Null
        | CollectionKey::Boolean(_)
        | CollectionKey::Number(_)
        | CollectionKey::String(_)
        | CollectionKey::BigInt(_)
        | CollectionKey::Object(_) => None,
    }
}

// HashMap iteration scans its bucket allocation even when few keys remain.
// Prepay this before any index traversal; per-key visits separately account for
// processing the actual stored Symbol wrappers needed by the physical census.
pub(super) fn physical_index_work(entry: &Entry) -> Result<usize, Error> {
    let Entry::Object(object) = entry else {
        return Ok(0);
    };
    let map = object.map.as_ref().map_or(0, |map| map.index.capacity());
    let set = object.set.as_ref().map_or(0, |set| set.index.capacity());
    map.checked_add(set).ok_or(Error::Limit)
}

pub(super) fn symbols(
    entry: &Entry,
    mode: Mode,
    visit: &mut impl FnMut(Option<&JsSymbol>) -> Result<(), Error>,
) -> Result<(), Error> {
    match entry {
        Entry::Environment(environment) => {
            let Environment {
                outer: _,
                bindings,
                private_names,
                binding_object: _,
                this,
                new_target: _,
                home_object: _,
                derived_constructor: _,
                class_field_initializer: _,
                // Only a handle to the separately scanned resource list.
                disposable_resources: _,
            } = environment;
            for value in bindings.values() {
                visit(value.value.as_ref().and_then(symbol))?;
            }
            for name in private_names.values() {
                visit(Some(name.symbol()))?;
            }
            visit(match this {
                Some(ThisBinding::Initialized(value)) => symbol(value),
                Some(ThisBinding::Uninitialized) | None => None,
            })?;
        }
        Entry::Object(object) => object_symbols(object, mode, visit)?,
    }
    Ok(())
}

fn object_symbols(
    object: &OrdinaryObject,
    mode: Mode,
    visit: &mut impl FnMut(Option<&JsSymbol>) -> Result<(), Error>,
) -> Result<(), Error> {
    // No `..`: a new slot requires an explicit ownership-inventory decision.
    let OrdinaryObject {
        prototype: _,
        extensible: _,
        properties,
        private_elements,
        max_properties: _,
        callable,
        constructible: _,
        proxy: _,
        primitive_data,
        date_value: _,
        array_buffer: _,
        data_view: _,
        typed_array: _,
        regexp: _,
        error_data: _,
        promise,
        promise_aggregate,
        disposable_stack: _,
        async_disposal,
        async_execution,
        generator,
        array_from_async,
        disposable_resources,
        raw_json: _,
        map,
        set,
        weak_map,
        // Only WeakJsSymbol/Handle identities, never an owning Symbol wrapper.
        weak_ref: _,
        finalization_registry,
        weak_set: _,
        immutable_prototype: _,
        arguments: _,
        parameter_map: _,
        array: _,
        iterator,
    } = object;
    for (key, property) in properties {
        visit(key.as_symbol())?;
        visit(match property {
            Property::Data(data) => symbol(&data.value),
            Property::Accessor(_) => None,
        })?;
    }
    for (name, element) in private_elements {
        visit(Some(name.symbol()))?;
        visit(match element {
            PrivateElement::Field(value) => symbol(value),
            PrivateElement::Method(_) => None,
        })?;
    }
    visit(match primitive_data {
        Some(PrimitiveData::Symbol(value)) => Some(value),
        Some(
            PrimitiveData::Boolean(_)
            | PrimitiveData::Number(_)
            | PrimitiveData::String(_)
            | PrimitiveData::BigInt(_),
        )
        | None => None,
    })?;
    if let Some(map) = map {
        for entry in &map.entries {
            if let Some((key, value)) = entry {
                visit(symbol(key))?;
                visit(symbol(value))?;
            } else {
                visit(None)?;
                visit(None)?;
            }
        }
        if matches!(mode, Mode::Physical) {
            for key in map.index.keys() {
                visit(collection_symbol(key))?;
            }
        }
    }
    if let Some(set) = set {
        for value in &set.entries {
            visit(value.as_ref().and_then(symbol))?;
        }
        if matches!(mode, Mode::Physical) {
            for key in set.index.keys() {
                visit(collection_symbol(key))?;
            }
        }
    }
    if let Some(resources) = disposable_resources {
        for value in resources
            .entries
            .iter()
            .flat_map(|resource| resource.values())
        {
            visit(symbol(value))?;
        }
    }
    if let Some(data) = array_from_async {
        for value in data.values() {
            visit(symbol(value))?;
        }
    }
    for data in async_execution.as_deref().into_iter().chain(
        generator
            .as_deref()
            .and_then(super::super::generator::GeneratorData::execution),
    ) {
        for value in data.values() {
            visit(symbol(value))?;
        }
        for symbol in data.symbols() {
            visit(Some(symbol))?;
        }
    }
    if let Some(data) = async_disposal {
        for value in data.values() {
            visit(symbol(value))?;
        }
    }
    if let Some(promise) = promise {
        for value in promise.values() {
            visit(symbol(value))?;
        }
    }
    if let Some(data) = promise_aggregate {
        for value in data.values() {
            visit(symbol(value))?;
        }
    }
    // Cells' held values are unconditional strong fields of a reached
    // registry, and physical owners even in an unreachable registry record.
    if let Some(registry) = finalization_registry {
        for cell in &registry.cells {
            visit(symbol(&cell.held))?;
        }
    }
    if matches!(mode, Mode::Physical)
        && let Some(map) = weak_map
    {
        for (_, value) in &map.entries {
            visit(symbol(value))?;
        }
        for (_, value) in &map.symbol_entries {
            visit(symbol(value))?;
        }
    }
    match callable {
        Some(Callable::Bound(bound)) => {
            visit(symbol(&bound.this))?;
            for value in &bound.arguments {
                visit(symbol(value))?;
            }
        }
        Some(Callable::ClassConstructor(class)) => {
            for field in class.elements.fields.iter() {
                visit(field.symbol_name())?;
            }
            for method in class.elements.private_methods.iter() {
                visit(Some(method.name.symbol()))?;
            }
        }
        // Parsed AST, source text and environment/home-object handles do not
        // own runtime Symbols. Their referenced heap records are traced normally.
        Some(
            Callable::Builtin(_)
            | Callable::Arrow(_)
            | Callable::Ordinary(_)
            | Callable::Method(_)
            | Callable::ProxyRevoker(_)
            | Callable::PromiseCapabilityExecutor(_)
            | Callable::PromiseResolvingFunction { .. }
            | Callable::PromiseFinally { .. }
            | Callable::PromiseAggregateElement { .. }
            | Callable::AsyncResume { .. }
            | Callable::AsyncFromSyncUnwrap { .. }
            | Callable::AsyncFromSyncClose(_),
        )
        | None => visit(None)?,
    }
    match iterator {
        Some(IteratorState::Wrapper(wrapper)) | Some(IteratorState::AsyncFromSync(wrapper)) => {
            visit(symbol(&wrapper.next))?
        }
        Some(IteratorState::Helper(helper)) => match &helper.closure {
            Some(HelperClosure::Concat(state)) => {
                if let Some(inner) = &state.inner {
                    visit(symbol(&inner.next))?;
                }
            }
            Some(HelperClosure::Callback(state)) => {
                visit(symbol(&state.iterated.next))?;
                if let Some(inner) = &state.inner {
                    visit(symbol(&inner.next))?;
                }
            }
            Some(HelperClosure::Limit(state)) => visit(symbol(&state.iterated.next))?,
            None => visit(None)?,
        },
        Some(
            IteratorState::Array(_)
            | IteratorState::Map(_)
            | IteratorState::Set(_)
            | IteratorState::String(_)
            | IteratorState::RegExp(_),
        )
        | None => visit(None)?,
    }
    Ok(())
}

pub(super) fn weak_symbols(
    entry: &Entry,
    visit: &mut impl FnMut(Option<&WeakJsSymbol>) -> Result<(), Error>,
) -> Result<(), Error> {
    if let Entry::Object(object) = entry {
        if let Some(map) = &object.weak_map {
            for (key, _) in &map.symbol_entries {
                visit(Some(key))?;
            }
        }
        if let Some(registry) = &object.finalization_registry {
            for cell in &registry.cells {
                for weak in [cell.target.as_ref(), cell.token.as_ref()] {
                    visit(match weak {
                        Some(WeakTarget::Symbol(symbol)) => Some(symbol),
                        Some(WeakTarget::Object(_)) | None => None,
                    })?;
                }
            }
        }
    }
    Ok(())
}

pub(super) fn shared_identity(entry: &Entry, kind: SharedKind) -> Option<*const ()> {
    let Entry::Object(object) = entry else {
        return None;
    };
    let Some(Callable::ClassConstructor(class)) = &object.callable else {
        return None;
    };
    match kind {
        SharedKind::Fields if !class.elements.fields.is_empty() => {
            Some(Rc::as_ptr(&class.elements.fields) as *const ())
        }
        SharedKind::PrivateMethods if !class.elements.private_methods.is_empty() => {
            Some(Rc::as_ptr(&class.elements.private_methods) as *const ())
        }
        _ => None,
    }
}

pub(super) fn shared_counts(entry: &Entry, kind: SharedKind) -> Option<(usize, usize)> {
    let Entry::Object(object) = entry else {
        return None;
    };
    let Some(Callable::ClassConstructor(class)) = &object.callable else {
        return None;
    };
    Some(match kind {
        SharedKind::Fields => (
            Rc::strong_count(&class.elements.fields),
            Rc::weak_count(&class.elements.fields),
        ),
        SharedKind::PrivateMethods => (
            Rc::strong_count(&class.elements.private_methods),
            Rc::weak_count(&class.elements.private_methods),
        ),
    })
}

pub(super) fn shared_symbols(
    entry: &Entry,
    kind: SharedKind,
    visit: &mut impl FnMut(Option<&JsSymbol>) -> Result<(), Error>,
) -> Result<(), Error> {
    let Entry::Object(object) = entry else {
        return Err(Error::CollectionState);
    };
    let Some(Callable::ClassConstructor(class)) = &object.callable else {
        return Err(Error::CollectionState);
    };
    match kind {
        SharedKind::Fields => {
            for field in class.elements.fields.iter() {
                visit(field.symbol_name())?;
            }
        }
        SharedKind::PrivateMethods => {
            for method in class.elements.private_methods.iter() {
                visit(Some(method.name.symbol()))?;
            }
        }
    }
    Ok(())
}
