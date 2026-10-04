//! UTF-16 substring search and predicates (22.1.3.7–9/11/24, 6.1.4.1–2).

use super::Builtin;
use crate::{Error, Realm, Value};
use spite_core::{JsString, Span};

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
        let start = clamp_position(position, string.len());
        let result = self.find_string(&string, &search, start, backwards, span)?;
        Ok(Value::Number(result.map_or(-1.0, |index| index as f64)))
    }

    pub(crate) fn string_search_predicate(
        &mut self,
        builtin: Builtin,
        receiver: Value,
        search: Value,
        position: Value,
        span: Span,
    ) -> Result<Value, Error> {
        Self::require_object_coercible(&receiver, span)?;
        let string = self.string(receiver, span)?;
        // IsRegExp (7.2.6) is false for all currently exposed values: neither
        // Symbol keys nor RegExpMatcher slots exist. Their lookup/rejection must
        // join here, before search ToString, when those facilities are exposed.
        let search = self.string(search, span)?;
        let position =
            if matches!(builtin, Builtin::StringEndsWith) && matches!(position, Value::Undefined) {
                string.len()
            } else {
                let number = self.number(position, span)?;
                let integer = if number.is_nan() { 0.0 } else { number.trunc() };
                clamp_position(integer, string.len())
            };
        if matches!(builtin, Builtin::StringIncludes) {
            return self
                .find_string(&string, &search, position, false, span)
                .map(|result| Value::Boolean(result.is_some()));
        }
        let range = if matches!(builtin, Builtin::StringEndsWith) {
            position
                .checked_sub(search.len())
                .map(|start| start..position)
        } else {
            position
                .checked_add(search.len())
                .filter(|&end| end <= string.len())
                .map(|end| position..end)
        };
        let Some(range) = range else {
            return Ok(Value::Boolean(false));
        };
        self.object_work(span, |_, budget| budget.charge(search.len()))?;
        Ok(Value::Boolean(
            &string.code_units()[range] == search.code_units(),
        ))
    }

    fn find_string(
        &mut self,
        string: &JsString,
        search: &JsString,
        start: usize,
        backwards: bool,
        span: Span,
    ) -> Result<Option<usize>, Error> {
        let Some(last) = string.len().checked_sub(search.len()) else {
            return Ok(None);
        };
        let start = if backwards { start.min(last) } else { start };
        if search.is_empty() {
            return Ok(Some(start));
        }
        if start > last {
            return Ok(None);
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
                return Ok(Some(index));
            }
        }
        Ok(None)
    }
}

fn clamp_position(position: f64, length: usize) -> usize {
    (position.max(0.0).min(length as f64) as usize).min(length)
}
