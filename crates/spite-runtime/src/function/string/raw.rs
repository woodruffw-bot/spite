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
            let key = JsString::from_index(index);
            let literal = self.get_property(&literals, &key, span)?;
            if count == 1 {
                let literal = self.string(literal, span)?;
                self.object_work(span, |_, budget| budget.charge(literal.len()))?;
                let value = Value::String(literal);
                self.check_string(&value, span)?;
                return Ok(value);
            }
            self.append_raw_value(&mut result, literal, span)?;
            if index + 1 < count
                && let Some(substitution) = substitutions.next()
            {
                self.append_raw_value(&mut result, substitution, span)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_literal_preserves_output_limits_work_and_agent_termination() {
        let mut realm = Realm::default();
        let template = realm.eval("({raw:['x\\uD800\\u4E2D\\uDC00']})").unwrap();
        let span = Span::default();
        let expected = Value::String(JsString::from([0x78_u16, 0xd800, 0x4e2d, 0xdc00]));
        realm.limits.max_string_units = Some(4);
        assert_eq!(
            realm.string_raw(template.clone(), Vec::new().into_iter(), span),
            Ok(expected)
        );
        realm.limits.max_string_units = Some(3);
        assert!(matches!(
            realm.string_raw(template.clone(), Vec::new().into_iter(), span),
            Err(Error::Limit { .. })
        ));
        realm.limits.max_string_units = None;
        realm.remaining_steps = Some(0);
        assert!(matches!(
            realm.string_raw(template.clone(), Vec::new().into_iter(), span),
            Err(Error::Limit { .. })
        ));
        realm.remaining_steps = None;
        realm.agent_control().terminate();
        assert!(matches!(
            realm.string_raw(template, Vec::new().into_iter(), span),
            Err(Error::Host { .. })
        ));
    }
}
