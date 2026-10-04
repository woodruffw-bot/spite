//! Math object constants and basic numeric operations (21.3.1, 21.3.2).

mod random;
mod sum;

use super::Builtin;
use crate::{
    Error, ObjectHandle, Realm, Value,
    object::DataDescriptor,
    value::{exponentiate, to_uint32},
};
use spite_core::{JsString, Span, WellKnownSymbol};

#[derive(Debug)]
pub(crate) struct MathIntrinsics {
    pub object: ObjectHandle,
    methods: Vec<ObjectHandle>,
    random: random::RandomSequence,
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
            Builtin::MathAcos,
            Builtin::MathAcosh,
            Builtin::MathAsin,
            Builtin::MathAsinh,
            Builtin::MathAtan,
            Builtin::MathAtanh,
            Builtin::MathAtan2,
            Builtin::MathCbrt,
            Builtin::MathCeil,
            Builtin::MathClz32,
            Builtin::MathCos,
            Builtin::MathCosh,
            Builtin::MathExp,
            Builtin::MathExpm1,
            Builtin::MathFloor,
            Builtin::MathFround,
            Builtin::MathF16round,
            Builtin::MathHypot,
            Builtin::MathImul,
            Builtin::MathLog,
            Builtin::MathLog1p,
            Builtin::MathLog2,
            Builtin::MathLog10,
            Builtin::MathMax,
            Builtin::MathMin,
            Builtin::MathPow,
            Builtin::MathRandom,
            Builtin::MathRound,
            Builtin::MathSign,
            Builtin::MathSin,
            Builtin::MathSinh,
            Builtin::MathSqrt,
            Builtin::MathSumPrecise,
            Builtin::MathTan,
            Builtin::MathTanh,
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
        Ok(MathIntrinsics {
            object,
            methods,
            random: random::RandomSequence::new(span)?,
        })
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
            Builtin::MathAcos
            | Builtin::MathAcosh
            | Builtin::MathAsin
            | Builtin::MathAsinh
            | Builtin::MathAtan
            | Builtin::MathAtanh
            | Builtin::MathCbrt => inverse_unary(number, builtin),
            Builtin::MathCos
            | Builtin::MathCosh
            | Builtin::MathSin
            | Builtin::MathSinh
            | Builtin::MathTan
            | Builtin::MathTanh => trigonometric_unary(number, builtin),
            Builtin::MathCeil => number.ceil(),
            Builtin::MathClz32 => f64::from(to_uint32(number).leading_zeros()),
            Builtin::MathExp => {
                if number == 0.0 {
                    1.0
                } else if number == f64::NEG_INFINITY {
                    0.0
                } else {
                    number.exp()
                }
            }
            Builtin::MathExpm1 => {
                if number == 0.0 || number.is_nan() {
                    number
                } else if number == f64::NEG_INFINITY {
                    -1.0
                } else {
                    number.exp_m1()
                }
            }
            Builtin::MathFloor => number.floor(),
            // sec-math.fround: Rust's narrowing float cast uses ties to even;
            // widening the binary32 result back to binary64 is exact.
            Builtin::MathFround => f64::from(number as f32),
            Builtin::MathF16round => binary16_round(number),
            Builtin::MathLog | Builtin::MathLog2 | Builtin::MathLog10 => logarithm(number, builtin),
            Builtin::MathLog1p => {
                if number == 0.0 || number.is_nan() {
                    number
                } else if number < -1.0 {
                    f64::NAN
                } else if number == -1.0 {
                    f64::NEG_INFINITY
                } else {
                    number.ln_1p()
                }
            }
            Builtin::MathRound => round(number),
            Builtin::MathSign => {
                if number.is_nan() || number == 0.0 {
                    number
                } else {
                    number.signum()
                }
            }
            Builtin::MathTrunc => number.trunc(),
            // sec-math.sqrt requires the nearest binary64 result. Rust's sqrt
            // is the correctly rounded IEEE squareRoot operation, including -0.
            Builtin::MathSqrt => number.sqrt(),
            _ => unreachable!("Math unary operation"),
        };
        Ok(Value::Number(result))
    }

    pub(super) fn math_extremum(
        &mut self,
        arguments: impl Iterator<Item = Value>,
        maximum: bool,
        span: Span,
    ) -> Result<Value, Error> {
        // sec-math.max/min: every ToNumber precedes the NaN result. Folding
        // converted numbers here is unobservable and avoids a second list.
        let mut result = if maximum {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
        let mut nan = false;
        for argument in arguments {
            let number = self.number(argument, span)?;
            nan |= number.is_nan();
            let better = if maximum {
                number > result
            } else {
                number < result
            };
            // +0 wins in max; -0 wins in min, in either argument order.
            let preferred_zero =
                number == 0.0 && result == 0.0 && number.is_sign_negative() != maximum;
            if better || preferred_zero {
                result = number;
            }
        }
        Ok(Value::Number(if nan { f64::NAN } else { result }))
    }

    pub(super) fn math_hypot(
        &mut self,
        arguments: impl Iterator<Item = Value>,
        span: Span,
    ) -> Result<Value, Error> {
        // sec-math.hypot: convert every argument before choosing Infinity or
        // NaN. Interleaved arithmetic is unobservable; later coercion errors
        // still win. Scale by the largest finite magnitude to avoid squaring
        // large/tiny arguments directly, with compensated square summation.
        let mut scale = 0.0;
        let mut squares = 0.0;
        let mut compensation = 0.0;
        let mut infinite = false;
        let mut nan = false;
        for argument in arguments {
            let number = self.number(argument, span)?.abs();
            if number.is_infinite() {
                infinite = true;
            } else if number.is_nan() {
                nan = true;
            } else if number != 0.0 {
                let square = if number > scale {
                    let ratio = scale / number;
                    let factor = ratio * ratio;
                    squares *= factor;
                    compensation *= factor;
                    scale = number;
                    1.0
                } else {
                    let ratio = number / scale;
                    ratio * ratio
                };
                let corrected = square - compensation;
                let total = squares + corrected;
                compensation = (total - squares) - corrected;
                squares = total;
            }
        }
        let result = if infinite {
            f64::INFINITY
        } else if nan {
            f64::NAN
        } else if scale == 0.0 {
            0.0
        } else {
            scale * squares.sqrt()
        };
        Ok(Value::Number(result))
    }

    pub(super) fn math_imul(
        &mut self,
        left: Value,
        right: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // sec-math.imul: ordered ToUint32, multiplication modulo 2^32, then
        // signed interpretation. Integer zero converts back to positive zero.
        let left = to_uint32(self.number(left, span)?);
        let right = to_uint32(self.number(right, span)?);
        Ok(Value::Number(f64::from(left.wrapping_mul(right) as i32)))
    }

    pub(super) fn math_pow(
        &mut self,
        base: Value,
        exponent: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // sec-math.pow: both ordered ToNumber conversions precede Number::
        // exponentiate, including when NaN or a zero exponent decides its result.
        let base = self.number(base, span)?;
        let exponent = self.number(exponent, span)?;
        Ok(Value::Number(exponentiate(base, exponent)))
    }

    pub(super) fn math_atan2(&mut self, y: Value, x: Value, span: Span) -> Result<Value, Error> {
        // sec-math.atan2: y is converted before x; both conversions precede
        // numeric special cases, even when y is NaN.
        let y = self.number(y, span)?;
        let x = self.number(x, span)?;
        Ok(Value::Number(atan2(y, x)))
    }
}

