//! Canonical equivalence and fixed host collation without ECMA-402 (22.1.3.10).

use super::normalize::Form;
use crate::{Error, Realm, Value};
use spite_core::Span;
use std::cmp::Ordering;

impl Realm {
    pub(crate) fn string_locale_compare(
        &mut self,
        receiver: Value,
        that: Value,
        span: Span,
    ) -> Result<Value, Error> {
        Self::require_object_coercible(&receiver, span)?;
        let string = self.string(receiver, span)?;
        let that = self.string(that, span)?;
        // ECMA-262 requires canonical equivalence even without ECMA-402.
        // The host's fixed locale-neutral ordering compares NFD UTF-16 units.
        // Invoke the retained native algorithm, never a public normalize hook.
        let string = self.normalize_string(&string, Form::Nfd, span)?;
        let that = self.normalize_string(&that, Form::Nfd, span)?;
        for (left, right) in string.code_units().iter().zip(that.code_units()) {
            self.tick(span)?;
            match left.cmp(right) {
                Ordering::Less => return Ok(Value::Number(-1.0)),
                Ordering::Greater => return Ok(Value::Number(1.0)),
                Ordering::Equal => {}
            }
        }
        Ok(Value::Number(
            match string.code_units().len().cmp(&that.code_units().len()) {
                Ordering::Less => -1.0,
                Ordering::Equal => 0.0,
                Ordering::Greater => 1.0,
            },
        ))
    }
}
