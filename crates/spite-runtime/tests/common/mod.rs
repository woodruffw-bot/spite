//! Shared Runtime integration test helpers.

use spite_runtime::{Realm, Value};

mod realm_entries;
pub use realm_entries::REALM_ENTRIES;

pub mod regexp_captures;
pub mod regexp_matches;
pub mod regexp_oracle;
pub mod unicode_samples;

/// Evaluates a behavioral assertion in a fresh Realm and reports its source.
#[track_caller]
pub fn assert_true(source: &str) {
    assert_eq!(
        spite_runtime::Realm::default().eval(source),
        Ok(spite_runtime::Value::Boolean(true)),
        "{source}"
    );
}

/// Checks an expression using SameValue, including NaN and the sign of zero.
#[track_caller]
pub fn assert_value(realm: &mut Realm, source: &str, expected: Value) {
    let actual = realm
        .eval(source)
        .unwrap_or_else(|error| panic!("{source}\nevaluation failed: {error:?}"));
    assert!(
        actual.same_value(&expected),
        "{source}\nexpected: {expected:?}\nactual: {actual:?}"
    );
}

/// Returns a string result for snapshots of structured JavaScript output.
#[track_caller]
pub fn eval_string(realm: &mut Realm, source: &str) -> String {
    match realm.eval(source) {
        Ok(Value::String(value)) => value.to_utf8().expect("snapshot text must be valid UTF-8"),
        result => panic!("{source}\nexpected a string result, got {result:?}"),
    }
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
