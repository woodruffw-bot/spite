//! Own-key reflection and EnumerableOwnProperties (20.1.2.5/10/11/19/24, 7.3.23).

use super::Builtin;
use crate::{Error, Realm, Value};
use spite_core::{PropertyKey, Span};

impl Realm {
    pub(crate) fn object_get_own_property_keys(
        &mut self,
        value: Value,
        symbols: bool,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(object) = self.box_primitive(value, span)? else {
            unreachable!("ToObject");
        };
        let keys = self.own_property_keys(&object, span)?;
        let mut values = Vec::new();
        for key in keys {
            self.tick(span)?;
            // GetOwnPropertyKeys filters key types without consulting property
            // descriptors or invoking getters, including for non-enumerables.
            match (key, symbols) {
                (PropertyKey::String(key), false) => values.push(Value::String(key)),
                (PropertyKey::Symbol(key), true) => values.push(Value::Symbol(key)),
                _ => {}
            }
        }
        self.create_array_from_list(values, span)
    }

    pub(crate) fn object_enumerable_properties(
        &mut self,
        value: Value,
        kind: Builtin,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(object) = self.box_primitive(value, span)? else {
            unreachable!("ToObject");
        };
        // Snapshot keys once; recheck each own descriptor immediately before
        // visiting it. Earlier getters can delete or reconfigure later keys.
        let keys = self.own_property_keys(&object, span)?;
        let mut results = Vec::new();
        for key in keys {
            self.tick(span)?;
            let PropertyKey::String(key) = key else {
                continue;
            };
            if !self
                .own_property_descriptor(&object, &key, span)?
                .is_some_and(|property| property.enumerable())
            {
                continue;
            }
            let result = if matches!(kind, Builtin::ObjectKeys) {
                Value::String(key)
            } else {
                let value = self.get_property(&object, &key, span)?;
                if matches!(kind, Builtin::ObjectValues) {
                    value
                } else {
                    debug_assert!(matches!(kind, Builtin::ObjectEntries));
                    self.create_array_from_list(vec![Value::String(key), value], span)?
                }
            };
            results.push(result);
        }
        self.create_array_from_list(results, span)
    }
}
