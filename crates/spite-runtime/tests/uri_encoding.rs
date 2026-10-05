//! URI percent encoding, Unicode validation, and intrinsic retention (19.2.6).

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn escape_sets_differ_and_octets_use_uppercase_hex_without_normalization() {
    check(
        "let s=\"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-.!~*'()\";encodeURI(s)===s && encodeURIComponent(s)===s",
    );
    check(
        "let s=';/?:@&=+$,#';encodeURI(s)===s && encodeURIComponent(s)==='%3B%2F%3F%3A%40%26%3D%2B%24%2C%23'",
    );
    for name in ["encodeURI", "encodeURIComponent"] {
        check(&format!(
            "{name}(' %[]\\\\^`{{}}|')==='%20%25%5B%5D%5C%5E%60%7B%7D%7C'"
        ));
        check(&format!(
            "{name}('\\0\\x7F\\u0080\\u07FF\\u0800\\uD7FF\\uE000\\uFFFF\\u{{10000}}\\u{{10FFFF}}')==='%00%7F%C2%80%DF%BF%E0%A0%80%ED%9F%BF%EE%80%80%EF%BF%BF%F0%90%80%80%F4%8F%BF%BF'"
        ));
        check(&format!(
            "{name}('é e\\u0301 \\uFEFF')==='%C3%A9%20e%CC%81%20%EF%BB%BF' && {name}('%20')==='%2520'"
        ));
    }
}

#[test]
fn primitive_and_object_inputs_use_once_only_string_hint_conversion() {
    for name in ["encodeURI", "encodeURIComponent"] {
        check(&format!(
            "{name}()==='undefined' && {name}(null)==='null' && {name}(true)==='true' && {name}(7n)==='7' && {name}(-0)==='0' && {name}(Infinity)==='Infinity' && {name}(new String('é'))==='%C3%A9'"
        ));
        check(&format!(
            "let log='',o={{[Symbol.toPrimitive](hint){{log+=hint;return 'é';}},toString(){{throw 7;}}}};{name}(o)==='%C3%A9' && log==='string'"
        ));
        check(&format!(
            "let log='',o={{toString(){{log+='s';return {{}};}},valueOf(){{log+='v';return 'é';}}}};{name}(o)==='%C3%A9' && log==='sv'"
        ));
        check(&format!(
            "let sentinel={{}},o={{toString(){{throw sentinel;}},valueOf(){{throw 7;}}}},caught=false;try{{{name}(o);}}catch(e){{caught=e===sentinel;}}caught"
        ));
        for input in [
            "Symbol()",
            "Object(Symbol())",
            "{[Symbol.toPrimitive](){return Symbol();}}",
            "{__proto__:null}",
        ] {
            assert!(
                matches!(
                    Realm::default().eval(&format!("{name}({input})")),
                    Err(Error::Exception {
                        kind: ExceptionKind::TypeError,
                        ..
                    })
                ),
                "{name}({input})"
            );
        }
        check(&format!(
            "let log='',o={{toString(){{throw 7;}}}};{name}.call(o,(log+='a','é'),(log+='b',o))==='%C3%A9' && log==='ab'"
        ));
    }
}

#[test]
fn lone_surrogates_throw_urierror_and_paired_surrogates_encode_normally() {
    for name in ["encodeURI", "encodeURIComponent"] {
        for input in [
            "\\uD800",
            "\\uDBFF",
            "\\uDC00",
            "\\uDFFF",
            "x\\uD800",
            "\\uD800x",
            "\\uD800\\uD800",
            "\\uDC00\\uD800",
            "\\uD800\\uDC00\\uDC00",
        ] {
            let source = format!("{name}('{input}')");
            assert!(
                matches!(
                    Realm::default().eval(&source),
                    Err(Error::Exception {
                        kind: ExceptionKind::URIError,
                        ..
                    })
                ),
                "{source}"
            );
            check(&format!(
                "let flag=0,caught=false;try{{{source};}}catch(e){{caught=e instanceof URIError && e instanceof Error && Error.isError(e);flag++;}}finally{{flag++;}}caught && flag===2"
            ));
        }
        check(&format!(
            "{name}('\\uD800\\uDC00\\uDBFF\\uDFFF')==='%F0%90%80%80%F4%8F%BF%BF'"
        ));
        check(&format!(
            "let Original=URIError;URIError=TypeError;let caught=false;try{{{name}('\\uD800');}}catch(e){{caught=e instanceof Original && !(e instanceof TypeError);}}caught"
        ));
    }
    let mut realm = Realm::default();
    let error = realm.eval("encodeURI('\\uD800')").unwrap_err();
    let Value::Object(object) = realm.exception_value(&error).unwrap().unwrap() else {
        panic!("URIError object")
    };
    assert!(
        realm
            .inspect_object(&object)
            .unwrap()
            .own_property(&JsString::from("message"))
            .is_some()
    );
}

#[test]
fn globals_and_function_metadata_have_standard_descriptors() {
    let mut realm = Realm::default();
    let Value::Object(global) = realm.eval("globalThis").unwrap() else {
        panic!("global object")
    };
    let mut functions = Vec::new();
    for name in ["encodeURI", "encodeURIComponent"] {
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
            ("name", Value::String(name.into())),
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
            realm.eval(&format!("new {name}({{toString(){{throw 7;}}}})")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        assert_eq!(realm.eval(&format!("{name}=7;globalThis.{name}===7 && delete globalThis.{name} && typeof {name}==='undefined'")),Ok(Value::Boolean(true)));
        functions.push(function);
    }
    realm.collect(usize::MAX).unwrap();
    assert!(
        functions
            .iter()
            .all(|function| realm.inspect_object(function).is_ok())
    );
}

#[test]
fn large_default_outputs_are_allowed_and_opted_in_host_failures_abort_handlers() {
    check(
        "let s='é😀 '.repeat(3000);encodeURI(s).length===63000 && encodeURIComponent(s).length===63000",
    );
    let mut realm = Realm::new(Limits {
        max_string_units: Some(2),
        ..Limits::default()
    });
    realm.eval("let flag=0;").unwrap();
    assert!(matches!(
        realm.eval("try{encodeURI(' ');}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::default();
    realm.eval("let flag=0;").unwrap();
    assert!(matches!(
        realm.eval(
            "try{encodeURI({toString(){Function('class C{#field;}');}});}catch{flag=1;}finally{flag=2;}"
        ),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
