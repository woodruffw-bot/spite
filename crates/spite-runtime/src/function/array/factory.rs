//! The generic Array.of factory (23.1.2.4).

use crate::{Error, Realm, Value, object::DataDescriptor};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn array_of(
        &mut self,
        receiver: Value,
        arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let length = arguments.len();
        let length_number = Value::Number(length as f64);
        let constructor = if let Value::Object(object) = &receiver {
            self.object_work(span, |objects, _| {
                Ok(objects.inspect(object)?.is_constructor())
            })?
        } else {
            false
        };
        let object = if constructor {
            let Value::Object(object) =
                self.construct(receiver, vec![length_number.clone()], span)?
            else {
                unreachable!("Construct returns an object");
            };
            object
        } else {
            self.create_intrinsic_array(length as u64, span)?
        };
        for (index, value) in arguments.enumerate() {
            self.tick(span)?;
            // CreateDataPropertyOrThrow also replaces configurable own
            // accessors/non-writable properties; inherited setters are skipped.
            self.define_property_or_throw(
                &object,
                JsString::from(index.to_string().as_str()),
                DataDescriptor {
                    value: Some(value),
                    writable: Some(true),
                    enumerable: Some(true),
                    configurable: Some(true),
                }
                .into(),
                span,
            )?;
        }
        // Set, unlike element creation, observes an inherited length setter.
        // Even an empty item list performs this strict assignment.
        self.set_property_or_throw(&object, JsString::from("length"), length_number, span)?;
        Ok(Value::Object(object))
    }
}
