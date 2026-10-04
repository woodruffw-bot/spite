//! Object.assign and Object.getOwnPropertyDescriptors (20.1.2.1/9).

use crate::{Error, ExceptionKind, Realm, Value, object::DataDescriptor};
use spite_core::Span;

impl Realm {
    pub(crate) fn object_assign(
        &mut self,
        target: Value,
        sources: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let target = self.box_primitive(target, span)?;
        for source in sources {
            self.tick(span)?;
            if matches!(source, Value::Undefined | Value::Null) {
                continue;
            }
            let Value::Object(source) = self.box_primitive(source, span)? else {
                unreachable!("ToObject");
            };
            let keys = self.own_property_keys(&source, span)?;
            for key in keys {
                if self
                    .own_property_descriptor(&source, &key, span)?
                    .is_some_and(|property| property.enumerable())
                {
                    let value = self.get_property(&source, &key, span)?;
                    // Set(..., true) invokes inherited setters and throws on a
                    // rejected write even when the surrounding script is sloppy.
                    if !self.set_property_value(&target, key, value, span)? {
                        return Err(Self::exception(
                            ExceptionKind::TypeError,
                            span,
                            "Object.assign property write was rejected",
                        ));
                    }
                }
            }
        }
        Ok(target)
    }

    pub(crate) fn object_get_own_property_descriptors(
        &mut self,
        target: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(target) = self.box_primitive(target, span)? else {
            unreachable!("ToObject");
        };
        let keys = self.own_property_keys(&target, span)?;
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .object_prototype
            .clone();
        let result = self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
        for key in keys {
            let property = self.own_property_descriptor(&target, &key, span)?;
            let descriptor = self.descriptor_object(property, span)?;
            if !matches!(descriptor, Value::Undefined) {
                // CreateDataProperty avoids inherited setters and treats the
                // string "__proto__" as an ordinary own key.
                self.define_property_or_throw(
                    &result,
                    key,
                    DataDescriptor {
                        value: Some(descriptor),
                        writable: Some(true),
                        enumerable: Some(true),
                        configurable: Some(true),
                    }
                    .into(),
                    span,
                )?;
            }
        }
        Ok(Value::Object(result))
    }
}
