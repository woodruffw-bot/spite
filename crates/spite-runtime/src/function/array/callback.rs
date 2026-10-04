//! Ordered callbacks over present properties (23.1.3.6/8/15/21/29).

use super::Builtin;
use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn array_callback(
        &mut self,
        builtin: Builtin,
        receiver: Value,
        callback: Value,
        this_arg: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(object) = self.box_primitive(receiver, span)? else {
            unreachable!("ToObject");
        };
        let length = self.length_of_array_like(&object, span)?;
        if !self.is_callable(&callback, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Array callback is not callable",
            ));
        }
        let Value::Object(callback) = callback else {
            unreachable!("callable object");
        };
        // 23.1.3.8/21: validate the callback before any constructor/species
        // access. Map preserves the range; filter starts with an empty result.
        let output = match builtin {
            Builtin::ArrayMap => Some(self.array_species_create(&object, length, span)?),
            Builtin::ArrayFilter => Some(self.array_species_create(&object, 0, span)?),
            _ => None,
        };
        let mut selected = 0u64;
        for index in 0..length {
            self.tick(span)?;
            let key = JsString::from(index.to_string().as_str());
            // Presence and value are observed anew on each iteration. Only the
            // range is fixed; callbacks may add, replace, or delete properties.
            if !self.has_property(&object, &key, span)? {
                continue;
            }
            let value = self.get_property(&object, &key, span)?;
            // Filter writes the value read before the callback, even when the
            // callback replaces/deletes that source property. Charge its copy.
            let retained = if matches!(builtin, Builtin::ArrayFilter) {
                self.object_work(span, |_, budget| budget.value(&value))?;
                Some(value.clone())
            } else {
                None
            };
            self.object_work(span, |_, budget| budget.value(&this_arg))?;
            let result = self.call(
                Value::Object(callback.clone()),
                this_arg.clone(),
                vec![
                    value,
                    Value::Number(index as f64),
                    Value::Object(object.clone()),
                ],
                span,
            )?;
            match builtin {
                Builtin::ArrayEvery if !result.to_boolean() => return Ok(Value::Boolean(false)),
                Builtin::ArraySome if result.to_boolean() => return Ok(Value::Boolean(true)),
                Builtin::ArrayMap => {
                    self.create_array_element(
                        output.as_ref().expect("map output"),
                        index,
                        result,
                        span,
                    )?;
                }
                Builtin::ArrayFilter if result.to_boolean() => {
                    self.create_array_element(
                        output.as_ref().expect("filter output"),
                        selected,
                        retained.expect("retained filter value"),
                        span,
                    )?;
                    selected += 1;
                }
                _ => {}
            }
        }
        Ok(match builtin {
            Builtin::ArrayForEach => Value::Undefined,
            Builtin::ArrayEvery => Value::Boolean(true),
            Builtin::ArraySome => Value::Boolean(false),
            Builtin::ArrayMap | Builtin::ArrayFilter => Value::Object(output.expect("output")),
            _ => unreachable!("callback iteration builtin"),
        })
    }
}
