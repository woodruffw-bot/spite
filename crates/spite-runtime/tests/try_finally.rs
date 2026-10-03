//! ECMA-262 14.15.3 finalizer ordering, completion values, and abrupt overrides.

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

fn thrown(source: &str, expected: Value) {
    for prefix in ["", "'use strict'; "] {
        let error = Realm::default()
            .eval(&format!("{prefix}{source}"))
            .unwrap_err();
        assert!(
            matches!(&error, Error::Thrown(value) if value.same_value(&expected)),
            "{prefix}{source}: {error:?} did not throw {expected:?}"
        );
    }
}

#[test]
fn normal_finalizers_preserve_body_values_and_fill_empty_with_undefined() {
    for (body, expected) in [
        ("", Value::Undefined),
        (";", Value::Undefined),
        ("let x = 1;", Value::Undefined),
        ("7;", Value::Number(7.0)),
        ("7; undefined;", Value::Undefined),
        ("-0;", Value::Number(-0.0)),
        ("NaN;", Value::Number(f64::NAN)),
        (
            "'\\ud800';",
            Value::String(JsString::from_code_units(vec![0xd800])),
        ),
    ] {
        for finalizer in ["", "99;", "undefined;", "{ let x = 1; }"] {
            result(
                &format!("13; try {{ {body} }} finally {{ {finalizer} }}"),
                expected.clone(),
            );
        }
    }
}

#[test]
fn normal_finalizers_preserve_break_and_continue_values() {
    for (value, expected) in [("", Value::Undefined), ("7;", Value::Number(7.0))] {
        for finalizer in ["", "99;"] {
            result(
                &format!("a: {{ 13; try {{ {value} break a; }} finally {{ {finalizer} }} }}"),
                expected.clone(),
            );
            result(
                &format!(
                    "for (let i = 0; i < 2; i = i + 1) {{
                    13; try {{ {value} continue; }} finally {{ {finalizer} }}
                }}"
                ),
                expected.clone(),
            );
        }
    }
}

#[test]
fn abrupt_finalizers_replace_all_supported_body_completion_kinds() {
    for body in [
        "",
        "7;",
        "throw 8;",
        "missing;",
        "7; break;",
        "7; continue;",
    ] {
        for (finalizer, expected) in [
            ("break;", Value::Undefined),
            ("continue;", Value::Undefined),
            ("9; break;", Value::Number(9.0)),
            ("9; continue;", Value::Number(9.0)),
        ] {
            result(
                &format!("do {{ 13; try {{ {body} }} finally {{ {finalizer} }} }} while (false)"),
                expected,
            );
        }
        thrown(
            &format!("do {{ try {{ {body} }} finally {{ throw 9; }} }} while (false)"),
            Value::Number(9.0),
        );
        for prefix in ["", "'use strict'; "] {
            let source = format!(
                "{prefix}do {{ try {{ {body} }} finally {{ missingFinal; }} }} while (false)"
            );
            let error = Realm::default().eval(&source).unwrap_err();
            assert!(matches!(error, Error::Exception {
                kind: ExceptionKind::ReferenceError, span, ..
            } if &source[span.start..span.end] == "missingFinal"));
        }
    }
}

#[test]
fn throws_preserve_exact_values_unless_overridden() {
    for (expression, expected) in [
        ("undefined", Value::Undefined),
        ("null", Value::Null),
        ("false", Value::Boolean(false)),
        ("-0", Value::Number(-0.0)),
        ("NaN", Value::Number(f64::NAN)),
        (
            "'\\ud800'",
            Value::String(JsString::from_code_units(vec![0xd800])),
        ),
    ] {
        thrown(
            &format!("try {{ throw {expression}; }} finally {{ 99; }}"),
            expected.clone(),
        );
        thrown(
            &format!("try {{ throw 99; }} finally {{ throw {expression}; }}"),
            expected,
        );
    }
}

