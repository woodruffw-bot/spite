//! Object descriptor reflection and conversion (6.2.6.4–5, 20.1.2.4/8/14).

use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{DataDescriptor, DescriptorKind, Property, PropertyDescriptor},
};
use spite_core::{JsString, Span};

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
        self.check_missing_intrinsic_mutation(&object, &key, span)?;
        let defined = self.object_work(span, |objects, budget| {
            objects.define(&object, key, descriptor, budget)
        })?;
        if !defined {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "property descriptor was rejected",
            ));
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

    fn property_descriptor(
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

    fn descriptor_object(
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
