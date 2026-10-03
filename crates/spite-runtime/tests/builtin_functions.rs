//! Builtin function identity, metadata, receivers, coercion, and collection.

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn string(source: &str, expected: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::String(JsString::from(expected))),
        "{source}"
    );
}

#[test]
fn builtin_methods_are_callable_with_standard_names_lengths_and_attributes() {
    let mut realm = Realm::default();
    for (source, name) in [("({}).toString", "toString"), ("({}).valueOf", "valueOf")] {
        let Value::Object(handle) = realm.eval(source).unwrap() else {
            panic!("function")
        };
        let function = realm.inspect_object(&handle).unwrap();
        assert!(function.is_callable());
        for key in ["name", "length"] {
            let property = function.own_property(&JsString::from(key)).unwrap();
            assert!(!property.writable && !property.enumerable && property.configurable);
        }
        assert_eq!(
            function
                .own_property(&JsString::from("name"))
                .unwrap()
                .value,
            Value::String(JsString::from(name))
        );
        assert_eq!(
            function
                .own_property(&JsString::from("length"))
                .unwrap()
                .value,
            Value::Number(0.0)
        );
        assert!(
            function
                .own_property(&JsString::from("prototype"))
                .is_none()
        );
    }
    string("typeof ({}).toString", "function");
    string("typeof ({}).valueOf", "function");
    assert_eq!(
        realm.eval("({}).toString === ({}).toString"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(
        realm.eval("'prototype' in ({}).toString"),
        Ok(Value::Boolean(false))
    );
}

#[test]
fn receivers_survive_parentheses_and_argument_side_effects() {
    for source in [
        "({}).toString()",
        "let o = {}; (o.toString)()",
        "let o = {}; o.toString(o.toString = 1)",
    ] {
        string(source, "[object Object]");
    }
    for source in [
        "let o = {}; o.valueOf() === o",
        "let o = {}; (o.valueOf)() === o",
        "let o = {}; let saved = o; o.valueOf(o = null) === saved",
        "let o = {__proto__: null, method: ({}).valueOf}; o.method() === o",
        "let f = ({}).toString; f.valueOf() === f",
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Ok(Value::Boolean(true)),
            "{source}"
        );
    }
    string("let f = ({}).toString; f()", "[object Undefined]");
    string("let o = {}; (0, o.toString)()", "[object Undefined]");
    string(
        "let f = ({}).toString; f.tag = ({}).toString; f.tag()",
        "[object Function]",
    );
    for source in ["let f = ({}).valueOf; f()", "let o = {}; (0, o.valueOf)()"] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{source}"
        );
    }
}

#[test]
fn ordinary_object_conversion_calls_builtin_methods() {
    string("'' + {}", "[object Object]");
    string("({}) + 1", "[object Object]1");
    string("1n + {}", "1[object Object]");
    string("`${{}}`", "[object Object]");
    assert_eq!(
        Realm::default().eval("({}) == '[object Object]'"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(
        Realm::default().eval("({[{}]: 7})[{}]"),
        Ok(Value::Number(7.0))
    );
    assert_eq!(
        Realm::default().eval("({}) in {'[object Object]': 1}"),
        Ok(Value::Boolean(true))
    );
    let Value::Number(number) = Realm::default().eval("+{}").unwrap() else {
        panic!("Number")
    };
    assert!(number.is_nan());
    assert!(matches!(
        Realm::default().eval("1n - {}"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    string("({valueOf: 1}) + 2", "[object Object]2");
    assert!(matches!(
        Realm::default().eval("({toString: 1}) + 2"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn function_metadata_is_readonly_but_configurable() {
    string(
        "let f = ({}).toString; f.name = 'changed'; f.name",
        "toString",
    );
    assert!(matches!(
        Realm::default().eval("'use strict'; let f = ({}).toString; f.name = 'changed'"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    string("let f = ({}).toString; delete f.name; f.name", ""); // inherited Function.prototype.name
    assert_eq!(
        Realm::default().eval("let f = ({}).toString; delete f.length"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn incomplete_function_intrinsics_are_guarded_including_restricted_writes() {
    for key in [
        "constructor",
        "call",
        "apply",
        "bind",
        "toString",
        "caller",
        "arguments",
    ] {
        assert_eq!(
            Realm::default().eval(&format!("'{key}' in ({{}}).toString")),
            Ok(Value::Boolean(true))
        );
        assert!(
            matches!(
                Realm::default().eval(&format!("({{}}).toString.{key}")),
                Err(Error::Unsupported { .. })
            ),
            "{key}"
        );
    }
    for prefix in ["", "'use strict';"] {
        let mut realm = Realm::default();
        realm.eval("let f = ({}).toString; let flag = 0").unwrap();
        assert!(matches!(
            realm.eval(&format!("{prefix} f.caller = (flag = 1)")),
            Err(Error::Unsupported { .. })
        ));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(1.0)));
        assert_eq!(
            realm.eval("let o = {__proto__: f, caller: 1}; o.caller = 2; o.caller"),
            Ok(Value::Number(2.0))
        );
        assert_eq!(realm.eval("delete f.caller"), Ok(Value::Boolean(true)));
    }
}

#[test]
fn builtin_graphs_and_host_roots_survive_collection() {
    let mut realm = Realm::default();
    let value = realm.eval("({}).toString").unwrap();
    let root = realm.root_value(value.clone(), 100).unwrap();
    let result = realm.collect(1000).unwrap();
    assert_eq!(result.live, 4);
    assert_eq!(result.reclaimed, 1);
    assert_eq!(realm.eval("({}).toString"), Ok(value));
    drop(root);
    assert_eq!(realm.collect(1000).unwrap().live, 4);
    string("({}).toString()", "[object Object]");
}

#[test]
fn partial_intrinsic_initialization_is_never_published() {
    for work in 0..160 {
        let mut realm = Realm::new(Limits {
            max_steps: work,
            ..Limits::default()
        });
        assert!(matches!(
            realm.eval("({})"),
            Ok(Value::Object(_)) | Err(Error::Limit { .. })
        ));
        let live = realm.collect(1000).unwrap().live;
        assert!(live == 0 || live == 4, "work={work}, live={live}");
    }
    for slots in 0..4 {
        let mut realm = Realm::new(Limits {
            max_objects: slots,
            ..Limits::default()
        });
        assert!(matches!(realm.eval("({})"), Err(Error::Limit { .. })));
        assert_eq!(realm.collect(1000).unwrap().live, 0);
    }
}