#[test]
fn finalizers_run_once_after_normal_and_abrupt_language_completions() {
    for body in ["", "7;", "throw 8;", "missing;", "break;", "continue;"] {
        for prefix in ["", "'use strict'; "] {
            let mut realm = Realm::default();
            realm.eval("let trace = ''").unwrap();
            let source = format!(
                "{prefix}do {{
                try {{ trace = trace + 'b'; {body} trace = trace + 'n'; }}
                finally {{ trace = trace + 'f'; }}
                trace = trace + 'a';
            }} while (false)"
            );
            match body {
                "throw 8;" => {
                    assert_eq!(realm.eval(&source), Err(Error::Thrown(Value::Number(8.0))))
                }
                "missing;" => assert!(matches!(
                    realm.eval(&source),
                    Err(Error::Exception {
                        kind: ExceptionKind::ReferenceError,
                        ..
                    })
                )),
                _ => {
                    realm.eval(&source).unwrap();
                }
            }
            let expected = if matches!(body, "" | "7;") {
                "bnfa"
            } else {
                "bf"
            };
            assert_eq!(
                realm.eval("trace"),
                Ok(Value::String(JsString::from(expected)))
            );
        }
    }
}

#[test]
fn nested_finalizers_unwind_in_order_and_ignore_normal_values() {
    for body in ["", "throw 8;", "missing;", "break a;"] {
        let source = format!(
            "let trace = ''; a: {{
            try {{
                try {{ trace = trace + 'b'; {body} }}
                finally {{ trace = trace + 'i'; }}
            }} finally {{ trace = trace + 'o'; break a; }}
        }} trace"
        );
        result(&source, Value::String(JsString::from("bio")));
    }
    thrown(
        "try { try { throw 1; } finally { throw 2; } } finally { 3; }",
        Value::Number(2.0),
    );
    thrown(
        "try { try { throw 1; } finally { throw 2; } } finally { throw 3; }",
        Value::Number(3.0),
    );
    result(
        "a: { try { throw 1; } finally { try { 7; break a; } finally { 99; } } }",
        Value::Number(7.0),
    );
    result(
        "try { 7; } finally { a: { 99; break a; } }",
        Value::Number(7.0),
    );
}

#[test]
fn control_target_overrides_determine_loop_updates_and_switch_exits() {
    result(
        "let x = 0; a: for (; x < 3; x = x + 1) {
        try { break a; } finally { continue a; }
    } x",
        Value::Number(3.0),
    );
    result(
        "let x = 0; a: for (;; x = 99) {
        try { continue a; } finally { break a; }
    } x",
        Value::Number(0.0),
    );
    result(
        "let trace = ''; a: for (let i = 0; i < 2; i = i + 1) {
        while (true) { try { break; } finally { trace = trace + i; continue a; } }
        trace = trace + 'bad';
    } trace",
        Value::String(JsString::from("01")),
    );
    result(
        "let x = 0; switch (0) {
        case 0: try { break; } finally { x = 7; }
        default: x = 99;
    } x",
        Value::Number(7.0),
    );
    result(
        "a: { try { break a; } finally { b: { 99; break b; } } }",
        Value::Undefined,
    );
    result(
        "a: { b: { try { break b; } finally { 7; break a; } } 99; }",
        Value::Number(7.0),
    );
}

#[test]
fn body_scope_is_restored_before_entering_the_finalizer() {
    result(
        "let x = 1; let y = 0;
        try { let x = 2; x; } finally { y = x; } y",
        Value::Number(1.0),
    );
    result(
        "let x = 1; let y = 0;
        try { let x = 2; x; } finally { let x = 3; y = x; } x + y",
        Value::Number(4.0),
    );
    result(
        "let x = 1; let y = 0; a: {
        try { let x = 2; throw x; } finally { y = x; break a; }
    } y",
        Value::Number(1.0),
    );
    result(
        "let x = 1; let y = 0; a: {
        try { let x = x; } finally { y = x; break a; }
    } y",
        Value::Number(1.0),
    );
    result(
        "let x = 0; for (let i = 0; i < 3; i = i + 1) {
        try { continue; } finally { x = x + i; }
    } x",
        Value::Number(3.0),
    );
}

