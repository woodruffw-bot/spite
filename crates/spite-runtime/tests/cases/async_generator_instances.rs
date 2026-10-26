//! Serialized async-generator requests and shared source suspension (27.6.3).

use spite_runtime::{Error, Limits, PromiseJobOutcome, Realm, Value};

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
fn collect(realm: &mut Realm) {
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn calls_initialize_parameters_and_instances_before_lazy_body_entry() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        "let events=[],results=[];async function*g(x=(events.push('parameter'),1)){events.push('body')}let iterator=g(),GP=Object.getPrototypeOf(g).prototype;events.join(',')==='parameter'&&Object.getPrototypeOf(iterator)===g.prototype&&Object.getPrototypeOf(g.prototype)===GP&&iterator[Symbol.asyncIterator]()===iterator&&Object.prototype.toString.call(iterator)==='[object AsyncGenerator]'",
    );
    check(
        &mut realm,
        "let promise=iterator.next(99);promise.then(r=>results.push(r));events.join(',')==='parameter,body'&&results.length===0",
    );
    drain(&mut realm);
    check(
        &mut realm,
        "results.length===1&&results[0].done&&results[0].value===undefined&&Reflect.ownKeys(results[0]).join(',')==='value,done'",
    );
}

#[test]
fn awaited_yields_returns_and_queued_next_values_preserve_fifo_order() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        "let events=[],results=[];async function*g(x=(events.push('parameter'),1)){events.push('body');let received=yield x;events.push(received);yield await received;return received+1}let iterator=g();for(let promise of [iterator.next(99),iterator.next(4),iterator.next(),iterator.next()])promise.then(r=>results.push(r));events.join(',')==='parameter,body'&&results.length===0",
    );
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='parameter,body,4'&&results.length===4&&results.map(r=>r.done).join(',')==='false,false,true,true'&&results[0].value===1&&results[1].value===4&&results[2].value===5&&results[3].value===undefined&&new Set(results).size===4",
    );
}

#[test]
fn initial_throw_and_return_skip_bodies_and_completed_requests_remain_serialized() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        "let body=0,results=[],reason=Symbol();async function*g(){body++}let a=g(),b=g();a.throw(reason).catch(e=>results.push(e===reason));a.next().then(r=>results.push(r.done));let release;let gate=new Promise(r=>release=r);b.return(gate).then(r=>results.push(r.value===7&&r.done));b.next(8).then(r=>results.push(r.value===undefined&&r.done));b.throw(reason).catch(e=>results.push(e===reason));body===0&&results.length===0",
    );
    drain(&mut realm);
    check(&mut realm, "results.join(',')==='true,true'&&body===0");
    realm.eval("release(7)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results.length===5&&results.every(value=>value)&&body===0",
    );
}

#[test]
fn return_arguments_await_before_finalizers_and_rejections_enter_generator_catch() {
    let mut realm = Realm::default();
    realm.eval("let events=[],results=[],release;async function*g(){try{yield 1}catch(e){events.push(e);yield 'recovered'}finally{events.push('finally')}}let a=g(),b=g();a.next();b.next()").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "let gate=new Promise(r=>release=r);a.return(gate).then(r=>results.push(r));b.return(Promise.reject('reason')).then(r=>results.push(r));events.length===0",
    );
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='reason'&&results.length===1&&results[0].value==='recovered'&&!results[0].done",
    );
    realm.eval("release(7)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='reason,finally'&&results[1].value===7&&results[1].done",
    );
    realm.eval("b.next()").unwrap();
    drain(&mut realm);
    check(&mut realm, "events.join(',')==='reason,finally,finally'");
}

#[test]
fn queued_throws_wait_for_current_await_and_body_errors_reject_without_synchronous_throw() {
    let mut realm = Realm::default();
    realm.eval("let release,events=[],results=[];let gate=new Promise(r=>release=r);async function*g(){try{await gate;yield 1}finally{events.push('finally')}}let iterator=g();iterator.next().then(r=>results.push(r.value));iterator.throw('reason').catch(e=>results.push(e));iterator.next().then(r=>results.push(r.done))").unwrap();
    drain(&mut realm);
    check(&mut realm, "results.length===0&&events.length===0");
    realm.eval("release()").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results.join(',')==='1,reason,true'&&events.join(',')==='finally'",
    );
    realm
        .eval("async function*h(){throw 7}let rejected=false;h().next().catch(e=>rejected=e===7)")
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "rejected");
}

#[test]
fn return_expressions_await_before_finalizers_and_draining_constructor_errors_keep_queue_order() {
    let mut realm = Realm::default();
    realm.eval("let events=[],result;async function*g(){try{return {get then(){events.push('get');return resolve=>{events.push('then');resolve(7)}}}}finally{events.push('finally')}}g().next().then(r=>result=r)").unwrap();
    check(&mut realm, "events.join(',')==='get'&&result===undefined");
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='get,then,finally'&&result.value===7&&result.done",
    );
    realm.eval("events=[];let iterator=(async function*(){})(),poisoned=Promise.resolve(1);Object.defineProperty(poisoned,'constructor',{get(){events.push('constructor');iterator.next().then(r=>events.push(r.done?'next':'bad'));throw 9}});iterator.return(poisoned).catch(e=>events.push(e===9?'catch':'bad'))").unwrap();
    check(&mut realm, "events.join(',')==='constructor'");
    drain(&mut realm);
    check(&mut realm, "events.join(',')==='constructor,next,catch'");
}

