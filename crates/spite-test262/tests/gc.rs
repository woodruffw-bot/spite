//! The host collects at execution boundaries and reports cleanup jobs faithfully.

use spite_test262::{Outcome, Runner, Stage};
use std::{fs, path::Path};

fn outcomes(metadata: &str, body: &str) -> Vec<Outcome> {
    let source = format!("/*---\n{metadata}\n---*/\n{body}");
    let harness =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/test262/upstream/harness");
    Runner::default()
        .run(&source, None, |name| {
            fs::read_to_string(harness.join(name)).map_err(|error| error.to_string())
        })
        .unwrap()
        .into_iter()
        .map(|case| case.outcome)
        .collect()
}

#[test]
fn synchronous_scripts_keep_current_targets_and_drain_requested_collection_reactions() {
    assert_eq!(
        outcomes(
            "",
            r#"
        let weak;(function(){weak=new WeakRef({})})();
        $262.gc().then(()=>assert.sameValue(weak.deref(),undefined));
        assert.notSameValue(weak.deref(),undefined);
    "#
        ),
        [Outcome::Passed, Outcome::Passed]
    );
}

#[test]
fn completed_module_promises_are_rooted_before_collection_without_a_source_loader() {
    assert_eq!(outcomes("flags: [module]", "$262.gc()"), [Outcome::Passed]);
}

#[test]
fn asynchronous_scripts_await_collection_before_checking_weak_references() {
    assert_eq!(
        outcomes(
            "flags: [async]\nincludes: [asyncHelpers.js]",
            r#"
        asyncTest(async function(){
            let weak;(function(){weak=new WeakRef({})})();
            await $262.gc();assert.sameValue(weak.deref(),undefined);
        });
    "#
        ),
        [Outcome::Passed, Outcome::Passed]
    );
}

#[test]
fn cleanup_jobs_and_reentrant_collection_run_at_separate_boundaries() {
    assert_eq!(
        outcomes(
            "flags: [async]\nincludes: [asyncHelpers.js]",
            r#"
        asyncTest(async function(){
            let count=0,registry=new FinalizationRegistry(value=>{count+=value;$262.gc()});
            (function(){registry.register({},7)})();
            await $262.gc();assert.sameValue(count,7);
        });
    "#
        ),
        [Outcome::Passed, Outcome::Passed]
    );
}

#[test]
fn reported_cleanup_exceptions_retain_runtime_types_and_host_failures_never_pass_negatives() {
    assert_eq!(
        outcomes(
            "negative:\n  phase: runtime\n  type: TypeError",
            r#"
        let registry=new FinalizationRegistry(()=>{throw new TypeError()});
        (function(){registry.register({},7)})();$262.gc();
    "#
        ),
        [Outcome::Passed, Outcome::Passed]
    );
    for outcome in outcomes(
        "negative:\n  phase: runtime\n  type: ReferenceError",
        r#"
        let registry=new FinalizationRegistry(()=>{$262.createRealm});
        (function(){registry.register({},7)})();$262.gc();
    "#,
    ) {
        assert!(matches!(
            outcome,
            Outcome::Unsupported {
                stage: Stage::Runtime,
                ..
            }
        ));
    }
}
