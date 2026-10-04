//! UTF-16 sequence methods and ordered observable conversions.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn concat_converts_values_with_string_hint_and_preserves_code_units() {
    check(
        "''.concat()==='' && 'abc'.concat()==='abc' && 'x'.concat(undefined,null,true,12,34n)==='xundefinednulltrue1234'",
    );
    check(
        "'\\uD800'.concat('\\uDC00')==='𐀀' && '\\uDC00'.concat('x','\\uD800')==='\\uDC00x\\uD800'",
    );
    check(
        "String.prototype.concat.call(12,3)==='123' && new String('a').concat('b')==='ab' && typeof new String('a').concat()==='string'",
    );
    check(
        "let log='';let r={toString:()=>{log+='r';return 'a';}},a={toString:()=>{log+='a';return 'b';},valueOf:()=>{throw 1;}},b={toString:()=>{log+='b';return {};},valueOf:()=>{log+='v';return 'c';}};String.prototype.concat.call(r,a,b)==='abc' && log==='rabv'",
    );
    let mut realm = Realm::default();
    realm.eval("let log='';let a={toString:()=>{log+='a';throw 7;}},b={toString:()=>{log+='b';return 'b';}}").unwrap();
    assert_eq!(
        realm.eval("''.concat(a,(log+='e',b))"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    assert_eq!(realm.eval("log==='ea'"), Ok(Value::Boolean(true)));
}

#[test]
fn slice_uses_relative_indices_and_returns_empty_for_reversed_endpoints() {
    for (arguments, expected) in [
        ("", "abcdef"),
        ("undefined,undefined", "abcdef"),
        ("1", "bcdef"),
        ("1,4", "bcd"),
        ("-3", "def"),
        ("1,-1", "bcde"),
        ("-4,-1", "cde"),
        ("-Infinity,Infinity", "abcdef"),
        ("Infinity", ""),
        ("0,-Infinity", ""),
        ("99", ""),
        ("-99", "abcdef"),
        ("4,2", ""),
        ("1.9,4.9", "bcd"),
        ("-1.9", "f"),
        ("-0.9", "abcdef"),
        ("NaN,NaN", ""),
        ("true,'3'", "bc"),
        ("0,null", ""),
    ] {
        check(&format!("'abcdef'.slice({arguments})==='{expected}'"));
    }
}

#[test]
fn substring_clamps_negatives_and_swaps_endpoints() {
    for (arguments, expected) in [
        ("", "abcdef"),
        ("undefined,undefined", "abcdef"),
        ("1", "bcdef"),
        ("1,4", "bcd"),
        ("4,1", "bcd"),
        ("-3", "abcdef"),
        ("1,-1", "a"),
        ("-4,-1", ""),
        ("-Infinity,Infinity", "abcdef"),
        ("Infinity,2", "cdef"),
        ("0,-Infinity", ""),
        ("99", ""),
        ("-99", "abcdef"),
        ("1.9,4.9", "bcd"),
        ("NaN,NaN", ""),
        ("true,'3'", "bc"),
        ("3,null", "abc"),
    ] {
        check(&format!("'abcdef'.substring({arguments})==='{expected}'"));
    }
}

#[test]
fn substrings_may_split_pairs_and_preserve_unpaired_surrogates() {
    for name in ["slice", "substring"] {
        check(&format!(
            "'a💩b'.{name}(1,2)==='\\uD83D' && 'a💩b'.{name}(2,3)==='\\uDCA9' && 'a💩b'.{name}(1,3)==='💩' && '\\uD800x\\uDC00'.{name}(0,2)==='\\uD800x'"
        ));
        check(&format!(
            "String.prototype.{name}.call(123n,1,2)==='2' && new String('abc').{name}(1)==='bc'"
        ));
    }
}

#[test]
fn substring_conversions_run_in_order_even_for_empty_or_reversed_ranges() {
    for name in ["slice", "substring"] {
        for value in ["''", "'abc'"] {
            check(&format!(
                "let log='';let r={{toString:()=>{{log+='r';return {value};}}}},a={{valueOf:()=>{{log+='a';return 3;}},toString:()=>{{throw 1;}}}},b={{valueOf:()=>{{log+='b';return 0;}}}};String.prototype.{name}.call(r,a,b);log==='rab'"
            ));
        }
        let mut realm = Realm::default();
        realm.eval("let log='';let r={toString:()=>{log+='r';throw 7;}},a={valueOf:()=>{log+='a';throw 8;}},b={valueOf:()=>{log+='b';throw 9;}}").unwrap();
        assert_eq!(
            realm.eval(&format!("String.prototype.{name}.call(r,a,b)")),
            Err(Error::Thrown(Value::Number(7.0)))
        );
        assert_eq!(realm.eval("log==='r'"), Ok(Value::Boolean(true)));
        assert_eq!(
            realm.eval(&format!("''.{name}(a,b)")),
            Err(Error::Thrown(Value::Number(8.0)))
        );
        assert_eq!(realm.eval("log==='ra'"), Ok(Value::Boolean(true)));
        assert_eq!(
            realm.eval(&format!("''.{name}(0,b)")),
            Err(Error::Thrown(Value::Number(9.0)))
        );
        assert_eq!(realm.eval("log==='rab'"), Ok(Value::Boolean(true)));
        for arguments in ["1n", "0,1n"] {
            assert!(matches!(
                Realm::default().eval(&format!("''.{name}({arguments})")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        }
    }
}

#[test]
fn generic_methods_reject_nullish_receivers_and_have_standard_metadata() {
    for (name, length) in [("concat", 1), ("slice", 2), ("substring", 2)] {
        for nullish in ["null", "undefined"] {
            assert!(matches!(Realm::default().eval(&format!("String.prototype.{name}.call({nullish},{{valueOf:()=>{{throw 1;}},toString:()=>{{throw 2;}}}})")), Err(Error::Exception{kind:ExceptionKind::TypeError,..})));
        }
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(String.prototype,'{name}');d.writable && !d.enumerable && d.configurable && d.value.name==='{name}' && d.value.length==={length} && !Object.hasOwn(d.value,'prototype')"
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
        realm.eval("'a'.concat('bc').slice(1).substring(1,0)==='b'"),
        Ok(Value::Boolean(true))
    );
}
