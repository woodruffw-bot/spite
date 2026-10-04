use spite_bigint::{BigInt, Budget, Error as IntegerError};
use spite_core::{JsString, JsSymbol, is_line_terminator, is_whitespace, parse_radix_integer};
use spite_heap::{Handle, Trace};
use std::{cmp::Ordering, fmt};

/// A conversion that failed or needs access to an object's realm.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConversionError {
    /// ToNumber rejects a BigInt with a JavaScript TypeError.
    BigIntToNumber,
    /// ToNumber rejects a Symbol with a JavaScript TypeError.
    SymbolToNumber,
    /// Implicit ToString rejects a Symbol with a JavaScript TypeError.
    SymbolToString,
    /// Object conversion requires ToPrimitive and callable hooks in the realm.
    ObjectNeedsContext,
    /// Integer conversion exhausted a host limit or failed arithmetic validation.
    Integer(IntegerError),
}

impl From<IntegerError> for ConversionError {
    fn from(error: IntegerError) -> Self {
        Self::Integer(error)
    }
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BigIntToNumber => f.write_str("cannot convert BigInt to Number"),
            Self::SymbolToNumber => f.write_str("cannot convert Symbol to Number"),
            Self::SymbolToString => f.write_str("cannot convert Symbol to String"),
            Self::ObjectNeedsContext => f.write_str("object conversion requires realm context"),
            Self::Integer(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for ConversionError {}

/// A supported ECMAScript value.
///
/// Object handles are unrooted; retain a host root across explicit collection.
/// Symbol values preserve identity across realms and host threads.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// The undefined value.
    Undefined,
    /// The null value.
    Null,
    /// A Boolean value.
    Boolean(bool),
    /// A binary64 Number, including signed zero, infinities, and NaN.
    Number(f64),
    /// An arbitrary-precision signed integer, distinct from Number.
    BigInt(BigInt),
    /// A sequence of UTF-16 code units.
    String(JsString),
    /// An immutable Symbol identity and its optional description.
    Symbol(JsSymbol),
    /// Identity of an object owned by a realm's heap.
    Object(Handle),
}

impl Trace for Value {
    fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        // Keep this exhaustive: adding a reference-bearing value must also add
        // its outgoing edge here. Primitive fields still consume scan work.
        std::iter::once(match self {
            Self::Undefined
            | Self::Null
            | Self::Boolean(_)
            | Self::Number(_)
            | Self::BigInt(_)
            | Self::String(_)
            | Self::Symbol(_) => None,
            Self::Object(handle) => Some(handle),
        })
    }
}

impl Value {
    /// Applies ToBoolean; every object is truthy in this non-browser host.
    pub fn to_boolean(&self) -> bool {
        match self {
            Self::Undefined | Self::Null => false,
            Self::Boolean(v) => *v,
            Self::Number(v) => *v != 0.0 && !v.is_nan(),
            Self::BigInt(v) => !v.is_zero(),
            Self::String(v) => !v.is_empty(),
            Self::Object(_) | Self::Symbol(_) => true,
        }
    }

    /// Applies primitive ToNumber; objects explicitly require realm context.
    /// BigInt requires a TypeError, as specified by ECMA-262 7.1.4.
    pub fn to_number(&self) -> Result<f64, ConversionError> {
        Ok(match self {
            Self::Undefined => f64::NAN,
            Self::Null => 0.0,
            Self::Boolean(v) => u8::from(*v) as f64,
            Self::Number(v) => *v,
            Self::BigInt(_) => return Err(ConversionError::BigIntToNumber),
            Self::Symbol(_) => return Err(ConversionError::SymbolToNumber),
            Self::String(v) => string_to_number(v),
            Self::Object(_) => return Err(ConversionError::ObjectNeedsContext),
        })
    }

    /// Applies primitive ToString; objects explicitly require realm context.
    /// Integer formatting consumes the supplied arithmetic work budget.
    pub fn to_js_string(&self, budget: &mut Budget) -> Result<JsString, ConversionError> {
        Ok(match self {
            Self::Undefined => JsString::from("undefined"),
            Self::Null => JsString::from("null"),
            Self::Boolean(v) => JsString::from(if *v { "true" } else { "false" }),
            Self::Number(v) => JsString::from(number_to_string(*v).as_str()),
            Self::BigInt(v) => JsString::from(v.to_radix(10, budget)?.as_str()),
            Self::String(v) => v.clone(),
            Self::Symbol(_) => return Err(ConversionError::SymbolToString),
            Self::Object(_) => return Err(ConversionError::ObjectNeedsContext),
        })
    }