// sec-math.cos/cosh/sin/sinh/tan/tanh: required endpoints precede
// implementation-approximated finite results. In particular, -0 must survive
// odd functions and either infinity is outside circular functions' domains.
fn trigonometric_unary(number: f64, builtin: Builtin) -> f64 {
    match builtin {
        Builtin::MathCos => {
            if number == 0.0 {
                1.0
            } else if !number.is_finite() {
                f64::NAN
            } else {
                number.cos()
            }
        }
        Builtin::MathSin | Builtin::MathTan => {
            if number == 0.0 || number.is_nan() {
                number
            } else if number.is_infinite() {
                f64::NAN
            } else if matches!(builtin, Builtin::MathSin) {
                number.sin()
            } else {
                number.tan()
            }
        }
        Builtin::MathSinh => {
            if number == 0.0 || !number.is_finite() {
                number
            } else {
                number.sinh()
            }
        }
        Builtin::MathCosh => {
            if number == 0.0 {
                1.0
            } else if number.is_infinite() {
                f64::INFINITY
            } else {
                number.cosh()
            }
        }
        Builtin::MathTanh => {
            if number == 0.0 || number.is_nan() {
                number
            } else if number.is_infinite() {
                number.signum()
            } else {
                number.tanh()
            }
        }
        _ => unreachable!("Math trigonometric operation"),
    }
}

