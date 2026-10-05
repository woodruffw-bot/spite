//! Builtin function identity, metadata, receivers, coercion, and collection.

mod common;
use common::REALM_ENTRIES;

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
            let property = function
                .own_property(&JsString::from(key))
                .unwrap()
                .as_data()
                .unwrap();
            assert!(!property.writable && !property.enumerable && property.configurable);
        }
        assert_eq!(
            function
                .own_property(&JsString::from("name"))
                .unwrap()
                .as_data()
                .unwrap()
                .value,
            Value::String(JsString::from(name))
        );
        assert_eq!(
            function
                .own_property(&JsString::from("length"))
                .unwrap()
                .as_data()
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
fn function_constructor_identity_is_exposed_and_compiles_ordinary_bodies() {
    assert_eq!(
        Realm::default().eval("'constructor' in ({}).toString"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(
        Realm::default().eval("({}).toString.constructor===Function"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(
        Realm::default().eval("Function('return 1;')()"),
        Ok(Value::Number(1.0))
    );
}

#[test]
fn restricted_function_properties_throw_in_both_modes_and_remain_shadowable() {
    for prefix in ["", "'use strict';"] {
        for key in ["caller", "arguments"] {
            let mut realm = Realm::default();
            realm.eval("let f = ({}).toString; let flag = 0").unwrap();
            for expression in [
                format!("f.{key}"),
                format!("f.{key} = (flag = 1)"),
                format!("f.{key} += (flag = 9)"),
                format!("typeof f.{key}"),
            ] {
                assert!(matches!(
                    realm.eval(&format!("{prefix} {expression}")),
                    Err(Error::Exception {
                        kind: ExceptionKind::TypeError,
                        ..
                    })
                ));
            }
            assert_eq!(realm.eval("flag"), Ok(Value::Number(1.0)));
            assert_eq!(
                realm.eval(&format!(
                    "let o = {{__proto__: f, {key}: 1}}; o.{key} = 2; o.{key}"
                )),
                Ok(Value::Number(2.0))
            );
            assert_eq!(
                realm.eval(&format!("delete f.{key}; '{key}' in f")),
                Ok(Value::Boolean(true))
            );
            assert_eq!(realm.eval(&format!("try {{ f.{key}(flag = 9); }} catch {{ flag += 2; }} finally {{ flag += 4; }} flag")), Ok(Value::Number(7.0)));
            assert!(matches!(
                realm.eval(&format!("delete o.{key}; o.{key}")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        }
    }
}

#[test]
fn builtin_graphs_and_host_roots_survive_collection() {
    let mut realm = Realm::default();
    let value = realm.eval("({}).toString").unwrap();
    let root = realm.root_value(value.clone(), 100).unwrap();
    let result = realm.collect(usize::MAX).unwrap();
    assert_eq!(result.live, REALM_ENTRIES);
    assert_eq!(result.reclaimed, 1);
    assert_eq!(realm.eval("({}).toString"), Ok(value));
    drop(root);
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    string("({}).toString()", "[object Object]");
}

#[test]
fn partial_intrinsic_initialization_is_never_published() {
    // Script work is separate from fixed, bounded realm initialization.
    let mut realm = Realm::new(Limits {
        max_steps: Some(0),
        ..Limits::default()
    });
    assert!(matches!(realm.eval("0"), Err(Error::Limit { .. })));
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    for slots in 0..REALM_ENTRIES - 1 {
        let mut realm = Realm::new(Limits {
            max_heap_entries: Some(slots),
            ..Limits::default()
        });
        assert!(matches!(realm.eval("({})"), Err(Error::Limit { .. })));
        assert_eq!(
            realm.collect(usize::MAX).unwrap().live,
            usize::from(slots > 0)
        );
    }
}
