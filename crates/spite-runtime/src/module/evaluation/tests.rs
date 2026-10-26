//! Dependency execution, SCC identity and persistent abrupt completions.

use super::*;
use crate::{Limits, PromiseState};
use spite_core::JsString;

fn graph(realm: &mut Realm, sources: &[(&str, &str)]) -> Vec<ModuleHandle> {
    let modules: Vec<_> = sources
        .iter()
        .map(|(_, source)| realm.create_module(source).unwrap())
        .collect();
    realm
        .load_module_graph(&modules[0], |_, request| {
            Ok(modules[sources
                .iter()
                .position(|(name, _)| request.specifier == JsString::from(*name))
                .unwrap()]
            .clone())
        })
        .unwrap();
    modules
}
fn check(realm: &mut Realm, source: &str) {
    assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
}
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
fn fulfilled(realm: &Realm, promise: &Value) {
    assert!(matches!(
        state(realm, promise),
        PromiseState::Fulfilled(Value::Undefined)
    ));
}
fn expose(realm: &mut Realm, module: &ModuleHandle) {
    let namespace = realm.module_namespace(module).unwrap();
    let global = Value::Object(realm.global_object());
    realm
        .set_property_value(&global, JsString::from("ns"), namespace, Span::new(0, 0))
        .unwrap();
}

#[test]
fn shared_dependencies_execute_once_in_depth_first_source_order_with_live_import_cells() {
    let mut realm = Realm::default();
    realm.eval("let events=[]").unwrap();
    let modules = graph(
        &mut realm,
        &[
            (
                "root",
                "import'left';import'right';import{x,set}from'leaf';events.push('root');export function read(){return x};export{set}",
            ),
            ("left", "import'leaf';events.push('left')"),
            ("right", "import'leaf';events.push('right')"),
            (
                "leaf",
                "events.push('leaf');export let x=1;export function set(){x++}",
            ),
        ],
    );
    let promise = realm.evaluate_module_record(&modules[0]).unwrap();
    fulfilled(&realm, &promise);
    expose(&mut realm, &modules[0]);
    check(
        &mut realm,
        "events.join(',')==='leaf,left,right,root'&&ns.read()===1",
    );
    check(&mut realm, "ns.set();ns.read()===2");
    assert_eq!(realm.evaluate_module_record(&modules[0]).unwrap(), promise);
    let leaf = realm.evaluate_module_record(&modules[3]).unwrap();
    fulfilled(&realm, &leaf);
    check(&mut realm, "events.join(',')==='leaf,left,right,root'");
}

#[test]
fn successful_cycles_share_the_cycle_roots_exact_evaluation_promise() {
    let mut realm = Realm::default();
    realm.eval("let events=[]").unwrap();
    let modules = graph(
        &mut realm,
        &[
            (
                "a",
                "import{b}from'b';export var a='a';events.push('a');export function read(){return b}",
            ),
            (
                "b",
                "import{a}from'a';events.push(a===undefined?'b':'wrong');export let b=1",
            ),
        ],
    );
    let promise = realm.evaluate_module_record(&modules[0]).unwrap();
    fulfilled(&realm, &promise);
    assert_eq!(realm.evaluate_module_record(&modules[1]).unwrap(), promise);
    assert_eq!(realm.evaluate_module_record(&modules[0]).unwrap(), promise);
    expose(&mut realm, &modules[0]);
    check(&mut realm, "events.join(',')==='b,a'&&ns.read()===1");
}

#[test]
fn abrupt_cycles_cache_the_same_reason_without_replaying_and_keep_completed_dependencies() {
    let mut realm = Realm::default();
    realm.eval("let events=[],reason=Symbol()").unwrap();
    let reason = realm.eval("reason").unwrap();
    let modules = graph(
        &mut realm,
        &[
            ("a", "import'b';import'c';events.push('a');throw reason"),
            ("b", "import'a';events.push('b')"),
            ("c", "events.push('c');export const value=1"),
        ],
    );
    let a = realm.evaluate_module_record(&modules[0]).unwrap();
    assert!(matches!(state(&realm,&a),PromiseState::Rejected(value) if value==&reason));
    let b = realm.evaluate_module_record(&modules[1]).unwrap();
    assert_ne!(a, b);
    assert!(matches!(state(&realm,&b),PromiseState::Rejected(value) if value==&reason));
    assert_eq!(realm.evaluate_module_record(&modules[0]).unwrap(), a);
    assert_eq!(realm.evaluate_module_record(&modules[1]).unwrap(), b);
    let c = realm.evaluate_module_record(&modules[2]).unwrap();
    fulfilled(&realm, &c);
    check(&mut realm, "events.join(',')==='b,c,a'");
}

#[test]
fn failed_dependencies_stop_later_branches_and_parent_bodies_and_preserve_error_object_identity() {
    let mut realm = Realm::default();
    realm.eval("let events=[]").unwrap();
    let modules = graph(
        &mut realm,
        &[
            ("root", "import'failed';import'later';events.push('root')"),
            (
                "failed",
                "events.push('failed');throw new TypeError('sentinel')",
            ),
            ("later", "events.push('later')"),
        ],
    );
    let root = realm.evaluate_module_record(&modules[0]).unwrap();
    let failed = realm.evaluate_module_record(&modules[1]).unwrap();
    let PromiseState::Rejected(reason) = state(&realm, &root) else {
        panic!("rejected root")
    };
    assert!(matches!(state(&realm,&failed),PromiseState::Rejected(value) if value==reason));
    check(&mut realm, "events.join(',')==='failed'");
    let later = realm.evaluate_module_record(&modules[2]).unwrap();
    fulfilled(&realm, &later);
    check(&mut realm, "events.join(',')==='failed,later'");
}

