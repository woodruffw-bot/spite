//! BigInt construction, wrappers, and branded prototype methods (21.2).

use super::Builtin;
use crate::{Error, ExceptionKind, Hint, ObjectHandle, Realm, Value, object::DataDescriptor};
use spite_bigint::BigInt;
use spite_core::{JsString, Span, WellKnownSymbol};

#[derive(Debug)]
pub(crate) struct BigIntIntrinsics {
    pub constructor: ObjectHandle,
    pub prototype: ObjectHandle,
    methods: [ObjectHandle; 3],
}

impl BigIntIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [&self.constructor, &self.prototype]
            .into_iter()
            .chain(self.methods.iter())
    }
}

impl Realm {
    pub(super) fn bigint_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<BigIntIntrinsics, Error> {
        let constructor = self.new_builtin(function_prototype, Builtin::BigInt, span)?;
        // BigInt.prototype is ordinary and does not itself have [[BigIntData]].
        let prototype =
            self.object_work(span, |objects, _| objects.create(Some(object_prototype)))?;
        self.object_work(span, |objects, budget| {
            objects.define(
                &constructor,
                "prototype",
                DataDescriptor {
                    value: Some(Value::Object(prototype.clone())),
                    writable: Some(false),
                    enumerable: Some(false),
                    configurable: Some(false),
                },
                budget,
            )?;
            objects.define(
                &prototype,
                WellKnownSymbol::ToStringTag.symbol(),
                DataDescriptor {
                    value: Some(Value::String(JsString::from("BigInt"))),
                    writable: Some(false),
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                budget,
            )?;
            Ok(())
        })?;
        self.define_builtin_property(
            &prototype,
            "constructor",
            Value::Object(constructor.clone()),
            true,
            span,
        )?;
        let to_string = self.new_builtin(function_prototype, Builtin::BigIntToString, span)?;
        let to_locale_string =
            self.new_builtin(function_prototype, Builtin::BigIntToLocaleString, span)?;
        let value_of = self.new_builtin(function_prototype, Builtin::BigIntValueOf, span)?;
        for (name, method) in [
            ("toString", &to_string),
            ("toLocaleString", &to_locale_string),
            ("valueOf", &value_of),
        ] {
            self.define_builtin_property(
                &prototype,
                name,
                Value::Object(method.clone()),
                true,
                span,
            )?;
        }
        Ok(BigIntIntrinsics {
            constructor,
            prototype,
            methods: [to_string, to_locale_string, value_of],
        })
    }

    pub(crate) fn bigint_constructor(&mut self, value: Value, span: Span) -> Result<Value, Error> {
        // 21.2.1.1 permits integral Numbers after ToPrimitive(number), unlike
        // abstract ToBigInt. Common Construct rejects BigInt before coercion.
        let value = self.primitive(value, Hint::Number, span)?;
        let value = match value {
            Value::Number(number) => self
                .integer_work(span, |budget| BigInt::from_f64(number, budget))?
                .ok_or_else(|| {
                    Self::exception(
                        ExceptionKind::RangeError,
                        span,
                        "Number is not a finite integer",
                    )
                })?,
            value => self.bigint(value, span)?,
        };
        Ok(Value::BigInt(value))
    }

    /// ToBigInt (7.1.13), preserving the Number rejection used by width APIs.
    pub(crate) fn bigint(&mut self, value: Value, span: Span) -> Result<BigInt, Error> {
        match self.primitive(value, Hint::Number, span)? {
            Value::BigInt(value) => Ok(value),
            Value::Boolean(value) => self.integer_work(span, |budget| {
                BigInt::parse_digits(if value { "1" } else { "0" }, 10, budget)
            }),
            Value::String(text) => self
                .integer_work(span, |budget| crate::value::string_to_bigint(&text, budget))?
                .ok_or_else(|| {
                    Self::exception(
                        ExceptionKind::SyntaxError,
                        span,
                        "String is not a valid BigInt integer",
                    )
                }),
            _ => Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "value cannot be converted to BigInt",
            )),
        }
    }

    pub(crate) fn this_bigint_value(&mut self, value: &Value, span: Span) -> Result<BigInt, Error> {
        let bigint = match value {
            Value::BigInt(value) => {
                self.object_work(span, |_, budget| {
                    budget.charge(value.bit_length().div_ceil(32))
                })?;
                Some(value.clone())
            }
            Value::Object(object) => self.object_work(span, |objects, budget| {
                let value = objects.inspect(object)?.bigint_data();
                if let Some(value) = value {
                    budget.charge(value.bit_length().div_ceil(32))?;
                }
                Ok(value.cloned())
            })?,
            _ => None,
        };
        bigint.ok_or_else(|| {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "receiver does not contain a BigInt value",
            )
        })
    }

    pub(crate) fn bigint_to_string(
        &mut self,
        receiver: &Value,
        radix: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // 21.2.3.3: validate the receiver before coercing/range-checking radix.
        let value = self.this_bigint_value(receiver, span)?;
        let radix = if matches!(radix, Value::Undefined) {
            10.0
        } else {
            self.number(radix, span)?.trunc()
        };
        if !(2.0..=36.0).contains(&radix) {
            return Err(Self::exception(
                ExceptionKind::RangeError,
                span,
                "radix must be between 2 and 36",
            ));
        }
        let text = self.integer_work(span, |budget| value.to_radix(radix as u32, budget))?;
        let text = Value::String(JsString::from(text.as_str()));
        self.check_string(&text, span)?;
        Ok(text)
    }
}
