//! Number constructor, NumberData wrappers, and supported methods (21.1).

use super::Builtin;
use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value, object::DataDescriptor};
use spite_core::{JsString, Span};

mod format;

#[derive(Debug)]
pub(crate) struct NumberIntrinsics {
    pub constructor: ObjectHandle,
    pub prototype: ObjectHandle,
    methods: [ObjectHandle; 9],
}

impl NumberIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [&self.constructor, &self.prototype]
            .into_iter()
            .chain(self.methods.iter())
    }
}

pub(super) fn predicate(builtin: Builtin, value: Option<Value>) -> bool {
    let Some(Value::Number(number)) = value else {
        return false;
    };
    match builtin {
        Builtin::NumberIsFinite => number.is_finite(),
        Builtin::NumberIsNaN => number.is_nan(),
        Builtin::NumberIsInteger => number.is_finite() && number.fract() == 0.0,
        Builtin::NumberIsSafeInteger => {
            number.is_finite() && number.fract() == 0.0 && number.abs() <= 9_007_199_254_740_991.0
        }
        _ => unreachable!("Number predicate builtin"),
    }
}

impl Realm {
    pub(super) fn number_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<NumberIntrinsics, Error> {
        let constructor = self.new_builtin(function_prototype, Builtin::Number, span)?;
        let prototype = self.object_work(span, |objects, _| {
            objects.create_number(object_prototype, 0.0)
        })?;
        self.object_work(span, |objects, budget| {
            objects.define(
                &constructor,
                JsString::from("prototype"),
                DataDescriptor {
                    value: Some(Value::Object(prototype.clone())),
                    writable: Some(false),
                    enumerable: Some(false),
                    configurable: Some(false),
                },
                budget,
            )
        })?;
        self.define_builtin_property(
            &prototype,
            "constructor",
            Value::Object(constructor.clone()),
            true,
            span,
        )?;
        let value_of = self.new_builtin(function_prototype, Builtin::NumberValueOf, span)?;
        let to_string = self.new_builtin(function_prototype, Builtin::NumberToString, span)?;
        let to_fixed = self.new_builtin(function_prototype, Builtin::NumberToFixed, span)?;
        let to_precision =
            self.new_builtin(function_prototype, Builtin::NumberToPrecision, span)?;
        let to_exponential =
            self.new_builtin(function_prototype, Builtin::NumberToExponential, span)?;
        self.define_builtin_property(
            &prototype,
            "toExponential",
            Value::Object(to_exponential.clone()),
            true,
            span,
        )?;
        self.define_builtin_property(
            &prototype,
            "toPrecision",
            Value::Object(to_precision.clone()),
            true,
            span,
        )?;
        self.define_builtin_property(
            &prototype,
            "toFixed",
            Value::Object(to_fixed.clone()),
            true,
            span,
        )?;
        self.define_builtin_property(
            &prototype,
            "valueOf",
            Value::Object(value_of.clone()),
            true,
            span,
        )?;
        self.define_builtin_property(
            &prototype,
            "toString",
            Value::Object(to_string.clone()),
            true,
            span,
        )?;
        let is_finite = self.new_builtin(function_prototype, Builtin::NumberIsFinite, span)?;
        let is_nan = self.new_builtin(function_prototype, Builtin::NumberIsNaN, span)?;
        let is_integer = self.new_builtin(function_prototype, Builtin::NumberIsInteger, span)?;
        let is_safe_integer =
            self.new_builtin(function_prototype, Builtin::NumberIsSafeInteger, span)?;
        for (name, value) in [
            ("isFinite", &is_finite),
            ("isNaN", &is_nan),
            ("isInteger", &is_integer),
            ("isSafeInteger", &is_safe_integer),
        ] {
            self.define_builtin_property(
                &constructor,
                name,
                Value::Object(value.clone()),
                true,
                span,
            )?;
        }
        for (name, value) in [
            ("EPSILON", f64::EPSILON),
            ("MAX_SAFE_INTEGER", 9_007_199_254_740_991.0),
            ("MIN_SAFE_INTEGER", -9_007_199_254_740_991.0),
            ("MAX_VALUE", f64::MAX),
            ("MIN_VALUE", f64::from_bits(1)),
            ("NaN", f64::NAN),
            ("NEGATIVE_INFINITY", f64::NEG_INFINITY),
            ("POSITIVE_INFINITY", f64::INFINITY),
        ] {
            self.object_work(span, |objects, budget| {
                objects.define(
                    &constructor,
                    JsString::from(name),
                    DataDescriptor {
                        value: Some(Value::Number(value)),
                        writable: Some(false),
                        enumerable: Some(false),
                        configurable: Some(false),
                    },
                    budget,
                )
            })?;
        }
        Ok(NumberIntrinsics {
            constructor,
            prototype,
            methods: [
                value_of,
                to_string,
                to_fixed,
                to_precision,
                to_exponential,
                is_finite,
                is_nan,
                is_integer,
                is_safe_integer,
            ],
        })
    }

    pub(super) fn number_constructor_value(
        &mut self,
        value: Option<Value>,
        span: Span,
    ) -> Result<f64, Error> {
        let Some(value) = value else {
            return Ok(0.0);
        };
        // 21.1.1.1 deliberately permits BigInt, unlike ordinary ToNumber.
        match self.numeric(value, span)? {
            Value::Number(value) => Ok(value),
            Value::BigInt(value) => self.integer_work(span, |budget| value.to_f64(budget)),
            _ => unreachable!("ToNumeric returns Number or BigInt"),
        }
    }

    pub(super) fn this_number_value(&mut self, value: &Value, span: Span) -> Result<f64, Error> {
        let value = match value {
            Value::Number(value) => Some(*value),
            Value::Object(object) => self.object_work(span, |objects, _| {
                Ok(objects.inspect(object)?.number_data())
            })?,
            _ => None,
        };
        value.ok_or_else(|| {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "receiver does not contain a Number value",
            )
        })
    }

    pub(super) fn number_prototype_to_string(
        &mut self,
        this: &Value,
        radix: Option<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        // 21.1.3.6 validates this before converting radix, then validates radix
        // before handling NaN, infinities, or zero.
        let value = self.this_number_value(this, span)?;
        let radix = match radix {
            None | Some(Value::Undefined) => 10.0,
            Some(radix) => self.number(radix, span)?.trunc(),
        };
        if !(2.0..=36.0).contains(&radix) {
            return Err(Self::exception(
                ExceptionKind::RangeError,
                span,
                "radix must be between 2 and 36",
            ));
        }
        if radix != 10.0 && value.is_finite() && value != 0.0 {
            return Err(Self::unsupported(
                span,
                "non-decimal Number formatting is not implemented",
            ));
        }
        Ok(Value::String(JsString::from(
            crate::value::number_to_string(value).as_str(),
        )))
    }
}