#[test]
fn host_aborts_leave_pending_promises_skip_catches_and_finally_and_never_replay_dependencies() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(20_000),
        ..Limits::default()
    });
    realm.eval("let events=[]").unwrap();
    let modules = graph(
        &mut realm,
        &[
            (
                "root",
                "import'leaf';try{while(true){}}catch(e){events.push('catch')}finally{events.push('finally')}",
            ),
            ("leaf", "events.push('leaf')"),
        ],
    );
    assert!(matches!(
        realm.evaluate_module_record(&modules[0]),
        Err(Error::Limit { .. })
    ));
    let pending = realm.evaluate_module_record(&modules[0]).unwrap();
    assert!(matches!(state(&realm, &pending), PromiseState::Pending));
    assert_eq!(realm.evaluate_module_record(&modules[0]).unwrap(), pending);
    let leaf = realm.evaluate_module_record(&modules[1]).unwrap();
    fulfilled(&realm, &leaf);
    let parent = realm
        .create_module("import'root';events.push('parent')")
        .unwrap();
    realm
        .load_module_graph(&parent, |_, _| Ok(modules[0].clone()))
        .unwrap();
    assert!(matches!(
        realm.evaluate_module_record(&parent),
        Err(Error::Limit { .. })
    ));
    let parent = realm.evaluate_module_record(&parent).unwrap();
    assert!(matches!(state(&realm, &parent), PromiseState::Pending));
    check(&mut realm, "events.join(',')==='leaf'");
}

#[test]
fn asynchronous_graphs_are_refused_before_body_entry_even_for_unreachable_awaits() {
    let mut realm = Realm::default();
    realm.eval("let effects=0").unwrap();
    let modules = graph(
        &mut realm,
        &[
            ("root", "import'leaf';effects++"),
            ("leaf", "effects++;if(false){await 0}"),
        ],
    );
    assert!(matches!(
        realm.evaluate_module_record(&modules[0]),
        Err(Error::Unsupported { .. })
    ));
    check(&mut realm, "effects===0");
    assert!(
        realm
            .objects
            .module_execution(modules[0].root.handle())
            .unwrap()
            .is_none()
    );
    assert!(
        realm
            .objects
            .module_execution(modules[1].root.handle())
            .unwrap()
            .is_none()
    );
}

#[test]
fn record_and_promise_roots_retain_symbol_rejection_reasons_and_release_graph_cycles() {
    let mut realm = Realm::default();
    realm.eval("let weak").unwrap();
    let modules = graph(
        &mut realm,
        &[
            ("a", "import'b';export function f(){}"),
            (
                "b",
                "import'a';let reason=Symbol();weak=new WeakRef(reason);throw reason",
            ),
        ],
    );
    let promise = realm.evaluate_module_record(&modules[0]).unwrap();
    let promise = realm.root_value(promise, usize::MAX).unwrap();
    drop(modules);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(&mut realm, "weak.deref()!==undefined");
    drop(promise);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(&mut realm, "weak.deref()===undefined");
}

#[test]
fn deep_dependency_graphs_evaluate_without_recursive_module_traversal() {
    let mut realm = Realm::default();
    realm.eval("let effects=0").unwrap();
    let mut modules = Vec::new();
    for index in 0..600 {
        modules.push(
            realm
                .create_module(&if index == 599 {
                    "effects++".into()
                } else {
                    format!("import'{}';effects++", index + 1)
                })
                .unwrap(),
        );
    }
    realm
        .load_module_graph(&modules[0], |_, request| {
            Ok(modules[String::from_utf16(request.specifier.code_units())
                .unwrap()
                .parse::<usize>()
                .unwrap()]
            .clone())
        })
        .unwrap();
    let promise = realm.evaluate_module_record(&modules[0]).unwrap();
    fulfilled(&realm, &promise);
    check(&mut realm, "effects===600");
}

#[test]
fn selected_work_refusals_release_active_states_and_cached_bodies_never_replay() {
    let mut realm = Realm::default();
    realm.eval("let events=[]").unwrap();
    for steps in [0, 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 999] {
        realm.eval("events=[]").unwrap();
        let modules = graph(
            &mut realm,
            &[
                ("root", "import'leaf';events.push('root')"),
                ("leaf", "events.push('leaf')"),
            ],
        );
        realm.link_module(&modules[0]).unwrap();
        realm.remaining_steps = Some(steps);
        realm.job_active = true;
        let result = realm.evaluate_module_record(&modules[0]);
        realm.job_active = false;
        if let Ok(promise) = &result {
            fulfilled(&realm, promise);
            continue;
        }
        assert!(
            matches!(result, Err(Error::Limit { .. })),
            "{steps}: {result:?}"
        );
        for module in &modules {
            assert!(
                realm
                    .objects
                    .module_execution(module.root.handle())
                    .unwrap()
                    .is_none_or(|execution| execution.status != ModuleExecutionStatus::Evaluating),
                "{steps}"
            );
        }
        if let Some(promise) = realm
            .objects
            .module_evaluation(modules[0].root.handle())
            .unwrap()
        {
            let before = realm.eval("events.join(',')").unwrap();
            assert_eq!(realm.evaluate_module_record(&modules[0]).unwrap(), promise);
            assert!(matches!(state(&realm, &promise), PromiseState::Pending));
            assert_eq!(realm.eval("events.join(',')").unwrap(), before);
        }
        drop(modules);
        realm.clear_kept_objects().unwrap();
        realm.collect(usize::MAX).unwrap();
    }
}
