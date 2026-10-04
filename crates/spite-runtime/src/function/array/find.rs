//! FindViaPredicate in both directions, including holes (23.1.3.9–12).

use super::Builtin;
use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn array_find(
        &mut self,
        builtin: Builtin,
        receiver: Value,
        predicate: Value,
        this_arg: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(object) = self.box_primitive(receiver, span)? else {
            unreachable!("ToObject");
        };
        let length = self.length_of_array_like(&object, span)?;
        if !self.is_callable(&predicate, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Array predicate is not callable",
            ));
        }
        let Value::Object(predicate) = predicate else {
            unreachable!("callable object");
        };
        let descending = matches!(
            builtin,
            Builtin::ArrayFindLast | Builtin::ArrayFindLastIndex
        );
        let return_index = matches!(
            builtin,
            Builtin::ArrayFindIndex | Builtin::ArrayFindLastIndex
        );
        for offset in 0..length {
            self.tick(span)?;
            let index = if descending {
                length - 1 - offset
            } else {
                offset
            };
            // Get every index, without HasProperty: holes and deleted elements
            // are visited as undefined unless the prototype supplies a value.
            let value =
                self.get_property(&object, &JsString::from(index.to_string().as_str()), span)?;
            self.object_work(span, |_, budget| {
                budget.value(&this_arg)?;
                budget.value(&value)
            })?;
            let result = self.call(
                Value::Object(predicate.clone()),
                this_arg.clone(),
                vec![
                    value.clone(),
                    Value::Number(index as f64),
                    Value::Object(object.clone()),
                ],
                span,
            )?;
            if result.to_boolean() {
                // Return the value read before the predicate, even if it has
                // since replaced or deleted the source property.
                return Ok(if return_index {
                    Value::Number(index as f64)
                } else {
                    value
                });
            }
        }
        Ok(if return_index {
            Value::Number(-1.0)
        } else {
            Value::Undefined
        })
    }
}
