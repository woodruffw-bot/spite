//! Object construction and mandatory prototype methods (20.1.1, 20.1.3).

use super::Builtin;
use crate::{Error, ObjectHandle, Realm, Value, object::DataDescriptor};
use spite_core::{JsString, Span};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub(crate) struct ObjectIntrinsics {
    pub constructor: ObjectHandle,
    methods: [ObjectHandle; 4],
}

impl ObjectIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        std::iter::once(&self.constructor).chain(self.methods.iter())
    }
}

impl Realm {
    pub(super) fn object_constructor_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<ObjectIntrinsics, Error> {
        let constructor = self.new_builtin(function_prototype, Builtin::Object, span)?;
        self.object_work(span, |objects, budget| {
            objects.define(
                &constructor,
                JsString::from("prototype"),
                DataDescriptor {
                    value: Some(Value::Object(object_prototype.clone())),
                    writable: Some(false),
                    enumerable: Some(false),
                    configurable: Some(false),
                },
                budget,
            )
        })?;
        self.define_builtin_property(
            object_prototype,
            "constructor",
            Value::Object(constructor.clone()),
            true,
            span,
        )?;
        let has_own = self.new_builtin(function_prototype, Builtin::ObjectHasOwnProperty, span)?;
        let enumerable = self.new_builtin(
            function_prototype,
            Builtin::ObjectPropertyIsEnumerable,
            span,
        )?;
        let is_prototype =
            self.new_builtin(function_prototype, Builtin::ObjectIsPrototypeOf, span)?;
        let to_locale =
            self.new_builtin(function_prototype, Builtin::ObjectToLocaleString, span)?;
        for (name, value) in [
            ("hasOwnProperty", &has_own),
            ("propertyIsEnumerable", &enumerable),
            ("isPrototypeOf", &is_prototype),
            ("toLocaleString", &to_locale),
        ] {
            self.define_builtin_property(
                object_prototype,
                name,
                Value::Object(value.clone()),
                true,
                span,
            )?;
        }
        Ok(ObjectIntrinsics {
            constructor,
            methods: [has_own, enumerable, is_prototype, to_locale],
        })
    }

    pub(super) fn object_constructor(
        &mut self,
        new_target: Option<ObjectHandle>,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let intrinsics = self.intrinsics.as_ref().expect("initialized");
        let default_prototype = intrinsics.object_prototype.clone();
        if let Some(target) = new_target.filter(|target| target != &intrinsics.object.constructor) {
            let prototype = self.get_property(&target, &JsString::from("prototype"), span)?;
            let prototype = match prototype {
                Value::Object(prototype) => prototype,
                _ => default_prototype,
            };
            return self
                .object_work(span, |objects, _| objects.create(Some(&prototype)))
                .map(Value::Object);
        }
        if matches!(value, Value::Undefined | Value::Null) {
            return self
                .object_work(span, |objects, _| objects.create(Some(&default_prototype)))
                .map(Value::Object);
        }
        self.box_primitive(value, span)
    }

    pub(super) fn object_property_predicate(
        &mut self,
        this: Value,
        key: Value,
        enumerable: bool,
        span: Span,
    ) -> Result<Value, Error> {
        // 20.1.3.2 / 20.1.3.4 deliberately convert the key before ToObject(this).
        let key = self.property_key(key, span)?;
        let Value::Object(object) = self.box_primitive(this, span)? else {
            unreachable!("ToObject");
        };
        let property = self.object_work(span, |objects, budget| {
            objects.get_own(&object, &key, budget)
        })?;
        if property.is_none() && self.missing_intrinsic_property(&object, &key) {
            return Err(Self::unsupported(
                span,
                "intrinsic property descriptor is not implemented",
            ));
        }
        Ok(Value::Boolean(property.is_some_and(|property| {
            !enumerable || property.enumerable()
        })))
    }

    pub(super) fn object_is_prototype_of(
        &mut self,
        this: Value,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // 20.1.3.3: non-object arguments return false even for a nullish receiver.
        let Value::Object(mut candidate) = value else {
            return Ok(Value::Boolean(false));
        };
        let Value::Object(prototype) = self.box_primitive(this, span)? else {
            unreachable!("ToObject");
        };
        loop {
            let next = self.object_work(span, |objects, _| {
                Ok(objects.inspect(&candidate)?.prototype().cloned())
            })?;
            let Some(next) = next else {
                return Ok(Value::Boolean(false));
            };
            if next == prototype {
                return Ok(Value::Boolean(true));
            }
            candidate = next;
        }
    }

    pub(super) fn object_to_locale_string(
        &mut self,
        this: Value,
        span: Span,
    ) -> Result<Value, Error> {
        Self::require_object_coercible(&this, span)?;
        let method = self.get_property_value(&this, &JsString::from("toString"), span)?;
        self.call(method, this, Vec::new(), span)
    }
}
