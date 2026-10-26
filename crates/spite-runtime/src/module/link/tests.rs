//! Observable linking, ResolveExport and active-component rollback.

use super::*;
use crate::PromiseJobOutcome;
use spite_core::JsString;

fn graph(realm: &mut Realm, sources: &[(&str, &str)]) -> Vec<ModuleHandle> {
    let modules: Vec<_> = sources
        .iter()
        .map(|(_, source)| realm.create_module(source).unwrap())
        .collect();
    realm
        .load_module_graph(&modules[0], |_, request| {
            let index = sources
                .iter()
                .position(|(name, _)| request.specifier == JsString::from(*name))
                .expect("test dependency");
            Ok(modules[index].clone())
        })
        .unwrap();
    modules
}

fn expose(realm: &mut Realm, module: &ModuleHandle, name: &str) {
    let namespace = realm.module_namespace(module).unwrap();
    let global = Value::Object(realm.global_object());
    assert!(
        realm
            .set_property_value(&global, JsString::from(name), namespace, Span::new(0, 0))
            .unwrap()
    );
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
fn named_default_and_quoted_imports_are_live_read_only_and_use_undefined_call_receiver() {
    let mut realm = Realm::default();
    realm.eval("var effects=0").unwrap();
    let modules = graph(
        &mut realm,
        &[
            (
                "root",
                "import d,{x,v,'quoted name' as q,receiver,set}from'leaf';effects++;export function read(){return[d,x,v,q,receiver()]};export function write(){x=99};export function shadow(x){return x};export function indirect(){return eval('x')};export{set}",
            ),
            (
                "leaf",
                "effects++;export let x=1;export var v=2;let quoted=3;export{quoted as 'quoted name'};export default 4;export function receiver(){return this};export function set(value){x=value}",
            ),
        ],
    );
    realm.link_module(&modules[0]).unwrap();
    expose(&mut realm, &modules[0], "ns");
    check(
        &mut realm,
        "effects===0&&ns.shadow(8)===8&&(()=>{try{ns.read()}catch(e){return e instanceof ReferenceError}})()",
    );
    realm.evaluate_module_record(&modules[1]).unwrap();
    check(
        &mut realm,
        "effects===1&&ns.read().slice(0,4).join(',')==='4,1,2,3'&&ns.read()[4]===undefined&&ns.indirect()===1",
    );
    check(
        &mut realm,
        "ns.set(7)===undefined&&ns.read()[1]===7&&ns.indirect()===7&&(()=>{try{ns.write()}catch(e){return e instanceof TypeError}})()",
    );
    assert!(matches!(
        realm.evaluate_module_record(&modules[0]),
        Err(Error::Unsupported { .. })
    ));
    check(&mut realm, "effects===1");
}

#[test]
fn namespace_import_reexports_normalize_to_the_target_namespace_including_self_cycles() {
    let mut realm = Realm::default();
    let modules = graph(
        &mut realm,
        &[
            (
                "root",
                "import{ns}from'bridge';export{ns};export * as direct from'leaf';export function read(){return ns}",
            ),
            ("bridge", "import * as ns from'leaf';export{ns}"),
            (
                "leaf",
                "import * as self from'leaf';export{self};export function f(){return 1}",
            ),
        ],
    );
    realm.link_module(&modules[0]).unwrap();
    expose(&mut realm, &modules[0], "ns");
    check(
        &mut realm,
        "ns.ns===ns.direct&&ns.read()===ns.ns&&ns.ns.self===ns.ns&&ns.ns.f()===1",
    );
}

#[test]
fn star_diamonds_compare_resolved_bindings_and_omit_ambiguous_and_default_names() {
    let mut realm = Realm::default();
    let modules = graph(
        &mut realm,
        &[
            (
                "root",
                "export * from'left';export * from'right';export{default as chosen}from'leaf'",
            ),
            ("left", "export * from'leaf';export const conflict=1"),
            ("right", "export * from'leaf';export const conflict=2"),
            (
                "leaf",
                "export default function(){return 9};export function shared(){return 7};export{shared as '*',shared as '𐀀',shared as ''}",
            ),
        ],
    );
    expose(&mut realm, &modules[0], "ns");
    check(
        &mut realm,
        "Object.keys(ns).join('|')==='*|chosen|shared|𐀀|'&&ns.shared()===7&&ns.chosen()===9&&!('default'in ns)&&!('conflict'in ns)",
    );
}

#[test]
fn missing_ambiguous_and_cyclic_indirect_exports_fail_during_linking() {
    for root in [
        "import{missing}from'leaf'",
        "export{missing}from'leaf'",
        "import{conflict}from'stars'",
        "export{conflict}from'stars'",
        "export{cycle}from'root'",
    ] {
        let mut realm = Realm::default();
        let modules = graph(
            &mut realm,
            &[
                ("root", root),
                ("leaf", "export const present=1"),
                ("stars", "export * from'a';export * from'b'"),
                ("a", "export const conflict=1"),
                ("b", "export const conflict=2"),
            ],
        );
        assert!(
            matches!(
                realm.link_module(&modules[0]),
                Err(Error::Exception {
                    kind: ExceptionKind::SyntaxError,
                    ..
                })
            ),
            "{root}"
        );
        assert_eq!(
            realm
                .objects
                .module_link_state(modules[0].root.handle())
                .unwrap(),
            ModuleLinkState::Unlinked
        );
    }
}

#[test]
fn cycles_hoist_functions_and_keep_imported_var_and_lexical_initialization_distinct() {
    let mut realm = Realm::default();
    let modules = graph(
        &mut realm,
        &[
            (
                "a",
                "import{b,v,x}from'b';export function a(){return b};export function readVar(){return v};export function readLet(){return x}",
            ),
            (
                "b",
                "import{a}from'a';export function b(){return a};export var v;export let x",
            ),
        ],
    );
    realm.link_module(&modules[0]).unwrap();
    expose(&mut realm, &modules[0], "ns");
    check(
        &mut realm,
        "ns.a()()===ns.a&&ns.readVar()===undefined&&(()=>{try{ns.readLet()}catch(e){return e instanceof ReferenceError}})()",
    );
}

#[test]
fn failed_component_rolls_back_ancestors_but_preserves_completed_dependency_identity() {
    let mut realm = Realm::default();
    let modules = graph(
        &mut realm,
        &[
            ("a", "import'b';export{missing}from'c'"),
            ("b", "import'a';export function b(){}"),
            ("c", "export function c(){}"),
        ],
    );
    for _ in 0..2 {
        assert!(matches!(
            realm.link_module(&modules[0]),
            Err(Error::Exception {
                kind: ExceptionKind::SyntaxError,
                ..
            })
        ));
        for module in &modules[..2] {
            assert_eq!(
                realm
                    .objects
                    .module_link_state(module.root.handle())
                    .unwrap(),
                ModuleLinkState::Unlinked
            );
            assert!(
                realm
                    .objects
                    .module_environment(module.root.handle())
                    .unwrap()
                    .is_none()
            );
        }
        assert_eq!(
            realm
                .objects
                .module_link_state(modules[2].root.handle())
                .unwrap(),
            ModuleLinkState::Linked
        );
        expose(&mut realm, &modules[2], "ns");
        realm.eval("globalThis.first??=ns.c").unwrap();
        check(&mut realm, "first===ns.c");
    }
}

#[test]
fn linker_rejects_unloaded_and_foreign_records_and_host_aborts_leave_no_active_component() {
    let mut realm = Realm::default();
    let unloaded = realm.create_module("import'leaf'").unwrap();
    assert!(matches!(
        realm.link_module(&unloaded),
        Err(Error::Unsupported { .. })
    ));
    let foreign = Realm::default().create_module("export let x").unwrap();
    assert!(matches!(
        realm.link_module(&foreign),
        Err(Error::InvalidObject(_))
    ));
    let modules = graph(
        &mut realm,
        &[("root", "import'leaf'"), ("leaf", "export function f(){}")],
    );
    // Exercise aborts at every point through catalog construction, DFS,
    // environment initialization and SCC publication, without raising quotas.
    let mut completed = false;
    for steps in 0..1000 {
        realm.remaining_steps = Some(steps);
        realm.job_active = true;
        let result = realm.link_module(&modules[0]);
        realm.job_active = false;
        if result.is_ok() {
            completed = true;
            break;
        }
        assert!(matches!(result, Err(Error::Limit { .. })), "{result:?}");
        for module in &modules {
            assert_ne!(
                realm
                    .objects
                    .module_link_state(module.root.handle())
                    .unwrap(),
                ModuleLinkState::Linking
            );
        }
    }
    assert!(completed);
}

#[test]
fn imported_records_and_symbols_remain_reachable_through_escaped_functions_and_await_references() {
    let mut realm = Realm::default();
    realm
        .eval("let weak,release;let gate=new Promise(r=>release=r)")
        .unwrap();
    let modules = graph(
        &mut realm,
        &[
            (
                "root",
                "import{x}from'leaf';export function read(){return x};export async function write(){x=await gate}",
            ),
            ("leaf", "export const x=Symbol();weak=new WeakRef(x)"),
        ],
    );
    realm.evaluate_module_record(&modules[1]).unwrap();
    expose(&mut realm, &modules[0], "ns");
    realm.eval("let read=ns.read;let result;ns.write().then(()=>result=false,e=>result=e instanceof TypeError);ns=null").unwrap();
    drop(modules);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(&mut realm, "read()===weak.deref()");
    realm.eval("release(3)").unwrap();
    drain(&mut realm);
    check(&mut realm, "result===true&&typeof read()==='symbol'");
    realm.eval("read=null").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(&mut realm, "weak.deref()===undefined");
}
