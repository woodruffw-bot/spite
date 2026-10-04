//! CopyDataProperties with an empty excluded-key list for object spread (7.3.25).

use crate::{Error, ObjectHandle, Realm, Value, object::DataDescriptor};
use spite_core::Span;

impl Realm {
    pub(crate) fn copy_spread_properties(
        &mut self,
        target: &ObjectHandle,
        source: Value,
        span: Span,
    ) -> Result<(), Error> {
        if matches!(source, Value::Undefined | Value::Null) {
            return Ok(());
        }
        let Value::Object(source) = self.box_primitive(source, span)? else {
            unreachable!("ToObject");
        };
        let keys = self.own_property_keys(&source, span)?;
        for key in keys {
            // Snapshot the ordered keys, but read descriptors and values live.
            // A prior getter can delete keys or change their enumerability.
            self.tick(span)?;
            if self
                .own_property_descriptor(&source, &key, span)?
                .is_some_and(|p| p.enumerable())
            {
                let value = self.get_property(&source, &key, span)?;
                // CreateDataPropertyOrThrow bypasses inherited setters, treats
                // __proto__ as data, and replaces configurable own accessors.
                self.define_property_or_throw(
                    target,
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
            }
        }
        Ok(())
    }
}
