//! Object-literal evaluation, identity, key order, and resource failures.

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Limits, ObjectHandle, Realm, Value};

fn object(realm: &mut Realm, source: &str) -> ObjectHandle {
    let Value::Object(handle) = realm.eval(source).unwrap() else {
        panic!("object value")
    };
    handle
}

fn own(realm: &Realm, handle: &ObjectHandle, key: &str) -> Value {
    realm
        .inspect_object(handle)
        .unwrap()
        .own_property(&JsString::from(key))
        .unwrap()
        .as_data()
        .unwrap()
        .value
        .clone()
}

#[test]
fn literals_create_unique_objects_and_preserve_shared_values() {
    for (source, expected) in [
        ("({}) === ({})", Value::Boolean(false)),
        ("let x = {}; let y = x; x === y", Value::Boolean(true)),
        ("let x = {}; x == x", Value::Boolean(true)),
        ("({}) == null", Value::Boolean(false)),
        ("({}) == undefined", Value::Boolean(false)),
        ("typeof {}", Value::String(JsString::from("object"))),
        ("!{}", Value::Boolean(false)),
        ("({}) && 4", Value::Number(4.0)),
        ("({}) ? 1 : 2", Value::Number(1.0)),
        (
            "let x = {}; try { throw x; } catch (e) { e === x; }",
            Value::Boolean(true),
        ),
    ] {
        assert_eq!(Realm::default().eval(source), Ok(expected), "{source}");
    }
    let mut realm = Realm::default();
    let handle = object(&mut realm, "let shared = {}; ({a: shared, b: shared})");
    assert!(own(&realm, &handle, "a").strictly_equal(&own(&realm, &handle, "b")));
    let other = object(&mut realm, "({})");
    assert_eq!(
        realm.inspect_object(&handle).unwrap().prototype(),
        realm.inspect_object(&other).unwrap().prototype()
    );
}

#[test]
fn properties_apply_in_source_order_with_ordinary_attributes() {
    let mut realm = Realm::default();
    let handle = object(
        &mut realm,
        "let a = 7; ({b: 1, a, 10: 2, 2: 3, b: 4, ['c']: 5,})",
    );
    assert_eq!(
        realm.inspect_object(&handle).unwrap().own_keys(),
        ["2", "10", "b", "a", "c"].map(JsString::from)
    );
    for (key, expected) in [("a", 7.0), ("b", 4.0), ("c", 5.0), ("2", 3.0), ("10", 2.0)] {
        let property = realm
            .inspect_object(&handle)
            .unwrap()
            .own_property(&JsString::from(key))
            .unwrap()
            .as_data()
            .unwrap();
        assert_eq!(property.value, Value::Number(expected));
        assert!(property.writable && property.enumerable && property.configurable);
    }
}

