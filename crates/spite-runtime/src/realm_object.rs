//! Realm ownership and collection outside active evaluation.

use crate::{
    Collection, Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{self, DataDescriptor, DescriptorKind, OrdinaryObject, Property, PropertyDescriptor},
};
use spite_bigint::BigInt;
use spite_core::{JsString, PropertyKey, PropertyKeyRef, Span, WellKnownSymbol};
use spite_parser::ast::{ExprKind, Literal, ObjectElement, PropertyKind, PropertyName};

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
    /// Reads a string- or symbol-keyed property with JavaScript Get semantics, including
    /// inherited accessors and the original receiver. Each host call may execute
    /// JavaScript getters and resets the work allowance if `max_steps` is enabled.
    ///
    /// The result is unrooted; use [`Self::root_value`] across explicit collection.
    /// Nullish values throw TypeError, and invalid object handles are rejected.
    pub fn read_property<'key>(
        &mut self,
        value: &Value,
        key: impl Into<PropertyKeyRef<'key>>,
    ) -> Result<Value, Error> {
        let key = key.into();
        if let Value::Object(handle) = value {
            self.objects.inspect(handle).map_err(Error::InvalidObject)?;
        }
        self.initialize_realm()?;
        self.remaining_steps = self.limits.max_steps;
        let span = Span::new(0, 0);
        self.tick(span)?;
        Self::require_object_coercible(value, span)?;
        if key.as_string().is_some_and(|key| {
            self.limits
                .max_string_units
                .is_some_and(|limit| key.len() > limit)
        }) {
            return Err(Error::Limit {
                span,
                message: "property key length limit exceeded".into(),
            });
        }
        self.object_work(span, |_, budget| {
            budget.charge(key.as_string().map_or(1, JsString::len))
        })?;
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
        // ToPrimitive, 7.1.1: GetMethod observes inherited accessors before
        // ordinary conversion. Only null/undefined mean the hook is absent.
        let method = self.get_property(&object, &WellKnownSymbol::ToPrimitive.symbol(), span)?;
        if !matches!(method, Value::Undefined | Value::Null) {
            if !self.is_callable(&method, span)? {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "Symbol.toPrimitive must be callable",
                ));
            }
            let hint = match hint {
                Hint::Default => "default",
                Hint::Number => "number",
                Hint::String => "string",
            };
            let result = self.call(
                method,
                Value::Object(object.clone()),
                vec![Value::String(JsString::from(hint))],
                span,
            )?;
            if !matches!(result, Value::Object(_)) {
                return Ok(result);
            }
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Symbol.toPrimitive returned an object",
            ));
        }
        // OrdinaryToPrimitive, 7.1.1.1, retains receiver and hint order.
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
        if let Value::String(string) = primitive {
            return Ok(string);
        }
        self.conversion_work(span, |budget| primitive.to_js_string(budget))
    }

    pub(super) fn has_property<'key>(
        &mut self,
        object: &ObjectHandle,
        key: impl Into<PropertyKeyRef<'key>>,
        span: Span,
    ) -> Result<bool, Error> {
        let key = key.into();
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

    pub(super) fn get_property_value<'key>(
        &mut self,
        base: &Value,
        key: impl Into<PropertyKeyRef<'key>>,
        span: Span,
    ) -> Result<Value, Error> {
        let key = key.into();
        if let Value::Object(object) = base {
            return self.get_property(object, key, span);
        }
        if let Value::String(string) = base {
            self.tick(span)?;
            if key_is(key, "length") {
                return Ok(Value::Number(string.len() as f64));
            }
            if let Some(index) = string_index(string, key) {
                return Ok(Value::String(JsString::from_code_units(vec![
                    string.code_units()[index],
                ])));
            }
        }
        if let Some(prototype) = self.primitive_prototype(base) {
            return self.get_property_with_receiver(&prototype, key, base.clone(), span);
        }
        self.tick(span)?;
        Ok(Value::Undefined)
    }

    pub(super) fn set_property_value(
        &mut self,
        base: &Value,
        key: impl Into<PropertyKey>,
        value: Value,
        span: Span,
    ) -> Result<bool, Error> {
        let key = key.into();
        if let Value::Object(object) = base {
            return self.set_property_with_receiver(object, key, value, base.clone(), span);
        }
        if matches!(base,Value::String(string) if key_is(&key,"length") || string_index(string,&key).is_some())
        {
            self.tick(span)?;
            return Ok(false);
        }
        if let Some(prototype) = self.primitive_prototype(base) {
            return self.set_property_with_receiver(&prototype, key, value, base.clone(), span);
        }
        self.tick(span)?;
        Ok(false)
    }

    pub(super) fn set_property_with_receiver(
        &mut self,
        target: &ObjectHandle,
        key: PropertyKey,
        value: Value,
        receiver: Value,
        span: Span,
    ) -> Result<bool, Error> {
        // OrdinarySet / OrdinarySetWithOwnDescriptor (10.1.9.1–2). Descriptor
        // reads use the Realm boundary so missing intrinsic properties cannot
        // be treated as absent, including on a distinct write receiver.
        let mut current = target.clone();
        let descriptor = loop {
            if let Some(property) = self.own_property_descriptor(&current, &key, span)? {
                break Some(property);
            }
            let parent = self.object_work(span, |objects, _| {
                Ok(objects.inspect(&current)?.prototype().cloned())
            })?;
            let Some(parent) = parent else { break None };
            current = parent;
        };
        match descriptor {
            Some(Property::Accessor(property)) => {
                let Some(setter) = property.set else {
                    return Ok(false);
                };
                self.call(Value::Object(setter), receiver, vec![value], span)?;
                return Ok(true);
            }
            Some(Property::Data(property)) if !property.writable => return Ok(false),
            _ => {}
        }
        let Value::Object(receiver) = receiver else {
            return Ok(false);
        };
        let descriptor = match self.own_property_descriptor(&receiver, &key, span)? {
            Some(Property::Data(property)) if property.writable => DataDescriptor {
                value: Some(value),
                ..Default::default()
            },
            Some(_) => return Ok(false),
            None => DataDescriptor {
                value: Some(value),
                writable: Some(true),
                enumerable: Some(true),
                configurable: Some(true),
            },
        };
        self.define_property(&receiver, key, descriptor.into(), span)
    }

    pub(super) fn delete_property_value<'key>(
        &mut self,
        base: &Value,
        key: impl Into<PropertyKeyRef<'key>>,
        span: Span,
    ) -> Result<bool, Error> {
        let key = key.into();
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

    pub(super) fn reference_key(
        &mut self,
        key: &mut Value,
        span: Span,
    ) -> Result<PropertyKey, Error> {
        let converted = self.property_key(key.clone(), span)?;
        *key = match &converted {
            PropertyKey::String(key) => Value::String(key.clone()),
            PropertyKey::Symbol(key) => Value::Symbol(key.clone()),
        };
        Ok(converted)
    }

    pub(super) fn get_property<'key>(
        &mut self,
        object: &ObjectHandle,
        key: impl Into<PropertyKeyRef<'key>>,
        span: Span,
    ) -> Result<Value, Error> {
        let key = key.into();
        self.get_property_with_receiver(object, key, Value::Object(object.clone()), span)
    }

    pub(super) fn get_property_with_receiver<'key>(
        &mut self,
        object: &ObjectHandle,
        key: impl Into<PropertyKeyRef<'key>>,
        receiver: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let key = key.into();
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

    /// Collects between evaluations, retaining bindings, intrinsics, cached templates,
    /// and host roots.
    ///
    /// Retain returned or thrown object values with [`Self::root_value`] before
    /// calling this method. Unrooted host values may become stale. Evaluation and
    /// allocation never call the collector, so live evaluation temporaries cannot
    /// be collected. Environment and template-registry scans consume the supplied
    /// work budget too.
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
            .and_then(|count| count.checked_add(self.template_map.len()))
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
        let roots = roots.chain(self.template_map.iter().map(|(_, array)| array));
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
        let mut budget = object::Budget::with_work_limit(self.remaining_steps);
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
            object::Error::UnnormalizedArrayLength => {
                unreachable!("realm converts array length values before storage")
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

    pub(super) fn property_key(&mut self, value: Value, span: Span) -> Result<PropertyKey, Error> {
        let value = self.primitive(value, Hint::String, span)?;
        // ToPropertyKey, 7.1.19: preserve Symbol identity after ToPrimitive.
        if let Value::Symbol(symbol) = value {
            return Ok(PropertyKey::Symbol(symbol));
        }
        let key = self.string(value, span)?;
        if self
            .limits
            .max_string_units
            .is_some_and(|limit| key.len() > limit)
        {
            return Err(Error::Limit {
                span,
                message: "property key length limit exceeded".into(),
            });
        }
        Ok(PropertyKey::String(key))
    }

    pub(super) fn object_literal(
        &mut self,
        properties: &[ObjectElement],
        span: Span,
    ) -> Result<Value, Error> {
        let prototype = self.ensure_object_intrinsics(span)?;
        let object = self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
        for element in properties {
            let property = match element {
                ObjectElement::Spread(expression) => {
                    let source = self.expression(expression)?;
                    self.copy_spread_properties(&object, source, expression.span)?;
                    continue;
                }
                ObjectElement::Property(property) => property,
            };
            self.tick(property.span)?;
            let key = match &property.name {
                PropertyName::Literal(literal) => self.literal_value(literal, property.span)?,
                PropertyName::Computed(expression) => self.expression(expression)?,
            };
            // ToPropertyKey precedes evaluation of the property's value.
            let key = self.property_key(key, property.span)?;
            if matches!(
                property.kind,
                PropertyKind::Method | PropertyKind::Getter | PropertyKind::Setter
            ) {
                let ExprKind::Function(syntax) = &property.value.kind else {
                    unreachable!("method syntax");
                };
                let function = self.method_function(
                    syntax,
                    &object,
                    key.clone(),
                    property.kind,
                    property.span,
                )?;
                let descriptor = match property.kind {
                    PropertyKind::Method => DataDescriptor {
                        value: Some(Value::Object(function)),
                        writable: Some(true),
                        enumerable: Some(true),
                        configurable: Some(true),
                    }
                    .into(),
                    PropertyKind::Getter => PropertyDescriptor {
                        kind: DescriptorKind::Accessor {
                            get: Some(Some(function)),
                            set: None,
                        },
                        enumerable: Some(true),
                        configurable: Some(true),
                    },
                    PropertyKind::Setter => PropertyDescriptor {
                        kind: DescriptorKind::Accessor {
                            get: None,
                            set: Some(Some(function)),
                        },
                        enumerable: Some(true),
                        configurable: Some(true),
                    },
                    _ => unreachable!("method property kind"),
                };
                self.define_property_or_throw(&object, key, descriptor, property.span)?;
                continue;
            }
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
                debug_assert!(created, "new literal has only configurable properties");
            }
        }
        Ok(Value::Object(object))
    }
}

