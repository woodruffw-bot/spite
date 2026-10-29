//! Host import completion uses intrinsic jobs and the cached Module evaluator.

use super::*;
use crate::{Limits, PromiseJobOutcome, PromiseState};
use spite_core::JsString;

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
fn value(realm: &Realm, promise: &Value) -> Value {
    let PromiseState::Fulfilled(value) = state(realm, promise) else {
        panic!("fulfilled Promise")
    };
    value.clone()
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
fn expose(realm: &mut Realm, name: &str, value: Value) {
    realm
        .set_property_value(
            &Value::Object(realm.global_object()),
            JsString::from(name),
            value,
            Span::new(0, 0),
        )
        .unwrap();
}
fn collect(realm: &mut Realm) {
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn synchronous_imports_deliver_one_cached_live_namespace_through_fresh_promises() {
    let mut realm = Realm::default();
    realm.eval("let effects=0").unwrap();
    let module = realm
        .create_module("effects++;export let x=1;export function bump(){x++}")
        .unwrap();
    let first = realm.import_module_record(&module).unwrap();
    let second = realm.import_module_record(&module).unwrap();
    assert_ne!(first, second);
    pending(&realm, &first);
    pending(&realm, &second);
    next(&mut realm);
    let namespace = value(&realm, &first);
    assert_eq!(namespace, realm.module_namespace(&module).unwrap());
    next(&mut realm);
    assert_eq!(value(&realm, &second), namespace);
    expose(&mut realm, "ns", namespace);
    check(&mut realm, "effects===1&&ns.x===1");
    check(&mut realm, "ns.bump();ns.x===2");
}

#[test]
fn async_dependencies_finish_before_the_namespace_delivery_job() {
    let mut realm = Realm::default();
    let root = realm.create_module("export {x} from'leaf'").unwrap();
    let leaf = realm.create_module("await 0;export let x=42").unwrap();
    realm
        .load_module_graph(&root, |_, _| Ok(leaf.clone()))
        .unwrap();
    let imported = realm.import_module_record(&root).unwrap();
    next(&mut realm);
    pending(&realm, &imported);
    next(&mut realm);
    pending(&realm, &imported);
    next(&mut realm);
    let namespace = value(&realm, &imported);
    expose(&mut realm, "ns", namespace);
    check(&mut realm, "ns.x===42");
}

#[test]
fn cycle_members_share_evaluation_but_deliver_the_requested_members_namespace() {
    let mut realm = Realm::default();
    let a = realm.create_module("import'b';export const a=1").unwrap();
    let b = realm.create_module("import'a';export const b=2").unwrap();
    realm
        .load_module_graph(&a, |_, request| {
            Ok(if request.specifier == JsString::from("a") {
                a.clone()
            } else {
                b.clone()
            })
        })
        .unwrap();
    let first = realm.import_module_record(&a).unwrap();
    let second = realm.import_module_record(&b).unwrap();
    assert_eq!(
        realm.evaluate_module_record(&a).unwrap(),
        realm.evaluate_module_record(&b).unwrap()
    );
    next(&mut realm);
    next(&mut realm);
    let first = value(&realm, &first);
    let second = value(&realm, &second);
    assert_ne!(first, second);
    assert_eq!(first, realm.module_namespace(&a).unwrap());
    assert_eq!(second, realm.module_namespace(&b).unwrap());
    expose(&mut realm, "first", first);
    expose(&mut realm, "second", second);
    check(
        &mut realm,
        "first.a===1&&!(\"b\" in first)&&second.b===2&&!(\"a\" in second)",
    );
}

#[test]
fn linking_errors_reject_and_cached_body_rejections_keep_the_original_reason() {
    let mut realm = Realm::default();
    let root = realm.create_module("import{missing}from'leaf'").unwrap();
    let leaf = realm.create_module("export const present=1").unwrap();
    realm
        .load_module_graph(&root, |_, _| Ok(leaf.clone()))
        .unwrap();
    let imported = realm.import_module_record(&root).unwrap();
    let PromiseState::Rejected(reason) = state(&realm, &imported) else {
        panic!("link rejection")
    };
    let reason = reason.clone();
    expose(&mut realm, "reason", reason);
    check(&mut realm, "reason instanceof SyntaxError");

    realm.eval("let token=Symbol(),effects=0").unwrap();
    let token = realm.eval("token").unwrap();
    let module = realm
        .create_module("effects++;await 0;throw token")
        .unwrap();
    let first = realm.import_module_record(&module).unwrap();
    let second = realm.import_module_record(&module).unwrap();
    for _ in 0..4 {
        next(&mut realm);
    }
    for imported in [&first, &second] {
        assert!(
            matches!(state(&realm, imported), PromiseState::Rejected(reason) if reason.same_value(&token))
        );
    }
    check(&mut realm, "effects===1");
}

#[test]
fn namespace_delivery_ignores_public_promise_constructor_and_then_getters() {
    let mut realm = Realm::default();
    realm.eval("let reads=0;Object.defineProperty(Promise.prototype,'constructor',{get(){reads++;throw 1}});Object.defineProperty(Promise.prototype,'then',{get(){reads++;throw 2}})").unwrap();
    let module = realm.create_module("export let x=1").unwrap();
    let imported = realm.import_module_record(&module).unwrap();
    next(&mut realm);
    assert_eq!(
        value(&realm, &imported),
        realm.module_namespace(&module).unwrap()
    );
    check(&mut realm, "reads===0");
}

#[test]
fn namespace_then_exports_follow_ordinary_promise_resolution_and_assimilation() {
    let mut realm = Realm::default();
    realm.eval("let calls=0,receiver").unwrap();
    let module = realm
        .create_module(
            "export function then(resolve,reject){calls++;receiver=this;resolve(99);reject(100)}",
        )
        .unwrap();
    let imported = realm.import_module_record(&module).unwrap();
    next(&mut realm);
    pending(&realm, &imported);
    check(&mut realm, "calls===0");
    next(&mut realm);
    assert_eq!(value(&realm, &imported), Value::Number(99.0));
    let namespace = realm.module_namespace(&module).unwrap();
    expose(&mut realm, "ns", namespace);
    check(&mut realm, "calls===1&&receiver===ns");

    let module = realm.create_module("export const then=7").unwrap();
    let imported = realm.import_module_record(&module).unwrap();
    next(&mut realm);
    assert_eq!(
        value(&realm, &imported),
        realm.module_namespace(&module).unwrap()
    );
}

#[test]
fn queued_delivery_and_fulfilled_namespace_promises_retain_and_release_module_symbols() {
    let mut realm = Realm::default();
    realm.eval("let weak").unwrap();
    let module = realm
        .create_module("export const token=Symbol();weak=new WeakRef(token)")
        .unwrap();
    let imported = realm.import_module_record(&module).unwrap();
    let imported = realm.root_value(imported, usize::MAX).unwrap();
    drop(module);
    collect(&mut realm);
    check(&mut realm, "weak.deref()!==undefined");
    next(&mut realm);
    collect(&mut realm);
    check(&mut realm, "weak.deref()!==undefined");
    drop(imported);
    collect(&mut realm);
    check(&mut realm, "weak.deref()===undefined");
}

#[test]
fn host_aborts_bypass_rejection_and_repeated_imports_never_replay_aborted_bodies() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(20_000),
        ..Limits::default()
    });
    realm.eval("let effects=0,finalized=0").unwrap();
    let module = realm
        .create_module("effects++;try{while(true){}}finally{finalized++}")
        .unwrap();
    assert!(matches!(
        realm.import_module_record(&module),
        Err(Error::Limit { .. })
    ));
    let imported = realm.import_module_record(&module).unwrap();
    pending(&realm, &imported);
    check(&mut realm, "effects===1&&finalized===0");
    assert!(matches!(
        realm.run_next_promise_job().unwrap(),
        PromiseJobOutcome::NoJob
    ));
}
