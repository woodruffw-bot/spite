//! Ordered callbacks over present properties (23.1.3.6/15/29).

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
        for index in 0..length {
            self.tick(span)?;
            let key = JsString::from(index.to_string().as_str());
            // Presence and value are observed anew on each iteration. Only the
            // range is fixed; callbacks may add, replace, or delete properties.
            if !self.has_property(&object, &key, span)? {
                continue;
            }
            let value = self.get_property(&object, &key, span)?;
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
                _ => {}
            }
        }
        Ok(match builtin {
            Builtin::ArrayForEach => Value::Undefined,
            Builtin::ArrayEvery => Value::Boolean(true),
            Builtin::ArraySome => Value::Boolean(false),
            _ => unreachable!("callback iteration builtin"),
        })
    }
}
