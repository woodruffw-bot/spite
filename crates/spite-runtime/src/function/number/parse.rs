//! Numeric prefix parsing (19.2.4–5) and shared Number aliases (21.1.2.12–13).

use crate::{Error, Realm, Value, value::to_uint32};
use spite_bigint::BigInt;
use spite_core::{JsString, Span, is_line_terminator, is_whitespace};

#[cfg(test)]
mod tests;

impl Realm {
    fn numeric_input(&mut self, value: Value, span: Span) -> Result<JsString, Error> {
        let text = self.string(value, span)?;
        if text.len() > self.limits.max_string_units {
            return Err(Error::Limit {
                span,
                message: "string length limit exceeded".into(),
            });
        }
        self.integer_work(span, |budget| budget.charge(text.len() + 1))?;
        Ok(text)
    }

    pub(crate) fn parse_float(&mut self, value: Value, span: Span) -> Result<f64, Error> {
        let text = self.numeric_input(value, span)?;
        let units = trim_start(text.code_units());
        let negative = units.first() == Some(&0x2d);
        let mut pos = usize::from(matches!(units.first(), Some(0x2b | 0x2d)));
        if units[pos..].starts_with(&[0x49, 0x6e, 0x66, 0x69, 0x6e, 0x69, 0x74, 0x79]) {
            return Ok(if negative {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            });
        }
        let mut digits = 0;
        while units.get(pos).is_some_and(decimal_digit) {
            pos += 1;
            digits += 1;
        }
        if units.get(pos) == Some(&0x2e) {
            pos += 1;
            while units.get(pos).is_some_and(decimal_digit) {
                pos += 1;
                digits += 1;
            }
        }
        if digits == 0 {
            return Ok(f64::NAN);
        }
        let mut end = pos;
        if matches!(units.get(pos), Some(0x65 | 0x45)) {
            pos += 1;
            if matches!(units.get(pos), Some(0x2b | 0x2d)) {
                pos += 1;
            }
            let exponent_start = pos;
            while units.get(pos).is_some_and(decimal_digit) {
                pos += 1;
            }
            if pos > exponent_start {
                end = pos;
            }
        }
        // Only ASCII decimal syntax entered the selected prefix. A non-ASCII
        // unit, including a lone surrogate, only terminates that prefix.
        let prefix: String = units[..end]
            .iter()
            .map(|unit| char::from(*unit as u8))
            .collect();
        Ok(prefix.parse().expect("validated StrDecimalLiteral prefix"))
    }

    pub(crate) fn parse_int(
        &mut self,
        value: Value,
        radix: Value,
        span: Span,
    ) -> Result<f64, Error> {
        let text = self.numeric_input(value, span)?;
        let mut units = trim_start(text.code_units());
        let negative = units.first() == Some(&0x2d);
        if matches!(units.first(), Some(0x2b | 0x2d)) {
            units = &units[1..];
        }
        let mut radix = to_uint32(self.number(radix, span)?) as i32;
        if radix != 0 && !(2..=36).contains(&radix) {
            return Ok(f64::NAN);
        }
        let strip = radix == 0 || radix == 16;
        if radix == 0 {
            radix = 10;
        }
        if strip && units.first() == Some(&0x30) && matches!(units.get(1), Some(0x78 | 0x58)) {
            units = &units[2..];
            radix = 16;
        }
        let digits: String = units
            .iter()
            .map_while(|unit| {
                char::from_u32(u32::from(*unit))
                    .filter(|ch| ch.is_ascii() && ch.is_digit(radix as u32))
            })
            .collect();
        if digits.is_empty() {
            return Ok(f64::NAN);
        }
        let number = self.integer_work(span, |budget| {
            BigInt::parse_digits(&digits, radix as u32, budget)?.to_f64(budget)
        })?;
        Ok(if negative { -number } else { number })
    }
}

fn trim_start(units: &[u16]) -> &[u16] {
    let start = units
        .iter()
        .position(|unit| {
            !char::from_u32(u32::from(*unit))
                .is_some_and(|ch| is_whitespace(ch) || is_line_terminator(ch))
        })
        .unwrap_or(units.len());
    &units[start..]
}

fn decimal_digit(unit: &u16) -> bool {
    (0x30..=0x39).contains(unit)
}
