//! Strict escaped UTF-8 and preserved literal UTF-16 (19.2.6.1–2, 19.2.6.6).

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
fn reserved_escapes_keep_their_original_spelling_only_for_decodeuri() {
    for (encoded, decoded) in [
        ("%3B%2F%3F%3A%40%26%3D%2B%24%2C%23", ";/?:@&=+$,#"),
        ("%3b%2f%3f%3a%40%26%3d%2b%24%2c%23", ";/?:@&=+$,#"),
    ] {
        check(&format!(
            "decodeURI('{encoded}')==='{encoded}' && decodeURIComponent('{encoded}')==='{decoded}'"
        ));
    }
    for name in ["decodeURI", "decodeURIComponent"] {
        check(&format!(
            "{name}('%25%32%46')==='%2F' && {name}('%252f')==='%2f' && {name}('+')==='+'"
        ));
        check(&format!(
            "{name}('%20%25%5b%5D%5c%5E%60%7b%7D%7c')===' %[]\\\\^`{{}}|'"
        ));
    }
    check(
        "decodeURI('http%3a%2f%2fexample.com%2f%C3%A9%3fkey%3dvalue%23x')==='http%3a%2f%2fexample.com%2fé%3fkey%3dvalue%23x' && decodeURIComponent('http%3a%2f%2fexample.com%2f%C3%A9%3fkey%3dvalue%23x')==='http://example.com/é?key=value#x'",
    );
}

#[test]
fn valid_utf8_boundaries_and_literal_surrogates_are_preserved_without_normalization() {
    for name in ["decodeURI", "decodeURIComponent"] {
        check(&format!(
            "{name}('%00%7f%c2%80%df%bf%e0%a0%80%ed%9f%bf%ee%80%80%ef%bf%bf%f0%90%80%80%f4%8f%bf%bf')==='\\0\\x7F\\u0080\\u07FF\\u0800\\uD7FF\\uE000\\uFFFF\\u{{10000}}\\u{{10FFFF}}'"
        ));
        check(&format!(
            "{name}('%C3%A9%20e%CC%81%20%EF%BB%BF')==='é e\\u0301 \\uFEFF'"
        ));
        check(&format!(
            "let s='x\\uD800\\uDC00\\uD800y\\uDC00';{name}(s)===s && {name}('\\uD800%41\\uDC00')==='\\uD800A\\uDC00'"
        ));
        check(&format!(
            "let s='é😀_-.!~*()';{name}(encodeURIComponent(s))===s"
        ));
    }
}

#[test]
fn malformed_percent_sequences_and_invalid_utf8_throw_urierror() {
    let invalid = [
        "%",
        "%0",
        "%GG",
        "%2g",
        "%%20",
        "%\\uFF16\\uFF10",
        "%80",
        "%BF",
        "%C0%80",
        "%C1%BF",
        "%C2",
        "%C2%41",
        "%C2x80",
        "%E0%9F%BF",
        "%E2%82",
        "%E2x82%AC",
        "%E2%82xAC",
        "%E2%82%G0",
        "%ED%A0%80",
        "%ED%BF%BF",
        "%ED%A0%80%ED%B0%80",
        "%F0%8F%BF%BF",
        "%F0%90%80",
        "%F4%90%80%80",
        "%F5%80%80%80",
        "%F8%88%80%80%80",
        "%FC%84%80%80%80%80",
        "%FF",
        "ok%20%FF",
    ];
    for name in ["decodeURI", "decodeURIComponent"] {
        for input in invalid {
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
        }
        // Every continuation byte is forbidden as a leading byte; F5–FF
        // cannot start the UTF-8 encoding of a Unicode scalar value.
        for first in (0x80..=0xbf).chain(0xf5..=0xff) {
            let source = format!("{name}('%{first:02X}%80%80%80')");
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
        }
        check(&format!(
            "let flag=0,caught=false;try{{{name}('%ED%A0%80');}}catch(e){{caught=e instanceof URIError && e instanceof Error && Error.isError(e);flag++;}}finally{{flag++;}}caught && flag===2"
        ));
        check(&format!(
            "let Original=URIError;URIError=TypeError;let caught=false;try{{{name}('%FF');}}catch(e){{caught=e instanceof Original && !(e instanceof TypeError);}}caught"
        ));
    }
}

#[test]
fn conversion_is_string_hinted_once_and_receivers_extra_values_are_ignored() {
    for name in ["decodeURI", "decodeURIComponent"] {
        check(&format!(
            "{name}()==='undefined' && {name}(null)==='null' && {name}(7n)==='7' && {name}(-0)==='0' && {name}(new String('%C3%A9'))==='é'"
        ));
        check(&format!(
            "let log='',o={{[Symbol.toPrimitive](hint){{log+=hint;return '%C3%A9';}},toString(){{throw 7;}}}};{name}(o)==='é' && log==='string'"
        ));
        check(&format!(
            "let log='',o={{toString(){{log+='s';return {{}};}},valueOf(){{log+='v';return '%C3%A9';}}}};{name}(o)==='é' && log==='sv'"
        ));
        check(&format!(
            "let sentinel={{}},o={{toString(){{throw sentinel;}}}},caught=false;try{{{name}(o);}}catch(e){{caught=e===sentinel;}}caught"
        ));
        for input in [
            "Symbol()",
            "Object(Symbol())",
            "{[Symbol.toPrimitive](){return Symbol();}}",
            "{__proto__:null}",
        ] {
            assert!(matches!(
                Realm::default().eval(&format!("{name}({input})")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        }
        check(&format!(
            "let log='',o={{toString(){{throw 7;}}}};{name}.call(o,(log+='a','%C3%A9'),(log+='b',o))==='é' && log==='ab'"
        ));
    }
}

#[test]
fn descriptors_nonconstruction_and_intrinsic_retention_follow_standard_globals() {
    let mut realm = Realm::default();
    let Value::Object(global) = realm.eval("globalThis").unwrap() else {
        panic!("global object")
    };
    let mut functions = Vec::new();
    for name in ["decodeURI", "decodeURIComponent"] {
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
fn large_default_inputs_and_opted_in_host_failures_keep_their_distinct_behavior() {
    check(
        "let s='%F0%9F%98%80%20'.repeat(3000),expected='😀 '.repeat(3000);decodeURI(s)===expected && decodeURIComponent(s)===expected",
    );
    check("let s='é😀\\uD800+ '.repeat(3000);decodeURI(s)===s && decodeURIComponent(s)===s");
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm
        .eval("let flag=0,o={toString(){while(true){};}};")
        .unwrap();
    assert!(matches!(
        realm.eval("try{decodeURI(o);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::default();
    realm.eval("let flag=0;").unwrap();
    assert!(matches!(
        realm.eval(
            "try{decodeURI({toString(){Function('class C extends Object{}');}});}catch{flag=1;}finally{flag=2;}"
        ),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