#[test]
fn key_conversion_precedes_value_evaluation_and_next_property() {
    let mut realm = Realm::default();
    let handle = object(
        &mut realm,
        "let x = 0; ({[x = x * 10 + 1]: x = x * 10 + 2, [x = x * 10 + 3]: x = x * 10 + 4})",
    );
    assert_eq!(own(&realm, &handle, "1"), Value::Number(12.0));
    assert_eq!(own(&realm, &handle, "123"), Value::Number(1234.0));
    assert_eq!(realm.eval("x"), Ok(Value::Number(1234.0)));
    assert!(matches!(
        realm.eval("({[{__proto__: null}]: x = 9})"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert_eq!(realm.eval("x"), Ok(Value::Number(1234.0)));
    assert!(matches!(
        realm.eval("({[missing]: x = 10})"),
        Err(Error::Exception { .. })
    ));
    assert_eq!(realm.eval("x"), Ok(Value::Number(1234.0)));
}

#[test]
fn literal_and_primitive_computed_names_use_ecmascript_string_conversion() {
    let mut realm = Realm::default();
    let handle = object(
        &mut realm,
        r"({0x10n: 1, 1e3: 2, [-0]: 3, [null]: 4, [undefined]: 5, [true]: 6, ['\ud800']: 7, [100000000000000000000000n]: 8})",
    );
    for (key, expected) in [
        ("16", 1.0),
        ("1000", 2.0),
        ("0", 3.0),
        ("null", 4.0),
        ("undefined", 5.0),
        ("true", 6.0),
        ("100000000000000000000000", 8.0),
    ] {
        assert_eq!(own(&realm, &handle, key), Value::Number(expected));
    }
    assert_eq!(
        realm
            .inspect_object(&handle)
            .unwrap()
            .own_property(&JsString::from_code_units(vec![0xd800]))
            .unwrap()
            .as_data()
            .unwrap()
            .value,
        Value::Number(7.0)
    );
}

#[test]
fn prototype_initializers_are_distinct_from_computed_and_shorthand_properties() {
    let mut realm = Realm::default();
    let base = object(&mut realm, "let base = {x: 1}; base");
    let child = object(&mut realm, "({__proto__: base, y: 2})");
    assert_eq!(
        realm.inspect_object(&child).unwrap().prototype(),
        Some(&base)
    );
    assert!(
        realm
            .inspect_object(&child)
            .unwrap()
            .own_property(&JsString::from("__proto__"))
            .is_none()
    );
    let null = object(&mut realm, "({__proto__: null})");
    assert!(realm.inspect_object(&null).unwrap().prototype().is_none());
    let ignored = object(&mut realm, "({__proto__: 7})");
    assert_eq!(
        realm.inspect_object(&ignored).unwrap().prototype(),
        realm.inspect_object(&base).unwrap().prototype()
    );
    assert_eq!(realm.inspect_object(&ignored).unwrap().property_count(), 0);
    let ordinary = object(
        &mut realm,
        "let __proto__ = 3; ({__proto__: null, ['__proto__']: 1, __proto__})",
    );
    assert!(
        realm
            .inspect_object(&ordinary)
            .unwrap()
            .prototype()
            .is_none()
    );
    assert_eq!(own(&realm, &ordinary, "__proto__"), Value::Number(3.0));
}

#[test]
fn early_errors_precede_all_effects_and_object_conversion_stays_explicit() {
    let mut realm = Realm::default();
    realm.eval("let effect = 0").unwrap();
    assert!(matches!(
        realm.eval("effect = 1; ({__proto__: null, __proto__: null})"),
        Err(Error::Parse(_))
    ));
    assert_eq!(realm.eval("effect"), Ok(Value::Number(0.0)));
    for source in [
        "+({}).toString",
        "({}).toString + 1",
        "1n - ({}).toString",
        "({}).toString == 1",
        "`${({}).toString}`",
        "({}).toString < 1",
    ] {
        assert!(
            matches!(realm.eval(source), Err(Error::Unsupported { .. })),
            "{source}"
        );
    }
}

#[test]
fn allocation_property_and_key_limits_are_host_failures() {
    for limits in [
        Limits {
            max_objects: 0,
            ..Limits::default()
        },
        Limits {
            max_objects: 1,
            ..Limits::default()
        },
        Limits {
            max_properties: 0,
            ..Limits::default()
        },
        Limits {
            max_string_units: 0,
            ..Limits::default()
        },
    ] {
        let mut realm = Realm::new(limits);
        realm.eval("let flag = 0").unwrap();
        assert!(matches!(
            realm.eval("try { ({x: 1}); } catch { flag = 1; } finally { flag = 2; }"),
            Err(Error::Limit { .. })
        ));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
    let mut realm = Realm::new(Limits {
        max_properties: 2,
        ..Limits::default()
    });
    let handle = object(&mut realm, "({a: 1, a: 2})");
    assert_eq!(own(&realm, &handle, "a"), Value::Number(2.0));
    assert!(matches!(
        realm.eval("({a: 1, b: 2, c: 3})"),
        Err(Error::Limit { .. })
    ));
}
