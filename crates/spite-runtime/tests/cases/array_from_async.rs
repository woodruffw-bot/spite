//! Sequential Array.fromAsync conversion, awaiting, and cleanup (23.1.2.2).

use spite_runtime::{Error, PromiseJobOutcome, Realm, Value};

fn check(realm: &mut Realm, source: &str) {
    assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
}

fn drain(realm: &mut Realm) {
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => (),
            PromiseJobOutcome::Reported(error) => panic!("{error}"),
        }
    }
}

#[test]
fn metadata_and_validation_return_fresh_intrinsic_promises_before_user_lookup() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let f=Array.fromAsync,errors=0,gets=0,items={get [Symbol.asyncIterator](){gets++;throw 1}};
        if(f.name!=='fromAsync'||f.length!==1||Object.hasOwn(f,'prototype')||Object.getPrototypeOf(f)!==Function.prototype)throw 2;
        let a=f(items,7),b=f(items,7);if(a===b||!(a instanceof Promise))throw 3;
        for(const p of[a,b,f(null),f(undefined)])p.catch(e=>{if(e instanceof TypeError)errors++});
        let saved=Promise;Promise=function(){throw 4};let c=f([]);if(Object.getPrototypeOf(c)!==saved.prototype)throw 5;
    "#).unwrap();
    check(&mut realm, "gets===0&&errors===0");
    drain(&mut realm);
    check(&mut realm, "errors===4");
}

#[test]
fn arraylike_values_and_mapping_are_awaited_sequentially_with_captured_length() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,gate=new Promise(r=>{release=r}),log=[],result;
        let source={get length(){log.push('length');return 2},get 0(){log.push('get0');return gate},get 1(){log.push('get1');return Promise.resolve(3)}};
        Array.fromAsync(source,function(value,index){'use strict';if(this!==source||arguments.length!==2)throw 1;log.push('map'+index);return Promise.resolve(value+index)},source).then(a=>{result=a});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "log.join(',')==='length,get0'&&result===undefined",
    );
    realm.eval("source.length=9;release(2)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "log.join(',')==='length,get0,map0,get1,map1'&&Array.isArray(result)&&result.join(',')==='2,4'",
    );
    realm
        .eval("let holes;Array.fromAsync({length:2}).then(a=>{holes=a})")
        .unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "holes.length===2&&Object.hasOwn(holes,'0')&&Object.hasOwn(holes,'1')&&holes[0]===undefined",
    );
}

#[test]
fn async_values_pass_through_while_sync_values_are_unwrapped_and_async_method_wins() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let value=Promise.resolve(7),asyncResult,syncResult,mappedResult;
        function source(async){let n=0;return{[async?Symbol.asyncIterator:Symbol.iterator](){return this},next(){return{value,done:n++>0}}}}
        let asyncSource=source(true);Object.defineProperty(asyncSource,Symbol.iterator,{get(){throw 1}});
        Array.fromAsync(asyncSource).then(a=>{asyncResult=a});
        Array.fromAsync(source(false)).then(a=>{syncResult=a});
        Array.fromAsync(source(true),(v,k)=>{if(v!==value||k!==0)throw 2;return v}).then(a=>{mappedResult=a});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "asyncResult.length===1&&asyncResult[0]===value&&syncResult[0]===7&&mappedResult[0]===7",
    );
}

#[test]
fn iterator_acquisition_precedes_construction_and_next_is_cached() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let log=[],n=0,result,source={get [Symbol.asyncIterator](){log.push('method');return function(){if(this!==source||arguments.length)throw 1;log.push('iterator');return this}},get next(){log.push('next');return function(){if(this!==source||arguments.length)throw 2;log.push('step');return{value:8,done:n++>0}}}};
        function C(){if(arguments.length)throw 3;log.push('construct');Object.defineProperty(source,'next',{value(){throw 4}})}
        Object.defineProperty(C,Symbol.species,{get(){throw 5}});
        Array.fromAsync.call(C,source).then(a=>{result=a});
        if(log.join(',')!=='method,iterator,next,construct,step')throw 6;
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result instanceof C&&result[0]===8&&result.length===1&&log.join(',')==='method,iterator,next,construct,step,step'",
    );
    realm.eval("let other;function D(n){if(arguments.length!==1||n!==2)throw 7}Array.fromAsync.call(D,{0:1,length:2}).then(a=>{other=a})").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "other instanceof D&&other.length===2&&Object.hasOwn(other,'1')",
    );
}

