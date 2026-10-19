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
        if !self.is_array(&Value::Object(source.clone()), span)? {
            return self.create_intrinsic_array(length, span);
        }
        let mut constructor = self.get_property(source, &JsString::from("constructor"), span)?;
        // ArraySpeciesCreate queries the constructor's realm before @@species,
        // even in one Realm: a Bound/Proxy target chain can contain a revoked
        // Proxy. Cross-realm intrinsic-Array normalization remains deferred.
        if self.is_constructor(&constructor, span)? {
            let Value::Object(function) = &constructor else {
                unreachable!("constructors are objects");
            };
            self.check_function_realm(function, span)?;
        }
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
