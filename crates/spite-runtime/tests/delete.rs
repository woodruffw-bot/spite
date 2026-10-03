//! Delete distinguishes references from values without reading binding values.

use spite_core::{DiagnosticKind, JsString};
use spite_runtime::{Error, ExceptionKind, Realm, Value};

#[test]
fn lexical_references_and_restricted_globals_cannot_be_deleted() {
    for source in [
        "let x = 1; delete x",
        "const x = 1; delete (x)",
        "{ delete x; let x; }",
        "delete undefined",
        "delete NaN",
        "delete Infinity",
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Ok(Value::Boolean(false)),
            "{source}"
        );
    }
    assert_eq!(
        Realm::default().eval("let x = 1; delete x; x"),
        Ok(Value::Number(1.0))
    );
    assert_eq!(
        Realm::default().eval("delete (missing)"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn sloppy_global_deletion_removes_the_binding_and_can_be_repeated() {
    let mut realm = Realm::default();
    realm.eval("x = 1;").unwrap();
    assert_eq!(realm.eval("delete x"), Ok(Value::Boolean(true)));
    assert_eq!(
        realm.eval("typeof x"),
        Ok(Value::String(JsString::from("undefined")))
    );
    assert!(matches!(
        realm.eval("x"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
    assert_eq!(realm.eval("delete x"), Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let x = 2; x"), Ok(Value::Number(2.0)));
}

#[test]
fn non_reference_operands_evaluate_before_delete_returns_true() {
    for prefix in ["", "'use strict'; "] {
        for source in [
            "delete 1",
            "delete null",
            "let x = 1; delete (0, x)",
            "let x = 1; delete ++x",
            "delete typeof missing",
        ] {
            assert_eq!(
                Realm::default().eval(&format!("{prefix}{source}")),
                Ok(Value::Boolean(true)),
                "{source}"
            );
        }
        assert_eq!(
            Realm::default().eval(&format!("{prefix}let x = 1; delete (x += 2); x")),
            Ok(Value::Number(3.0))
        );
        assert!(matches!(
            Realm::default().eval(&format!("{prefix}delete (0, missing)")),
            Err(Error::Exception {
                kind: ExceptionKind::ReferenceError,
                ..
            })
        ));
    }
}

#[test]
fn strict_delete_is_an_early_error_before_any_side_effects() {
    let mut realm = Realm::default();
    realm.eval("let x = 1;").unwrap();
    assert!(
        matches!(realm.eval("'use strict'; x = 99; delete x"), Err(Error::Parse(d)) if d.kind == DiagnosticKind::Syntax)
    );
    assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
}

#[test]
fn missing_standard_objects_stay_unsupported_when_deleted() {
    assert!(matches!(
        Realm::default().eval("try { delete Object; } catch { 1; }"),
        Err(Error::Unsupported { .. })
    ));
}
