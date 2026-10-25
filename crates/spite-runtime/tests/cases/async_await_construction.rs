//! Constructor and argument evaluation across source Await suspension.

use spite_runtime::{Error, Limits, PromiseJobOutcome, Realm, Value};

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
fn constructor_lookup_precedes_arguments_and_replacement_does_not_change_new_target() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},events=[],result;
        function C(a,b,c){events.push('construct');this.sum=a+b+c;this.target=new.target}
        let holder={get C(){events.push('get');return C}};
        state.gate=new Promise(resolve=>{state.release=resolve});
        async function f(){return new holder.C((events.push('first'),1),await state.gate,(events.push('last'),3))}
        f().then(value=>{result=value});Object.defineProperty(holder,'C',{value(){throw 1}});
    "#).unwrap();
    check(
        &mut realm,
        "events.join(',')==='get,first'&&result===undefined",
    );
    realm
        .eval("C.prototype={marker:true};state.release(2)")
        .unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='get,first,last,construct'&&result.sum===6&&result.target===C&&result.marker===true&&result instanceof C",
    );
}

#[test]
fn awaited_constructor_values_and_member_callees_construct_with_and_without_arguments() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let result,noArgs;
        function C(value=17){this.value=value;this.target=new.target}
        let holder={C};
        async function f(){return new ((await holder)[await 'C'])(await 7)}
        let g=async()=>new (await C);
        f().then(value=>{result=value});g().then(value=>{noArgs=value});
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result!==holder&&result.value===7&&result.target===C&&result instanceof C&&noArgs.value===17&&noArgs.target===C",
    );
}

#[test]
fn nonconstructors_evaluate_all_arguments_and_lookup_failures_skip_arguments() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},events=[],reason,marker={},lookupReason,operand=0,bad={get C(){throw marker}};
        state.gate=new Promise(resolve=>{state.release=resolve});
        async function f(){return new (1)((events.push('first'),1),await state.gate,(events.push('last'),3))}
        async function g(){return new bad.C(await(operand++))}
        f().catch(value=>{reason=value});g().catch(value=>{lookupReason=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='first'&&reason===undefined&&lookupReason===marker&&operand===0",
    );
    realm.eval("state.release(2)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "reason instanceof TypeError&&events.join(',')==='first,last'",
    );
}

#[test]
fn awaited_constructor_spread_uses_sync_iteration_without_awaiting_yielded_promises() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let result,promise=Promise.resolve(9);function C(...values){this.values=values}
        async function f(){return new C(...await [1,promise],await 3)}
        f().then(value=>{result=value});
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result.values.length===3&&result.values[0]===1&&result.values[1]===promise&&result.values[2]===3",
    );
}

#[test]
fn nested_construction_in_bindings_arrays_and_calls_resumes_the_original_value_action() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result;class C{constructor(value){this.value=value}}
        function target(value){return value}
        async function f(){const x=new C(await 2);return target([new C(x.value+await 3),new(await C)])}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result[0].value===5&&result[0] instanceof C&&result[1].value===undefined&&result[1] instanceof C",
    );
}

#[test]
fn selected_constructors_stay_alive_across_arguments_and_release_after_rejection() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},reason,marker={};state.gate=new Promise((resolve,reject)=>{state.reject=reject});
        let C=function(value){this.value=value};state.weak=new WeakRef(C);
        async function f(){return new C(await state.gate)}
        f().catch(value=>{reason=value});C=null;
    "#).unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "state.weak.deref()!==undefined&&reason===undefined",
    );
    realm.eval("state.reject(marker)").unwrap();
    drain(&mut realm);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "state.weak.deref()===undefined&&reason===marker",
    );
}

#[test]
fn resumed_constructor_argument_limit_failure_bypasses_rejection_and_resource_cleanup() {
    let mut realm = Realm::new(Limits {
        max_arguments: Some(2),
        ..Default::default()
    });
    realm.eval(r#"
        let state={},called=false,caught=false,disposed=0;
        state.gate=new Promise(resolve=>{state.release=resolve});function C(){called=true}
        async function f(){using resource={[Symbol.dispose](){disposed++}};return new C(1,await state.gate,3)}
        f().catch(()=>{caught=true});
    "#).unwrap();
    realm.eval("state.release(2)").unwrap();
    let mut reported = false;
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => (),
            PromiseJobOutcome::Reported(Error::Limit { .. }) => reported = true,
            PromiseJobOutcome::Reported(error) => panic!("{error}"),
        }
    }
    assert!(reported);
    check(&mut realm, "!called&&!caught&&disposed===0");
}

#[test]
fn proxy_constructor_prototype_lookup_follows_completed_argument_evaluation() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},events=[],result;
        let C=new Proxy(function(value){events.push('construct');this.value=value;this.target=new.target},{get(target,key,receiver){if(key==='prototype')events.push('prototype');return Reflect.get(target,key,receiver)}}),saved=C;
        state.gate=new Promise(resolve=>{state.release=resolve});
        async function f(){return new C((events.push('arg'),await state.gate))}
        f().then(value=>{result=value});C=function(){throw 1};
    "#).unwrap();
    check(&mut realm, "events.join(',')==='arg'&&result===undefined");
    realm.eval("state.release(7)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='arg,prototype,construct'&&result.value===7&&result.target===saved",
    );
}
