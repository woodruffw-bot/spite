//! Generic mutations with ordered partial effects (23.1.3.22–23/26).

use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn array_reverse(&mut self, receiver: Value, span: Span) -> Result<Value, Error> {
        let Value::Object(object) = self.box_primitive(receiver, span)? else {
            unreachable!("ToObject");
        };
        let length = self.length_of_array_like(&object, span)?;
        for lower in 0..length / 2 {
            self.tick(span)?;
            let lower_key = JsString::from(lower.to_string().as_str());
            let upper_key = JsString::from((length - lower - 1).to_string().as_str());
            let lower_value = if self.has_property(&object, &lower_key, span)? {
                Some(self.get_property(&object, &lower_key, span)?)
            } else {
                None
            };
            // The lower getter can change whether the upper property exists.
            // Both reads precede mutations; values are moved into Set afterward.
            let upper_value = if self.has_property(&object, &upper_key, span)? {
                Some(self.get_property(&object, &upper_key, span)?)
            } else {
                None
            };
            match (lower_value, upper_value) {
                (Some(lower), Some(upper)) => {
                    self.set_property_or_throw(&object, lower_key, upper, span)?;
                    self.set_property_or_throw(&object, upper_key, lower, span)?;
                }
                (None, Some(upper)) => {
                    self.set_property_or_throw(&object, lower_key, upper, span)?;
                    self.delete_property_or_throw(&object, &upper_key, span)?;
                }
                (Some(lower), None) => {
                    self.delete_property_or_throw(&object, &lower_key, span)?;
                    self.set_property_or_throw(&object, upper_key, lower, span)?;
                }
                (None, None) => {}
            }
        }
        // No length assignment is performed, including on empty receivers.
        Ok(Value::Object(object))
    }

    pub(crate) fn array_push(
        &mut self,
        receiver: Value,
        arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(object) = self.box_primitive(receiver, span)? else {
            unreachable!("ToObject");
        };
        let mut length = self.length_of_array_like(&object, span)?;
        if arguments.len() as u64 > 9_007_199_254_740_991 - length {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Array push exceeds the maximum safe length",
            ));
        }
        for value in arguments {
            self.tick(span)?;
            self.set_property_or_throw(
                &object,
                JsString::from(length.to_string().as_str()),
                value,
                span,
            )?;
            length += 1;
        }
        // Even with no arguments, Set must observe a setter or reject a
        // read-only length. For Arrays this can throw after indexed writes.
        self.set_property_or_throw(
            &object,
            JsString::from("length"),
            Value::Number(length as f64),
            span,
        )?;
        Ok(Value::Number(length as f64))
    }

    pub(crate) fn array_pop(&mut self, receiver: Value, span: Span) -> Result<Value, Error> {
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
        let new_length = length - 1;
        let key = JsString::from(new_length.to_string().as_str());
        let element = self.get_property(&object, &key, span)?;
        self.delete_property_or_throw(&object, &key, span)?;
        self.set_property_or_throw(
            &object,
            JsString::from("length"),
            Value::Number(new_length as f64),
            span,
        )?;
        Ok(element)
    }
}
