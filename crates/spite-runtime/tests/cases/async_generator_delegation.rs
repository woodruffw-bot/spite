//! Async YieldExpression delegation (15.5) through the shared source interpreter.

use spite_runtime::{Error, PromiseJobOutcome, Realm, Value};

fn check(realm: &mut Realm, source: &str) {
    assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
}
fn drain(realm: &mut Realm) {
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => (),
            PromiseJobOutcome::Reported(error) => panic!("{error:?}"),
        }
    }
}

#[test]
fn async_results_are_awaited_cached_next_receives_one_argument_and_values_stay_raw() {
    let mut realm = Realm::default();
    realm.eval("let events=[],results=[],count=0,raw=Promise.resolve(7),release;let inner={get next(){events.push('get-next');return function(value){events.push([this===inner,arguments.length,value].join(':'));return new Promise(r=>release=r)}},[Symbol.asyncIterator](){events.push('async');return this},[Symbol.iterator](){throw 99}};async function*g(){return yield*inner}let iterator=g();iterator.next(99).then(r=>results.push(r));iterator.next(4).then(r=>results.push(r))").unwrap();
    check(
        &mut realm,
        "events.join(',')==='async,get-next,true:1:'&&results.length===0",
    );
    realm.eval("Object.defineProperty(inner,'next',{value(){throw 98}});release({get done(){events.push('done');return false},get value(){events.push('value');return raw}})").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results.length===1&&results[0].value===raw&&!results[0].done&&events.join(',')==='async,get-next,true:1:,done,value,true:1:4'",
    );
    realm.eval("release({done:true,value:8})").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results.length===2&&results[1].done&&results[1].value===8",
    );
}

#[test]
fn sync_fallback_unwraps_values_and_nested_async_delegation_serves_consumers() {
    let mut realm = Realm::default();
    realm.eval("let results=[],array,events=[];async function*g(){try{return yield*[Promise.resolve(1),2]}finally{events.push('finally')}}async function*h(){yield*g()}let iterator=h();iterator.next().then(r=>results.push(r.value));iterator.next().then(r=>results.push(r.value));iterator.next().then(r=>results.push(r.done));Array.fromAsync(h()).then(a=>array=a)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results.join(',')==='1,2,true'&&array.join(',')==='1,2'&&events.join(',')==='finally,finally'",
    );
}

#[test]
fn live_throw_and_return_methods_await_results_and_return_can_keep_delegating() {
    let mut realm = Realm::default();
    realm.eval("let events=[],results=[],count=0;let inner={[Symbol.asyncIterator](){return this},next(value){events.push('next:'+value);return {done:false,value:++count}}};async function*g(){try{return yield*inner}finally{events.push('finally')}}let iterator=g();iterator.next().then(r=>results.push(r))").unwrap();
    drain(&mut realm);
    realm.eval("inner.return=function(value){events.push('return:'+value+':'+arguments.length);return Promise.resolve({done:false,value:9})};iterator.return(Promise.resolve(4)).then(r=>results.push(r))").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results[1].value===9&&!results[1].done&&events.join(',')==='next:undefined,return:4:1'",
    );
    realm.eval("inner.throw=function(value){events.push('throw:'+value+':'+arguments.length);return Promise.resolve({done:true,value:10})};iterator.throw(7).then(r=>results.push(r))").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results[2].done&&results[2].value===10&&events.join(',')==='next:undefined,return:4:1,throw:7:1,finally'",
    );

    // 27.1.4.5.3-4 reject abrupt GetMethod/Call/result-field operations.
    // They neither IteratorClose nor bypass yield*'s Await of that rejection.
    for (method, expected, prefix) in [
        (
            "get throw(){events.push('get');throw reason}",
            "error===reason",
            "get",
        ),
        (
            "get throw(){events.push('get');return 1}",
            "error instanceof TypeError",
            "get",
        ),
        (
            "throw(value){events.push('call:'+value);return 1}",
            "error instanceof TypeError",
            "call:7",
        ),
        (
            "throw(value){events.push('call:'+value);throw reason}",
            "error===reason",
            "call:7",
        ),
        (
            "get throw(){events.push('get');return class Uncallable{}}",
            "error instanceof TypeError",
            "get",
        ),
        (
            "get throw(){events.push('get');let pair=Proxy.revocable(()=>{},{});pair.revoke();return pair.proxy}",
            "error instanceof TypeError",
            "get",
        ),
        (
            "throw(value){events.push('call:'+value);return {get done(){events.push('done');throw reason}}}",
            "error===reason",
            "call:7,done",
        ),
        (
            "throw(value){events.push('call:'+value);return {done:false,get value(){events.push('value');throw reason}}}",
            "error===reason",
            "call:7,value",
        ),
    ] {
        let mut realm = Realm::default();
        realm.eval(&format!(
            "let events=[],outcomes=[],reason={{}},inner={{[Symbol.iterator](){{return this}},next(){{return {{done:false,value:1}}}},return(){{events.push('close');return {{done:true}}}},{method}}};async function*g(){{try{{yield*inner}}finally{{events.push('finally')}}}}let iterator=g(),first=iterator.next(),failed=iterator.throw(7),last=iterator.next();first.then(result=>{{events.push('first');outcomes.push(result.value===1&&!result.done)}});failed.catch(error=>{{events.push('rejected');outcomes.push({expected})}});last.then(result=>{{events.push('last');outcomes.push(result.done)}})"
        )).unwrap();
        realm.collect(usize::MAX).unwrap();
        loop {
            match realm.run_next_promise_job().unwrap() {
                PromiseJobOutcome::NoJob => break,
                PromiseJobOutcome::Completed => {
                    realm.collect(usize::MAX).unwrap();
                }
                PromiseJobOutcome::Reported(error) => panic!("{error:?}"),
            }
        }
        check(
            &mut realm,
            &format!(
                "outcomes.join(',')==='true,true,true'&&events.join(',')==='{prefix},first,finally,rejected,last'"
            ),
        );
    }
}

