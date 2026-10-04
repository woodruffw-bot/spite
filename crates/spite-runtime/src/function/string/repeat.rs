//! String repetition and padding (22.1.3.16–18).

use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn string_repeat(
        &mut self,
        receiver: Value,
        count: Value,
        span: Span,
    ) -> Result<Value, Error> {
        Self::require_object_coercible(&receiver, span)?;
        let string = self.string(receiver, span)?;
        let number = self.number(count, span)?;
        let count = if number.is_nan() { 0.0 } else { number.trunc() };
        if count < 0.0 || count == f64::INFINITY {
            return Err(Self::exception(
                ExceptionKind::RangeError,
                span,
                "invalid string repetition count",
            ));
        }
        // Invalid counts still throw for empty strings. Valid finite counts,
        // however large, repeat an empty string without a loop or allocation.
        if count == 0.0 || string.is_empty() {
            return Ok(Value::String(JsString::from("")));
        }
        if count >= usize::MAX as f64 {
            return Err(length_limit(span));
        }
        let count = count as usize;
        let length = string
            .len()
            .checked_mul(count)
            .ok_or_else(|| length_limit(span))?;
        let mut result = self.repeated_string_buffer(length, span)?;
        for _ in 0..count {
            result.extend_from_slice(string.code_units());
        }
        Ok(Value::String(JsString::from_code_units(result)))
    }

    pub(crate) fn string_pad(
        &mut self,
        receiver: Value,
        max_length: Value,
        fill: Value,
        at_start: bool,
        span: Span,
    ) -> Result<Value, Error> {
        Self::require_object_coercible(&receiver, span)?;
        let string = self.string(receiver, span)?;
        let number = self.number(max_length, span)?;
        // ToLength (7.1.20): NaN/negatives become zero; +Infinity is clamped.
        let length = if number.is_nan() || number <= 0.0 {
            0.0
        } else {
            number.trunc().min(9_007_199_254_740_991.0)
        };
        if length <= string.len() as f64 {
            return Ok(Value::String(string));
        }
        // The short-circuit above precedes fill conversion. An empty converted
        // fill returns S even if maxLength exceeds host allocation limits.
        let fill = match fill {
            Value::Undefined => JsString::from(" "),
            value => self.string(value, span)?,
        };
        if fill.is_empty() {
            return Ok(Value::String(string));
        }
        if length >= usize::MAX as f64 {
            return Err(length_limit(span));
        }
        let length = length as usize;
        let fill_length = length - string.len();
        let mut result = self.repeated_string_buffer(length, span)?;
        if !at_start {
            result.extend_from_slice(string.code_units());
        }
        result.extend(fill.code_units().iter().copied().cycle().take(fill_length));
        if at_start {
            result.extend_from_slice(string.code_units());
        }
        Ok(Value::String(JsString::from_code_units(result)))
    }

    fn repeated_string_buffer(&mut self, length: usize, span: Span) -> Result<Vec<u16>, Error> {
        if length > self.limits.max_string_units {
            return Err(length_limit(span));
        }
        self.object_work(span, |_, budget| budget.charge(length))?;
        let mut result = Vec::new();
        // Checked reservation also rejects platform capacity overflow when an
        // embedding disables work accounting and raises the output limit.
        result
            .try_reserve_exact(length)
            .map_err(|_| length_limit(span))?;
        Ok(result)
    }
}

fn length_limit(span: Span) -> Error {
    Error::Limit {
        span,
        message: "string allocation limit exceeded".into(),
    }
}
