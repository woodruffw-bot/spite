//! The ECMA-262 Array locale conversion algorithm without ECMA-402.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

use crate::common::assert_true;

#[test]
fn fixed_locale_separator_nullish_holes_and_nested_arrays() {
    assert_true("[].toLocaleString()==='' && ['',''].toLocaleString()===','");
    assert_true("[1,true,'x',null,undefined,,3].toLocaleString()==='1,true,x,,,,3'");
    assert_true("Array.prototype[1]=9;[1,,3].toLocaleString()==='1,9,3'");
    assert_true("[[1,2],[],[3,[4]]].toLocaleString()==='1,2,,3,4'");
    assert_true("['\\uD800','💩'].toLocaleString()==='\\uD800,💩'");
    assert_true(
        "let a=[1,2];a.join=()=>{throw 7;};a.toString=()=>{throw 8;};a.toLocaleString()==='1,2'",
    );
}

#[test]
fn invokes_current_methods_with_original_receivers_and_no_arguments() {
    assert_true(
        "let o={toLocaleString:function(){'use strict';return this===o && arguments.length===0;}},bad={toString:()=>{throw 7;}};[o,o].toLocaleString(bad,bad,bad)==='true,true'",
    );
    assert_true(
        "let log='',o={};Object.defineProperty(o,'toLocaleString',{get:function(){'use strict';log+=this===o?'g':'x';return function(){'use strict';log+=this===o?'c':'y';return 7;};}});[o].toLocaleString()==='7' && log==='gc'",
    );
    assert_true(
        "'use strict';let log='';Object.defineProperty(Boolean.prototype,'toLocaleString',{get:function(){log+=typeof this;return function(){log+=typeof this;return this;};}});[true].toLocaleString()==='true' && log==='booleanboolean'",
    );
    assert_true(
        "let seen;Boolean.prototype.toLocaleString=function(){seen=this;return this.valueOf();};[true].toLocaleString()==='true' && seen instanceof Boolean",
    );
    assert_true(
        "let n=0,o={toLocaleString:()=>{n++;return 7;}};[null,undefined,,o].toLocaleString()===',,,7' && n===1",
    );
}

#[test]
fn method_results_are_converted_to_strings_before_visiting_the_next_element() {
    assert_true(
        "let log='',a=[{toLocaleString:()=>{log+='c';return {toString:()=>{log+='s';return 7;},valueOf:()=>{throw 7;}};}},2];Object.defineProperty(a,'1',{get:()=>{log+='g';return 2;}});a.toLocaleString()==='7,2' && log==='csg'",
    );
    assert_true(
        "let o={toLocaleString:()=>undefined},n={toLocaleString:()=>null},b={toLocaleString:()=>7n};[o,n,b].toLocaleString()==='undefined,null,7'",
    );
    assert!(matches!(
        Realm::default()
            .eval("[{toLocaleString:()=>({toString:()=>({}),valueOf:()=>({})})}].toLocaleString()"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn captured_length_and_live_reads_preserve_mutation_order() {
    assert_true(
        "let a=[{toLocaleString:()=>{a[1]=9;a[2]=3;return 1;}},2];a.toLocaleString()==='1,9' && a.length===3",
    );
    assert_true(
        "let a=[{toLocaleString:()=>{a.length=0;return 1;}},2,3];a.toLocaleString()==='1,,' && a.length===0",
    );
    assert_true(
        "let a=[{toLocaleString:()=>{delete a[1];return 1;}},2];Array.prototype[1]=9;a.toLocaleString()==='1,9'",
    );
    assert_true(
        "let log='',o={0:1};Object.defineProperty(o,'length',{get:()=>{log+='l';return {valueOf:()=>{log+='n';return 1;}};}});Object.defineProperty(o,'0',{get:()=>{log+='g';return 7;}});Array.prototype.toLocaleString.call(o)==='7' && log==='lng'",
    );
}

#[test]
fn generic_receivers_and_abrupt_operations_follow_the_specification() {
    assert_true("Array.prototype.toLocaleString.call('💩')==='\\uD83D,\\uDCA9'");
    assert_true("Array.prototype.toLocaleString.call({0:'x',1:'y',length:'2.9'})==='x,y'");
    assert_true(
        "Array.prototype.toLocaleString.call(false)==='' && Array.prototype.toLocaleString.call(7)===''",
    );
    for source in [
        "Array.prototype.toLocaleString.call(null)",
        "Array.prototype.toLocaleString.call(undefined)",
        "Array.prototype.toLocaleString.call({length:1n})",
        "[{toLocaleString:null}].toLocaleString()",
        "[{toLocaleString:{}}].toLocaleString()",
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
    for source in [
        "Array.prototype.toLocaleString.call({length:{valueOf:()=>{throw 7;}}})",
        "let a=[1];Object.defineProperty(a,'0',{get:()=>{throw 7;}});a.toLocaleString()",
        "let o={};Object.defineProperty(o,'toLocaleString',{get:()=>{throw 7;}});[o].toLocaleString()",
        "[{toLocaleString:()=>{throw 7;}}].toLocaleString()",
        "[{toLocaleString:()=>({toString:()=>{throw 7;}})}].toLocaleString()",
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Err(Error::Thrown(Value::Number(7.0))),
            "{source}"
        );
    }
    assert_eq!(
        Realm::default().eval("[1n].toLocaleString()"),
        Ok(Value::String("1".into()))
    );
}

#[test]
fn metadata_nonconstructibility_and_collection_preserve_the_method() {
    assert_true(
        "let f=Array.prototype.toLocaleString,d=Object.getOwnPropertyDescriptor(Array.prototype,'toLocaleString');f.name==='toLocaleString' && f.length===0 && f.prototype===undefined && d.value===f && d.writable && !d.enumerable && d.configurable",
    );
    assert!(matches!(
        Realm::default().eval("new Array.prototype.toLocaleString"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    let mut realm = Realm::default();
    realm
        .eval("let f=Array.prototype.toLocaleString;delete Array.prototype.toLocaleString")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("f.call([1,2])==='1,2'"),
        Ok(Value::Boolean(true))
    );
}
