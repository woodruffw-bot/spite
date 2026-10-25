//! Source Await continuation timing, contexts and cleanup.

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
fn direct_await_yields_once_and_resumes_each_statement_without_replay() {
    let mut realm = Realm::default();
    realm.eval("let events=[];async function f(){events.push('a');await 0;events.push('b');await 0;events.push('c');return 3}let p=f();events.push('outside');p.then(v=>{events.push(v)})").unwrap();
    check(&mut realm, "events.join(',')==='a,outside'");
    assert!(matches!(
        realm.run_next_promise_job().unwrap(),
        PromiseJobOutcome::Completed
    ));
    check(&mut realm, "events.join(',')==='a,outside,b'");
    assert!(matches!(
        realm.run_next_promise_job().unwrap(),
        PromiseJobOutcome::Completed
    ));
    check(&mut realm, "events.join(',')==='a,outside,b,c'");
    drain(&mut realm);
    check(&mut realm, "events.join(',')==='a,outside,b,c,3'");
}

#[test]
fn return_await_and_concise_arrows_forward_fulfillment_and_rejection() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let marker={},seen=[],id=async value=>await value;
        async function block(value){return (await value)}
        for(const f of[id,block]){
            f(Promise.resolve(7)).then(v=>{seen.push(v===7)});
            f({then(resolve){resolve(9)}}).then(v=>{seen.push(v===9)});
            f(Promise.reject(marker)).catch(e=>{seen.push(e===marker)});
        }
        let input=Promise.resolve(1),output=id(input);if(output===input)throw 1;
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "seen.length===6&&seen.every(v=>v)");
}

#[test]
fn native_await_ignores_public_then_species_and_replaced_promise_names() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let Saved=Promise,then=Promise.prototype.then,wait=Promise.resolve(7),result;
        Object.defineProperty(wait,'then',{get(){throw 1}});
        Object.defineProperty(Promise,Symbol.species,{get(){throw 2}});
        async function f(){return await wait}
        let output=f();Object.defineProperty(output,'constructor',{value:undefined});
        Reflect.apply(then,output,[v=>{result=v}]);Promise=function(){throw 3};
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "result===7");
}

#[test]
fn unreachable_await_is_skipped_and_nested_await_contexts_stay_explicit_gaps() {
    let mut realm = Realm::default();
    realm
        .eval(
            "let result;async function f(){if(false)await missing;return 3}f().then(v=>{result=v})",
        )
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "result===3");
    realm
        .eval("let flag=0;async function g(){return 1+await (flag=1)}")
        .unwrap();
    assert!(matches!(
        realm.eval("try{g()}catch(e){flag=2}finally{flag=3}"),
        Err(Error::Unsupported { .. })
    ));
    check(&mut realm, "flag===0");
}

#[test]
fn resumed_functions_preserve_arguments_this_eval_and_lexical_arrow_context() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},receiver={x:7},results=[];
        state.gate=new Promise(resolve=>{state.release=resolve});
        async function ordinary(a){arguments[0]=4;await state.gate;return[this,a,arguments[0],eval('a')]}
        function maker(v){return async()=>{await state.gate;return[this,arguments[0],new.target]}}
        ordinary.call(receiver,2).then(v=>{results.push(v[0]===receiver&&v[1]===4&&v[2]===4&&v[3]===4)});
        maker.call(receiver,11)().then(v=>{results.push(v[0]===receiver&&v[1]===11&&v[2]===undefined)});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "results.length===0");
    realm.eval("state.release(0)").unwrap();
    drain(&mut realm);
    check(&mut realm, "results.length===2&&results.every(v=>v)");
}

#[test]
fn methods_private_names_super_and_dynamic_functions_resume_in_their_own_contexts() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let state={},results=[];state.gate=new Promise(resolve=>{state.release=resolve});
        class Base{get value(){return this.x}}
        class C extends Base{
            #hidden=7;
            async #read(p){await p;return this.#hidden+super.value}
            run(p){return this.#read(p)}
            static async read(p){await p;return this.x}
        }
        let c=new C;c.x=3;C.x=8;
        c.run(state.gate).then(v=>{results.push(v===10)});
        C.read(state.gate).then(v=>{results.push(v===8)});
        let AF=(async function(){}).constructor,compiled=AF('p','await p;return this.x');
        compiled.call(c,state.gate).then(v=>{results.push(v===3)});
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "results.length===0");
    realm.eval("state.release(0)").unwrap();
    drain(&mut realm);
    check(&mut realm, "results.length===3&&results.every(v=>v)");
}

