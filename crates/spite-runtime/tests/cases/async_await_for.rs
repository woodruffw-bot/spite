//! ForStatement header phases, iteration environments and source suspension.

use crate::common::UnavailableHostRealm;
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

fn collect(realm: &mut Realm) {
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn for_awaits_all_header_phases_and_body_in_order_without_replaying_initialization() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],result;
        async function f(){for(let i=await(events.push('init'),0);await(events.push('test'+i),i<2);await(events.push('update'+i),i++)){try{events.push('body'+i);await 0}finally{events.push('finally'+i);await 0}}return 7}
        f().then(value=>{result=value});
    "#).unwrap();
    check(&mut realm, "events.join(',')==='init'&&result===undefined");
    drain(&mut realm);
    check(
        &mut realm,
        "result===7&&events.join(',')==='init,test0,body0,finally0,update0,test1,body1,finally1,update1,test2'",
    );
}

#[test]
fn for_copies_let_environments_before_first_test_and_before_each_update() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let heads=[],tests=[],bodies=[],updates=[],result;
        async function f(){for(let i=(heads.push(()=>i),await 0);await(tests.push(()=>i),i<3);await(updates.push(()=>i),i++)){await 0;bodies.push(()=>i)}return 7}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===7&&heads[0]()===0&&tests.map(f=>f()).join(',')==='0,1,2,3'&&bodies.map(f=>f()).join(',')==='0,1,2'&&updates.map(f=>f()).join(',')==='1,2,3'",
    );
}

#[test]
fn for_initializers_share_tdz_patterns_and_mapped_var_argument_behavior() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let tdz,result,alias,patterns=[],a='outer';
        async function f(){let sum=0;for(let a=await 1,b=await(a+1);a<2;a++){await 0;sum=a+b}for(let [x,y]=await [1,2];x<2;x++){await 0;patterns.push(x+y)}return sum}
        async function g(x){for(var x=await 4;x<5;x++)await 0;return arguments[0]}
        async function h(){try{for(let a=await b,b=1;;){}}catch(error){return error instanceof ReferenceError}}
        f().then(value=>{result=value});g(1).then(value=>{alias=value});h().then(value=>{tdz=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===3&&alias===5&&tdz===true&&a==='outer'&&patterns.join(',')==='3'",
    );
}

#[test]
fn for_labelled_continue_updates_after_finally_and_break_skips_update() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],result,updates=0,tailUpdates=0,returned,thrown,reason={},finalized=0;
        async function f(){outer:for(let i=await 0;await(i<4);await(updates++,i++)){try{await 0;if(i<2)continue outer;break outer}finally{events.push(i);await 0}}return 7}
        async function g(){for(let i=0;await true;tailUpdates++){try{await 0;return 11}finally{await 0;finalized++}}}
        async function h(){try{for(let i=0;await true;tailUpdates++){try{await 0;throw reason}finally{await 0;finalized++}}}catch(error){return error===reason}}
        f().then(value=>{result=value});g().then(value=>{returned=value});h().then(value=>{thrown=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===7&&updates===2&&events.join(',')==='0,1,2'&&returned===11&&thrown===true&&tailUpdates===0&&finalized===2",
    );
}

#[test]
fn for_using_header_disposes_once_after_body_finalizers_and_on_initializer_failure() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],result,failed,reason={},disposeError={},i=0;
        let resource={[Symbol.dispose](){events.push('header')}},bad={[Symbol.dispose](){throw disposeError}};
        async function f(){for(using r=await resource;await(i<2);await(i++)){using local={[Symbol.dispose](){events.push('body'+i)}};try{await 0}finally{events.push('finally'+i);await 0}}return 7}
        async function g(){try{for(using first=await bad,second=await Promise.reject(reason);;){}}catch(error){return error instanceof SuppressedError&&error.error===disposeError&&error.suppressed===reason}}
        f().then(value=>{result=value});g().then(value=>{failed=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===7&&failed===true&&events.join(',')==='finally0,body0,finally1,body1,header'",
    );
}

#[test]
fn for_const_header_uses_one_environment_and_restores_the_outer_scope_after_update_errors() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let callbacks=[],result,failed,x='outer';
        async function f(){for(const object=await {i:0};await(object.i<3);await(object.i++)){await 0;callbacks.push(()=>object.i)}return 7}
        async function g(){try{for(const x=await 0;await true;x++){await 0}}catch(error){return error instanceof TypeError&&x==='outer'}}
        f().then(value=>{result=value});g().then(value=>{failed=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===7&&failed===true&&callbacks.map(f=>f()).join(',')==='3,3,3'&&x==='outer'",
    );
}

