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

#[test]
fn break_skips_the_remaining_body_and_do_while_condition() {
    result(
        "let x = 0; while (true) { x = 1; break; x = 99; } x",
        Value::Number(1.0),
    );
    result(
        "let x = 0; do { x = 1; break; x = 99; } while (missing); x",
        Value::Number(1.0),
    );
    result("while (true) { break; throw 99; }", Value::Undefined);
    result("while (true) { break\nmissing; }", Value::Undefined);
}

#[test]
fn continue_resumes_at_the_condition() {
    result(
        "let i = 0; let count = 0; while ((i = i + 1) < 4) { if (i === 2) continue; count = count + 1; } count",
        Value::Number(2.0),
    );
    result(
        "let i = 0; do { continue; throw 99; } while ((i = i + 1) < 3); i",
        Value::Number(3.0),
    );
    result(
        "let i = 0; while ((i = i + 1) < 3) { continue\nmissing; } i",
        Value::Number(3.0),
    );
    assert!(matches!(
        Realm::default().eval("do continue; while (missing);"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
}

#[test]
fn abrupt_completions_keep_statement_list_values() {
    for source in [
        "while (true) { 7; break; }",
        "while (true) { 7; { break; } }",
        "do { 7; break; } while (missing)",
        "do { 7; { continue; } } while (false)",
        "let i = 0; while ((i = i + 1) < 3) { 7; continue; }",
    ] {
        result(source, Value::Number(7.0));
    }
    result("while (true) { -0; break; }", Value::Number(-0.0));
    result(
        "do { NaN; continue; } while (false)",
        Value::Number(f64::NAN),
    );
    result("99; while (true) break;", Value::Undefined);
    result("99; do continue; while (false)", Value::Undefined);
}

#[test]
fn if_applies_update_empty_to_abrupt_completions() {
    // ECMA-262 14.6.2 fills an empty break/continue with undefined before the
    // containing statement list can inherit the earlier numeric value.
    for source in [
        "while (true) { 7; if (true) break; }",
        "while (true) { 7; if (false) ; else break; }",
        "do { 7; if (true) continue; } while (false)",
        "do { 7; if (false) ; else continue; } while (false)",
        "while (true) { 7; { if (true) break; } }",
    ] {
        result(source, Value::Undefined);
    }
    result(
        "while (true) { 7; if (true) { 8; break; } }",
        Value::Number(8.0),
    );
    result(
        "do { 7; if (true) { 8; continue; } } while (false)",
        Value::Number(8.0),
    );
}

#[test]
fn nested_loops_consume_only_their_own_unlabelled_control() {
    result(
        "let i = 0; let count = 0; while (i < 3) { i = i + 1; do { break; } while (missing); count = count + 1; } count",
        Value::Number(3.0),
    );
    result(
        "let i = 0; let count = 0; while (i < 3) { i = i + 1; let j = 0; while ((j = j + 1) < 3) continue; count = count + j; } count",
        Value::Number(9.0),
    );
}

#[test]
fn control_transfers_restore_scopes() {
    result(
        "let x = 1; while (true) { let x = 2; { let x = 3; break; } } x",
        Value::Number(1.0),
    );
    result(
        "let x = 1; let i = 0; while ((i = i + 1) < 3) { let x = 2; continue; } x",
        Value::Number(1.0),
    );
    result(
        "let x = 0; do { let x = 99; continue; } while ((x = x + 1) < 3); x",
        Value::Number(3.0),
    );
    for source in [
        "while (true) { let x = 2; continue; }",
        "do { let x = 2; continue; } while (true)",
    ] {
        let mut realm = Realm::new(Limits {
            max_steps: 100,
            ..Limits::default()
        });
        realm.eval("let x = 1").unwrap();
        assert!(matches!(realm.eval(source), Err(Error::Limit { .. })));
        assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
    }
}

#[test]
fn invalid_loop_control_is_rejected_before_execution_or_instantiation() {
    for source in [
        "x = 1; break;",
        "x = 1; if (false) continue;",
        "let y; while (false) ; break;",
    ] {
        let mut realm = Realm::default();
        realm.eval("let x = 0").unwrap();
        assert!(
            matches!(realm.eval(source), Err(Error::Parse(d)) if d.kind == spite_core::DiagnosticKind::Syntax)
        );
        assert_eq!(realm.eval("x"), Ok(Value::Number(0.0)));
        assert_eq!(realm.eval("let y = 2; y"), Ok(Value::Number(2.0)));
    }
}

#[test]
fn let_can_be_an_expression_body_in_non_strict_code() {
    for (source, expected) in [
        ("let = 7; do let\nwhile (false)", 7.0),
        ("let = 7; let i = 0; while ((i = i + 1) < 3) let\n{}", 7.0),
        ("let = 7; if (true) let\n{}", 7.0),
        ("while (false) let\nx = 1; x", 1.0),
        ("if (false) let\nx = 2; x", 2.0),
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Ok(Value::Number(expected)),
            "{source}"
        );
    }
}
