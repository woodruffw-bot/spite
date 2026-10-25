//! Array literal accumulation across source Await suspension.

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
fn elements_and_holes_accumulate_once_in_order_across_multiple_awaits() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},events=[],result;
        state.gate=new Promise(resolve=>{state.release=resolve});
        async function f(){return[(events.push('first'),1),,await state.gate,,(events.push('next'),await 3),]}
        f().then(value=>{result=value});
    "#).unwrap();
    check(&mut realm, "events.join(',')==='first'&&result===undefined");
    realm.eval("state.release(2)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='first,next'&&result.length===5&&result[0]===1&&result[2]===2&&result[4]===3&&!(1 in result)&&!(3 in result)",
    );
}

#[test]
fn awaited_spread_iterates_synchronously_and_keeps_yielded_promises_as_values() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],result,promise=Promise.resolve(9),source={
            [Symbol.iterator](){events.push('iterator');let index=0;return{next(){events.push('next');return{done:index++===2,value:index===1?promise:4}}}}
        };
        let f=async()=>[...await source,(events.push('tail'),await 5)];
        f().then(value=>{result=value});
    "#).unwrap();
    check(&mut realm, "events.length===0&&result===undefined");
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='iterator,next,next,next,tail'&&result.length===3&&result[0]===promise&&result[1]===4&&result[2]===5",
    );
}

#[test]
fn nested_array_and_call_tasks_use_intrinsic_arrays_and_do_not_infer_element_names() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let result,setterCalls=0,saved=Array;
        Object.defineProperty(Array.prototype,'0',{set(){setterCalls++},configurable:true});
        globalThis.Array=function(){throw 1};
        function target(value){return value}
        async function f(){return target([await 1,[await function(){},await class{}],,])}
        f().then(value=>{result=value});
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "setterCalls===0&&result instanceof saved&&result.length===3&&result[0]===1&&result[1][0].name===''&&result[1][1].name===''&&!(2 in result)",
    );
}

#[test]
fn rejection_skips_later_elements_and_preserves_resource_suppression() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let marker={},cleanup={},reason,later=0,disposed=0;
        async function f(){using resource={[Symbol.dispose](){disposed++;throw cleanup}};return[1,await Promise.reject(marker),(later++,3)]}
        f().catch(value=>{reason=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "later===0&&disposed===1&&reason instanceof SuppressedError&&reason.error===cleanup&&reason.suppressed===marker",
    );
}

#[test]
fn partial_arrays_keep_earlier_values_alive_and_release_them_after_rejection() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},reason,marker={};state.gate=new Promise((resolve,reject)=>{state.reject=reject});
        let first={x:7};state.weak=new WeakRef(first);
        async function f(){return[first,await state.gate]}
        f().catch(value=>{reason=value});first=null;
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
fn resumed_array_capacity_failure_bypasses_rejection_disposal_and_iterator_closing() {
    let mut realm = Realm::new(Limits {
        max_properties: Some(100),
        ..Default::default()
    });
    realm.eval(r#"
        let state={},steps=0,closed=0,disposed=0,caught=false;
        state.gate=new Promise(resolve=>{state.release=resolve});
        let source={[Symbol.iterator](){return{next(){return{done:++steps>200,value:1}},return(){closed++;return{}}}}};
        async function f(){using resource={[Symbol.dispose](){disposed++}};return[await state.gate,...source]}
        f().catch(()=>{caught=true});
    "#).unwrap();
    realm.eval("state.release(0)").unwrap();
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
    check(
        &mut realm,
        "steps>0&&steps<200&&closed===0&&disposed===0&&!caught",
    );
}
