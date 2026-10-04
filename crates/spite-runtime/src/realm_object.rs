//! Realm ownership and collection outside active evaluation.

use crate::{
    Collection, Error, ObjectHandle, Realm, Value,
    object::{self, DataDescriptor, OrdinaryObject, Property, SetAction},
};
use spite_bigint::BigInt;
use spite_core::{JsString, Span};
use spite_parser::ast::{Literal, ObjectProperty, PropertyKind, PropertyName};

pub(super) enum Hint {
    Default,
    Number,
    String,
}

/// An embedding value whose object, if any, remains rooted while this token lives.
#[derive(Clone, Debug)]
pub struct RootedValue {
    value: Value,
    // Cloning/dropping this token controls the root's lifetime.
    _root: Option<object::Root>,
}

impl RootedValue {
    /// Borrows the value. Cloning this value alone does not clone its root token.
    pub fn value(&self) -> &Value {
        &self.value
    }
}

impl Realm {
    /// Reads a string-keyed property with JavaScript Get semantics, including
    /// inherited accessors and the original receiver. Each host call starts a
    /// fresh evaluation work budget and may execute JavaScript getters.
    ///
    /// The result is unrooted; use [`Self::root_value`] across explicit collection.
    /// Nullish values throw TypeError, and invalid object handles are rejected.
    pub fn read_property(&mut self, value: &Value, key: &JsString) -> Result<Value, Error> {
        if let Value::Object(handle) = value {
            self.objects.inspect(handle).map_err(Error::InvalidObject)?;
        }
        self.initialize_realm()?;
        self.remaining_steps = self.limits.max_steps;
        let span = Span::new(0, 0);
        self.tick(span)?;
        Self::require_object_coercible(value, span)?;
        if key.len() > self.limits.max_string_units {
            return Err(Error::Limit {
                span,
                message: "property key length limit exceeded".into(),
            });
        }
        self.object_work(span, |_, budget| budget.charge(key.len()))?;
        let result = self.get_property_value(value, key, span)?;
        self.check_string(&result, span)?;
        Ok(result)
    }

    pub(super) fn primitive(
        &mut self,
        value: Value,
        hint: Hint,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(object) = value else {
            return Ok(value);
        };
        // 7.1.1 / 7.1.1.1. Symbol-keyed hooks cannot be installed yet. Ordinary
        // method lookup and calls retain their receiver and requested hint order.
        let names = match hint {
            Hint::String => ["toString", "valueOf"],
            Hint::Default | Hint::Number => ["valueOf", "toString"],
        };
        for name in names {
            let method = self.get_property(&object, &JsString::from(name), span)?;
            if self.is_callable(&method, span)? {
                let result = self.call(method, Value::Object(object.clone()), Vec::new(), span)?;
                if !matches!(result, Value::Object(_)) {
                    return Ok(result);
                }
            }
        }
        Err(Self::exception(
            crate::ExceptionKind::TypeError,
            span,
            "cannot convert object to a primitive value",
        ))
    }

    pub(super) fn string(&mut self, value: Value, span: Span) -> Result<JsString, Error> {
        let primitive = self.primitive(value, Hint::String, span)?;
        self.conversion_work(span, |budget| primitive.to_js_string(budget))
    }

    pub(super) fn has_property(
        &mut self,
        object: &ObjectHandle,
        key: &JsString,
        span: Span,
    ) -> Result<bool, Error> {
        let mut next = Some(object.clone());
        while let Some(handle) = next {
            if self.object_work(span, |objects, budget| {
                objects.has_own(&handle, key, budget)
            })? || self.missing_intrinsic_property(&handle, key)
            {
                return Ok(true);
            }
            next = self.object_work(span, |objects, _| {
                Ok(objects.inspect(&handle)?.prototype().cloned())
            })?;
        }
        Ok(false)
    }

    pub(super) fn require_object_coercible(base: &Value, span: Span) -> Result<(), Error> {
        match base {
            Value::Null | Value::Undefined => Err(Self::exception(
                crate::ExceptionKind::TypeError,
                span,
                "cannot access a property of null or undefined",
            )),
            _ => Ok(()),
        }
    }