#[test]
fn pending_frames_keep_used_parameters_and_release_unrelated_callers_and_completed_locals() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},seen;state.gate=new Promise(resolve=>{state.release=resolve});
        async function wait(argument){await state.gate;seen=argument.x}
        (()=>{let value={x:9},unused={};state.used=new WeakRef(value);state.unused=new WeakRef(unused);state.output=wait(value)})();
    "#).unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "state.used.deref()!==undefined&&state.unused.deref()===undefined&&seen===undefined",
    );
    realm.eval("state.release(0)").unwrap();
    drain(&mut realm);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(&mut realm, "seen===9&&state.used.deref()===undefined");
}

#[test]
fn using_resources_survive_suspension_and_dispose_after_return_or_rejection() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},events=[],result;state.gate=new Promise(resolve=>{state.release=resolve});
        async function f(){using resource={[Symbol.dispose](){events.push('dispose')}};events.push('start');await state.gate;events.push('end');return 7}
        f().then(v=>{result=v});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "events.join(',')==='start'&&result===undefined");
    realm.eval("state.release(0)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='start,end,dispose'&&result===7",
    );
    realm.eval(r#"
        let original={},cleanup={},suppressed;
        async function rejected(){using resource={[Symbol.dispose](){throw cleanup}};return await Promise.reject(original)}
        rejected().catch(e=>{suppressed=e});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "suppressed instanceof SuppressedError&&suppressed.error===cleanup&&suppressed.suppressed===original",
    );
}

#[test]
fn promise_resolve_getter_failure_rejects_without_suspending_and_runs_lexical_cleanup() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let marker={},events=[],reason,input=Promise.resolve(1);
        Object.defineProperty(input,'constructor',{get(){events.push('get');throw marker}});
        async function f(){using resource={[Symbol.dispose](){events.push('dispose')}};return await input}
        f().catch(e=>{reason=e});events.push('outside');
    "#).unwrap();
    check(
        &mut realm,
        "events.join(',')==='get,dispose,outside'&&reason===undefined",
    );
    drain(&mut realm);
    check(&mut realm, "reason===marker");
}

#[test]
fn resumed_host_failures_skip_rejection_and_disposal_and_release_cached_methods_even_at_budget_zero()
 {
    for limited in [false, true] {
        let mut realm = Realm::new(spite_runtime::Limits {
            max_steps: limited.then_some(10_000),
            ..Default::default()
        });
        let tail = if limited {
            "while(true){}"
        } else {
            "new SharedArrayBuffer(0)"
        };
        realm.eval("let resource={[Symbol.dispose](){disposed++}},disposed=0,flag=0,read,weak=new WeakRef(resource[Symbol.dispose]);").unwrap();
        realm
            .eval(&format!(
                "async function f(){{using r=resource;read=()=>r;await 0;{tail}}}"
            ))
            .unwrap();
        realm
            .eval("let output=f();delete resource[Symbol.dispose];output.catch(()=>{flag=1});")
            .unwrap();
        realm.clear_kept_objects().unwrap();
        realm.collect(usize::MAX).unwrap();
        check(&mut realm, "weak.deref()!==undefined");
        let mut reported = false;
        loop {
            match realm.run_next_promise_job().unwrap() {
                PromiseJobOutcome::NoJob => break,
                PromiseJobOutcome::Completed => (),
                PromiseJobOutcome::Reported(error) => {
                    assert!(matches!(error, Error::Limit { .. }) == limited, "{error}");
                    if !limited {
                        assert!(matches!(error, Error::Unsupported { .. }), "{error}");
                    }
                    reported = true;
                }
            }
        }
        assert!(reported);
        realm.clear_kept_objects().unwrap();
        realm.collect(usize::MAX).unwrap();
        check(
            &mut realm,
            "disposed===0&&flag===0&&read()===resource&&weak.deref()===undefined",
        );
    }
}
