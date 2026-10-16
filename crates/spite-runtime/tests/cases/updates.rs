//! Updates convert once, write the same reference, and return old or new numbers.

use spite_core::{DiagnosticKind, JsString};
use spite_runtime::{Error, ExceptionKind, Realm, Value};

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
fn prefix_and_postfix_return_numeric_values_and_update_the_binding() {
    for (initial, number) in [
        ("'4'", 4.0),
        ("true", 1.0),
        ("false", 0.0),
        ("null", 0.0),
        ("''", 0.0),
        ("-0", -0.0),
        ("undefined", f64::NAN),
        ("'bad'", f64::NAN),
        ("Infinity", f64::INFINITY),
    ] {
        for (op, new) in [("++", number + 1.0), ("--", number - 1.0)] {
            result(&format!("let x = {initial}; x{op}"), Value::Number(number));
            result(&format!("let x = {initial}; {op}x"), Value::Number(new));
            result(&format!("let x = {initial}; x{op}; x"), Value::Number(new));
        }
    }
    result(
        "let x = 9007199254740992; ++x",
        Value::Number(9007199254740992.0),
    );
}

#[test]
fn updates_follow_expression_order_precedence_and_asi() {
    result("let x = 2; x++ + ++x", Value::Number(6.0));
    result("let x = 2; ++x ** 2", Value::Number(9.0));
    result("let x = 2; x++ ** 2 + x", Value::Number(7.0));
    result(
        "let x = 2; typeof x++",
        Value::String(JsString::from("number")),
    );
    result("let x = 2; let y = 3; x\n++y; x + y", Value::Number(6.0));
    result(
        "let x = 2; let y = 3; x/*\n*/--y; x + y",
        Value::Number(4.0),
    );
    result("let x = 2; ++(x); ((x))--; x", Value::Number(2.0));
    result(
        "let count = 0; for (let i = 0; i < 3; i++) count++; count",
        Value::Number(3.0),
    );
    result(
        "let x = 1; false && x++; true || ++x; x",
        Value::Number(1.0),
    );
}

#[test]
fn updates_obey_tdz_immutability_and_reference_resolution() {
    for (source, kind) in [
        ("missing++", ExceptionKind::ReferenceError),
        ("++missing", ExceptionKind::ReferenceError),
        ("let x = x++;", ExceptionKind::ReferenceError),
        ("const x = 1; x++;", ExceptionKind::TypeError),
        ("const x = 1; --x;", ExceptionKind::TypeError),
        ("'use strict'; undefined++;", ExceptionKind::TypeError),
    ] {
        assert!(
            matches!(Realm::default().eval(source), Err(Error::Exception {kind: actual, ..}) if actual == kind),
            "{source}"
        );
    }
    result("let x = 1; { let x = 4; x++; } x", Value::Number(1.0));
    result("const x = 1; try { x++; } catch { x; }", Value::Number(1.0));
    let mut realm = Realm::default();
    assert!(matches!(
        realm.eval("missing++"),
        Err(Error::Exception { .. })
    ));
    assert_eq!(
        realm.eval("typeof missing"),
        Ok(Value::String(JsString::from("undefined")))
    );
    assert_eq!(realm.eval("Infinity++"), Ok(Value::Number(f64::INFINITY)));
    assert_eq!(realm.eval("Infinity"), Ok(Value::Number(f64::INFINITY)));
}

#[test]
fn invalid_updates_are_early_errors_before_side_effects() {
    let mut realm = Realm::default();
    realm.eval("let x = 1;").unwrap();
    assert!(
        matches!(realm.eval("x = 99; 1++;"), Err(Error::Parse(d)) if d.kind == DiagnosticKind::Syntax)
    );
    assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
}
