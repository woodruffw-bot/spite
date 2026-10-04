//! Bound callable identity, receiver/argument capture, metadata, and collection.

mod common;
use common::REALM_ENTRIES;

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn string(source: &str, expected: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::String(expected.into())),
        "{source}"
    );
}

#[test]
fn bound_calls_ignore_later_receivers_and_prepend_arguments() {
    for source in [
        "({}).toString.bind(null)()",
        "({}).toString.bind(null).call(true)",
        "({}).toString.bind(null).apply(true, {})",
        "({}).toString.bind(null).bind(true)()",
        "({}).toString.call.bind(({}).toString, null)()",
        "({}).toString.call.bind(({}).toString, null)(true)",
    ] {
        string(source, "[object Null]");
    }
    string("({}).toString.bind()()", "[object Undefined]");
    for source in [
        "let o = {}; let f = ({}).valueOf.bind(o); f() === o",
        "let o = {}; let f = ({}).valueOf.call.bind(({}).valueOf, o); f() === o",
        "let o = {}; let f = ({}).valueOf.call.bind(({}).valueOf); let g = f.bind(null, o); g() === o",
        "let f = ({}).toString; f.bind(null) !== f.bind(null)",
        "typeof ({}).toString.bind(null) === 'function'",
        "!('prototype' in ({}).toString.bind(null))",
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Ok(Value::Boolean(true)),
            "{source}"
        );
    }
}

#[test]
fn bound_name_length_and_source_follow_function_rules() {
    string("({}).valueOf.bind(null).name", "bound valueOf");
    string(
        "({}).valueOf.bind(null).bind(null).name",
        "bound bound valueOf",
    );
    string(
        "({}).valueOf.bind(null).toString()",
        "function () { [native code] }",
    );
    for (source, expected) in [
        ("({}).toString.call.bind(null).length", 1.0),
        ("({}).toString.call.bind(null, null).length", 0.0),
        ("({}).toString.apply.bind(null, null).length", 1.0),
        (
            "({}).toString.apply.bind(null, null, null, null).length",
            0.0,
        ),
    ] {
        assert_eq!(Realm::default().eval(source), Ok(Value::Number(expected)));
    }
    assert!(matches!(
        Realm::default().eval("'use strict'; let f = ({}).valueOf.bind(null); f.name = 'changed'"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    string("let f = ({}).valueOf.bind(null); delete f.name; f.name", "");
}

#[test]
fn callability_failures_follow_argument_evaluation_and_captures_are_values() {
    let mut realm = Realm::default();
    realm
        .eval("let flag = 0; let o = {bind: ({}).toString.bind}")
        .unwrap();
    assert!(matches!(
        realm.eval("o.bind(flag = 1)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(1.0)));
    string(
        "let receiver = null; let f = ({}).toString.bind(receiver); receiver = true; f()",
        "[object Null]",
    );
    string(
        "let f = ({}).toString; f.bind(null, f = 1)()",
        "[object Null]",
    );
}

#[test]
fn bound_targets_receivers_arguments_and_cycles_survive_collection() {
    let mut realm = Realm::default();
    realm.eval("let f; { let receiver = {}; let extra = {}; let target = ({}).valueOf.call.bind(({}).valueOf, receiver); f = target.bind(null, extra); receiver.f = f; extra.f = f; }").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES + 4); // global environment, global object, nine intrinsics, two bound functions, two captures
    assert_eq!(realm.eval("f().f === f"), Ok(Value::Boolean(true)));
    let value = realm.eval("f").unwrap();
    let root = realm.root_value(value, 100).unwrap();
    realm.eval("f = null").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES + 4);
    drop(root);
    assert_eq!(realm.collect(usize::MAX).unwrap().reclaimed, 4);
}

#[test]
fn bound_arguments_share_call_limits_and_failure_skips_finalizers() {
    let mut realm = Realm::new(Limits {
        max_arguments: Some(3),
        ..Limits::default()
    });
    realm
        .eval("let flag = 0; let f = ({}).toString.bind(null, 1, 2)")
        .unwrap();
    assert_eq!(
        realm.eval("f(3)"),
        Ok(Value::String("[object Null]".into()))
    );
    assert!(matches!(
        realm.eval("try { f(3, 4); } catch { flag = 1; } finally { flag = 2; }"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(realm.eval("f()"), Ok(Value::String("[object Null]".into())));
}