    pub(super) fn get_property_value(
        &mut self,
        base: &Value,
        key: &JsString,
        span: Span,
    ) -> Result<Value, Error> {
        if let Value::Object(object) = base {
            return self.get_property(object, key, span);
        }
        if let Some(prototype) = self.primitive_prototype(base) {
            return self.get_property_with_receiver(&prototype, key, base.clone(), span);
        }
        self.tick(span)?;
        if let Value::String(string) = base {
            if key_is(key, "length") {
                return Ok(Value::Number(string.len() as f64));
            }
            if let Some(index) = string_index(string, key) {
                return Ok(Value::String(JsString::from_code_units(vec![
                    string.code_units()[index],
                ])));
            }
        }
        if missing_primitive_method(base, key) {
            return Err(Self::unsupported(
                span,
                "primitive prototype method is not implemented",
            ));
        }
        Ok(Value::Undefined)
    }

    pub(super) fn set_property_value(
        &mut self,
        base: &Value,
        key: JsString,
        value: Value,
        span: Span,
    ) -> Result<bool, Error> {
        if let Value::Object(object) = base {
            self.check_global_property_operation(object, &key, span)?;
            self.check_missing_intrinsic_mutation(object, &key, span)?;
            let action = self.object_work(span, |objects, budget| {
                budget.value(&value)?;
                objects.set(object, key, value.clone(), Some(object), budget)
            })?;
            return match action {
                SetAction::Done(result) => Ok(result),
                SetAction::Call(setter) => {
                    self.call(Value::Object(setter), base.clone(), vec![value], span)?;
                    Ok(true)
                }
            };
        }
        if let Some(prototype) = self.primitive_prototype(base) {
            let action = self.object_work(span, |objects, budget| {
                budget.value(&value)?;
                objects.set(&prototype, key, value.clone(), None, budget)
            })?;
            return match action {
                SetAction::Done(result) => Ok(result),
                SetAction::Call(setter) => {
                    self.call(Value::Object(setter), base.clone(), vec![value], span)?;
                    Ok(true)
                }
            };
        }
        // GetThisValue retains the primitive receiver. String own properties
        // reject writes; ordinary inherited data properties reject non-objects
        // as receivers (10.1.9.2). The remaining primitive prototypes do not yet
        // expose setters.
        self.tick(span)?;
        Ok(false)
    }

    pub(super) fn delete_property_value(
        &mut self,
        base: &Value,
        key: &JsString,
        span: Span,
    ) -> Result<bool, Error> {
        if let Value::Object(object) = base {
            self.check_global_property_operation(object, key, span)?;
            self.check_missing_intrinsic_mutation(object, key, span)?;
            return self.object_work(span, |objects, budget| objects.delete(object, key, budget));
        }
        self.tick(span)?;
        Ok(
            !matches!(base, Value::String(string) if key_is(key, "length") || string_index(string, key).is_some()),
        )
    }

    pub(super) fn reference_key(&mut self, key: &mut Value, span: Span) -> Result<JsString, Error> {
        let converted = self.property_key(key.clone(), span)?;
        *key = Value::String(converted.clone());
        Ok(converted)
    }

    pub(super) fn get_property(
        &mut self,
        object: &ObjectHandle,
        key: &JsString,
        span: Span,
    ) -> Result<Value, Error> {
        self.get_property_with_receiver(object, key, Value::Object(object.clone()), span)
    }

    fn get_property_with_receiver(
        &mut self,
        object: &ObjectHandle,
        key: &JsString,
        receiver: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let mut next = Some(object.clone());
        while let Some(handle) = next {
            let own = self.object_work(span, |objects, budget| {
                objects.get_own(&handle, key, budget)
            })?;
            if let Some(property) = own {
                return match property {
                    Property::Data(data) => Ok(data.value),
                    Property::Accessor(accessor) => match accessor.get {
                        Some(getter) => {
                            self.call(Value::Object(getter), receiver.clone(), Vec::new(), span)
                        }
                        None => Ok(Value::Undefined),
                    },
                };
            }
            // Missing standard methods must not appear to be absent. Until
            // callable intrinsics are implemented, accessing one is a host gap.
            if self.missing_intrinsic_property(&handle, key) {
                return Err(Self::unsupported(
                    span,
                    "intrinsic prototype property is not implemented",
                ));
            }
            next = self.object_work(span, |objects, _| {
                Ok(objects.inspect(&handle)?.prototype().cloned())
            })?;
        }
        Ok(Value::Undefined)
    }

