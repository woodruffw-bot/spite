//! Species-aware deletion and insertion with ordered sparse moves (23.1.3.31).

use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn array_splice(
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
        let delete_count = match arguments.next() {
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
        let insert_count = arguments.len() as u64;
        let retained = length - delete_count;
        // Subtract first to preserve exact full-width length arithmetic. This
        // check precedes species construction and all indexed source reads.
        if insert_count > 9_007_199_254_740_991 - retained {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Array splice exceeds the maximum safe integer length",
            ));
        }
        let new_length = retained + insert_count;
        let result = self.array_species_create(&object, delete_count, span)?;
        for index in 0..delete_count {
            self.tick(span)?;
            let key = JsString::from((start + index).to_string().as_str());
            if self.has_property(&object, &key, span)? {
                let value = self.get_property(&object, &key, span)?;
                self.create_array_element(&result, index, value, span)?;
            }
        }
        // Species may return the source itself. Finish these definitions and
        // this strict Set before any moves, then read the source live below.
        self.set_property_or_throw(
            &result,
            JsString::from("length"),
            Value::Number(delete_count as f64),
            span,
        )?;
        if insert_count < delete_count {
            for index in start..length - delete_count {
                self.tick(span)?;
                self.copy_array_element(&object, index + delete_count, index + insert_count, span)?;
            }
            // Delete obsolete indices from highest to lowest. A failed delete
            // keeps every preceding copy/delete; there is no rollback.
            for index in (new_length..length).rev() {
                self.tick(span)?;
                self.delete_property_or_throw(
                    &object,
                    &JsString::from(index.to_string().as_str()),
                    span,
                )?;
            }
        } else if insert_count > delete_count {
            for index in (start..length - delete_count).rev() {
                self.tick(span)?;
                self.copy_array_element(&object, index + delete_count, index + insert_count, span)?;
            }
        }
        for (offset, value) in arguments.enumerate() {
            self.tick(span)?;
            self.set_property_or_throw(
                &object,
                JsString::from((start + offset as u64).to_string().as_str()),
                value,
                span,
            )?;
        }
        // Even zero arguments perform this Set, observing setters and rejecting
        // a read-only length. Arrays can throw after earlier indexed writes.
        self.set_property_or_throw(
            &object,
            JsString::from("length"),
            Value::Number(new_length as f64),
            span,
        )?;
        Ok(Value::Object(result))
    }
}
