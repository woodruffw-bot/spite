//! ArraySpeciesCreate with ordered constructor/species lookup (7.3.22).

use crate::{Error, ObjectHandle, Realm, Value};
use spite_core::{JsString, Span, WellKnownSymbol};

impl Realm {
    pub(super) fn array_species_create(
        &mut self,
        source: &ObjectHandle,
        length: u64,
        span: Span,
    ) -> Result<ObjectHandle, Error> {
        if !self.object_work(span, |objects, _| Ok(objects.inspect(source)?.is_array()))? {
            return self.create_intrinsic_array(length, span);
        }
        let mut constructor = self.get_property(source, &JsString::from("constructor"), span)?;
        // All exposed constructors belong to this realm. Cross-realm intrinsic
        // normalization and Proxy IsArray traversal join this operation when
        // those object kinds can be exposed; foreign heap handles are rejected.
        if let Value::Object(object) = constructor {
            constructor = self.get_property(&object, &WellKnownSymbol::Species.symbol(), span)?;
            if matches!(constructor, Value::Null) {
                constructor = Value::Undefined;
            }
        }
        if matches!(constructor, Value::Undefined) {
            return self.create_intrinsic_array(length, span);
        }
        // Construct performs IsConstructor without coercing the value, then
        // passes exactly one Number argument. Custom results need not be Arrays
        // and must not be subjected to ArrayCreate's uint32 length restriction.
        let Value::Object(result) =
            self.construct(constructor, vec![Value::Number(length as f64)], span)?
        else {
            unreachable!("Construct returns an object");
        };
        Ok(result)
    }
}
