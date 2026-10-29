//! Internal reactions bypass public Promise hooks and retain only live captures.

use crate::{Error, ObjectHandle, PromiseJobOutcome, Realm, Value};
use spite_core::Span;

fn object(realm: &mut Realm, source: &str) -> ObjectHandle {
    let Value::Object(object) = realm.eval(source).unwrap() else {
        panic!("object");
    };
    object
}

fn subscribe(realm: &mut Realm, promise: &ObjectHandle, fulfill: Value, reject: Value) {
    assert_eq!(
        realm.perform_promise_then(promise, fulfill, reject, None, Span::new(0, 0)),
        Ok(Value::Undefined)
    );
}

fn completed(realm: &mut Realm) {
    assert!(matches!(
        realm.run_next_promise_job(),
        Ok(PromiseJobOutcome::Completed)
    ));
}

#[test]
fn internal_reactions_skip_then_species_and_result_assimilation() {
    for source in [
        "Promise.resolve(7)",
        "Promise.reject(7)",
        "new Promise(resolve=>{settle=resolve})",
        "new Promise((resolve,reject)=>{settle=reject})",
    ] {
        let mut realm = Realm::default();
        realm.eval("let settle,log=[],handler=function(value){'use strict';if(this!==undefined||arguments.length!==1||value!==7)throw 1;log.push(value);return{get then(){throw 2}}}").unwrap();
        let promise = object(
            &mut realm,
            &format!(
                "let p={source};Object.defineProperties(p,{{then:{{get(){{throw 3}}}},constructor:{{get(){{throw 4}}}}}});p"
            ),
        );
        let handler = realm.eval("handler").unwrap();
        subscribe(&mut realm, &promise, handler.clone(), handler);
        realm.eval("if(settle)settle(7)").unwrap();
        assert_eq!(realm.eval("log.length"), Ok(Value::Number(0.0)));
        completed(&mut realm);
        assert_eq!(realm.eval("log.join(',')"), Ok(Value::String("7".into())));
        assert!(matches!(
            realm.run_next_promise_job(),
            Ok(PromiseJobOutcome::NoJob)
        ));
    }
}

#[test]
fn internal_and_public_reactions_share_registration_and_fifo_order() {
    for source in ["Promise.resolve(1)", "new Promise(r=>{settle=r})"] {
        let mut realm = Realm::default();
        let promise = object(&mut realm, &format!("let settle,log=[],p={source};p"));
        let first = realm.eval("()=>{log.push('first')}").unwrap();
        subscribe(&mut realm, &promise, first, Value::Undefined);
        realm.eval("p.then(()=>{log.push('public')})").unwrap();
        let last = realm.eval("()=>{log.push('last')}").unwrap();
        subscribe(&mut realm, &promise, last, Value::Undefined);
        realm.eval("if(settle)settle(1)").unwrap();
        for _ in 0..3 {
            completed(&mut realm);
        }
        assert_eq!(
            realm.eval("log.join(',')"),
            Ok(Value::String("first,public,last".into()))
        );
        assert!(matches!(
            realm.run_next_promise_job(),
            Ok(PromiseJobOutcome::NoJob)
        ));
    }
}

#[test]
fn pending_and_queued_internal_reactions_trace_and_release_objects_and_symbols() {
    let mut realm = Realm::default();
    let promise = object(
        &mut realm,
        "let observed=false,state={};(()=>{let retained={},symbol=Symbol();state.fulfilled=()=>{observed=retained===state.object.deref()&&symbol===state.symbol.deref();return retained};state.object=new WeakRef(retained);state.symbol=new WeakRef(symbol)})();(()=>{let discarded={};state.rejected=()=>discarded;state.unused=new WeakRef(discarded)})();state.promise=new Promise(r=>{state.settle=r});state.promise",
    );
    let fulfill = realm.eval("state.fulfilled").unwrap();
    let reject = realm.eval("state.rejected").unwrap();
    subscribe(&mut realm, &promise, fulfill, reject);
    realm
        .eval("state.fulfilled=null;state.rejected=null")
        .unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("state.object.deref()!==undefined&&state.symbol.deref()!==undefined&&state.unused.deref()!==undefined"), Ok(Value::Boolean(true)));
    realm
        .eval("state.settle();state.promise=null;state.settle=null")
        .unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("state.object.deref()!==undefined&&state.symbol.deref()!==undefined&&state.unused.deref()===undefined"), Ok(Value::Boolean(true)));
    completed(&mut realm);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("observed&&state.object.deref()===undefined&&state.symbol.deref()===undefined"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn internal_job_preflight_rollback_and_host_failures_preserve_later_jobs() {
    let mut realm = Realm::default();
    let promise = object(&mut realm, "let calls=0,p=Promise.resolve();p");
    realm.reserve_unsupported_global("spiteUnavailableHost");
    let bad = realm.eval("()=>spiteUnavailableHost()").unwrap();
    let good = realm.eval("()=>{calls++}").unwrap();
    subscribe(&mut realm, &promise, bad, Value::Undefined);
    subscribe(&mut realm, &promise, good, Value::Undefined);
    realm.limits.max_steps = Some(0);
    assert!(matches!(
        realm.run_next_promise_job(),
        Err(Error::Limit { .. })
    ));
    realm.limits.max_steps = None;
    assert!(matches!(
        realm.run_next_promise_job(),
        Ok(PromiseJobOutcome::Reported(Error::Unsupported { .. }))
    ));
    assert_eq!(realm.eval("calls"), Ok(Value::Number(0.0)));
    completed(&mut realm);
    assert_eq!(realm.eval("calls"), Ok(Value::Number(1.0)));
    assert!(matches!(
        realm.run_next_promise_job(),
        Ok(PromiseJobOutcome::NoJob)
    ));
}
