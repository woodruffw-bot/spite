//! Realm ownership and collection outside active evaluation.

use crate::{
    Collection, Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{self, DataDescriptor, DescriptorKind, OrdinaryObject, Property, PropertyDescriptor},
};
use spite_bigint::BigInt;
use spite_core::{JsString, PropertyKey, PropertyKeyRef, Span, WellKnownSymbol};
use spite_parser::ast::{
    ExprKind, Literal, ObjectElement, ObjectProperty, PropertyKind, PropertyName,
};

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
    /// JavaScript getters and resets the work allowance if `max_steps` is enabled,
    /// except during a host job whose callbacks share one allowance.
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
        if !self.job_active {
            self.remaining_steps = self.limits.max_steps;
        }
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
        self.ordinary_to_primitive(object, hint, span)
    }

    // Date's @@toPrimitive must enter the ordinary algorithm directly rather
    // than redispatching its own hook through ToPrimitive (7.1.1.1, 21.4.4.45).
    pub(super) fn ordinary_to_primitive(
        &mut self,
        object: ObjectHandle,
        hint: Hint,
        span: Span,
    ) -> Result<Value, Error> {
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
            let (own, numeric_absence, proxy) = self.object_work(span, |objects, budget| {
                if objects.proxy_state(&handle, budget)?.is_some() {
                    return Ok((false, false, true));
                }
                let own = objects.has_own(&handle, key, budget)?;
                let numeric_absence = !own
                    && objects
                        .typed_array_numeric_index(&handle, key, budget)?
                        .is_some();
                Ok((own, numeric_absence, false))
            })?;
            if proxy {
                return self.proxy_has_property(&handle, crate::realm_proxy::owned_key(key), span);
            }
            if own || self.missing_intrinsic_property(&handle, key) {
                return Ok(true);
            }
            if numeric_absence {
                // Canonical numeric names are terminal at a TypedArray even
                // when invalid, detached, or outside its current view bounds.
                return Ok(false);
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
            if self
                .objects
                .inspect(&current)
                .map_err(Error::InvalidObject)?
                .is_module_namespace()
            {
                return Ok(false);
            };
            let action = self.object_work(span, |objects, budget| {
                if objects.proxy_state(&current, budget)?.is_some() {
                    return Ok(None);
                }
                objects
                    .typed_array_set_action(
                        &current,
                        &key,
                        if let Value::Object(receiver) = &receiver {
                            Some(receiver)
                        } else {
                            None
                        },
                        budget,
                    )
                    .map(Some)
            })?;
            let Some(action) = action else {
                return self.proxy_set_property(&current, key, value, receiver, span);
            };
            match action {
                object::TypedArraySetAction::Ordinary => {}
                object::TypedArraySetAction::Done(result) => return Ok(result),
                object::TypedArraySetAction::Element { index } => {
                    self.set_typed_array_element(&current, index, value, span)?;
                    return Ok(true);
                }
            }
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
            let result = self.object_work(span, |objects, budget| {
                if objects.proxy_state(object, budget)?.is_some() {
                    return Ok(None);
                }
                objects.delete(object, key, budget).map(Some)
            })?;
            return match result {
                Some(result) => Ok(result),
                None => {
                    self.proxy_delete_property(object, crate::realm_proxy::owned_key(key), span)
                }
            };
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
            if let Some(Property::Data(property)) =
                self.namespace_export_property(&handle, key, span)?
            {
                return Ok(property.value);
            };
            let (own, numeric_absence, proxy) = self.object_work(span, |objects, budget| {
                if objects.proxy_state(&handle, budget)?.is_some() {
                    return Ok((None, false, true));
                }
                let own = objects.get_own(&handle, key, budget)?;
                let numeric_absence = own.is_none()
                    && objects
                        .typed_array_numeric_index(&handle, key, budget)?
                        .is_some();
                Ok((own, numeric_absence, false))
            })?;
            if proxy {
                return self.proxy_get_property(
                    &handle,
                    crate::realm_proxy::owned_key(key),
                    receiver,
                    span,
                );
            }
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
            if numeric_absence {
                return Ok(Value::Undefined);
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

    // AddToKeptObjects (9.11). Only fixed-size Object/Symbol identities enter
    // this list; charge their copies and any list movement before allocation.
    pub(super) fn add_to_kept_objects(&mut self, target: &Value, span: Span) -> Result<(), Error> {
        debug_assert!(matches!(target, Value::Object(_) | Value::Symbol(_)));
        let len = self.kept_alive.len();
        let growing = len == self.kept_alive.capacity();
        self.object_work(span, |_, budget| {
            budget.charge(if growing {
                len.checked_add(1).ok_or(object::Error::WorkLimit)?
            } else {
                1
            })
        })?;
        if growing {
            self.kept_alive.try_reserve(1).map_err(|_| Error::Limit {
                span,
                message: "kept-alive list capacity exceeded".into(),
            })?;
        }
        // Reuse the established root registry. Its optional work checks happen
        // before registration; the new kept list already has room for append.
        let root = if let Value::Object(handle) = target {
            Some(self.object_work(span, |objects, budget| objects.root(handle, budget))?)
        } else {
            None
        };
        self.kept_alive.push(RootedValue {
            value: target.clone(),
            _root: root,
        });
        Ok(())
    }

    /// Ends the embedding's synchronous execution sequence (ClearKeptObjects, 9.10).
    ///
    /// Call this only after the whole sequence, including callbacks, has ended.
    /// A sequence may span several evaluations and host property reads. These
    /// operations and [`Self::collect`] never clear the kept-alive list themselves.
    ///
    /// This releases only WeakRef's temporary kept roots. Independent host roots
    /// remain live. It neither collects nor executes JavaScript, initializes the
    /// realm, or resets a work allowance. An active execution returns a host error
    /// without changing any roots. After any evaluation result, an inactive host
    /// may clear the list and then explicitly collect.
    ///
    /// Realms currently have independent heaps and execution domains. A host that
    /// groups several domains into one synchronous sequence must wait until the
    /// complete sequence finishes before calling their checkpoints.
    pub fn clear_kept_objects(&mut self) -> Result<(), Error> {
        if self.call_depth != 0 || self.evaluation_depth != 0 || self.job_active {
            return Err(Error::Host {
                span: Span::new(0, 0),
                message: "cannot clear kept objects during active execution".into(),
            });
        }
        // Detach before dropping. Root/Symbol destruction is outside every
        // object borrow and Symbol gate, including allocator reentry on drop.
        let kept = std::mem::take(&mut self.kept_alive);
        drop(kept);
        Ok(())
    }

    /// Borrows implemented object storage for host inspection.
    ///
    /// This is not a JavaScript reflection operation. Intrinsic methods that are
    /// not yet implemented do not have stored descriptors at this stage.
    pub fn inspect_object(&self, handle: &ObjectHandle) -> Result<&OrdinaryObject, object::Error> {
        self.objects.inspect(handle)
    }

    /// Collects between evaluations, retaining bindings, intrinsics, cached templates,
    /// queued Promise jobs, and host roots.
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
            .and_then(|count| count.checked_add(self.active_module.iter().count()))
            .and_then(|count| {
                count.checked_add(
                    self.intrinsics
                        .as_ref()
                        .map_or(0, |intrinsics| intrinsics.roots().count()),
                )
            })
            .and_then(|count| count.checked_add(self.template_map.len()))
            .and_then(|count| {
                self.promise_jobs
                    .iter()
                    .try_fold(count, |count, job| count.checked_add(job.roots().count()))
            })
            .ok_or(spite_heap::Error::Limit)?;
        let remaining = max_work
            .checked_sub(scanned)
            .ok_or(spite_heap::Error::Limit)?;
        let roots = self
            .global_object
            .iter()
            .chain(self.active_module.iter())
            .chain(self.scopes.iter().map(|scope| &scope.0))
            .chain(
                self.intrinsics
                    .iter()
                    .flat_map(|intrinsics| intrinsics.roots()),
            );
        let roots = roots
            .chain(self.template_map.iter().map(|(_, array)| array))
            .chain(self.promise_jobs.iter().flat_map(|job| job.roots()));
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
        let mut budget =
            object::Budget::with_limits(self.limits.max_bigint_bits, self.remaining_steps);
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
            object::Error::RequiresRuntime => {
                Self::unsupported(span, "object internal method requires Realm dispatch")
            }
            object::Error::PoisonedSharedStorage => Error::Host {
                span,
                message: "shared backing storage lock is poisoned".into(),
            },
            object::Error::NotCallable | object::Error::WrongKind => {
                unreachable!("realm operations validate function callability before storage")
            }
            object::Error::UnnormalizedArrayLength => {
                unreachable!("realm converts array length values before storage")
            }
            object::Error::UnnormalizedTypedArrayElement => {
                unreachable!("realm converts typed element values before storage")
            }
            object::Error::Integer(error) => Self::integer_error(error, span),
            object::Error::Heap(
                spite_heap::Error::ForeignHandle
                | spite_heap::Error::StaleHandle
                | spite_heap::Error::CollectionState,
            ) => {
                // CollectionState can only originate in explicit collection,
                // never in ordinary evaluator object_work operations.
                unreachable!("object work retains valid handles and never creates collection state")
            }
        })
    }

    pub(super) fn literal_value(&mut self, literal: &Literal, span: Span) -> Result<Value, Error> {
        Ok(match literal {
            Literal::Null => Value::Null,
            Literal::Boolean(value) => Value::Boolean(*value),
            Literal::Number(value) => Value::Number(*value),
            Literal::String(value) => Value::String(value.clone()),
            // 13.2.7: each evaluation calls intrinsic RegExpCreate with the
            // literal's Strings, independent of public RegExp bindings/hooks.
            Literal::RegExp { body, flags } => self.regexp_create(
                Value::String(body.clone()),
                Value::String(flags.clone()),
                span,
            )?,
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

    pub(crate) fn define_method_property(
        &mut self,
        object: &ObjectHandle,
        key: PropertyKey,
        property: &ObjectProperty,
        enumerable: bool,
    ) -> Result<(), Error> {
        let ExprKind::Function(syntax) = &property.value.kind else {
            unreachable!("method syntax");
        };
        let function =
            self.method_function(syntax, object, key.clone(), property.kind, property.span)?;
        let descriptor = match property.kind {
            PropertyKind::Method => DataDescriptor {
                value: Some(Value::Object(function)),
                writable: Some(true),
                enumerable: Some(enumerable),
                configurable: Some(true),
            }
            .into(),
            PropertyKind::Getter => PropertyDescriptor {
                kind: DescriptorKind::Accessor {
                    get: Some(Some(function)),
                    set: None,
                },
                enumerable: Some(enumerable),
                configurable: Some(true),
            },
            PropertyKind::Setter => PropertyDescriptor {
                kind: DescriptorKind::Accessor {
                    get: None,
                    set: Some(Some(function)),
                },
                enumerable: Some(enumerable),
                configurable: Some(true),
            },
            _ => unreachable!("method property kind"),
        };
        self.define_property_or_throw(object, key, descriptor, property.span)
    }

    pub(crate) fn new_literal_object(&mut self, span: Span) -> Result<ObjectHandle, Error> {
        let prototype = self.ensure_object_intrinsics(span)?;
        let object = self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
        Ok(object)
    }

    pub(crate) fn literal_property_key(
        &mut self,
        property: &ObjectProperty,
    ) -> Result<PropertyKey, Error> {
        let key = match &property.name {
            PropertyName::Private(_) => {
                return Err(Self::unsupported(
                    property.span,
                    "private elements are not implemented",
                ));
            }
            PropertyName::Literal(literal) => self.literal_value(literal, property.span)?,
            PropertyName::Computed(expression) => self.expression(expression)?,
        };
        // ToPropertyKey precedes evaluation of the property's value.
        self.property_key(key, property.span)
    }

    pub(crate) fn define_literal_data_property(
        &mut self,
        object: &ObjectHandle,
        key: PropertyKey,
        property: &ObjectProperty,
        value: Value,
    ) -> Result<(), Error> {
        if property.kind == PropertyKind::Prototype {
            let prototype = match &value {
                Value::Object(handle) => Some(handle),
                Value::Null => None,
                _ => return Ok(()),
            };
            let changed = self.object_work(property.span, |objects, budget| {
                objects.set_prototype(object, prototype, budget)
            })?;
            debug_assert!(
                changed,
                "new literal is extensible and cannot be its own ancestor"
            );
        } else {
            let created = self.object_work(property.span, |objects, budget| {
                objects.define(
                    object,
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
        Ok(())
    }

    pub(super) fn object_literal(
        &mut self,
        properties: &[ObjectElement],
        span: Span,
    ) -> Result<Value, Error> {
        let object = self.new_literal_object(span)?;
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
            let key = self.literal_property_key(property)?;
            if matches!(
                property.kind,
                PropertyKind::Method | PropertyKind::Getter | PropertyKind::Setter
            ) {
                self.define_method_property(&object, key, property, true)?;
                continue;
            }
            let value = if property.kind == PropertyKind::Prototype {
                self.expression(&property.value)?
            } else {
                self.named_expression(&property.value, key.clone())?
            };
            self.define_literal_data_property(&object, key, property, value)?;
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
        self.length_from_value(length, span)
    }

    pub(crate) fn length_from_value(&mut self, value: Value, span: Span) -> Result<u64, Error> {
        let number = self.number(value, span)?;
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
        let result = self.object_work(span, |objects, budget| {
            if objects.proxy_state(object, budget)?.is_some() {
                return Ok(None);
            }
            objects.own_keys(object, budget).map(Some)
        })?;
        match result {
            Some(result) => Ok(result),
            None => self.proxy_own_property_keys(object, span),
        }
    }

    pub(crate) fn own_property_descriptor<'key>(
        &mut self,
        object: &ObjectHandle,
        key: impl Into<PropertyKeyRef<'key>>,
        span: Span,
    ) -> Result<Option<Property>, Error> {
        let key = key.into();
        if let Some(property) = self.namespace_export_property(object, key, span)? {
            return Ok(Some(property));
        };
        let property = self.object_work(span, |objects, budget| {
            if objects.proxy_state(object, budget)?.is_some() {
                return Ok(None);
            }
            objects.get_own(object, key, budget).map(Some)
        })?;
        let Some(property) = property else {
            return self.proxy_own_property_descriptor(
                object,
                crate::realm_proxy::owned_key(key),
                span,
            );
        };
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
        false
    }
}

fn key_is<'key>(key: impl Into<PropertyKeyRef<'key>>, name: &str) -> bool {
    key.into()
        .as_string()
        .is_some_and(|key| key.code_units().iter().copied().eq(name.encode_utf16()))
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
    // At most 15 decimal digits give an exact integer below 10^15 < 2^53,
    // with fixed decimal Number::toString spelling below 10^21. Wider keys
    // retain the rounding/canonical-spelling check.
    if index >= string.len()
        || units.len() > 15 && !key_is(key, &crate::value::number_to_string(index as f64))
    {
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

    #[test]
    fn canonical_small_string_indices_preserve_utf16_units_and_bounds() {
        // A paired scalar and separate lone high/low surrogates stay code units.
        let text =
            JsString::from_code_units(vec![0x41, 0xd83d, 0xde00, 0xd800, 0x58, 0xdc00, 0x5a]);
        for (key, index, unit) in [
            ("0", 0, 0x41),
            ("1", 1, 0xd83d),
            ("2", 2, 0xde00),
            ("3", 3, 0xd800),
            ("4", 4, 0x58),
            ("5", 5, 0xdc00),
            ("6", 6, 0x5a),
        ] {
            let found = string_index(&text, &JsString::from(key));
            assert_eq!(found, Some(index), "{key}");
            assert_eq!(found.map(|at| text.code_units()[at]), Some(unit), "{key}");
        }
        let hex = JsString::from("0123456789ABCDEF");
        for (key, expected) in [
            ("0", Some(0)),
            ("9", Some(9)),
            ("10", Some(10)),
            ("15", Some(15)),
            ("16", None),
        ] {
            assert_eq!(string_index(&hex, &JsString::from(key)), expected, "{key}");
        }
        for key in [
            "",
            "00",
            "01",
            "-0",
            "-1",
            "+1",
            "1.0",
            "1e0",
            "0x1",
            "1.5",
            " 1",
            "1 ",
            "NaN",
            "Infinity",
            "١",
            "１",
            "7",
            "8",
            // These long keys are out of range on a real small string. They do
            // not require enormous storage or assume a 64-bit usize.
            "999999999999998",
            "999999999999999",
            "1000000000000000",
            "9007199254740991",
            "9007199254740992",
            "9007199254740993",
            "9007199254740994",
            "18446744073709551615",
            "18446744073709551616",
            "99999999999999999999",
            "100000000000000000000",
        ] {
            assert_eq!(string_index(&text, &JsString::from(key)), None, "{key}");
        }
        assert_eq!(
            string_index(&JsString::from(""), &JsString::from("0")),
            None
        );
        let symbol = spite_core::JsSymbol::new(Some(JsString::from("0")));
        assert_eq!(string_index(&text, &symbol), None);
    }

    #[test]
    fn string_index_fast_acceptance_keeps_coercion_boxing_and_prototype_fallback() {
        let mut realm = Realm::default();
        assert_eq!(realm.eval(r#"
            let calls='',key={toString(){calls+='k';return '1'}};
            let s='A😀\uD800X\uDC00Z',box=new String(s),a=s[key],b=box[key];
            String.prototype['0']='shadow';String.prototype['01']='leading';
            String.prototype['999999999999999']='fifteen';
            String.prototype['1000000000000000']='sixteen';
            String.prototype['9007199254740992']='even';
            String.prototype['9007199254740993']='rounded-neighbor';
            String.prototype['18446744073709551616']='overflow';
            calls==='kk'&&a==='\uD83D'&&b==='\uD83D'&&s.length===7&&box.length===7
              &&s[0]==='A'&&box[0]==='A'&&s[2]==='\uDE00'&&box[3]==='\uD800'
              &&s[5]==='\uDC00'&&s[7]===undefined&&box[7]===undefined
              &&s['01']==='leading'&&box['01']==='leading'
              &&s['999999999999999']==='fifteen'&&box['999999999999999']==='fifteen'
              &&s['1000000000000000']==='sixteen'&&box['1000000000000000']==='sixteen'
              &&s['9007199254740992']==='even'&&box['9007199254740992']==='even'
              &&s['9007199254740993']==='rounded-neighbor'&&box['9007199254740993']==='rounded-neighbor'
              &&s['18446744073709551616']==='overflow'&&box['18446744073709551616']==='overflow'
        "#), Ok(Value::Boolean(true)));
    }
}
