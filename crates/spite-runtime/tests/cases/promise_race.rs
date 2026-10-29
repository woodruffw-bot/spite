//! Promise.race subscription ordering, iterator cleanup and host failures.

use crate::common::UnavailableHostRealm;
use spite_runtime::{Error, Limits, PromiseJobOutcome, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

fn drain(realm: &mut Realm) {
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => {}
            PromiseJobOutcome::Reported(error) => panic!("{error}"),
        }
    }
}

#[test]
fn capability_resolve_lookup_and_all_subscriptions_are_ordered_and_generic() {
    check(
        "let log='',firstResolve,firstReject,values=[],result={};function C(executor){log+='c';executor(function(v){'use strict';if(this!==undefined||arguments.length!==1)throw 1;values.push(v)},()=>{throw 2});return result}Object.defineProperty(C,'resolve',{get(){log+='g';return function(v){'use strict';if(this!==C||arguments.length!==1)throw 3;log+='r'+v;return{then(a,b){if(arguments.length!==2)throw 4;if(!firstResolve){firstResolve=a;firstReject=b}else if(a!==firstResolve||b!==firstReject)throw 5;log+='t';a(v)}}}}});let out=Promise.race.call(C,{[Symbol.iterator](){log+='i';let n=0;return{next(){log+='n';return n<3?{value:++n,done:false}:{done:true}},return(){throw 6}}}});out===result&&log==='cginr1tnr2tnr3tn'&&values.join(',')==='1,2,3'",
    );
    check(
        "let log='',result={};function C(executor){log+='c';executor(()=>{},()=>{});return result}Object.defineProperty(C,'resolve',{get(){log+='g';return()=>{throw 1}}});Promise.race.call(C,[])===result&&log==='cg'",
    );
}

#[test]
fn first_settlement_wins_asynchronously_and_empty_input_stays_pending() {
    let mut realm = Realm::default();
    realm.eval("let values=[],finish;Promise.race([new Promise(r=>{finish=r}),Promise.resolve(2),Promise.reject(3)]).then(v=>{values.push(v)},()=>{throw 1});Promise.race([]).then(()=>{throw 2},()=>{throw 3});Promise.race([{then(resolve,reject){reject(4);resolve(5)}},new Promise(()=>{})]).catch(e=>{values.push(e)});if(values.length)throw 4;finish(1)").unwrap();
    drain(&mut realm);
    assert_eq!(
        realm.eval("values.join(',')"),
        Ok(Value::String("2,4".into()))
    );
}

#[test]
fn body_errors_close_but_step_errors_do_not_and_original_throws_win() {
    check(
        "let original={},other={},caught,closed=0;function C(executor){executor(()=>{},e=>{caught=e})}C.resolve=v=>v;let iterable={[Symbol.iterator](){return{next(){return{value:{get then(){throw original}},done:false}},get return(){closed++;throw other}}}};Promise.race.call(C,iterable);caught===original&&closed===1",
    );
    check(
        "let original={},caught,closed=0;function C(executor){executor(()=>{},e=>{caught=e})}C.resolve=v=>v;Promise.race.call(C,{[Symbol.iterator](){return{next(){return{done:false,get value(){throw original}}},return(){closed++;return{}}}}});caught===original&&closed===0",
    );
    check(
        "let original={},other={},touched=false;function C(executor){executor(()=>{},e=>{if(e!==original)throw 1;throw other})}Object.defineProperty(C,'resolve',{get(){throw original}});let caught;try{Promise.race.call(C,{get [Symbol.iterator](){touched=true}})}catch(e){caught=e}caught===other&&!touched",
    );
}

#[test]
fn host_failures_bypass_rejection_iterator_return_and_language_handlers() {
    let mut realm = Realm::with_unavailable_host();
    let result=realm.eval("let closed=0,rejected=0,caught=0;function C(executor){executor(()=>{},()=>{rejected++})}C.resolve=()=>spiteUnavailableHost();let iterable={[Symbol.iterator](){return{next(){return{value:1,done:false}},return(){closed++;return{}}}}};try{Promise.race.call(C,iterable)}catch(e){caught++}finally{caught++}");
    assert!(
        matches!(result, Err(Error::Unsupported { .. })),
        "{result:?}"
    );
    assert_eq!(realm.eval("closed+rejected+caught"), Ok(Value::Number(0.0)));
    let mut limited = Realm::with_unavailable_host_limits(Limits {
        max_steps: Some(100_000),
        ..Limits::default()
    });
    let result=limited.eval("let closed=0,rejected=0;function C(executor){executor(()=>{},()=>{rejected++})}C.resolve=()=>{while(true){}};Promise.race.call(C,{[Symbol.iterator](){return{next(){return{value:1,done:false}},return(){closed++;return{}}}}})");
    assert!(matches!(result, Err(Error::Limit { .. })), "{result:?}");
    assert_eq!(limited.eval("closed+rejected"), Ok(Value::Number(0.0)));
}
