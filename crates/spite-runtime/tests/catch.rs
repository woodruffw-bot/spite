//! ECMA-262 14.15: catch without a binding handles only throw completions.

use spite_core::{DiagnosticKind, JsString};
use spite_runtime::{Error, Limits, Realm, Value};

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
fn catch_handles_explicit_throws_and_built_in_exceptions() {
    for body in [
        "throw undefined;",
        "throw null;",
        "throw false;",
        "throw -0;",
        "throw NaN;",
        "throw 'x';",
        "missing;",
        "let x = x;",
        "const x = 0; x = 1;",
    ] {
        result(
            &format!("try {{ {body} }} catch {{ 7; }}"),
            Value::Number(7.0),
        );
        result(
            &format!("13; try {{ {body} }} catch {{}}"),
            Value::Undefined,
        );
    }
}

#[test]
fn catch_ignores_normal_break_and_continue_completions() {
    for (body, expected) in [("", Value::Undefined), ("7;", Value::Number(7.0))] {
        result(
            &format!("13; try {{ {body} }} catch {{ throw 99; }}"),
            expected.clone(),
        );
        for control in ["break", "continue"] {
            result(
                &format!(
                    "a: do {{ 13; try {{ {body} {control} a; }} catch {{ throw 99; }} }} while (false)"
                ),
                expected.clone(),
            );
        }
    }
}

#[test]
fn catch_completion_values_and_control_targets_propagate() {
    result(
        "a: { 13; try { throw 1; } catch { break a; } }",
        Value::Undefined,
    );
    result(
        "a: { try { throw 1; } catch { 7; break a; } }",
        Value::Number(7.0),
    );
    result(
        "for (let i = 0; i < 3; i = i + 1) { try { throw i; } catch { i; continue; } }",
        Value::Number(2.0),
    );
    result(
        "try { throw 1; } catch { try { throw 2; } catch { 7; } }",
        Value::Number(7.0),
    );
    assert_eq!(
        Realm::default().eval("try { throw 1; } catch { throw 2; }"),
        Err(Error::Thrown(Value::Number(2.0)))
    );
    result(
        "try { try { throw 1; } catch { missing; } } catch { 7; }",
        Value::Number(7.0),
    );
}

#[test]
fn finalizer_preserves_or_replaces_the_selected_body_or_catch_result() {
    for caught in [false, true] {
        let body = if caught { "throw 1;" } else { "7;" };
        result(
            &format!("try {{ {body} }} catch {{ 7; }} finally {{ 99; }}"),
            Value::Number(7.0),
        );
        result(
            &format!("a: {{ 13; try {{ {body} }} catch {{ 7; }} finally {{ break a; }} }}"),
            Value::Undefined,
        );
        assert_eq!(
            Realm::default().eval(&format!(
                "try {{ {body} }} catch {{ 7; }} finally {{ throw 9; }}"
            )),
            Err(Error::Thrown(Value::Number(9.0)))
        );
    }
    assert_eq!(
        Realm::default().eval("try { throw 1; } catch { throw 2; } finally { 99; }"),
        Err(Error::Thrown(Value::Number(2.0)))
    );
    result(
        "a: { try { throw 1; } catch { throw 2; } finally { 7; break a; } }",
        Value::Number(7.0),
    );
    result(
        "a: { try { throw 1; } catch { missing; } finally { 7; break a; } }",
        Value::Number(7.0),
    );
    // An exception from finally cannot be handled by its own catch clause.
    assert_eq!(
        Realm::default().eval("try {} catch { 7; } finally { throw 9; }"),
        Err(Error::Thrown(Value::Number(9.0)))
    );
}

#[test]
fn execution_order_and_block_scopes_are_preserved() {
    result(
        "let trace = ''; let x = 'o';
        try { let x = 'b'; trace = trace + x; throw 1; }
        catch { trace = trace + x; { let x = 'c'; trace = trace + x; } }
        finally { trace = trace + x; }
        trace",
        Value::String(JsString::from("boco")),
    );
    result(
        "let x = 1; let seen = 0;
        try { let x = x; } catch { seen = x; let y = 3; } finally { seen = seen + x; }
        seen + x",
        Value::Number(3.0),
    );
    result(
        "let x = 1;
        try { try { throw 1; } catch { let x = 2; throw x; } }
        catch { x; }",
        Value::Number(1.0),
    );
    result(
        "let x = 1; a: { try { throw 1; } catch { let x = 2; break a; } } x",
        Value::Number(1.0),
    );
    assert_eq!(
        Realm::default().eval("try { throw 1; } catch { 'use strict'; let eval = 7; eval; }"),
        Ok(Value::Number(7.0))
    );
}

#[test]
fn host_failures_in_body_or_catch_abort_pending_handlers_and_finalizers() {
    for failure in ["Proxy;", "for (;;) ;"] {
        for in_catch in [false, true] {
            let mut realm = Realm::new(Limits {
                // This explicitly opted-in budget permits intrinsic setup, then
                // aborts the infinite loop before any catch/finally handlers.
                max_steps: (failure == "for (;;) ;").then_some(1024),
                ..Limits::default()
            });
            realm.eval("let x = 1;").unwrap();
            realm.eval("let flag = 0;").unwrap();
            let inner = if in_catch {
                format!("try {{ throw 1; }} catch {{ let x = 2; {failure} }}")
            } else {
                format!("try {{ let x = 2; {failure} }} catch {{ flag = 9; }}")
            };
            let source = format!("try {{ {inner} }} catch {{ flag = 8; }} finally {{ flag = 7; }}");
            let error = realm.eval(&source).unwrap_err();
            if failure == "Proxy;" {
                assert!(matches!(error, Error::Unsupported { .. }));
            } else {
                assert!(matches!(error, Error::Limit { .. }));
            }
            assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
            assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
        }
    }
}

#[test]
fn invalid_or_unsupported_handlers_prevent_all_execution() {
    for (handler, kind) in [
        ("catch { let x; let x; }", DiagnosticKind::Syntax),
        ("catch { continue; }", DiagnosticKind::Syntax),
        ("catch { class C {field;} }", DiagnosticKind::Unsupported),
    ] {
        let mut realm = Realm::default();
        realm.eval("let x = 1;").unwrap();
        let source = format!("x = 99; try {{ x = 2; }} {handler} finally {{ x = 3; }}");
        assert!(matches!(realm.eval(&source), Err(Error::Parse(d)) if d.kind == kind));
        assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
    }
}
