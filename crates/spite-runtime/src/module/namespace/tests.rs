//! Namespace exotic behavior through ordinary JavaScript reflection and Proxies.

use super::*;
use crate::{ExceptionKind, PromiseJobOutcome, PromiseState};

fn setup(source: &str) -> (Realm, ModuleHandle, Value) {
    let mut realm = Realm::default();
    realm.eval("var effects=0").unwrap();
    let module = realm.create_module(source).unwrap();
    let namespace = realm.module_namespace(&module).unwrap();
    let global = Value::Object(realm.global_object());
    assert!(
        realm
            .set_property_value(
                &global,
                JsString::from("ns"),
                namespace.clone(),
                Span::new(0, 0)
            )
            .unwrap()
    );
    (realm, module, namespace)
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
fn collect(realm: &mut Realm) {
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn namespace_preparation_keeps_tdz_and_hoists_functions_without_body_entry_or_reinitialization() {
    let (mut realm, module, namespace) = setup(
        "effects++;export var v=1;export let x=2;export function f(){return x};export default function(){return 3}",
    );
    check(
        &mut realm,
        "effects===0&&ns.v===undefined&&ns.f.name==='f'&&ns.default.name==='default'&&Reflect.has(ns,'x')&&!Reflect.has(ns,'missing')",
    );
    check(
        &mut realm,
        "(()=>{try{ns.x}catch(e){return e instanceof ReferenceError}})()&&(()=>{try{Object.hasOwn(ns,'x')}catch(e){return e instanceof ReferenceError}})()",
    );
    realm.eval("let firstDefault=ns.default").unwrap();
    assert_eq!(realm.module_namespace(&module.clone()), Ok(namespace));
    realm.evaluate_module_record(&module).unwrap();
    check(
        &mut realm,
        "effects===1&&ns.v===1&&ns.x===2&&ns.f()===2&&ns.default===firstDefault&&ns.default()===3",
    );
}

#[test]
fn export_keys_use_utf16_lexical_order_including_numeric_names_before_the_symbol_tag() {
    let (mut realm, module, _) = setup(
        "let value=7;export{value as '2',value as '10',value as '𐀀',value as '',value as '__proto__',value as a}",
    );
    check(
        &mut realm,
        "let keys=Reflect.ownKeys(ns);keys.length===7&&keys.slice(0,-1).join('|')==='10|2|__proto__|a|𐀀|'&&keys[6]===Symbol.toStringTag",
    );
    realm.evaluate_module_record(&module).unwrap();
    check(
        &mut realm,
        "Object.keys(ns).join('|')==='10|2|__proto__|a|𐀀|'&&Object.values(ns).every(v=>v===7)&&Object.entries(ns).length===6&&ns.__proto__===7",
    );
}

#[test]
fn descriptors_remain_live_writable_data_descriptions_while_all_assignment_receivers_are_rejected()
{
    let (mut realm, module, _) = setup("export let x=1;globalThis.change=value=>x=value");
    realm.evaluate_module_record(&module).unwrap();
    check(
        &mut realm,
        "let d=Object.getOwnPropertyDescriptor(ns,'x');d.value===1&&d.writable&&d.enumerable&&!d.configurable&&!('get'in d)&&!Reflect.set(ns,'x',1)",
    );
    check(
        &mut realm,
        "change(2)===2&&ns.x===2&&Object.getOwnPropertyDescriptor(ns,'x').value===2",
    );
    check(
        &mut realm,
        "let calls=0,receiver={set x(v){calls++}};!Reflect.set(ns,'x',3,receiver)&&!Reflect.set(ns,'missing',3,receiver)&&calls===0&&ns.x===2",
    );
    check(
        &mut realm,
        "(()=>{'use strict';try{ns.x=4}catch(e){return e instanceof TypeError}})()",
    );
}

#[test]
fn defining_exports_accepts_only_compatible_attributes_and_same_value() {
    let (mut realm, module, _) = setup("export let x=NaN;export const minus=-0");
    realm.evaluate_module_record(&module).unwrap();
    check(
        &mut realm,
        "Reflect.defineProperty(ns,'x',{})&&Reflect.defineProperty(ns,'x',{value:NaN,writable:true,enumerable:true,configurable:false})&&!Reflect.defineProperty(ns,'x',{value:0})&&!Reflect.defineProperty(ns,'x',{writable:false})&&!Reflect.defineProperty(ns,'x',{enumerable:false})&&!Reflect.defineProperty(ns,'x',{configurable:true})&&!Reflect.defineProperty(ns,'x',{get:undefined})",
    );
    check(
        &mut realm,
        "Reflect.defineProperty(ns,'minus',{value:-0})&&!Reflect.defineProperty(ns,'minus',{value:0})&&!Reflect.defineProperty(ns,'missing',{})&&!Reflect.defineProperty(ns,Symbol(),{value:1})&&Reflect.defineProperty(ns,Symbol.toStringTag,{value:'Module'})&&!Reflect.defineProperty(ns,Symbol.toStringTag,{value:'Other'})",
    );
}

#[test]
fn tdz_precedes_descriptor_validation_but_presence_set_and_delete_do_not_read_cells() {
    let (mut realm, _module, _) = setup("export let x=1");
    check(
        &mut realm,
        "'x'in ns&&!Reflect.set(ns,'x',2)&&!Reflect.deleteProperty(ns,'x')&&Reflect.deleteProperty(ns,'missing')&&!Reflect.defineProperty(ns,'missing',{configurable:true})",
    );
    for expression in [
        "Object.getOwnPropertyDescriptor(ns,'x')",
        "Reflect.defineProperty(ns,'x',{configurable:true})",
        "new Proxy(ns,{}).x",
        "new Proxy(ns,{get(){return 7}}).x",
    ] {
        assert!(
            matches!(
                realm.eval(expression),
                Err(Error::Exception {
                    kind: ExceptionKind::ReferenceError,
                    ..
                })
            ),
            "{expression}"
        );
    }
}

#[test]
fn namespace_prototype_extensibility_sealing_freezing_and_tag_follow_exotic_rules() {
    let (mut realm, module, _) = setup("export const x=1");
    realm.evaluate_module_record(&module).unwrap();
    check(
        &mut realm,
        "Object.getPrototypeOf(ns)===null&&!Object.isExtensible(ns)&&Object.preventExtensions(ns)===ns&&Reflect.setPrototypeOf(ns,null)&&!Reflect.setPrototypeOf(ns,{})&&Object.prototype.toString.call(ns)==='[object Module]'",
    );
    check(
        &mut realm,
        "let tag=Object.getOwnPropertyDescriptor(ns,Symbol.toStringTag);tag.value==='Module'&&!tag.writable&&!tag.enumerable&&!tag.configurable&&Object.isSealed(ns)&&!Object.isFrozen(ns)&&Object.seal(ns)===ns",
    );
    check(
        &mut realm,
        "(()=>{try{Object.freeze(ns)}catch(e){return e instanceof TypeError}})()",
    );
    let empty = realm.create_module("export {}").unwrap();
    let namespace = realm.module_namespace(&empty).unwrap();
    let global = Value::Object(realm.global_object());
    realm
        .set_property_value(&global, JsString::from("empty"), namespace, Span::new(0, 0))
        .unwrap();
    check(
        &mut realm,
        "Object.isFrozen(empty)&&Object.freeze(empty)===empty&&Reflect.ownKeys(empty).length===1",
    );
}

#[test]
fn proxy_forwarding_and_invariants_observe_dynamic_namespace_descriptors() {
    let (mut realm, module, _) = setup("export let x=1");
    realm.evaluate_module_record(&module).unwrap();
    check(
        &mut realm,
        "let p=new Proxy(ns,{});p.x===1&&!Reflect.set(p,'x',2)&&Object.getOwnPropertyDescriptor(p,'x').writable&&Reflect.defineProperty(p,'x',{value:1})&&!Reflect.defineProperty(p,'x',{value:2})&&Reflect.ownKeys(p).length===2",
    );
    check(
        &mut realm,
        "Reflect.set(new Proxy(ns,{set(){return true}}),'x',7)&&ns.x===1&&new Proxy(ns,{get(){return 9}}).x===9",
    );
    for expression in [
        "Reflect.deleteProperty(new Proxy(ns,{deleteProperty(){return true}}),'x')",
        "Reflect.ownKeys(new Proxy(ns,{ownKeys(){return []}}))",
        "new Proxy(ns,{get(){return 'Other'}})[Symbol.toStringTag]",
        "Object.getOwnPropertyDescriptor(new Proxy(ns,{getOwnPropertyDescriptor(){return{value:1,writable:false,enumerable:true,configurable:false}}}),'x')",
    ] {
        assert!(
            matches!(
                realm.eval(expression),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{expression}"
        );
    }
}

#[test]
fn namespace_reads_stay_live_across_await_and_namespace_roots_retain_record_cycles() {
    let (mut realm, module, namespace) =
        setup("export let x=1;export const held=Symbol();weak=new WeakRef(held);await gate;x=2");
    realm
        .eval("let weak,release;let gate=new Promise(r=>release=r)")
        .unwrap();
    let promise = realm.evaluate_module_record(&module).unwrap();
    check(&mut realm, "ns.x===1");
    realm.eval("release()").unwrap();
    drain(&mut realm);
    check(&mut realm, "ns.x===2");
    let Value::Object(promise) = promise else {
        panic!("Promise")
    };
    assert!(matches!(
        realm.inspect_object(&promise).unwrap().promise_state(),
        Some(PromiseState::Fulfilled(_))
    ));
    let namespace = realm.root_value(namespace, usize::MAX).unwrap();
    drop(module);
    realm.eval("globalThis.ns=null").unwrap();
    collect(&mut realm);
    check(&mut realm, "weak.deref()!==undefined");
    drop(namespace);
    collect(&mut realm);
    check(&mut realm, "weak.deref()===undefined");
}

#[test]
fn namespace_and_evaluation_host_calls_reset_opted_in_work_without_resetting_active_jobs() {
    let mut realm = Realm::new(crate::Limits {
        max_steps: Some(20_000),
        ..crate::Limits::default()
    });
    let module = realm.create_module("export let x=1").unwrap();
    let namespace = realm.module_namespace(&module).unwrap();
    assert!(matches!(
        realm.eval("while(true){}"),
        Err(Error::Limit { .. })
    ));
    let promise = realm.evaluate_module_record(&module).unwrap();
    let Value::Object(promise) = promise else {
        panic!("Promise")
    };
    assert!(matches!(
        realm.inspect_object(&promise).unwrap().promise_state(),
        Some(PromiseState::Fulfilled(_))
    ));
    assert_eq!(
        realm.read_property(&namespace, &JsString::from("x")),
        Ok(Value::Number(1.0))
    );
    let other = realm.create_module("export let y=2").unwrap();
    realm.evaluate_module_record(&other).unwrap();
    assert!(matches!(
        realm.eval("while(true){}"),
        Err(Error::Limit { .. })
    ));
    assert!(realm.module_namespace(&other).is_ok());
    let waiting = realm.create_module("export let z=3").unwrap();
    realm.module_namespace(&waiting).unwrap();
    realm.remaining_steps = Some(0);
    realm.job_active = true;
    assert!(matches!(
        realm.evaluate_module_record(&waiting),
        Err(Error::Limit { .. })
    ));
    realm.job_active = false;
}
