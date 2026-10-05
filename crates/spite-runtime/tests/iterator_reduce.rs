//! Direct iterator reduction and initial-value presence (27.1.3.3.9).

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
fn initial_value_presence_controls_first_value_and_callback_index() {
    check(
        "let i={next:()=>({done:true,get value(){throw 7;}}),get return(){throw 8;}},caught=false;try{Iterator.prototype.reduce.call(i,()=>{throw 9;});}catch(e){caught=e instanceof TypeError;}caught && Iterator.prototype.reduce.call(i,()=>{throw 10;},undefined)===undefined",
    );
    check(
        "let initial={},i={next:()=>({done:true}),return:()=>{throw 7;}};Iterator.prototype.reduce.call(i,()=>{throw 8;},initial)===initial",
    );
    check(
        "let initial={},n=0,calls=0;let result=Iterator.prototype.reduce.call({next:()=>({done:n++>0,value:initial})},()=>{calls++;throw 7;});result===initial && calls===0",
    );
    for initial in ["", ",10", ",undefined"] {
        let first = if initial.is_empty() { "1" } else { "0" };
        let values = if initial.is_empty() {
            "'1,2'"
        } else {
            "'0,1,2'"
        };
        check(&format!(
            "let seen=[],n=0,argc,receiver;let result=Iterator.prototype.reduce.call({{next:()=>({{done:n===3,value:n++}})}},function(memo,value,index){{'use strict';receiver=this;argc=arguments.length;seen.push(index);return value;}}{initial});result===2 && seen[0]==={first} && seen.join(',')==={values} && receiver===undefined && argc===3"
        ));
    }
    check(
        "Iterator.concat([1,2],[],[3]).reduce((memo,value)=>memo+value)===6 && [1,2].values().reduce((memo,value)=>memo+value,10)===13",
    );
}

