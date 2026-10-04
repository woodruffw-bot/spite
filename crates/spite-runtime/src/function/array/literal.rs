//! ArrayAccumulation with elisions and iterable spread (13.2.4.1–2).

use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value};
use spite_core::{JsString, Span, WellKnownSymbol};
use spite_parser::ast::ArrayElement;

impl Realm {
    pub(crate) fn array_literal(
        &mut self,
        elements: &[ArrayElement],
        span: Span,
    ) -> Result<Value, Error> {
        let array = self.create_intrinsic_array(0, span)?;
        let mut index = 0;
        for element in elements {
            match element {
                ArrayElement::Expression(expression) => {
                    // Anonymous functions do not acquire a name here.
                    let value = self.expression(expression)?;
                    self.create_array_element(&array, index, value, expression.span)?;
                    index = Self::next_array_literal_index(index, span)?;
                }
                ArrayElement::Spread(expression) => {
                    let value = self.expression(expression)?;
                    index = self.accumulate_array_spread(&array, index, value, expression.span)?;
                }
                ArrayElement::Elision => {
                    index = Self::next_array_literal_index(index, span)?;
                    // Elisions assign length at this point, before later
                    // expressions. The fresh Array owns a writable length.
                    self.set_property_or_throw(
                        &array,
                        JsString::from("length"),
                        Value::Number(index as f64),
                        span,
                    )?;
                }
            }
        }
        self.set_property_or_throw(
            &array,
            JsString::from("length"),
            Value::Number(index as f64),
            span,
        )?;
        Ok(Value::Object(array))
    }

    fn accumulate_array_spread(
        &mut self,
        array: &ObjectHandle,
        mut index: u64,
        value: Value,
        span: Span,
    ) -> Result<u64, Error> {
        let method = self
            .get_method(&value, &WellKnownSymbol::Iterator.symbol(), span)?
            .ok_or_else(|| {
                Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "spread value is not iterable",
                )
            })?;
        let mut iterator = self.get_iterator_from_method(value, method, span)?;
        loop {
            self.tick(span)?;
            let Some(value) = self.iterator_step_value(&mut iterator, span)? else {
                return Ok(index);
            };
            // ArrayAccumulation propagates step failures directly. The fresh
            // Array's element definitions have no user-code cleanup on failure.
            self.create_array_element(array, index, value, span)?;
            index = Self::next_array_literal_index(index, span)?;
        }
    }

    fn next_array_literal_index(index: u64, span: Span) -> Result<u64, Error> {
        index.checked_add(1).ok_or_else(|| Error::Limit {
            span,
            message: "Array literal index exceeds platform capacity".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spread_finishes_before_an_overflowing_final_array_length_write() {
        let mut realm = Realm::default();
        let span = Span::new(0, 0);
        let source = realm.eval("let steps=0,closed=0;({[Symbol.iterator](){return {next(){steps++;return {done:steps===3,value:steps};},return(){closed++;return {};}};}})").unwrap();
        let array = realm.create_intrinsic_array(0, span).unwrap();
        let index = realm
            .accumulate_array_spread(&array, u64::from(u32::MAX) - 1, source, span)
            .unwrap();
        assert_eq!(index, u64::from(u32::MAX) + 1);
        let value = Value::Object(array.clone());
        assert_eq!(
            realm.read_property(&value, &JsString::from("length")),
            Ok(Value::Number(f64::from(u32::MAX)))
        );
        assert_eq!(
            realm.read_property(&value, &JsString::from("4294967294")),
            Ok(Value::Number(1.0))
        );
        assert_eq!(
            realm.read_property(&value, &JsString::from("4294967295")),
            Ok(Value::Number(2.0))
        );
        assert!(matches!(
            realm.set_property_or_throw(
                &array,
                JsString::from("length"),
                Value::Number(index as f64),
                span
            ),
            Err(Error::Exception {
                kind: ExceptionKind::RangeError,
                ..
            })
        ));
        assert_eq!(
            realm.eval("steps===3 && closed===0"),
            Ok(Value::Boolean(true))
        );
    }

    #[test]
    fn spread_property_keys_round_indices_to_number_before_stringification() {
        let mut realm = Realm::default();
        let span = Span::new(0, 0);
        let source=realm.eval("let steps=0;({[Symbol.iterator](){return {next(){return {done:++steps===2,value:7};}};}})").unwrap();
        let array = realm.create_intrinsic_array(0, span).unwrap();
        assert_eq!(
            realm.accumulate_array_spread(&array, 9_007_199_254_740_993, source, span),
            Ok(9_007_199_254_740_994)
        );
        let value = Value::Object(array);
        assert_eq!(
            realm.read_property(&value, &JsString::from("9007199254740992")),
            Ok(Value::Number(7.0))
        );
        assert_eq!(
            realm.read_property(&value, &JsString::from("9007199254740993")),
            Ok(Value::Undefined)
        );
    }
}
