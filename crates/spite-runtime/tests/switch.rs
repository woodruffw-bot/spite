//! Switch selection, completion values, lexical scope, and abrupt exits.

use spite_core::{DiagnosticKind, JsString};
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
fn first_strictly_equal_case_is_selected() {
    for (input, cases, expected) in [
        ("1", "case '1': 9; break; case 1: 7; break;", 7.0),
        ("1", "case true: 9; break; case 1: 7; break;", 7.0),
        (
            "null",
            "case undefined: 9; break; case null: 7; break;",
            7.0,
        ),
        ("NaN", "case NaN: 9; break; default: 7;", 7.0),
        ("-0", "case 0: 7; break; default: 9;", 7.0),
        ("1", "case 1: 7; break; case 1: 9;", 7.0),
        ("'💩'", "case '\\ud83d\\udca9': 7; break; default: 9;", 7.0),
    ] {
        result(
            &format!("switch ({input}) {{ {cases} }}"),
            Value::Number(expected),
        );
    }
    result(
        "switch ('\\ud800') { case '\\ud800': -0; break; }",
        Value::Number(-0.0),
    );
}

#[test]
fn default_is_selected_only_after_all_selectors_fail() {
    for (input, trace) in [(1, "daAxBC"), (2, "dabBC"), (3, "dabcC"), (9, "dabcxBC")] {
        let source = format!(
            "let trace = ''; switch ((trace = trace + 'd', {input})) {{
            case (trace = trace + 'a', 1): trace = trace + 'A';
            default: trace = trace + 'x';
            case (trace = trace + 'b', 2): trace = trace + 'B';
            case (trace = trace + 'c', 3): trace = trace + 'C';
        }} trace"
        );
        result(&source, Value::String(JsString::from(trace)));
    }
    result(
        "switch (1) { default: throw 99; case 1: 7; }",
        Value::Number(7.0),
    );
    result(
        "switch (9) { case 1: throw 99; default: 7; }",
        Value::Number(7.0),
    );
    result(
        "switch (1) { case 1: 7; break; default: throw 99; case missing: ; }",
        Value::Number(7.0),
    );
}

#[test]
fn discriminant_is_evaluated_once_and_selectors_stop_at_the_match() {
    result(
        "let x = 0; switch (x = x + 1) { case (x = x + 1): 9; break; case 1: 7; break; }",
        Value::Number(7.0),
    );
    result(
        "let x = 0; switch (x = x + 1) { case (x = x + 1): break; case 1: break; } x",
        Value::Number(2.0),
    );
    result(
        "switch (1) { case 1: 7; case missing: ; default: ; }",
        Value::Number(7.0),
    );
    assert!(matches!(
        Realm::default().eval("switch (1) { case 0: ; default: 7; case missing: ; }"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
}

#[test]
fn empty_clauses_preserve_values_and_abrupt_completions() {
    for source in [
        "99; switch (0) {}",
        "99; switch (0) { case 1: 7; }",
        "99; switch (0) { default: ; }",
        "switch (0) { default: break; }",
        "switch (0) { case 0: 7; case 1: if (true) break; }",
        "a: { 7; switch (0) { default: break a; } }",
    ] {
        result(source, Value::Undefined);
    }
    for source in [
        "switch (0) { case 0: 7; case 1: ; default: {} }",
        "switch (0) { case 0: 7; case 1: break; }",
        "switch (0) { case 0: 7; case 1: let x; default: ; }",
        "a: switch (0) { case 0: 7; default: { break a; } }",
    ] {
        result(source, Value::Number(7.0));
    }
    result(
        "switch (0) { case 0: 7; case 1: undefined; }",
        Value::Undefined,
    );
    result(
        "switch (0) { default: NaN; break; }",
        Value::Number(f64::NAN),
    );
}

#[test]
fn switch_break_is_local_and_continue_reaches_the_enclosing_loop() {
    result(
        "let count = 0; for (let i = 0; i < 3; i = i + 1) { switch (i) { default: break; } count = count + 1; } count",
        Value::Number(3.0),
    );
    result(
        "let count = 0; for (let i = 0; i < 3; i = i + 1) { switch (i) { case 1: continue; default: ; } count = count + 1; } count",
        Value::Number(2.0),
    );
    result(
        "for (let i = 0; i < 3; i = i + 1) { switch (i) { default: i; continue; } }",
        Value::Number(2.0),
    );
    result(
        "a: for (let i = 0; i < 3; i = i + 1) { switch (i) { default: while (true) { i; continue a; } } }",
        Value::Number(2.0),
    );
    result(
        "switch (0) { default: switch (1) { default: break; } 7; break; }",
        Value::Number(7.0),
    );
    result(
        "switch (0) { default: while (true) break; 7; break; }",
        Value::Number(7.0),
    );
    result(
        "a: for (;;) { switch (0) { default: 7; break a; } throw 99; }",
        Value::Number(7.0),
    );
}

#[test]
fn discriminant_uses_outer_scope_and_selectors_use_case_scope() {
    result(
        "let x = 1; switch (x) { case 1: let x = 7; x; break; }",
        Value::Number(7.0),
    );
    result(
        "let x = 1; switch (x) { case 1: let x = 7; } x",
        Value::Number(1.0),
    );
    for source in [
        "let x = 1; switch (x) { case x: let x = 7; }",
        "switch (0) { default: let x = 1; case x: ; }",
        "let x = 7; switch (1) { case 0: let x = 1; case 1: x; }",
        "switch (0) { case typeof x: ; default: let x; }",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Exception {
                    kind: ExceptionKind::ReferenceError,
                    ..
                })
            ),
            "{source}"
        );
    }
    result(
        "switch (0) { case 0: let x = 7; default: x; case x: ; }",
        Value::Number(7.0),
    );
}

