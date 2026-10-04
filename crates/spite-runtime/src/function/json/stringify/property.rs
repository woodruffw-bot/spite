//! SerializeJSONProperty: hooks precede raw branding and wrapper conversion.

use super::{Serialized, State};
use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(super) fn json_serialize_property(
        &mut self,
        state: &State,
        holder: &ObjectHandle,
        key: &JsString,
        span: Span,
    ) -> Result<Serialized, Error> {
        let mut value = self.get_property(holder, key, span)?;
        if matches!(value, Value::Object(_) | Value::BigInt(_)) {
            let to_json = self.get_property_value(&value, &JsString::from("toJSON"), span)?;
            if self.is_callable(&to_json, span)? {
                value = self.call(to_json, value, vec![Value::String(key.clone())], span)?;
            }
        }
        if let Some(replacer) = &state.replacer {
            value = self.call(
                replacer.clone(),
                Value::Object(holder.clone()),
                vec![Value::String(key.clone()), value],
                span,
            )?;
        }
        if let Value::Object(object) = &value {
            let (raw, number, string, boolean, bigint) = self.object_work(span, |objects, _| {
                let object = objects.inspect(object)?;
                Ok((
                    object.is_raw_json(),
                    object.number_data().is_some(),
                    object.string_data().is_some(),
                    object.boolean_data(),
                    object.bigint_data().cloned(),
                ))
            })?;
            if raw {
                let Value::String(text) =
                    self.get_property(object, &JsString::from("rawJSON"), span)?
                else {
                    unreachable!("frozen raw JSON text")
                };
                return Ok(Serialized::Raw(text));
            }
            if number {
                value = Value::Number(self.number(value, span)?);
            } else if string {
                value = Value::String(self.string(value, span)?);
            } else if let Some(boolean) = boolean {
                value = Value::Boolean(boolean);
            } else if let Some(bigint) = bigint {
                value = Value::BigInt(bigint);
            }
        }
        Ok(match value {
            Value::Null => Serialized::Raw(JsString::from("null")),
            Value::Boolean(value) => {
                Serialized::Raw(JsString::from(if value { "true" } else { "false" }))
            }
            Value::String(value) => Serialized::String(value),
            Value::Number(value) => Serialized::Raw(if value.is_finite() {
                self.string(Value::Number(value), span)?
            } else {
                JsString::from("null")
            }),
            Value::BigInt(_) => {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "BigInt cannot be serialized as JSON",
                ));
            }
            Value::Object(object) if !self.is_callable(&Value::Object(object.clone()), span)? => {
                let array =
                    self.object_work(span, |objects, _| Ok(objects.inspect(&object)?.is_array()))?;
                Serialized::Container(object, array)
            }
            _ => Serialized::Omit,
        })
    }
}
