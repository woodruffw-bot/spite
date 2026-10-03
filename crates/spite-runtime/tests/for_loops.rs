//! For-loop evaluation order, completions, and resource limits.

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
fn initializer_test_body_and_update_are_ordered() {
    result(
        "let i = 99; for (i = 0; i < 3; i = i + 1) ; i",
        Value::Number(3.0),
    );
    result(
        "let trace = ''; let i = 0; for (trace = 'i'; (trace = trace + 't', i < 2); (trace = trace + 'u', i = i + 1)) { trace = trace + 'b'; } trace",
        Value::String(JsString::from("itbutbut")),
    );
    result(
        "let i = 0; let j = 0; for (i = 0, j = 1; i < 3; i = i + 1, j = j * 2) ; j",
        Value::Number(8.0),
    );
    result(
        "let i = 0; for (;;) { i = i + 1; if (i === 3) break; } i",
        Value::Number(3.0),
    );
    result("let i = 0; for (;i < 3;) i = i + 1; i", Value::Number(3.0));
}

#[test]
fn continue_runs_the_update_but_break_skips_it() {
    result(
        "let i = 0; let count = 0; for (;i < 4;i = i + 1) { if (i === 1) continue; count = count + 1; } count",
        Value::Number(3.0),
    );
    result("for (;;missing) break;", Value::Undefined);
    result("a: for (;;missing) break a;", Value::Undefined);
    result(
        "let i = 0; a: b: for (;i < 3;i = i + 1) continue a; i",
        Value::Number(3.0),
    );
    result(
        "let i = 0; a: for (;i < 3;i = i + 1) for (;;missing) continue a; i",
        Value::Number(3.0),
    );
    result(
        "let i = 0; a: while ((i = i + 1) < 3) for (;;missing) continue a; i",
        Value::Number(3.0),
    );
    assert_eq!(
        Realm::default().eval("for (;;missing) throw 7;"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
}

#[test]
fn only_body_values_contribute_to_the_loop_completion() {
    for source in [
        "99; for (7;false;8) 9;",
        "99; for (;;) break;",
        "for (;;) { 7; if (true) break; }",
        "let i = 0; for (;i < 3;i = i + 1) ;",
    ] {
        result(source, Value::Undefined);
    }
    result(
        "let i = 0; for (99;i < 3;i = i + 1) { 7; continue; }",
        Value::Number(7.0),
    );
    result("for (;;) { 7; { break; } }", Value::Number(7.0));
    result(
        "a: for (;;) { do { 7; break a; } while (missing); }",
        Value::Number(7.0),
    );
    result("for (;;) { -0; break; }", Value::Number(-0.0));
}

#[test]
fn false_conditions_skip_body_and_update() {
    for condition in ["false", "0", "-0", "NaN", "null", "undefined", "''"] {
        result(
            &format!("for (;{condition};missing) throw 99;"),
            Value::Undefined,
        );
    }
    result("for (; '0'; missing) break;", Value::Undefined);
}

#[test]
fn failed_headers_preserve_prior_effects_and_restore_outer_blocks() {
    for (source, expected) in [
        ("for (missing;;) ;", 0.0),
        ("for (x = 1;missing;) ;", 1.0),
        ("for (;;missing) { x = 2; }", 2.0),
    ] {
        let mut realm = Realm::default();
        realm.eval("let x = 0").unwrap();
        assert!(matches!(
            realm.eval(source),
            Err(Error::Exception {
                kind: ExceptionKind::ReferenceError,
                ..
            })
        ));
        assert_eq!(realm.eval("x"), Ok(Value::Number(expected)));
    }
    let mut realm = Realm::default();
    realm.eval("let x = 1").unwrap();
    assert!(matches!(
        realm.eval("{ let x = 2; for (;;missing) ; }"),
        Err(Error::Exception { .. })
    ));
    assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
}

#[test]
fn bodies_restore_scopes_before_updates_and_on_budget_exhaustion() {
    result(
        "let i = 0; for (;i < 3;i = i + 1) { let i = 99; continue; } i",
        Value::Number(3.0),
    );
    for source in [
        "for (;;) ;",
        "for (;;) continue;",
        "for (;;) { let x = 2; }",
        "a: for (;;) { let x = 2; continue a; }",
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
