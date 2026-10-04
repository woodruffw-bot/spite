//! Generic relative indexed access (23.1.3.1).

use crate::{Error, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn array_at(
        &mut self,
        receiver: Value,
        index: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(object) = self.box_primitive(receiver, span)? else {
            unreachable!("ToObject");
        };
        let length = self.length_of_array_like(&object, span)?;
        let number = self.number(index, span)?;
        let relative = if number.is_nan() { 0.0 } else { number.trunc() };
        let index = if relative >= 0.0 {
            relative
        } else {
            length as f64 + relative
        };
        if index < 0.0 || index >= length as f64 {
            return Ok(Value::Undefined);
        }
        // LengthOfArrayLike bounds this integral index below 2^53 - 1.
        self.get_property(
            &object,
            &JsString::from((index as u64).to_string().as_str()),
            span,
        )
    }
}
