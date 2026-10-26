//! Parsed for-await source remains a host boundary until async stepping lands.

use spite_runtime::{Error, PromiseJobOutcome, Realm, Value};

#[test]
fn for_await_boundary_precedes_rhs_iterator_work_and_bypasses_handlers_and_disposal() {
    let mut realm = Realm::default();
    realm
        .eval("let before=0,operand=0,handled=0,disposed=0;const dispose=Symbol.dispose")
        .unwrap();
    assert!(matches!(realm.eval("async function f(){using r={[dispose](){disposed++}};before++;try{for await(var x of (operand++,[])){} }catch{handled=1}finally{handled=2}}try{f().catch(()=>{handled=3})}catch{handled=4}finally{handled=5}"),Err(Error::Unsupported{..})));
    assert_eq!(
        realm.eval("before===1&&operand===0&&handled===0&&disposed===0"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn unexecuted_for_await_source_and_dynamic_async_functions_keep_their_real_syntax() {
    let mut realm = Realm::default();
    realm.eval("let result,dynamic;async function f(){if(false){for await(const x of []){}}return 7}f().then(x=>{result=x});let g=(async function(){}).constructor('if(false){for await(using x of []){}}return 9');g().then(x=>{dynamic=x})").unwrap();
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => (),
            PromiseJobOutcome::Reported(error) => panic!("{error}"),
        }
    }
    assert_eq!(realm.eval("result===7&&dynamic===9&&f.toString().includes('for await')&&g.toString().includes('for await')"),Ok(Value::Boolean(true)));
}