    /// Implements IsStrictlyEqual for the supported values.
    pub fn strictly_equal(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Undefined, Self::Undefined) | (Self::Null, Self::Null) => true,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::BigInt(a), Self::BigInt(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Symbol(a), Self::Symbol(b)) => a == b,
            (Self::Object(a), Self::Object(b)) => a == b,
            _ => false,
        }
    }

    /// Implements SameValue, distinguishing signed zeros and equating NaNs.
    pub fn same_value(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => {
                a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan())
            }
            _ => self.strictly_equal(other),
        }
    }

    pub(crate) fn loosely_equal(
        &self,
        other: &Self,
        budget: &mut Budget,
    ) -> Result<bool, ConversionError> {
        Ok(match (self, other) {
            (Self::Null, Self::Undefined) | (Self::Undefined, Self::Null) => true,
            (Self::Number(a), Self::String(b)) => *a == string_to_number(b),
            (Self::String(a), Self::Number(b)) => string_to_number(a) == *b,
            (Self::BigInt(a), Self::String(b)) | (Self::String(b), Self::BigInt(a)) => {
                string_to_bigint(b, budget)?.is_some_and(|b| *a == b)
            }
            (Self::BigInt(a), Self::Number(b)) | (Self::Number(b), Self::BigInt(a)) => {
                a.cmp_f64(*b, budget)? == Some(Ordering::Equal)
            }
            (Self::Boolean(a), _) => {
                Self::Number(u8::from(*a) as f64).loosely_equal(other, budget)?
            }
            (_, Self::Boolean(b)) => {
                self.loosely_equal(&Self::Number(u8::from(*b) as f64), budget)?
            }
            (
                Self::Object(_),
                Self::String(_) | Self::Number(_) | Self::BigInt(_) | Self::Symbol(_),
            )
            | (
                Self::String(_) | Self::Number(_) | Self::BigInt(_) | Self::Symbol(_),
                Self::Object(_),
            ) => {
                return Err(ConversionError::ObjectNeedsContext);
            }
            _ => self.strictly_equal(other),
        })
    }

    // ECMA-262 7.2.12. Primitive conversion has no user-observable side effects.
    pub(crate) fn compare(
        &self,
        other: &Self,
        budget: &mut Budget,
    ) -> Result<Option<Ordering>, ConversionError> {
        Ok(match (self, other) {
            (Self::Object(_), _) | (_, Self::Object(_)) => {
                return Err(ConversionError::ObjectNeedsContext);
            }
            (Self::String(a), Self::String(b)) => Some(a.cmp(b)),
            (Self::BigInt(a), Self::String(b)) => string_to_bigint(b, budget)?.map(|b| a.cmp(&b)),
            (Self::String(a), Self::BigInt(b)) => string_to_bigint(a, budget)?.map(|a| a.cmp(b)),
            (Self::BigInt(a), Self::BigInt(b)) => Some(a.cmp(b)),
            (Self::BigInt(a), b) => a.cmp_f64(b.to_number()?, budget)?,
            (a, Self::BigInt(b)) => b.cmp_f64(a.to_number()?, budget)?.map(Ordering::reverse),
            (a, b) => a.to_number()?.partial_cmp(&b.to_number()?),
        })
    }

    pub(crate) fn type_name(&self) -> &'static str {
        match self {
            Self::Undefined => "undefined",
            Self::Null => "object",
            Self::Boolean(_) => "boolean",
            Self::Number(_) => "number",
            Self::BigInt(_) => "bigint",
            Self::String(_) => "string",
            Self::Symbol(_) => "symbol",
            Self::Object(_) => "object",
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(value) => write!(f, "{value:?}"),
            // Host diagnostics use an exact, linear-time representation. Language
            // ToString remains decimal and uses the evaluator's arithmetic budget.
            Self::BigInt(value) => write!(f, "{value:#x}n"),
            Self::Number(value) => f.write_str(&number_to_string(*value)),
            Self::Undefined => f.write_str("undefined"),
            Self::Null => f.write_str("null"),
            Self::Boolean(value) => write!(f, "{value}"),
            Self::Object(handle) => write!(f, "Object({handle:?})"),
            Self::Symbol(symbol) => write!(f, "{symbol:?}"),
        }
    }
}

