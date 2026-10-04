//! Array-producing copies without species lookup (23.1.3.33/39).

use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn array_to_reversed(
        &mut self,
        receiver: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(object) = self.box_primitive(receiver, span)? else {
            unreachable!("ToObject");
        };
        let length = self.length_of_array_like(&object, span)?;
        let array = self.create_intrinsic_array(length, span)?;
        for index in 0..length {
            self.tick(span)?;
            let from = JsString::from((length - index - 1).to_string().as_str());
            let value = self.get_property(&object, &from, span)?;
            // ArrayCreate has validated the u32 length bound. Get turns holes
            // into own undefined elements; inherited setters are bypassed.
            self.create_array_element(&array, index as u32, value, span)?;
        }
        Ok(Value::Object(array))
    }

    pub(crate) fn array_with(
        &mut self,
        receiver: Value,
        index: Value,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(object) = self.box_primitive(receiver, span)? else {
            unreachable!("ToObject");
        };
        let length = self.length_of_array_like(&object, span)?;
        let number = self.number(index, span)?;
        let relative = if number.is_nan() { 0.0 } else { number.trunc() };
        let actual = if relative >= 0.0 {
            relative
        } else {
            length as f64 + relative
        };
        if actual < 0.0 || actual >= length as f64 {
            return Err(Self::exception(
                ExceptionKind::RangeError,
                span,
                "Array replacement index is out of range",
            ));
        }
        let actual = actual as u64;
        let array = self.create_intrinsic_array(length, span)?;
        let mut replacement = Some(value);
        for index in 0..length {
            self.tick(span)?;
            let value = if index == actual {
                // The replaced source property is never read. Move the single
                // supplied replacement rather than cloning its payload.
                replacement.take().expect("one replacement index")
            } else {
                self.get_property(&object, &JsString::from(index.to_string().as_str()), span)?
            };
            self.create_array_element(&array, index as u32, value, span)?;
        }
        Ok(Value::Object(array))
    }
}
