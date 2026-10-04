//! Object descriptor reflection and conversion (6.2.6.4–5, 20.1.2.4/8/14).

use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{DataDescriptor, DescriptorKind, Property, PropertyDescriptor},
};
use spite_core::{JsString, PropertyKey, Span};

impl Realm {
    pub(crate) fn object_define_property(
        &mut self,
        target: Value,
        key: Value,
        attributes: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // Unlike getOwnPropertyDescriptor, defineProperty never boxes its target.
        let Value::Object(object) = target else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Object.defineProperty requires an object",
            ));
        };
        let key = self.property_key(key, span)?;
        let descriptor = self.property_descriptor(attributes, span)?;
        self.define_property_or_throw(&object, key, descriptor, span)?;
        Ok(Value::Object(object))
    }

    pub(crate) fn define_property_or_throw(
        &mut self,
        object: &ObjectHandle,
        key: impl Into<PropertyKey>,
        descriptor: PropertyDescriptor,
        span: Span,
    ) -> Result<(), Error> {
        let defined = self.define_property(object, key, descriptor, span)?;
        if !defined {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "property descriptor was rejected",
            ));
        }
        Ok(())
    }

    pub(crate) fn define_property(
        &mut self,
        object: &ObjectHandle,
        key: impl Into<PropertyKey>,
        mut descriptor: PropertyDescriptor,
        span: Span,
    ) -> Result<bool, Error> {
        let key = key.into();
        self.check_missing_intrinsic_mutation(object, &key, span)?;
        self.convert_array_length(object, &key, &mut descriptor, span)?;
        self.object_work(span, |objects, budget| {
            objects.define(object, key, descriptor, budget)
        })
    }

    pub(crate) fn object_create(
        &mut self,
        prototype: Value,
        properties: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let prototype = match prototype {
            Value::Object(object) => Some(object),
            Value::Null => None,
            _ => {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "Object.create requires an object or null prototype",
                ));
            }
        };
        let object = self.object_work(span, |objects, _| objects.create(prototype.as_ref()))?;
        if matches!(properties, Value::Undefined) {
            return Ok(Value::Object(object));
        }
        self.object_define_properties(Value::Object(object), properties, span)
    }

    pub(crate) fn object_define_properties(
        &mut self,
        target: Value,
        properties: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(object) = target else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Object.defineProperties requires an object",
            ));
        };
        let Value::Object(properties) = self.box_primitive(properties, span)? else {
            unreachable!("ToObject");
        };
        // 20.1.2.3.1: snapshot keys, then convert every still-enumerable own
        // descriptor before applying any definitions. Getters may mutate props.
        let keys = self.own_property_keys(&properties, span)?;
        let mut descriptors = Vec::new();
        for key in keys {
            if self
                .own_property_descriptor(&properties, &key, span)?
                .is_some_and(|property| property.enumerable())
            {
                let attributes = self.get_property(&properties, &key, span)?;
                let descriptor = self.property_descriptor(attributes, span)?;
                descriptors.push((key, descriptor));
            }
        }
        // Rejection here preserves earlier successful definitions; it is not a
        // transaction. A conversion failure above performs no definitions.
        for (key, descriptor) in descriptors {
            self.define_property_or_throw(&object, key, descriptor, span)?;
        }
        Ok(Value::Object(object))
    }

    pub(crate) fn object_own_property(
        &mut self,
        target: Value,
        key: Value,
        presence_only: bool,
        span: Span,
    ) -> Result<Value, Error> {
        // Both static algorithms convert the target before the property key.
        let Value::Object(object) = self.box_primitive(target, span)? else {
            unreachable!("ToObject");
        };
        let key = self.property_key(key, span)?;
        let property = self.own_property_descriptor(&object, &key, span)?;
        if presence_only {
            return Ok(Value::Boolean(property.is_some()));
        }
        self.descriptor_object(property, span)
    }

    pub(in crate::function) fn property_descriptor(
        &mut self,
        attributes: Value,
        span: Span,
    ) -> Result<PropertyDescriptor, Error> {
        let Value::Object(object) = attributes else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "property attributes must be an object",
            ));
        };
        // ToPropertyDescriptor observes inherited fields and performs each
        // HasProperty/Get pair in order. Validate a getter before reading set;
        // only reject mixed data/accessor descriptors after both accessor reads.
        let enumerable = self
            .descriptor_field(&object, "enumerable", span)?
            .map(|value| value.to_boolean());
        let configurable = self
            .descriptor_field(&object, "configurable", span)?
            .map(|value| value.to_boolean());
        let value = self.descriptor_field(&object, "value", span)?;
        let writable = self
            .descriptor_field(&object, "writable", span)?
            .map(|value| value.to_boolean());
        let get = self.descriptor_accessor(&object, "get", span)?;
        let set = self.descriptor_accessor(&object, "set", span)?;
        let kind = if get.is_some() || set.is_some() {
            if value.is_some() || writable.is_some() {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "property descriptor mixes data and accessors",
                ));
            }
            DescriptorKind::Accessor { get, set }
        } else {
            DescriptorKind::Data { value, writable }
        };
        Ok(PropertyDescriptor {
            kind,
            enumerable,
            configurable,
        })
    }

    fn descriptor_field(
        &mut self,
        object: &ObjectHandle,
        name: &str,
        span: Span,
    ) -> Result<Option<Value>, Error> {
        let key = JsString::from(name);
        if self.has_property(object, &key, span)? {
            self.get_property(object, &key, span).map(Some)
        } else {
            Ok(None)
        }
    }

    fn descriptor_accessor(
        &mut self,
        object: &ObjectHandle,
        name: &str,
        span: Span,
    ) -> Result<Option<Option<ObjectHandle>>, Error> {
        let Some(value) = self.descriptor_field(object, name, span)? else {
            return Ok(None);
        };
        if matches!(value, Value::Undefined) {
            return Ok(Some(None));
        }
        if self.is_callable(&value, span)? {
            let Value::Object(object) = value else {
                unreachable!("callable object");
            };
            return Ok(Some(Some(object)));
        }
        Err(Self::exception(
            ExceptionKind::TypeError,
            span,
            "property accessor must be callable or undefined",
        ))
    }

    pub(super) fn descriptor_object(
        &mut self,
        property: Option<Property>,
        span: Span,
    ) -> Result<Value, Error> {
        let Some(property) = property else {
            return Ok(Value::Undefined);
        };
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .object_prototype
            .clone();
        let enumerable = property.enumerable();
        let configurable = property.configurable();
        let fields = match property {
            Property::Data(data) => [
                ("value", data.value),
                ("writable", Value::Boolean(data.writable)),
            ],
            Property::Accessor(accessor) => [
                ("get", accessor.get.map_or(Value::Undefined, Value::Object)),
                ("set", accessor.set.map_or(Value::Undefined, Value::Object)),
            ],
        };
        self.object_work(span, |objects, budget| {
            let object = objects.create(Some(&prototype))?;
            for (name, value) in fields.into_iter().chain([
                ("enumerable", Value::Boolean(enumerable)),
                ("configurable", Value::Boolean(configurable)),
            ]) {
                let defined = objects.define(
                    &object,
                    JsString::from(name),
                    DataDescriptor {
                        value: Some(value),
                        writable: Some(true),
                        enumerable: Some(true),
                        configurable: Some(true),
                    },
                    budget,
                )?;
                debug_assert!(defined, "fresh descriptor object");
            }
            Ok(Value::Object(object))
        })
    }
}
