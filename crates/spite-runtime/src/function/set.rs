//! Set construction, intrinsics, and branded methods (24.2).

use super::Builtin;
use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{DataDescriptor, DescriptorKind, PropertyDescriptor, SetData},
};
use spite_core::{JsString, PropertyKey, Span, WellKnownSymbol};

mod combine;
mod iteration;

#[derive(Debug)]
pub(crate) struct SetIntrinsics {
    pub constructor: ObjectHandle,
    pub prototype: ObjectHandle,
    iterator_prototype: ObjectHandle,
    methods: Vec<ObjectHandle>,
}

impl SetIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [&self.constructor, &self.prototype, &self.iterator_prototype]
            .into_iter()
            .chain(&self.methods)
    }
}

impl Realm {
    pub(super) fn set_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        iterator_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<SetIntrinsics, Error> {
        let constructor = self.new_builtin(function_prototype, Builtin::Set, span)?;
        let prototype =
            self.object_work(span, |objects, _| objects.create(Some(object_prototype)))?;
        let iterator_prototype =
            self.object_work(span, |objects, _| objects.create(Some(iterator_prototype)))?;
        self.define_property_or_throw(
            &constructor,
            JsString::from("prototype"),
            DataDescriptor {
                value: Some(Value::Object(prototype.clone())),
                writable: Some(false),
                enumerable: Some(false),
                configurable: Some(false),
            }
            .into(),
            span,
        )?;
        self.define_builtin_property(
            &prototype,
            "constructor",
            Value::Object(constructor.clone()),
            true,
            span,
        )?;
        let mut methods = Vec::new();
        for builtin in [
            Builtin::SetAdd,
            Builtin::SetClear,
            Builtin::SetDelete,
            Builtin::SetEntries,
            Builtin::SetForEach,
            Builtin::SetHas,
            Builtin::SetValues,
            Builtin::SetDifference,
            Builtin::SetIntersection,
            Builtin::SetIsDisjointFrom,
            Builtin::SetIsSubsetOf,
            Builtin::SetIsSupersetOf,
            Builtin::SetSymmetricDifference,
            Builtin::SetUnion,
        ] {
            let method = self.new_builtin(function_prototype, builtin, span)?;
            self.define_builtin_property(
                &prototype,
                builtin.initial_name(),
                Value::Object(method.clone()),
                true,
                span,
            )?;
            if matches!(builtin, Builtin::SetValues) {
                for key in [
                    PropertyKey::from(JsString::from("keys")),
                    WellKnownSymbol::Iterator.symbol().into(),
                ] {
                    self.define_property_or_throw(
                        &prototype,
                        key,
                        DataDescriptor {
                            value: Some(Value::Object(method.clone())),
                            writable: Some(true),
                            enumerable: Some(false),
                            configurable: Some(true),
                        }
                        .into(),
                        span,
                    )?;
                }
            }
            methods.push(method);
        }
        for (object, key, builtin) in [
            (
                &prototype,
                PropertyKey::from(JsString::from("size")),
                Builtin::SetSize,
            ),
            (
                &constructor,
                WellKnownSymbol::Species.symbol().into(),
                Builtin::SetSpecies,
            ),
        ] {
            let getter = self.new_builtin(function_prototype, builtin, span)?;
            self.define_property_or_throw(
                object,
                key,
                PropertyDescriptor {
                    enumerable: Some(false),
                    configurable: Some(true),
                    kind: DescriptorKind::Accessor {
                        get: Some(Some(getter.clone())),
                        set: Some(None),
                    },
                },
                span,
            )?;
            methods.push(getter);
        }
        let next = self.new_builtin(function_prototype, Builtin::SetIteratorNext, span)?;
        self.define_builtin_property(
            &iterator_prototype,
            "next",
            Value::Object(next.clone()),
            true,
            span,
        )?;
        methods.push(next);
        for (object, name) in [(&prototype, "Set"), (&iterator_prototype, "Set Iterator")] {
            self.define_property_or_throw(
                object,
                WellKnownSymbol::ToStringTag.symbol(),
                DataDescriptor {
                    value: Some(Value::String(JsString::from(name))),
                    writable: Some(false),
                    enumerable: Some(false),
                    configurable: Some(true),
                }
                .into(),
                span,
            )?;
        }
        Ok(SetIntrinsics {
            constructor,
            prototype,
            iterator_prototype,
            methods,
        })
    }

    pub(super) fn set_constructor(
        &mut self,
        new_target: ObjectHandle,
        iterable: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let prototype = self.get_property(&new_target, &JsString::from("prototype"), span)?;
        let prototype = if let Value::Object(prototype) = prototype {
            prototype
        } else {
            self.intrinsics
                .as_ref()
                .expect("initialized")
                .set
                .prototype
                .clone()
        };
        let set = self.object_work(span, |objects, budget| {
            objects.create_set(&prototype, SetData::default(), budget)
        })?;
        if matches!(iterable, Value::Undefined | Value::Null) {
            return Ok(Value::Object(set));
        }
        let adder = self.get_property(&set, &JsString::from("add"), span)?;
        if !self.is_callable(&adder, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Set adder is not callable",
            ));
        }
        let method = self
            .get_method(&iterable, &WellKnownSymbol::Iterator.symbol(), span)?
            .ok_or_else(|| {
                Self::exception(ExceptionKind::TypeError, span, "Set input is not iterable")
            })?;
        let mut iterator = self.get_iterator_from_method(iterable, method, span)?;
        loop {
            self.tick(span)?;
            let Some(value) = self.iterator_step_value(&mut iterator, span)? else {
                return Ok(Value::Object(set));
            };
            if let Err(error) =
                self.call(adder.clone(), Value::Object(set.clone()), vec![value], span)
            {
                return Err(self.iterator_close_error(&iterator, error, span));
            }
        }
    }

    pub(super) fn set_receiver(
        &mut self,
        receiver: Value,
        span: Span,
    ) -> Result<ObjectHandle, Error> {
        if let Value::Object(object) = receiver {
            if self.object_work(span, |objects, _| Ok(objects.inspect(&object)?.is_set()))? {
                return Ok(object);
            }
        }
        Err(Self::exception(
            ExceptionKind::TypeError,
            span,
            "receiver has no SetData internal slot",
        ))
    }

    pub(super) fn set_method(
        &mut self,
        builtin: Builtin,
        receiver: Value,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let set = self.set_receiver(receiver, span)?;
        match builtin {
            Builtin::SetAdd => {
                self.object_work(span, |objects, budget| {
                    objects.set_insert(&set, value, budget)
                })?;
                Ok(Value::Object(set))
            }
            Builtin::SetHas => self
                .object_work(span, |objects, budget| {
                    objects.set_data(&set)?.has(&value, budget)
                })
                .map(Value::Boolean),
            Builtin::SetDelete => self
                .object_work(span, |objects, budget| {
                    objects.set_data_mut(&set)?.delete(&value, budget)
                })
                .map(Value::Boolean),
            Builtin::SetClear => {
                self.object_work(span, |objects, budget| {
                    objects.set_data_mut(&set)?.clear(budget)
                })?;
                Ok(Value::Undefined)
            }
            Builtin::SetSize => self
                .object_work(span, |objects, budget| {
                    budget.charge(1)?;
                    Ok(objects.set_data(&set)?.size())
                })
                .map(|size| Value::Number(size as f64)),
            _ => unreachable!("branded Set method"),
        }
    }
}