// ECMA-262 7.1.14: signed decimal integers, unsigned radix prefixes, and empty
// whitespace are accepted. Separators, suffixes, fractions, and exponents are not.
pub(crate) fn string_to_bigint(
    value: &JsString,
    budget: &mut Budget,
) -> Result<Option<BigInt>, IntegerError> {
    budget.charge(value.len().checked_add(1).ok_or(IntegerError::Limit)?)?;
    let Ok(text) = value.to_utf8() else {
        return Ok(None);
    };
    let text = text.trim_matches(|c| is_whitespace(c) || is_line_terminator(c));
    if text.is_empty() {
        return Ok(Some(BigInt::default()));
    }
    for (prefix, radix) in [
        ("0x", 16),
        ("0X", 16),
        ("0o", 8),
        ("0O", 8),
        ("0b", 2),
        ("0B", 2),
    ] {
        if let Some(digits) = text.strip_prefix(prefix) {
            if digits.is_empty() || !digits.bytes().all(|b| char::from(b).is_digit(radix)) {
                return Ok(None);
            }
            return BigInt::parse_digits(digits, radix, budget).map(Some);
        }
    }
    let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Ok(None);
    }
    let value = BigInt::parse_digits(digits, 10, budget)?;
    Ok(Some(if text.starts_with('-') {
        value.neg(budget)?
    } else {
        value
    }))
}

// https://262.ecma-international.org/17.0/#sec-tonumber-applied-to-the-string-type
// StringNumericLiteral differs from source NumericLiteral: no separators,
// no BigInt, no legacy octal, and no signed radix prefixes.
fn string_to_number(value: &JsString) -> f64 {
    let Ok(text) = value.to_utf8() else {
        return f64::NAN;
    };
    let text = text.trim_matches(|c| is_whitespace(c) || is_line_terminator(c));
    if text.is_empty() {
        return 0.0;
    }
    match text {
        "Infinity" | "+Infinity" => return f64::INFINITY,
        "-Infinity" => return f64::NEG_INFINITY,
        _ => {}
    }
    for (prefix, radix) in [
        ("0x", 16),
        ("0X", 16),
        ("0o", 8),
        ("0O", 8),
        ("0b", 2),
        ("0B", 2),
    ] {
        if let Some(digits) = text.strip_prefix(prefix) {
            return parse_radix_integer(digits, radix).unwrap_or(f64::NAN);
        }
    }
    let bytes = text.as_bytes();
    let mut pos = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let mut digits = 0;
    while bytes.get(pos).is_some_and(u8::is_ascii_digit) {
        pos += 1;
        digits += 1;
    }
    if bytes.get(pos) == Some(&b'.') {
        pos += 1;
        while bytes.get(pos).is_some_and(u8::is_ascii_digit) {
            pos += 1;
            digits += 1;
        }
    }
    if digits == 0 {
        return f64::NAN;
    }
    if matches!(bytes.get(pos), Some(b'e' | b'E')) {
        pos += 1;
        if matches!(bytes.get(pos), Some(b'+' | b'-')) {
            pos += 1;
        }
        let start = pos;
        while bytes.get(pos).is_some_and(u8::is_ascii_digit) {
            pos += 1;
        }
        if pos == start {
            return f64::NAN;
        }
    }
    if pos != bytes.len() {
        return f64::NAN;
    }
    text.parse().unwrap_or(f64::NAN)
}