#[test]
fn missing_throw_awaits_close_without_reading_done_or_value_and_cleanup_failure_wins() {
    for (close, expected) in [
        (
            "resolve({get done(){throw 90},get value(){throw 91}})",
            "error instanceof TypeError",
        ),
        ("reject(8)", "error===8"),
        ("resolve(1)", "error instanceof TypeError"),
    ] {
        let mut realm = Realm::default();
        realm.eval("let events=[],error,resolve,reject;let inner={[Symbol.asyncIterator](){return this},next(){return {done:false,value:1}},return(){events.push(arguments.length);return new Promise((r,j)=>{resolve=r;reject=j})}};async function*g(){try{yield*inner}finally{events.push('finally')}}let iterator=g();iterator.next()").unwrap();
        drain(&mut realm);
        realm.eval("iterator.throw(7).catch(e=>error=e)").unwrap();
        check(&mut realm, "events.join(',')==='0'&&error===undefined");
        realm.eval(close).unwrap();
        drain(&mut realm);
        check(
            &mut realm,
            &format!("{expected}&&events.join(',')==='0,finally'"),
        );
    }
}

#[test]
fn rejected_method_results_reach_outer_catch_instead_of_delegate_throw() {
    let mut realm = Realm::default();
    realm.eval("let results=[],forwarded=0;let inner={[Symbol.asyncIterator](){return this},next(){return Promise.reject(7)},throw(){forwarded++;throw 99}};async function*g(){try{yield*inner}catch(e){yield e}}let iterator=g();iterator.next().then(r=>results.push(r));iterator.next().then(r=>results.push(r))").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "forwarded===0&&results[0].value===7&&!results[0].done&&results[1].done",
    );
}

