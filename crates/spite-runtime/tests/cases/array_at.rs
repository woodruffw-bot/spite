//! Array.prototype.at and generic relative indexing (23.1.3.1).

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn relative_indices_truncate_and_preserve_absence() {
    check(
        "let a=[1,,3];a.at()===1 && a.at(undefined)===1 && a.at(null)===1 && a.at(NaN)===1 && a.at(-0)===1 && a.at(-0.5)===1 && a.at(-1)===3 && a.at(-1.9)===3 && a.at('2')===3 && a.at(1)===undefined",
    );
    check(
        "let a=[1,2,3];a.at(-3)===1 && a.at(-4)===undefined && a.at(3)===undefined && a.at(Infinity)===undefined && a.at(-Infinity)===undefined && [].at(0)===undefined",
    );
    check(
        "Object.defineProperty(Array.prototype,'0',{value:7,writable:true,configurable:true});let a=[,];a.at(0)===7 && !Object.hasOwn(a,'0')",
    );
}

#[test]
fn generic_receivers_use_utf16_and_the_entire_to_length_range() {
    check(
        "Array.prototype.at.call('💩x',-2)==='\\uDCA9' && Array.prototype.at.call(true,0)===undefined && Array.prototype.at.call(7,0)===undefined",
    );
    check("Array.prototype.at.call({0:'a',1:'b',length:'2.9'},-1)==='b'");
    check(
        "let o={0:'first',9007199254740990:'last',9007199254740991:'outside',length:Infinity};Array.prototype.at.call(o,-1)==='last' && Array.prototype.at.call(o,9007199254740990)==='last' && Array.prototype.at.call(o,-9007199254740991)==='first' && Array.prototype.at.call(o,-9007199254740992)===undefined && Array.prototype.at.call(o,9007199254740991)===undefined",
    );
    for length in ["undefined", "null", "NaN", "-Infinity", "-1", "0.9"] {
        check(&format!(
            "Array.prototype.at.call({{0:7,length:{length}}},0)===undefined"
        ));
    }
}

#[test]
fn length_is_converted_once_before_index_and_get_observes_mutations() {
    check(
        "let log='',o={};Object.defineProperty(o,'length',{get:()=>{log+='l';return {valueOf:()=>{log+='n';return 2;}};}});Object.defineProperty(o,'1',{get:()=>{log+='g';return 7;}});let i={valueOf:()=>{log+='i';return -1;}};Array.prototype.at.call(o,i)===7 && log==='lnig'",
    );
    check("let a=[1,2,3],i={valueOf:()=>{a.length=1;a[2]=9;return -1;}};a.at(i)===9");
    check("let a=[1,2],i={valueOf:()=>{a.length=0;return 1;}};a.at(i)===undefined && a.length===0");
    check("let n=0;[].at({valueOf:()=>{n++;return 0;}})===undefined && n===1");
    check(
        "let o={length:1};Object.defineProperty(o,'1',{get:()=>{throw 7;}});Array.prototype.at.call(o,1)===undefined",
    );
}

#[test]
fn coercion_and_get_failures_propagate_in_spec_order() {
    for source in [
        "let o={};Object.defineProperty(o,'length',{get:()=>{throw 7;}});Array.prototype.at.call(o,{valueOf:()=>{throw 8;}})",
        "[].at({valueOf:()=>{throw 7;}})",
        "let a=[1];Object.defineProperty(a,'0',{get:()=>{throw 7;}});a.at(0)",
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Err(Error::Thrown(Value::Number(7.0))),
            "{source}"
        );
    }
    for source in [
        "Array.prototype.at.call(null,0)",
        "Array.prototype.at.call(undefined,0)",
        "[1].at(0n)",
        "Array.prototype.at.call({length:1n},0)",
    ] {
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
fn method_metadata_nonconstructibility_and_roots_are_standard() {
    check(
        "let f=Array.prototype.at,d=Object.getOwnPropertyDescriptor(Array.prototype,'at');f.name==='at' && f.length===1 && f.prototype===undefined && d.value===f && d.writable && !d.enumerable && d.configurable",
    );
    assert!(matches!(
        Realm::default().eval("new Array.prototype.at"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    let mut realm = Realm::default();
    realm
        .eval("let f=Array.prototype.at;delete Array.prototype.at")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("f.call([1,2],-1)"), Ok(Value::Number(2.0)));
}
