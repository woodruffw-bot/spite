//! Ordinary String.raw calls, array-like access, and complete constructor reflection.

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

use crate::common::assert_true;

#[test]
fn raw_interleaves_available_substitutions_with_utf16_literals() {
    assert_true(
        r#"
            String.raw({
                raw: {
                    0: 'A',
                    1: 'B',
                    2: 'C',
                    length: 3
                }
            }, 1, 2) === 'A1B2C' && String.raw({
                raw: {
                    0: 'A',
                    1: 'B',
                    2: 'C',
                    length: 3
                }
            }) === 'ABC' && String.raw({
                raw: {
                    0: 'A',
                    1: 'B',
                    length: 2
                }
            }, undefined) === 'AundefinedB'
        "#,
    );
    assert_true(
        "String.raw({raw:{length:3,1:'B'}},'?')==='undefined?Bundefined' && String.raw({raw:Object.create({0:'A'},{length:{value:1}})})==='A'",
    );
    assert_true(
        r"String.raw({raw:{0:'\n',length:1}})==='\n' && String.raw({raw:{0:'\\n',length:1}})==='\\n'",
    );
    assert_true(
        "String.raw({raw:{0:'\\uD800',1:'x',length:2}},'\\uDC00')==='𐀀x' && String.raw({raw:{0:'\\uDC00',length:1}})==='\\uDC00'",
    );
}

