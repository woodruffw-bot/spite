//! Tagged evaluation with awaited tags/substitutions (13.3.11.1).

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
fn tagged_templates_capture_method_and_receiver_before_substitutions() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],result,receiver,object={get tag(){events.push('get');return function(strings,a,b){events.push('call');receiver=this;return a+b}}};
        function first(){events.push('first');return 7}function second(){events.push('second');return 8}
        async function f(){return object.tag`a${await first()}b${second()}c`}
        f().then(value=>{result=value});
    "#).unwrap();
    check(
        &mut realm,
        "events.join(',')==='get,first'&&result===undefined",
    );
    realm
        .eval("Object.defineProperty(object,'tag',{value(){throw 9}})")
        .unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===15&&receiver===object&&events.join(',')==='get,first,second,call'",
    );
}

#[test]
fn awaited_tag_values_lose_receivers_while_parenthesized_member_tags_preserve_them() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let first,second,object={tag:function(strings,value){'use strict';return this===object?value+1:value+2}};
        async function f(){return((await object).tag)`x${await 7}`}
        async function g(){return(await object.tag)`x${await 7}`}
        f().then(value=>{first=value});g().then(value=>{second=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "first===8&&second===9");
}

#[test]
fn tagged_substitutions_preserve_raw_symbols_objects_and_promises_without_conversion() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result,symbol=Symbol('x'),object={[Symbol.toPrimitive](){throw 9}},promise=Promise.resolve(7);
        function tag(strings,a,b,c,d){return a===symbol&&b===object&&c===promise&&d===8}
        async function f(){return tag`${symbol}${object}${promise}${await 8}`}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result===true");
}

#[test]
fn suspended_tagged_calls_share_frozen_template_sites_and_preserve_invalid_escape_raw_text() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let sites=[],results=[];
        function tag(strings,value){sites.push(strings);return Object.isFrozen(strings)&&Object.isFrozen(strings.raw)&&strings[0]===undefined&&strings.raw[0]==='\\u{110000}'&&strings[1]==='tail'&&value===7}
        async function f(){return tag`\u{110000}${await 7}tail`}
        f().then(value=>{results.push(value)});f().then(value=>{results.push(value)});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results.length===2&&results.every(value=>value)&&sites[0]===sites[1]",
    );
}

#[test]
fn tagged_callability_is_checked_after_every_substitution_and_eval_receives_the_template_object() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let effects=0,result,template,bad=3;
        async function f(){try{return bad`x${await(effects++,7)}y${effects++}z`}catch(error){return error instanceof TypeError}}
        async function g(){return eval`a${await 7}b`}
        f().then(value=>{result=value});g().then(value=>{template=value});
    "#).unwrap();
    check(&mut realm, "effects===1&&result===undefined");
    drain(&mut realm);
    check(
        &mut realm,
        "effects===2&&result===true&&Array.isArray(template)&&Object.isFrozen(template)&&template[0]==='a'&&template[1]==='b'",
    );
}

#[test]
fn private_super_and_object_environment_tags_keep_their_receivers_and_contexts() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let first,second,third,state={},holder={tag(strings,value){return this===holder?value+30:0}};
        class Base{tag(strings,value){return this.x+value}}
        class C extends Base{#x=10;#tag(strings,value){return this.#x+value}async f(){return this.#tag`x${await 7}`}async g(){this.x=20;return super.tag`x${await 8}`}}
        let object=new C();object.f().then(value=>{first=value});object.g().then(value=>{second=value});
        with(holder){state.f=async()=>tag`x${await 9}`}
        state.f().then(value=>{third=value});holder.tag=()=>{throw 9};
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "first===17&&second===28&&third===39");
}

#[test]
fn tagged_calls_trace_receivers_and_earlier_arguments_then_release_them_after_invocation() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,releaseSecond,gate=new Promise(resolve=>{release=resolve}),second=new Promise(resolve=>{releaseSecond=resolve}),receiverWeak,argumentWeak,called,result;
        function tag(strings,a,b){called=this.marker+a.x+b;return called}
        function receiver(){const value={marker:1,tag};receiverWeak=new WeakRef(value);return value}
        function earlier(){const value={x:2};argumentWeak=new WeakRef(value);return value}
        async function f(){receiver().tag`${earlier()}${await gate}`;0;await second;return 7}
        f().then(value=>{result=value});
    "#).unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "receiverWeak.deref()!==undefined&&argumentWeak.deref()!==undefined&&called===undefined&&result===undefined",
    );
    realm.eval("release(3)").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "receiverWeak.deref()===undefined&&argumentWeak.deref()===undefined&&called===6&&result===undefined",
    );
    realm.eval("releaseSecond(0)").unwrap();
    drain(&mut realm);
    check(&mut realm, "result===7");
}

#[test]
fn tagged_argument_quotas_count_the_template_object_and_host_aborts_skip_cleanup() {
    let mut realm = Realm::with_unavailable_host_limits(Limits {
        max_arguments: Some(2),
        ..Default::default()
    });
    realm.eval("let effects=0,called=0,disposed=0,finalized=0,caught=false;const dispose=Symbol.dispose").unwrap();
    realm
        .eval("let resource={[dispose](){disposed++}};function tag(strings,a,b){called++}")
        .unwrap();
    realm.eval("async function f(){using root=resource;try{return tag`x${await(effects++,7)}y${(effects++,8)}z`}catch{caught=true}finally{finalized++}}f()").unwrap();
    let mut reported = false;
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => (),
            PromiseJobOutcome::Reported(error) => {
                assert!(matches!(error, Error::Limit { .. }), "{error}");
                reported = true;
            }
        }
    }
    assert!(reported);
    check(
        &mut realm,
        "effects===2&&called===0&&disposed===0&&finalized===0&&!caught",
    );

    for limited in [false, true] {
        let mut realm = Realm::with_unavailable_host_limits(Limits {
            max_steps: limited.then_some(10_000),
            ..Default::default()
        });
        realm.eval("let state={},called=0,disposed=0,finalized=0,caught=false;const dispose=Symbol.dispose;").unwrap();
        realm
            .eval("let resource={[dispose](){disposed++}};function tag(strings,value){called++}")
            .unwrap();
        realm
            .eval("state.weak=new WeakRef(resource[dispose]);")
            .unwrap();
        realm.eval("function spin(){while(true){}}").unwrap();
        let tail = if limited {
            "spin()"
        } else {
            "spiteUnavailableHost()"
        };
        let code="async function f(){using root=resource;try{return tag`${await 0}${TAIL}`}finally{finalized++}}".replace("TAIL",tail);
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
            "called===0&&disposed===0&&finalized===0&&!caught&&state.weak.deref()===undefined",
        );
    }
}
