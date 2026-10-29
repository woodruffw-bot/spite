//! Collection requests run between executions, with rooted responses and jobs.

use spite_runtime::{Error, FinalizationCleanupOutcome, PromiseJobOutcome, Realm, Value};

fn host() -> Realm {
    let mut realm = Realm::default();
    realm.install_test262_host().unwrap();
    realm
}
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
fn an_empty_collection_queue_is_inert_without_installing_host_globals() {
    let mut realm = Realm::default();
    assert!(!realm.has_requested_gc());
    assert_eq!(realm.run_requested_gc(0), Ok(false));
    check(&mut realm, "typeof $262==='undefined'");
}

#[test]
fn responses_are_fresh_intrinsic_promises_and_callbacks_follow_collection() {
    let mut realm = host();
    realm.eval(r#"
        let NativePromise=Promise,events=[];globalThis.Promise=function(){throw 'public constructor'};
        NativePromise.resolve=()=>{throw 'public resolve'};
        let p=$262.gc(),q=$262.gc(),fresh=p!==q&&Object.getPrototypeOf(p)===NativePromise.prototype;
        p.then(v=>events.push(v===undefined?'first':'wrong'));q.then(()=>events.push('second'));
    "#).unwrap();
    check(&mut realm, "fresh&&events.length===0");
    assert!(realm.has_requested_gc());
    assert_eq!(realm.run_requested_gc(usize::MAX), Ok(true));
    assert!(!realm.has_requested_gc());
    check(&mut realm, "events.length===0");
    drain(&mut realm);
    check(&mut realm, "events.join(',')==='first,second'");
}

#[test]
fn kept_objects_and_symbols_survive_current_execution_and_retire_at_the_checkpoint() {
    let mut realm = host();
    check(
        &mut realm,
        r#"
        let objectWeak,symbolWeak;
        (function(){objectWeak=new WeakRef({});symbolWeak=new WeakRef(Symbol())})();
        $262.gc();objectWeak.deref()!==undefined&&symbolWeak.deref()!==undefined
    "#,
    );
    assert_eq!(realm.run_requested_gc(usize::MAX), Ok(true));
    check(
        &mut realm,
        "objectWeak.deref()===undefined&&symbolWeak.deref()===undefined",
    );
}

#[test]
fn refused_collection_preserves_pending_requests_without_rejecting_javascript_promises() {
    let mut realm = host();
    realm
        .eval("let settled=false;let p=$262.gc();p.then(()=>settled=true,()=>settled=true)")
        .unwrap();
    assert!(matches!(
        realm.run_requested_gc(0),
        Err(Error::Limit { .. })
    ));
    assert!(realm.has_requested_gc());
    assert!(matches!(
        realm.run_next_promise_job(),
        Ok(PromiseJobOutcome::NoJob)
    ));
    check(&mut realm, "settled===false");
}

#[test]
fn finalization_callbacks_wait_for_a_separate_cleanup_job() {
    let mut realm = host();
    realm.eval("let finalized=[],registry=new FinalizationRegistry(v=>finalized.push(v));(function(){let target={};registry.register(target,7)})();$262.gc()").unwrap();
    realm.run_requested_gc(usize::MAX).unwrap();
    check(&mut realm, "finalized.length===0");
    assert!(matches!(
        realm.run_finalization_cleanup_job(),
        Ok(FinalizationCleanupOutcome::Completed)
    ));
    check(&mut realm, "finalized.join(',')==='7'");
}

#[test]
fn suspended_async_contexts_survive_collection_and_release_completed_captures() {
    let mut realm = host();
    realm.eval("let weak,result;(async function(){let target={x:7};weak=new WeakRef(target);await $262.gc();result=target.x===7&&weak.deref()===target})()").unwrap();
    realm.run_requested_gc(usize::MAX).unwrap();
    drain(&mut realm);
    check(&mut realm, "result===true");
    realm.eval("$262.gc()").unwrap();
    realm.run_requested_gc(usize::MAX).unwrap();
    check(&mut realm, "weak.deref()===undefined");
}

#[test]
fn suspended_generators_retain_locals_until_completion_without_collecting_on_the_stack() {
    let mut realm = host();
    realm.eval("let weak;function*g(){let target={x:7};weak=new WeakRef(target);$262.gc();yield 1;return target.x}let iterator=g();iterator.next()").unwrap();
    realm.run_requested_gc(usize::MAX).unwrap();
    check(
        &mut realm,
        "weak.deref()!==undefined&&iterator.next().value===7",
    );
    realm.eval("$262.gc()").unwrap();
    realm.run_requested_gc(usize::MAX).unwrap();
    check(&mut realm, "weak.deref()===undefined");
}

#[test]
fn embedding_roots_keep_completed_module_results_alive_across_requested_collection() {
    let mut realm = host();
    let result = realm.eval_module("$262.gc()").unwrap();
    let rooted = realm.root_value(result, usize::MAX).unwrap();
    realm.run_requested_gc(usize::MAX).unwrap();
    let Value::Object(promise) = rooted.value() else {
        panic!("module Promise")
    };
    assert!(matches!(
        realm.inspect_object(promise).unwrap().promise_state(),
        Some(spite_runtime::PromiseState::Fulfilled(_))
    ));
}
