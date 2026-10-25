//! Execute the unchanged doneprintHandle.js protocol and queued Promise jobs.

use spite_runtime::{Limits, Realm, Value};
use spite_test262::{Outcome, Runner, Stage};
use std::{fs, path::Path};

fn run_with(runner: Runner, metadata: &str, body: &str) -> Vec<Outcome> {
    let source = format!("/*---\nflags: [async]\n{metadata}\n---*/\n{body}");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/test262/upstream/harness");
    runner
        .run(&source, None, |name| {
            fs::read_to_string(root.join(name)).map_err(|error| error.to_string())
        })
        .unwrap()
        .into_iter()
        .map(|case| case.outcome)
        .collect()
}

fn run(body: &str) -> Vec<Outcome> {
    run_with(Runner::default(), "", body)
}

#[test]
fn queued_fulfillment_rejection_and_nested_thenables_reach_the_original_done_helper() {
    for body in [
        "Promise.resolve(7).then(v=>assert.sameValue(v,7)).then($DONE,$DONE)",
        "Promise.reject(7).catch(v=>assert.sameValue(v,7)).then($DONE,$DONE)",
        "Promise.resolve({then(resolve){resolve({then(resolve){resolve(9)}})}}).then(v=>assert.sameValue(v,9)).then($DONE,$DONE)",
        "$DONE()",
    ] {
        assert_eq!(run(body), [Outcome::Passed, Outcome::Passed], "{body}");
    }
    assert_eq!(
        run_with(
            Runner::default(),
            "includes: [asyncHelpers.js]",
            "asyncTest(()=>assert.throwsAsync(TypeError,()=>Promise.reject(new TypeError('sentinel'))))",
        ),
        [Outcome::Passed, Outcome::Passed]
    );
}

#[test]
fn missing_duplicate_and_failed_completions_never_pass() {
    assert!(
        run_with(
            Runner::default(),
            "includes: [asyncHelpers.js]",
            "asyncTest(()=>assert.throwsAsync(TypeError,()=>Promise.resolve(1)))",
        )
        .iter()
        .all(|outcome| matches!(outcome, Outcome::Failed { .. }))
    );
    for body in [
        "Promise.resolve(1).then(()=>{})",
        "$DONE();$DONE()",
        "Promise.resolve().then($DONE);Promise.resolve().then($DONE)",
        "$DONE(new Test262Error('sentinel'))",
        "Promise.resolve(1).then(v=>assert.sameValue(v,2)).then($DONE,$DONE)",
    ] {
        assert!(
            run(body).iter().all(|outcome| matches!(
                outcome,
                Outcome::Failed {
                    stage: Stage::Runtime,
                    ..
                }
            )),
            "{body}"
        );
    }
}

#[test]
fn failure_text_cannot_satisfy_a_runtime_negative_but_actual_job_throws_can() {
    let negative = "negative:\n  phase: runtime\n  type: TypeError";
    let failed = run_with(
        Runner::default(),
        negative,
        "$DONE({name:'TypeError',message:'spoof'})",
    );
    assert!(
        failed
            .iter()
            .all(|outcome| matches!(outcome, Outcome::Failed { .. }))
    );
    let passed = run_with(
        Runner::default(),
        negative,
        "let p=Promise.resolve(1);function C(executor){executor(()=>{throw new TypeError('sentinel')},()=>{})}p.constructor={[Symbol.species]:C};p.then(()=>2);",
    );
    assert_eq!(passed, [Outcome::Passed, Outcome::Passed]);
    let rejected = run_with(
        Runner::default(),
        negative,
        "Promise.resolve(1).then(()=>{throw new TypeError('sentinel')});",
    );
    assert!(
        rejected
            .iter()
            .all(|outcome| matches!(outcome, Outcome::Failed { .. }))
    );
}

#[test]
fn job_host_failures_preserve_nonpassing_categories() {
    let unsupported = run_with(
        Runner::default(),
        "negative:\n  phase: runtime\n  type: ReferenceError",
        "Promise.resolve().then(()=>{$262.createRealm()}).then($DONE,$DONE)",
    );
    assert!(unsupported.iter().all(|outcome| matches!(
        outcome,
        Outcome::Unsupported {
            stage: Stage::Runtime,
            ..
        }
    )));
    let limited = run_with(
        Runner {
            limits: Limits {
                max_steps: Some(100_000),
                ..Limits::default()
            },
        },
        "",
        "Promise.resolve().then(()=>{while(true){}}).then($DONE,$DONE)",
    );
    assert!(limited.iter().all(|outcome| matches!(
        outcome,
        Outcome::Limit {
            stage: Stage::Runtime,
            ..
        }
    )));
}

#[test]
fn print_capture_ignores_unrelated_text_and_uses_its_retained_string_intrinsic() {
    assert_eq!(
        run("print('unrelated');print('\\uD800');String=function(){throw 1};$DONE()"),
        [Outcome::Passed, Outcome::Passed]
    );
    assert_eq!(
        Realm::default().eval("typeof print==='undefined'&&typeof $DONE==='undefined'"),
        Ok(Value::Boolean(true))
    );
}
