//! WeakMap constructor and object/Symbol-key methods (24.3).

use super::Builtin;
use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value, object::DataDescriptor};
use spite_core::{JsString, Span, WellKnownSymbol};

#[derive(Debug)]
pub(crate) struct WeakMapIntrinsics {
    pub constructor: ObjectHandle,
    pub prototype: ObjectHandle,
    methods: Vec<ObjectHandle>,
}

impl WeakMapIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [&self.constructor, &self.prototype]
            .into_iter()
            .chain(&self.methods)
    }
}

impl Realm {
    pub(super) fn weak_map_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<WeakMapIntrinsics, Error> {
        let constructor = self.new_builtin(function_prototype, Builtin::WeakMap, span)?;
        // WeakMap.prototype is ordinary and has no WeakMapData slot (24.3.3).
        let prototype =
            self.object_work(span, |objects, _| objects.create(Some(object_prototype)))?;
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
        methods.try_reserve_exact(6).map_err(|_| Error::Limit {
            span,
            message: "WeakMap intrinsic storage exceeds platform capacity".into(),
        })?;
        for builtin in [
            Builtin::WeakMapDelete,
            Builtin::WeakMapGet,
            Builtin::WeakMapGetOrInsert,
            Builtin::WeakMapGetOrInsertComputed,
            Builtin::WeakMapHas,
            Builtin::WeakMapSet,
        ] {
            let method = self.new_builtin(function_prototype, builtin, span)?;
            self.define_builtin_property(
                &prototype,
                builtin.initial_name(),
                Value::Object(method.clone()),
                true,
                span,
            )?;
            methods.push(method);
        }
        self.define_property_or_throw(
            &prototype,
            WellKnownSymbol::ToStringTag.symbol(),
            DataDescriptor {
                value: Some(Value::String(JsString::from("WeakMap"))),
                writable: Some(false),
                enumerable: Some(false),
                configurable: Some(true),
            }
            .into(),
            span,
        )?;
        Ok(WeakMapIntrinsics {
            constructor,
            prototype,
            methods,
        })
    }

    pub(super) fn weak_map_constructor(
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
                .expect("initialized realm")
                .weak_map
                .prototype
                .clone()
        };
        let map = self.object_work(span, |objects, _| objects.create_weak_map(&prototype))?;
        if matches!(iterable, Value::Undefined | Value::Null) {
            return Ok(Value::Object(map));
        }
        let adder = self.get_property(&map, &JsString::from("set"), span)?;
        if !self.is_callable(&adder, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "WeakMap adder is not callable",
            ));
        }
        let method = self
            .get_method(&iterable, &WellKnownSymbol::Iterator.symbol(), span)?
            .ok_or_else(|| {
                Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "WeakMap input is not iterable",
                )
            })?;
        let mut iterator = self.get_iterator_from_method(iterable, method, span)?;
        loop {
            self.tick(span)?;
            // AddEntriesFromIterable (24.1.1.2): failures in stepping/done/value
            // do not close. Entry validation, key/value Get and adder calls do.
            let Some(entry) = self.iterator_step_value(&mut iterator, span)? else {
                return Ok(Value::Object(map));
            };
            let add = (|| {
                let Value::Object(entry) = entry else {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "WeakMap entry is not an object",
                    ));
                };
                let key = self.get_property(&entry, &JsString::from("0"), span)?;
                let value = self.get_property(&entry, &JsString::from("1"), span)?;
                // Call the captured adder without prevalidating keys. A custom
                // adder may accept Symbols or other primitives without storing
                // them in WeakMapData. Native set owns the staged key boundary.
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

    fn weak_map_receiver(&mut self, receiver: Value, span: Span) -> Result<ObjectHandle, Error> {
        if let Value::Object(object) = receiver {
            if self.object_work(span, |objects, _| objects.weak_map_has_slot(&object))? {
                return Ok(object);
            }
        }
        Err(Self::exception(
            ExceptionKind::TypeError,
            span,
            "receiver has no WeakMapData internal slot",
        ))
    }

    pub(super) fn weak_map_method(
        &mut self,
        builtin: Builtin,
        receiver: Value,
        key: Value,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let map = self.weak_map_receiver(receiver, span)?;
        if !self.can_be_held_weakly(&key, span)? {
            return match builtin {
                Builtin::WeakMapGet => Ok(Value::Undefined),
                Builtin::WeakMapHas | Builtin::WeakMapDelete => Ok(Value::Boolean(false)),
                Builtin::WeakMapSet
                | Builtin::WeakMapGetOrInsert
                | Builtin::WeakMapGetOrInsertComputed => Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "WeakMap key cannot be held weakly",
                )),
                _ => unreachable!("WeakMap keyed methods"),
            };
        }
        let computed = matches!(builtin, Builtin::WeakMapGetOrInsertComputed);
        // 24.3.3.5 checks callability before lookup, including existing keys.
        if computed && !self.is_callable(&value, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "WeakMap callback is not callable",
            ));
        }
        match builtin {
            Builtin::WeakMapDelete => self
                .object_work(span, |objects, budget| match &key {
                    Value::Object(key) => objects.weak_map_delete(&map, key, budget),
                    Value::Symbol(key) => objects.weak_map_delete_symbol(&map, key, budget),
                    _ => unreachable!("validated weak key"),
                })
                .map(Value::Boolean),
            Builtin::WeakMapHas => self
                .object_work(span, |objects, budget| match &key {
                    Value::Object(key) => objects.weak_map_has(&map, key, budget),
                    Value::Symbol(key) => objects.weak_map_has_symbol(&map, key, budget),
                    _ => unreachable!("validated weak key"),
                })
                .map(Value::Boolean),
            Builtin::WeakMapGet => self
                .weak_map_entry_get(&map, &key, span)
                .map(|value| value.unwrap_or(Value::Undefined)),
            Builtin::WeakMapSet => {
                self.object_work(span, |objects, budget| match &key {
                    Value::Object(key) => objects.weak_map_set(&map, key, value, budget),
                    Value::Symbol(key) => objects.weak_map_set_symbol(&map, key, value, budget),
                    _ => unreachable!("validated weak key"),
                })?;
                Ok(Value::Object(map))
            }
            Builtin::WeakMapGetOrInsert | Builtin::WeakMapGetOrInsertComputed => {
                if let Some(value) = self.weak_map_entry_get(&map, &key, span)? {
                    return Ok(value);
                }
                let value = if computed {
                    self.call(value, Value::Undefined, vec![key.clone()], span)?
                } else {
                    value
                };
                self.object_work(span, |objects, budget| {
                    // Charge the returned/stored payload copy before cloning.
                    // No entry borrow/index survives the reentrant callback;
                    // set freshly finds and overwrites its same-key insertion.
                    budget.value(&value)?;
                    match &key {
                        Value::Object(key) => {
                            objects.weak_map_set(&map, key, value.clone(), budget)
                        }
                        Value::Symbol(key) => {
                            objects.weak_map_set_symbol(&map, key, value.clone(), budget)
                        }
                        _ => unreachable!("validated weak key"),
                    }
                })?;
                Ok(value)
            }
            _ => unreachable!("WeakMap keyed methods"),
        }
    }

    fn weak_map_entry_get(
        &mut self,
        map: &ObjectHandle,
        key: &Value,
        span: Span,
    ) -> Result<Option<Value>, Error> {
        self.object_work(span, |objects, budget| match key {
            Value::Object(key) => objects.weak_map_get(map, key, budget),
            Value::Symbol(key) => objects.weak_map_get_symbol(map, key, budget),
            _ => unreachable!("validated weak key"),
        })
    }
}
