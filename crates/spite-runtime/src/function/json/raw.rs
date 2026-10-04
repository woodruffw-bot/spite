//! Raw JSON creation and the unforgeable [[IsRawJSON]] brand (25.5.1, 25.5.3).

use crate::{Error, ExceptionKind, Realm, Value, object::DataDescriptor};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn json_raw(&mut self, text: Value, span: Span) -> Result<Value, Error> {
        let text = self.string(text, span)?;
        // Edition 17 restricts the first/last code units before ParseJSON.
        // Containers and surrounding whitespace fail without materialization.
        let units = text.code_units();
        let allowed = |unit: u16| matches!(unit, 0x61..=0x7a | 0x30..=0x39 | 0x22);
        if !units
            .first()
            .is_some_and(|unit| allowed(*unit) || *unit == 0x2d)
            || !units.last().is_some_and(|unit| allowed(*unit))
        {
            return Err(Self::exception(
                ExceptionKind::SyntaxError,
                span,
                "raw JSON must contain a primitive without surrounding whitespace",
            ));
        }
        self.json_document(&text, span)?;
        self.check_json_string(&text, span)?;
        let object = self.object_work(span, |objects, _| objects.create_raw_json())?;
        self.define_property_or_throw(
            &object,
            JsString::from("rawJSON"),
            DataDescriptor {
                value: Some(Value::String(text)),
                writable: Some(true),
                enumerable: Some(true),
                configurable: Some(true),
            }
            .into(),
            span,
        )?;
        self.object_set_integrity(Value::Object(object), true, span)
    }

    pub(crate) fn json_is_raw(&mut self, value: Value, span: Span) -> Result<Value, Error> {
        let branded = match value {
            Value::Object(object) => self.object_work(span, |objects, _| {
                Ok(objects.inspect(&object)?.is_raw_json())
            })?,
            _ => false,
        };
        Ok(Value::Boolean(branded))
    }
}