#[test]
fn built_in_exceptions_run_finalizers_and_can_be_preserved_or_overridden() {
    for (body, kind) in [
        ("missing;", ExceptionKind::ReferenceError),
        ("let x = x;", ExceptionKind::ReferenceError),
        ("const x = 1; x = 2;", ExceptionKind::TypeError),
    ] {
        for prefix in ["", "'use strict'; "] {
            let mut realm = Realm::default();
            realm.eval("let flag = 0; let x = 7;").unwrap();
            let source = format!("{prefix}try {{ {body} }} finally {{ flag = x; }}");
            let error = realm.eval(&source).unwrap_err();
            assert!(matches!(error, Error::Exception {kind: actual, ..} if actual == kind));
            assert_eq!(realm.eval("flag"), Ok(Value::Number(7.0)));
            assert_eq!(realm.eval("x"), Ok(Value::Number(7.0)));
        }
        result(
            &format!("a: {{ try {{ {body} }} finally {{ 9; break a; }} }}"),
            Value::Number(9.0),
        );
        thrown(
            &format!("try {{ {body} }} finally {{ throw 9; }}"),
            Value::Number(9.0),
        );
    }
}

#[test]
fn finalizer_errors_restore_the_outer_environment() {
    for (finalizer, kind) in [
        ("let x = x;", ExceptionKind::ReferenceError),
        ("const x = 2; x = 3;", ExceptionKind::TypeError),
    ] {
        let mut realm = Realm::default();
        realm.eval("let x = 1;").unwrap();
        let source = format!("try {{ let x = 99; throw 8; }} finally {{ {finalizer} }}");
        assert!(
            matches!(realm.eval(&source), Err(Error::Exception {kind: actual, ..}) if actual == kind)
        );
        assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
    }
    let mut realm = Realm::default();
    realm.eval("let x = 1;").unwrap();
    assert_eq!(
        realm.eval("try { let x = 99; } finally { let x = 2; throw x; }"),
        Err(Error::Thrown(Value::Number(2.0)))
    );
    assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
}

#[test]
fn host_failures_abort_without_running_or_being_overridden_by_finalizers() {
    for (body, is_limit) in [("Object;", false), ("'ab';", true), ("for (;;) ;", true)] {
        let mut realm = Realm::new(Limits {
            max_steps: 64,
            max_string_units: 1,
            ..Limits::default()
        });
        realm.eval("let x = 1; let flag = 0;").unwrap();
        let source = format!(
            "a: {{ try {{
            try {{ let x = 2; {body} }} finally {{ flag = 1; break a; }}
        }} finally {{ flag = 2; throw 99; }} }}"
        );
        let error = realm.eval(&source).unwrap_err();
        if is_limit {
            assert!(matches!(error, Error::Limit { .. }), "{error:?}");
        } else {
            assert!(matches!(error, Error::Unsupported { .. }), "{error:?}");
        }
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
        assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
    }
}

#[test]
fn host_failure_in_a_finalizer_stops_outer_finalizers() {
    for (finalizer, is_limit) in [("Object;", false), ("for (;;) ;", true)] {
        let mut realm = Realm::new(Limits {
            max_steps: 64,
            ..Limits::default()
        });
        realm.eval("let x = 1; let flag = 0;").unwrap();
        let source = format!(
            "a: {{ try {{
            try {{ throw 8; }} finally {{ let x = 2; flag = 1; {finalizer} }}
        }} finally {{ flag = 2; break a; }} }}"
        );
        let error = realm.eval(&source).unwrap_err();
        if is_limit {
            assert!(matches!(error, Error::Limit { .. }), "{error:?}");
        } else {
            assert!(matches!(error, Error::Unsupported { .. }), "{error:?}");
        }
        assert_eq!(realm.eval("flag"), Ok(Value::Number(1.0)));
        assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
    }
}

#[test]
fn all_early_errors_precede_body_and_finalizer_effects() {
    for source in [
        "x = 99; try { x = 2; let y; let y; } finally { x = 3; }",
        "x = 99; try { x = 2; } finally { x = 3; let y; let y; }",
        "x = 99; try { x = 2; } finally { x = 3; break; }",
        "x = 99; try { x = 2; continue; } finally { x = 3; }",
    ] {
        let mut realm = Realm::default();
        realm.eval("let x = 1;").unwrap();
        assert!(
            matches!(realm.eval(source), Err(Error::Parse(d)) if d.kind == DiagnosticKind::Syntax)
        );
        assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
    }
}