fn inverse_unary(number: f64, builtin: Builtin) -> f64 {
    // sec-math.acos/acosh/asin/asinh/atan/atanh/cbrt: handle required domain
    // and signed endpoint results before platform finite approximations.
    use std::f64::consts::FRAC_PI_2;
    match builtin {
        Builtin::MathAcos => {
            if number.abs() > 1.0 {
                f64::NAN
            } else if number == 1.0 {
                0.0
            } else {
                number.acos()
            }
        }
        Builtin::MathAcosh => {
            if number < 1.0 {
                f64::NAN
            } else if number == 1.0 {
                0.0
            } else if number > 268_435_456.0 {
                log_twice(number)
            } else {
                number.acosh()
            }
        }
        Builtin::MathAsin => {
            if number.abs() > 1.0 {
                f64::NAN
            } else if number == 0.0 {
                number
            } else if number.abs() == 1.0 {
                FRAC_PI_2.copysign(number)
            } else {
                number.asin()
            }
        }
        Builtin::MathAsinh => {
            if number == 0.0 || !number.is_finite() {
                number
            } else if number.abs() > 268_435_456.0 {
                log_twice(number.abs()).copysign(number)
            } else {
                number.asinh()
            }
        }
        Builtin::MathCbrt => {
            if number == 0.0 || !number.is_finite() {
                number
            } else {
                number.cbrt()
            }
        }
        Builtin::MathAtan => {
            if number == 0.0 || number.is_nan() {
                number
            } else if number.is_infinite() {
                FRAC_PI_2.copysign(number)
            } else {
                number.atan()
            }
        }
        Builtin::MathAtanh => {
            if number.abs() > 1.0 {
                f64::NAN
            } else if number == 0.0 || number.is_nan() {
                number
            } else if number.abs() == 1.0 {
                f64::INFINITY.copysign(number)
            } else {
                // Evaluate the positive magnitude then restore its sign. The
                // negative formula can round its log1p argument to -1 even
                // for the closest Number strictly greater than -1.
                let magnitude = number.abs();
                (0.5 * ((2.0 * magnitude) / (1.0 - magnitude)).ln_1p()).copysign(number)
            }
        }
        _ => unreachable!("Math inverse/cube-root operation"),
    }
}

fn log_twice(magnitude: f64) -> f64 {
    // For x > 2^28, asinh(x) and acosh(x) differ from log(2*x) by
    // O(1/x^2), below binary64 spacing here. Avoid forming 2*x, which
    // overflows in Rust 1.85's inverse-hyperbolic implementations.
    magnitude.ln() + std::f64::consts::LN_2
}

fn atan2(y: f64, x: f64) -> f64 {
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};
    if y.is_nan() || x.is_nan() {
        return f64::NAN;
    }
    if y == 0.0 {
        return if x.is_sign_negative() {
            PI.copysign(y)
        } else {
            y
        };
    }
    if x == 0.0 {
        return FRAC_PI_2.copysign(y);
    }
    if y.is_infinite() {
        let angle = if x.is_infinite() {
            if x > 0.0 { FRAC_PI_4 } else { 3.0 * FRAC_PI_4 }
        } else {
            FRAC_PI_2
        };
        return angle.copysign(y);
    }
    if x.is_infinite() {
        return if x > 0.0 {
            0.0_f64.copysign(y)
        } else {
            PI.copysign(y)
        };
    }
    y.atan2(x)
}

