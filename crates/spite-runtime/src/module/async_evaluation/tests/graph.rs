//! Async dependencies, cycles, ready ordering and host aborts.

use super::*;
use crate::ModuleHandle;
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

fn drain(realm: &mut Realm) {
    for _ in 0..2000 {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => return,
            PromiseJobOutcome::Completed => (),
            outcome => panic!("{outcome:?}"),
        }
    }
    panic!("unexpectedly unbounded job queue");
}

fn rejected(realm: &Realm, promise: &Value, reason: &Value) {
    assert!(
        matches!(state(realm, promise), PromiseState::Rejected(value) if value.same_value(reason))
    );
}

#[test]
fn suspended_dependencies_do_not_block_siblings_and_parents_read_initialized_live_bindings() {
    let mut realm = Realm::default();
    realm
        .eval("let release;let gate=new Promise(r=>release=r);let events=[]")
        .unwrap();
    let modules = graph(
        &mut realm,
        &[
            (
                "root",
                "import {n} from'leaf';import {snapshot} from'sibling';events.push(snapshot+':root'+n)",
            ),
            (
                "leaf",
                "events.push('start');await gate;events.push('end');export let n=1",
            ),
            (
                "sibling",
                "events.push('sibling');export const snapshot=events.join(',')",
            ),
        ],
    );
    let root = realm.evaluate_module_record(&modules[0]).unwrap();
    let leaf = realm.evaluate_module_record(&modules[1]).unwrap();
    pending(&realm, &root);
    pending(&realm, &leaf);
    check(&mut realm, "events.join(',')==='start,sibling'");
    assert!(matches!(
        realm.run_next_promise_job().unwrap(),
        PromiseJobOutcome::NoJob
    ));
    realm.eval("release()").unwrap();
    next(&mut realm);
    pending(&realm, &root);
    check(&mut realm, "events.join(',')==='start,sibling,end'");
    next(&mut realm);
    fulfilled(&realm, &root);
    fulfilled(&realm, &leaf);
    assert_ne!(root, leaf);
    check(
        &mut realm,
        "events.join(',')==='start,sibling,end,start,sibling:root1'",
    );
}

#[test]
fn shared_diamonds_execute_ready_ancestors_in_async_evaluation_order() {
    let mut realm = Realm::default();
    realm.eval("let events=[]").unwrap();
    let modules = graph(
        &mut realm,
        &[
            (
                "root",
                "import'left';import'right';import'indirect';events.push('root')",
            ),
            ("left", "import'leaf';events.push('left')"),
            ("right", "import'leaf';events.push('right')"),
            (
                "indirect",
                "import'left';import'right';events.push('indirect')",
            ),
            ("leaf", "await 0;events.push('leaf')"),
        ],
    );
    let root = realm.evaluate_module_record(&modules[0]).unwrap();
    pending(&realm, &root);
    drain(&mut realm);
    fulfilled(&realm, &root);
    check(
        &mut realm,
        "events.join(',')==='leaf,left,right,indirect,root'",
    );
}

#[test]
fn importers_of_an_async_cycle_leaf_wait_for_the_cycle_root() {
    let mut realm = Realm::default();
    realm.eval("let events=[]").unwrap();
    let modules = graph(
        &mut realm,
        &[
            (
                "entry",
                "import'cycle';import'importer';events.push('entry')",
            ),
            (
                "cycle",
                "import'leaf';events.push('cycle-start');await 0;events.push('cycle-end')",
            ),
            (
                "leaf",
                "import'cycle';events.push('leaf-start');await 0;events.push('leaf-end')",
            ),
            ("importer", "import'leaf';events.push('importer')"),
        ],
    );
    let entry = realm.evaluate_module_record(&modules[0]).unwrap();
    let cycle = realm.evaluate_module_record(&modules[1]).unwrap();
    assert_eq!(realm.evaluate_module_record(&modules[2]).unwrap(), cycle);
    check(&mut realm, "events.join(',')==='leaf-start'");
    next(&mut realm);
    next(&mut realm);
    pending(&realm, &entry);
    check(
        &mut realm,
        "events.join(',')==='leaf-start,leaf-end,cycle-start'",
    );
    next(&mut realm);
    pending(&realm, &entry);
    next(&mut realm);
    fulfilled(&realm, &entry);
    fulfilled(&realm, &cycle);
    check(
        &mut realm,
        "events.join(',')==='leaf-start,leaf-end,cycle-start,cycle-end,importer,entry'",
    );
}

