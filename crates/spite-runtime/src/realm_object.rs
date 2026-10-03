//! Realm ownership and collection outside active evaluation.

use crate::{
    Collection, Error, ObjectHandle, Realm, Value,
    object::{self, DataDescriptor, OrdinaryObject},
};
use spite_bigint::BigInt;
use spite_core::{JsString, Span};
use spite_parser::ast::{Literal, ObjectProperty, PropertyKind, PropertyName};

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
    pub(super) fn property_object(base: &Value, span: Span) -> Result<&ObjectHandle, Error> {
        match base {
            Value::Object(handle) => Ok(handle),
            Value::Null | Value::Undefined => Err(Self::exception(
                crate::ExceptionKind::TypeError,
                span,
                "cannot access a property of null or undefined",
            )),
            _ => Err(Self::unsupported(
                span,
                "primitive property access requires wrapper objects",
            )),
        }
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
        let mut next = Some(object.clone());
        while let Some(handle) = next {
            let own = self.object_work(span, |objects, budget| {
                objects.get_own(&handle, key, budget)
            })?;
            if let Some(property) = own {
                return Ok(property.value);
            }
            // Missing standard methods must not appear to be absent. Until
            // callable intrinsics are implemented, accessing one is a host gap.
            if self.object_prototype.as_ref() == Some(&handle) && missing_object_method(key) {
                return Err(Self::unsupported(
                    span,
                    "Object.prototype method is not implemented",
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
        let scanned = self.scopes.iter().try_fold(
            self.globals
                .len()
                .checked_add(1)
                .ok_or(spite_heap::Error::Limit)?,
            |count, scope| {
                count
                    .checked_add(scope.len())
                    .and_then(|n| n.checked_add(1))
                    .ok_or(spite_heap::Error::Limit)
            },
        )?;
        let remaining = max_work
            .checked_sub(scanned)
            .ok_or(spite_heap::Error::Limit)?;
        let bindings = self
            .scopes
            .iter()
            .flat_map(|scope| scope.values())
            .filter_map(|binding| binding.value.as_ref());
        let globals = self.globals.values().map(|binding| &binding.value);
        let roots = bindings
            .chain(globals)
            .filter_map(|value| match value {
                Value::Object(handle) => Some(handle),
                _ => None,
            })
            .chain(self.object_prototype.iter());
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
        let key = self.conversion_work(span, |budget| value.to_js_string(budget))?;
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
        // Preserve the intrinsic's identity before its callable methods arrive.
        // Missing intrinsic operations remain Unsupported at language boundaries.
        let prototype = if let Some(handle) = &self.object_prototype {
            handle.clone()
        } else {
            let handle = self.object_work(span, |objects, _| objects.create(None))?;
            self.object_prototype = Some(handle.clone());
            handle
        };
        let object = self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
        for property in properties {
            self.tick(property.span)?;
            let key = match &property.name {
                PropertyName::Literal(literal) => self.literal_value(literal, property.span)?,
                PropertyName::Computed(expression) => self.expression(expression)?,
            };
            // ToPropertyKey precedes evaluation of the property's value.
            let key = self.property_key(key, property.span)?;
            let value = self.expression(&property.value)?;
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

fn missing_object_method(key: &JsString) -> bool {
    [
        "constructor",
        "hasOwnProperty",
        "isPrototypeOf",
        "propertyIsEnumerable",
        "toLocaleString",
        "toString",
        "valueOf",
    ]
    .iter()
    .any(|name| key.code_units().iter().copied().eq(name.encode_utf16()))
}
