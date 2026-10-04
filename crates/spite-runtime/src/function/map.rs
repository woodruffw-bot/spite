//! Map construction, intrinsics, and keyed methods (24.1).

use super::Builtin;
use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{DataDescriptor, DescriptorKind, PropertyDescriptor},
};
use spite_core::{JsString, PropertyKey, Span, WellKnownSymbol};

mod group_by;
mod iteration;

#[derive(Debug)]
pub(crate) struct MapIntrinsics {
    pub constructor: ObjectHandle,
    pub prototype: ObjectHandle,
    iterator_prototype: ObjectHandle,
    methods: Vec<ObjectHandle>,
}

impl MapIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [&self.constructor, &self.prototype, &self.iterator_prototype]
            .into_iter()
            .chain(&self.methods)
    }
}

impl Realm {
    pub(super) fn map_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        iterator_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<MapIntrinsics, Error> {
        let constructor = self.new_builtin(function_prototype, Builtin::Map, span)?;
        // Map.prototype is an ordinary object without [[MapData]] (24.1.3).
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
            Builtin::MapClear,
            Builtin::MapDelete,
            Builtin::MapEntries,
            Builtin::MapForEach,
            Builtin::MapGet,
            Builtin::MapGetOrInsert,
            Builtin::MapGetOrInsertComputed,
            Builtin::MapHas,
            Builtin::MapKeys,
            Builtin::MapSet,
            Builtin::MapValues,
        ] {
            let method = self.new_builtin(function_prototype, builtin, span)?;
            self.define_builtin_property(
                &prototype,
                builtin.initial_name(),
                Value::Object(method.clone()),
                true,
                span,
            )?;
            if matches!(builtin, Builtin::MapEntries) {
                self.define_property_or_throw(
                    &prototype,
                    WellKnownSymbol::Iterator.symbol(),
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
            methods.push(method);
        }
        for (object, key, builtin) in [
            (
                &prototype,
                PropertyKey::from(JsString::from("size")),
                Builtin::MapSize,
            ),
            (
                &constructor,
                WellKnownSymbol::Species.symbol().into(),
                Builtin::MapSpecies,
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
        for (object, builtin) in [
            (&constructor, Builtin::MapGroupBy),
            (&iterator_prototype, Builtin::MapIteratorNext),
        ] {
            let method = self.new_builtin(function_prototype, builtin, span)?;
            self.define_builtin_property(
                object,
                builtin.initial_name(),
                Value::Object(method.clone()),
                true,
                span,
            )?;
            methods.push(method);
        }
        for (object, name) in [(&prototype, "Map"), (&iterator_prototype, "Map Iterator")] {
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
        Ok(MapIntrinsics {
            constructor,
            prototype,
            iterator_prototype,
            methods,
        })
    }

    pub(super) fn map_constructor(
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
                .map
                .prototype
                .clone()
        };
        let map = self.object_work(span, |objects, _| objects.create_map(&prototype))?;
        if matches!(iterable, Value::Undefined | Value::Null) {
            return Ok(Value::Object(map));
        }
        let adder = self.get_property(&map, &JsString::from("set"), span)?;
        if !self.is_callable(&adder, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Map adder is not callable",
            ));
        }
        let method = self
            .get_method(&iterable, &WellKnownSymbol::Iterator.symbol(), span)?
            .ok_or_else(|| {
                Self::exception(ExceptionKind::TypeError, span, "Map input is not iterable")
            })?;
        let mut iterator = self.get_iterator_from_method(iterable, method, span)?;
        loop {
            self.tick(span)?;
            // AddEntriesFromIterable does not close failures in stepping, done,
            // or value reads. Entry validation, reads, and adder calls do close.
            let Some(entry) = self.iterator_step_value(&mut iterator, span)? else {
                return Ok(Value::Object(map));
            };
            let add = (|| {
                let Value::Object(entry) = entry else {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "Map entry is not an object",
                    ));
                };
                let key = self.get_property(&entry, &JsString::from("0"), span)?;
                let value = self.get_property(&entry, &JsString::from("1"), span)?;
                self.call(
                    adder.clone(),
                    Value::Object(map.clone()),
                    vec![key, value],
                    span,
                )
            })();
            if let Err(error) = add {
                return Err(self.iterator_close_error(&iterator, error, span));
            }
        }
    }

    pub(super) fn map_receiver(&mut self, value: Value, span: Span) -> Result<ObjectHandle, Error> {
        if let Value::Object(object) = value {
            if self.object_work(span, |objects, _| Ok(objects.inspect(&object)?.is_map()))? {
                return Ok(object);
            }
        }
        Err(Self::exception(
            ExceptionKind::TypeError,
            span,
            "receiver has no MapData internal slot",
        ))
    }

    pub(super) fn map_method(
        &mut self,
        builtin: Builtin,
        receiver: Value,
        key: Value,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let map = self.map_receiver(receiver, span)?;
        match builtin {
            Builtin::MapClear => {
                self.object_work(span, |objects, budget| objects.map_clear(&map, budget))?;
                Ok(Value::Undefined)
            }
            Builtin::MapSize => self
                .object_work(span, |objects, budget| objects.map_size(&map, budget))
                .map(|size| Value::Number(size as f64)),
            Builtin::MapHas => self
                .object_work(span, |objects, budget| objects.map_has(&map, &key, budget))
                .map(Value::Boolean),
            Builtin::MapDelete => self
                .object_work(span, |objects, budget| {
                    objects.map_delete(&map, &key, budget)
                })
                .map(Value::Boolean),
            Builtin::MapGet => self
                .object_work(span, |objects, budget| objects.map_get(&map, &key, budget))
                .map(|value| value.unwrap_or(Value::Undefined)),
            Builtin::MapSet => {
                self.object_work(span, |objects, budget| {
                    objects.map_set(&map, key, value, budget)
                })?;
                Ok(Value::Object(map))
            }
            Builtin::MapGetOrInsert | Builtin::MapGetOrInsertComputed => {
                let computed = matches!(builtin, Builtin::MapGetOrInsertComputed);
                // Callback validation precedes lookup, even for an existing key.
                if computed && !self.is_callable(&value, span)? {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "Map callback is not callable",
                    ));
                }
                if let Some(value) =
                    self.object_work(span, |objects, budget| objects.map_get(&map, &key, budget))?
                {
                    return Ok(value);
                }
                let key = if matches!(key, Value::Number(number) if number == 0.0) {
                    Value::Number(0.0)
                } else {
                    key
                };
                let value = if computed {
                    self.call(value, Value::Undefined, vec![key.clone()], span)?
                } else {
                    value
                };
                // Callback mutations require a fresh lookup; map_set updates an
                // inserted entry in place, otherwise appending after live entries.
                self.object_work(span, |objects, budget| {
                    objects.map_set(&map, key, value.clone(), budget)
                })?;
                Ok(value)
            }
            _ => unreachable!("keyed Map method"),
        }
    }
}
