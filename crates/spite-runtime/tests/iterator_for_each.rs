//! Ordered direct traversal and procedure-error closing (27.1.3.3.7).

mod common;
use common::REALM_ENTRIES;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn invalid_procedure_closes_before_reading_next_and_preserves_its_type_error() {
    for procedure in [
        "undefined",
        "null",
        "true",
        "7",
        "7n",
        "'x'",
        "Symbol()",
        "{}",
    ] {
        for close in ["return 7;", "throw 8;", "return {};"] {
            check(&format!(
                "let closed=0,i={{get next(){{throw 9;}},return(){{closed++;{close}}}}},caught=false;try{{Iterator.prototype.forEach.call(i,{procedure});}}catch(e){{caught=e instanceof TypeError;}}caught && closed===1"
            ));
        }
    }
    check(
        "let closed=0,i={get next(){throw 7;},get return(){closed++;throw 8;}},caught=false;try{Iterator.prototype.forEach.call(i);}catch(e){caught=e instanceof TypeError;}caught && closed===1",
    );
    for receiver in ["undefined", "null", "true", "1", "1n", "'x'", "Symbol()"] {
        assert!(
            matches!(
                Realm::default().eval(&format!(
                    "Iterator.prototype.forEach.call({receiver},()=>0)"
                )),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{receiver}"
        );
    }
}

#[test]
fn traversal_caches_next_preserves_live_mutation_and_passes_two_callback_arguments() {
    check(
        "let gets=0,calls=0,seen=[],nextReceiver,nextArgc,callbackThis,i={};Object.defineProperty(i,Symbol.iterator,{get:()=>{throw 7;}});Object.defineProperty(i,'next',{configurable:true,get:()=>{gets++;return function(){'use strict';nextReceiver=this;nextArgc=arguments.length;return {done:calls===3,value:calls++};};}});let result=Iterator.prototype.forEach.call(i,function(value,index){'use strict';callbackThis=this;seen.push(value,index,arguments.length);Object.defineProperty(i,'next',{value:()=>{throw 8;}});return {valueOf:()=>{throw 9;}};},{toString:()=>{throw 10;}});result===undefined && callbackThis===undefined && gets===1 && nextReceiver===i && nextArgc===0 && seen.join(',')==='0,0,2,1,1,2,2,2,2'",
    );
    check(
        "let receiver,n=0;Iterator.prototype.forEach.call({next:()=>({done:n++>0,value:7})},function(){receiver=this;});receiver===globalThis",
    );
    check(
        "let seen=[];Iterator.concat([1],[],[2]).forEach((value,index)=>seen.push(value,index));seen.join(',')==='1,0,2,1'",
    );
}

#[test]
fn callback_throws_close_once_preserve_identity_and_ignore_close_language_errors() {
    for close in ["return {};", "return 7;", "throw 8;"] {
        check(&format!(
            "let sentinel={{}},closed=0,calls=0,receiver,argc,i={{next:()=>({{done:false,value:7}}),return:function(){{'use strict';closed++;receiver=this;argc=arguments.length;{close}}}}},caught=false;try{{Iterator.prototype.forEach.call(i,()=>{{calls++;throw sentinel;}});}}catch(e){{caught=e===sentinel;}}caught && calls===1 && closed===1 && receiver===i && argc===0"
        ));
    }
    check(
        "let sentinel={},closed=0,i={next:()=>({done:false,value:7}),get return(){closed++;throw 8;}},caught=false;try{Iterator.prototype.forEach.call(i,()=>{throw sentinel;});}catch(e){caught=e===sentinel;}caught && closed===1",
    );
}

#[test]
fn acquisition_and_step_errors_do_not_close_and_done_skips_value() {
    for next in [
        "()=>{throw 7;}",
        "()=>7",
        "()=>({get done(){throw 7;}})",
        "()=>({done:false,get value(){throw 7;}})",
    ] {
        check(&format!(
            "let closed=0,calls=0,i={{next:{next},return(){{closed++;throw 8;}}}},caught=false;try{{Iterator.prototype.forEach.call(i,()=>{{calls++;}});}}catch(e){{caught=e===7 || e instanceof TypeError;}}caught && closed===0 && calls===0"
        ));
    }
    check(
        "let closed=0,i={get next(){throw 7;},return(){closed++;return {};}},caught=false;try{Iterator.prototype.forEach.call(i,()=>0);}catch(e){caught=e===7;}caught && closed===0",
    );
    check(
        "let i={next:()=>({done:true,get value(){throw 7;}}),return:()=>{throw 8;}};Iterator.prototype.forEach.call(i,()=>{throw 9;})===undefined",
    );
}

#[test]
fn metadata_constructibility_and_deleted_link_retention_are_standard() {
    check(
        "let f=Iterator.prototype.forEach,d=Object.getOwnPropertyDescriptor(Iterator.prototype,'forEach');f.name==='forEach' && f.length===1 && Object.getPrototypeOf(f)===Function.prototype && !Object.hasOwn(f,'prototype') && d.value===f && d.writable && !d.enumerable && d.configurable",
    );
    assert!(matches!(
        Realm::default().eval("new Iterator.prototype.forEach(()=>0)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    let mut realm = Realm::default();
    realm
        .eval("let f=Iterator.prototype.forEach;delete Iterator.prototype.forEach;")
        .unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(
        realm.eval("let sum=0;f.call([1,2].values(),value=>{sum+=value;});sum"),
        Ok(Value::Number(3.0))
    );
}

#[test]
fn large_default_iterations_work_and_host_failures_bypass_handlers_and_closing() {
    check(
        "let n=0,sum=0,indices=0;Iterator.prototype.forEach.call({next:()=>({done:n===10000,value:n++})},(value,index)=>{sum+=value;indices+=index;});sum===49995000 && indices===49995000",
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm
        .eval("let flag=0,i={next:()=>({done:false,value:7}),return:()=>{flag=3;return {};}}")
        .unwrap();
    assert!(matches!(
        realm.eval("try{Iterator.prototype.forEach.call(i,()=>0);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::default();
    realm
        .eval("let flag=0,i={next:()=>({done:false,value:7}),return:()=>{flag=3;return {};}}")
        .unwrap();
    assert!(matches!(
        realm.eval(
            "try{Iterator.prototype.forEach.call(i,()=>Function('class C{#field;}'));}catch{flag=1;}finally{flag=2;}"
        ),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
