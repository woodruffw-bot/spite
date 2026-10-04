//! UTF-16 character construction/access (22.1.2.1–2, 22.1.3.1–4).

use super::Builtin;
use crate::{Error, ExceptionKind, Realm, Value, value::to_uint32};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn string_from_codes(
        &mut self,
        arguments: std::vec::IntoIter<Value>,
        code_points: bool,
        span: Span,
    ) -> Result<Value, Error> {
        let mut result = Vec::new();
        for argument in arguments {
            self.tick(span)?;
            let number = self.number(argument, span)?;
            let point = if code_points {
                if !number.is_finite()
                    || number.fract() != 0.0
                    || !(0.0..=0x10ffff as f64).contains(&number)
                {
                    return Err(Self::exception(
                        ExceptionKind::RangeError,
                        span,
                        "invalid Unicode code point",
                    ));
                }
                number as u32
            } else {
                // ToUint16 is the low 16 bits of the same modulo conversion.
                u32::from(to_uint32(number) as u16)
            };
            let (encoded, count) = if point <= 0xffff {
                ([point as u16, 0], 1)
            } else {
                let supplementary = point - 0x10000;
                (
                    [
                        0xd800 + (supplementary >> 10) as u16,
                        0xdc00 + (supplementary & 0x3ff) as u16,
                    ],
                    2,
                )
            };
            if result.len().checked_add(count).is_none_or(|length| {
                self.limits
                    .max_string_units
                    .is_some_and(|limit| length > limit)
            }) {
                return Err(Error::Limit {
                    span,
                    message: "string length limit exceeded".into(),
                });
            }
            result.extend_from_slice(&encoded[..count]);
        }
        Ok(Value::String(JsString::from_code_units(result)))
    }

    pub(crate) fn string_character(
        &mut self,
        builtin: Builtin,
        receiver: Value,
        position: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // These methods are generic: receiver ToString precedes index ToNumber.
        Self::require_object_coercible(&receiver, span)?;
        let string = self.string(receiver, span)?;
        let number = self.number(position, span)?;
        let relative = if number.is_nan() { 0.0 } else { number.trunc() };
        let index = if matches!(builtin, Builtin::StringAt) && relative < 0.0 {
            string.len() as f64 + relative
        } else {
            relative
        };
        if index < 0.0 || index >= string.len() as f64 {
            return Ok(match builtin {
                Builtin::StringCharAt => Value::String(JsString::from("")),
                Builtin::StringCharCodeAt => Value::Number(f64::NAN),
                _ => Value::Undefined,
            });
        }
        let index = index as usize;
        let first = string.code_units()[index];
        match builtin {
            Builtin::StringAt | Builtin::StringCharAt => {
                Ok(Value::String(JsString::from_code_units(vec![first])))
            }
            Builtin::StringCharCodeAt => Ok(Value::Number(f64::from(first))),
            Builtin::StringCodePointAt => {
                let mut point = u32::from(first);
                if (0xd800..=0xdbff).contains(&first) {
                    if let Some(&second) = string.code_units().get(index + 1) {
                        if (0xdc00..=0xdfff).contains(&second) {
                            point = 0x10000
                                + (u32::from(first) - 0xd800) * 0x400
                                + (u32::from(second) - 0xdc00);
                        }
                    }
                }
                Ok(Value::Number(f64::from(point)))
            }
            _ => unreachable!("String character builtin"),
        }
    }
}
