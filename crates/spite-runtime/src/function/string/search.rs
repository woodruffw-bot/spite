//! UTF-16 substring search (22.1.3.9/11, 6.1.4.1–2).

use crate::{Error, Realm, Value};
use spite_core::Span;

impl Realm {
    pub(crate) fn string_index_of(
        &mut self,
        receiver: Value,
        search: Value,
        position: Value,
        backwards: bool,
        span: Span,
    ) -> Result<Value, Error> {
        Self::require_object_coercible(&receiver, span)?;
        let string = self.string(receiver, span)?;
        let search = self.string(search, span)?;
        let number = self.number(position, span)?;
        // lastIndexOf explicitly maps NaN (including undefined) to +Infinity;
        // indexOf instead uses ToIntegerOrInfinity's zero result.
        let position = if number.is_nan() {
            if backwards { f64::INFINITY } else { 0.0 }
        } else {
            number.trunc()
        };
        let Some(last) = string.len().checked_sub(search.len()) else {
            return Ok(Value::Number(-1.0));
        };
        let bound = if backwards { last } else { string.len() };
        let start = (position.max(0.0).min(bound as f64) as usize).min(bound);
        if search.is_empty() {
            return Ok(Value::Number(start as f64));
        }
        if start > last {
            return Ok(Value::Number(-1.0));
        }
        let count = if backwards {
            start + 1
        } else {
            last - start + 1
        };
        for offset in 0..count {
            let index = if backwards {
                start - offset
            } else {
                start + offset
            };
            // The simple search is potentially quadratic. Charge each candidate
            // comparison before accessing its units, including failed matches.
            self.object_work(span, |_, budget| budget.charge(search.len()))?;
            if &string.code_units()[index..index + search.len()] == search.code_units() {
                return Ok(Value::Number(index as f64));
            }
        }
        Ok(Value::Number(-1.0))
    }
}
