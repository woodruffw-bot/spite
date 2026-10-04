//! UTF-16 indexOf/lastIndexOf and observable conversion order.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn search_positions_distinguish_forward_and_backward_nan_defaults() {
    for (position, forward, backward) in [
        ("undefined", 0, 3),
        ("NaN", 0, 3),
        ("null", 0, 0),
        ("false", 0, 0),
        ("true", 3, 0),
        ("-Infinity", 0, 0),
        ("Infinity", -1, 3),
        ("-1", 0, 0),
        ("-0", 0, 0),
        ("-0.9", 0, 0),
        ("1.9", 3, 0),
        ("3.9", 3, 3),
        ("'3'", 3, 3),
        ("4", -1, 3),
        ("100", -1, 3),
    ] {
        check(&format!(
            "'abcabc'.indexOf('ab',{position})==={forward} && 'abcabc'.lastIndexOf('ab',{position})==={backward}"
        ));
    }
    check(
        "'abcabc'.indexOf('ab')===0 && 'abcabc'.lastIndexOf('ab')===3 && 'aba'.lastIndexOf('ba',1)===1 && 'aba'.lastIndexOf('aba',0)===0",
    );
    check(
        "'undefinedundefined'.indexOf()===0 && 'undefinedundefined'.lastIndexOf()===9 && 'null'.indexOf(null)===0",
    );
}

#[test]
fn empty_needles_and_impossible_matches_have_exact_boundary_results() {
    for name in ["indexOf", "lastIndexOf"] {
        for (position, expected) in [
            ("-Infinity", 0),
            ("-1", 0),
            ("-0", 0),
            ("1.9", 1),
            ("3", 3),
            ("Infinity", 3),
        ] {
            check(&format!("'abc'.{name}('',{position})==={expected}"));
        }
        check(&format!(
            "''.{name}('')===0 && ''.{name}('a')===-1 && 'ab'.{name}('abc')===-1 && 'abab'.{name}('c')===-1"
        ));
        check(&format!("Object.is(''.{name}('',-0),0)"));
    }
    check(
        "'abc'.indexOf('',NaN)===0 && 'abc'.lastIndexOf('',NaN)===3 && 'abc'.indexOf('',undefined)===0 && 'abc'.lastIndexOf('',undefined)===3",
    );
}

#[test]
fn searches_compare_code_units_including_surrogate_halves() {
    check(
        "'💩x💩'.indexOf('💩')===0 && '💩x💩'.lastIndexOf('💩')===3 && '💩x💩'.indexOf('\\uDCA9')===1 && '💩x💩'.lastIndexOf('\\uDCA9')===4",
    );
    check(
        "'\\uD800x\\uD800'.indexOf('\\uD800')===0 && '\\uD800x\\uD800'.lastIndexOf('\\uD800')===2 && 'é'.indexOf('é')===-1",
    );
    check(
        "'aaaa'.indexOf('aa',1)===1 && 'aaaa'.lastIndexOf('aa')===2 && 'aaaa'.lastIndexOf('aa',1)===1",
    );
}

#[test]
fn receiver_and_search_use_string_hint_before_position_conversion() {
    for name in ["indexOf", "lastIndexOf"] {
        check(&format!(
            "let log='';let r={{toString:()=>{{log+='r';return 'abc';}}}},s={{toString:()=>{{log+='s';return 'b';}},valueOf:()=>{{throw 1;}}}},p={{valueOf:()=>{{log+='p';return 1;}},toString:()=>{{throw 2;}}}};String.prototype.{name}.call(r,s,p)===1 && log==='rsp'"
        ));
        check(&format!(
            "String.prototype.{name}.call(123n,2)===1 && new String('abc').{name}('b')===1 && String.prototype.{name}.call(true,'r')===1"
        ));
        // Even an impossible length comparison observes the position conversion.
        assert_eq!(
            Realm::default().eval(&format!("''.{name}('longer',{{valueOf:()=>{{throw 8;}}}})")),
            Err(Error::Thrown(Value::Number(8.0)))
        );
        assert_eq!(Realm::default().eval(&format!("String.prototype.{name}.call({{toString:()=>{{throw 6;}}}},{{toString:()=>{{throw 7;}}}},{{valueOf:()=>{{throw 8;}}}})")),Err(Error::Thrown(Value::Number(6.0))));
        assert_eq!(
            Realm::default().eval(&format!(
                "''.{name}({{toString:()=>{{throw 7;}}}},{{valueOf:()=>{{throw 8;}}}})"
            )),
            Err(Error::Thrown(Value::Number(7.0)))
        );
        assert!(matches!(
            Realm::default().eval(&format!("''.{name}('',1n)")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
}

#[test]
fn search_methods_reject_nullish_receivers_and_preserve_standard_metadata() {
    for name in ["indexOf", "lastIndexOf"] {
        for receiver in ["null", "undefined"] {
            assert!(matches!(
                Realm::default().eval(&format!(
                    "String.prototype.{name}.call({receiver},{{toString:()=>{{throw 1;}}}})"
                )),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        }
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(String.prototype,'{name}');d.writable && !d.enumerable && d.configurable && d.value.name==='{name}' && d.value.length===1 && !Object.hasOwn(d.value,'prototype')"
        ));
        assert!(matches!(
            Realm::default().eval(&format!("new String.prototype.{name}()")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    let mut realm = Realm::default();
    realm.eval("delete globalThis.String").unwrap();
    realm.collect(100_000).unwrap();
    assert_eq!(
        realm.eval("'aba'.indexOf('a')===0 && 'aba'.lastIndexOf('a')===2"),
        Ok(Value::Boolean(true))
    );
}
