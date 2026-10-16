//! Identifier rest parameter arrays and FunctionDeclarationInstantiation (15.2.3, 10.2.11).

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn remaining_arguments_form_fresh_dense_intrinsic_arrays() {
    check(
        "function f(a,...rest){return rest;}let a=f(1,undefined,3),b=f(1);Array.isArray(a) && a.length===2 && Object.hasOwn(a,'0') && a[0]===undefined && a[1]===3 && Array.isArray(b) && b.length===0 && a!==b && f(1)!==b && Object.getPrototypeOf(a)===Array.prototype",
    );
    check(
        "let marker={},f=(...r)=>r,a=f(marker,null,undefined);a.length===3 && a[0]===marker && a[1]===null && a[2]===undefined && Object.keys(a).join(',')==='0,1,2'",
    );
    check(
        "let f=(a,b,...r)=>r;f().length===0 && f(1).length===0 && f(1,2).length===0 && f(1,2,3)[0]===3",
    );
}

#[test]
fn defaults_initialize_before_rest_and_rest_remains_in_tdz_during_defaults() {
    check(
        "let calls=0;function f(a=(calls++,1),...r){return [a,r];}let a=f(undefined,2,3),b=f(7);calls===1 && a[0]===1 && a[1].join(',')==='2,3' && b[0]===7 && b[1].length===0",
    );
    check(
        "let caught=false;try{((a=r,...r)=>r)();}catch(e){caught=e instanceof ReferenceError;}caught",
    );
    check(
        "function f(a=()=>r,...r){return a;}let f1=f(undefined,1,2),f2=f(undefined,3);f1().join(',')==='1,2' && f2().join(',')==='3'",
    );
    check("function f(a=()=>r,...r){var r=7;return a;}f(undefined,1)()[0]===1");
}

#[test]
fn rest_lists_use_unmapped_arguments_in_non_strict_and_strict_functions() {
    check(
        "function f(a,...r){a=8;r[0]=9;return arguments[0]===1 && arguments[1]===2 && arguments.length===2;}f(1,2)",
    );
    check("function f(a,...r){arguments[0]=8;arguments[1]=9;return a===1 && r[0]===2;}f(1,2)");
    check(
        "'use strict';function f(a,...r){a=8;r[0]=9;return arguments[0]===1 && arguments[1]===2;}f(1,2)",
    );
    check(
        "function f(...r){let caught=false;try{arguments.callee;}catch(e){caught=e instanceof TypeError;}return caught && Object.getOwnPropertyDescriptor(arguments,'callee').get!==undefined;}f(1)",
    );
    check(
        "function f(...arguments){return Array.isArray(arguments) && arguments.join(',')==='1,2';}f(1,2)",
    );
}

#[test]
fn rest_without_parameter_expressions_shares_body_vars_and_suppresses_arguments() {
    check("function f(...r){var r;return r;}f(1,2).join(',')==='1,2'");
    check("function f(...r){var r=7;return r;}f(1)===7");
    check("function f(...r){function arguments(){return 7;}return arguments();}f(1)===7");
    check("function f(...r){let arguments=8;return arguments;}f(1)===8");
    check(
        "function f(a=()=>arguments,...r){function arguments(){return 7;}return a;}f(undefined,2)()[1]===2",
    );
    check("function f(...r){function r(){return 7;}return r();}f(1)===7");
}

#[test]
fn rest_in_arrows_methods_constructors_and_bound_functions() {
    check("function outer(){let f=(...r)=>arguments[0]+r[0];return f(2);}outer(3)===5");
    check(
        "let obj={x:7,m(a,...r){return this.x+a+r[0];}};obj.m(1,2)===10 && !Object.hasOwn(obj.m,'prototype')",
    );
    check(
        "function F(x,...r){this.x=x;this.r=r;this.target=new.target;}let Bound=F.bind(null,1,2),a=new Bound(3);a.x===1 && a.r.join(',')==='2,3' && a.target===F && a instanceof F",
    );
    check(
        "let f=(a,...r)=>r.join(',');f.bind(null,1,2)(3)==='2,3' && f.call(null,1,2,3)==='2,3' && f.apply(null,[1,2,3])==='2,3'",
    );
}

#[test]
fn length_source_and_property_attributes_include_rest_semantics() {
    check(
        "let a=(...r)=>r,b=(x,...r)=>r,c=(x,y=1,...r)=>r,d=(x=1,...r)=>r;function f(x,y,...r){}let o={m(x,...r){}};a.length===0 && b.length===1 && c.length===1 && d.length===0 && f.length===2 && o.m.length===1 && f.bind(null,1).length===1",
    );
    check(
        "let f = (a, /*rest*/ ...r) => r;f.toString()==='(a, /*rest*/ ...r) => r' && f.name==='f' && !Object.getOwnPropertyDescriptor(f,'length').writable && Object.getOwnPropertyDescriptor(f,'length').configurable",
    );
    check(
        "function f(...r){return r;}let a=f(7),d=Object.getOwnPropertyDescriptor(a,'0'),l=Object.getOwnPropertyDescriptor(a,'length');d.value===7 && d.writable && d.enumerable && d.configurable && l.value===1 && l.writable && !l.enumerable && !l.configurable",
    );
}

#[test]
fn rest_array_creation_bypasses_setters_species_and_replaced_globals() {
    check(
        "let calls=0;Object.defineProperty(Array.prototype,'0',{set(v){calls++;},configurable:true});let source=[1,2];Object.defineProperty(source,'constructor',{get(){throw 7;}});let a=((...r)=>r)(...source);a.length===2 && a[0]===1 && calls===0 && Object.hasOwn(a,'0')",
    );
    check(
        "let proto=Array.prototype;Array=()=>{throw 7;};let a=((...r)=>r)(1);Object.getPrototypeOf(a)===proto && a[0]===1",
    );
    check(
        "let calls=0;Array.prototype[Symbol.iterator]=function(){calls++;throw 7;};let a=((...r)=>r)(1,2);a.length===2 && a[1]===2 && calls===0",
    );
}

#[test]
fn rest_arrays_and_defaults_survive_explicit_collection() {
    let mut realm = Realm::default();
    realm
        .eval("let marker={},f;function build(a=()=>r,...r){return a;}f=build(undefined,marker)")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("f()[0]===marker"), Ok(Value::Boolean(true)));
}

#[test]
fn default_failures_skip_body_and_restore_caller_scopes() {
    check(
        "let marker={},calls=0,flag=0;function f(a=(()=>{calls++;throw marker;})(),...r){flag++;}let caught=false;try{f(undefined,1);}catch(e){caught=e===marker;}caught && calls===1 && flag===0 && typeof r==='undefined'",
    );
}

#[test]
fn opted_in_rest_property_limits_stop_before_body_without_language_cleanup() {
    let mut realm = Realm::new(Limits {
        max_properties: Some(100),
        ..Limits::default()
    });
    realm.eval("let flag=0,calls=0,source={[Symbol.iterator](){let n=0;return {next(){return {done:n===150,value:n++};},return(){calls++;return {};}}}};let f=(...r)=>(flag=3)").unwrap();
    assert!(matches!(
        realm.eval("try{f(...source);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag+calls"), Ok(Value::Number(0.0)));
}
