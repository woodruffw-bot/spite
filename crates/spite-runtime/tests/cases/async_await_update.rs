//! Awaited update targets share ordinary ToNumeric and PutValue (13.4.2–5).

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
fn awaited_prefix_and_postfix_targets_return_numeric_values_and_preserve_bigint() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result,object={x:'4',big:1n};
        async function f(){let old=(await object)[await 'x']++,next=++(await object).x,big=(await object).big++,small=--(await object)[await 'big'];return old===4&&next===6&&big===1n&&small===1n&&object.x===6&&object.big===1n}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result===true");
}

#[test]
fn awaited_update_caches_the_key_before_numeric_conversion_and_writes_with_the_receiver() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],result,saved,receiver,key={[Symbol.toPrimitive](){events.push('key');return 'x'}};
        let left={valueOf(){events.push('number');key[Symbol.toPrimitive]=()=>{throw 9};return 20}},object={get x(){events.push('get');return left},set x(value){events.push('set');saved=value;receiver=this}};
        async function f(){return(await object)[await key]++}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===20&&saved===21&&receiver===object&&events.join(',')==='key,get,number,set'",
    );
}

#[test]
fn private_updates_with_awaited_bases_and_ordinary_super_updates_keep_brand_and_receiver_rules() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result,bad,rhs=0;class Base{get x(){return 3}set x(value){this.saved=value}}
        class C extends Base{#x=1;async f(){let old=(await this).#x++;await 0;let next=++super.x;return old+this.#x+next+this.saved}async bad(){try{++(await {}).#x}catch(error){return error instanceof TypeError}}}
        let object=new C();object.f().then(value=>{result=value});object.bad().then(value=>{bad=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result===11&&object.saved===4&&bad===true");
}

#[test]
fn awaited_updates_share_strict_readonly_and_nonnumeric_failures_without_extra_writes() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let results=[],gets=0,sets=0,coerced=0,frozen={},bad={get x(){gets++;return Symbol('x')},set x(value){sets++}};
        Object.defineProperty(frozen,'x',{value:{valueOf(){coerced++;return 5}},writable:false});
        async function a(){'use strict';try{(await frozen).x++}catch(error){return error instanceof TypeError}}
        async function b(){return(await frozen).x++===5}
        async function c(){try{++(await bad)[await 'x']}catch(error){return error instanceof TypeError}}
        a().then(value=>{results.push(value)});b().then(value=>{results.push(value)});c().then(value=>{results.push(value)});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results.length===3&&results.every(value=>value)&&coerced===2&&gets===1&&sets===0&&typeof frozen.x==='object'",
    );
}

#[test]
fn awaited_updates_work_in_for_headers_and_preserve_parenthesized_references() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result,state={i:0};
        async function f(){let sum=0;for(;await(state.i<3);(await state).i++){sum+=state.i;await 0}let previous=((await state)[await 'i'])--;let next=--((await state).i);return sum===3&&previous===3&&next===1&&state.i===1}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result===true");
}

#[test]
fn awaited_key_update_retains_its_base_until_write_then_releases_it() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,releaseSecond,gate=new Promise(resolve=>{release=resolve}),second=new Promise(resolve=>{releaseSecond=resolve}),weak,result,old;
        function make(){const object={x:1};weak=new WeakRef(object);return object}
        async function f(){old=make()[await gate]++;0;await second;return 7}
        f().then(value=>{result=value});
    "#).unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "weak.deref()!==undefined&&old===undefined&&result===undefined",
    );
    realm.eval("release('x')").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "weak.deref()===undefined&&old===1&&result===undefined",
    );
    realm.eval("releaseSecond(0)").unwrap();
    drain(&mut realm);
    check(&mut realm, "result===7");
}

#[test]
fn host_abort_during_awaited_key_update_skips_get_put_handlers_and_disposal() {
    for limited in [false, true] {
        let mut realm = Realm::with_unavailable_host_limits(Limits {
            max_steps: limited.then_some(10_000),
            ..Default::default()
        });
        realm.eval("let state={},gets=0,sets=0,disposed=0,finalized=0,caught=false;const dispose=Symbol.dispose;").unwrap();
        realm.eval("let object={get x(){gets++;return 1},set x(value){sets++}},resource={[dispose](){disposed++}};").unwrap();
        realm
            .eval("state.weak=new WeakRef(resource[dispose]);")
            .unwrap();
        realm.eval("function spin(){while(true){}}").unwrap();
        let tail = if limited {
            "spin()"
        } else {
            "spiteUnavailableHost()"
        };
        let code="async function f(){using root=resource;try{object[(await 0,TAIL)]++}finally{finalized++}}".replace("TAIL",tail);
        realm.eval(&code).unwrap();
        realm.eval("f().catch(()=>{caught=true});").unwrap();
        realm.eval("delete resource[dispose]").unwrap();
        let mut reported = false;
        loop {
            match realm.run_next_promise_job().unwrap() {
                PromiseJobOutcome::NoJob => break,
                PromiseJobOutcome::Completed => (),
                PromiseJobOutcome::Reported(error) => {
                    assert!(matches!(error, Error::Limit { .. }) == limited, "{error}");
                    if !limited {
                        assert!(matches!(error, Error::Unsupported { .. }), "{error}")
                    }
                    reported = true;
                }
            }
        }
        assert!(reported);
        collect(&mut realm);
        check(
            &mut realm,
            "gets===0&&sets===0&&disposed===0&&finalized===0&&!caught&&state.weak.deref()===undefined",
        );
    }
}
