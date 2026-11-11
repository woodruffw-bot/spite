//! Shared Runtime integration test helpers.

mod realm_entries;
pub use realm_entries::REALM_ENTRIES;

pub mod regexp_oracle;

/// Evaluates a behavioral assertion in a fresh Realm and reports its source.
#[track_caller]
pub fn assert_true(source: &str) {
    assert_eq!(
        spite_runtime::Realm::default().eval(source),
        Ok(spite_runtime::Value::Boolean(true)),
        "{source}"
    );
}

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
