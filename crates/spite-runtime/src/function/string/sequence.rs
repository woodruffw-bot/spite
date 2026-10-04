//! UTF-16 concatenation and substrings (22.1.3.5/22/25).

use crate::{Error, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn string_concat(
        &mut self,
        receiver: Value,
        arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        Self::require_object_coercible(&receiver, span)?;
        let string = self.string(receiver, span)?;
        let mut result = Vec::new();
        self.object_work(span, |_, budget| budget.charge(string.len()))?;
        self.append_string(&mut result, &string, span)?;
        for argument in arguments {
            self.tick(span)?;
            let part = self.string(argument, span)?;
            self.object_work(span, |_, budget| budget.charge(part.len()))?;
            self.append_string(&mut result, &part, span)?;
        }
        Ok(Value::String(JsString::from_code_units(result)))
    }

    pub(crate) fn string_substring(
        &mut self,
        receiver: Value,
        start: Value,
        end: Value,
        relative: bool,
        span: Span,
    ) -> Result<Value, Error> {
        Self::require_object_coercible(&receiver, span)?;
        let string = self.string(receiver, span)?;
        let mut from = position(self.number(start, span)?, string.len(), relative);
        // Undefined is special, including an explicitly supplied undefined.
        // Even an empty input must observe both argument conversions in order.
        let mut to = if matches!(end, Value::Undefined) {
            string.len()
        } else {
            position(self.number(end, span)?, string.len(), relative)
        };
        if from > to {
            if relative {
                to = from;
            } else {
                std::mem::swap(&mut from, &mut to);
            }
        }
        let count = to - from;
        if count > self.limits.max_string_units {
            return Err(Error::Limit {
                span,
                message: "string length limit exceeded".into(),
            });
        }
        self.object_work(span, |_, budget| budget.charge(count))?;
        Ok(Value::String(JsString::from_code_units(
            string.code_units()[from..to].to_vec(),
        )))
    }
}

fn position(number: f64, length: usize, relative: bool) -> usize {
    let integer = if number.is_nan() { 0.0 } else { number.trunc() };
    let index = if relative && integer < 0.0 {
        length as f64 + integer
    } else {
        integer
    };
    (index.max(0.0).min(length as f64) as usize).min(length)
}
