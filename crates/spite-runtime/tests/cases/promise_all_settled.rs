//! Ordered settlement snapshots, shared callback guards and intrinsic allocation.

use crate::common::UnavailableHostRealm;
use spite_runtime::{Error, PromiseJobOutcome, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

fn drain(realm: &mut Realm) {
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => {}
            PromiseJobOutcome::Reported(error) => panic!("{error}"),
        }
    }
}

#[test]
fn mixed_snapshots_keep_input_order_descriptors_and_original_values() {
    let mut realm = Realm::default();
    realm.eval("let cap=Promise.withResolvers(),value={},reason=Symbol(),result,empty=false;Promise.allSettled([cap.promise,Promise.reject(reason),3]).then(v=>{result=v});Promise.allSettled([]).then(a=>{empty=Array.isArray(a)&&a.length===0});if(empty||result!==undefined)throw 1").unwrap();
    drain(&mut realm);
    assert_eq!(
        realm.eval("empty&&result===undefined"),
        Ok(Value::Boolean(true))
    );
    realm.eval("cap.resolve(value)").unwrap();
    drain(&mut realm);
    assert_eq!(realm.eval("result.length===3&&result[0].status==='fulfilled'&&result[0].value===value&&result[1].status==='rejected'&&result[1].reason===reason&&result[2].value===3&&Reflect.ownKeys(result[0]).join(',')==='status,value'&&Reflect.ownKeys(result[1]).join(',')==='status,reason'&&result.every(o=>Object.getPrototypeOf(o)===Object.prototype&&Reflect.ownKeys(o).every(k=>{let d=Object.getOwnPropertyDescriptor(o,k);return d.writable&&d.enumerable&&d.configurable}))"),Ok(Value::Boolean(true)));
}

#[test]
fn fulfill_and_reject_callbacks_share_one_guard_and_preserve_custom_resolve_returns() {
    check(
        r#"
        let pairs=[],resolved=0,output,result={},sentinel={};
        function C(executor){
            executor(function(a){
                'use strict';
                if(this!==undefined||arguments.length!==1)throw 1;
                resolved++;output=a;
                for(const pair of pairs)for(const fn of pair)if(fn('ignored')!==undefined)throw 2;
                return sentinel;
            },()=>{throw 3});
            return result;
        }
        C.resolve=function(v){
            'use strict';
            if(this!==C||arguments.length!==1)throw 4;
            return {then(a,b){
                if(arguments.length!==2||a===b)throw 5;
                for(const fn of [a,b]){
                    if(fn.name!==''||fn.length!==1||Object.hasOwn(fn,'prototype'))throw 6;
                    try{new fn();throw 7}catch(e){if(!(e instanceof TypeError))throw e}
                }
                pairs.push([a,b]);
                if(v===1){a('yes');b('ignored')}
                if(resolved)throw 8;
            }};
        };
        let out=Promise.allSettled.call(C,[1,2]);
        let returned=pairs[1][1]('no');
        out===result&&returned===sentinel&&resolved===1&&output[0].status==='fulfilled'&&output[0].value==='yes'&&output[1].status==='rejected'&&output[1].reason==='no'
    "#,
    );
}

#[test]
fn snapshot_and_array_creation_bypass_replaced_globals_and_inherited_setters() {
    let mut realm = Realm::default();
    realm.eval("let objectProto=Object.prototype,arrayProto=Array.prototype,reason={},result;Object.defineProperty(objectProto,'status',{__proto__:null,set(){throw 1},configurable:true});Object.defineProperty(objectProto,'value',{__proto__:null,set(){throw 2},configurable:true});Object.defineProperty(objectProto,'reason',{__proto__:null,set(){throw 3},configurable:true});Object.defineProperty(arrayProto,'0',{__proto__:null,set(){throw 4},configurable:true});Object=function(){throw 5};Array=function(){throw 6};Promise.allSettled([1,Promise.reject(reason)]).then(a=>{result=a})").unwrap();
    drain(&mut realm);
    assert_eq!(realm.eval("Reflect.getPrototypeOf(result)===arrayProto&&Reflect.getPrototypeOf(result[0])===objectProto&&Reflect.getPrototypeOf(result[1])===objectProto&&result[0].value===1&&result[1].reason===reason"),Ok(Value::Boolean(true)));
}

#[test]
fn snapshot_objects_retain_values_and_symbol_reasons_until_results_are_released() {
    let mut realm = Realm::default();
    realm.eval("let state=(()=>{let value={},symbol=Symbol(),cap=Promise.withResolvers();return{all:Promise.allSettled([Promise.resolve(value),Promise.reject(symbol),cap.promise]),finish:cap.resolve,object:new WeakRef(value),symbol:new WeakRef(symbol)}})()").unwrap();
    drain(&mut realm);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("state.object.deref()!==undefined&&state.symbol.deref()!==undefined"),
        Ok(Value::Boolean(true))
    );
    realm.eval("state.finish(3)").unwrap();
    drain(&mut realm);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("state.object.deref()!==undefined&&state.symbol.deref()!==undefined"),
        Ok(Value::Boolean(true))
    );
    realm.eval("state.all=null;state.finish=null").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("state.object.deref()===undefined&&state.symbol.deref()===undefined"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn iteration_failures_reject_without_snapshots_and_host_failures_bypass_cleanup() {
    check(
        "let original={},other={},closed=0,caught;function C(executor){executor(()=>{throw 1},e=>{caught=e})}C.resolve=()=>{throw original};Promise.allSettled.call(C,{[Symbol.iterator](){return{next(){return{value:1,done:false}},return(){closed++;throw other}}}});caught===original&&closed===1",
    );
    check(
        "let original={},closed=0,caught;function C(executor){executor(()=>{throw 1},e=>{caught=e})}C.resolve=v=>v;Promise.allSettled.call(C,{[Symbol.iterator](){return{next(){throw original},return(){closed++;return{}}}}});caught===original&&closed===0",
    );
    let mut realm = Realm::with_unavailable_host();
    let result=realm.eval("let closed=0,rejected=0,caught=0;function C(executor){executor(()=>{},()=>{rejected++})}C.resolve=()=>spiteUnavailableHost();try{Promise.allSettled.call(C,{[Symbol.iterator](){return{next(){return{value:1,done:false}},return(){closed++;return{}}}}})}catch(e){caught++}finally{caught++}");
    assert!(
        matches!(result, Err(Error::Unsupported { .. })),
        "{result:?}"
    );
    assert_eq!(realm.eval("closed+rejected+caught"), Ok(Value::Number(0.0)));
}