fn logarithm(number: f64, builtin: Builtin) -> f64 {
    // sec-math.log/log2/log10: domain and endpoint results precede the
    // implementation-approximated finite logarithm.
    if number.is_nan() || number < 0.0 {
        return f64::NAN;
    }
    if number == 0.0 {
        return f64::NEG_INFINITY;
    }
    if number == 1.0 {
        return 0.0;
    }
    if number == f64::INFINITY {
        return number;
    }
    match builtin {
        Builtin::MathLog => number.ln(),
        Builtin::MathLog10 => number.log10(),
        Builtin::MathLog2 => {
            // Decode powers of two exactly, including subnormals, independent
            // of the platform logarithm's approximation.
            let bits = number.to_bits();
            let exponent = ((bits >> 52) & 0x7ff) as i32;
            let fraction = bits & ((1_u64 << 52) - 1);
            if exponent != 0 && fraction == 0 {
                f64::from(exponent - 1023)
            } else if exponent == 0 && fraction.is_power_of_two() {
                f64::from(fraction.trailing_zeros()) - 1074.0
            } else {
                number.log2()
            }
        }
        _ => unreachable!("Math logarithm operation"),
    }
}

fn binary16_round(number: f64) -> f64 {
    // sec-math.f16round: direct binary64 -> binary16 rounding, never through
    // binary32 (which would misround values adjacent to binary16 half ties).
    if !number.is_finite() || number == 0.0 {
        return number;
    }
    let magnitude = number.abs();
    if magnitude >= 65520.0 {
        // Halfway between the largest finite half (65504) and 2^16 overflows
        // under roundTiesToEven, just like larger magnitudes.
        return f64::INFINITY.copysign(number);
    }
    let quantum = if magnitude < 1.0 / 16384.0 {
        // Subnormal binary16 spacing is fixed at 2^-24.
        1.0 / 16777216.0
    } else {
        // Normal spacing is 2^(floor(log2(magnitude)) - 10). The bounded
        // magnitude has a binary64 exponent field >= 1009, so subtraction
        // cannot underflow. Constructing this power of two is exact.
        let exponent = (magnitude.to_bits() >> 52) & 0x7ff;
        f64::from_bits((exponent - 10) << 52)
    };
    // Scaling by this power of two is exact even for the smallest
    // binary64 subnormal. The rounded integer and rescaling are also exact.
    ((magnitude / quantum).round_ties_even() * quantum).copysign(number)
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

#[cfg(test)]
mod tests {
    use super::binary16_round;

    fn half_value(bits: u16) -> f64 {
        let exponent = i32::from(bits >> 10);
        let fraction = f64::from(bits & 1023);
        if exponent == 0 {
            fraction / 16777216.0
        } else {
            (1024.0 + fraction) * 2.0_f64.powi(exponent - 25)
        }
    }

    fn check_both_signs(input: f64, expected: f64) {
        assert_eq!(
            binary16_round(input).to_bits(),
            expected.to_bits(),
            "{input}"
        );
        assert_eq!(
            binary16_round(-input).to_bits(),
            (-expected).to_bits(),
            "{}",
            -input
        );
    }

    #[test]
    fn every_finite_half_value_and_adjacent_rounding_boundary_is_exact() {
        for bits in 0..=0x7bff {
            let lower = half_value(bits);
            check_both_signs(lower, lower);
            if bits < 0x7bff {
                let upper = half_value(bits + 1);
                let midpoint = (lower + upper) / 2.0;
                let even = if bits & 1 == 0 { lower } else { upper };
                check_both_signs(f64::from_bits(midpoint.to_bits() - 1), lower);
                check_both_signs(midpoint, even);
                check_both_signs(f64::from_bits(midpoint.to_bits() + 1), upper);
            }
        }
        check_both_signs(f64::from_bits(65520.0_f64.to_bits() - 1), 65504.0);
        check_both_signs(65520.0, f64::INFINITY);
        check_both_signs(f64::MAX, f64::INFINITY);
        check_both_signs(f64::from_bits(1), 0.0);
    }
}
