//! Ordered reduction over present own or inherited elements (23.1.3.24–25).

use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn array_reduce(
        &mut self,
        receiver: Value,
        callback: Value,
        mut accumulator: Option<Value>,
        backwards: bool,
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
                "Array reducer is not callable",
            ));
        }
        let Value::Object(callback) = callback else {
            unreachable!("callable object");
        };
        for offset in 0..length {
            self.tick(span)?;
            let index = if backwards {
                length - 1 - offset
            } else {
                offset
            };
            let key = JsString::from(index.to_string().as_str());
            if !self.has_property(&object, &key, span)? {
                continue;
            }
            let value = self.get_property(&object, &key, span)?;
            // None means no initial argument or present element has supplied
            // an accumulator yet. Some(undefined) is an ordinary accumulator.
            // Moving it into Call avoids cloning retained strings or BigInts.
            accumulator = Some(match accumulator {
                None => value,
                Some(previous) => self.call(
                    Value::Object(callback.clone()),
                    Value::Undefined,
                    vec![
                        previous,
                        value,
                        Value::Number(index as f64),
                        Value::Object(object.clone()),
                    ],
                    span,
                )?,
            });
        }
        accumulator.ok_or_else(|| {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "Array reduction has no initial value or present element",
            )
        })
    }
}
