//! StatementList values remain owned across suspension until replaced.

use spite_runtime::{PromiseJobOutcome, Realm, Value};

fn check(realm: &mut Realm, source: &str) {
    assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
}

fn drain(realm: &mut Realm) {
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => (),
            PromiseJobOutcome::Reported(error) => panic!("{error}"),
        }
    }
}

fn collect(realm: &mut Realm) {
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
}

fn setup(realm: &mut Realm) {
    realm
        .eval(
            r#"
        let state={},result;
        state.first=new Promise(resolve=>{state.releaseFirst=resolve});
        state.second=new Promise(resolve=>{state.releaseSecond=resolve});
        state.third=new Promise(resolve=>{state.releaseThird=resolve});
        function make(){const value={x:7};state.weak=new WeakRef(value);return value}
    "#,
        )
        .unwrap();
}

#[test]
fn root_statement_values_survive_empty_declarations_and_release_when_replaced() {
    let mut realm = Realm::default();
    setup(&mut realm);
    realm.eval("async function f(){make();const x=await state.first;0;const y=await state.second;return 7}f().then(value=>{result=value})").unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "state.weak.deref()!==undefined&&result===undefined",
    );
    realm.eval("state.releaseFirst(0)").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "state.weak.deref()===undefined&&result===undefined",
    );
    realm.eval("state.releaseSecond(0)").unwrap();
    drain(&mut realm);
    check(&mut realm, "result===7");
}

fn forwarded_value(source: &str) {
    let mut realm = Realm::default();
    setup(&mut realm);
    realm.eval(source).unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "state.weak.deref()!==undefined&&result===undefined",
    );
    realm.eval("state.releaseFirst(0)").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "state.weak.deref()!==undefined&&result===undefined",
    );
    realm.eval("state.releaseSecond(0)").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "state.weak.deref()===undefined&&result===undefined",
    );
    realm.eval("state.releaseThird(0)").unwrap();
    drain(&mut realm);
    check(&mut realm, "result===7");
}

#[test]
fn nested_block_completion_values_forward_to_the_parent_statement_list() {
    forwarded_value(
        "async function f(){{make();const x=await state.first}const y=await state.second;0;const z=await state.third;return 7}f().then(value=>{result=value})",
    );
}

#[test]
fn conditional_branch_completion_values_forward_after_the_selected_block_exits() {
    forwarded_value(
        "async function f(){if(true){make();const x=await state.first}const y=await state.second;0;const z=await state.third;return 7}f().then(value=>{result=value})",
    );
}

#[test]
fn missing_false_branch_applies_update_empty_and_replaces_the_previous_value() {
    let mut realm = Realm::default();
    setup(&mut realm);
    realm.eval("async function f(){make();if(await false);const x=await state.first;return 7}f().then(value=>{result=value})").unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "state.weak.deref()!==undefined&&result===undefined",
    );
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "state.weak.deref()===undefined&&result===undefined",
    );
    realm.eval("state.releaseFirst(0)").unwrap();
    drain(&mut realm);
    check(&mut realm, "result===7");
}
