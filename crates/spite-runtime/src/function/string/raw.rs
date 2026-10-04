//! String.raw's ordinary array-like template processing (22.1.2.4).

use crate::{Error, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn string_raw(
        &mut self,
        template: Value,
        mut substitutions: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(cooked) = self.box_primitive(template, span)? else {
            unreachable!("ToObject");
        };
        let raw = self.get_property(&cooked, &JsString::from("raw"), span)?;
        let Value::Object(literals) = self.box_primitive(raw, span)? else {
            unreachable!("ToObject");
        };
        let count = self.length_of_array_like(&literals, span)?;
        let mut result = Vec::new();
        for index in 0..count {
            self.tick(span)?;
            let key = JsString::from(index.to_string().as_str());
            let literal = self.get_property(&literals, &key, span)?;
            self.append_raw_value(&mut result, literal, span)?;
            if index + 1 < count {
                if let Some(substitution) = substitutions.next() {
                    self.append_raw_value(&mut result, substitution, span)?;
                }
            }
        }
        Ok(Value::String(JsString::from_code_units(result)))
    }

    fn append_raw_value(
        &mut self,
        result: &mut Vec<u16>,
        value: Value,
        span: Span,
    ) -> Result<(), Error> {
        let string = self.string(value, span)?;
        self.object_work(span, |_, budget| budget.charge(string.len()))?;
        self.append_string(result, &string, span)
    }
}