#[test]
fn iterator_result_then_getters_enqueue_without_reentering_the_executing_body() {
    let mut realm = Realm::default();
    realm.eval("let events=[],results=[],added=false;async function*g(){yield 1;events.push('body-after');yield 2}let iterator=g();Object.defineProperty(Object.prototype,'then',{configurable:true,get(){if(Object.hasOwn(this,'done')&&!added){added=true;events.push('getter-start');iterator.next().then(r=>results.push(r.value));events.push('getter-end')}return undefined}});iterator.next().then(r=>results.push(r.value))").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='getter-start,getter-end,body-after'&&results.join(',')==='1,2'",
    );
}

#[test]
fn private_methods_this_super_and_awaited_binding_state_use_shared_source_contexts() {
    let mut realm = Realm::default();
    realm.eval("let results=[];class B{get value(){return this.tag}}class C extends B{tag=7;async*#g(argument){let {x=await Promise.resolve(2)}=argument;yield this.tag;yield super.value;return x}get(){return this.#g}}let object=new C(),method=object.get(),iterator=method.call(object,{});for(let promise of [iterator.next(),iterator.next(),iterator.next()])promise.then(r=>results.push(r.value))").unwrap();
    drain(&mut realm);
    check(&mut realm, "results.join(',')==='7,7,2'");
}

#[test]
fn async_consumers_and_disposal_close_the_source_without_body_replay() {
    let mut realm = Realm::default();
    realm.eval("let events=[],arrayResult,loopResult;async function*g(){using r={[Symbol.dispose](){events.push('dispose')}};try{yield 1;yield 2}finally{events.push('finally')}}Array.fromAsync(g(),x=>x*2).then(a=>arrayResult=a);(async()=>{let result=[];for await(const value of g()){result.push(value);break}return result})().then(a=>loopResult=a)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "arrayResult.join(',')==='2,4'&&loopResult.join(',')==='1'&&events.filter(e=>e==='finally').length===2&&events.filter(e=>e==='dispose').length===2",
    );
    realm.eval("let iterator=g(),disposed=false;iterator.next().then(()=>iterator[Symbol.asyncDispose]()).then(()=>disposed=true)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "disposed&&events.slice(-2).join(',')==='finally,dispose'",
    );
}

#[test]
fn queued_symbol_values_and_suspended_captures_are_traced_and_unreachable_cycles_release() {
    let mut realm = Realm::default();
    realm.eval("let weak,release,iterator,first;let gate=new Promise(r=>release=r);iterator=(()=>{let held=Symbol();weak=new WeakRef(held);return(async function*(){await gate;yield held})()})();first=iterator.next()").unwrap();
    collect(&mut realm);
    check(&mut realm, "weak.deref()!==undefined");
    realm.eval("release();release=null").unwrap();
    drain(&mut realm);
    realm.eval("iterator=null;first=null;gate=null").unwrap();
    collect(&mut realm);
    check(&mut realm, "weak.deref()===undefined");
    realm.eval("let queuedWeak,block=new Promise(()=>{}),queuedIterator=(async function*(){})();queuedIterator.return(block);(()=>{let held=Symbol();queuedWeak=new WeakRef(held);queuedIterator.next(held)})()").unwrap();
    collect(&mut realm);
    check(&mut realm, "queuedWeak.deref()!==undefined");
    realm.eval("queuedIterator=null;block=null").unwrap();
    collect(&mut realm);
    check(&mut realm, "queuedWeak.deref()===undefined");
}

#[test]
fn host_aborts_complete_instances_skip_javascript_cleanup_and_release_captures() {
    let mut realm = Realm::default();
    realm.reserve_unsupported_global("unsupportedHost");
    realm.eval("let weak,closed=0,caught=0,finalized=0;let resource={[Symbol.dispose](){closed++}};weak=new WeakRef(resource[Symbol.dispose]);async function*g(){using r=resource;delete resource[Symbol.dispose];try{yield 1;unsupportedHost}catch{caught++}finally{finalized++}}let iterator=g();iterator.next()").unwrap();
    drain(&mut realm);
    assert!(matches!(
        realm.eval("iterator.next()"),
        Err(Error::Unsupported { .. })
    ));
    collect(&mut realm);
    check(
        &mut realm,
        "closed===0&&caught===0&&finalized===0&&weak.deref()===undefined",
    );
    realm
        .eval("let completed;iterator.next().then(r=>completed=r.done)")
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "completed");
    let mut limited = Realm::new(Limits {
        max_steps: Some(20_000),
        ..Limits::default()
    });
    limited.eval("let finalized=0;async function*g(){try{yield 1;while(true){}}finally{finalized++}}let iterator=g();iterator.next()").unwrap();
    drain(&mut limited);
    assert!(matches!(
        limited.eval("iterator.next()"),
        Err(Error::Limit { .. })
    ));
    check(&mut limited, "finalized===0");
}