#[test]
fn validation_closes_before_next_lookup_and_rejects_primitive_receivers() {
    for reducer in [
        "undefined",
        "null",
        "false",
        "1",
        "1n",
        "'x'",
        "Symbol()",
        "{}",
    ] {
        for close in ["return {};", "return 7;", "throw 8;"] {
            check(&format!(
                "let closed=0,i={{get next(){{throw 9;}},return(){{closed++;{close}}}}},caught=false;try{{Iterator.prototype.reduce.call(i,{reducer},{{valueOf(){{throw 10;}}}});}}catch(e){{caught=e instanceof TypeError;}}caught && closed===1"
            ));
        }
    }
    check(
        "let closed=0,i={get next(){throw 7;},get return(){closed++;throw 8;}},caught=false;try{Iterator.prototype.reduce.call(i);}catch(e){caught=e instanceof TypeError;}caught && closed===1",
    );
    for receiver in ["undefined", "null", "true", "1", "1n", "'x'", "Symbol()"] {
        assert!(
            matches!(
                Realm::default().eval(&format!(
                    "Iterator.prototype.reduce.call({receiver},()=>0,7)"
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
fn cached_next_and_live_values_preserve_accumulator_identity_without_coercion() {
    check(
        "let gets=0,n=0,seen=[],initial={valueOf(){throw 7;}},a={},b={},c={},values=[a,b,c],receiver,argc,i={};Object.defineProperty(i,Symbol.iterator,{get(){throw 8;}});Object.defineProperty(i,'next',{configurable:true,get(){gets++;return function(){'use strict';receiver=this;argc=arguments.length;return {done:n===3,value:values[n++]};};}});let result=Iterator.prototype.reduce.call(i,function(memo,value,index){seen.push(memo,value,index,arguments.length);Object.defineProperty(i,'next',{value:()=>{throw 9;}});return value;},initial,{valueOf(){throw 10;}});gets===1 && receiver===i && argc===0 && result===c && seen[0]===initial && seen[1]===a && seen[2]===0 && seen[3]===3 && seen[4]===a && seen[5]===b && seen[6]===1 && seen[8]===b && seen[9]===c && seen[10]===2",
    );
    check(
        "let receiver;[1,2].values().reduce(function(memo){receiver=this;return memo;});receiver===globalThis",
    );
    check(
        "let original={x:1},replacement={},slot=original,n=0;let result=Iterator.prototype.reduce.call({next:()=>({done:n++===2,value:slot})},memo=>{memo.x=2;slot=replacement;return memo;});result===original && result.x===2 && slot===replacement",
    );
    for value in [
        "undefined",
        "null",
        "true",
        "false",
        "0",
        "-0",
        "NaN",
        "Infinity",
        "'x'",
        "Symbol()",
        "0n",
        "{}",
        "[]",
        "()=>0",
    ] {
        check(&format!(
            "let result={value};Object.is([1,2].values().reduce(()=>result),result)"
        ));
    }
}

#[test]
fn reducer_throws_close_once_and_preserve_the_incoming_error_over_close_errors() {
    for initial in ["", ",0"] {
        for ret in ["return {};", "return 7;", "throw 8;"] {
            check(&format!(
                "let sentinel={{}},closed=0,calls=0,receiver,argc,i={{next:()=>({{done:false,value:7}}),return:function(){{'use strict';closed++;receiver=this;argc=arguments.length;{ret}}}}},caught=false;try{{Iterator.prototype.reduce.call(i,()=>{{calls++;throw sentinel;}}{initial});}}catch(e){{caught=e===sentinel;}}caught && closed===1 && calls===1 && receiver===i && argc===0"
            ));
        }
        check(&format!(
            "let sentinel={{}},closed=0,i={{next:()=>({{done:false,value:7}}),get return(){{closed++;throw 8;}}}},caught=false;try{{Iterator.prototype.reduce.call(i,()=>{{throw sentinel;}}{initial});}}catch(e){{caught=e===sentinel;}}caught && closed===1"
        ));
    }
}

#[test]
fn acquisition_and_both_initial_and_later_step_errors_do_not_close() {
    for initial in ["", ",0"] {
        for next in [
            "()=>{throw 7;}",
            "()=>7",
            "()=>({get done(){throw 7;}})",
            "()=>({done:false,get value(){throw 7;}})",
            "7",
        ] {
            check(&format!(
                "let closed=0,calls=0,i={{next:{next},return(){{closed++;throw 8;}}}},caught=false;try{{Iterator.prototype.reduce.call(i,()=>{{calls++;}}{initial});}}catch(e){{caught=e===7 || e instanceof TypeError;}}caught && closed===0 && calls===0"
            ));
        }
        check(&format!(
            "let closed=0,i={{get next(){{throw 7;}},return(){{closed++;return {{}};}}}},caught=false;try{{Iterator.prototype.reduce.call(i,()=>0{initial});}}catch(e){{caught=e===7;}}caught && closed===0"
        ));
        check(&format!(
            "let n=0,closed=0,i={{next(){{if(n++===0)return {{done:false,value:7}};throw 8;}},return(){{closed++;return {{}};}}}},caught=false;try{{Iterator.prototype.reduce.call(i,memo=>memo{initial});}}catch(e){{caught=e===8;}}caught && closed===0"
        ));
    }
}

#[test]
fn metadata_non_constructibility_and_intrinsic_retention_are_standard() {
    check(
        "let f=Iterator.prototype.reduce,d=Object.getOwnPropertyDescriptor(Iterator.prototype,'reduce');f.name==='reduce' && f.length===1 && Object.getPrototypeOf(f)===Function.prototype && !Object.hasOwn(f,'prototype') && d.value===f && d.writable && !d.enumerable && d.configurable",
    );
    assert!(matches!(
        Realm::default().eval("new Iterator.prototype.reduce(()=>0,7)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    let mut realm = Realm::default();
    realm
        .eval("let f=Iterator.prototype.reduce;delete Iterator.prototype.reduce;")
        .unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(
        realm.eval("f.call([1,2,3].values(),(memo,value)=>memo+value)"),
        Ok(Value::Number(6.0))
    );
}

#[test]
fn large_default_inputs_work_and_host_aborts_skip_handlers_and_cleanup() {
    check(
        "let n=0;Iterator.prototype.reduce.call({next:()=>({done:n===10000,value:n++})},(memo,value)=>memo+value)===49995000",
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm
        .eval("let flag=0,i={next:()=>({done:false,value:7}),return:()=>{flag=3;return {};}};")
        .unwrap();
    assert!(matches!(realm.eval("try{Iterator.prototype.reduce.call(i,(memo,value)=>memo+value,0);}catch{flag=1;}finally{flag=2;}"),Err(Error::Limit{..})));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    for initial in ["", ",0"] {
        let mut realm = Realm::default();
        realm
            .eval("let flag=0,i={next:()=>({done:false,value:7}),return:()=>{flag=3;return {};}};")
            .unwrap();
        assert!(matches!(realm.eval(&format!("try{{Iterator.prototype.reduce.call(i,()=>Function('class C extends Object{{}}'){initial});}}catch{{flag=1;}}finally{{flag=2;}}")),Err(Error::Unsupported{..})));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
}
