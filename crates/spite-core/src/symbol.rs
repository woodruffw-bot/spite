//! Immutable symbol identity and property-key representation (6.1.5, 6.1.7).

use crate::JsString;
use std::{
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

/// An immutable ECMAScript Symbol identity with an optional UTF-16 description.
///
/// Cloning preserves identity. Description text never determines equality, and
/// an absent description is distinct from an empty description. Symbol registry
/// lookup and well-known identities belong to the runtime, not this constructor.
#[derive(Clone)]
pub struct JsSymbol(Arc<Option<JsString>>);

impl JsSymbol {
    /// Creates a fresh symbol, distinct from every other live symbol.
    pub fn new(description: Option<JsString>) -> Self {
        Self(Arc::new(description))
    }

    /// Borrows the description without copying its UTF-16 code units.
    pub fn description(&self) -> Option<&JsString> {
        self.0.as_ref().as_ref()
    }
}

impl PartialEq for JsSymbol {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for JsSymbol {}

impl Hash for JsSymbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // The allocation remains alive for every copy of this identity. Pointer
        // hashing is safe and process-local; addresses are never JS values.
        Arc::as_ptr(&self.0).hash(state);
    }
}

impl fmt::Debug for JsSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JsSymbol")
            .field("description", &self.description())
            .finish_non_exhaustive()
    }
}

/// An Object property key; symbols and strings occupy distinct key spaces.
///
/// This type represents an already converted key. JavaScript ToPropertyKey is
/// a runtime operation and may execute user code or complete abruptly.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PropertyKey {
    /// A UTF-16 string key, including array indices and ordinary names.
    String(JsString),
    /// A symbol key compared by identity, never by description text.
    Symbol(JsSymbol),
}

/// A borrowed property key, allowing lookup without copying string storage.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PropertyKeyRef<'a> {
    /// A borrowed UTF-16 string key.
    String(&'a JsString),
    /// A borrowed symbol identity.
    Symbol(&'a JsSymbol),
}

impl<'a> PropertyKeyRef<'a> {
    /// Borrows a string key, or returns `None` for a symbol key.
    pub fn as_string(self) -> Option<&'a JsString> {
        match self {
            Self::String(value) => Some(value),
            Self::Symbol(_) => None,
        }
    }
}

impl<'a> From<&'a JsString> for PropertyKeyRef<'a> {
    fn from(value: &'a JsString) -> Self {
        Self::String(value)
    }
}

impl<'a> From<&'a JsSymbol> for PropertyKeyRef<'a> {
    fn from(value: &'a JsSymbol) -> Self {
        Self::Symbol(value)
    }
}

impl<'a> From<&'a PropertyKey> for PropertyKeyRef<'a> {
    fn from(value: &'a PropertyKey) -> Self {
        match value {
            PropertyKey::String(value) => Self::String(value),
            PropertyKey::Symbol(value) => Self::Symbol(value),
        }
    }
}

impl<'a> From<&PropertyKeyRef<'a>> for PropertyKeyRef<'a> {
    fn from(value: &PropertyKeyRef<'a>) -> Self {
        *value
    }
}

impl PropertyKey {
    /// Borrows a string key, or returns `None` for a symbol key.
    pub fn as_string(&self) -> Option<&JsString> {
        match self {
            Self::String(value) => Some(value),
            Self::Symbol(_) => None,
        }
    }

    /// Borrows a symbol key, or returns `None` for a string key.
    pub fn as_symbol(&self) -> Option<&JsSymbol> {
        match self {
            Self::Symbol(value) => Some(value),
            Self::String(_) => None,
        }
    }
}

impl From<JsString> for PropertyKey {
    fn from(value: JsString) -> Self {
        Self::String(value)
    }
}

impl From<JsSymbol> for PropertyKey {
    fn from(value: JsSymbol) -> Self {
        Self::Symbol(value)
    }
}

impl From<&str> for PropertyKey {
    fn from(value: &str) -> Self {
        Self::String(JsString::from(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn equal_descriptions_do_not_alias_and_clones_preserve_identity() {
        for description in [None, Some(JsString::from("")), Some(JsString::from("name"))] {
            let first = JsSymbol::new(description.clone());
            let second = JsSymbol::new(description.clone());
            let copy = first.clone();
            assert_ne!(first, second);
            assert_eq!(first, copy);
            assert_eq!(first.description(), description.as_ref());
            let mut set = HashSet::new();
            assert!(set.insert(first));
            assert!(!set.insert(copy));
            assert!(set.insert(second));
            assert_eq!(set.len(), 2);
        }
    }

    #[test]
    fn descriptions_preserve_absence_empty_strings_and_lone_surrogates() {
        assert_eq!(JsSymbol::new(None).description(), None);
        let empty = JsSymbol::new(Some(JsString::from("")));
        assert_eq!(empty.description().unwrap().code_units(), &[]);
        let raw = JsString::from_code_units(vec![0xD800, 0, 0xDC00]);
        let symbol = JsSymbol::new(Some(raw.clone()));
        assert_eq!(symbol.description(), Some(&raw));
    }

    #[test]
    fn dropping_the_original_keeps_cloned_identity_and_description_alive() {
        let (copy, weak) = {
            let original = JsSymbol::new(Some(JsString::from("kept")));
            (original.clone(), Arc::downgrade(&original.0))
        };
        assert_eq!(copy.description(), Some(&JsString::from("kept")));
        assert!(weak.upgrade().is_some());
        drop(copy);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn symbol_identity_survives_thread_transfer_without_global_counters() {
        let symbol = JsSymbol::new(Some(JsString::from("shared")));
        let copy = symbol.clone();
        let (returned, fresh) =
            std::thread::spawn(move || (copy, JsSymbol::new(Some(JsString::from("shared")))))
                .join()
                .unwrap();
        assert_eq!(symbol, returned);
        assert_ne!(symbol, fresh);
    }

    #[test]
    fn property_keys_separate_symbol_identity_from_equal_string_names() {
        for name in ["0", "length", "__proto__", "Symbol.iterator", ""] {
            let string = PropertyKey::from(name);
            let symbol = PropertyKey::from(JsSymbol::new(Some(JsString::from(name))));
            let other = PropertyKey::from(JsSymbol::new(Some(JsString::from(name))));
            assert_ne!(string, symbol);
            assert_ne!(symbol, other);
            let mut map = HashMap::new();
            map.insert(string.clone(), 1);
            map.insert(symbol.clone(), 2);
            map.insert(other.clone(), 3);
            assert_eq!(map.get(&PropertyKey::from(name)), Some(&1));
            assert_eq!(map.get(&symbol), Some(&2));
            assert_eq!(map.get(&other), Some(&3));
            assert!(string.as_string().is_some());
            assert!(string.as_symbol().is_none());
            assert!(symbol.as_string().is_none());
            assert!(symbol.as_symbol().is_some());
        }
    }

    #[test]
    fn string_keys_keep_exact_utf16_equality() {
        let units = vec![0xD800, 0x0061];
        let key = PropertyKey::from(JsString::from_code_units(units.clone()));
        assert_eq!(key, PropertyKey::from(JsString::from_code_units(units)));
        assert_ne!(key, PropertyKey::from("�a"));
        assert_eq!(
            PropertyKey::from(JsString::from("name")),
            PropertyKey::from("name")
        );
    }
}
