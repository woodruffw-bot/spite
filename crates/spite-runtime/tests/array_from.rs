//! Array.from: live reads, iterator protocol, construction, and closing (23.1.2.1).

mod common;

use common::REALM_ENTRIES;
use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn arrays_strings_and_array_likes_produce_dense_results() {
    check(
        "let a=Array.from([,2,,]);a.length===3 && Object.keys(a).join(',')==='0,1,2' && a[0]===undefined && a[1]===2 && a[2]===undefined",
    );
    check(
        "let a=Array.from('A\\uD834\\uDF06\\uD800B');a.length===4 && a[1]==='\\uD834\\uDF06' && a[2]==='\\uD800' && a.join('')==='A\\uD834\\uDF06\\uD800B'",
    );
    check(
        "let o=Object.create({1:7});o.length=3;let a=Array.from(o);a.length===3 && a[0]===undefined && a[1]===7 && Object.keys(a).length===3",
    );
    check(
        "Array.from(7).length===0 && Array.from(true).length===0 && Array.from(Symbol()).length===0 && Array.from(7n).length===0",
    );
    check("let a=Array.from({length:2.9,0:4,1:5,2:6});a.join(',')==='4,5'");
}

#[test]
fn mapping_observes_live_values_and_snapshot_or_iterated_lengths() {
    check(
        "let source={length:3,0:1,1:2,2:3},context={n:7},calls=0;let a=Array.from(source,function(v,i){'use strict';if(this!==context || arguments.length!==2)throw 8;calls++;if(i===0){source[1]=4;delete source[2];source.length=1;}return v===undefined?i:v+this.n;},context);a.join(',')==='8,11,2' && calls===3",
    );
    check(
        "let source=[1,2];let a=Array.from(source,(v,i)=>{if(i===0)source.push(3);return v+i;});a.join(',')==='1,3,5'",
    );
    check(
        "let source=[1,2,3];let a=Array.from(source,(v,i)=>{source.length=1;return v;});a.length===1 && a[0]===1",
    );
    check("function f(){'use strict';return this;}Array.from([1],f,7)[0]===7");
    check("function f(){return this;}Array.from([1],f,true)[0] instanceof Boolean");
}

