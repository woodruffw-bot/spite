//! Retained Module identity, evaluation caching and host roots (16.2.1).

use spite_core::{DiagnosticKind, JsString};
use spite_runtime::{Error, Limits, PromiseJobOutcome, PromiseState, Realm, Value};

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
fn state<'a>(realm: &'a Realm, promise: &Value) -> &'a PromiseState {
    let Value::Object(handle) = promise else {
        panic!("Promise")
    };
    realm
        .inspect_object(handle)
        .unwrap()
        .promise_state()
        .unwrap()
}
fn collect(realm: &mut Realm) {
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn retained_records_run_once_while_distinct_records_and_fresh_evaluation_remain_independent() {
    let mut realm = Realm::default();
    realm.eval("let effects=0").unwrap();
    let source = "effects++;export let value=effects;globalThis.read=()=>value";
    let module = realm.create_module(source).unwrap();
    check(&mut realm, "effects===0");
    let first = realm.evaluate_module_record(&module).unwrap();
    assert!(matches!(
        state(&realm, &first),
        PromiseState::Fulfilled(Value::Undefined)
    ));
    let clone = module.clone();
    assert_eq!(clone, module);
    assert_eq!(realm.evaluate_module_record(&clone), Ok(first));
    check(&mut realm, "effects===1&&read()===1");
    let other = realm.create_module(source).unwrap();
    assert_ne!(module, other);
    realm.evaluate_module_record(&other).unwrap();
    check(&mut realm, "effects===2&&read()===2");
    realm.eval_module(source).unwrap();
    check(&mut realm, "effects===3&&read()===3");
}

#[test]
fn pending_and_rejected_records_keep_the_same_promise_and_original_reason() {
    let mut realm = Realm::default();
    realm
        .eval("let effects=0,release,reason=Symbol();let gate=new Promise(r=>release=r)")
        .unwrap();
    let module = realm
        .create_module("effects++;await gate;throw reason")
        .unwrap();
    let first = realm.evaluate_module_record(&module).unwrap();
    assert!(matches!(state(&realm, &first), PromiseState::Pending));
    assert_eq!(realm.evaluate_module_record(&module), Ok(first.clone()));
    check(&mut realm, "effects===1");
    realm.eval("release()").unwrap();
    drain(&mut realm);
    let PromiseState::Rejected(reason) = state(&realm, &first) else {
        panic!("rejected")
    };
    let reason = reason.clone();
    assert_eq!(realm.eval("reason"), Ok(reason));
    assert_eq!(realm.evaluate_module_record(&module), Ok(first));
    check(&mut realm, "effects===1");
}

#[test]
fn host_aborted_records_keep_their_pending_promise_without_replaying_effects_or_cleanup() {
    let mut realm = Realm::default();
    realm.reserve_unsupported_global("unsupportedHost");
    realm.eval("let effects=0,finalized=0").unwrap();
    let module = realm
        .create_module("effects++;try{unsupportedHost}finally{finalized++}")
        .unwrap();
    assert!(matches!(
        realm.evaluate_module_record(&module),
        Err(Error::Unsupported { .. })
    ));
    let pending = realm.evaluate_module_record(&module).unwrap();
    assert!(matches!(state(&realm, &pending), PromiseState::Pending));
    assert_eq!(realm.evaluate_module_record(&module), Ok(pending));
    check(&mut realm, "effects===1&&finalized===0");
}

#[test]
fn the_last_module_handle_keeps_finished_environments_and_rejection_values_alive() {
    for source in [
        "const held=Symbol();weak=new WeakRef(held)",
        "throw (()=>{const held=Symbol();weak=new WeakRef(held);return held})()",
    ] {
        let mut realm = Realm::default();
        realm.eval("let weak").unwrap();
        let module = realm.create_module(source).unwrap();
        realm.evaluate_module_record(&module).unwrap();
        collect(&mut realm);
        check(&mut realm, "weak.deref()!==undefined");
        let clone = module.clone();
        drop(module);
        collect(&mut realm);
        check(&mut realm, "weak.deref()!==undefined");
        drop(clone);
        collect(&mut realm);
        check(&mut realm, "weak.deref()===undefined");
    }
}

#[test]
fn record_ownership_and_lossless_ast_input_use_the_shared_validation_and_source_paths() {
    let mut realm = Realm::default();
    let module = realm.create_module("export {}").unwrap();
    assert!(matches!(
        Realm::default().evaluate_module_record(&module),
        Err(Error::InvalidObject(_))
    ));
    let mut units: Vec<u16> = "globalThis.result='".encode_utf16().collect();
    units.push(0xd800);
    units.extend("';export {}".encode_utf16());
    let parsed = spite_parser::parse_module_utf16(&JsString::from_code_units(units)).unwrap();
    let module = realm.create_module_from_ast(&parsed).unwrap();
    realm.evaluate_module_record(&module).unwrap();
    assert_eq!(
        realm.eval("result"),
        Ok(Value::String(JsString::from_code_units(vec![0xd800])))
    );
}

#[test]
fn parsing_and_opted_in_source_limits_precede_creation_or_body_effects() {
    let mut realm = Realm::default();
    realm.eval("let effects=0").unwrap();
    assert!(matches!(
        realm.create_module("effects++;with({}){}"),
        Err(Error::Parse(_))
    ));
    let module = realm.create_module("effects++").unwrap();
    check(&mut realm, "effects===0");
    drop(module);
    collect(&mut realm);
    check(&mut realm, "effects===0");
    let mut limited = Realm::new(Limits {
        max_source_bytes: Some(20),
        ..Limits::default()
    });
    assert!(limited.create_module("globalThis.effects=1").is_ok());
    assert!(
        matches!(limited.create_module("globalThis.effects=1 "),Err(Error::Parse(ref diagnostic)) if diagnostic.kind==DiagnosticKind::Limit)
    );
    assert_eq!(
        limited.eval("typeof effects"),
        Ok(Value::String(JsString::from("undefined")))
    );
}
