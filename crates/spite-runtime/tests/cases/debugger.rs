//! The host does not activate an implementation-defined debugging facility.

use spite_runtime::{Error, Limits, Realm, Value};

#[test]
fn debugger_has_an_empty_completion_and_no_effects() {
    for prefix in ["", "'use strict'; void 0; "] {
        for (source, expected) in [
            ("debugger", Value::Undefined),
            ("7; debugger;", Value::Number(7.0)),
            ("7; { debugger; }", Value::Number(7.0)),
            ("7; if (true) debugger;", Value::Undefined),
            ("try { 7; } finally { debugger; }", Value::Number(7.0)),
            ("let x = 1; debugger; x", Value::Number(1.0)),
        ] {
            assert_eq!(
                Realm::default().eval(&format!("{prefix}{source}")),
                Ok(expected)
            );
        }
    }
}

#[test]
fn debugger_statements_consume_the_host_work_budget() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(8),
        ..Limits::default()
    });
    assert!(matches!(
        realm.eval(&"debugger;".repeat(16)),
        Err(Error::Limit { .. })
    ));
}
