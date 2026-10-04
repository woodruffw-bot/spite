//! Math object constants and basic numeric operations (21.3.1, 21.3.2).

use super::Builtin;
use crate::{Error, ObjectHandle, Realm, Value, object::DataDescriptor};
use spite_core::{JsString, Span, WellKnownSymbol};

#[derive(Debug)]
pub(crate) struct MathIntrinsics {
    pub object: ObjectHandle,
    methods: Vec<ObjectHandle>,
}

impl MathIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        std::iter::once(&self.object).chain(self.methods.iter())
    }
}

impl Realm {
    pub(super) fn math_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<MathIntrinsics, Error> {
        let object = self.object_work(span, |objects, _| objects.create(Some(object_prototype)))?;
        // 21.3.1: fixed mathematical constants use their nearest binary64 values.
        for (name, value) in [
            ("E", std::f64::consts::E),
            ("LN10", std::f64::consts::LN_10),
            ("LN2", std::f64::consts::LN_2),
            ("LOG10E", std::f64::consts::LOG10_E),
            ("LOG2E", std::f64::consts::LOG2_E),
            ("PI", std::f64::consts::PI),
            ("SQRT1_2", std::f64::consts::FRAC_1_SQRT_2),
            ("SQRT2", std::f64::consts::SQRT_2),
        ] {
            self.object_work(span, |objects, budget| {
                objects.define(
                    &object,
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
        self.object_work(span, |objects, budget| {
            objects.define(
                &object,
                WellKnownSymbol::ToStringTag.symbol(),
                DataDescriptor {
                    value: Some(Value::String(JsString::from("Math"))),
                    writable: Some(false),
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                budget,
            )
        })?;
        let mut methods = Vec::new();
        for builtin in [
            Builtin::MathAbs,
            Builtin::MathCeil,
            Builtin::MathFloor,
            Builtin::MathRound,
            Builtin::MathSign,
            Builtin::MathTrunc,
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
        Ok(MathIntrinsics { object, methods })
    }

    pub(super) fn math_unary(
        &mut self,
        builtin: Builtin,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let number = self.number(value, span)?;
        let result = match builtin {
            Builtin::MathAbs => number.abs(),
            Builtin::MathCeil => number.ceil(),
            Builtin::MathFloor => number.floor(),
            Builtin::MathRound => round(number),
            Builtin::MathSign => {
                if number.is_nan() || number == 0.0 {
                    number
                } else {
                    number.signum()
                }
            }
            Builtin::MathTrunc => number.trunc(),
            _ => unreachable!("Math unary operation"),
        };
        Ok(Value::Number(result))
    }
}

fn round(number: f64) -> f64 {
    // Math.round (sec-math.round): halfway ties go toward +Infinity. Preserve -0,
    // and avoid adding 0.5 to the input: that addition can round near 0.5 or
    // change already-integral odd values above 2^52.
    if !number.is_finite() || number == 0.0 {
        return number;
    }
    if (-0.5..0.0).contains(&number) {
        return -0.0;
    }
    let lower = number.floor();
    if number - lower < 0.5 {
        lower
    } else {
        lower + 1.0
    }
}