#[test]
fn failed_next_propagates_without_closing_sync_or_async_delegates() {
    // 15.5 propagates abrupt Call/Await results. The sync adapter's next
    // rejects IteratorNext failures before AsyncFromSyncIteratorContinuation
    // (sec-%asyncfromsynciteratorprototype%.next), without IteratorClose.
    for (protocol, failure) in [
        ("iterator", "throw reason"),
        ("iterator", "return {get done(){throw reason}}"),
        ("iterator", "return {done:false,get value(){throw reason}}"),
        ("asyncIterator", "throw reason"),
        ("asyncIterator", "return {get done(){throw reason}}"),
        (
            "asyncIterator",
            "return {done:false,get value(){throw reason}}",
        ),
        ("asyncIterator", "return Promise.reject(reason)"),
    ] {
        let mut realm = Realm::default();
        realm.eval(&format!(
            "let events=[],results,reason={{}},calls=0,inner={{[Symbol.{protocol}](){{return this}},next(){{events.push('next');if(calls++===0)return{{value:1,done:false}};{failure}}},get return(){{events.push('return');throw 99}}}};async function*outer(){{try{{yield*inner}}catch(e){{yield e===reason}}}}let iterator=outer();Promise.all([iterator.next(),iterator.next(),iterator.next()]).then(value=>results=value)"
        )).unwrap();
        drain(&mut realm);
        check(
            &mut realm,
            "events.join(',')==='next,next'&&results[0].value===1&&!results[0].done&&results[1].value===true&&!results[1].done&&results[2].done&&results[2].value===undefined",
        );
    }
}

#[test]
fn completed_return_preserves_raw_value_while_missing_return_performs_its_own_await() {
    let mut realm = Realm::default();
    realm.eval("let results=[],raw=Promise.resolve(7);let inner={[Symbol.asyncIterator](){return this},next(){return {done:false,value:1}},return(){return {done:true,value:raw}}};async function*g(){try{yield*inner}finally{}}let iterator=g();iterator.next()").unwrap();
    drain(&mut realm);
    realm
        .eval("iterator.return(4).then(r=>results.push(r))")
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "results[0].done&&results[0].value===raw");
    realm.eval("let events=[],other=(async function*(){try{yield*{[Symbol.asyncIterator](){return this},next(){return {done:false,value:1}}}}finally{events.push('finally')}})();other.next()").unwrap();
    drain(&mut realm);
    realm.eval("other.return(5).then(r=>events.push(r.value));Promise.resolve().then(()=>events.push('tick'))").unwrap();
    drain(&mut realm);
    check(&mut realm, "events.join(',')==='tick,finally,5'");
}

#[test]
fn delegation_records_keep_iterators_and_cached_next_alive_then_release_them() {
    let mut realm = Realm::default();
    realm.eval("let iterator,weakIterator,weakNext,release;(()=>{let held=Symbol(),next=function(){return new Promise(r=>release=r)},inner={[Symbol.asyncIterator](){return this},next};next.held=held;weakIterator=new WeakRef(inner);weakNext=new WeakRef(held);iterator=(async function*(){yield*inner})()})();iterator.next()").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "weakIterator.deref()!==undefined&&weakNext.deref()!==undefined",
    );
    realm.eval("release({done:true});release=null").unwrap();
    drain(&mut realm);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "weakIterator.deref()===undefined&&weakNext.deref()===undefined",
    );

    realm.eval("let weakQueued,queuedSeen=false,queuedPromise;let queuedInner={[Symbol.asyncIterator](){return this},next(){return new Promise(r=>release=r)},throw(value){queuedSeen=value===weakQueued.deref();return {done:true,value:7}}};let queued=(async function*(){return yield*queuedInner})();queued.next();(()=>{let held={};weakQueued=new WeakRef(held);queuedPromise=queued.throw(held)})();queuedPromise.then(result=>queuedSeen=queuedSeen&&result.done&&result.value===7)").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(&mut realm, "weakQueued.deref()!==undefined");
    realm
        .eval("release({done:false,value:1});release=null")
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "queuedSeen");
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(&mut realm, "weakQueued.deref()===undefined");
}

#[test]
fn host_abort_from_delegate_bypasses_outer_and_iterator_cleanup() {
    let mut realm = Realm::default();
    realm.reserve_unsupported_global("unsupportedHost");
    realm.eval("let caught=0,finalized=0,closed=0;let inner={[Symbol.asyncIterator](){return this},next(){unsupportedHost},return(){closed++;return {done:true}}};async function*g(){try{yield*inner}catch{caught++}finally{finalized++}}let iterator=g()").unwrap();
    assert!(matches!(
        realm.eval("iterator.next()"),
        Err(Error::Unsupported { .. })
    ));
    check(&mut realm, "caught===0&&finalized===0&&closed===0");
    realm
        .eval("let done=false;iterator.next().then(r=>done=r.done)")
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "done");
}
