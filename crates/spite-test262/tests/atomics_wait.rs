//! Explicit native timeout jobs in the Test262 host loop.

use spite_test262::{Outcome, Runner};
use std::{fs, path::Path};

fn run(metadata: &str, body: &str) -> Vec<Outcome> {
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
fn native_timeouts_complete_async_scripts_and_rooted_module_awaits() {
    for kind in ["Int32Array", "BigInt64Array"] {
        let expected = if kind == "BigInt64Array" { "0n" } else { "0" };
        let body = format!(
            "const a=new {kind}(new SharedArrayBuffer(8));Atomics.waitAsync(a,0,{expected},1).value.then(value=>{{assert.sameValue(value,'timed-out');$DONE()}}).catch($DONE);"
        );
        assert_eq!(
            run("flags: [async]", &body),
            [Outcome::Passed, Outcome::Passed]
        );
    }
    assert_eq!(
        run(
            "flags: [module]",
            "const a=new Int32Array(new SharedArrayBuffer(8));$262.gc();export const result=await Atomics.waitAsync(a,0,0,1).value;if(result!=='timed-out')throw 7;"
        ),
        [Outcome::Passed]
    );
}

#[test]
fn completion_does_not_wait_for_unrelated_future_timers_or_accept_missing_done() {
    assert_eq!(
        run(
            "flags: [async]",
            "print('diagnostic');$DONE();Atomics.waitAsync(new Int32Array(new SharedArrayBuffer(8)),0,0,Number.MAX_VALUE);"
        ),
        [Outcome::Passed, Outcome::Passed]
    );
    for body in [
        "Atomics.waitAsync(new Int32Array(new SharedArrayBuffer(8)),0,0,1);",
        "print('Test262:AsyncTestCompleteExtra');Atomics.waitAsync(new Int32Array(new SharedArrayBuffer(8)),0,0,1);",
    ] {
        assert!(
            run("flags: [async]", body)
                .iter()
                .all(|outcome| matches!(outcome, Outcome::Failed { .. }))
        );
    }
}

#[test]
fn timer_reaction_errors_keep_runtime_phase_and_host_failures_never_pass_negatives() {
    let metadata = "flags: [async]\nnegative:\n  phase: runtime\n  type: TypeError";
    let body = "let p=Atomics.waitAsync(new Int32Array(new SharedArrayBuffer(8)),0,0,1).value;function C(executor){executor(()=>{throw new TypeError('real')},()=>{})}p.constructor={[Symbol.species]:C};p.then(()=>2);";
    assert_eq!(run(metadata, body), [Outcome::Passed, Outcome::Passed]);
    let spoof = "Atomics.waitAsync(new Int32Array(new SharedArrayBuffer(8)),0,0,1).value.then(()=>$DONE({name:'TypeError',message:'spoof'}));";
    assert!(
        run(metadata, spoof)
            .iter()
            .all(|outcome| matches!(outcome, Outcome::Failed { .. }))
    );
    let unavailable = "Atomics.waitAsync(new Int32Array(new SharedArrayBuffer(8)),0,0,1).value.then(()=>$262.agent).catch($DONE);";
    assert!(
        run(metadata, unavailable)
            .iter()
            .all(|outcome| matches!(outcome, Outcome::Unsupported { .. }))
    );
}
