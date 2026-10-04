//! Object.fromEntries (20.1.2.7) and AddEntriesFromIterable (24.1.1.2).

use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value, object::DataDescriptor};
use spite_core::{JsString, Span, WellKnownSymbol};

impl Realm {
    pub(crate) fn object_from_entries(
        &mut self,
        iterable: Value,
        span: Span,
    ) -> Result<Value, Error> {
        Self::require_object_coercible(&iterable, span)?;
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .object_prototype
            .clone();
        let result = self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
        let method = self
            .get_method(&iterable, &WellKnownSymbol::Iterator.symbol(), span)?
            .ok_or_else(|| {
                Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "fromEntries input is not iterable",
                )
            })?;
        let mut iterator = self.get_iterator_from_method(iterable, method, span)?;
        loop {
            self.tick(span)?;
            // IteratorStepValue failures do not close. Entry reads and the
            // private CreateDataProperty adder close on language exceptions.
            let Some(entry) = self.iterator_step_value(&mut iterator, span)? else {
                return Ok(Value::Object(result));
            };
            if let Err(error) = self.define_entry(&result, entry, span) {
                return Err(self.iterator_close_error(&iterator, error, span));
            }
        }
    }

    fn define_entry(
        &mut self,
        result: &ObjectHandle,
        entry: Value,
        span: Span,
    ) -> Result<(), Error> {
        let Value::Object(entry) = entry else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "fromEntries entry is not an object",
            ));
        };
        // Read both properties before converting the key. Entry objects are
        // never iterated, and boxed Strings are valid without special handling.
        let key = self.get_property(&entry, &JsString::from("0"), span)?;
        let value = self.get_property(&entry, &JsString::from("1"), span)?;
        let key = self.property_key(key, span)?;
        self.define_property_or_throw(
            result,
            key,
            DataDescriptor {
                value: Some(value),
                writable: Some(true),
                enumerable: Some(true),
                configurable: Some(true),
            }
            .into(),
            span,
        )
    }
}
