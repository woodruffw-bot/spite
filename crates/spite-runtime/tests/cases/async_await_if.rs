//! Conditional statement suspension and selection of awaited branches.

use crate::common::UnavailableHostRealm;
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

#[test]
fn awaited_conditions_select_one_suspended_branch_before_parent_execution_continues() {
    for (condition, expected) in [
        (true, "start,yes,yes-end,after"),
        (false, "start,no,no-end,after"),
    ] {
        let mut realm = Realm::default();
        realm.eval(r#"
            let state={},events=[],result;state.gate=new Promise(resolve=>{state.release=resolve});
            async function f(){events.push('start');if(await state.gate){events.push('yes');await 0;events.push('yes-end')}else{events.push('no');await 0;events.push('no-end')}events.push('after');return events.join(',')}
            f().then(value=>{result=value});
        "#).unwrap();
        check(&mut realm, "events.join(',')==='start'&&result===undefined");
        realm.eval(&format!("state.release({condition})")).unwrap();
        drain(&mut realm);
        check(
            &mut realm,
            &format!("events.join(',')==='{expected}'&&result==='{expected}'"),
        );
    }
}

#[test]
fn nested_else_if_paths_keep_block_scope_tdz_and_hoisted_functions() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result,before,outside,tdz=false;
        async function f(){'use strict';let value=1;if(await false){throw 1}else if(await true){before=typeof local;const read=()=>value;try{read()}catch(error){tdz=error instanceof ReferenceError}const value=await 3;function local(){return value}if(await true){const value=await 5;result=value+local()+read()}}outside=typeof local;return value}
        let root;f().then(value=>{root=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "before==='function'&&tdz&&result===11&&outside==='undefined'&&root===1",
    );
}

#[test]
fn unselected_awaits_and_remaining_unsupported_forms_are_never_evaluated() {
    let mut realm = Realm::with_unavailable_host();
    realm.eval(r#"
        let result;
        async function f(){if(await true)await 0;else return{x:await missing};if(await false)await missing;if(false){spiteUnavailableHost();await missing}return 7}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result===7");
}

#[test]
fn selected_abrupt_branches_and_rejected_conditions_unwind_the_correct_resources() {
    for (mode, condition, tail) in [
        (0, "await true", "return await 7"),
        (1, "await true", "throw await marker"),
        (2, "await Promise.reject(marker)", "return 0"),
    ] {
        let mut realm = Realm::default();
        realm.eval(&format!("let mode={mode};")).unwrap();
        let source=r#"
            let marker={},r={},a={},b={},events=[],result,reason,later=0,otherwise=0;
            async function f(){using root={[Symbol.dispose](){events.push('root');if(mode===1)throw r}};{using outer={[Symbol.dispose](){events.push('a');if(mode===1)throw a}};if(CONDITION){using inner={[Symbol.dispose](){events.push('b');if(mode===1)throw b}};TAIL}else{otherwise++}later++}}
            f().then(value=>{result=value},value=>{reason=value});
        "#.replace("CONDITION",condition).replace("TAIL",tail);
        realm.eval(&source).unwrap();
        drain(&mut realm);
        check(&mut realm, "otherwise===0&&later===0");
        check(
            &mut realm,
            match mode {
                0 => "result===7&&reason===undefined&&events.join(',')==='b,a,root'",
                1 => {
                    "reason instanceof SuppressedError&&reason.error===r&&reason.suppressed.error===a&&reason.suppressed.suppressed.error===b&&reason.suppressed.suppressed.suppressed===marker&&events.join(',')==='b,a,root'"
                }
                _ => "reason===marker&&result===undefined&&events.join(',')==='a,root'",
            },
        );
    }
}

#[test]
fn var_branches_do_not_add_a_scope_and_keep_mapped_parameter_references() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let result;
        async function f(x){if(await true)var x=await 7;else var x=0;return arguments[0]+x}
        f(1).then(value=>{result=value});
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "result===14");
}

#[test]
fn private_eval_this_and_arguments_survive_a_condition_and_selected_branch() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result;
        class C{#x=10;async read(input){if(await input){await 0;return eval('this.#x')+arguments.length}return 0}}
        new C().read(true).then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result===11");
}

#[test]
fn exited_branch_environments_are_collectible_during_a_later_root_await() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},result;state.first=new Promise(resolve=>{state.releaseFirst=resolve});state.second=new Promise(resolve=>{state.releaseSecond=resolve});
        async function f(){let saved;if(await true){const value={x:7};state.weak=new WeakRef(value);await state.first;saved=value.x}await state.second;return saved}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "state.weak.deref()!==undefined&&result===undefined",
    );
    realm.eval("state.releaseFirst(0)").unwrap();
    drain(&mut realm);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "state.weak.deref()===undefined&&result===undefined",
    );
    realm.eval("state.releaseSecond(0)").unwrap();
    drain(&mut realm);
    check(&mut realm, "result===7");
}

#[test]
fn completed_condition_values_use_boolean_truthiness_without_primitive_conversion() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let conversions=0,result,first=0,second=0,obj={valueOf(){conversions++;throw 1},[Symbol.toPrimitive](){conversions++;throw 2}};
        async function f(){if(await obj)first=1;if(await Symbol())second=2;return first+second}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result===3&&conversions===0");
}
