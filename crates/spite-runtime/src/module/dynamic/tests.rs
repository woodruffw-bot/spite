//! Embedding requests, deferred import jobs and defining ScriptOrModule identity.

use super::*;
use crate::{PromiseJobOutcome, PromiseState};
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
fn fulfilled(realm: &Realm, promise: &Value) -> Value {
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
fn host_completion_defers_linking_and_body_entry_and_imports_keep_fresh_promises() {
    let mut realm = Realm::default();
    realm.enable_module_imports();
    realm
        .eval("let events=[];Promise.resolve().then(()=>events.push('before'))")
        .unwrap();
    let imported = realm.eval("import('target')").unwrap();
    let request = realm.take_module_import_request().unwrap();
    assert_eq!(request.request().specifier, JsString::from("target"));
    assert!(request.referrer().is_none());
    let module = realm
        .create_module("events.push('body');export const x=42")
        .unwrap();
    realm
        .finish_module_import(request, Ok(module.clone()))
        .unwrap();
    pending(&realm, &imported);
    check(&mut realm, "events.length===0");
    next(&mut realm);
    check(&mut realm, "events.join(',')==='before'");
    next(&mut realm);
    pending(&realm, &imported);
    check(&mut realm, "events.join(',')==='before,body'");
    next(&mut realm);
    let namespace = fulfilled(&realm, &imported);
    assert_eq!(namespace, realm.module_namespace(&module).unwrap());
    let again = realm.eval("import('target')").unwrap();
    assert_ne!(again, imported);
    let request = realm.take_module_import_request().unwrap();
    realm.finish_module_import(request, Ok(module)).unwrap();
    next(&mut realm);
    next(&mut realm);
    assert_eq!(fulfilled(&realm, &again), namespace);
    check(&mut realm, "events.join(',')==='before,body'");
}

#[test]
fn enabling_is_idempotent_and_requests_preserve_fifo_and_canonical_attributes() {
    let mut realm = Realm::default();
    realm.enable_module_imports();
    realm
        .eval("import('first',{with:{z:'last',a:'first'}})")
        .unwrap();
    realm.enable_module_imports();
    realm.eval("import('second')").unwrap();
    let first = realm.take_module_import_request().unwrap();
    let second = realm.take_module_import_request().unwrap();
    assert_eq!(first.request().specifier, JsString::from("first"));
    assert_eq!(
        first
            .request()
            .attributes
            .iter()
            .map(|a| (a.key.to_utf8().unwrap(), a.value.to_utf8().unwrap()))
            .collect::<Vec<_>>(),
        [("a".into(), "first".into()), ("z".into(), "last".into())]
    );
    assert_eq!(second.request().specifier, JsString::from("second"));
    assert!(realm.take_module_import_request().is_none());
}

#[test]
fn operand_conversion_and_attribute_getters_finish_before_type_validation_or_host_requests() {
    let mut realm = Realm::default();
    realm.enable_module_imports();
    let imported = realm.eval("let log='';import({toString(){log+='s';return'x'}},{get with(){log+='w';return{get z(){log+='z';return'last'},get a(){log+='a';return'first'}}}})").unwrap();
    pending(&realm, &imported);
    check(&mut realm, "log==='swza'");
    let request = realm.take_module_import_request().unwrap();
    assert_eq!(request.request().attributes[0].key, JsString::from("a"));
    let imported = realm.eval("log='';import('bad',{with:{get a(){log+='a';return 7},get b(){log+='b';return'good'}}})").unwrap();
    let PromiseState::Rejected(reason) = state(&realm, &imported) else {
        panic!("type rejection")
    };
    let reason = reason.clone();
    expose(&mut realm, "reason", reason);
    check(&mut realm, "log==='ab'&&reason instanceof TypeError");
    assert!(realm.take_module_import_request().is_none());
}

#[test]
fn module_referrers_survive_foreign_callers_dynamic_functions_eval_fields_and_suspension() {
    let mut realm = Realm::default();
    realm.enable_module_imports();
    let module = realm
        .create_module(
            r#"
        export function own(){return import('x')}
        export const arrow=()=>import('x');
        export const method={m(){return import('x')}};
        export const constructed=Function("return import('x')");
        export const indirect=(0,eval)("(function(){return import('x')})");
        export function* g(){yield import('x')}
        export async function af(){await 0;return import('x')}
        export async function* ag(){yield import('x')}
        export function fails(){throw 1}
        export class C{x=import('x')}
    "#,
        )
        .unwrap();
    realm.evaluate_module_record(&module).unwrap();
    let namespace = realm.module_namespace(&module).unwrap();
    expose(&mut realm, "ns", namespace);
    for call in [
        "ns.own()",
        "ns.arrow()",
        "ns.method.m()",
        "ns.constructed()",
        "ns.indirect()",
        "ns.g().next()",
        "new ns.C()",
        "ns.ag().next()",
    ] {
        realm.eval(call).unwrap();
        let request = realm.take_module_import_request().unwrap();
        assert_eq!(request.referrer(), Some(&module), "{call}");
    }
    realm.eval("ns.af()").unwrap();
    next(&mut realm);
    let request = realm.take_module_import_request().unwrap();
    assert_eq!(request.referrer(), Some(&module));
    realm
        .eval("try{ns.fails()}catch{};import('script')")
        .unwrap();
    assert!(
        realm
            .take_module_import_request()
            .unwrap()
            .referrer()
            .is_none()
    );
    realm
        .eval("globalThis.fromScript=()=>import('script')")
        .unwrap();
    let caller = realm.create_module("fromScript()").unwrap();
    realm.evaluate_module_record(&caller).unwrap();
    assert!(
        realm
            .take_module_import_request()
            .unwrap()
            .referrer()
            .is_none()
    );
}

#[test]
fn queued_builtin_eval_reactions_restore_the_enqueuing_modules_identity() {
    let mut realm = Realm::default();
    realm.enable_module_imports();
    let module = realm
        .create_module(r#"Promise.resolve("import('x')").then(eval)"#)
        .unwrap();
    realm.evaluate_module_record(&module).unwrap();
    next(&mut realm);
    assert_eq!(
        realm.take_module_import_request().unwrap().referrer(),
        Some(&module)
    );
}

#[test]
fn global_environment_dynamic_functions_retain_their_defining_module_and_release_it() {
    let mut realm = Realm::default();
    realm.enable_module_imports();
    realm.eval("let weak").unwrap();
    let module = realm.create_module("export const token=Symbol();weak=new WeakRef(token);export const f=Function(\"return import('x')\")").unwrap();
    realm.evaluate_module_record(&module).unwrap();
    let namespace = realm.module_namespace(&module).unwrap();
    let function = realm
        .get_property_value(&namespace, &JsString::from("f"), Span::new(0, 0))
        .unwrap();
    let function = realm.root_value(function, usize::MAX).unwrap();
    drop(module);
    collect(&mut realm);
    check(&mut realm, "weak.deref()!==undefined");
    realm
        .call(
            function.value().clone(),
            Value::Undefined,
            vec![],
            Span::new(0, 0),
        )
        .unwrap();
    let request = realm.take_module_import_request().unwrap();
    assert!(request.referrer().is_some());
    drop(request);
    drop(function);
    collect(&mut realm);
    check(&mut realm, "weak.deref()===undefined");
}

#[test]
fn host_rejection_preserves_the_original_reason_and_delivers_handlers_as_jobs() {
    let mut realm = Realm::default();
    realm.enable_module_imports();
    let imported = realm.eval("let token=Symbol(),caught=false;let p=import('missing');p.catch(e=>{caught=e===token});p").unwrap();
    let token = realm.eval("token").unwrap();
    let request = realm.take_module_import_request().unwrap();
    realm
        .finish_module_import(request, Err(token.clone()))
        .unwrap();
    assert!(
        matches!(state(&realm, &imported), PromiseState::Rejected(reason) if reason.same_value(&token))
    );
    check(&mut realm, "caught===false");
    next(&mut realm);
    check(&mut realm, "caught===true");
}

#[test]
fn linking_rejections_and_async_graphs_use_the_deferred_import_continuation() {
    let mut realm = Realm::default();
    realm.enable_module_imports();
    let imported = realm.eval("import('bad')").unwrap();
    let root = realm.create_module("import{missing}from'leaf'").unwrap();
    let leaf = realm.create_module("export const present=1").unwrap();
    realm
        .load_module_graph(&root, |_, _| Ok(leaf.clone()))
        .unwrap();
    let request = realm.take_module_import_request().unwrap();
    realm.finish_module_import(request, Ok(root)).unwrap();
    pending(&realm, &imported);
    next(&mut realm);
    let PromiseState::Rejected(reason) = state(&realm, &imported) else {
        panic!("link rejection")
    };
    let reason = reason.clone();
    expose(&mut realm, "reason", reason);
    check(&mut realm, "reason instanceof SyntaxError");

    let imported = realm.eval("import('async')").unwrap();
    let root = realm.create_module("export{x}from'leaf'").unwrap();
    let leaf = realm.create_module("await 0;export const x=42").unwrap();
    realm
        .load_module_graph(&root, |_, _| Ok(leaf.clone()))
        .unwrap();
    let request = realm.take_module_import_request().unwrap();
    realm.finish_module_import(request, Ok(root)).unwrap();
    for _ in 0..3 {
        next(&mut realm);
        pending(&realm, &imported);
    }
    next(&mut realm);
    let namespace = fulfilled(&realm, &imported);
    expose(&mut realm, "ns", namespace);
    check(&mut realm, "ns.x===42");
}

#[test]
fn foreign_completion_handles_are_host_failures_without_language_rejection() {
    let mut realm = Realm::default();
    realm.enable_module_imports();
    let imported = realm.eval("import('x')").unwrap();
    let request = realm.take_module_import_request().unwrap();
    let foreign = Realm::default().create_module("export const x=1").unwrap();
    assert!(matches!(
        realm.finish_module_import(request, Ok(foreign)),
        Err(Error::InvalidObject(_))
    ));
    pending(&realm, &imported);
}

#[test]
fn requests_and_queued_import_jobs_root_capabilities_and_eventual_namespace_symbols() {
    let mut realm = Realm::default();
    realm.enable_module_imports();
    realm.eval("let weak").unwrap();
    let imported = realm.eval("import('x')").unwrap();
    let request = realm.take_module_import_request().unwrap();
    collect(&mut realm);
    pending(&realm, &imported);
    let imported = realm.root_value(imported, usize::MAX).unwrap();
    let module = realm
        .create_module("export const token=Symbol();weak=new WeakRef(token)")
        .unwrap();
    realm.finish_module_import(request, Ok(module)).unwrap();
    next(&mut realm);
    collect(&mut realm);
    check(&mut realm, "weak.deref()!==undefined");
    next(&mut realm);
    collect(&mut realm);
    check(&mut realm, "weak.deref()!==undefined");
    drop(imported);
    collect(&mut realm);
    check(&mut realm, "weak.deref()===undefined");
}