    /// Retains an embedding value across explicit collection in this realm.
    ///
    /// Objects from another realm and already-collected handles are rejected.
    /// Primitive values need no heap root. `max_work` bounds root registration.
    pub fn root_value(
        &mut self,
        value: Value,
        max_work: usize,
    ) -> Result<RootedValue, object::Error> {
        let root = if let Value::Object(handle) = &value {
            Some(
                self.objects
                    .root(handle, &mut object::Budget::new(max_work))?,
            )
        } else {
            None
        };
        Ok(RootedValue { value, _root: root })
    }

    /// Borrows implemented object storage for host inspection.
    ///
    /// This is not a JavaScript reflection operation. Intrinsic methods that are
    /// not yet implemented do not have stored descriptors at this stage.
    pub fn inspect_object(&self, handle: &ObjectHandle) -> Result<&OrdinaryObject, object::Error> {
        self.objects.inspect(handle)
    }

    /// Collects between evaluations, retaining bindings, intrinsics, and host roots.
    ///
    /// Retain returned or thrown object values with [`Self::root_value`] before
    /// calling this method. Unrooted host values may become stale. Evaluation and
    /// allocation never call the collector, so live evaluation temporaries cannot
    /// be collected. Environment scans consume the supplied work budget too.
    pub fn collect(&mut self, max_work: usize) -> Result<Collection, spite_heap::Error> {
        let scanned = self
            .scopes
            .len()
            .checked_add(1)
            .and_then(|count| {
                count.checked_add(
                    self.intrinsics
                        .as_ref()
                        .map_or(0, |intrinsics| intrinsics.roots().count()),
                )
            })
            .ok_or(spite_heap::Error::Limit)?;
        let remaining = max_work
            .checked_sub(scanned)
            .ok_or(spite_heap::Error::Limit)?;
        let roots = self
            .global_object
            .iter()
            .chain(self.scopes.iter().map(|scope| &scope.0))
            .chain(
                self.intrinsics
                    .iter()
                    .flat_map(|intrinsics| intrinsics.roots()),
            );
        let mut result = self.objects.collect(roots, remaining)?;
        result.work_used += scanned;
        Ok(result)
    }

    pub(super) fn object_work<T>(
        &mut self,
        span: Span,
        work: impl FnOnce(&mut object::Objects, &mut object::Budget) -> Result<T, object::Error>,
    ) -> Result<T, Error> {
        self.tick(span)?;
        let mut budget = object::Budget::new(self.remaining_steps);
        let result = work(&mut self.objects, &mut budget);
        self.remaining_steps = budget.remaining_work();
        result.map_err(|error| match error {
            object::Error::WorkLimit
            | object::Error::PropertyLimit
            | object::Error::Heap(spite_heap::Error::Capacity | spite_heap::Error::Limit) => {
                Error::Limit {
                    span,
                    message: error.to_string(),
                }
            }
            object::Error::NotCallable | object::Error::WrongKind => {
                unreachable!("realm operations validate function callability before storage")
            }
            object::Error::Heap(
                spite_heap::Error::ForeignHandle | spite_heap::Error::StaleHandle,
            ) => {
                unreachable!("realm-internal values retain valid handles between collections")
            }
        })
    }

    pub(super) fn literal_value(&mut self, literal: &Literal, span: Span) -> Result<Value, Error> {
        Ok(match literal {
            Literal::Null => Value::Null,
            Literal::Boolean(value) => Value::Boolean(*value),
            Literal::Number(value) => Value::Number(*value),
            Literal::String(value) => Value::String(value.clone()),
            Literal::BigInt { digits, radix } => Value::BigInt(
                self.integer_work(span, |budget| BigInt::parse_digits(digits, *radix, budget))?,
            ),
        })
    }

