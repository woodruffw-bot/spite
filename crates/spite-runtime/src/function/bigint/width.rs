//! BigInt width conversions with ordered ToIndex/ToBigInt (21.2.2.1–2).

use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::Span;

impl Realm {
    pub(crate) fn bigint_width(
        &mut self,
        bits: Value,
        value: Value,
        signed: bool,
        span: Span,
    ) -> Result<Value, Error> {
        // ToIndex (7.1.22): truncate first, with NaN/undefined mapping to zero.
        // Width conversion completes before any coercion of the integer value.
        let number = self.number(bits, span)?;
        let bits = if number.is_nan() { 0.0 } else { number.trunc() };
        if !(0.0..=9_007_199_254_740_991.0).contains(&bits) {
            return Err(Self::exception(
                ExceptionKind::RangeError,
                span,
                "BigInt width is outside the safe integer range",
            ));
        }
        let value = self.bigint(value, span)?;
        self.integer_work(span, |budget| {
            if signed {
                value.as_int_n(bits as u64, budget)
            } else {
                value.as_uint_n(bits as u64, budget)
            }
        })
        .map(Value::BigInt)
    }
}
