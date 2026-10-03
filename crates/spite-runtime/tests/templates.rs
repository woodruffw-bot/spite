//! Untagged template strings use ordered ToString and preserve UTF-16 units.

use spite_core::{DiagnosticKind, JsString};
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn result(source: &str, expected: &str) {
    for prefix in ["", "'use strict'; "] {
        assert_eq!(
            Realm::default().eval(&format!("{prefix}{source}")),
            Ok(Value::String(JsString::from(expected))),
            "{prefix}{source}"
        );
    }
}

#[test]
fn template_values_use_cooked_text_and_primitive_string_conversion() {
    for (source, expected) in [
        ("``", ""),
        ("`abc`", "abc"),
        ("`a\r\nb\rc`", "a\nb\nc"),
        (r"`a\n\x41\u{1f4a9}`", "a\nA💩"),
        (r"`\`\${x}`", "`${x}"),
        ("`a\\\r\nb`", "ab"),
        (
            "`${undefined},${null},${true},${false}`",
            "undefined,null,true,false",
        ),
        (
            "`${-0},${NaN},${Infinity},${-Infinity}`",
            "0,NaN,Infinity,-Infinity",
        ),
        ("`${1 + 2}:${'x'}:${''}`", "3:x:"),
        ("`${1, 2}`", "2"),
        ("`${false ? 1 : 2}`", "2"),
        ("`a${`b${3}c`}d`", "ab3cd"),
        ("`a` + `b`", "ab"),
    ] {
        result(source, expected);
    }
    assert_eq!(
        Realm::default().eval(r"`\ud800${'\udfff'}`"),
        Ok(Value::String(JsString::from_code_units(vec![
            0xd800, 0xdfff
        ])))
    );
}

#[test]
fn substitutions_run_once_in_order_and_capture_each_value() {
    result("let x = 0; `${++x}:${x++}:${x}`", "1:1:2");
    result("let x = 0; `${x = 1}${x = 2}${x = 3}`", "123");
    result("let x = 'a'; `${x}${x = 'b'}`", "ab");
    result("let x = 0; `a${`${++x}${++x}`}b${++x}`", "a12b3");
    result("let x = 0; `${x += 1} ${x += 2}`", "1 3");
}

#[test]
fn abrupt_substitutions_stop_later_effects_and_run_language_finalizers() {
    let mut realm = Realm::default();
    realm.eval("let x = 0; let finalizer = 0;").unwrap();
    assert!(matches!(
        realm.eval("try { `${x = 1}${missing}${x = 2}`; } finally { finalizer = 7; }"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
    assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
    assert_eq!(realm.eval("finalizer"), Ok(Value::Number(7.0)));
    result("try { `${x}`; let x; } catch { 'caught'; }", "caught");
}

#[test]
fn template_limits_abort_without_exposing_partial_results() {
    for source in ["`abcd`", "`ab${'cd'}`", "`${'ab'}cd`", "`${'💩'}${'💩'}`"] {
        let mut realm = Realm::new(Limits {
            max_string_units: 3,
            ..Limits::default()
        });
        realm.eval("let x = 'ok'; let flag = 0;").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "try {{ x = {source}; }} catch {{ flag = 1; }} finally {{ flag = 2; }}"
                )),
                Err(Error::Limit { .. })
            ),
            "{source}"
        );
        assert_eq!(realm.eval("x"), Ok(Value::String(JsString::from("ok"))));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
    let mut realm = Realm::new(Limits {
        max_steps: 16,
        ..Limits::default()
    });
    let source = format!("`{}`", "${''}".repeat(32));
    assert!(matches!(realm.eval(&source), Err(Error::Limit { .. })));
}

#[test]
fn invalid_templates_and_tags_prevent_execution() {
    for (template, kind) in [
        (r"`\1`", DiagnosticKind::Syntax),
        (r"`${1}\x`", DiagnosticKind::Syntax),
        ("tag`x`", DiagnosticKind::Unsupported),
    ] {
        let mut realm = Realm::default();
        realm.eval("let x = 1;").unwrap();
        assert!(
            matches!(realm.eval(&format!("x = 99; {template}")), Err(Error::Parse(d)) if d.kind == kind)
        );
        assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
    }
    assert_eq!(
        Realm::default().eval("`use strict`; let eval = 7; eval"),
        Ok(Value::Number(7.0))
    );
}
