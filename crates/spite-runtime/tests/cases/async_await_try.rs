//! Try/Catch/Finally execution through source suspension.

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
fn catch_handles_await_rejection_and_promise_resolve_getter_failure_without_replay() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let first={},second={},reason,rejected,events=[],input=Promise.resolve(1);
        Object.defineProperty(input,'constructor',{get(){events.push('get');throw second}});
        async function f(){try{await Promise.reject(first)}catch(error){rejected=error;await 0;events.push('caught')}}
        async function g(){try{await input}catch(error){reason=error;events.push('caught-get')}}
        f();g();events.push('outside');
    "#).unwrap();
    check(
        &mut realm,
        "reason===second&&rejected===undefined&&events.join(',')==='get,caught-get,outside'",
    );
    drain(&mut realm);
    check(
        &mut realm,
        "rejected===first&&reason===second&&events.join(',')==='get,caught-get,outside,caught'",
    );
}

#[test]
fn protected_returns_and_raw_throws_wait_for_finally_before_settlement_or_assimilation() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},events=[],result,reason,thenCalls=0,returned={then(resolve){thenCalls++;events.push('then');resolve(7)}},thrown={then(){throw 1}};
        state.gate=new Promise(resolve=>{state.release=resolve});
        async function f(){try{return returned}finally{events.push('return-enter');await state.gate;events.push('return-end')}}
        async function g(){try{throw thrown}finally{events.push('throw-enter');await state.gate;events.push('throw-end')}}
        f().then(value=>{result=value});g().catch(value=>{reason=value});
    "#).unwrap();
    check(
        &mut realm,
        "events.join(',')==='return-enter,throw-enter'&&thenCalls===0&&result===undefined&&reason===undefined",
    );
    realm.eval("state.release(0)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===7&&reason===thrown&&thenCalls===1&&events.indexOf('then')>events.indexOf('return-end')",
    );
}

#[test]
fn abrupt_finalizers_override_protected_returns_and_throws() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let first={},second=Symbol(),result,reason;
        async function f(){try{return first}finally{await 0;return second}}
        async function g(){try{throw first}finally{await 0;throw second}}
        f().then(value=>{result=value});g().catch(value=>{reason=value});
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "result===second&&reason===second");
}

#[test]
fn inner_disposal_precedes_catch_and_outer_disposal_follows_the_awaited_finalizer() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let marker={},cleanup={},events=[],caught,result;
        async function f(){using root={[Symbol.dispose](){events.push('root')}};try{using inner={[Symbol.dispose](){events.push('inner');throw cleanup}};await Promise.reject(marker)}catch(error){caught=error;events.push('catch');await 0}finally{events.push('finally');await 0}events.push('after');return 7}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===7&&events.join(',')==='inner,catch,finally,after,root'&&caught instanceof SuppressedError&&caught.error===cleanup&&caught.suppressed===marker",
    );
}

#[test]
fn catch_patterns_optional_bindings_private_context_and_var_aliases_survive_awaits() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result,optional,aliases,marker={};
        class C{#x=10;async run(){try{throw{a:2}}catch({a:x}){var seen=await x;return eval('this.#x')+seen+arguments.length}finally{await 0}}}
        async function f(){let x=1;try{throw marker}catch{await 0;eval('x=2')}return x}
        async function g(x){try{throw 5}catch(error){var x=await error+2}return[x,arguments[0]]}
        new C().run('arg').then(value=>{result=value});f().then(value=>{optional=value});g(1).then(value=>{aliases=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===13&&optional===2&&aliases.join(',')==='7,7'",
    );
}

#[test]
fn catch_parameter_failure_skips_its_body_and_runs_finally_once() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let marker={},events=[],body=0,reason;
        async function f(){try{throw{get x(){throw marker}}}catch({x}){body++}finally{events.push('finally');await 0;events.push('end')}}
        f().catch(value=>{reason=value});
    "#).unwrap();
    check(
        &mut realm,
        "body===0&&reason===undefined&&events.join(',')==='finally'",
    );
    drain(&mut realm);
    check(
        &mut realm,
        "body===0&&reason===marker&&events.join(',')==='finally,end'",
    );
}

