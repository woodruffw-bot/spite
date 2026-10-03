use spite_core::{JsString, is_line_terminator, is_whitespace, parse_radix_integer};
use std::fmt;

/// A supported ECMAScript primitive value.
///
/// BigInt, Symbol, and Object values are not implemented yet.
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
    /// A sequence of UTF-16 code units.
    String(JsString),
}

impl Value {
    /// Applies ToBoolean to the supported primitive types.
    pub fn to_boolean(&self) -> bool {
        match self {
            Self::Undefined | Self::Null => false,
            Self::Boolean(v) => *v,
            Self::Number(v) => *v != 0.0 && !v.is_nan(),
            Self::String(v) => !v.is_empty(),
        }
    }

    /// Applies ToNumber to the supported primitive types.
    pub fn to_number(&self) -> f64 {
        match self {
            Self::Undefined => f64::NAN,
            Self::Null => 0.0,
            Self::Boolean(v) => u8::from(*v) as f64,
            Self::Number(v) => *v,
            Self::String(v) => string_to_number(v),
        }
    }

    /// Applies ToString to the supported primitive types.
    pub fn to_js_string(&self) -> JsString {
        match self {
            Self::Undefined => JsString::from("undefined"),
            Self::Null => JsString::from("null"),
            Self::Boolean(v) => JsString::from(if *v { "true" } else { "false" }),
            Self::Number(v) => JsString::from(number_to_string(*v).as_str()),
            Self::String(v) => v.clone(),
        }
    }

    /// Implements IsStrictlyEqual for the supported values.
    pub fn strictly_equal(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Undefined, Self::Undefined) | (Self::Null, Self::Null) => true,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
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

    pub(crate) fn loosely_equal(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Undefined) | (Self::Undefined, Self::Null) => true,
            (Self::Number(a), Self::String(_)) => *a == other.to_number(),
            (Self::String(_), Self::Number(b)) => self.to_number() == *b,
            (Self::Boolean(_), _) => Self::Number(self.to_number()).loosely_equal(other),
            (_, Self::Boolean(_)) => self.loosely_equal(&Self::Number(other.to_number())),
            _ => self.strictly_equal(other),
        }
    }

    pub(crate) fn type_name(&self) -> &'static str {
        match self {
            Self::Undefined => "undefined",
            Self::Null => "object",
            Self::Boolean(_) => "boolean",
            Self::Number(_) => "number",
            Self::String(_) => "string",
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(value) => write!(f, "{value:?}"),
            value => f.write_str(&value.to_js_string().to_utf8().map_err(|_| fmt::Error)?),
        }
    }
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
fn number_to_string(value: f64) -> String {
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