impl Realm {
    pub(crate) fn set_property_or_throw(
        &mut self,
        object: &ObjectHandle,
        key: impl Into<PropertyKey>,
        value: Value,
        span: Span,
    ) -> Result<(), Error> {
        let key = key.into();
        // Set(O, P, V, true), 7.3.4, including inherited setters.
        if !self.set_property_value(&Value::Object(object.clone()), key, value, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "property write was rejected",
            ));
        }
        Ok(())
    }

    pub(crate) fn delete_property_or_throw<'key>(
        &mut self,
        object: &ObjectHandle,
        key: impl Into<PropertyKeyRef<'key>>,
        span: Span,
    ) -> Result<(), Error> {
        let key = key.into();
        // DeletePropertyOrThrow, 7.3.10, retains earlier observable effects.
        if !self.delete_property_value(&Value::Object(object.clone()), key, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "property deletion was rejected",
            ));
        }
        Ok(())
    }

    pub(crate) fn length_of_array_like(
        &mut self,
        object: &ObjectHandle,
        span: Span,
    ) -> Result<u64, Error> {
        // LengthOfArrayLike/ToLength, 7.3.18/7.1.20. Read length once and use
        // u64 to represent every valid index independently of host usize.
        let length = self.get_property(object, &JsString::from("length"), span)?;
        let number = self.number(length, span)?;
        Ok(if number.is_nan() || number <= 0.0 {
            0
        } else {
            number.trunc().min(9_007_199_254_740_991.0) as u64
        })
    }

    pub(crate) fn own_property_keys(
        &mut self,
        object: &ObjectHandle,
        span: Span,
    ) -> Result<Vec<PropertyKey>, Error> {
        let intrinsics = self.intrinsics.as_ref().expect("initialized");
        if self.global_object.as_ref() == Some(object)
            || object == &intrinsics.string.prototype
            || object == &intrinsics.array.constructor
        {
            return Err(Self::unsupported(
                span,
                "own keys of this incomplete intrinsic are not implemented",
            ));
        }
        self.object_work(span, |objects, budget| objects.own_keys(object, budget))
    }

    pub(crate) fn own_property_descriptor<'key>(
        &mut self,
        object: &ObjectHandle,
        key: impl Into<PropertyKeyRef<'key>>,
        span: Span,
    ) -> Result<Option<Property>, Error> {
        let key = key.into();
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

    pub(crate) fn check_missing_intrinsic_mutation<'key>(
        &mut self,
        object: &ObjectHandle,
        key: impl Into<PropertyKeyRef<'key>>,
        span: Span,
    ) -> Result<(), Error> {
        let key = key.into();
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

    pub(crate) fn missing_intrinsic_property<'key>(
        &self,
        object: &ObjectHandle,
        key: impl Into<PropertyKeyRef<'key>>,
    ) -> bool {
        let key = key.into();
        let Some(key) = key.as_string() else {
            return false;
        };
        if self.global_object.as_ref() == Some(object) && self.missing_global_property(key) {
            return true;
        }
        let Some(intrinsics) = &self.intrinsics else {
            return false;
        };
        (object == &intrinsics.string.prototype && missing_string_method(key))
            || (object == &intrinsics.array.constructor && missing_array_static(key))
    }
}

fn missing_array_static(key: &JsString) -> bool {
    key_is(key, "fromAsync")
}

fn key_is<'key>(key: impl Into<PropertyKeyRef<'key>>, name: &str) -> bool {
    key.into()
        .as_string()
        .is_some_and(|key| key.code_units().iter().copied().eq(name.encode_utf16()))
}

fn missing_string_method(key: &JsString) -> bool {
    ["localeCompare", "match", "matchAll", "normalize", "search"]
        .iter()
        .any(|name| key_is(key, name))
}

// 10.4.3.5: only canonical, non-negative integral Number names below the string
// length identify characters. Decimal parsing bounds work by usize's width;
// the final Number::toString check rejects decimal integers rounded by binary64.
fn string_index<'key>(string: &JsString, key: impl Into<PropertyKeyRef<'key>>) -> Option<usize> {
    let key = key.into().as_string()?;
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