    pub(super) fn property_key(&mut self, value: Value, span: Span) -> Result<JsString, Error> {
        let key = self.string(value, span)?;
        if key.len() > self.limits.max_string_units {
            return Err(Error::Limit {
                span,
                message: "property key length limit exceeded".into(),
            });
        }
        Ok(key)
    }

    pub(super) fn object_literal(
        &mut self,
        properties: &[ObjectProperty],
        span: Span,
    ) -> Result<Value, Error> {
        let prototype = self.ensure_object_intrinsics(span)?;
        let object = self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
        for property in properties {
            self.tick(property.span)?;
            let key = match &property.name {
                PropertyName::Literal(literal) => self.literal_value(literal, property.span)?,
                PropertyName::Computed(expression) => self.expression(expression)?,
            };
            // ToPropertyKey precedes evaluation of the property's value.
            let key = self.property_key(key, property.span)?;
            let value = if property.kind == PropertyKind::Prototype {
                self.expression(&property.value)?
            } else {
                self.named_expression(&property.value, key.clone())?
            };
            if property.kind == PropertyKind::Prototype {
                let prototype = match &value {
                    Value::Object(handle) => Some(handle),
                    Value::Null => None,
                    _ => continue,
                };
                let changed = self.object_work(property.span, |objects, budget| {
                    objects.set_prototype(&object, prototype, budget)
                })?;
                debug_assert!(
                    changed,
                    "new literal is extensible and cannot be its own ancestor"
                );
            } else {
                let created = self.object_work(property.span, |objects, budget| {
                    objects.define(
                        &object,
                        key,
                        DataDescriptor {
                            value: Some(value),
                            writable: Some(true),
                            enumerable: Some(true),
                            configurable: Some(true),
                        },
                        budget,
                    )
                })?;
                debug_assert!(created, "new literal has only configurable data properties");
            }
        }
        Ok(Value::Object(object))
    }
}

impl Realm {
    pub(crate) fn own_property_keys(
        &mut self,
        object: &ObjectHandle,
        span: Span,
    ) -> Result<Vec<JsString>, Error> {
        let intrinsics = self.intrinsics.as_ref().expect("initialized");
        if self.global_object.as_ref() == Some(object)
            || object == &intrinsics.object.constructor
            || object == &intrinsics.function_prototype
        {
            return Err(Self::unsupported(
                span,
                "own keys of this incomplete intrinsic are not implemented",
            ));
        }
        self.object_work(span, |objects, budget| objects.own_keys(object, budget))
    }

    pub(crate) fn own_property_descriptor(
        &mut self,
        object: &ObjectHandle,
        key: &JsString,
        span: Span,
    ) -> Result<Option<Property>, Error> {
        let property =
            self.object_work(span, |objects, budget| objects.get_own(object, key, budget))?;
        if property.is_none() && self.missing_intrinsic_property(object, key) {
            return Err(Self::unsupported(
                span,
                "intrinsic property descriptor is not implemented",
            ));
        }
        Ok(property)
    }

    pub(crate) fn check_missing_intrinsic_mutation(
        &mut self,
        object: &ObjectHandle,
        key: &JsString,
        span: Span,
    ) -> Result<(), Error> {
        if self.missing_intrinsic_property(object, key)
            && !self.object_work(span, |objects, budget| objects.has_own(object, key, budget))?
        {
            return Err(Self::unsupported(
                span,
                "intrinsic property descriptor is not implemented",
            ));
        }
        Ok(())
    }

    pub(crate) fn missing_intrinsic_property(&self, object: &ObjectHandle, key: &JsString) -> bool {
        if self.global_object.as_ref() == Some(object) && self.missing_global_property(key) {
            return true;
        }
        let Some(intrinsics) = &self.intrinsics else {
            return false;
        };
        (object == &intrinsics.object.constructor && missing_object_static(key))
            || (object == &intrinsics.function_prototype && key_is(key, "constructor"))
    }
}

fn missing_object_static(key: &JsString) -> bool {
    [
        "assign",
        "entries",
        "fromEntries",
        "getOwnPropertyDescriptors",
        "getOwnPropertyNames",
        "getOwnPropertySymbols",
        "groupBy",
        "keys",
        "values",
    ]
    .iter()
    .any(|name| key_is(key, name))
}

