//! Ordered Promise.all aggregation, guarded callbacks and traced result lists.

use spite_runtime::{Error, Limits, PromiseJobOutcome, Realm, Value};

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
fn results_follow_input_order_and_wait_for_all_values_including_empty_input() {
    let mut realm = Realm::default();
    realm.eval("let first=Promise.withResolvers(),second=Promise.withResolvers(),seen=[];Promise.all([first.promise,second.promise,3]).then(a=>{if(!Array.isArray(a)||a.length!==3)throw 1;seen.push(a.join(','))});Promise.all([]).then(a=>{if(a.length!==0)throw 2;seen.push('empty')});second.resolve(2);if(seen.length)throw 3").unwrap();
    drain(&mut realm);
    assert_eq!(
        realm.eval("seen.join(';')"),
        Ok(Value::String("empty".into()))
    );
    realm.eval("first.resolve(1)").unwrap();
    drain(&mut realm);
    assert_eq!(
        realm.eval("seen.join(';')"),
        Ok(Value::String("empty;1,2,3".into()))
    );
    realm.eval("let marker={},caught=false;Promise.all([Promise.reject(marker),new Promise(()=>{})]).catch(e=>{caught=e===marker})").unwrap();
    drain(&mut realm);
    assert_eq!(realm.eval("caught"), Ok(Value::Boolean(true)));
    realm.eval("let ArrayIntrinsic=Array,result;Array=function(){throw 4};Object.defineProperty(ArrayIntrinsic.prototype,'0',{set(){throw 5},get(){return 6},configurable:true});Promise.all([1,2]).then(a=>{result=a})").unwrap();
    drain(&mut realm);
    assert_eq!(realm.eval("ArrayIntrinsic.isArray(result)&&Object.getPrototypeOf(result)===ArrayIntrinsic.prototype&&result.join(',')==='1,2'"),Ok(Value::Boolean(true)));
}

#[test]
fn generic_resolve_elements_are_guarded_and_delay_resolution_until_iteration_finishes() {
    check(
        r#"
        let log='',callbacks=[],result={},sentinel={},resolved=0,output;
        function C(executor) {
            log+='c';
            executor(function(a) {
                'use strict';
                if(this!==undefined||arguments.length!==1)throw 1;
                resolved++;output=a;log+='f';
                for(const callback of callbacks)if(callback(99)!==undefined)throw 2;
                return sentinel;
            },()=>{throw 3});
            return result;
        }
        Object.defineProperty(C,'resolve',{get(){
            log+='g';
            return function(v){
                'use strict';
                if(this!==C||arguments.length!==1)throw 4;
                log+='r'+v;
                return {then(callback,reject){
                    if(arguments.length!==2||callback.length!==1||callback.name!==''||Object.hasOwn(callback,'prototype'))throw 5;
                    try{new callback();throw 6}catch(e){if(!(e instanceof TypeError))throw e}
                    callbacks.push(callback);
                    if(v!==3){callback(v);callback(99)}
                    if(resolved!==0)throw 7;
                }};
            };
        }});
        let out=Promise.all.call(C,[1,2,3]);
        let returned=callbacks[2](3);
        out===result&&returned===sentinel&&resolved===1&&output.join(',')==='1,2,3'&&log==='cgr1r2r3f'
    "#,
    );
}

#[test]
fn close_and_rejection_rules_distinguish_body_step_and_exhausted_resolve_failures() {
    check(
        "let original={},other={},closed=0,caught;function C(executor){executor(()=>{},e=>{caught=e})}C.resolve=()=>{throw original};Promise.all.call(C,{[Symbol.iterator](){return{next(){return{value:1,done:false}},return(){closed++;throw other}}}});caught===original&&closed===1",
    );
    check(
        "let original={},closed=0,caught;function C(executor){executor(()=>{},e=>{caught=e})}C.resolve=v=>v;Promise.all.call(C,{[Symbol.iterator](){return{next(){return{done:false,get value(){throw original}}},return(){closed++;return{}}}}});caught===original&&closed===0",
    );
    check(
        "let original={},closed=0,caught;function C(executor){executor(()=>{throw original},e=>{caught=e})}C.resolve=v=>v;Promise.all.call(C,{[Symbol.iterator](){return{next(){return{done:true}},return(){closed++;return{}}}}});caught===original&&closed===0",
    );
}

#[test]
fn pending_aggregate_values_and_symbols_survive_collection_and_release_after_completion() {
    let mut realm = Realm::default();
    realm.eval("let state=(()=>{let value={},symbol=Symbol(),cap=Promise.withResolvers();return{all:Promise.all([Promise.resolve(value),Promise.resolve(symbol),cap.promise]),finish:cap.resolve,object:new WeakRef(value),symbol:new WeakRef(symbol)}})()").unwrap();
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
fn host_failures_skip_rejection_closing_and_language_handlers() {
    let mut realm = Realm::default();
    let result=realm.eval("let closed=0,rejected=0,caught=0;function C(executor){executor(()=>{},()=>{rejected++})}C.resolve=()=>new SharedArrayBuffer(0);try{Promise.all.call(C,{[Symbol.iterator](){return{next(){return{value:1,done:false}},return(){closed++;return{}}}}})}catch(e){caught++}finally{caught++}");
    assert!(
        matches!(result, Err(Error::Unsupported { .. })),
        "{result:?}"
    );
    assert_eq!(realm.eval("closed+rejected+caught"), Ok(Value::Number(0.0)));
    let mut limited = Realm::new(Limits {
        max_steps: Some(100_000),
        ..Limits::default()
    });
    let result=limited.eval("let closed=0,rejected=0;function C(executor){executor(()=>{},()=>{rejected++})}C.resolve=()=>{while(true){}};Promise.all.call(C,{[Symbol.iterator](){return{next(){return{value:1,done:false}},return(){closed++;return{}}}}})");
    assert!(matches!(result, Err(Error::Limit { .. })), "{result:?}");
    assert_eq!(limited.eval("closed+rejected"), Ok(Value::Number(0.0)));
}
