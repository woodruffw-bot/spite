//! Array string conversion and generic join (23.1.3.18/36).

use crate::{Error, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn array_join(
        &mut self,
        receiver: Value,
        separator: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(object) = self.box_primitive(receiver, span)? else {
            unreachable!("ToObject");
        };
        let length = self.length_of_array_like(&object, span)?;
        let separator = if matches!(separator, Value::Undefined) {
            JsString::from(",")
        } else {
            self.string(separator, span)?
        };
        let mut result = Vec::new();
        for index in 0..length {
            self.tick(span)?;
            if index > 0 {
                self.object_work(span, |_, budget| budget.charge(separator.len()))?;
                self.append_string(&mut result, &separator, span)?;
            }
            let element =
                self.get_property(&object, &JsString::from(index.to_string().as_str()), span)?;
            if !matches!(element, Value::Undefined | Value::Null) {
                let string = self.string(element, span)?;
                self.object_work(span, |_, budget| budget.charge(string.len()))?;
                self.append_string(&mut result, &string, span)?;
            }
        }
        Ok(Value::String(JsString::from_code_units(result)))
    }

    pub(crate) fn array_to_string(&mut self, receiver: Value, span: Span) -> Result<Value, Error> {
        let Value::Object(object) = self.box_primitive(receiver, span)? else {
            unreachable!("ToObject");
        };
        let join = self.get_property(&object, &JsString::from("join"), span)?;
        let function = if self.is_callable(&join, span)? {
            join
        } else {
            // Use the intrinsic, not the possibly replaced public property.
            Value::Object(
                self.intrinsics
                    .as_ref()
                    .expect("initialized")
                    .object_to_string
                    .clone(),
            )
        };
        self.call(function, Value::Object(object), Vec::new(), span)
    }
}
