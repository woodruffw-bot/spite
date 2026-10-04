//! String construction, UTF-16 indexed descriptors, and primitive receivers.

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

fn type_error(source: &str) {
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

#[test]
fn call_and_construction_distinguish_missing_undefined_and_object_arguments() {
    check(
        "String()==='' && String(undefined)==='undefined' && String(null)==='null' && String(true)==='true' && String(-0)==='0' && String(42n)==='42' && String(NaN)==='NaN'",
    );
    check(
        "let a=new String,b=new String(undefined),c=Object('x');a.valueOf()==='' && b.valueOf()==='undefined' && c.valueOf()==='x' && c instanceof String && new String('x')!==c && typeof c==='object'",
    );
    check(
        "let log='',o={toString:()=>{log+='s';return 'x';},valueOf:()=>{throw 1;}};String(o)==='x' && new String(o).valueOf()==='x' && log==='ss'",
    );
    check(
        "let F=String.bind(null,'bound');new F().valueOf()==='bound' && String.call(null,7)==='7' && new String('x').constructor===String",
    );
    assert_eq!(
        Realm::default().eval("String({toString:()=>{throw 9;}})"),
        Err(Error::Thrown(Value::Number(9.0)))
    );
    type_error("String({toString:undefined,valueOf:undefined})");
}

#[test]
fn wrappers_expose_immutable_utf16_indices_length_and_exact_key_order() {
    let mut realm = Realm::default();
    let Value::Object(object) = realm
        .eval("let s=new String('💩x');s.extra=1;s[10]=2;s")
        .unwrap()
    else {
        panic!("wrapper");
    };
    assert_eq!(
        realm.inspect_object(&object).unwrap().own_keys(),
        ["0", "1", "2", "10", "length", "extra"].map(spite_core::PropertyKey::from)
    );
    assert_eq!(
        realm.eval(
            "s.length===3 && s[0]==='\\uD83D' && s[1]==='\\uDCA9' && s[2]==='x' && s[10]===2"
        ),
        Ok(Value::Boolean(true))
    );
    for key in ["0", "1", "2", "length"] {
        assert_eq!(realm.eval(&format!("var d=Object.getOwnPropertyDescriptor(s,'{key}');!d.writable && !d.configurable && d.enumerable==={}",key!="length")),Ok(Value::Boolean(true)));
    }
    check(
        "let s=Object('abc');s[0]='z';s.length=7;s[0]==='a' && s.length===3 && !delete s[0] && !delete s.length && s.hasOwnProperty('1') && !s.hasOwnProperty('-0') && !s.hasOwnProperty('01')",
    );
    check(
        "let s=Object('x');Object.defineProperty(s,'0',{value:'x'});Object.defineProperty(s,'length',{value:1});Object.freeze(s);Object.isFrozen(s) && s.valueOf()==='x'",
    );
    for change in [
        "{value:'z'}",
        "{writable:true}",
        "{enumerable:false}",
        "{configurable:true}",
        "{get:()=> 'x'}",
    ] {
        type_error(&format!("Object.defineProperty(Object('x'),'0',{change})"));
    }
    type_error("'use strict';let s=new String('x');s[0]='z'");
}

#[test]
fn branded_methods_reject_fakes_and_keep_string_data_after_prototype_changes() {
    check(
        "String.prototype.valueOf()==='' && String.prototype.toString()==='' && 'x'.valueOf()==='x' && 'x'.toString()==='x' && 'x'.toLocaleString()==='x'",
    );
    for method in ["toString", "valueOf"] {
        for value in [
            "undefined",
            "null",
            "1",
            "1n",
            "true",
            "{}",
            "Object.create(String.prototype)",
        ] {
            type_error(&format!("String.prototype.{method}.call({value})"));
        }
    }
    check(
        "let s=Object('abc');Object.setPrototypeOf(s,null);String.prototype.valueOf.call(s)==='abc' && Object.prototype.toString.call(s)==='[object String]' && s.length===3",
    );
    check(
        "Object.prototype.toString.call(String.prototype)==='[object String]' && Object.prototype.toString.call(Object.create(String.prototype))==='[object Object]'",
    );
}

#[test]
fn primitive_property_getters_and_setters_preserve_receivers_and_own_index_precedence() {
    check(
        "Object.defineProperty(String.prototype,'x',{get:function(){'use strict';return this;}});'abc'.x==='abc'",
    );
    check(
        "Object.defineProperty(String.prototype,'x',{get:function(){return this;}});let a='abc'.x,b='abc'.x;a instanceof String && a.valueOf()==='abc' && a!==b",
    );
    check(
        "'use strict';let receiver,value;Object.defineProperty(String.prototype,'x',{set:function(v){receiver=this;value=v;}});'abc'.x=7;receiver==='abc' && value===7",
    );
    check(
        "let count=0;Object.defineProperty(String.prototype,'0',{get:()=>{count++;return 'p';},set:()=>{count++;}});let s='abc';s[0]='z';s[0]==='a' && count===0 && ''[0]==='p' && count===1",
    );
    check(
        "function f(){return this;}let s=f.call('abc');s instanceof String && s.length===3 && s[1]==='b' && Object.getPrototypeOf(s)===String.prototype",
    );
    check("function f(){'use strict';return this;}f.call('abc')==='abc' && f.apply('abc')==='abc'");
}

#[test]
fn object_algorithms_on_strings_use_index_descriptors_without_special_substitutions() {
    check(
        "let ds=Object.getOwnPropertyDescriptors('💩');ds[0].value==='\\uD83D' && ds[1].value==='\\uDCA9' && ds.length.value===2 && !ds.length.enumerable && ds[0].enumerable",
    );
    check(
        "let o=Object.assign({},'ab','x');o[0]==='x' && o[1]==='b' && !Object.hasOwn(o,'length') && 'ab'.hasOwnProperty('1') && Object.hasOwn('ab','length')",
    );
    type_error("Object.assign('x',{0:'z'})");
    type_error("Object.defineProperties({},'x')");
    check(
        "Object.getPrototypeOf('abc')===String.prototype && Object.getPrototypeOf(Object.create(null,''))===null",
    );
}

#[test]
fn wrapper_property_capacity_is_a_host_limit_while_primitive_reads_remain_available() {
    let mut realm = Realm::new(Limits {
        max_properties: Some(64),
        ..Limits::default()
    });
    let text = "x".repeat(64);
    realm.eval(&format!("let text='{text}',flag=0")).unwrap();
    assert_eq!(realm.eval("String(text).length"), Ok(Value::Number(64.0)));
    assert_eq!(
        realm.eval("text[63]"),
        Ok(Value::String(JsString::from("x")))
    );
    assert!(matches!(
        realm.eval("try{Object(text);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(
        realm.eval("Object('ok').valueOf()"),
        Ok(Value::String(JsString::from("ok")))
    );
}

#[test]
fn metadata_rooting_and_unimplemented_methods_remain_explicit() {
    check(
        "String.name==='String' && String.length===1 && String.prototype.constructor===String && String.prototype.length===0 && String.prototype.toString.length===0 && String.prototype.valueOf.length===0",
    );
    check(
        "let d=Object.getOwnPropertyDescriptor(String,'prototype');!d.writable && !d.enumerable && !d.configurable",
    );
    type_error("new String.prototype.toString()");
    for source in [
        "String.prototype.normalize",
        "String.prototype.normalize=1",
        "delete String.prototype.normalize",
        "Object.getOwnPropertyDescriptors(String.prototype)",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    let mut realm = Realm::default();
    realm
        .eval("let C=String;let s=new C('abc');delete globalThis.String")
        .unwrap();
    realm.collect(100_000).unwrap();
    assert_eq!(
        realm
            .eval("s.valueOf()==='abc' && s[1]==='b' && C(7)==='7' && Object('x').constructor===C"),
        Ok(Value::Boolean(true))
    );
}
