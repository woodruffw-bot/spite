//! Reflect calls, construction, prototypes, and extensibility (28.1).

use super::Builtin;
use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value, object::DataDescriptor};
use spite_core::{JsString, PropertyKey, Span, WellKnownSymbol};

#[derive(Debug)]
pub(crate) struct ReflectIntrinsics {
    pub object: ObjectHandle,
    methods: Vec<ObjectHandle>,
}

impl ReflectIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        std::iter::once(&self.object).chain(self.methods.iter())
    }
}

impl Realm {
    pub(super) fn reflect_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<ReflectIntrinsics, Error> {
        let object = self.object_work(span, |objects, _| objects.create(Some(object_prototype)))?;
        self.object_work(span, |objects, budget| {
            objects.define(
                &object,
                WellKnownSymbol::ToStringTag.symbol(),
                DataDescriptor {
                    value: Some(Value::String(JsString::from("Reflect"))),
                    writable: Some(false),
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                budget,
            )
        })?;
        let mut methods = Vec::new();
        for builtin in [
            Builtin::ReflectApply,
            Builtin::ReflectConstruct,
            Builtin::ReflectDefineProperty,
            Builtin::ReflectDeleteProperty,
            Builtin::ReflectGet,
            Builtin::ReflectGetOwnPropertyDescriptor,
            Builtin::ReflectGetPrototypeOf,
            Builtin::ReflectHas,
            Builtin::ReflectIsExtensible,
            Builtin::ReflectOwnKeys,
            Builtin::ReflectPreventExtensions,
            Builtin::ReflectSet,
            Builtin::ReflectSetPrototypeOf,
        ] {
            let method = self.new_builtin(function_prototype, builtin, span)?;
            self.define_builtin_property(
                &object,
                builtin.initial_name(),
                Value::Object(method.clone()),
                true,
                span,
            )?;
            methods.push(method);
        }
        Ok(ReflectIntrinsics { object, methods })
    }

    pub(super) fn reflect_construct(
        &mut self,
        target: Value,
        list: Value,
        new_target: Option<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        // Validate constructors before reading the argument list. Explicit
        // undefined differs from an absent third argument (28.1.2).
        if !self.is_constructor(&target, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Reflect.construct requires a constructor target",
            ));
        }
        let new_target = if let Some(value) = new_target {
            if !self.is_constructor(&value, span)? {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "Reflect.construct requires a constructor newTarget",
                ));
            }
            let Value::Object(object) = value else {
                unreachable!("constructor object")
            };
            Some(object)
        } else {
            None
        };
        let arguments = self.argument_list_from_array_like(list, span)?;
        self.construct_with_new_target(target, arguments, new_target, span)
    }

    pub(super) fn reflect_object(target: Value, span: Span) -> Result<ObjectHandle, Error> {
        if let Value::Object(object) = target {
            Ok(object)
        } else {
            Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Reflect target must be an object",
            ))
        }
    }

    pub(super) fn reflect_property(
        &mut self,
        builtin: Builtin,
        target: Value,
        key: Value,
        receiver: Option<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        // 28.1.4/5/8: reject primitive targets before ToPropertyKey. get uses
        // target only when receiver is absent, preserving explicit undefined.
        let target = Self::reflect_object(target, span)?;
        let key = self.property_key(key, span)?;
        match builtin {
            Builtin::ReflectGet => self.get_property_with_receiver(
                &target,
                &key,
                receiver.unwrap_or_else(|| Value::Object(target.clone())),
                span,
            ),
            Builtin::ReflectHas => self.has_property(&target, &key, span).map(Value::Boolean),
            Builtin::ReflectDeleteProperty => self
                .delete_property_value(&Value::Object(target), &key, span)
                .map(Value::Boolean),
            _ => unreachable!("Reflect property operation"),
        }
    }

    pub(super) fn reflect_define_property(
        &mut self,
        target: Value,
        key: Value,
        attributes: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // 28.1.3: validate target, then convert key and attributes in order.
        // Descriptor rejection is false; conversion failures still throw.
        let target = Self::reflect_object(target, span)?;
        let key = self.property_key(key, span)?;
        let descriptor = self.property_descriptor(attributes, span)?;
        self.define_property(&target, key, descriptor, span)
            .map(Value::Boolean)
    }

    pub(super) fn reflect_set(
        &mut self,
        target: Value,
        key: Value,
        value: Value,
        receiver: Option<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        // 28.1.12 preserves an explicitly supplied primitive receiver and
        // returns [[Set]]'s boolean without strict-mode rejection handling.
        let target = Self::reflect_object(target, span)?;
        let key = self.property_key(key, span)?;
        let receiver = receiver.unwrap_or_else(|| Value::Object(target.clone()));
        self.set_property_with_receiver(&target, key, value, receiver, span)
            .map(Value::Boolean)
    }

    pub(super) fn reflect_own_keys(&mut self, target: Value, span: Span) -> Result<Value, Error> {
        let target = Self::reflect_object(target, span)?;
        let keys = self.own_property_keys(&target, span)?;
        // 28.1.10 / CreateArrayFromList: preserve String/Symbol key order and
        // include non-enumerable keys without reading any property values.
        self.create_array_from_list(
            keys.into_iter().map(|key| match key {
                PropertyKey::String(key) => Value::String(key),
                PropertyKey::Symbol(key) => Value::Symbol(key),
            }),
            span,
        )
    }

    pub(super) fn reflect_set_prototype_of(
        &mut self,
        target: Value,
        prototype: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // 28.1.13 validates target before proto and returns [[SetPrototypeOf]]'s
        // boolean, including false for cycles or a non-extensible change.
        let target = Self::reflect_object(target, span)?;
        let prototype = match prototype {
            Value::Null => None,
            Value::Object(object) => Some(object),
            _ => {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "prototype must be an object or null",
                ));
            }
        };
        self.object_work(span, |objects, budget| {
            objects.set_prototype(&target, prototype.as_ref(), budget)
        })
        .map(Value::Boolean)
    }
}
