//! Nested await value expressions, operation order and retained operand lifetimes.

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
fn unary_binary_chains_and_awaited_initializers_keep_precedence_and_value_semantics() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let results=[];
        async function f(){const x=await 2*3;var y=(await 3)+4;let z=!(await false);return[x,y,z]}
        f().then(value=>{results.push(value.join(',')==='6,7,true')});
        (async()=>~await 2)().then(value=>{results.push(value===-3)});
        (async()=>-(await 2))().then(value=>{results.push(value===-2)});
        (async()=>typeof(await function(){}))().then(value=>{results.push(value==='function')});
        (async()=>void(await 2))().then(value=>{results.push(value===undefined)});
        (async()=>delete(await 2))().then(value=>{results.push(value===true)});
        (async()=>(await 2)**3+await 4+5)().then(value=>{results.push(value===17)});
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results.length===7&&results.every(value=>value)",
    );
}

#[test]
fn nested_awaits_each_suspend_before_the_surrounding_expression_completes() {
    let mut realm = Realm::default();
    realm.eval("let events=[];async function f(){events.push('start');const value=await await 1;events.push(value);return value}f().then(()=>{events.push('done')});events.push('outside')").unwrap();
    check(&mut realm, "events.join(',')==='start,outside'");
    assert!(matches!(
        realm.run_next_promise_job().unwrap(),
        PromiseJobOutcome::Completed
    ));
    check(&mut realm, "events.join(',')==='start,outside'");
    assert!(matches!(
        realm.run_next_promise_job().unwrap(),
        PromiseJobOutcome::Completed
    ));
    check(&mut realm, "events.join(',')==='start,outside,1'");
    drain(&mut realm);
    check(&mut realm, "events.join(',')==='start,outside,1,done'");
}

#[test]
fn lhs_evaluation_runs_once_but_primitive_conversion_waits_for_the_rhs_and_survives_gc() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},events=[],result,left={x:5,get valueOf(){events.push('get');return function(){events.push('coerce');return this.x}}};
        state.gate=new Promise(resolve=>{state.release=resolve});state.weak=new WeakRef(left);
        function getLeft(){events.push('left');return left}
        async function f(){return getLeft()+await (events.push('operand'),state.gate)}
        f().then(value=>{result=value});left=null;
    "#).unwrap();
    check(&mut realm, "events.join(',')==='left,operand'");
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(&mut realm, "state.weak.deref()!==undefined");
    realm.eval("state.release(7)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===12&&events.join(',')==='left,operand,get,coerce'",
    );
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(&mut realm, "state.weak.deref()===undefined");
}

#[test]
fn logical_short_circuit_skips_unreachable_awaits_and_continues_later_chain_steps() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let result;
        async function a(){return (await false)&&Math.abs(await missing)}
        async function b(){return (await true)||Math.abs(await missing)}
        async function c(){return (await 7)??Math.abs(await missing)}
        async function d(){return (await false)&&Math.abs(await missing)||await 9}
        Promise.all([a(),b(),c(),d()]).then(value=>{result=value});
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "result.join(',')==='false,true,7,9'");
}

#[test]
fn conditional_expressions_select_one_awaited_branch_and_do_not_infer_function_names() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let result,name;
        async function f(value){return await value ? (await 2)+1 : (await 4)*2}
        async function g(){const value=(await false)?()=>1:()=>2;return value.name}
        Promise.all([f(true),f(false)]).then(value=>{result=value});g().then(value=>{name=value});
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "result.join(',')==='3,8'&&name===''");
}

#[test]
fn var_target_is_saved_before_the_complete_rhs_and_operand_values_are_not_replayed() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},write,result;state.gate=new Promise(resolve=>{state.release=resolve});
        async function f(value){write=x=>{arguments[0]=x};var value=value+await state.gate;return[value,arguments[0]]}
        f(2).then(value=>{result=value});write(100);
    "#).unwrap();
    realm.eval("state.release(9)").unwrap();
    drain(&mut realm);
    check(&mut realm, "result.join(',')==='11,11'");
}

#[test]
fn rejected_rhs_skips_lhs_conversion_and_preserves_resource_suppression() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let original={},cleanup={},events=[],reason,left={valueOf(){events.push('convert');return 1}};
        async function f(){using resource={[Symbol.dispose](){events.push('dispose');throw cleanup}};return left+await Promise.reject(original)}
        f().catch(error=>{reason=error});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "reason instanceof SuppressedError&&reason.error===cleanup&&reason.suppressed===original&&events.join(',')==='dispose'",
    );
}
