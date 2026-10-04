//! Generic join, dynamic toString dispatch, and observable conversion order.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn join_preserves_holes_nullish_values_utf16_and_separator_rules() {
    check(
        "[].join()==='' && [1,2,3].join()==='1,2,3' && [1,2].join(undefined)==='1,2' && [1,2].join(null)==='1null2'",
    );
    check("[,,].join()===',' && [null,undefined,,3].join('|')==='|||3' && [1,2].join('')==='12'");
    check(
        "['\\uD800','\\uDC00'].join('')==='\\uD800\\uDC00' && ['a','b'].join('\\uD800')==='a\\uD800b'",
    );
    check("[true,false,-0,NaN,Infinity,1n].join()==='true,false,0,NaN,Infinity,1'");
    check("[[1,2],[3],[]].join(';')==='1,2;3;' && String([1,,3])==='1,,3' && +[]===0 && +[2]===2");
    check(
        "Object.defineProperty(Array.prototype,'1',{value:9,writable:true,configurable:true});[1,,3].join()==='1,9,3'",
    );
}

#[test]
fn join_is_generic_and_uses_length_of_array_like() {
    check("Array.prototype.join.call({0:'a',2:'c',length:'3.8'},'|')==='a||c'");
    check(
        "Array.prototype.join.call('💩','|')==='\\uD83D|\\uDCA9' && Array.prototype.join.call(true)==='' && Array.prototype.join.call(3)===''",
    );
    for length in [
        "undefined",
        "null",
        "false",
        "NaN",
        "-Infinity",
        "-3",
        "0.9",
    ] {
        check(&format!(
            "Array.prototype.join.call({{0:'x',length:{length}}})===''"
        ));
    }
    check(
        "let n=0;Array.prototype.join.call({length:0},{toString:()=>{n++;return ',';}})==='' && n===1",
    );
    for value in ["null", "undefined"] {
        assert!(matches!(
            Realm::default().eval(&format!("Array.prototype.join.call({value})")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    assert!(matches!(
        Realm::default().eval("Array.prototype.join.call({length:1n})"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn join_reads_length_then_separator_then_live_elements_in_order() {
    check(
        "let log='',o={0:{toString:()=>{log+='a';return 'A';}},1:{toString:()=>{log+='b';return 'B';}}};Object.defineProperty(o,'length',{get:()=>{log+='l';return {valueOf:()=>{log+='n';return 2;}};}});let s={toString:()=>{log+='s';return '|';}};Array.prototype.join.call(o,s)==='A|B' && log==='lnsab'",
    );
    check("let a=[1,2,3],s={toString:()=>{a.length=1;return '|';}};a.join(s)==='1||'");
    check(
        "let a=[{toString:()=>{a[1]=9;a[3]=7;return 'x';}},2,3];a.join()==='x,9,3' && a.length===4",
    );
    check(
        "let n=0,o={length:2};Object.defineProperty(o,'0',{get:()=>{n++;o[1]='b';return 'a';}});Array.prototype.join.call(o)==='a,b' && n===1",
    );
    check(
        "let log='',o={length:2};Object.defineProperty(o,'0',{get:()=>{log+='0';return null;}});Object.defineProperty(o,'1',{get:()=>{log+='1';return undefined;}});Array.prototype.join.call(o)===',' && log==='01'",
    );
}

#[test]
fn abrupt_length_separator_and_element_operations_stop_immediately() {
    for source in [
        "let o={};Object.defineProperty(o,'length',{get:()=>{throw 7;}});Array.prototype.join.call(o)",
        "[].join({toString:()=>{throw 7;}})",
        "[{toString:()=>{throw 7;}}].join()",
        "let a=[1];Object.defineProperty(a,'0',{get:()=>{throw 7;}});a.join()",
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Err(Error::Thrown(Value::Number(7.0))),
            "{source}"
        );
    }
    let mut realm = Realm::default();
    realm
        .eval("let n=0,a=[{toString:()=>{throw 7;}},{toString:()=>{n++;return 'x';}}]")
        .unwrap();
    assert_eq!(
        realm.eval("a.join()"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    assert_eq!(realm.eval("n"), Ok(Value::Number(0.0)));
}

#[test]
fn to_string_invokes_the_current_join_with_boxed_receiver_and_no_arguments() {
    check("let a=[1,2];a.toString()==='1,2' && a.toString('|')==='1,2'");
    check(
        "let a=[1];a.join=function(){'use strict';return this===a && arguments.length===0;};a.toString()===true",
    );
    check(
        "let n=0,o={};Object.defineProperty(o,'join',{get:()=>{n++;return function(){return this===o?7:0;};}});Array.prototype.toString.call(o)===7 && n===1",
    );
    check(
        "String.prototype.join=function(){'use strict';return typeof this==='object' && this.valueOf()==='x';};Array.prototype.toString.call('x')===true",
    );
    assert_eq!(
        Realm::default()
            .eval("let a=[];Object.defineProperty(a,'join',{get:()=>{throw 7;}});a.toString()"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
}

#[test]
fn to_string_falls_back_to_intrinsic_object_tag_and_accepts_generic_receivers() {
    check(
        "let a=[1];a.join=null;Object.prototype.toString=()=>{throw 7;};a.toString()==='[object Array]'",
    );
    check(
        "Array.prototype.toString.call({join:7})==='[object Object]' && Array.prototype.toString.call('x')==='[object String]' && Array.prototype.toString.call(3)==='[object Number]'",
    );
    check(
        "let a=[];Object.setPrototypeOf(a,null);Array.prototype.toString.call(a)==='[object Array]'",
    );
    for value in ["null", "undefined"] {
        assert!(matches!(
            Realm::default().eval(&format!("Array.prototype.toString.call({value})")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
}

#[test]
fn methods_have_standard_descriptors_and_are_not_constructors() {
    for (name, length) in [("join", 1), ("toString", 0)] {
        check(&format!(
            "let f=Array.prototype.{name},d=Object.getOwnPropertyDescriptor(Array.prototype,'{name}');f.name==='{name}' && f.length==={length} && f.prototype===undefined && d.value===f && d.writable && !d.enumerable && d.configurable"
        ));
        assert!(matches!(
            Realm::default().eval(&format!("new Array.prototype.{name}")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    let mut realm = Realm::default();
    realm.eval("let j=Array.prototype.join,t=Array.prototype.toString;delete Array.prototype.join;delete Array.prototype.toString").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("j.call([1,2])==='1,2' && t.call({join:j,0:'x',length:1})==='x'"),
        Ok(Value::Boolean(true))
    );
}
