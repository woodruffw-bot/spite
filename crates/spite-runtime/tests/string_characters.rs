//! String code-unit/code-point construction and generic indexed character access.

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn code_unit_construction_uses_modulo_uint16_and_keeps_surrogates() {
    check(
        "String.fromCharCode()==='' && String.fromCharCode(65,66,67)==='ABC' && String.fromCharCode(0xD83D,0xDCA9)==='💩' && String.fromCharCode(0xD800)==='\\uD800'",
    );
    for (input, unit) in [
        ("undefined", 0),
        ("NaN", 0),
        ("Infinity", 0),
        ("-Infinity", 0),
        ("null", 0),
        ("true", 1),
        ("-0", 0),
        ("65536", 0),
        ("65537.9", 1),
        ("-1.9", 65535),
        ("4294967361", 65),
        ("9007199254740991", 65535),
    ] {
        assert_eq!(
            Realm::default().eval(&format!("String.fromCharCode({input})")),
            Ok(Value::String(JsString::from_code_units(vec![unit]))),
            "{input}"
        );
    }
    assert!(matches!(
        Realm::default().eval("String.fromCharCode(1n)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn code_point_construction_accepts_surrogates_but_rejects_non_integral_or_out_of_range_values() {
    check(
        "String.fromCodePoint()==='' && String.fromCodePoint(65,0x1F4A9,0xD800)==='A💩\\uD800' && String.fromCodePoint(-0)==='\\u0000' && String.fromCodePoint('66')==='B'",
    );
    for (point, units) in [
        (0xffff, vec![0xffff]),
        (0x10000, vec![0xd800, 0xdc00]),
        (0x10ffff, vec![0xdbff, 0xdfff]),
        (0xdc00, vec![0xdc00]),
    ] {
        assert_eq!(
            Realm::default().eval(&format!("String.fromCodePoint({point})")),
            Ok(Value::String(JsString::from_code_units(units)))
        );
    }
    for input in [
        "undefined",
        "NaN",
        "Infinity",
        "-Infinity",
        "-1",
        "0.5",
        "1114112",
    ] {
        assert!(
            matches!(
                Realm::default().eval(&format!("String.fromCodePoint({input})")),
                Err(Error::Exception {
                    kind: ExceptionKind::RangeError,
                    ..
                })
            ),
            "{input}"
        );
    }
    assert!(matches!(
        Realm::default().eval("String.fromCodePoint(1n)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn variadic_construction_converts_each_argument_once_and_stops_on_abrupt_completion() {
    for name in ["fromCharCode", "fromCodePoint"] {
        check(&format!(
            "let log='';let a={{valueOf:()=>{{log+='a';return 65;}},toString:()=>{{throw 1;}}}},b={{valueOf:()=>{{log+='b';return 66;}}}};String.{name}(a,b)==='AB' && log==='ab'"
        ));
        let mut realm = Realm::default();
        realm.eval("let log='',evaluated=0;let a={valueOf:()=>{log+='a';return 65;}},b={valueOf:()=>{log+='b';throw 9;}},c={valueOf:()=>{log+='c';return 67;}}").unwrap();
        assert_eq!(
            realm.eval(&format!("String.{name}(a,b,(evaluated=1,c))")),
            Err(Error::Thrown(Value::Number(9.0)))
        );
        assert_eq!(
            realm.eval("log==='ab' && evaluated===1"),
            Ok(Value::Boolean(true))
        );
    }
}

#[test]
fn indexed_methods_use_utf16_and_keep_distinct_out_of_range_results() {
    check(
        "'💩x'.at(0)==='\\uD83D' && '💩x'.at(1)==='\\uDCA9' && '💩x'.at(-1)==='x' && '💩x'.at(-2)==='\\uDCA9' && '💩x'.at(-3)==='\\uD83D'",
    );
    check(
        "'💩x'.charAt(0)==='\\uD83D' && '💩x'.charAt(1)==='\\uDCA9' && '💩x'.charCodeAt(0)===0xD83D && '💩x'.charCodeAt(1)===0xDCA9 && '💩x'.codePointAt(0)===0x1F4A9 && '💩x'.codePointAt(1)===0xDCA9",
    );
    check(
        "'\\uD800x'.codePointAt(0)===0xD800 && '\\uDC00\\uD800'.codePointAt(0)===0xDC00 && '\\uD800'.codePointAt(0)===0xD800",
    );
    for index in ["3", "4", "Infinity", "-Infinity", "-4"] {
        check(&format!(
            "'abc'.at({index})===undefined && 'abc'.charAt({index})==='' && Number.isNaN('abc'.charCodeAt({index})) && 'abc'.codePointAt({index})===undefined"
        ));
    }
    check(
        "''.at()===undefined && ''.charAt()==='' && Number.isNaN(''.charCodeAt()) && ''.codePointAt()===undefined && 'abc'.charAt(-1)==='' && 'abc'.at(-1)==='c'",
    );
    for index in ["undefined", "NaN", "null", "false", "-0", "-0.5"] {
        check(&format!(
            "'abc'.at({index})==='a' && 'abc'.charAt({index})==='a' && 'abc'.charCodeAt({index})===97 && 'abc'.codePointAt({index})===97"
        ));
    }
    check("'abc'.at(-1.9)==='c' && 'abc'.charAt(1.9)==='b' && 'abc'.charAt('2')==='c'");
}

#[test]
fn character_methods_are_generic_and_convert_receivers_before_positions() {
    for name in ["at", "charAt", "charCodeAt", "codePointAt"] {
        let mut realm = Realm::default();
        realm.eval("let log='';let r={toString:()=>{log+='s';return 'ab';},valueOf:()=>{throw 1;}},i={valueOf:()=>{log+='i';return 1;},toString:()=>{throw 2;}}").unwrap();
        let expected = if matches!(name, "at" | "charAt") {
            "'b'"
        } else {
            "98"
        };
        assert_eq!(
            realm.eval(&format!(
                "String.prototype.{name}.call(r,i)==={expected} && log==='si'"
            )),
            Ok(Value::Boolean(true))
        );
        assert_eq!(realm.eval(&format!("String.prototype.{name}.call({{toString:()=>{{throw 7;}}}},{{valueOf:()=>{{throw 8;}}}})")),Err(Error::Thrown(Value::Number(7.0))));
        for nullish in ["null", "undefined"] {
            assert!(matches!(
                realm.eval(&format!(
                    "String.prototype.{name}.call({nullish},{{valueOf:()=>{{throw 8;}}}})"
                )),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        }
        assert!(matches!(
            realm.eval(&format!("'abc'.{name}(1n)")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    check(
        "String.prototype.charAt.call(123,1)==='2' && String.prototype.charAt.call(123n,1)==='2' && String.prototype.at.call(true,-1)==='e' && new String('abc').charAt(2)==='c'",
    );
}

#[test]
fn methods_have_standard_metadata_and_survive_global_deletion() {
    for (owner, names) in [
        ("String", &["fromCharCode", "fromCodePoint"][..]),
        (
            "String.prototype",
            &["at", "charAt", "charCodeAt", "codePointAt"][..],
        ),
    ] {
        for name in names {
            check(&format!(
                "let d=Object.getOwnPropertyDescriptor({owner},'{name}');d.writable && !d.enumerable && d.configurable && d.value.name==='{name}' && d.value.length===1"
            ));
            assert!(matches!(
                Realm::default().eval(&format!("new {owner}.{name}()")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        }
    }
    let mut realm = Realm::default();
    realm.eval("let C=String;delete globalThis.String").unwrap();
    realm.collect(100_000).unwrap();
    assert_eq!(
        realm.eval(
            "C.fromCodePoint(0x1F4A9).codePointAt(0)===0x1F4A9 && C.fromCharCode(65).at(0)==='A'"
        ),
        Ok(Value::Boolean(true))
    );
}
