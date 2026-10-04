//! Array species and unscopables properties (23.1.2.6, 23.1.3.41).

use super::*;
use spite_core::WellKnownSymbol;

impl Realm {
    pub(super) fn array_symbol_properties(
        &mut self,
        constructor: &ObjectHandle,
        prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<(ObjectHandle, ObjectHandle), Error> {
        let species = self.new_builtin(function_prototype, Builtin::ArraySpecies, span)?;
        let unscopables = self.object_work(span, |objects, budget| {
            let table = objects.create(None)?;
            for name in [
                "at",
                "copyWithin",
                "entries",
                "fill",
                "find",
                "findIndex",
                "findLast",
                "findLastIndex",
                "flat",
                "flatMap",
                "includes",
                "keys",
                "toReversed",
                "toSorted",
                "toSpliced",
                "values",
            ] {
                objects.define(
                    &table,
                    name,
                    DataDescriptor {
                        value: Some(Value::Boolean(true)),
                        writable: Some(true),
                        enumerable: Some(true),
                        configurable: Some(true),
                    },
                    budget,
                )?;
            }
            objects.define(
                constructor,
                WellKnownSymbol::Species.symbol(),
                PropertyDescriptor {
                    kind: DescriptorKind::Accessor {
                        get: Some(Some(species.clone())),
                        set: Some(None),
                    },
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                budget,
            )?;
            objects.define(
                prototype,
                WellKnownSymbol::Unscopables.symbol(),
                DataDescriptor {
                    value: Some(Value::Object(table.clone())),
                    writable: Some(false),
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                budget,
            )?;
            Ok(table)
        })?;
        Ok((species, unscopables))
    }
}
