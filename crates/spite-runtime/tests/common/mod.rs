//! Shared Runtime integration test helpers.

use spite_runtime::{PromiseJobOutcome, Realm, Value};

mod realm_entries;
pub use realm_entries::REALM_ENTRIES;

pub mod regexp_captures;
pub mod regexp_matches;
pub mod regexp_oracle;
pub mod unicode_samples;

/// Evaluates a behavioral assertion in a fresh Realm and reports its source.
#[track_caller]
pub fn assert_true(source: &str) {
    assert_true_in(&mut Realm::default(), source);
}

/// Evaluates a behavioral assertion in an existing Realm and reports its source.
#[track_caller]
pub fn assert_true_in(realm: &mut Realm, source: &str) {
    assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
}

/// Runs queued promise jobs, failing the test if a job reports an error.
#[track_caller]
pub fn drain_promise_jobs(realm: &mut Realm) {
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => (),
            PromiseJobOutcome::Reported(error) => panic!("{error:?}"),
        }
    }
}

/// Ends the job's kept-object lifetime and runs a full collection.
#[track_caller]
pub fn checkpoint_and_collect(realm: &mut Realm) {
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
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
