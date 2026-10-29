//! Suspended ChainEvaluation and retained final References (13.3.10).

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
fn optional_nullish_steps_skip_awaited_keys_arguments_and_the_entire_ungrouped_suffix() {
    let mut realm = Realm::with_unavailable_host();
    realm.eval(r#"
        let operand=0,result,n=null,missing;
        async function f(){return[(await n)?.[await(++operand)],n?.method(await(++operand)).x,n?.[spiteUnavailableHost()](await(++operand)),missing?.(await(++operand)),n?.[await missing].x]}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "operand===0&&result.length===5&&result.every(value=>value===undefined)",
    );
}

#[test]
fn optional_awaited_bases_and_keys_convert_and_read_once_before_later_steps() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],result,keyObject={[Symbol.toPrimitive](hint){events.push('coerce:'+hint);return 'x'}},object={get x(){events.push('get');return{y:7}}};
        function base(){events.push('base');return object}function key(){events.push('key');return keyObject}
        async function f(){return(await base())?.[await key()].y}
        f().then(value=>{result=value});
    "#).unwrap();
    check(&mut realm, "events.join(',')==='base'&&result===undefined");
    drain(&mut realm);
    check(
        &mut realm,
        "result===7&&events.join(',')==='base,key,coerce:string,get'",
    );
}

#[test]
fn awaited_optional_member_bases_and_parenthesized_calls_and_tags_preserve_references() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let first,second,third,fourth,object={x:10,m:function(value){'use strict';return this===object?this.x+value:0},tag:function(strings,value){'use strict';return this===object?this.x+value:0}};
        async function f(){return(await object).m?.(await 7)}
        async function g(){return((await object)?.m)(await 8)}
        async function h(){return((await object)?.m)?.(await 9)}
        async function j(){return((await object)?.tag)`x${await 11}`}
        f().then(value=>{first=value});g().then(value=>{second=value});h().then(value=>{third=value});j().then(value=>{fourth=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "first===17&&second===18&&third===19&&fourth===21",
    );
}

#[test]
fn optional_calls_save_the_callee_before_awaited_spread_and_defer_callability_until_arguments() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result,failed,skipped,gets=0,effects=0,receiver,promise=Promise.resolve(7),object={get method(){gets++;return function(...values){receiver=this;return values[0]===promise&&values[1]===3&&values[2]===4}},bad:3};
        async function f(){return object?.method(promise,...await [3,4])}
        async function g(){try{return object?.bad(await(++effects))}catch(error){return error instanceof TypeError}}
        async function h(){let empty=null;return empty?.(await(++effects))}
        f().then(value=>{result=value});g().then(value=>{failed=value});h().then(value=>{skipped=value});
    "#).unwrap();
    check(&mut realm, "gets===1&&effects===1&&result===undefined");
    realm
        .eval("Object.defineProperty(object,'method',{value(){throw 9}})")
        .unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===true&&failed===true&&skipped===undefined&&effects===1&&gets===1&&receiver===object",
    );
}

#[test]
fn deleting_optional_awaited_properties_avoids_getters_and_preserves_nullish_key_timing() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let first,second,third,gets=0,keys=0,coerced=0,badCoerced=0,object={get x(){gets++;throw 9}},key={[Symbol.toPrimitive](){coerced++;return 'x'}},badKey={[Symbol.toPrimitive](){badCoerced++;return 'x'}};
        async function f(){return delete object?.[await key]}
        async function g(){return delete(await null)?.[await(++keys)]}
        async function h(){try{return delete({missing:null})?.missing[await badKey]}catch(error){return error instanceof TypeError}}
        f().then(value=>{first=value});g().then(value=>{second=value});h().then(value=>{third=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "first===true&&second===true&&third===true&&gets===0&&keys===0&&coerced===1&&badCoerced===0&&!Object.hasOwn(object,'x')",
    );
}

