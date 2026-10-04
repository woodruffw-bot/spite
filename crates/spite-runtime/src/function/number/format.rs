//! Exact fixed-point formatting from binary64 components (21.1.3.3).

use crate::{Error, ExceptionKind, Realm, Value};
use spite_bigint::{BigInt, Budget, Error as IntegerError};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn number_prototype_to_fixed(
        &mut self,
        this: &Value,
        fraction: Option<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let value = self.this_number_value(this, span)?;
        let fraction = self.number(fraction.unwrap_or(Value::Undefined), span)?;
        let fraction = if fraction.is_nan() {
            0.0
        } else {
            fraction.trunc()
        };
        if !(0.0..=100.0).contains(&fraction) {
            return Err(Self::exception(
                ExceptionKind::RangeError,
                span,
                "fraction digits must be between 0 and 100",
            ));
        }
        if !value.is_finite() || value.abs() >= 1e21 {
            return Ok(Value::String(JsString::from(
                crate::value::number_to_string(value).as_str(),
            )));
        }
        let fraction = fraction as usize;
        let digits = self.integer_work(span, |budget| {
            scaled_integer(value.abs(), fraction, budget)?.to_radix(10, budget)
        })?;
        let mut result = String::new();
        if value < 0.0 {
            result.push('-');
        }
        if fraction == 0 {
            result.push_str(&digits);
        } else if digits.len() <= fraction {
            result.push_str("0.");
            result.extend(std::iter::repeat_n('0', fraction - digits.len()));
            result.push_str(&digits);
        } else {
            let point = digits.len() - fraction;
            result.push_str(&digits[..point]);
            result.push('.');
            result.push_str(&digits[point..]);
        }
        Ok(Value::String(JsString::from(result.as_str())))
    }
}

fn scaled_integer(
    value: f64,
    fraction: usize,
    budget: &mut Budget,
) -> Result<BigInt, IntegerError> {
    debug_assert!((0.0..1e21).contains(&value));
    if value == 0.0 {
        budget.charge(1)?;
        return Ok(BigInt::default());
    }
    let bits = value.to_bits();
    let biased = ((bits >> 52) & 0x7ff) as i64;
    let significand = (bits & ((1u64 << 52) - 1)) | if biased == 0 { 0 } else { 1u64 << 52 };
    let exponent = if biased == 0 {
        -1074
    } else {
        biased - 1023 - 52
    };
    let scale = BigInt::from(10).pow(&BigInt::from(fraction as i64), budget)?;
    let numerator = BigInt::from(significand as i64).mul(&scale, budget)?;
    if exponent >= 0 {
        return numerator.shl(&BigInt::from(exponent), budget);
    }
    let denominator = BigInt::from(1).shl(&BigInt::from(-exponent), budget)?;
    let (integer, remainder) = numerator.div_rem(&denominator, budget)?;
    let twice = remainder.shl(&BigInt::from(1), budget)?;
    budget.charge(
        twice
            .bit_length()
            .max(denominator.bit_length())
            .div_ceil(32),
    )?;
    if twice >= denominator {
        integer.add(&BigInt::from(1), budget)
    } else {
        Ok(integer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_work_and_generated_strings_obey_host_limits() {
        assert_eq!(
            scaled_integer(1.25, 2, &mut Budget::new(4096, 0)),
            Err(IntegerError::Limit)
        );
        let mut realm = Realm::default();
        realm
            .eval("Number.prototype.f=Number.prototype.toFixed; let flag=0")
            .unwrap();
        realm.limits.max_string_units = 4;
        assert_eq!(
            realm.eval("(1).f(2)"),
            Ok(Value::String(JsString::from("1.00")))
        );
        assert!(matches!(
            realm.eval("try{(1).f(3);}catch{flag=1;}finally{flag=2;}"),
            Err(Error::Limit { .. })
        ));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
        assert_eq!(
            realm.eval("(1).f(2)"),
            Ok(Value::String(JsString::from("1.00")))
        );
    }
}
