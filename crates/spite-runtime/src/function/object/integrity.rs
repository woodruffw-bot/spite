//! SetIntegrityLevel/TestIntegrityLevel (7.3.15–16).

use crate::{
    Error, Realm, Value,
    object::{DataDescriptor, Property},
};
use spite_core::Span;

impl Realm {
    pub(crate) fn object_set_integrity(
        &mut self,
        target: Value,
        frozen: bool,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(object) = &target else {
            return Ok(target);
        };
        self.object_work(span, |objects, _| objects.prevent_extensions(object))?;
        let keys = self.own_property_keys(object, span)?;
        for key in keys {
            let writable = if frozen {
                match self.own_property_descriptor(object, &key, span)? {
                    Some(Property::Data(_)) => Some(false),
                    Some(Property::Accessor(_)) => None,
                    None => continue,
                }
            } else {
                None
            };
            self.define_property_or_throw(
                object,
                key,
                DataDescriptor {
                    configurable: Some(false),
                    writable,
                    ..DataDescriptor::default()
                }
                .into(),
                span,
            )?;
        }
        Ok(target)
    }

    pub(crate) fn object_test_integrity(
        &mut self,
        target: Value,
        frozen: bool,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(object) = target else {
            return Ok(Value::Boolean(true));
        };
        // An extensible object fails before any own-key/descriptor inspection.
        if self.object_work(span, |objects, _| {
            Ok(objects.inspect(&object)?.is_extensible())
        })? {
            return Ok(Value::Boolean(false));
        }
        let keys = self.own_property_keys(&object, span)?;
        for key in keys {
            if let Some(property) = self.own_property_descriptor(&object, &key, span)? {
                if property.configurable()
                    || (frozen && property.as_data().is_some_and(|data| data.writable))
                {
                    return Ok(Value::Boolean(false));
                }
            }
        }
        Ok(Value::Boolean(true))
    }
}
