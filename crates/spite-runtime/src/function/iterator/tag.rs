//! Protected Iterator constructor/tag setters (27.1.3.3.2, 27.1.3.3.14, 7.3.37).

use crate::{Error, ExceptionKind, Realm, Value, object::DataDescriptor};
use spite_core::{PropertyKey, Span};

impl Realm {
    pub(crate) fn iterator_prototype_setter(
        &mut self,
        receiver: Value,
        key: PropertyKey,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(object) = receiver else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Iterator prototype setter requires an object receiver",
            ));
        };
        if object
            == self
                .intrinsics
                .as_ref()
                .expect("initialized")
                .iterator
                .prototype
        {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Iterator prototype setter cannot change Iterator.prototype",
            ));
        }
        if self.own_property_descriptor(&object, &key, span)?.is_none() {
            // Create an own property without consulting inherited descriptors.
            self.define_property_or_throw(
                &object,
                key,
                DataDescriptor {
                    value: Some(value),
                    writable: Some(true),
                    enumerable: Some(true),
                    configurable: Some(true),
                }
                .into(),
                span,
            )?;
        } else if !self.set_property_value(&Value::Object(object), key, value, span)? {
            // Set(..., true) rejects failures even when called by sloppy code.
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Iterator property assignment was rejected",
            ));
        }
        Ok(Value::Undefined)
    }
}
