//! Awaited while/do-while control, completions and per-iteration blocks.

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
fn while_conditions_and_bodies_await_once_in_order_for_each_iteration() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let tests=0,events=[],result;
        async function f(){while(await(++tests<=3)){events.push(tests);await 0;events.push('end'+tests)}return 7}
        f().then(value=>{result=value});
    "#).unwrap();
    check(
        &mut realm,
        "tests===1&&events.length===0&&result===undefined",
    );
    drain(&mut realm);
    check(
        &mut realm,
        "tests===4&&events.join(',')==='1,end1,2,end2,3,end3'&&result===7",
    );
}

#[test]
fn while_skips_a_false_body_and_do_while_runs_its_body_before_the_first_test() {
    let mut realm = Realm::with_unavailable_host();
    realm
        .eval(
            r#"
        let events=[],first,second;
        async function f(){while(await false){spiteUnavailableHost();await missing}return 1}
        async function g(){do{events.push('body');await 0}while(await false);return 2}
        f().then(value=>{first=value});g().then(value=>{second=value});
    "#,
        )
        .unwrap();
    check(
        &mut realm,
        "events.join(',')==='body'&&first===undefined&&second===undefined",
    );
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='body'&&first===1&&second===2",
    );
}

#[test]
fn break_and_continue_wait_for_finalizers_and_dispose_body_resources_before_control_moves() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],result;
        async function f(){let i=0;while(await(i<4)){const n=i++;using resource={[Symbol.dispose](){events.push('dispose'+n)}};try{await 0;if(n===0)continue;if(n===2)break;events.push('body'+n)}finally{events.push('finally'+n);await 0;events.push('end'+n)}}return i}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===3&&events.join(',')==='finally0,end0,dispose0,body1,finally1,end1,dispose1,finally2,end2,dispose2'",
    );
}

#[test]
fn do_while_continue_runs_the_test_and_block_closures_keep_distinct_iteration_bindings() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let tests=0,result;
        async function f(){let i=0;const closures=[];do{const value=i++;await 0;closures.push(()=>value);if(i<3)continue;break}while(await(++tests,true));return closures.map(fn=>fn()).join(',')}
        f().then(value=>{result=value});
    "#).unwrap();
    check(&mut realm, "tests===0&&result===undefined");
    drain(&mut realm);
    check(&mut realm, "tests===2&&result==='0,1,2'");
}

#[test]
fn rejected_conditions_and_body_throws_reach_outer_catches_without_retesting() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let marker={},reason,thrown,body=0,tests=0,finalized=0,first,second;
        async function f(){try{while(await Promise.reject(marker)){body++}}catch(error){reason=error;await 0}finally{finalized++}return 7}
        async function g(){try{do{await 0;throw marker}while(await(++tests,false))}catch(error){thrown=error;return 8}}
        f().then(value=>{first=value});g().then(value=>{second=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "reason===marker&&thrown===marker&&body===0&&tests===0&&finalized===1&&first===7&&second===8",
    );
}

#[test]
fn loop_values_survive_suspended_retests_and_forward_until_the_parent_replaces_them() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},tests=0,result;state.first=new Promise(resolve=>{state.releaseFirst=resolve});state.second=new Promise(resolve=>{state.releaseSecond=resolve});state.third=new Promise(resolve=>{state.releaseThird=resolve});
        function make(){const value={x:7};state.weak=new WeakRef(value);return value}
        async function f(){while(tests++===0?true:await state.first){make()}const x=await state.second;0;const y=await state.third;return 7}
        f().then(value=>{result=value});
    "#).unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "tests===2&&state.weak.deref()!==undefined&&result===undefined",
    );
    realm.eval("state.releaseFirst(false)").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "tests===2&&state.weak.deref()!==undefined&&result===undefined",
    );
    realm.eval("state.releaseSecond(0)").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "state.weak.deref()===undefined&&result===undefined",
    );
    realm.eval("state.releaseThird(0)").unwrap();
    drain(&mut realm);
    check(&mut realm, "result===7");
}

#[test]
fn loop_bodies_preserve_private_eval_arguments_and_var_parameter_aliases() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result,alias;
        class C{#x=10;async read(limit){let total=0;while(await(limit>0)){total+=eval('this.#x')+arguments.length;limit--;await 0}return total}}
        async function f(x){while(await x)var x=await 0;return arguments[0]}
        new C().read(2).then(value=>{result=value});f(1).then(value=>{alias=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result===22&&alias===0");
}

#[test]
fn host_failures_in_loop_bodies_bypass_finally_and_release_resources_at_budget_zero() {
    for limited in [false, true] {
        let mut realm = Realm::with_unavailable_host_limits(Limits {
            max_steps: limited.then_some(10_000),
            ..Default::default()
        });
        realm.eval("let state={},disposed=0,finalized=0,caught=false,read;const dispose=Symbol.dispose;").unwrap();
        realm
            .eval("let resource={[dispose](){disposed++}};")
            .unwrap();
        realm
            .eval("state.weak=new WeakRef(resource[dispose]);")
            .unwrap();
        let tail = if limited {
            "while(true){}"
        } else {
            "spiteUnavailableHost()"
        };
        let source="async function f(){using root=resource;try{while(true){using inner=resource;read=()=>inner;await 0;TAIL}}finally{finalized++}}".replace("TAIL",tail);
        realm.eval(&source).unwrap();
        realm.eval("f().catch(()=>{caught=true});").unwrap();
        realm.eval("delete resource[dispose];").unwrap();
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
            "disposed===0&&finalized===0&&!caught&&read()===resource&&state.weak.deref()===undefined",
        );
    }
}