#[test]
fn optional_eval_is_indirect_and_private_super_using_and_dynamic_contexts_resume() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let scope='global',evalResult,privateResult,superResult,oldDone=false,usingDone=false,constructors=[],flag=0;
        async function indirect(){let scope='local';return eval?.(await 'scope')}
        class Base{method(value){return this.base+value}}
        class C extends Base{#x=10;#m(value){return this.#x+value}async read(value){return(await this)?.#m?.(await value)}async inherited(value){this.base=20;return super.method?.(await value)}}
        async function old(){return({})?.[await(flag=1)]}
        async function resource(){using value=({})?.[await 'x'];return value===undefined}
        const Constructor=(async function(){}).constructor;
        indirect().then(value=>{evalResult=value});let object=new C();object.read(7).then(value=>{privateResult=value});object.inherited(8).then(value=>{superResult=value});old().then(value=>{oldDone=value===undefined});resource().then(value=>{usingDone=value});Constructor('p','return ({})?.[await p]')('x').then(value=>{constructors.push(value===undefined)});(new Constructor('p','return ({})?.[await p]'))('x').then(value=>{constructors.push(value===undefined)});
    "#).unwrap();
    check(&mut realm, "flag===1&&evalResult===undefined");
    drain(&mut realm);
    check(
        &mut realm,
        "evalResult==='global'&&privateResult===17&&superResult===28&&oldDone&&usingDone&&constructors.length===2&&constructors.every(value=>value)",
    );
}

#[test]
fn optional_keys_and_calls_trace_bases_receivers_and_earlier_arguments_then_release_them() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,releaseSecond,gate=new Promise(resolve=>{release=resolve}),second=new Promise(resolve=>{releaseSecond=resolve}),baseWeak,receiverWeak,argumentWeak,observed,called,first,last;
        function base(){const value={x:7};baseWeak=new WeakRef(value);return value}
        function tag(a,b){called=this.marker===1&&a.x===2&&b==='x';return called}
        function receiver(){const value={marker:1,tag};receiverWeak=new WeakRef(value);return value}
        function earlier(){const value={x:2};argumentWeak=new WeakRef(value);return value}
        async function f(){let value=base()?.[await gate];observed=value;0;await second;return 7}
        async function g(){receiver()?.tag(earlier(),await gate);0;await second;return 8}
        f().then(value=>{first=value});g().then(value=>{last=value});
    "#).unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "baseWeak.deref()!==undefined&&receiverWeak.deref()!==undefined&&argumentWeak.deref()!==undefined&&observed===undefined&&called===undefined",
    );
    realm.eval("release('x')").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "baseWeak.deref()===undefined&&receiverWeak.deref()===undefined&&argumentWeak.deref()===undefined&&observed===7&&called===true&&first===undefined&&last===undefined",
    );
    realm.eval("releaseSecond(0)").unwrap();
    drain(&mut realm);
    check(&mut realm, "first===7&&last===8");
}

#[test]
fn optional_call_argument_limits_and_host_aborts_skip_handlers_and_resource_cleanup() {
    let mut realm = Realm::with_unavailable_host_limits(Limits {
        max_arguments: Some(2),
        ..Default::default()
    });
    realm.eval("let effects=0,called=0,disposed=0,finalized=0,caught=false;const dispose=Symbol.dispose").unwrap();
    realm
        .eval("let resource={[dispose](){disposed++}};function target(a,b,c){called++}")
        .unwrap();
    realm.eval("async function f(){using root=resource;try{return target?.(await(effects++,1),(effects++,2),(effects++,3))}catch{caught=true}finally{finalized++}}f()").unwrap();
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
        "effects===3&&called===0&&disposed===0&&finalized===0&&!caught",
    );
    for limited in [false, true] {
        let mut realm = Realm::with_unavailable_host_limits(Limits {
            max_steps: limited.then_some(10_000),
            ..Default::default()
        });
        realm.eval("let state={},called=0,disposed=0,finalized=0,caught=false;const dispose=Symbol.dispose;").unwrap();
        realm
            .eval("let resource={[dispose](){disposed++}},object={method(){called++}};")
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
        let code="async function f(){using root=resource;try{return object?.method(await 0,TAIL)}finally{finalized++}}".replace("TAIL",tail);
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
