//! Decimal formatting from binary64 components (21.1.3.2–3, 21.1.3.5).

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
            scaled_integer(value.abs(), fraction as i32, budget)?.to_radix(10, budget)
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

    pub(crate) fn number_prototype_to_precision(
        &mut self,
        this: &Value,
        precision: Option<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let value = self.this_number_value(this, span)?;
        let Some(precision) = precision.filter(|value| !matches!(value, Value::Undefined)) else {
            return Ok(Value::String(JsString::from(
                crate::value::number_to_string(value).as_str(),
            )));
        };
        let precision = self.number(precision, span)?.trunc();
        // Unlike toFixed, nonfinite values return before the range check.
        if !value.is_finite() {
            return Ok(Value::String(JsString::from(
                crate::value::number_to_string(value).as_str(),
            )));
        }
        if !(1.0..=100.0).contains(&precision) {
            return Err(Self::exception(
                ExceptionKind::RangeError,
                span,
                "precision must be between 1 and 100",
            ));
        }
        let precision = precision as usize;
        let (digits, exponent) = self.integer_work(span, |budget| {
            significant_digits(value.abs(), precision, budget)
        })?;
        let mut result = String::new();
        if value < 0.0 {
            result.push('-');
        }
        if exponent < -6 || exponent >= precision as i32 {
            append_exponential(&mut result, &digits, exponent);
        } else if exponent < 0 {
            result.push_str("0.");
            result.extend(std::iter::repeat_n('0', (-exponent - 1) as usize));
            result.push_str(&digits);
        } else {
            let point = exponent as usize + 1;
            result.push_str(&digits[..point]);
            if point < precision {
                result.push('.');
                result.push_str(&digits[point..]);
            }
        }
        Ok(Value::String(JsString::from(result.as_str())))
    }

    pub(crate) fn number_prototype_to_exponential(
        &mut self,
        this: &Value,
        fraction: Option<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let value = self.this_number_value(this, span)?;
        let fraction = fraction.unwrap_or(Value::Undefined);
        let shortest = matches!(fraction, Value::Undefined);
        let fraction = self.number(fraction, span)?;
        let fraction = if fraction.is_nan() {
            0.0
        } else {
            fraction.trunc()
        };
        if !value.is_finite() {
            return Ok(Value::String(JsString::from(
                crate::value::number_to_string(value).as_str(),
            )));
        }
        if !(0.0..=100.0).contains(&fraction) {
            return Err(Self::exception(
                ExceptionKind::RangeError,
                span,
                "fraction digits must be between 0 and 100",
            ));
        }
        let (digits, exponent) = if shortest {
            let text = format!("{:e}", value.abs());
            let (significand, exponent) = text
                .split_once('e')
                .expect("scientific float representation");
            (
                significand.replace('.', ""),
                exponent.parse().expect("decimal exponent"),
            )
        } else {
            self.integer_work(span, |budget| {
                significant_digits(value.abs(), fraction as usize + 1, budget)
            })?
        };
        let mut result = String::new();
        if value < 0.0 {
            result.push('-');
        }
        append_exponential(&mut result, &digits, exponent);
        Ok(Value::String(JsString::from(result.as_str())))
    }
}

fn append_exponential(result: &mut String, digits: &str, exponent: i32) {
    result.push_str(&digits[..1]);
    if digits.len() > 1 {
        result.push('.');
        result.push_str(&digits[1..]);
    }
    result.push('e');
    if exponent >= 0 {
        result.push('+');
    }
    result.push_str(&exponent.to_string());
}

fn scaled_integer(value: f64, fraction: i32, budget: &mut Budget) -> Result<BigInt, IntegerError> {
    debug_assert!(value >= 0.0 && value.is_finite());
    if value == 0.0 {
        budget.charge(1)?;
        return Ok(BigInt::default());
    }
    let (numerator, denominator) = scaled_ratio(value, fraction, budget)?;
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

pub(super) fn components(value: f64) -> (i64, i32) {
    let bits = value.to_bits();
    let biased = ((bits >> 52) & 0x7ff) as i32;
    let significand = (bits & ((1u64 << 52) - 1)) | if biased == 0 { 0 } else { 1u64 << 52 };
    (
        significand as i64,
        if biased == 0 {
            -1074
        } else {
            biased - 1023 - 52
        },
    )
}

fn scaled_ratio(
    value: f64,
    fraction: i32,
    budget: &mut Budget,
) -> Result<(BigInt, BigInt), IntegerError> {
    let (significand, exponent) = components(value);
    // value * 10**fraction = significand * 5**fraction * 2**(exponent+fraction).
    // Cancel powers of two before constructing numerator and denominator.
    let power = BigInt::from(5).pow(&BigInt::from(i64::from(fraction.unsigned_abs())), budget)?;
    let mut numerator = BigInt::from(significand);
    let mut denominator = BigInt::from(1);
    if fraction >= 0 {
        numerator = numerator.mul(&power, budget)?;
    } else {
        denominator = power;
    }
    let binary = exponent + fraction;
    if binary >= 0 {
        numerator = numerator.shl(&BigInt::from(i64::from(binary)), budget)?;
    } else {
        denominator = denominator.shl(&BigInt::from(i64::from(-binary)), budget)?;
    }
    Ok((numerator, denominator))
}

fn compare_power(
    value: f64,
    exponent: i32,
    budget: &mut Budget,
) -> Result<std::cmp::Ordering, IntegerError> {
    let (numerator, denominator) = scaled_ratio(value, -exponent, budget)?;
    budget.charge(
        numerator
            .bit_length()
            .max(denominator.bit_length())
            .div_ceil(32),
    )?;
    Ok(numerator.cmp(&denominator))
}

fn significant_digits(
    value: f64,
    precision: usize,
    budget: &mut Budget,
) -> Result<(String, i32), IntegerError> {
    if value == 0.0 {
        budget.charge(precision)?;
        return Ok(("0".repeat(precision), 0));
    }
    let (significand, binary) = components(value);
    let binary_exponent = binary + 63 - significand.leading_zeros() as i32;
    // 1233/4096 approximates log10(2); exact comparisons correct the estimate.
    let mut exponent = (binary_exponent * 1233).div_euclid(4096);
    while compare_power(value, exponent, budget)?.is_lt() {
        exponent -= 1;
    }
    while !compare_power(value, exponent + 1, budget)?.is_lt() {
        exponent += 1;
    }
    let integer = scaled_integer(value, precision as i32 - 1 - exponent, budget)?;
    let mut digits = integer.to_radix(10, budget)?;
    if digits.len() > precision {
        debug_assert_eq!(digits.len(), precision + 1);
        debug_assert!(digits.ends_with('0'));
        digits.pop();
        exponent += 1;
    }
    debug_assert_eq!(digits.len(), precision);
    Ok((digits, exponent))
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
        realm.limits.max_string_units = Some(4);
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