#[test]
fn rejected_dependencies_propagate_the_original_reason_without_running_parent_bodies() {
    let mut realm = Realm::default();
    realm.eval("let reason=Symbol();let events=[]").unwrap();
    let reason = realm.eval("reason").unwrap();
    let modules = graph(
        &mut realm,
        &[
            ("root", "import'leaf';import'sibling';events.push('root')"),
            ("leaf", "await 0;throw reason"),
            ("sibling", "events.push('sibling')"),
        ],
    );
    let root = realm.evaluate_module_record(&modules[0]).unwrap();
    let leaf = realm.evaluate_module_record(&modules[1]).unwrap();
    next(&mut realm);
    pending(&realm, &root);
    next(&mut realm);
    rejected(&realm, &root, &reason);
    rejected(&realm, &leaf, &reason);
    assert_eq!(realm.evaluate_module_record(&modules[0]).unwrap(), root);
    let later = realm
        .create_module("import'leaf';events.push('later')")
        .unwrap();
    realm
        .load_module_graph(&later, |_, _| Ok(modules[1].clone()))
        .unwrap();
    let later = realm.evaluate_module_record(&later).unwrap();
    rejected(&realm, &later, &reason);
    check(&mut realm, "events.join(',')==='sibling'");
}

#[test]
fn abrupt_synchronous_ancestors_skip_their_parents_but_other_ready_branches_still_execute() {
    let mut realm = Realm::default();
    realm.eval("let reason=Symbol();let events=[]").unwrap();
    let reason = realm.eval("reason").unwrap();
    let modules = graph(
        &mut realm,
        &[
            ("root", "import'left';import'right';events.push('root')"),
            ("left", "import'leaf';events.push('left');throw reason"),
            ("right", "import'leaf';events.push('right')"),
            ("leaf", "await 0;events.push('leaf')"),
        ],
    );
    let root = realm.evaluate_module_record(&modules[0]).unwrap();
    let right = realm.evaluate_module_record(&modules[2]).unwrap();
    drain(&mut realm);
    rejected(&realm, &root, &reason);
    fulfilled(&realm, &right);
    check(&mut realm, "events.join(',')==='leaf,left,right'");
}

#[test]
fn independent_host_evaluations_share_pending_dependencies_and_preserve_completion_order() {
    let mut realm = Realm::default();
    realm.eval("let events=[]").unwrap();
    let modules = graph(
        &mut realm,
        &[
            ("b", "import'leaf';events.push('b')"),
            ("leaf", "events.push('start');await 0;events.push('end')"),
        ],
    );
    let b = realm.evaluate_module_record(&modules[0]).unwrap();
    let a = realm.create_module("import'b';events.push('a')").unwrap();
    realm
        .load_module_graph(&a, |_, _| Ok(modules[0].clone()))
        .unwrap();
    let a = realm.evaluate_module_record(&a).unwrap();
    for (name, promise) in [("pa", a.clone()), ("pb", b.clone())] {
        realm
            .set_property_value(
                &Value::Object(realm.global_object()),
                JsString::from(name),
                promise,
                Span::new(0, 0),
            )
            .unwrap();
    }
    realm
        .eval("pa.then(()=>events.push('a-done'));pb.then(()=>events.push('b-done'))")
        .unwrap();
    drain(&mut realm);
    fulfilled(&realm, &a);
    fulfilled(&realm, &b);
    check(
        &mut realm,
        "events.join(',')==='start,end,b,a,b-done,a-done'",
    );
}

#[test]
fn resumed_dependency_host_aborts_leave_all_ancestors_pending_without_language_cleanup() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(20_000),
        ..Limits::default()
    });
    realm
        .eval("let started=0,caught=0,finalized=0,parent=0")
        .unwrap();
    let modules = graph(
        &mut realm,
        &[
            ("root", "import'leaf';parent++"),
            (
                "leaf",
                "await 0;started++;try{while(true){}}catch(e){caught++}finally{finalized++}",
            ),
        ],
    );
    let root = realm.evaluate_module_record(&modules[0]).unwrap();
    assert!(matches!(
        realm.run_next_promise_job().unwrap(),
        PromiseJobOutcome::Reported(Error::Limit { .. })
    ));
    pending(&realm, &root);
    assert_eq!(realm.evaluate_module_record(&modules[0]).unwrap(), root);
    for module in &modules {
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
    check(
        &mut realm,
        "started===1&&caught===0&&finalized===0&&parent===0",
    );
}