fn missing_object_method(key: &JsString) -> bool {
    [
        "constructor",
        "hasOwnProperty",
        "isPrototypeOf",
        "propertyIsEnumerable",
        "toLocaleString",
    ]
    .iter()
    .any(|name| key_is(key, name))
}

fn key_is(key: &JsString, name: &str) -> bool {
    key.code_units().iter().copied().eq(name.encode_utf16())
}

fn missing_primitive_method(base: &Value, key: &JsString) -> bool {
    if missing_object_method(key) || key_is(key, "toString") || key_is(key, "valueOf") {
        return true;
    }
    let names: &[&str] = match base {
        Value::String(_) => &[
            "at",
            "charAt",
            "charCodeAt",
            "codePointAt",
            "concat",
            "endsWith",
            "includes",
            "indexOf",
            "isWellFormed",
            "lastIndexOf",
            "localeCompare",
            "match",
            "matchAll",
            "normalize",
            "padEnd",
            "padStart",
            "repeat",
            "replace",
            "replaceAll",
            "search",
            "slice",
            "split",
            "startsWith",
            "substring",
            "toLocaleLowerCase",
            "toLocaleUpperCase",
            "toLowerCase",
            "toUpperCase",
            "toWellFormed",
            "trim",
            "trimEnd",
            "trimStart",
        ],
        Value::Number(_) => &["toExponential", "toLocaleString"],
        Value::BigInt(_) => &["toLocaleString"],
        _ => &[],
    };
    names.iter().any(|name| key_is(key, name))
}

// 10.4.3.5: only canonical, non-negative integral Number names below the string
// length identify characters. Decimal parsing bounds work by usize's width;
// the final Number::toString check rejects decimal integers rounded by binary64.
fn string_index(string: &JsString, key: &JsString) -> Option<usize> {
    let units = key.code_units();
    if units.is_empty() || units.len() > 20 || units.len() > 1 && units[0] == u16::from(b'0') {
        return None;
    }
    let mut index = 0usize;
    for &unit in units {
        let digit = unit.checked_sub(u16::from(b'0'))?;
        if digit > 9 {
            return None;
        }
        index = index.checked_mul(10)?.checked_add(usize::from(digit))?;
    }
    if index >= string.len() || !key_is(key, &crate::value::number_to_string(index as f64)) {
        return None;
    }
    Some(index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ExceptionKind,
        object::{Budget, DescriptorKind, PropertyDescriptor},
    };

    #[test]
    fn inherited_accessors_receive_the_original_object_and_do_not_create_data_properties() {
        let mut realm = Realm::default();
        let Value::Object(base) = realm
            .eval("let base = {}; let child = {__proto__: base}; base")
            .unwrap()
        else {
            panic!("object")
        };
        let value_of = realm.intrinsics.as_ref().unwrap().object_value_of.clone();
        for (name, get, set) in [
            ("both", Some(value_of.clone()), Some(value_of)),
            ("empty", None, None),
        ] {
            realm
                .objects
                .define(
                    &base,
                    JsString::from(name),
                    PropertyDescriptor {
                        kind: DescriptorKind::Accessor {
                            get: Some(get),
                            set: Some(set),
                        },
                        enumerable: Some(true),
                        configurable: Some(true),
                    },
                    &mut Budget::new(1000),
                )
                .unwrap();
        }
        assert_eq!(
            realm.eval("child.both === child && base.both === base"),
            Ok(Value::Boolean(true))
        );
        assert_eq!(
            realm.eval("child.both = 7; child.both === child"),
            Ok(Value::Boolean(true))
        );
        assert_eq!(
            realm.eval("'empty' in child && child.empty === undefined"),
            Ok(Value::Boolean(true))
        );
        assert_eq!(
            realm.eval("child.empty = 7; child.empty"),
            Ok(Value::Undefined)
        );
        assert!(matches!(
            realm.eval("'use strict'; child.empty = 7"),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        assert_eq!(
            realm.eval("delete child.both; child.both === child"),
            Ok(Value::Boolean(true))
        );
        assert_eq!(
            realm.eval("delete base.both; child.both"),
            Ok(Value::Undefined)
        );
    }
}
