//! Iteration evaluation, completions, and environment restoration.

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn result(source: &str, expected: Value) {
    for prefix in ["", "'use strict'; "] {
        let actual = Realm::default().eval(&format!("{prefix}{source}")).unwrap();
        assert!(
            actual.same_value(&expected),
            "{prefix}{source}: {actual:?} != {expected:?}"
        );
    }
}

#[test]
fn loops_follow_condition_and_body_order() {
    result("let i = 0; while (i < 3) i = i + 1; i", Value::Number(3.0));
    result(
        "let i = 0; do i = i + 1; while (i < 3); i",
        Value::Number(3.0),
    );
    result(
        "let i = 0; do i = i + 1; while (false); i",
        Value::Number(1.0),
    );
    result("let i = 0; while (false) i = i + 1; i", Value::Number(0.0));
    result("let i = 0; while ((i = i + 1) < 3) ; i", Value::Number(3.0));
    result(
        "let i = 0; do ; while ((i = i + 1) < 3); i",
        Value::Number(3.0),
    );
    result(
        "let trace = ''; let i = 0; while ((trace = trace + 't', i < 2)) { trace = trace + 'b'; i = i + 1; } trace",
        Value::String(JsString::from("tbtbt")),
    );
    result(
        "let trace = ''; let i = 0; do { trace = trace + 'b'; i = i + 1; } while ((trace = trace + 't', i < 2)); trace",
        Value::String(JsString::from("btbt")),
    );
}

#[test]
fn loop_results_are_body_values_not_condition_values() {
    for source in [
        "99; while (false) 1;",
        "99; do ; while (false);",
        "99; do {} while (false)",
    ] {
        result(source, Value::Undefined);
    }
    result(
        "let i = 0; while (i < 2) { i = i + 1; 7; let empty; ; {} }",
        Value::Number(7.0),
    );
    result(
        "let i = 0; do { i = i + 1; 8; let empty; ; {} } while (i < 2)",
        Value::Number(8.0),
    );
    result("do { 7; undefined; } while (false)", Value::Undefined);
    result("do -0; while (false)", Value::Number(-0.0));
    result("do NaN; while (false)", Value::Number(f64::NAN));
}

#[test]
fn conditions_use_to_boolean() {
    for condition in ["false", "undefined", "null", "0", "-0", "NaN", "''"] {
        result(
            &format!("let i = 0; while ({condition}) i = 99; i"),
            Value::Number(0.0),
        );
        result(
            &format!("let i = 0; do i = i + 1; while ({condition}); i"),
            Value::Number(1.0),
        );
    }
    result(
        "let condition = '0'; while (condition) { condition = ''; 5; }",
        Value::Number(5.0),
    );
}

#[test]
fn each_block_iteration_has_fresh_lexical_bindings() {
    result(
        "let i = 0; while (i < 3) { let next = i + 1; i = next; } i",
        Value::Number(3.0),
    );
    result(
        "let i = 0; do { const next = i + 1; i = next; } while (i < 3); i",
        Value::Number(3.0),
    );
    result(
        "let i = 0; while (i < 2) { let x = 0; while (x < 2) x = x + 1; i = i + x; } i",
        Value::Number(2.0),
    );
    assert!(matches!(
        Realm::default().eval("while (true) { x; let x; }"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
}

#[test]
fn throws_and_failed_conditions_restore_block_scopes() {
    for source in [
        "while (true) { let x = 2; throw x; }",
        "do { let x = 2; throw x; } while (missing)",
    ] {
        let mut realm = Realm::default();
        realm.eval("let x = 1").unwrap();
        assert_eq!(realm.eval(source), Err(Error::Thrown(Value::Number(2.0))));
        assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
    }
    let mut realm = Realm::default();
    realm.eval("let x = 1").unwrap();
    assert!(matches!(
        realm.eval("{ let x = 2; do ; while (missing); }"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
    assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
}

#[test]
fn infinite_loops_exhaust_the_host_budget_and_restore_scopes() {
    for source in [
        "while (true) ;",
        "do ; while (true)",
        "while (true) { let x = 2; }",
        "do { let x = 2; } while (true)",
    ] {
        let mut realm = Realm::new(Limits {
            max_steps: 100,
            ..Limits::default()
        });
        realm.eval("let x = 1").unwrap();
        assert!(
            matches!(realm.eval(source), Err(Error::Limit { .. })),
            "{source}"
        );
        assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
    }
}