#[test]
fn pending_normal_return_and_throw_values_remain_live_until_an_overriding_finalizer() {
    for body in ["make()", "return make()", "throw make()"] {
        let mut realm = Realm::default();
        realm
            .eval(
                r#"
            let state={},result;state.gate=new Promise(resolve=>{state.release=resolve});
            function make(){const value={x:7};state.weak=new WeakRef(value);return value}
        "#,
            )
            .unwrap();
        let source="async function f(){try{BODY}finally{await state.gate;return 7}}f().then(value=>{result=value})".replace("BODY",body);
        realm.eval(&source).unwrap();
        collect(&mut realm);
        check(
            &mut realm,
            "state.weak.deref()!==undefined&&result===undefined",
        );
        realm.eval("state.release(0)").unwrap();
        drain(&mut realm);
        collect(&mut realm);
        check(&mut realm, "state.weak.deref()===undefined&&result===7");
    }
}

#[test]
fn host_failures_in_try_catch_and_finally_bypass_remaining_handlers_and_release_captures() {
    for phase in 0..3 {
        let mut realm = Realm::default();
        realm.eval(r#"
            let state={},disposed=0,caught=0,finalized=0,observed=false,read,marker={},resource={[Symbol.dispose](){disposed++}};
            state.weakMethod=new WeakRef(resource[Symbol.dispose]);
            function make(){const value={x:7};state.weakValue=new WeakRef(value);return value}
        "#).unwrap();
        let (body, handler, finalizer) = match phase {
            0 => (
                "await 0;new SharedArrayBuffer(0)",
                "caught++",
                "finalized++",
            ),
            1 => (
                "throw marker",
                "caught++;await 0;new SharedArrayBuffer(0)",
                "finalized++",
            ),
            _ => (
                "return make()",
                "caught++",
                "finalized++;await 0;new SharedArrayBuffer(0)",
            ),
        };
        let source="async function f(){using r=resource;read=()=>r;try{BODY}catch(error){HANDLER}finally{FINALIZER}}f().catch(()=>{observed=true});delete resource[Symbol.dispose];".replace("BODY",body).replace("HANDLER",handler).replace("FINALIZER",finalizer);
        realm.eval(&source).unwrap();
        if phase == 2 {
            collect(&mut realm);
            check(&mut realm, "state.weakValue.deref()!==undefined");
        }
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
        collect(&mut realm);
        check(
            &mut realm,
            "disposed===0&&!observed&&read()===resource&&state.weakMethod.deref()===undefined",
        );
        check(
            &mut realm,
            &format!(
                "caught==={}&&finalized==={}",
                usize::from(phase == 1),
                usize::from(phase == 2)
            ),
        );
        if phase == 2 {
            check(&mut realm, "state.weakValue.deref()===undefined");
        }
    }
}

#[test]
fn host_budget_exhaustion_in_finally_releases_all_disposers_without_cleanup() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(10_000),
        ..Default::default()
    });
    realm
        .eval("let state={},disposed=0,caught=false,read;const dispose=Symbol.dispose;")
        .unwrap();
    realm
        .eval("let resource={[dispose](){disposed++}};")
        .unwrap();
    realm
        .eval("state.weak=new WeakRef(resource[dispose]);")
        .unwrap();
    realm.eval("async function f(){using root=resource;try{return 7}finally{using inner=resource;read=()=>inner;await 0;while(true){}}}").unwrap();
    realm.eval("f().catch(()=>{caught=true});").unwrap();
    realm.eval("delete resource[dispose];").unwrap();
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
    collect(&mut realm);
    check(
        &mut realm,
        "disposed===0&&!caught&&read()===resource&&state.weak.deref()===undefined",
    );
}

#[test]
fn normal_finalizer_values_are_ignored_and_empty_try_completion_replaces_prior_values() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},result,normal='unset';state.first=new Promise(resolve=>{state.releaseFirst=resolve});state.second=new Promise(resolve=>{state.releaseSecond=resolve});
        function make(){const value={x:7};state.weak=new WeakRef(value);return value}
        async function f(){make();try{}finally{await state.first;99}const x=await state.second;return 7}
        async function g(){try{41}finally{await 0;99}}
        f().then(value=>{result=value});g().then(value=>{normal=value});
    "#).unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "state.weak.deref()!==undefined&&result===undefined",
    );
    realm.eval("state.releaseFirst(0)").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "state.weak.deref()===undefined&&normal===undefined&&result===undefined",
    );
    realm.eval("state.releaseSecond(0)").unwrap();
    drain(&mut realm);
    check(&mut realm, "result===7");
}
