//! Sparse species-dependent copies with a final strict length Set (23.1.3.28).

use crate::{Error, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn array_slice(
        &mut self,
        receiver: Value,
        start: Value,
        end: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(source) = self.box_primitive(receiver, span)? else {
            unreachable!("ToObject");
        };
        let length = self.length_of_array_like(&source, span)?;
        let start = self.array_relative_index(start, length, span)?;
        let end = if matches!(end, Value::Undefined) {
            length
        } else {
            self.array_relative_index(end, length, span)?
        };
        // Conversion may mutate the source. The snapshotted bounds determine
        // the result length, while each presence check and Get remain live.
        let count = end.saturating_sub(start);
        let result = self.array_species_create(&source, count, span)?;
        for index in start..end {
            self.tick(span)?;
            let key = JsString::from(index.to_string().as_str());
            if self.has_property(&source, &key, span)? {
                let value = self.get_property(&source, &key, span)?;
                self.create_array_element(&result, index - start, value, span)?;
            }
        }
        // Unlike map/filter, slice sets length even on empty results. Custom
        // species may return ordinary objects, setters, or read-only lengths.
        self.set_property_or_throw(
            &result,
            JsString::from("length"),
            Value::Number(count as f64),
            span,
        )?;
        Ok(Value::Object(result))
    }
}