#[test]
fn mapper_validation_precedes_iterator_lookup_and_nullish_source_errors() {
    check(
        "let gets=0,o={get [Symbol.iterator](){gets++;return null;}};let caught=false;try{Array.from(o,null);}catch(e){caught=e instanceof TypeError;}caught && gets===0",
    );
    for source in ["undefined", "null"] {
        check(&format!(
            "let caught=false;try{{Array.from({source});}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    check("let a=Array.from({length:1,0:7,[Symbol.iterator]:null});a[0]===7 && a.length===1");
    check(
        "let reads=0,o={get length(){reads++;return 1;},[Symbol.iterator]:7};let caught=false;try{Array.from(o);}catch(e){caught=e instanceof TypeError;}caught && reads===0",
    );
}

#[test]
fn construction_uses_zero_or_one_argument_in_specified_order_without_species() {
    check(
        "let log='',o={get [Symbol.iterator](){log+='g';return function(){log+='i';return {next(){log+='n';return {done:true};}};};},get length(){throw 8;},get constructor(){throw 9;}};function C(){log+='c'+arguments.length;Object.defineProperty(this,'length',{set(v){log+='l'+v;}});}let a=Array.from.call(C,o);a instanceof C && log==='gc0inl0'",
    );
    check(
        "let log='',o={get [Symbol.iterator](){log+='g';return undefined;},get length(){log+='l';return {valueOf(){log+='v';return 1;}};},get 0(){log+='e';return 7;}};function C(n){log+='c'+arguments.length+n;}let a=Array.from.call(C,o);a[0]===7 && a.length===1 && log==='glvc11e'",
    );
    check(
        "let calls=0;function C(){calls++;throw 7;}let o={[Symbol.iterator](){calls+=10;return {};}};let caught=false;try{Array.from.call(C,o);}catch(e){caught=e===7;}caught && calls===1",
    );
    check("let a=Array.from.call(()=>{},[1,2]);Array.isArray(a) && a.join(',')==='1,2'");
    check("let a=Array.from.call(null,{length:1,0:7});Array.isArray(a) && a[0]===7");
    check(
        "let got;function C(n){got=n;return Object.freeze({});}let caught=false;try{Array.from.call(C,{length:9007199254740991});}catch(e){caught=e instanceof TypeError;}caught && got===9007199254740991",
    );
    check(
        "let calls=0,o={length:4294967296,get 0(){calls++;return 7;}};let caught=false;try{Array.from(o);}catch(e){caught=e instanceof RangeError;}caught && calls===0",
    );
}

#[test]
fn iterator_next_is_cached_and_done_precedes_value() {
    check(
        "let log='',i={get next(){log+='g';return function(){if(this!==i || arguments.length!==0)throw 8;log+='n';this.next=7;return log==='gn'?{get done(){log+='d';return false;},get value(){log+='v';return 7;}}:{get done(){log+='d';return 1;},get value(){throw 9;}};};}},o={[Symbol.iterator](){if(this!==o || arguments.length!==0)throw 10;return i;}};let a=Array.from(o);a.length===1 && a[0]===7 && log==='gndvnd'",
    );
    for iterator in ["7", "{next:7}", "{next(){return 7;}}"] {
        check(&format!(
            "let caught=false;try{{Array.from({{[Symbol.iterator](){{return {iterator};}}}});}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
}

#[test]
fn mapping_and_definition_failures_close_and_preserve_original_exceptions() {
    for close in ["return undefined;", "return 7;", "throw 8;", "return {}; "] {
        check(&format!(
            "let calls=0,original={{}},i={{next(){{return {{value:1,done:false}};}},return(){{if(this!==i || arguments.length!==0)throw 9;calls++;{close}}}}};let caught=false;try{{Array.from({{[Symbol.iterator](){{return i;}}}},()=>{{throw original;}});}}catch(e){{caught=e===original;}}caught && calls===1"
        ));
    }
    check(
        "let gets=0,i={next(){return {value:1};},get return(){gets++;throw 8;}};let caught=false;try{Array.from({[Symbol.iterator](){return i;}},()=>{throw 7;});}catch(e){caught=e===7;}caught && gets===1",
    );
    check(
        "let i={next(){return {value:1};},return:7};let caught=false;try{Array.from({[Symbol.iterator](){return i;}},()=>{throw 8;});}catch(e){caught=e===8;}caught",
    );
    check(
        "let closed=0,target={};Object.defineProperty(target,'0',{value:7,configurable:false});function C(){return target;}let i={next(){return {value:1};},return(){closed++;throw 8;}};let caught=false;try{Array.from.call(C,{[Symbol.iterator](){return i;}});}catch(e){caught=e instanceof TypeError;}caught && closed===1 && target[0]===7",
    );
}

#[test]
fn iterator_step_failures_and_final_length_failures_do_not_close() {
    for body in [
        "throw 7;",
        "return 7;",
        "return {get done(){throw 7;}};",
        "return {done:false,get value(){throw 7;}};",
    ] {
        check(&format!(
            "let closed=0,i={{next(){{{body}}},return(){{closed++;return {{}};}}}};let caught=false;try{{Array.from({{[Symbol.iterator](){{return i;}}}});}}catch(e){{caught=true;}}caught && closed===0"
        ));
    }
    check(
        "let closed=0;function C(){Object.defineProperty(this,'length',{writable:false,value:0});}let i={next(){return {done:true};},return(){closed++;return {};}};let caught=false;try{Array.from.call(C,{[Symbol.iterator](){return i;}});}catch(e){caught=e instanceof TypeError;}caught && closed===0",
    );
}

#[test]
fn definitions_bypass_setters_preserve_partial_effects_and_can_alias_source() {
    check(
        "let setter=0,target=Object.create({set 0(v){setter++;}});Object.defineProperty(target,'1',{value:7,writable:false,configurable:true});function C(){return target;}let a=Array.from.call(C,{length:2,0:4,1:5});a===target && a[0]===4 && a[1]===5 && a.length===2 && setter===0 && Object.getOwnPropertyDescriptor(a,'1').writable",
    );
    check(
        "let target={},closed=0;Object.defineProperty(target,'1',{value:9,configurable:false});function C(){return target;}let i={next(){return {value:7};},return(){closed++;return {};}};let caught=false;try{Array.from.call(C,{[Symbol.iterator](){return i;}});}catch(e){caught=e instanceof TypeError;}caught && closed===1 && target[0]===7 && target[1]===9 && !Object.hasOwn(target,'length')",
    );
    check(
        "let source=[1,2];function C(){return source;}let a=Array.from.call(C,source,(v,i)=>v+i);a===source && source.join(',')==='1,3'",
    );
}

#[test]
fn standard_function_metadata_and_intrinsic_retention() {
    check(
        "let d=Object.getOwnPropertyDescriptor(Array,'from');Array.from.name==='from' && Array.from.length===1 && d.writable && !d.enumerable && d.configurable && !Object.hasOwn(Array.from,'prototype')",
    );
    check(
        "let called=0,o={get [Symbol.iterator](){called++;throw 7;}};let caught=false;try{new Array.from(o);}catch(e){caught=e instanceof TypeError;}caught && called===0",
    );
    check("delete Array.from;Array.from===undefined && !Object.hasOwn(Array,'from')");
    let mut realm = Realm::default();
    realm
        .eval("let from=Array.from;delete globalThis.Array")
        .unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(
        realm.eval("from('ab').join(',')==='a,b'"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn host_failures_stop_execution_without_running_iterator_cleanup() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let closed=0,flag=0;let source={[Symbol.iterator](){return {next(){return {value:1};},return(){closed++;return {};}};}}").unwrap();
    assert!(matches!(
        realm.eval("try{Array.from(source);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("closed===0 && flag===0"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::default();
    realm.eval("let flag=0").unwrap();
    assert!(matches!(realm.eval("try{Array.from({[Symbol.iterator](){return {next(){return {value:1};},return(){Proxy;}};}},()=>{throw 7;});}catch{flag=1;}finally{flag=2;}"),Err(Error::Unsupported{..})));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
