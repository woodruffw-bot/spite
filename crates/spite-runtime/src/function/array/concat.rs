//! Species-dependent concatenation and IsConcatSpreadable (23.1.3.2–2.1).

use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{JsString, Span, WellKnownSymbol};

impl Realm {
    pub(crate) fn array_concat(
        &mut self,
        receiver: Value,
        arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(source) = self.box_primitive(receiver, span)? else {
            unreachable!("ToObject");
        };
        // Construct before checking spreadability or reading source length.
        let result = self.array_species_create(&source, 0, span)?;
        let mut target = 0u64;
        const MAX_LENGTH: u64 = 9_007_199_254_740_991;
        for item in std::iter::once(Value::Object(source)).chain(arguments) {
            self.tick(span)?;
            if self.array_concat_spreadable(&item, span)? {
                let Value::Object(object) = item else {
                    unreachable!("only objects are spreadable");
                };
                let length = self.length_of_array_like(&object, span)?;
                // Subtraction avoids overflow and preserves exact integer work.
                // Reject before any indexed presence checks or reads.
                if length > MAX_LENGTH - target {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "Array concatenation exceeds the maximum safe integer length",
                    ));
                }
                for index in 0..length {
                    self.tick(span)?;
                    let key = JsString::from(index.to_string().as_str());
                    if self.has_property(&object, &key, span)? {
                        let value = self.get_property(&object, &key, span)?;
                        self.create_array_element(&result, target, value, span)?;
                    }
                    // A hole advances the result cursor without deleting any
                    // property already supplied by a custom species result.
                    target += 1;
                }
            } else {
                if target == MAX_LENGTH {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "Array concatenation exceeds the maximum safe integer length",
                    ));
                }
                self.create_array_element(&result, target, item, span)?;
                target += 1;
            }
        }
        // Do not prevalidate an intrinsic Array's uint32 length. Definitions
        // and custom setters happen in order; this final strict Set can fail
        // after prior effects. Huge sparse scans remain bounded host work.
        self.set_property_or_throw(
            &result,
            JsString::from("length"),
            Value::Number(target as f64),
            span,
        )?;
        Ok(Value::Object(result))
    }

    fn array_concat_spreadable(&mut self, value: &Value, span: Span) -> Result<bool, Error> {
        let Value::Object(object) = value else {
            return Ok(false);
        };
        let spreadable =
            self.get_property(object, &WellKnownSymbol::IsConcatSpreadable.symbol(), span)?;
        if !matches!(spreadable, Value::Undefined) {
            return Ok(spreadable.to_boolean());
        }
        // IsArray uses the internal brand, independently of public prototypes.
        self.object_work(span, |objects, _| Ok(objects.inspect(object)?.is_array()))
    }
}
