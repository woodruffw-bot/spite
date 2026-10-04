//! Array-producing copies without species lookup (23.1.3.33/35/39).

use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn array_to_spliced(
        &mut self,
        receiver: Value,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(object) = self.box_primitive(receiver, span)? else {
            unreachable!("ToObject");
        };
        let length = self.length_of_array_like(&object, span)?;
        let start = arguments.next();
        let has_start = start.is_some();
        let start = self.array_relative_index(start.unwrap_or(Value::Undefined), length, span)?;
        let skip = match arguments.next() {
            Some(value) => {
                let number = self.number(value, span)?;
                if number.is_nan() {
                    0
                } else {
                    number.trunc().max(0.0).min((length - start) as f64) as u64
                }
            }
            None if has_start => length - start,
            None => 0,
        };
        let retained = length - skip;
        let insert_count = arguments.len() as u64;
        // 23.1.3.35: the safe-integer check precedes ArrayCreate's u32 bound.
        // Subtract first so length arithmetic stays exact and cannot overflow.
        if insert_count > 9_007_199_254_740_991 - retained {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Array copy exceeds the maximum safe integer length",
            ));
        }
        let array = self.create_intrinsic_array(retained + insert_count, span)?;
        let mut target = 0;
        for from in 0..start {
            self.tick(span)?;
            let value =
                self.get_property(&object, &JsString::from(from.to_string().as_str()), span)?;
            self.create_array_element(&array, target, value, span)?;
            target += 1;
        }
        for value in arguments {
            self.tick(span)?;
            self.create_array_element(&array, target, value, span)?;
            target += 1;
        }
        // Discarded elements are never read. Source indices retain the full
        // ToLength range even when deleting most of a huge array-like object.
        for from in start + skip..length {
            self.tick(span)?;
            let value =
                self.get_property(&object, &JsString::from(from.to_string().as_str()), span)?;
            self.create_array_element(&array, target, value, span)?;
            target += 1;
        }
        Ok(Value::Object(array))
    }

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
            self.create_array_element(&array, index, value, span)?;
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
            self.create_array_element(&array, index, value, span)?;
        }
        Ok(Value::Object(array))
    }
}
