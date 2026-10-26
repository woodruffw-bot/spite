//! Standalone Module environments and top-level Await (16.2.1).

use spite_runtime::{Error, PromiseJobOutcome, Realm, Value, object::PromiseState};

fn check(realm: &mut Realm, source: &str) {
    assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
}
fn drain(realm: &mut Realm) {
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => (),
            PromiseJobOutcome::Reported(error) => panic!("{error:?}"),
        }
    }
}
fn state<'a>(realm: &'a Realm, value: &Value) -> &'a PromiseState {
    let Value::Object(handle) = value else {
        panic!("evaluation Promise")
    };
    realm
        .inspect_object(handle)
        .unwrap()
        .promise_state()
        .unwrap()
}

#[test]
fn module_bindings_are_private_and_functions_are_initialized_before_body_entry() {
    let mut realm = Realm::default();
    realm.eval("var shared=99;let observations=[]").unwrap();
    let promise = realm.eval_module("observations.push(f(),shared);export var shared=1;export let value=2;export function f(){return shared};globalThis.readModule=()=>[shared,value,f()];globalThis.writeModule=x=>shared=x").unwrap();
    assert!(matches!(
        state(&realm, &promise),
        PromiseState::Fulfilled(Value::Undefined)
    ));
    check(
        &mut realm,
        "observations.length===2&&observations.every(x=>x===undefined)&&shared===99&&readModule().join(',')==='1,2,1'&&typeof value==='undefined'&&typeof f==='undefined'&&!Object.hasOwn(globalThis,'value')",
    );
    check(
        &mut realm,
        "writeModule(7)===7&&readModule().join(',')==='7,2,7'&&shared===99",
    );
}

#[test]
fn module_this_and_direct_eval_keep_module_context_across_functions_and_arrows() {
    let mut realm = Realm::default();
    realm.eval("let results=[]").unwrap();
    let promise=realm.eval_module("results.push(this===undefined,(()=>this)()===undefined,eval('this')===undefined);try{eval('new.target')}catch(e){results.push(e instanceof SyntaxError)};eval('var local=1');results.push(typeof local==='undefined');globalThis.arrow=()=>eval('new.target');globalThis.ordinary=function(){return eval('new.target')}").unwrap();
    assert!(matches!(
        state(&realm, &promise),
        PromiseState::Fulfilled(_)
    ));
    check(
        &mut realm,
        "results.length===5&&results.every(x=>x)&&ordinary()===undefined&&(()=>{try{arrow()}catch(e){return e instanceof SyntaxError}})()&&this===globalThis",
    );
}

#[test]
fn top_level_await_keeps_module_cells_and_restores_the_embedding_script_context() {
    let mut realm = Realm::default();
    realm
        .eval("let release,events=[];let gate=new Promise(r=>release=r);var value=99")
        .unwrap();
    let promise=realm.eval_module("export let value=1;events.push('start');await gate;value++;events.push(value);globalThis.read=()=>value").unwrap();
    assert!(matches!(state(&realm, &promise), PromiseState::Pending));
    check(
        &mut realm,
        "events.join(',')==='start'&&value===99&&this===globalThis",
    );
    realm.eval("release()").unwrap();
    drain(&mut realm);
    assert!(matches!(
        state(&realm, &promise),
        PromiseState::Fulfilled(Value::Undefined)
    ));
    check(
        &mut realm,
        "events.join(',')==='start,2'&&read()===2&&value===99&&this===globalThis",
    );
}

#[test]
fn language_errors_reject_evaluation_and_host_errors_bypass_rejection_and_cleanup() {
    let mut realm = Realm::default();
    for source in ["throw 7", "await Promise.reject(7)"] {
        let promise = realm.eval_module(source).unwrap();
        drain(&mut realm);
        assert!(matches!(
            state(&realm, &promise),
            PromiseState::Rejected(Value::Number(7.0))
        ));
    }
    realm.reserve_unsupported_global("unsupportedHost");
    realm
        .eval("let finalized=0,release;let gate=new Promise(r=>release=r)")
        .unwrap();
    assert!(matches!(
        realm.eval_module("try{unsupportedHost}finally{finalized++}"),
        Err(Error::Unsupported { .. })
    ));
    let promise = realm
        .eval_module("try{await gate;unsupportedHost}finally{finalized++}")
        .unwrap();
    realm.eval("release()").unwrap();
    assert!(matches!(
        realm.run_next_promise_job(),
        Ok(PromiseJobOutcome::Reported(Error::Unsupported { .. }))
    ));
    assert!(matches!(state(&realm, &promise), PromiseState::Pending));
    check(&mut realm, "finalized===0&&this===globalThis");
}

#[test]
fn dependencies_stop_before_effects_while_default_declarations_and_expressions_execute() {
    let mut realm = Realm::default();
    realm.eval("let effects=0").unwrap();
    for source in ["effects++;import 'other'", "effects++;export*from'other'"] {
        assert!(
            matches!(realm.eval_module(source), Err(Error::Unsupported { .. })),
            "{source}"
        );
    }
    check(&mut realm, "effects===0");
    for source in [
        "effects++;export default 1",
        "effects++;export default function(){}",
        "effects++;export default class{}",
    ] {
        let promise = realm.eval_module(source).unwrap();
        assert!(matches!(
            state(&realm, &promise),
            PromiseState::Fulfilled(_)
        ));
    }
    check(&mut realm, "effects===3");

    for source in [
        "export default function f(){return 7};globalThis.result=f()",
        "export default class C{static value=7};globalThis.result=C.value",
    ] {
        let promise = realm.eval_module(source).unwrap();
        assert!(matches!(
            state(&realm, &promise),
            PromiseState::Fulfilled(_)
        ));
        check(&mut realm, "result===7");
    }
}

#[test]
fn suspended_module_environments_trace_symbols_and_release_finished_scopes() {
    let mut realm = Realm::default();
    realm
        .eval("let weak,release;let gate=new Promise(r=>release=r)")
        .unwrap();
    let promise = realm
        .eval_module(
            "const held=Symbol();weak=new WeakRef(held);await gate;globalThis.observed=typeof held",
        )
        .unwrap();
    let promise = realm.root_value(promise, usize::MAX).unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(&mut realm, "weak.deref()!==undefined");
    realm.eval("release();release=null").unwrap();
    drain(&mut realm);
    assert!(matches!(
        state(&realm, promise.value()),
        PromiseState::Fulfilled(_)
    ));
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(&mut realm, "observed==='symbol'&&weak.deref()===undefined");
}
