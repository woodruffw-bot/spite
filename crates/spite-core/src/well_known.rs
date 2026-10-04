//! Shared edition-17 well-known Symbol identities (6.1.5.1).

use crate::{JsString, JsSymbol};
use std::sync::OnceLock;

/// A well-known symbol named by ECMA-262's edition-17 symbol table.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WellKnownSymbol {
    /// The default asynchronous iterator method.
    AsyncIterator,
    /// The custom instance-check method.
    HasInstance,
    /// The Array concatenation spreading flag.
    IsConcatSpreadable,
    /// The default iterator method.
    Iterator,
    /// The regular expression matching method/brand hook.
    Match,
    /// The all-matches iterator method.
    MatchAll,
    /// The string replacement method.
    Replace,
    /// The string search method.
    Search,
    /// The derived-result constructor selection hook.
    Species,
    /// The string splitting method.
    Split,
    /// The object-to-primitive conversion method.
    ToPrimitive,
    /// The Object.prototype.toString tag.
    ToStringTag,
    /// The property-exclusion table for with environments.
    Unscopables,
}

impl WellKnownSymbol {
    /// All well-known symbols in edition-17 table order.
    pub const ALL: [Self; 13] = [
        Self::AsyncIterator,
        Self::HasInstance,
        Self::IsConcatSpreadable,
        Self::Iterator,
        Self::Match,
        Self::MatchAll,
        Self::Replace,
        Self::Search,
        Self::Species,
        Self::Split,
        Self::ToPrimitive,
        Self::ToStringTag,
        Self::Unscopables,
    ];

    /// The corresponding property name on the Symbol constructor.
    pub const fn name(self) -> &'static str {
        match self {
            Self::AsyncIterator => "asyncIterator",
            Self::HasInstance => "hasInstance",
            Self::IsConcatSpreadable => "isConcatSpreadable",
            Self::Iterator => "iterator",
            Self::Match => "match",
            Self::MatchAll => "matchAll",
            Self::Replace => "replace",
            Self::Search => "search",
            Self::Species => "species",
            Self::Split => "split",
            Self::ToPrimitive => "toPrimitive",
            Self::ToStringTag => "toStringTag",
            Self::Unscopables => "unscopables",
        }
    }

    /// Returns this immutable identity, shared across realms and host threads.
    ///
    /// The fixed table is initialized once without user code. These identities
    /// are separate from fresh symbols with the same description and from the
    /// runtime's global symbol registry.
    pub fn symbol(self) -> JsSymbol {
        static SYMBOLS: OnceLock<[JsSymbol; 13]> = OnceLock::new();
        SYMBOLS.get_or_init(|| {
            Self::ALL.map(|key| {
                JsSymbol::new(Some(JsString::from(
                    format!("Symbol.{}", key.name()).as_str(),
                )))
            })
        })[self as usize]
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn well_known_identities_are_unique_stable_and_distinct_from_descriptive_lookalikes() {
        let mut identities = HashSet::new();
        for key in WellKnownSymbol::ALL {
            let symbol = key.symbol();
            assert_eq!(symbol, key.symbol());
            assert!(identities.insert(symbol.clone()));
            assert_eq!(
                symbol.description(),
                Some(&JsString::from(format!("Symbol.{}", key.name()).as_str()))
            );
            assert_ne!(symbol, JsSymbol::new(symbol.description().cloned()));
        }
        assert_eq!(identities.len(), 13);
        assert_eq!(WellKnownSymbol::HasInstance.name(), "hasInstance");
        assert_eq!(WellKnownSymbol::ToPrimitive.name(), "toPrimitive");
        assert_eq!(WellKnownSymbol::Iterator.name(), "iterator");
    }

    #[test]
    fn initialization_and_identity_are_shared_between_threads() {
        let threads: Vec<_> = (0..8)
            .map(|_| std::thread::spawn(|| WellKnownSymbol::ALL.map(WellKnownSymbol::symbol)))
            .collect();
        for thread in threads {
            assert_eq!(
                thread.join().unwrap(),
                WellKnownSymbol::ALL.map(WellKnownSymbol::symbol)
            );
        }
    }
}