#[test]
fn raw_boxes_template_and_raw_values_and_ignores_its_this_value() {
    assert_true(
        "String.raw({raw:'abc'},1,2)==='a1b2c' && String.raw.call(null,{raw:'ab'},'x')==='axb'",
    );
    assert_true("Number.prototype.raw='ab';String.raw(3,'-')==='a-b'");
    assert_true(
        "Boolean.prototype.length=1;Boolean.prototype[0]='b';String.raw({raw:false})==='b'",
    );
    for template in ["undefined", "null", "{}", "{raw:undefined}", "{raw:null}"] {
        assert!(matches!(
            Realm::default().eval(&format!("String.raw({template})")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    assert!(matches!(
        Realm::default().eval("String.raw({raw:{length:1n}})"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn raw_reads_length_once_and_observes_each_conversion_in_order() {
    assert_true(
        r#"
            let log = '',
                t = {},
                r = {};
            Object.defineProperty(t, 'raw', {
                get: function() {
                    if (this !== t) throw 1;
                    log += 'r';
                    return r;
                }
            });
            Object.defineProperty(r, 'length', {
                get: function() {
                    if (this !== r) throw 2;
                    log += 'l';
                    return {
                        valueOf: () => {
                            log += 'n';
                            return 2;
                        }
                    };
                }
            });
            Object.defineProperty(r, '0', {
                get: () => {
                    log += '0';
                    return {
                        toString: () => {
                            log += 'a';
                            return 'A';
                        }
                    };
                }
            });
            Object.defineProperty(r, '1', {
                get: () => {
                    log += '1';
                    return {
                        toString: () => {
                            log += 'b';
                            return 'B';
                        }
                    };
                }
            });
            String.raw(t, {
                toString: () => {
                    log += 's';
                    return '-';
                }
            }) === 'A-B' && log === 'rln0as1b'
        "#,
    );
    assert_true(
        r#"
            let r = {
                    0: 'A',
                    1: 'B',
                    length: 2
                },
                sub = {
                    toString: () => {
                        r.length = 0;
                        r[1] = 'Z';
                        return '-';
                    }
                };
            String.raw({
                raw: r
            }, sub) === 'A-Z'
        "#,
    );
    for length in [
        "undefined",
        "NaN",
        "null",
        "false",
        "0",
        "-1",
        "-Infinity",
        "0.9",
    ] {
        assert_true(&format!(
            r#"
                let r = {{
                    length: {length}
                }};
                Object.defineProperty(r, '0', {{
                    get: () => {{
                        throw 1;
                    }}
                }});
                String.raw({{
                    raw: r
                }}, {{
                    toString: () => {{
                        throw 2;
                    }}
                }}) === ''
            "#
        ));
    }
    assert_true("String.raw({raw:{0:'A',1:'B',length:'2.9'}},'-')==='A-B'");
}

#[test]
fn only_needed_substitutions_are_converted_but_all_arguments_are_evaluated() {
    assert_true(
        r#"
            let n = 0,
                bad = {
                    toString: () => {
                        throw 1;
                    }
                };
            String.raw({
                raw: {
                    0: 'A',
                    length: 1
                }
            }, (n++, bad), bad) === 'A' && n === 1
        "#,
    );
    assert_true("String.raw({raw:{0:'A',1:'B',length:2}},'-',{toString:()=>{throw 1;}})==='A-B'");
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
                let log = '',
                    r = {
                        length: 3
                    };
                Object.defineProperty(r, '0', {
                    get: () => {
                        log += '0';
                        return 'A';
                    }
                });
                Object.defineProperty(r, '1', {
                    get: () => {
                        log += '1';
                        throw 7;
                    }
                });
                Object.defineProperty(r, '2', {
                    get: () => {
                        log += '2';
                        return 'C';
                    }
                })
            "#,
        )
        .unwrap();
    assert_eq!(realm.eval("String.raw({raw:r},{toString:()=>{log+='s';return '-';}},{toString:()=>{log+='t';return '-';}})"),Err(Error::Thrown(Value::Number(7.0))));
    assert_eq!(realm.eval("log==='0s1'"), Ok(Value::Boolean(true)));
    assert_eq!(
        Realm::default().eval(
            "String.raw({raw:{0:{toString:()=>{throw 8;}},length:2}},{toString:()=>{throw 9;}})"
        ),
        Err(Error::Thrown(Value::Number(8.0)))
    );
    assert_eq!(
        Realm::default().eval("String.raw({raw:{0:'A',length:2}},{toString:()=>{throw 9;}})"),
        Err(Error::Thrown(Value::Number(9.0)))
    );
}

#[test]
fn constructor_static_properties_can_be_reflected_and_frozen() {
    assert_true(
        r#"
            let d = Object.getOwnPropertyDescriptor(String, 'raw');
            d.writable && !d.enumerable && d.configurable && d.value.name === 'raw' && d.value.length ===
                1 && !Object.hasOwn(d.value, 'prototype')
        "#,
    );
    assert!(matches!(
        Realm::default().eval("new String.raw()"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    let mut realm = Realm::default();
    let Value::Object(descriptors) = realm
        .eval("Object.getOwnPropertyDescriptors(String)")
        .unwrap()
    else {
        panic!("descriptors");
    };
    let mut keys = realm.inspect_object(&descriptors).unwrap().own_keys();
    keys.sort_by(|a, b| {
        a.as_string()
            .expect("string key")
            .cmp(b.as_string().expect("string key"))
    });
    assert_eq!(
        keys,
        [
            "fromCharCode",
            "fromCodePoint",
            "length",
            "name",
            "prototype",
            "raw"
        ]
        .map(spite_core::PropertyKey::from)
    );
    assert_true(
        r#"
            Object.freeze(String) === String && Object.isFrozen(String) && String.raw({
                raw: 'a'
            }) === 'a' && !Object.isFrozen(String.prototype)
        "#,
    );
    assert_true(
        r#"
            let r = String.raw;
            delete String.raw;
            String.raw === undefined && !Object.hasOwn(String, 'raw') && !Object.hasOwn(Object
                .getOwnPropertyDescriptors(String), 'raw') && r({
                raw: 'a'
            }) === 'a'
        "#,
    );
    realm
        .eval("let r=String.raw;delete globalThis.String")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("r({raw:'ab'},'-')"),
        Ok(Value::String(JsString::from("a-b")))
    );
}

#[test]
fn output_and_iteration_limits_abort_without_running_catch_or_finally() {
    let mut realm = Realm::new(Limits {
        max_string_units: Some(64),
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval("try{String.raw({raw:'ab'},'x'.repeat(63));}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(
        realm.eval("String.raw({raw:'ab'},'x'.repeat(62)).length"),
        Ok(Value::Number(64.0))
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(1000),
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval("try{String.raw({raw:{length:Infinity}});}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