#[test]
fn a_host_abort_in_a_ready_ancestor_preserves_completed_dependencies_and_stops_other_ready_bodies()
{
    let mut realm = Realm::new(Limits {
        max_steps: Some(20_000),
        ..Limits::default()
    });
    realm.eval("let effects=0").unwrap();
    let modules = graph(
        &mut realm,
        &[
            ("root", "import'left';import'right';effects++"),
            ("left", "import'leaf';while(true){}"),
            ("right", "import'leaf';effects++"),
            ("leaf", "await 0"),
        ],
    );
    let root = realm.evaluate_module_record(&modules[0]).unwrap();
    let leaf = realm.evaluate_module_record(&modules[3]).unwrap();
    next(&mut realm);
    assert!(matches!(
        realm.run_next_promise_job().unwrap(),
        PromiseJobOutcome::Reported(Error::Limit { .. })
    ));
    fulfilled(&realm, &leaf);
    pending(&realm, &root);
    for module in &modules[..3] {
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
    check(&mut realm, "effects===0");
}

#[test]
fn queued_async_parent_edges_retain_dropped_graphs_and_symbol_reasons_until_the_promise_is_released()
 {
    let mut realm = Realm::default();
    realm.eval("let weak").unwrap();
    let modules = graph(
        &mut realm,
        &[
            ("root", "import'leaf'"),
            (
                "leaf",
                "const token=Symbol();weak=new WeakRef(token);await 0;throw token",
            ),
        ],
    );
    let promise = realm.evaluate_module_record(&modules[0]).unwrap();
    let promise = realm.root_value(promise, usize::MAX).unwrap();
    drop(modules);
    collect(&mut realm);
    check(&mut realm, "weak.deref()!==undefined");
    drain(&mut realm);
    assert!(matches!(
        state(&realm, promise.value()),
        PromiseState::Rejected(Value::Symbol(_))
    ));
    collect(&mut realm);
    check(&mut realm, "weak.deref()!==undefined");
    drop(promise);
    collect(&mut realm);
    check(&mut realm, "weak.deref()===undefined");
}

#[test]
fn deep_async_dependency_completion_uses_iterative_traversal() {
    let mut realm = Realm::default();
    realm.eval("let effects=0").unwrap();
    let names: Vec<_> = (0..600).map(|index| format!("m{index}")).collect();
    let sources: Vec<_> = (0..600)
        .map(|index| {
            if index == 599 {
                "await 0;effects++".into()
            } else {
                format!("import'm{}';effects++", index + 1)
            }
        })
        .collect();
    let inputs: Vec<_> = names
        .iter()
        .zip(&sources)
        .map(|(name, source)| (name.as_str(), source.as_str()))
        .collect();
    let modules = graph(&mut realm, &inputs);
    let root = realm.evaluate_module_record(&modules[0]).unwrap();
    pending(&realm, &root);
    drain(&mut realm);
    fulfilled(&realm, &root);
    check(&mut realm, "effects===600");
}

#[test]
fn job_work_refusals_preserve_preflight_queues_and_abort_started_module_reactions() {
    for steps in [0, 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 999] {
        for completion_job in [false, true] {
            let mut realm = Realm::default();
            realm.eval("let effects=0").unwrap();
            let modules = graph(
                &mut realm,
                &[
                    ("root", "import'leaf';effects++"),
                    ("leaf", "await 0;effects++"),
                ],
            );
            let promise = realm.evaluate_module_record(&modules[0]).unwrap();
            if completion_job {
                next(&mut realm);
            }
            realm.limits.max_steps = Some(steps);
            let outcome = realm.run_next_promise_job();
            realm.limits.max_steps = None;
            match outcome {
                Err(Error::Limit { .. }) => {
                    drain(&mut realm);
                    fulfilled(&realm, &promise);
                    check(&mut realm, "effects===2");
                }
                Ok(PromiseJobOutcome::Reported(Error::Limit { .. })) => {
                    for module in &modules {
                        let status = realm
                            .objects
                            .module_execution(module.root.handle())
                            .unwrap()
                            .unwrap()
                            .status;
                        assert!(
                            matches!(
                                status,
                                ModuleExecutionStatus::Aborted | ModuleExecutionStatus::Evaluated
                            ),
                            "{steps}/{completion_job}: {status:?}"
                        );
                    }
                    let before = realm.eval("effects").unwrap();
                    assert_eq!(realm.evaluate_module_record(&modules[0]).unwrap(), promise);
                    drain(&mut realm);
                    assert_eq!(realm.eval("effects").unwrap(), before);
                }
                Ok(PromiseJobOutcome::Completed) => {
                    drain(&mut realm);
                    fulfilled(&realm, &promise);
                    check(&mut realm, "effects===2");
                }
                outcome => panic!("{steps}/{completion_job}: {outcome:?}"),
            }
        }
    }
}

#[test]
fn initial_graph_host_aborts_cancel_already_suspended_bodies_without_running_finalizers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(20_000),
        ..Limits::default()
    });
    realm.eval("let effects=0,finalized=0").unwrap();
    let modules = graph(
        &mut realm,
        &[
            ("root", "import'leaf';import'bad';effects++"),
            ("leaf", "try{await 0;effects++}finally{finalized++}"),
            ("bad", "while(true){}"),
        ],
    );
    assert!(matches!(
        realm.evaluate_module_record(&modules[0]),
        Err(Error::Limit { .. })
    ));
    let root = realm.evaluate_module_record(&modules[0]).unwrap();
    pending(&realm, &root);
    next(&mut realm);
    check(&mut realm, "effects===0&&finalized===0");
    assert!(matches!(
        realm.run_next_promise_job().unwrap(),
        PromiseJobOutcome::NoJob
    ));
    pending(&realm, &root);
}