#[test]
fn reached_clauses_share_bindings_while_nested_blocks_can_shadow() {
    result(
        "switch (0) { case 0: let x = 7; case 1: x; }",
        Value::Number(7.0),
    );
    result(
        "switch (0) { case 0: let x = 7; case 1: x = 8; x; }",
        Value::Number(8.0),
    );
    result(
        "switch (0) { case 0: let x = 7; case 1: { let x = 99; } x; }",
        Value::Number(7.0),
    );
    result(
        "switch (0) { default: let undefined = 7; undefined; }",
        Value::Number(7.0),
    );
    assert!(matches!(
        Realm::default().eval("switch (0) { case 0: const x = 7; case 1: x = 8; }"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert!(matches!(
        Realm::default().eval("switch (0) { default: let x = 1; } x"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
}

#[test]
fn all_abrupt_exits_restore_the_outer_environment() {
    for (source, expected) in [
        (
            "switch (missing) { default: let x = 2; }",
            ExceptionKind::ReferenceError,
        ),
        (
            "switch (0) { case missing: let x = 2; }",
            ExceptionKind::ReferenceError,
        ),
        (
            "switch (0) { default: let x = missing; }",
            ExceptionKind::ReferenceError,
        ),
        (
            "switch (0) { default: const x = 2; x = 3; }",
            ExceptionKind::TypeError,
        ),
    ] {
        let mut realm = Realm::default();
        realm.eval("let x = 1").unwrap();
        assert!(
            matches!(realm.eval(source), Err(Error::Exception { kind, .. }) if kind == expected)
        );
        assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
    }
    let mut realm = Realm::default();
    realm.eval("let x = 1").unwrap();
    assert_eq!(
        realm.eval("switch (0) { default: let x = 2; throw x; }"),
        Err(Error::Thrown(Value::Number(2.0)))
    );
    assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
    assert!(matches!(
        realm.eval("switch (0) { case Math: let x; }"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
    result(
        "let x = 1; a: switch (0) { default: let x = 2; break a; } x",
        Value::Number(1.0),
    );
    result(
        "let x = 0; a: do { switch (0) { default: let x = 99; continue a; } } while ((x = x + 1) < 3); x",
        Value::Number(3.0),
    );
}

#[test]
fn empty_fallthrough_clauses_consume_the_host_budget() {
    let clauses = (1..2_000).map(|i| format!("case {i}:")).collect::<String>();
    let source = format!("switch (0) {{ case 0: let x = 2; {clauses} }}");
    let mut realm = Realm::new(Limits {
        max_steps: Some(1_000),
        ..Limits::default()
    });
    realm.eval("let x = 1").unwrap();
    assert!(matches!(realm.eval(&source), Err(Error::Limit { .. })));
    assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
}

#[test]
fn switch_early_errors_precede_all_execution() {
    for source in [
        "x = 99; switch (0) { case 0: let y; case 1: let y; }",
        "x = 99; switch (0) { default: ; default: ; }",
        "x = 99; switch (0) { default: continue; }",
    ] {
        let mut realm = Realm::default();
        realm.eval("let x = 1").unwrap();
        assert!(
            matches!(realm.eval(source), Err(Error::Parse(d)) if d.kind == DiagnosticKind::Syntax)
        );
        assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
    }
}