#[test]
fn for_supports_absent_headers_and_awaited_expression_initializers_without_new_scopes() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result,started=0;
        async function f(){let i=0;for(;;){await 0;if(++i===3)break}for(await(started++);await(i<5);await(i++)){await 0}return i}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result===5&&started===1");
}

#[test]
fn for_rejections_in_each_header_phase_restore_lexical_scopes_before_catch() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let reason={},results=[],x='outer';
        async function a(){try{for(let x=await Promise.reject(reason);;){}}catch(error){return error===reason&&x==='outer'}}
        async function b(){try{for(let x=await 0;await Promise.reject(reason);){}}catch(error){return error===reason&&x==='outer'}}
        async function c(){try{for(let x=await 0;await true;await Promise.reject(reason)){await 0}}catch(error){return error===reason&&x==='outer'}}
        a().then(value=>{results.push(value)});b().then(value=>{results.push(value)});c().then(value=>{results.push(value)});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results.length===3&&results.every(value=>value)&&x==='outer'",
    );
}

#[test]
fn for_traces_header_environments_and_body_values_then_releases_them_after_exit() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,releaseSecond,gate=new Promise(resolve=>{release=resolve}),second=new Promise(resolve=>{releaseSecond=resolve}),bodyWeak,envWeak,first,secondResult;
        function makeBody(){const value={x:1};bodyWeak=new WeakRef(value);return value}
        function makeEnv(){const value={x:2};envWeak=new WeakRef(value);return value}
        async function a(){let n=0;for(;n++===0;await gate){makeBody()}0;await second;return 7}
        async function b(){for(let object=makeEnv();await gate;){break}0;await second;return 8}
        a().then(value=>{first=value});b().then(value=>{secondResult=value});
    "#).unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "bodyWeak.deref()!==undefined&&envWeak.deref()!==undefined&&first===undefined&&secondResult===undefined",
    );
    realm.eval("release(false)").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "bodyWeak.deref()===undefined&&envWeak.deref()===undefined&&first===undefined&&secondResult===undefined",
    );
    realm.eval("releaseSecond(0)").unwrap();
    drain(&mut realm);
    check(&mut realm, "first===7&&secondResult===8");
}

#[test]
fn for_headers_and_bodies_preserve_private_eval_this_and_arguments() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result;class C{#x=10;async read(){let total=0;for(let i=await 0;await(i<arguments.length);await(i++)){await 0;total+=eval('this.#x')}return total}}
        new C().read('a','b').then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result===20");
}

#[test]
fn for_host_abort_releases_header_resources_without_handlers_or_disposal_at_budget_zero() {
    for limited in [false, true] {
        let mut realm = Realm::with_unavailable_host_limits(Limits {
            max_steps: limited.then_some(10_000),
            ..Default::default()
        });
        realm.eval("let state={},disposed=0,finalized=0,caught=false,read;const dispose=Symbol.dispose;").unwrap();
        realm
            .eval("let resource={[dispose](){disposed++}};")
            .unwrap();
        realm
            .eval("state.weak=new WeakRef(resource[dispose]);")
            .unwrap();
        let tail = if limited {
            "while(true){}"
        } else {
            "spiteUnavailableHost()"
        };
        let code="async function f(){using root=resource;try{for(using header=resource;true;){read=()=>header;await 0;TAIL}}finally{finalized++}}".replace("TAIL",tail);
        realm.eval(&code).unwrap();
        realm.eval("f().catch(()=>{caught=true});").unwrap();
        realm.eval("delete resource[dispose];").unwrap();
        let mut reported = false;
        loop {
            match realm.run_next_promise_job().unwrap() {
                PromiseJobOutcome::NoJob => break,
                PromiseJobOutcome::Completed => (),
                PromiseJobOutcome::Reported(error) => {
                    assert!(matches!(error, Error::Limit { .. }) == limited, "{error}");
                    if !limited {
                        assert!(matches!(error, Error::Unsupported { .. }), "{error}")
                    }
                    reported = true;
                }
            }
        }
        assert!(reported);
        collect(&mut realm);
        check(
            &mut realm,
            "disposed===0&&finalized===0&&!caught&&read()===resource&&state.weak.deref()===undefined",
        );
    }
}
