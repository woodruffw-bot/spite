//! Execute unchanged pinned harness code and verify success/failure paths.

use spite_test262::{Outcome, Runner, Stage};
use std::{fs, path::Path};

fn run(body: &str) -> Vec<Outcome> {
    run_with_includes(body, "compareArray.js")
}

fn run_with_includes(body: &str, includes: &str) -> Vec<Outcome> {
    let source = format!(
        "/*---\ndescription: Local control for the pinned assertion harness\nincludes: [{includes}]\n---*/\n{body}"
    );
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/test262/upstream/harness");
    Runner::default()
        .run(&source, None, |name| {
            fs::read_to_string(root.join(name)).map_err(|error| error.to_string())
        })
        .unwrap()
        .into_iter()
        .map(|case| case.outcome)
        .collect()
}

#[test]
fn original_constructor_helper_checks_constructibility_without_masking_host_gaps() {
    assert_eq!(
        run_with_includes(
            "function F(){}assert.sameValue(isConstructor(F),true);assert.sameValue(isConstructor(F.bind(null)),true);assert.sameValue(isConstructor(Array),true);assert.sameValue(isConstructor(()=>{}),false);assert.sameValue(isConstructor(Reflect.apply),false);assert.sameValue(isConstructor(Reflect.construct),false);assert.throws(Test262Error,()=>isConstructor({}));let N=F.bind(null);Object.defineProperty(N,'prototype',{get(){throw new TypeError();}});assert.sameValue(isConstructor(N),false);",
            "isConstructor.js"
        ),
        [Outcome::Passed, Outcome::Passed]
    );
    let outcomes = run_with_includes(
        "let N=(function(){}).bind(null);Object.defineProperty(N,'prototype',{get(){Proxy;}});isConstructor(N);",
        "isConstructor.js",
    );
    assert_eq!(outcomes.len(), 2);
    assert!(
        outcomes.iter().all(|outcome| matches!(
            outcome,
            Outcome::Unsupported {
                stage: Stage::Runtime,
                ..
            }
        )),
        "{outcomes:?}"
    );
}

#[test]
fn original_assertions_compare_values_and_catch_test262_error_objects() {
    let outcomes = run(
        "assert(true);assert.sameValue(NaN,NaN);assert.notSameValue(0,-0);let o={};assert.sameValue(o,o);assert.notSameValue(o,{});assert.throws(Test262Error,()=>Test262Error.thrower('sentinel'));let caught=false;try{assert(false,'sentinel');}catch(e){caught=true;assert.sameValue(e.constructor,Test262Error);assert.sameValue(e.message,'sentinel');assert.sameValue(e.toString(),'Test262Error: sentinel');}assert(caught);",
    );
    assert_eq!(outcomes, [Outcome::Passed, Outcome::Passed]);
}

#[test]
fn original_throws_assertions_check_native_error_constructors() {
    let outcomes = run(
        "assert.throws(TypeError,()=>+1n);assert.throws(ReferenceError,()=>missing);assert.throws(RangeError,()=>1n/0n);assert.throws(TypeError,()=>{throw new TypeError('explicit');});",
    );
    assert_eq!(outcomes, [Outcome::Passed, Outcome::Passed]);
    let outcomes = run("assert.throws(TypeError,()=>{throw new RangeError('wrong kind');});");
    assert!(
        outcomes.iter().all(|outcome| matches!(
            outcome,
            Outcome::Failed {
                stage: Stage::Runtime,
                ..
            }
        )),
        "{outcomes:?}"
    );
}

#[test]
fn deliberate_assertion_failures_are_runtime_failures_in_both_modes() {
    for body in [
        "assert(false,'deliberate failure');",
        "assert.throws(Test262Error,()=>{},'missing throw');",
        "assert.sameValue(1,2,'deliberate mismatch');",
        "assert.notSameValue(1n,1n,'deliberate mismatch');",
    ] {
        let outcomes = run(body);
        assert_eq!(outcomes.len(), 2);
        assert!(
            outcomes.iter().all(|outcome| matches!(
                outcome,
                Outcome::Failed {
                    stage: Stage::Runtime,
                    ..
                }
            )),
            "{outcomes:?}"
        );
    }
}

#[test]
fn missing_diagnostic_formatting_is_an_explicit_non_passing_gap() {
    // String comparison diagnostics consult the unimplemented JSON global. They cannot
    // be mistaken for a passing assertion or a test's expected exception.
    let outcomes = run("assert.sameValue('left','right','deliberate mismatch');");
    assert_eq!(outcomes.len(), 2);
    assert!(
        outcomes.iter().all(|outcome| matches!(
            outcome,
            Outcome::Unsupported {
                stage: Stage::Runtime,
                ..
            }
        )),
        "{outcomes:?}"
    );
}

#[test]
fn original_array_comparisons_preserve_same_value_semantics() {
    assert_eq!(
        run(
            "let o={};assert.compareArray([NaN,-0,1n,o],[NaN,-0,1n,o]);assert.sameValue(compareArray([0],[-0]),false);assert.sameValue(compareArray([1],[2]),false);assert.sameValue(compareArray([],[1]),false);"
        ),
        [Outcome::Passed, Outcome::Passed]
    );
    let outcomes = run("assert.compareArray([1],[2],'deliberate mismatch');");
    assert_eq!(outcomes.len(), 2);
    // Numeric array mismatch formatting now executes and throws Test262Error.
    assert!(
        outcomes.iter().all(|outcome| matches!(
            outcome,
            Outcome::Failed {
                stage: Stage::Runtime,
                ..
            }
        )),
        "{outcomes:?}"
    );
    assert_eq!(
        run(
            "let caught=false;try{assert.compareArray([1],[2],'deliberate mismatch');}catch(e){caught=true;assert.sameValue(e.constructor,Test262Error);assert.sameValue(e.message,'Actual [1] and expected [2] should have the same contents. deliberate mismatch');}assert(caught);"
        ),
        [Outcome::Passed, Outcome::Passed]
    );
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/test262/upstream/harness");
    let mut realm = spite_runtime::Realm::default();
    for name in ["assert.js", "sta.js", "compareArray.js"] {
        realm
            .eval(&fs::read_to_string(root.join(name)).unwrap())
            .unwrap();
    }
    let message = realm.eval("let message;try{assert.compareArray([1],[2],'deliberate mismatch');}catch(e){message=e.toString();}message").unwrap();
    insta::assert_snapshot!("array_comparison_mismatch", message);
}
