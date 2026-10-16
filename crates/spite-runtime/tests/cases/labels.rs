//! Labelled completion propagation and environment restoration.

use spite_runtime::{Error, Limits, Realm, Value};

fn result(source: &str, expected: Value) {
    for prefix in ["", "'use strict'; "] {
        let actual = Realm::default().eval(&format!("{prefix}{source}")).unwrap();
        assert!(
            actual.same_value(&expected),
            "{source}: {actual:?} != {expected:?}"
        );
    }
}

#[test]
fn break_exits_the_target_statement() {
    result(
        "let x = 0; a: { x = 1; break a; x = 99; } x",
        Value::Number(1.0),
    );
    result(
        "let x = 0; a: b: { x = 1; break a; x = 99; } x",
        Value::Number(1.0),
    );
    result("a: if (true) { 7; break a; throw 99; }", Value::Number(7.0));
    result(
        "let x = 0; a: { b: { break b; } x = 2; } x",
        Value::Number(2.0),
    );
    result("a: do { 7; break a; } while (missing)", Value::Number(7.0));
}

#[test]
fn labelled_continue_resumes_the_target_loop() {
    result(
        "let i = 0; let count = 0; outer: while ((i = i + 1) < 4) { do { count = count + 1; continue outer; } while (missing); throw 99; } count",
        Value::Number(3.0),
    );
    result(
        "let i = 0; outer: do { while (true) continue outer; } while ((i = i + 1) < 3); i",
        Value::Number(3.0),
    );
    for target in ["a", "b"] {
        result(
            &format!(
                "let i = 0; a: b: while ((i = i + 1) < 3) {{ continue {target}; throw 99; }} i"
            ),
            Value::Number(3.0),
        );
        result(
            &format!(
                "let i = 0; a: b: do {{ continue {target}; throw 99; }} while ((i = i + 1) < 3); i"
            ),
            Value::Number(3.0),
        );
    }
}

#[test]
fn inner_control_does_not_exit_an_outer_loop() {
    result(
        "let i = 0; let count = 0; a: while (i < 3) { i = i + 1; b: while (true) break b; count = count + 1; } count",
        Value::Number(3.0),
    );
    result(
        "let i = 0; let count = 0; a: while (i < 3) { i = i + 1; let j = 0; b: do { j = j + 1; continue b; } while (j < 2); count = count + j; } count",
        Value::Number(6.0),
    );
    result(
        "let i = 0; a: while (i < 3) { while (true) break; i = i + 1; } i",
        Value::Number(3.0),
    );
}

#[test]
fn labels_preserve_empty_completions_and_loop_values() {
    result("7; a: { break a; }", Value::Number(7.0));
    result("7; a: b: ;", Value::Number(7.0));
    result("7; a: { undefined; break a; }", Value::Undefined);
    result("7; a: while (true) break a;", Value::Undefined);
    result("a: { 7; if (true) break a; }", Value::Undefined);
    result(
        "a: while (true) { 7; while (true) break a; }",
        Value::Undefined,
    );
    result(
        "a: while (true) { while (true) { 7; break a; } }",
        Value::Number(7.0),
    );
    result(
        "a: do { while (true) { 7; continue a; } } while (false)",
        Value::Number(7.0),
    );
    result(
        "a: do { 7; while (true) continue a; } while (false)",
        Value::Undefined,
    );
    result("a: { -0; break a; }", Value::Number(-0.0));
    result("a: while (true) { NaN; break a; }", Value::Number(f64::NAN));
}

#[test]
fn line_terminators_change_the_target() {
    for separator in ["\n", "\r", "\u{2028}", "/*\n*/"] {
        result(
            &format!("let x = 0; a: {{ while (true) {{ break{separator}a; }} x = 1; }} x"),
            Value::Number(1.0),
        );
    }
    result(
        "let x = 0; a: { while (true) { break /*no newline*/ a; } x = 1; } x",
        Value::Number(0.0),
    );
}

#[test]
fn escaping_a_label_restores_all_nested_scopes() {
    result(
        "let x = 1; a: while (true) { let x = 2; while (true) { let x = 3; break a; } } x",
        Value::Number(1.0),
    );
    result(
        "let x = 0; a: do { let x = 99; while (true) { let x = 100; continue a; } } while ((x = x + 1) < 3); x",
        Value::Number(3.0),
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(100),
        ..Limits::default()
    });
    realm.eval("let x = 1").unwrap();
    assert!(matches!(
        realm.eval("a: while (true) { let x = 2; while (true) continue a; }"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
    assert_eq!(
        realm.eval("a: { let x = 2; throw x; }"),
        Err(Error::Thrown(Value::Number(2.0)))
    );
    assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
}

#[test]
fn invalid_targets_are_rejected_before_effects() {
    for source in [
        "x = 1; a: { while (false) continue a; }",
        "x = 1; a: { a: ; }",
        "x = 1; while (false) break missing;",
    ] {
        let mut realm = Realm::default();
        realm.eval("let x = 0").unwrap();
        assert!(
            matches!(realm.eval(source), Err(Error::Parse(d)) if d.kind == spite_core::DiagnosticKind::Syntax)
        );
        assert_eq!(realm.eval("x"), Ok(Value::Number(0.0)));
    }
}

#[test]
fn escaped_names_and_binding_names_have_independent_identity() {
    result(r"let a = 7; \u0061: { a; break a; }", Value::Number(7.0));
    result(
        r"let i = 0; π: while ((i = i + 1) < 3) continue \u03c0; i",
        Value::Number(3.0),
    );
    result(
        "let x = 0; é: { e\u{301}: { break e\u{301}; } x = 1; } x",
        Value::Number(1.0),
    );
    result("a: { 1; break a; } a: { 2; break a; }", Value::Number(2.0));
}
