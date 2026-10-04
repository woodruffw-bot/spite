//! Bounded in-place range writes and overlapping copies (23.1.3.4/7).

use crate::{Error, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn array_fill(
        &mut self,
        receiver: Value,
        value: Value,
        start: Value,
        end: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(object) = self.box_primitive(receiver, span)? else {
            unreachable!("ToObject");
        };
        let length = self.length_of_array_like(&object, span)?;
        let start = self.array_relative_index(start, length, span)?;
        let end = if matches!(end, Value::Undefined) {
            length
        } else {
            self.array_relative_index(end, length, span)?
        };
        for index in start..end {
            self.tick(span)?;
            self.object_work(span, |_, budget| budget.value(&value))?;
            self.set_property_or_throw(
                &object,
                JsString::from(index.to_string().as_str()),
                value.clone(),
                span,
            )?;
        }
        Ok(Value::Object(object))
    }

    pub(crate) fn array_copy_within(
        &mut self,
        receiver: Value,
        target: Value,
        start: Value,
        end: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(object) = self.box_primitive(receiver, span)? else {
            unreachable!("ToObject");
        };
        let length = self.length_of_array_like(&object, span)?;
        let target = self.array_relative_index(target, length, span)?;
        let start = self.array_relative_index(start, length, span)?;
        let end = if matches!(end, Value::Undefined) {
            length
        } else {
            self.array_relative_index(end, length, span)?
        };
        // A negative specification count performs no iterations. Saturation
        // represents it as zero without narrowing the ToLength index range.
        let count = end.saturating_sub(start).min(length - target);
        let backwards = start < target && target < start + count;
        for offset in 0..count {
            self.tick(span)?;
            let offset = if backwards {
                count - 1 - offset
            } else {
                offset
            };
            let from_key = JsString::from((start + offset).to_string().as_str());
            let to_key = JsString::from((target + offset).to_string().as_str());
            // Even identical source/target indices perform these operations.
            // Reads are live, and copying a hole deletes an own target property.
            if self.has_property(&object, &from_key, span)? {
                let value = self.get_property(&object, &from_key, span)?;
                self.set_property_or_throw(&object, to_key, value, span)?;
            } else {
                self.delete_property_or_throw(&object, &to_key, span)?;
            }
        }
        Ok(Value::Object(object))
    }

    fn array_relative_index(
        &mut self,
        value: Value,
        length: u64,
        span: Span,
    ) -> Result<u64, Error> {
        let number = self.number(value, span)?;
        let integer = if number.is_nan() { 0.0 } else { number.trunc() };
        Ok(if integer < 0.0 {
            (length as f64 + integer).max(0.0)
        } else {
            integer.min(length as f64)
        } as u64)
    }
}
