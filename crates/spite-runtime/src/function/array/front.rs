//! Front removal/insertion with ordered sparse movement (23.1.3.27/37).

use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn array_shift(&mut self, receiver: Value, span: Span) -> Result<Value, Error> {
        let Value::Object(object) = self.box_primitive(receiver, span)? else {
            unreachable!("ToObject");
        };
        let length = self.length_of_array_like(&object, span)?;
        if length == 0 {
            self.set_property_or_throw(
                &object,
                JsString::from("length"),
                Value::Number(0.0),
                span,
            )?;
            return Ok(Value::Undefined);
        }
        // Retain the value read before any subsequent getters or writes.
        let first = self.get_property(&object, &JsString::from("0"), span)?;
        for from in 1..length {
            self.tick(span)?;
            self.copy_array_element(&object, from, from - 1, span)?;
        }
        self.delete_property_or_throw(
            &object,
            &JsString::from((length - 1).to_string().as_str()),
            span,
        )?;
        self.set_property_or_throw(
            &object,
            JsString::from("length"),
            Value::Number((length - 1) as f64),
            span,
        )?;
        Ok(first)
    }

    pub(crate) fn array_unshift(
        &mut self,
        receiver: Value,
        arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(object) = self.box_primitive(receiver, span)? else {
            unreachable!("ToObject");
        };
        let length = self.length_of_array_like(&object, span)?;
        let count = arguments.len() as u64;
        if count > 0 {
            if count > 9_007_199_254_740_991 - length {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "Array unshift exceeds the maximum safe length",
                ));
            }
            for from in (0..length).rev() {
                self.tick(span)?;
                self.copy_array_element(&object, from, from + count, span)?;
            }
            for (index, value) in arguments.enumerate() {
                self.tick(span)?;
                self.set_property_or_throw(
                    &object,
                    JsString::from(index.to_string().as_str()),
                    value,
                    span,
                )?;
            }
        }
        // Zero arguments skip indexed traversal, but still perform this Set.
        self.set_property_or_throw(
            &object,
            JsString::from("length"),
            Value::Number((length + count) as f64),
            span,
        )?;
        Ok(Value::Number((length + count) as f64))
    }
}
