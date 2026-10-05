//! Append-only GlobalSymbolRegistry shared across realms (20.4.2.2/6).

use crate::{Error, ExceptionKind, Realm, Value, object::Budget};
use spite_core::{JsString, JsSymbol, Span};
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Eq, PartialEq)]
enum RegistryError {
    Work,
    Capacity,
}

struct Registry {
    symbols: Vec<JsSymbol>,
    units: usize,
    max_entries: Option<usize>,
    max_units: Option<usize>,
}

static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();

impl Registry {
    fn new(max_entries: Option<usize>, max_units: Option<usize>) -> Self {
        Self {
            symbols: Vec::new(),
            units: 0,
            max_entries,
            max_units,
        }
    }

    fn intern(&mut self, key: JsString, budget: &mut Budget) -> Result<JsSymbol, RegistryError> {
        for symbol in &self.symbols {
            budget.charge(1).map_err(|_| RegistryError::Work)?;
            let description = symbol.description().expect("registered key");
            if description.len() == key.len() {
                budget.charge(key.len()).map_err(|_| RegistryError::Work)?;
                if description == &key {
                    return Ok(symbol.clone());
                }
            }
        }
        budget.charge(1).map_err(|_| RegistryError::Work)?;
        let units = self
            .units
            .checked_add(key.len())
            .ok_or(RegistryError::Capacity)?;
        if self
            .max_entries
            .is_some_and(|limit| self.symbols.len() >= limit)
            || self.max_units.is_some_and(|limit| units > limit)
        {
            return Err(RegistryError::Capacity);
        }
        self.symbols
            .try_reserve(1)
            .map_err(|_| RegistryError::Capacity)?;
        let symbol = JsSymbol::new(Some(key));
        self.symbols.push(symbol.clone());
        self.units = units;
        Ok(symbol)
    }

    fn contains(&self, symbol: &JsSymbol, budget: &mut Budget) -> Result<bool, RegistryError> {
        for registered in &self.symbols {
            budget.charge(1).map_err(|_| RegistryError::Work)?;
            if registered == symbol {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl Realm {
    fn registry_work<T>(
        &mut self,
        span: Span,
        work: impl FnOnce(&mut Registry, &mut Budget) -> Result<T, RegistryError>,
    ) -> Result<T, Error> {
        self.tick(span)?;
        let mut budget = Budget::with_work_limit(self.remaining_steps);
        let result = {
            let mut registry = REGISTRY
                .get_or_init(|| Mutex::new(Registry::new(None, None)))
                .lock()
                .map_err(|_| Error::Limit {
                    span,
                    message: "Symbol registry lock unavailable".into(),
                })?;
            // Only bounded storage work runs while locked, never JavaScript or
            // an output conversion that could re-enter the realm.
            work(&mut registry, &mut budget)
        };
        self.remaining_steps = budget.remaining_work();
        result.map_err(|error| Error::Limit {
            span,
            message: match error {
                RegistryError::Work => "Symbol registry work limit exceeded",
                RegistryError::Capacity => "Symbol registry capacity limit exceeded",
            }
            .into(),
        })
    }

    pub(crate) fn symbol_for(&mut self, key: Value, span: Span) -> Result<Value, Error> {
        // Conversion can call Symbol.for itself; it must finish before locking.
        let key = self.string(key, span)?;
        if self
            .limits
            .max_string_units
            .is_some_and(|limit| key.len() > limit)
        {
            return Err(Error::Limit {
                span,
                message: "Symbol registry key length limit exceeded".into(),
            });
        }
        self.registry_work(span, |registry, budget| registry.intern(key, budget))
            .map(Value::Symbol)
    }

    pub(crate) fn symbol_key_for(&mut self, symbol: Value, span: Span) -> Result<Value, Error> {
        let Value::Symbol(symbol) = symbol else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Symbol.keyFor requires a Symbol primitive",
            ));
        };
        if self.symbol_is_registered(&symbol, span)? {
            // The immutable description is the registered key. Copy after unlock.
            self.symbol_string(&symbol, "", "", span).map(Value::String)
        } else {
            Ok(Value::Undefined)
        }
    }

    pub(crate) fn symbol_is_registered(
        &mut self,
        symbol: &JsSymbol,
        span: Span,
    ) -> Result<bool, Error> {
        self.registry_work(span, |registry, budget| registry.contains(symbol, budget))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_limits_preserve_existing_identities_and_do_not_insert_failed_keys() {
        let mut registry = Registry::new(Some(2), Some(3));
        let first = registry
            .intern(JsString::from("ab"), &mut Budget::new(100))
            .unwrap();
        assert_eq!(
            registry.intern(JsString::from("cd"), &mut Budget::new(100)),
            Err(RegistryError::Capacity)
        );
        assert_eq!(registry.symbols.len(), 1);
        let empty = registry
            .intern(JsString::from(""), &mut Budget::new(100))
            .unwrap();
        assert_eq!(
            registry.intern(JsString::from("x"), &mut Budget::new(100)),
            Err(RegistryError::Capacity)
        );
        assert_eq!(
            registry.intern(JsString::from("ab"), &mut Budget::new(100)),
            Ok(first)
        );
        assert_eq!(
            registry.intern(JsString::from(""), &mut Budget::new(100)),
            Ok(empty)
        );
        assert_eq!(registry.units, 2);
    }

    #[test]
    fn work_limits_precede_comparison_and_insertion_and_keys_preserve_utf16() {
        let mut registry = Registry::new(Some(3), Some(8));
        assert_eq!(
            registry.intern(JsString::from("x"), &mut Budget::new(0)),
            Err(RegistryError::Work)
        );
        assert!(registry.symbols.is_empty());
        let key = JsString::from_code_units(vec![0xD800, 0, 0xDC00]);
        let symbol = registry.intern(key.clone(), &mut Budget::new(1)).unwrap();
        assert_eq!(
            registry.intern(key.clone(), &mut Budget::new(3)),
            Err(RegistryError::Work)
        );
        assert_eq!(
            registry.intern(key.clone(), &mut Budget::new(4)),
            Ok(symbol.clone())
        );
        assert_eq!(
            registry.contains(&symbol, &mut Budget::new(0)),
            Err(RegistryError::Work)
        );
        assert_eq!(registry.contains(&symbol, &mut Budget::new(1)), Ok(true));
        assert_eq!(
            registry.contains(&JsSymbol::new(Some(key)), &mut Budget::new(1)),
            Ok(false)
        );
        assert_eq!(registry.units, 3);
    }
}