#[test]
fn mapper_failures_await_close_and_original_reason_wins_over_close_rejection() {
    for rejected in [false, true] {
        let mut realm = Realm::default();
        realm.eval(&format!(r#"
            let marker={{}},release,closed=0,rejected=false,gate=new Promise(r=>{{release=r}}),source={{[Symbol.asyncIterator](){{return this}},next(){{return{{value:1,done:false}}}},return(){{if(this!==source||arguments.length)throw 2;closed++;return gate}}}};
            Array.fromAsync(source,()=>{{{mapper}}}).catch(e=>{{rejected=e===marker}});
        "#,mapper=if rejected {"return Promise.reject(marker)"} else {"throw marker"})).unwrap();
        drain(&mut realm);
        check(&mut realm, "closed===1&&!rejected");
        realm
            .eval("release({then(resolve,reject){reject(9)}})")
            .unwrap();
        drain(&mut realm);
        check(&mut realm, "closed===1&&rejected");
    }
    for tail in [
        "get return(){throw 7}",
        "return:7",
        "return(){throw 7}",
        "return(){return 7}",
    ] {
        let mut realm = Realm::default();
        realm.eval(&format!("let marker={{}},ok=false,source={{[Symbol.asyncIterator](){{return this}},next(){{return{{value:1,done:false}}}},{tail}}};Array.fromAsync(source,()=>{{throw marker}}).catch(e=>{{ok=e===marker}})")).unwrap();
        drain(&mut realm);
        check(&mut realm, "ok");
    }
}

#[test]
fn step_errors_do_not_close_and_done_skips_value_lookup() {
    for step in [
        "throw marker",
        "return Promise.reject(marker)",
        "return{get done(){throw marker}}",
        "return{done:false,get value(){throw marker}}",
    ] {
        let mut realm = Realm::default();
        realm.eval(&format!("let marker={{}},closed=0,ok=false,source={{[Symbol.asyncIterator](){{return this}},next(){{{step}}},return(){{closed++;return{{}}}}}};Array.fromAsync(source).catch(e=>{{ok=e===marker}})")).unwrap();
        drain(&mut realm);
        check(&mut realm, "ok&&closed===0");
    }
    let mut realm = Realm::default();
    realm.eval("let ok=false;Array.fromAsync({[Symbol.asyncIterator](){return this},next(){return{done:true,get value(){throw 1}}}}).then(a=>{ok=a.length===0})").unwrap();
    drain(&mut realm);
    check(&mut realm, "ok");
}

#[test]
fn define_failure_closes_but_final_length_failure_does_not() {
    for define in [false, true] {
        let mut realm = Realm::default();
        realm.eval(&format!(r#"
            let n=0,closed=0,ok=false,source={{[Symbol.asyncIterator](){{return this}},next(){{return{{value:1,done:n++>0}}}},return(){{closed++;return{{}}}}}};
            function C(){{Object.defineProperty(this,'{key}',{{value:0,writable:false,configurable:false}})}}
            Array.fromAsync.call(C,source).catch(e=>{{ok=e instanceof TypeError}});
        "#,key=if define {"0"} else {"length"})).unwrap();
        drain(&mut realm);
        check(&mut realm, &format!("ok&&closed==={}", u8::from(define)));
    }
}

#[test]
fn pending_native_state_survives_gc_and_releases_source_after_completion() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},result;
        state.gate=new Promise(r=>{state.release=r});
        (()=>{let source={0:state.gate,length:1};state.weak=new WeakRef(source);Array.fromAsync(source).then(a=>{result=a})})();
    "#).unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "state.weak.deref()!==undefined&&result===undefined",
    );
    realm.eval("state.release(9)").unwrap();
    drain(&mut realm);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(&mut realm, "state.weak.deref()===undefined&&result[0]===9");
}

#[test]
fn host_failures_bypass_rejection_and_iterator_close() {
    let mut realm = Realm::default();
    realm.eval("let caught=false,closed=false,source={[Symbol.asyncIterator](){return this},next(){return{value:1,done:false}},return(){closed=true;return{}}};Array.fromAsync(source,()=>new SharedArrayBuffer(0)).catch(()=>{caught=true})").unwrap();
    let mut reported = false;
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => (),
            PromiseJobOutcome::Reported(Error::Unsupported { .. }) => reported = true,
            PromiseJobOutcome::Reported(error) => panic!("{error}"),
        }
    }
    assert!(reported);
    check(&mut realm, "!caught&&!closed");
}

#[test]
fn intrinsic_await_bypasses_public_then_species_and_replaced_bindings() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let Saved=Promise,then=Promise.prototype.then,wait=Promise.resolve(7),result;
        Object.defineProperty(wait,'then',{get(){throw 1}});
        Object.defineProperty(Promise,Symbol.species,{get(){throw 2}});
        let output=Array.fromAsync({0:wait,length:1},()=>wait);
        Object.defineProperty(output,'constructor',{value:undefined});
        Reflect.apply(then,output,[a=>{result=a}]);Promise=function(){throw 3};
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "result.length===1&&result[0]===7");
}