// https://262.ecma-international.org/17.0/#sec-numeric-types-number-tostring
// Number::toString decimal presentation. Rust supplies the shortest round-trip
// significand. ECMA-262 specifies different fixed/scientific cutoffs and signs.
pub(crate) fn number_to_string(value: f64) -> String {
    if value.is_nan() {
        return "NaN".into();
    }
    if value == 0.0 {
        return "0".into();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-Infinity"
        } else {
            "Infinity"
        }
        .into();
    }
    let sign = if value.is_sign_negative() { "-" } else { "" };
    let raw = value.abs().to_string();
    let (mantissa, exponent) = raw
        .split_once(['e', 'E'])
        .map_or((raw.as_str(), 0), |(m, e)| {
            (m, e.parse::<i32>().unwrap_or(0))
        });
    let point = mantissa.find('.').unwrap_or(mantissa.len()) as i32;
    let all: String = mantissa.chars().filter(|c| *c != '.').collect();
    let leading = all.bytes().take_while(|c| *c == b'0').count();
    let digits = all[leading..].trim_end_matches('0');
    let n = point + exponent - leading as i32;
    let k = digits.len() as i32;
    if k <= n && n <= 21 {
        format!("{sign}{digits}{}", "0".repeat((n - k) as usize))
    } else if 0 < n && n <= 21 {
        format!("{sign}{}.{}", &digits[..n as usize], &digits[n as usize..])
    } else if -6 < n && n <= 0 {
        format!("{sign}0.{}{digits}", "0".repeat((-n) as usize))
    } else {
        let fraction = if digits.len() == 1 {
            String::new()
        } else {
            format!(".{}", &digits[1..])
        };
        format!("{sign}{}{fraction}e{:+}", &digits[..1], n - 1)
    }
}

// https://262.ecma-international.org/17.0/#sec-touint32
pub(crate) fn to_uint32(value: f64) -> u32 {
    if !value.is_finite() || value == 0.0 {
        return 0;
    }
    let n = value.trunc() % 4294967296.0;
    (if n < 0.0 { n + 4294967296.0 } else { n }) as u32
}

// https://262.ecma-international.org/17.0/#sec-numeric-types-number-exponentiate
pub(crate) fn exponentiate(base: f64, exponent: f64) -> f64 {
    if exponent.is_nan() {
        return f64::NAN;
    }
    if exponent == 0.0 {
        return 1.0;
    }
    if base.is_nan() {
        return f64::NAN;
    }
    let odd = exponent.is_finite()
        && exponent.fract() == 0.0
        && exponent.abs() < 9007199254740992.0
        && exponent % 2.0 != 0.0;
    if base.is_infinite() {
        let magnitude = if exponent > 0.0 { f64::INFINITY } else { 0.0 };
        return if base.is_sign_negative() && odd {
            -magnitude
        } else {
            magnitude
        };
    }
    if base == 0.0 {
        let magnitude = if exponent > 0.0 { 0.0 } else { f64::INFINITY };
        return if base.is_sign_negative() && odd {
            -magnitude
        } else {
            magnitude
        };
    }
    if exponent.is_infinite() {
        if base.abs() == 1.0 {
            return f64::NAN;
        }
        return if (base.abs() > 1.0) == (exponent > 0.0) {
            f64::INFINITY
        } else {
            0.0
        };
    }
    if base < 0.0 && exponent.fract() != 0.0 {
        return f64::NAN;
    }
    base.powf(exponent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::Objects;

    #[test]
    fn object_equality_only_requests_conversion_when_required() {
        let mut objects = Objects::new(2, 0);
        let a = Value::Object(objects.create(None).unwrap());
        let b = Value::Object(objects.create(None).unwrap());
        let mut budget = Budget::new(100, 100);
        assert_eq!(a.loosely_equal(&a, &mut budget), Ok(true));
        for other in [b, Value::Null, Value::Undefined] {
            assert_eq!(a.loosely_equal(&other, &mut budget), Ok(false));
            assert_eq!(other.loosely_equal(&a, &mut budget), Ok(false));
        }
        for other in [
            Value::Boolean(false),
            Value::String(JsString::from("")),
            Value::Number(0.0),
            Value::BigInt(BigInt::from(0)),
        ] {
            assert_eq!(
                a.loosely_equal(&other, &mut budget),
                Err(ConversionError::ObjectNeedsContext)
            );
            assert_eq!(
                other.loosely_equal(&a, &mut budget),
                Err(ConversionError::ObjectNeedsContext)
            );
        }
        assert_eq!(
            a.compare(&a, &mut budget),
            Err(ConversionError::ObjectNeedsContext)
        );
    }
}
