//! Promise.any fulfillment, ordered intrinsic AggregateErrors and guarded rejections.

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
fn first_fulfillment_wins_without_waiting_for_pending_inputs() {
    let mut realm = Realm::default();
    realm.eval("let result,cap=Promise.withResolvers();Promise.any([Promise.reject(1),Promise.resolve(2),cap.promise]).then(v=>{result=v},()=>{throw 1});if(result!==undefined)throw 2").unwrap();
    drain(&mut realm);
    assert_eq!(realm.eval("result"), Ok(Value::Number(2.0)));
    realm.eval("cap.reject(3)").unwrap();
    drain(&mut realm);
    assert_eq!(realm.eval("result"), Ok(Value::Number(2.0)));
}

#[test]
fn exhausted_rejections_and_empty_input_create_ordered_branded_aggregate_errors() {
    let mut realm = Realm::default();
    realm.eval("let first=Promise.withResolvers(),second=Promise.withResolvers(),reason={},symbol=Symbol(),result,empty;Promise.any([first.promise,second.promise,Promise.reject(3)]).catch(e=>{result=e});Promise.any([]).catch(e=>{empty=e});second.reject(symbol)").unwrap();
    drain(&mut realm);
    assert_eq!(realm.eval("result===undefined&&empty instanceof AggregateError&&Error.isError(empty)&&empty.errors.length===0"),Ok(Value::Boolean(true)));
    realm.eval("first.reject(reason)").unwrap();
    drain(&mut realm);
    assert_eq!(realm.eval("let d=Object.getOwnPropertyDescriptor(result,'errors');result instanceof AggregateError&&Error.isError(result)&&Object.getPrototypeOf(result)===AggregateError.prototype&&result.errors.length===3&&result.errors[0]===reason&&result.errors[1]===symbol&&result.errors[2]===3&&d.writable&&!d.enumerable&&d.configurable&&!Object.hasOwn(result,'message')&&!Object.hasOwn(result,'cause')"),Ok(Value::Boolean(true)));
}

#[test]
fn custom_capabilities_receive_original_resolve_and_guarded_reject_elements() {
    check(
        r#"
        let callbacks=[],originalResolve,output,rejected=0,result={},sentinel={};
        function C(executor){
            originalResolve=function(){};
            executor(originalResolve,function(e){
                'use strict';
                if(this!==undefined||arguments.length!==1)throw 1;
                rejected++;output=e;
                for(const fn of callbacks)if(fn('ignored')!==undefined)throw 2;
                return sentinel;
            });
            return result;
        }
        C.resolve=function(v){
            'use strict';
            if(this!==C||arguments.length!==1)throw 3;
            return {then(a,b){
                if(arguments.length!==2||a!==originalResolve||b.length!==1||b.name!==''||Object.hasOwn(b,'prototype'))throw 4;
                try{new b();throw 5}catch(e){if(!(e instanceof TypeError))throw e}
                callbacks.push(b);
                if(v===1){b('first');b('duplicate')}
                if(rejected)throw 6;
            }};
        };
        let out=Promise.any.call(C,[1,2]);
        let returned=callbacks[1]('second');
        out===result&&returned===sentinel&&rejected===1&&output instanceof AggregateError&&output.errors.join(',')==='first,second'
    "#,
    );
}

#[test]
fn aggregate_error_and_error_array_allocation_bypass_public_constructors_and_iterators() {
    let mut realm = Realm::default();
    realm.eval("let intrinsic=AggregateError,arrayProto=Array.prototype,result,inputs=[Promise.reject(1),Promise.reject(2)],input={[Symbol.iterator](){let i=0;return{next(){return i<inputs.length?{value:inputs[i++],done:false}:{done:true}}}}};Object.defineProperty(intrinsic.prototype,'errors',{set(){throw 1},configurable:true});arrayProto[Symbol.iterator]=()=>{throw 2};AggregateError=function(){throw 3};Array=function(){throw 4};Promise.any(input).catch(e=>{result=e})").unwrap();
    drain(&mut realm);
    assert_eq!(realm.eval("result instanceof intrinsic&&Error.isError(result)&&Object.getPrototypeOf(result)===intrinsic.prototype&&Object.getPrototypeOf(result.errors)===arrayProto&&result.errors.join(',')==='1,2'"),Ok(Value::Boolean(true)));
}

#[test]
fn pending_reason_lists_and_completed_errors_trace_object_and_symbol_values() {
    let mut realm = Realm::default();
    realm.eval("let state=(()=>{let value={},symbol=Symbol(),cap=Promise.withResolvers();return{any:Promise.any([Promise.reject(value),Promise.reject(symbol),cap.promise]),finish:cap.reject,object:new WeakRef(value),symbol:new WeakRef(symbol)}})()").unwrap();
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
    realm.eval("state.any=null;state.finish=null").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("state.object.deref()===undefined&&state.symbol.deref()===undefined"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn iterator_language_errors_reject_directly_and_host_failures_skip_cleanup() {
    check(
        "let original={},other={},closed=0,caught;function C(executor){executor(()=>{throw 1},e=>{caught=e})}C.resolve=()=>{throw original};Promise.any.call(C,{[Symbol.iterator](){return{next(){return{value:1,done:false}},return(){closed++;throw other}}}});caught===original&&closed===1",
    );
    check(
        "let original={},closed=0,caught;function C(executor){executor(()=>{throw 1},e=>{caught=e})}C.resolve=v=>v;Promise.any.call(C,{[Symbol.iterator](){return{next(){throw original},return(){closed++;return{}}}}});caught===original&&closed===0",
    );
    let mut realm = Realm::default();
    let result=realm.eval("let closed=0,rejected=0,caught=0;function C(executor){executor(()=>{},()=>{rejected++})}C.resolve=()=>new SharedArrayBuffer(0);try{Promise.any.call(C,{[Symbol.iterator](){return{next(){return{value:1,done:false}},return(){closed++;return{}}}}})}catch(e){caught++}finally{caught++}");
    assert!(
        matches!(result, Err(Error::Unsupported { .. })),
        "{result:?}"
    );
    assert_eq!(realm.eval("closed+rejected+caught"), Ok(Value::Number(0.0)));
}
