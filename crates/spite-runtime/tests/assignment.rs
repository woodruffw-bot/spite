//! Compound assignment ordering, coercion, and conditional writes.

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
fn compound_assignments_use_the_underlying_numeric_operator() {
    for (initial, op, rhs, expected) in [
        ("7", "+=", "3", 10.0),
        ("7", "-=", "3", 4.0),
        ("7", "*=", "3", 21.0),
        ("7", "/=", "2", 3.5),
        ("7", "%=", "3", 1.0),
        ("2", "**=", "3", 8.0),
        ("7", "&=", "3", 3.0),
        ("7", "|=", "8", 15.0),
        ("7", "^=", "3", 4.0),
        ("1", "<<=", "31", -2147483648.0),
        ("-8", ">>=", "2", -2.0),
        ("-1", ">>>=", "1", 2147483647.0),
        ("'7'", "*=", "true", 7.0),
        ("false", "+=", "null", 0.0),
        ("-0", "*=", "2", -0.0),
        ("0", "/=", "0", f64::NAN),
        ("1", "/=", "0", f64::INFINITY),
    ] {
        result(
            &format!("let x = {initial}; x {op} {rhs}"),
            Value::Number(expected),
        );
        result(
            &format!("let x = {initial}; x {op} {rhs}; x"),
            Value::Number(expected),
        );
    }
    result("let x = 'a'; x += 7", Value::String(JsString::from("a7")));
    result("let x = 7; x += 'a'", Value::String(JsString::from("7a")));
    result(
        "let x = '\\ud800'; x += '\\udc00'",
        Value::String(JsString::from_code_units(vec![0xd800, 0xdc00])),
    );
}

#[test]
fn compound_assignments_read_before_the_rhs_and_associate_right() {
    result("let x = 1; x += x = 2", Value::Number(3.0));
    result("let x = 1; x += x += 2", Value::Number(4.0));
    result("let x = 2; x *= x++", Value::Number(4.0));
    result(
        "let x = 2; let y = 3; x += y *= 4; x + y",
        Value::Number(26.0),
    );
    result("let x = 1; x += true ? 7 : 9", Value::Number(8.0));
    result("let x = 0; x += 1, 7", Value::Number(7.0));
    result("let x = 1; (x) **= -1; x", Value::Number(1.0));
    result("let x = 1; { let x = 9; x += 2; } x", Value::Number(1.0));
}

#[test]
fn logical_assignments_short_circuit_both_rhs_and_put_value() {
    for (initial, op, expected) in [
        ("false", "&&=", Value::Boolean(false)),
        ("0", "&&=", Value::Number(0.0)),
        ("-0", "&&=", Value::Number(-0.0)),
        ("NaN", "&&=", Value::Number(f64::NAN)),
        ("''", "&&=", Value::String(JsString::from(""))),
        ("true", "||=", Value::Boolean(true)),
        ("'x'", "||=", Value::String(JsString::from("x"))),
        ("0", "??=", Value::Number(0.0)),
        ("false", "??=", Value::Boolean(false)),
        ("NaN", "??=", Value::Number(f64::NAN)),
    ] {
        result(
            &format!("const x = {initial}; x {op} missing"),
            expected.clone(),
        );
        result(
            &format!("const x = {initial}; let effect = 0; x {op} (effect = 99); effect"),
            Value::Number(0.0),
        );
    }
    for (initial, op) in [
        ("true", "&&="),
        ("0", "||="),
        ("null", "??="),
        ("undefined", "??="),
    ] {
        result(
            &format!("let x = {initial}; x {op} 'yes'"),
            Value::String(JsString::from("yes")),
        );
        result(
            &format!("let x = {initial}; x {op} 'yes'; x"),
            Value::String(JsString::from("yes")),
        );
        result(
            &format!(
                "const x = {initial}; let effect = 0; try {{ x {op} (effect = 7); }} catch {{ effect; }}"
            ),
            Value::Number(7.0),
        );
    }
}

#[test]
fn get_value_failures_precede_rhs_but_put_value_failures_follow_it() {
    for op in ["+=", "*=", "&&=", "||=", "??="] {
        let mut realm = Realm::default();
        realm.eval("let effect = 0;").unwrap();
        assert!(matches!(
            realm.eval(&format!("missing {op} (effect = 1)")),
            Err(Error::Exception {
                kind: ExceptionKind::ReferenceError,
                ..
            })
        ));
        assert_eq!(realm.eval("effect"), Ok(Value::Number(0.0)));
        assert!(matches!(
            realm.eval(&format!("{{ x {op} (effect = 1); let x; }}")),
            Err(Error::Exception {
                kind: ExceptionKind::ReferenceError,
                ..
            })
        ));
        assert_eq!(realm.eval("effect"), Ok(Value::Number(0.0)));
    }
    let mut realm = Realm::default();
    realm.eval("const x = 1; let effect = 0;").unwrap();
    assert!(matches!(
        realm.eval("x += (effect = 7)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert_eq!(realm.eval("effect"), Ok(Value::Number(7.0)));
    assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
    result(
        "let x = 1; try { x += missing; } catch { x; }",
        Value::Number(1.0),
    );
}

#[test]
fn unsupported_operations_and_limits_never_write_partial_results() {
    let mut realm = Realm::new(Limits {
        max_string_units: 3,
        ..Limits::default()
    });
    realm.eval("let x = 'ab'; let effect = 0;").unwrap();
    assert!(matches!(
        realm.eval("try { x += 'cd'; } catch { effect = 1; }"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("x"), Ok(Value::String(JsString::from("ab"))));
    assert_eq!(realm.eval("effect"), Ok(Value::Number(0.0)));
    assert!(matches!(
        realm.eval("Math += (effect = 1)"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("effect"), Ok(Value::Number(0.0)));
}
