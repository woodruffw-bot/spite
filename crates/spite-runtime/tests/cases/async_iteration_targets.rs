//! Awaited ForIn/Of assignment references after each iteration value (14.7.5).

use crate::common::UnavailableHostRealm;
use spite_runtime::{Error, Limits, PromiseJobOutcome, Realm, Value};

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

fn collect(realm: &mut Realm) {
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn iteration_values_precede_awaited_bases_keys_coercion_and_writes_without_getting_the_target() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],values=[],gets=0,result;
        let target={get value(){gets++;throw 1},set value(x){events.push('set'+x);values.push(x)}};
        function base(){events.push('base');return target}function key(){events.push('key');return{[Symbol.toPrimitive](){events.push('coerce');return 'value'}}}
        let source={i:0,[Symbol.iterator](){return this},next(){events.push('next'+(++this.i));return this.i<3?{value:this.i,done:false}:{done:true}}};
        async function f(){for((await base())[await key()] of source){events.push('body'+values[values.length-1])}return 7}f().then(value=>{result=value});
    "#).unwrap();
    check(
        &mut realm,
        "events.join(',')==='next1,base'&&result===undefined",
    );
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='next1,base,key,coerce,set1,body1,next2,base,key,coerce,set2,body2,next3'&&values.join(',')==='1,2'&&gets===0&&result===7",
    );
}

#[test]
fn for_in_retains_the_selected_key_and_keeps_later_enumeration_live() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,gate=new Promise(resolve=>{release=resolve}),object={a:1,b:2,c:3},target={},values=[],result;
        async function f(){for(target[await gate] in object){values.push(target.x);if(target.x==='a')delete object.b}return values.join(',')}f().then(value=>{result=value});
    "#).unwrap();
    check(&mut realm, "values.length===0&&result===undefined");
    realm.eval("delete object.a;release('x')").unwrap();
    drain(&mut realm);
    check(&mut realm, "result==='a,c'&&target.x==='c'");
}

#[test]
fn for_await_keeps_the_raw_async_value_pending_until_target_evaluation_finishes() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,gate=new Promise(resolve=>{release=resolve}),promise=Promise.resolve(7),stored,calls=0,result;
        let source={[Symbol.asyncIterator](){return this},next(){return ++calls===1?{done:false,value:promise}:{done:true}}};
        let target={set x(value){stored=value}};async function f(){for await(target[await gate] of source){await 0}return true}f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "calls===1&&stored===undefined&&result===undefined",
    );
    realm.eval("release('x')").unwrap();
    drain(&mut realm);
    check(&mut realm, "calls===2&&stored===promise&&result===true");
}

#[test]
fn target_rejection_closes_sync_and_async_iterators_but_not_internal_for_in_enumeration() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let reason={},events=[],results=[],target={};function source(name){return{[Symbol.iterator](){return this},[Symbol.asyncIterator](){return this},next(){return{done:false,value:7}},return(){events.push(name);return name==='async'?Promise.resolve({done:true}):{done:true}}}}
        async function sync(){try{for(target[await Promise.reject(reason)] of source('sync')){}}catch(error){return error===reason}}
        async function asyncLoop(){try{for await(target[await Promise.reject(reason)] of source('async')){}}catch(error){return error===reason}}
        async function keys(){try{for(target[await Promise.reject(reason)] in {a:1,return(){events.push('wrong')}}){}}catch(error){return error===reason}}
        sync().then(x=>results.push(x));asyncLoop().then(x=>results.push(x));keys().then(x=>results.push(x));
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results.length===3&&results.every(x=>x===true)&&events.length===2&&events.includes('sync')&&events.includes('async')",
    );
}

#[test]
fn private_and_super_targets_keep_their_receiver_and_empty_inputs_skip_target_awaits() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result,empty,effects=0;class Base{get x(){throw 1}set x(value){this.saved=value}}
        class C extends Base{#x=0;async f(){for((await this).#x of [7]){}for(super[await 'x'] of [9]){}return this.#x+this.saved}}
        async function skipped(){for((await(effects++,null)).x of []){}return effects}new C().f().then(value=>{result=value});skipped().then(value=>{empty=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result===16&&empty===0&&effects===0");
}

#[test]
fn pending_input_is_traced_then_released_before_an_outer_catch_suspends() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let reject,resume,gate=new Promise((resolve,fail)=>{reject=fail}),recovery=new Promise(resolve=>{resume=resolve}),weak,reason={},result;
        function make(){const value={};weak=new WeakRef(value);return value}let values=[make()];
        async function f(){try{for((await gate).x of values){}}catch(error){await recovery;return error===reason}}f().then(value=>{result=value});values[0]=null;
    "#).unwrap();
    collect(&mut realm);
    check(&mut realm, "weak.deref()!==undefined&&result===undefined");
    realm.eval("reject(reason)").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(&mut realm, "weak.deref()===undefined&&result===undefined");
    realm.eval("resume()").unwrap();
    drain(&mut realm);
    check(&mut realm, "result===true");
}

#[test]
fn host_abort_in_an_awaited_target_skips_all_iterator_close_and_javascript_cleanup() {
    for kind in ["of", "in", "await"] {
        for limited in [false, true] {
            let mut realm = Realm::with_unavailable_host_limits(Limits {
                max_steps: limited.then_some(10_000),
                ..Default::default()
            });
            realm.eval("let state={},closed=0,disposed=0,finalized=0,caught=false,proceed,gate=new Promise(resolve=>{proceed=resolve}),holder={};const dispose=Symbol.dispose;").unwrap();
            realm.eval("let resource={[dispose](){disposed++}};state.weak=new WeakRef(resource[dispose]);function spin(){while(true){}};let source={[Symbol.iterator](){return this},[Symbol.asyncIterator](){return this},next(){return{done:false,value:7}},return(){closed++;return{done:true}}}").unwrap();
            let tail = if limited {
                "spin()"
            } else {
                "spiteUnavailableHost()"
            };
            let header = match kind {
                "of" => format!("for(holder[(await gate,{tail})] of source)"),
                "in" => format!("for(holder[(await gate,{tail})] in {{a:1}})"),
                _ => format!("for await(holder[(await gate,{tail})] of source)"),
            };
            realm.eval(&format!("async function f(){{using r=resource;try{{{header}{{}}}}catch{{caught=true}}finally{{finalized++}}}}f().catch(()=>{{caught=true}})")).unwrap();
            drain(&mut realm);
            realm.eval("delete resource[dispose];proceed('x')").unwrap();
            let mut reported = false;
            loop {
                match realm.run_next_promise_job().unwrap() {
                    PromiseJobOutcome::NoJob => break,
                    PromiseJobOutcome::Completed => (),
                    PromiseJobOutcome::Reported(error) => {
                        assert!(
                            matches!(error, Error::Limit { .. }) == limited,
                            "{kind}: {error}"
                        );
                        if !limited {
                            assert!(matches!(error, Error::Unsupported { .. }), "{error}")
                        }
                        reported = true;
                    }
                }
            }
            assert!(reported, "{kind}");
            collect(&mut realm);
            check(
                &mut realm,
                "closed===0&&disposed===0&&finalized===0&&!caught&&state.weak.deref()===undefined",
            );
        }
    }
}
