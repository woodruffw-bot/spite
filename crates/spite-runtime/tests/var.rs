//! Script-level var instantiation, initialization, and persistent global state.

use spite_core::{DiagnosticKind, JsString};
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn result(source: &str, expected: Value) {
    for prefix in ["", "'use strict'; void 0; "] {
        let actual = Realm::default().eval(&format!("{prefix}{source}")).unwrap();
        assert!(
            actual.same_value(&expected),
            "{prefix}{source}: {actual:?} != {expected:?}"
        );
    }
}

#[test]
fn vars_are_hoisted_and_initializers_run_only_when_reached() {
    for declaration in [
        "var x = 99;",
        "{ var x = 99; }",
        "if (false) var x = 99;",
        "while (false) var x = 99;",
        "for (var x = 99; false;) ;",
        "switch (0) { case 1: var x = 99; }",
        "try {} catch { var x = 99; }",
    ] {
        result(
            &format!("let before = x; {declaration} before"),
            Value::Undefined,
        );
    }
    result("x = 7; if (false) var x = 99; x", Value::Number(7.0));
    result(
        "let count = 0; if (false) var x = ++count; count",
        Value::Number(0.0),
    );
    result("var x = x; x", Value::Undefined);
    result("var x = y, y = 7; x", Value::Undefined);
    result("var x = 1, y = x + 1; y", Value::Number(2.0));
}

#[test]
fn repeated_declarations_do_not_reset_values_and_complete_empty() {
    result("var x = 7; var x; x", Value::Number(7.0));
    result("var x = 7, x, x = x + 1; x", Value::Number(8.0));
    result("7; var x = 99", Value::Number(7.0));
    result("var x", Value::Undefined);
    result("var x = 1; { var x = 7; } x", Value::Number(7.0));
    result("var x = 1; { let x = 99; } x", Value::Number(1.0));
    result(
        "for (var i = 0, sum = 0; i < 3; i++) sum += i; sum + i",
        Value::Number(6.0),
    );
    result(
        "for (var i = 0; i < 3; i++) { if (i < 2) continue; break; } i",
        Value::Number(2.0),
    );
}

#[test]
fn abrupt_execution_preserves_hoisted_vars_and_completed_initializers() {
    let mut realm = Realm::default();
    assert_eq!(
        realm.eval("var a = 7; throw 9; var b = 99;"),
        Err(Error::Thrown(Value::Number(9.0)))
    );
    assert_eq!(realm.eval("a"), Ok(Value::Number(7.0)));
    assert_eq!(realm.eval("b"), Ok(Value::Undefined));
    assert!(matches!(
        realm.eval("var c = missing, d = 99;"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
    assert_eq!(realm.eval("c"), Ok(Value::Undefined));
    assert_eq!(realm.eval("d"), Ok(Value::Undefined));
    result(
        "try { var x = 7; throw 9; } catch { var y = x + 1; } finally { var z = y + 1; } z",
        Value::Number(9.0),
    );
}

#[test]
fn global_var_and_lexical_conflicts_are_checked_before_execution() {
    for (first, next) in [
        ("let x = 1;", "var fresh; var x;"),
        ("var x = 1;", "let fresh; let x;"),
    ] {
        let mut realm = Realm::default();
        realm.eval(first).unwrap();
        realm.eval("let effect = 0;").unwrap();
        assert!(matches!(
            realm.eval(&format!("effect = 99; {next}")),
            Err(Error::Exception {
                kind: ExceptionKind::SyntaxError,
                ..
            })
        ));
        assert_eq!(realm.eval("effect"), Ok(Value::Number(0.0)));
        assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
        assert_eq!(realm.eval("let fresh = 7; fresh"), Ok(Value::Number(7.0)));
    }
    let mut realm = Realm::default();
    realm.eval("let x = 1;").unwrap();
    assert!(matches!(
        realm.eval("try { if (false) var x; } catch { 7; }"),
        Err(Error::Exception {
            kind: ExceptionKind::SyntaxError,
            ..
        })
    ));
    assert!(
        matches!(realm.eval("var y; let y;"), Err(Error::Parse(d)) if d.kind == DiagnosticKind::Syntax)
    );
}

#[test]
fn new_global_vars_are_not_deletable() {
    let mut realm = Realm::default();
    realm.eval("var x = 7;").unwrap();
    assert_eq!(realm.eval("delete x"), Ok(Value::Boolean(false)));
    assert_eq!(realm.eval("x = 8; delete x"), Ok(Value::Boolean(false)));
    assert_eq!(realm.eval("x"), Ok(Value::Number(8.0)));
    assert!(matches!(
        realm.eval("let x;"),
        Err(Error::Exception {
            kind: ExceptionKind::SyntaxError,
            ..
        })
    ));
    assert_eq!(
        Realm::default().eval("x = 1; var x; delete x"),
        Ok(Value::Boolean(false))
    );
}

#[test]
fn vars_preserve_existing_configurable_properties_for_later_lexical_declarations() {
    // Edition 17 GlobalDeclarationInstantiation checks configurability, not a
    // separate list of names previously declared with var.
    let mut realm = Realm::default();
    realm.eval("x = 7;").unwrap();
    assert_eq!(realm.eval("var x; x"), Ok(Value::Number(7.0)));
    assert_eq!(realm.eval("delete x"), Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let x = 8; x"), Ok(Value::Number(8.0)));
    let mut realm = Realm::default();
    realm.eval("x = 7;").unwrap();
    realm.eval("var x;").unwrap();
    assert_eq!(realm.eval("let x = 8; x"), Ok(Value::Number(8.0)));
    assert_eq!(realm.eval("delete x"), Ok(Value::Boolean(false)));
}

#[test]
fn restricted_global_values_follow_existing_writability_rules() {
    result("var undefined; undefined", Value::Undefined);
    result("var Infinity; Infinity", Value::Number(f64::INFINITY));
    assert_eq!(
        Realm::default().eval("var Infinity = 7; Infinity"),
        Ok(Value::Number(f64::INFINITY))
    );
    assert!(matches!(
        Realm::default().eval("'use strict'; var Infinity = 7;"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert!(matches!(
        Realm::default().eval("var Object;"),
        Err(Error::Unsupported { .. })
    ));
}

#[test]
fn declaration_work_consumes_budget_even_in_unreachable_code() {
    let mut realm = Realm::new(Limits {
        max_steps: 64,
        ..Limits::default()
    });
    let names = (0..128)
        .map(|i| format!("v{i}"))
        .collect::<Vec<_>>()
        .join(",");
    assert!(matches!(
        realm.eval(&format!("if (false) {{ var {names}; }}")),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("typeof v0"),
        Ok(Value::String(JsString::from("undefined")))
    );
    assert_eq!(realm.eval("let v0 = 7; v0"), Ok(Value::Number(7.0)));
}
