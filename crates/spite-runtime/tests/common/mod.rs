//! Shared Runtime integration test helpers.

mod realm_entries;
pub use realm_entries::REALM_ENTRIES;

pub mod regexp_oracle;

/// Host-abort controls must remain independent of partially implemented globals.
pub trait UnavailableHostRealm {
    fn with_unavailable_host() -> Self;
    fn with_unavailable_host_limits(limits: spite_runtime::Limits) -> Self;
}

impl UnavailableHostRealm for spite_runtime::Realm {
    fn with_unavailable_host() -> Self {
        Self::with_unavailable_host_limits(spite_runtime::Limits::default())
    }

    fn with_unavailable_host_limits(limits: spite_runtime::Limits) -> Self {
        let mut realm = Self::new(limits);
        realm.reserve_unsupported_global("spiteUnavailableHost");
        realm
    }
}
