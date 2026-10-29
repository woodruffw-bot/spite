//! Module body completion and top-level Promise settlement are separate jobs.

use super::*;
use crate::{Limits, PromiseJobOutcome, PromiseState};

fn state<'a>(realm: &'a Realm, promise: &Value) -> &'a PromiseState {
    let Value::Object(promise) = promise else {
        panic!("Promise")
    };
    realm
        .inspect_object(promise)
        .unwrap()
        .promise_state()
        .unwrap()
}
fn pending(realm: &Realm, promise: &Value) {
    assert!(matches!(state(realm, promise), PromiseState::Pending));
}
fn fulfilled(realm: &Realm, promise: &Value) {
    assert!(matches!(
        state(realm, promise),
        PromiseState::Fulfilled(Value::Undefined)
    ));
}
fn next(realm: &mut Realm) {
    assert!(matches!(
        realm.run_next_promise_job().unwrap(),
        PromiseJobOutcome::Completed
    ));
}
fn check(realm: &mut Realm, source: &str) {
    assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
}
fn collect(realm: &mut Realm) {
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unreachable_top_level_await_still_requires_a_module_completion_job() {
    let mut realm = Realm::default();
    realm.eval("let effects=0").unwrap();
    let module = realm.create_module("if(false){await 0};effects++").unwrap();
    let promise = realm.evaluate_module_record(&module).unwrap();
    pending(&realm, &promise);
    check(&mut realm, "effects===1");
    assert_eq!(realm.evaluate_module_record(&module).unwrap(), promise);
    next(&mut realm);
    fulfilled(&realm, &promise);
    assert!(matches!(
        realm.run_next_promise_job().unwrap(),
        PromiseJobOutcome::NoJob
    ));
    let sync = realm.eval_module("effects++").unwrap();
    fulfilled(&realm, &sync);
    check(&mut realm, "effects===2");
}

#[test]
fn resuming_the_body_precedes_a_distinct_top_level_settlement_job() {
    let mut realm = Realm::default();
    realm.eval("let effects=0").unwrap();
    let module = realm.create_module("await 0;effects++").unwrap();
    let promise = realm.evaluate_module_record(&module).unwrap();
    pending(&realm, &promise);
    check(&mut realm, "effects===0");
    next(&mut realm);
    check(&mut realm, "effects===1");
    pending(&realm, &promise);
    assert_eq!(realm.evaluate_module_record(&module).unwrap(), promise);
    next(&mut realm);
    fulfilled(&realm, &promise);
    assert!(matches!(
        realm.run_next_promise_job().unwrap(),
        PromiseJobOutcome::NoJob
    ));
}

#[test]
fn abrupt_body_completion_preserves_queued_symbol_reasons_and_top_level_promise_roots() {
    let mut realm = Realm::default();
    realm
        .eval("var token=Symbol(),weak=new WeakRef(token)")
        .unwrap();
    let module = realm.create_module("throw token;await 0").unwrap();
    let promise = realm.evaluate_module_record(&module).unwrap();
    pending(&realm, &promise);
    realm.eval("token=null").unwrap();
    collect(&mut realm);
    check(&mut realm, "weak.deref()!==undefined");
    next(&mut realm);
    let expected = realm.eval("weak.deref()").unwrap();
    assert!(
        matches!(state(&realm,&promise),PromiseState::Rejected(reason) if reason.same_value(&expected))
    );
    drop(expected);
    let promise = realm.root_value(promise, usize::MAX).unwrap();
    drop(module);
    collect(&mut realm);
    check(&mut realm, "weak.deref()!==undefined");
    drop(promise);
    collect(&mut realm);
    check(&mut realm, "weak.deref()===undefined");
}

#[test]
fn internal_module_reactions_ignore_public_promise_constructor_and_then_getters() {
    let mut realm = Realm::default();
    realm.eval("let reads=0;Object.defineProperty(Promise.prototype,'constructor',{get(){reads++;throw 1}});Object.defineProperty(Promise.prototype,'then',{get(){reads++;throw 2}})").unwrap();
    let promise = realm.eval_module("if(false){await 0}").unwrap();
    pending(&realm, &promise);
    next(&mut realm);
    fulfilled(&realm, &promise);
    check(&mut realm, "reads===0");
}

#[test]
fn initial_host_aborts_preserve_pending_evaluation_identity_without_language_cleanup() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(20_000),
        ..Limits::default()
    });
    realm.eval("let started=0,caught=0,finalized=0").unwrap();
    let module = realm
        .create_module("started++;try{while(true){}}catch(e){caught++}finally{finalized++};await 0")
        .unwrap();
    assert!(matches!(
        realm.evaluate_module_record(&module),
        Err(Error::Limit { .. })
    ));
    let promise = realm.evaluate_module_record(&module).unwrap();
    pending(&realm, &promise);
    assert_eq!(realm.evaluate_module_record(&module).unwrap(), promise);
    check(&mut realm, "started===1&&caught===0&&finalized===0");
    assert_eq!(
        realm
            .objects
            .module_execution(module.root.handle())
            .unwrap()
            .unwrap()
            .status,
        ModuleExecutionStatus::Aborted
    );
}

#[test]
fn resumed_host_aborts_mark_the_record_aborted_and_never_settle_its_top_level_promise() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(20_000),
        ..Limits::default()
    });
    realm.eval("let started=0,caught=0,finalized=0").unwrap();
    let module = realm
        .create_module("await 0;started++;try{while(true){}}catch(e){caught++}finally{finalized++}")
        .unwrap();
    let promise = realm.evaluate_module_record(&module).unwrap();
    pending(&realm, &promise);
    assert!(matches!(
        realm.run_next_promise_job().unwrap(),
        PromiseJobOutcome::Reported(Error::Limit { .. })
    ));
    pending(&realm, &promise);
    assert_eq!(realm.evaluate_module_record(&module).unwrap(), promise);
    check(&mut realm, "started===1&&caught===0&&finalized===0");
    assert_eq!(
        realm
            .objects
            .module_execution(module.root.handle())
            .unwrap()
            .unwrap()
            .status,
        ModuleExecutionStatus::Aborted
    );
    assert!(matches!(
        realm.run_next_promise_job().unwrap(),
        PromiseJobOutcome::NoJob
    ));
}
