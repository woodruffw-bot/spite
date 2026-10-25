//! Awaited call targets/arguments retain lookup, receiver and invocation order.

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

#[test]
fn callee_lookup_and_arguments_run_once_in_order_and_method_replacement_does_not_change_the_target()
{
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},events=[],result,obj={x:7,get method(){events.push('get');return function(a,b,c){'use strict';if(this!==obj||arguments.length!==3)throw 1;events.push('call');return a+b+c}}};
        state.gate=new Promise(resolve=>{state.release=resolve});
        async function f(){return obj.method((events.push('first'),1),await state.gate,(events.push('last'),3))}
        f().then(value=>{result=value});Object.defineProperty(obj,'method',{value(){throw 2}});
    "#).unwrap();
    check(
        &mut realm,
        "events.join(',')==='get,first'&&result===undefined",
    );
    realm.eval("state.release(2)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===6&&events.join(',')==='get,first,last,call'",
    );
}

#[test]
fn member_references_keep_receivers_while_comma_and_awaited_callee_values_do_not() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let state={},result,obj={method:function(value){'use strict';return[this,value]}};
        state.gate=new Promise(resolve=>{state.release=resolve});
        async function a(){return (obj.method)(await state.gate)}
        async function b(){return (0,obj.method)(await state.gate)}
        async function c(){return (await obj.method)(await state.gate)}
        Promise.all([a(),b(),c()]).then(value=>{result=value});
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "result===undefined");
    realm.eval("state.release(9)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result[0][0]===obj&&result[1][0]===undefined&&result[2][0]===undefined&&result.every(value=>value[1]===9)",
    );
}

#[test]
fn direct_eval_with_awaited_arguments_keeps_original_identity_variables_and_private_context() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},result,privateResult;state.gate=new Promise(resolve=>{state.release=resolve});
        async function f(){let value=1;const result=eval(await state.gate);return[result,value]}
        class C{#x=12;async read(p){return eval(await p)}}
        f().then(value=>{result=value});new C().read(Promise.resolve('this.#x')).then(value=>{privateResult=value});
        globalThis.eval=function(){throw 1};
    "#).unwrap();
    realm.eval("state.release('value=7;value')").unwrap();
    drain(&mut realm);
    check(&mut realm, "result.join(',')==='7,7'&&privateResult===12");
}

#[test]
fn awaited_eval_callee_is_an_indirect_value_call() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let marker=99,result,saved=eval;
        async function f(){let marker=1;return (await saved)('marker')}
        f().then(value=>{result=value});
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "result===99");
}

#[test]
fn awaited_spread_uses_sync_iteration_and_does_not_await_yielded_arguments() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let result,promise=Promise.resolve(4);function target(...values){return values}
        async function f(){return target(...await [1,2],await 3,...[promise])}
        f().then(value=>{result=value});
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result.length===4&&result[0]===1&&result[1]===2&&result[2]===3&&result[3]===promise",
    );
}

#[test]
fn noncallable_targets_evaluate_arguments_but_lookup_failures_skip_them() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},events=[],reason,marker={},lookupReason,operand=0,bad={get method(){throw marker}};
        state.gate=new Promise(resolve=>{state.release=resolve});
        async function f(){return (1)((events.push('first'),1),await state.gate,(events.push('last'),3))}
        async function g(){return bad.method(await (operand++))}
        f().catch(error=>{reason=error});g().catch(error=>{lookupReason=error});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='first'&&reason===undefined&&lookupReason===marker&&operand===0",
    );
    realm.eval("state.release(2)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "reason instanceof TypeError&&events.join(',')==='first,last'",
    );
}

#[test]
fn pending_call_frames_trace_receivers_and_earlier_arguments_and_release_them_after_invocation() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},result;state.gate=new Promise(resolve=>{state.release=resolve});
        function target(first,second){return this.x+first.x+second}
        let obj={x:3,method:target},arg={x:4};state.receiver=new WeakRef(obj);state.argument=new WeakRef(arg);
        async function f(){return obj.method(arg,await state.gate)}
        f().then(value=>{result=value});obj=null;arg=null;
    "#).unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "state.receiver.deref()!==undefined&&state.argument.deref()!==undefined&&result===undefined",
    );
    realm.eval("state.release(5)").unwrap();
    drain(&mut realm);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "result===12&&state.receiver.deref()===undefined&&state.argument.deref()===undefined",
    );
}

#[test]
fn resumed_argument_limit_failure_skips_invocation_rejection_and_resource_cleanup() {
    let mut realm = Realm::new(Limits {
        max_arguments: Some(2),
        ..Default::default()
    });
    realm.eval("let state={},called=false,caught=false,disposed=0;state.gate=new Promise(resolve=>{state.release=resolve});function target(){called=true}").unwrap();
    realm.eval("async function f(){using resource={[Symbol.dispose](){disposed++}};return target(1,await state.gate,3)}let output=f();output.catch(()=>{caught=true})").unwrap();
    realm.eval("state.release(2)").unwrap();
    let mut reported = false;
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => (),
            PromiseJobOutcome::Reported(Error::Limit { .. }) => reported = true,
            PromiseJobOutcome::Reported(error) => panic!("{error}"),
        }
    }
    assert!(reported);
    check(&mut realm, "!called&&!caught&&disposed===0");
}
