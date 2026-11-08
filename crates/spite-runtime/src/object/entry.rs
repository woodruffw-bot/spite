//! One checked heap graph for objects and captured declarative environments.

use super::OrdinaryObject;
use crate::environment::Environment;
use spite_heap::{Handle, Trace};

#[derive(Debug)]
#[allow(
    clippy::large_enum_variant,
    reason = "Heap slots primarily hold objects; keep records inline and box optional collection payloads"
)]
pub(super) enum Entry {
    Object(OrdinaryObject),
    Environment(Environment),
}

// Keep only the selected iterator. Flat-mapping both optional variants stores
// two inner iterators for each and makes unoptimized collection frames large.
enum EntryTrace<O, E> {
    Object(O),
    Environment(E),
}

impl<'a, O, E> Iterator for EntryTrace<O, E>
where
    O: Iterator<Item = Option<&'a Handle>>,
    E: Iterator<Item = Option<&'a Handle>>,
{
    type Item = Option<&'a Handle>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Object(iterator) => iterator.next(),
            Self::Environment(iterator) => iterator.next(),
        }
    }
}

impl Trace for Entry {
    #[inline]
    fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        match self {
            Self::Object(object) => EntryTrace::Object(object.trace()),
            Self::Environment(environment) => EntryTrace::Environment(environment.trace()),
        }
    }

    fn ephemerons(&self) -> impl Iterator<Item = (&Handle, Option<&Handle>)> {
        let object = match self {
            Self::Object(object) => Some(object),
            Self::Environment(_) => None,
        };
        object.into_iter().flat_map(Trace::ephemerons)
    }

    fn retain_ephemerons(&mut self, retain: impl Fn(&Handle) -> bool) {
        if let Self::Object(object) = self {
            object.retain_ephemerons(retain);
        }
    }
}
