//! Ordinary metadata objects, lexical module identity and retained continuations.

use super::*;
use crate::{ExceptionKind, PromiseJobOutcome};
use spite_core::JsString;

fn expose(realm: &mut Realm, module: &ModuleHandle) {
    let namespace = realm.module_namespace(module).unwrap();
    let global = Value::Object(realm.global_object());
    realm
        .set_property_value(&global, JsString::from("ns"), namespace, Span::new(0, 0))
        .unwrap();
}
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

#[test]
fn metadata_is_one_lazy_null_prototype_extensible_ordinary_object_per_record() {
    let mut realm = Realm::default();
    realm.eval("let effects=0").unwrap();
    let module = realm
        .create_module(
            "effects++;export function meta(){return import.meta};globalThis.fromBody=import.meta",
        )
        .unwrap();
    expose(&mut realm, &module);
    check(
        &mut realm,
        "let meta=ns.meta();effects===0&&typeof meta==='object'&&meta!==null&&Object.getPrototypeOf(meta)===null&&Object.isExtensible(meta)&&Reflect.ownKeys(meta).length===0&&meta===ns.meta()",
    );
    realm.evaluate_module_record(&module).unwrap();
    check(&mut realm, "effects===1&&fromBody===meta&&ns.meta()===meta");
    check(
        &mut realm,
        "meta.x=7;let d=Object.getOwnPropertyDescriptor(meta,'x');d.value===7&&d.writable&&d.enumerable&&d.configurable&&delete meta.x&&!('x'in meta)",
    );
    check(
        &mut realm,
        "let proto={inherited:1};Object.setPrototypeOf(meta,proto);meta.inherited===1&&Object.getPrototypeOf(meta)===proto&&Object.freeze(meta)===meta&&Object.isFrozen(meta)&&ns.meta()===meta",
    );
    for expression in ["ns.meta()()", "new (ns.meta())()"] {
        assert!(matches!(
            realm.eval(expression),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    let other = realm
        .create_module("export function meta(){return import.meta}")
        .unwrap();
    expose(&mut realm, &other);
    check(
        &mut realm,
        "ns.meta()!==meta&&Object.getPrototypeOf(ns.meta())===null&&Reflect.ownKeys(ns.meta()).length===0",
    );
}

#[test]
fn imported_functions_and_class_initializers_use_the_defining_module_under_foreign_callers() {
    let mut realm = Realm::default();
    let root = realm.create_module("import{meta,C}from'leaf';export function own(){return import.meta};export function foreign(){return meta()};export function construct(){return new C};export function klass(){return C}").unwrap();
    let leaf = realm.create_module("export function meta(){return import.meta};export class C{field=import.meta;method(){return import.meta};static meta=import.meta}").unwrap();
    realm
        .load_module_graph(&root, |_, _| Ok(leaf.clone()))
        .unwrap();
    realm.evaluate_module_record(&leaf).unwrap();
    expose(&mut realm, &root);
    check(
        &mut realm,
        "let own=ns.own(),foreign=ns.foreign(),instance=ns.construct();own!==foreign&&instance.field===foreign&&instance.method.call({})===foreign&&ns.klass().meta===foreign&&ns.own.call({})===own",
    );
}

#[test]
fn metadata_identity_survives_top_level_await_async_calls_and_both_generator_kinds() {
    let mut realm = Realm::default();
    realm
        .eval("let release;let gate=new Promise(r=>release=r)")
        .unwrap();
    let module=realm.create_module("globalThis.first=import.meta;await gate;globalThis.second=import.meta;export function meta(){return import.meta};export async function f(){await gate;return import.meta};export function* g(){yield import.meta;return import.meta};export async function* ag(){yield import.meta;return import.meta}").unwrap();
    expose(&mut realm, &module);
    realm.evaluate_module_record(&module).unwrap();
    realm.eval("let a,b,c;ns.f().then(v=>a=v);let ag=ns.ag();ag.next().then(v=>b=v.value);ag.next().then(v=>c=v.value);let g=ns.g()").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "first===ns.meta()&&g.next().value===first&&g.next().value===first&&b===first&&c===first&&a===undefined",
    );
    realm.eval("release()").unwrap();
    drain(&mut realm);
    check(&mut realm, "a===first&&second===first&&ns.meta()===first");
}

#[test]
fn escaped_functions_retain_metadata_records_and_symbols_after_a_refused_host_read() {
    let mut realm = Realm::default();
    realm.eval("let weak").unwrap();
    let module=realm.create_module("export function meta(){return import.meta};export function store(){import.meta.held=Symbol();weak=new WeakRef(import.meta.held)}").unwrap();
    realm.link_module(&module).unwrap();
    let environment = realm
        .objects
        .module_environment(module.root.handle())
        .unwrap()
        .unwrap();
    realm.scopes.push(environment);
    realm.job_active = true;
    realm.remaining_steps = Some(0);
    assert!(matches!(
        realm.import_meta_value(Span::new(0, 0)),
        Err(Error::Limit { .. })
    ));
    realm.job_active = false;
    realm.scopes.pop();
    expose(&mut realm, &module);
    realm.eval("ns.store();let get=ns.meta;ns=null").unwrap();
    drop(module);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(&mut realm, "get().held===weak.deref()");
    realm.eval("get=null").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(&mut realm, "weak.deref()===undefined");
}
