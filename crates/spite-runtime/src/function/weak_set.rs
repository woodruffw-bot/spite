//! WeakSet construction and branded methods with CanBeHeldWeakly (24.4).

use super::Builtin;
use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value, object::DataDescriptor};
use spite_core::{JsString, Span, WellKnownSymbol};

#[derive(Debug)]
pub(crate) struct WeakSetIntrinsics {
    pub constructor: ObjectHandle,
    pub prototype: ObjectHandle,
    methods: Vec<ObjectHandle>,
}

impl WeakSetIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [&self.constructor, &self.prototype]
            .into_iter()
            .chain(&self.methods)
    }
}

impl Realm {
    pub(super) fn weak_set_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<WeakSetIntrinsics, Error> {
        let constructor = self.new_builtin(function_prototype, Builtin::WeakSet, span)?;
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
        for builtin in [
            Builtin::WeakSetAdd,
            Builtin::WeakSetDelete,
            Builtin::WeakSetHas,
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
                value: Some(Value::String(JsString::from("WeakSet"))),
                writable: Some(false),
                enumerable: Some(false),
                configurable: Some(true),
            }
            .into(),
            span,
        )?;
        Ok(WeakSetIntrinsics {
            constructor,
            prototype,
            methods,
        })
    }

    pub(super) fn weak_set_constructor(
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
                .weak_set
                .prototype
                .clone()
        };
        let set = self.object_work(span, |objects, _| objects.create_weak_set(&prototype))?;
        if matches!(iterable, Value::Undefined | Value::Null) {
            return Ok(Value::Object(set));
        }
        let adder = self.get_property(&set, &JsString::from("add"), span)?;
        if !self.is_callable(&adder, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "WeakSet adder is not callable",
            ));
        }
        let method = self
            .get_method(&iterable, &WellKnownSymbol::Iterator.symbol(), span)?
            .ok_or_else(|| {
                Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "WeakSet input is not iterable",
                )
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

    fn weak_set_receiver(&mut self, receiver: Value, span: Span) -> Result<ObjectHandle, Error> {
        if let Value::Object(object) = receiver {
            if self.object_work(span, |objects, _| objects.weak_set_has_slot(&object))? {
                return Ok(object);
            }
        }
        Err(Self::exception(
            ExceptionKind::TypeError,
            span,
            "receiver has no WeakSetData internal slot",
        ))
    }

    fn can_be_held_weakly(&mut self, value: &Value, span: Span) -> Result<bool, Error> {
        match value {
            Value::Object(_) => Ok(true),
            Value::Symbol(symbol) => self.symbol_is_registered(symbol, span).map(|value| !value),
            _ => Ok(false),
        }
    }

    pub(super) fn weak_set_method(
        &mut self,
        builtin: Builtin,
        receiver: Value,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let set = self.weak_set_receiver(receiver, span)?;
        if !self.can_be_held_weakly(&value, span)? {
            return if matches!(builtin, Builtin::WeakSetAdd) {
                Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "WeakSet value cannot be held weakly",
                ))
            } else {
                Ok(Value::Boolean(false))
            };
        }
        match builtin {
            Builtin::WeakSetAdd => {
                self.object_work(span, |objects, budget| {
                    objects.weak_set_add(&set, &value, budget)
                })?;
                Ok(Value::Object(set))
            }
            Builtin::WeakSetDelete => self
                .object_work(span, |objects, budget| {
                    objects.weak_set_delete(&set, &value, budget)
                })
                .map(Value::Boolean),
            Builtin::WeakSetHas => self
                .object_work(span, |objects, budget| {
                    objects.weak_set_has(&set, &value, budget)
                })
                .map(Value::Boolean),
            _ => unreachable!("WeakSet method"),
        }
    }
}
