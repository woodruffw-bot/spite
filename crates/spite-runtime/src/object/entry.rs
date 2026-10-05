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

impl Trace for Entry {
    fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        let (object, environment) = match self {
            Self::Object(object) => (Some(object), None),
            Self::Environment(environment) => (None, Some(environment)),
        };
        object
            .into_iter()
            .flat_map(Trace::trace)
            .chain(environment.into_iter().flat_map(Trace::trace))
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
