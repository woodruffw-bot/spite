//! Global numeric predicates coerce through ToNumber (19.2.2–3).

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Realm, Value};

#[test]
fn predicates_coerce_primitives_and_wrappers_without_accepting_numeric_prefixes() {
    for (argument, finite, nan) in [
        ("", false, true),
        ("undefined", false, true),
        ("null", true, false),
        ("false", true, false),
        ("true", true, false),
        ("0", true, false),
        ("-0", true, false),
        ("Number.MIN_VALUE", true, false),
        ("Number.MAX_VALUE", true, false),
        ("Infinity", false, false),
        ("-Infinity", false, false),
        ("NaN", false, true),
        ("''", true, false),
        ("'\\uFEFF\\u2028  '", true, false),
        ("'  -12.5  '", true, false),
        ("'0x10'", true, false),
        ("'Infinity'", false, false),
        ("'1e9999'", false, false),
        ("'1e-9999'", true, false),
        ("'12x'", false, true),
        ("'1e'", false, true),
        ("'NaN'", false, true),
        ("'1\\uD800'", false, true),
        ("new Boolean(false)", true, false),
        ("new Number(3)", true, false),
        ("new Number(NaN)", false, true),
        ("new Number(Infinity)", false, false),
    ] {
        let mut realm = Realm::default();
        for (name, expected) in [("isFinite", finite), ("isNaN", nan)] {
            let source = format!("{name}({argument})");
            assert_eq!(
                realm.eval(&source),
                Ok(Value::Boolean(expected)),
                "{source}"
            );
        }
    }
}

#[test]
fn conversion_uses_number_hint_and_preserves_abrupt_results() {
    for name in ["isFinite", "isNaN"] {
        let mut realm = Realm::default();
        let source = format!(
            "let order=''; let o={{valueOf:function(){{order+='v';return {{}};}},toString:function(){{order+='s';return '7';}}}}; {name}(o); order==='vs'"
        );
        assert_eq!(realm.eval(&source), Ok(Value::Boolean(true)));
        assert_eq!(
            realm.eval(&format!(
                "order=''; o.valueOf=()=>{{order+='v';return 8;}};{name}(o);order==='v'"
            )),
            Ok(Value::Boolean(true))
        );
        assert_eq!(
            realm.eval(&format!("order='';o.valueOf=()=>{{throw 9;}};{name}(o)")),
            Err(Error::Thrown(Value::Number(9.0)))
        );
        assert_eq!(realm.eval("order"), Ok(Value::String(JsString::from(""))));
        assert_eq!(
            realm.eval(&format!(
                "o.valueOf=()=>({{}});o.toString=()=>{{throw 10;}};{name}(o)"
            )),
            Err(Error::Thrown(Value::Number(10.0)))
        );
        for argument in ["1n", "{valueOf:()=>1n}", "{__proto__:null}"] {
            assert!(matches!(
                realm.eval(&format!("{name}({argument})")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        }
        // Argument evaluation still runs left-to-right; extra values and this
        // are not themselves coerced by either builtin.
        assert_eq!(realm.eval(&format!("order='';let unused={{valueOf:()=>{{throw 11;}}}};{name}.call(unused,(order+='a',1),(order+='b',unused));order==='ab'")),Ok(Value::Boolean(true)));
        assert_eq!(realm.eval("after=3"), Ok(Value::Number(3.0)));
    }
}

#[test]
fn globals_have_standard_descriptors_and_differ_from_number_predicates() {
    let mut realm = Realm::default();
    assert_eq!(
        realm
            .eval("isFinite('1') && !Number.isFinite('1') && isNaN('bad') && !Number.isNaN('bad')"),
        Ok(Value::Boolean(true))
    );
    let Value::Object(global) = realm.eval("globalThis").unwrap() else {
        panic!("global object")
    };
    for name in ["isFinite", "isNaN"] {
        assert_eq!(
            realm.eval(&format!("{name}!==Number.{name}")),
            Ok(Value::Boolean(true))
        );
        let Value::Object(function) = realm.eval(name).unwrap() else {
            panic!("function")
        };
        let descriptor = realm
            .inspect_object(&global)
            .unwrap()
            .own_property(&JsString::from(name))
            .unwrap()
            .as_data()
            .unwrap();
        assert!(descriptor.writable && !descriptor.enumerable && descriptor.configurable);
        assert_eq!(descriptor.value, Value::Object(function.clone()));
        let object = realm.inspect_object(&function).unwrap();
        assert!(object.is_callable() && !object.is_constructor());
        assert!(object.own_property(&JsString::from("prototype")).is_none());
        for (key, value) in [
            ("name", Value::String(JsString::from(name))),
            ("length", Value::Number(1.0)),
        ] {
            let descriptor = object
                .own_property(&JsString::from(key))
                .unwrap()
                .as_data()
                .unwrap();
            assert_eq!(descriptor.value, value);
            assert!(!descriptor.writable && !descriptor.enumerable && descriptor.configurable);
        }
        assert!(matches!(
            realm.eval(&format!("new {name}(1)")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        assert_eq!(realm.eval(&format!("{name}=4;globalThis.{name}===4 && delete globalThis.{name} && typeof {name}==='undefined'")),Ok(Value::Boolean(true)));
    }
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("Number.isFinite(1) && Number.isNaN(NaN)"),
        Ok(Value::Boolean(true))
    );
}
