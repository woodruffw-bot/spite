//! Catch parameter Array BindingInitialization across Await (14.15.3).

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
fn parameter_defaults_finish_before_the_body_and_catch_scope_ends_before_finally() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let x=1,release,gate=new Promise(resolve=>{release=resolve}),entered=false,finallySeen=0,result;
        async function f(){try{throw []}catch([x=await gate]){entered=true;x++;return x}finally{await 0;finallySeen=x}}f().then(value=>{result=value});
    "#).unwrap();
    check(
        &mut realm,
        "!entered&&finallySeen===0&&x===1&&result===undefined",
    );
    realm.eval("release(7)").unwrap();
    drain(&mut realm);
    check(&mut realm, "entered&&finallySeen===1&&x===1&&result===8");
}

#[test]
fn nested_patterns_rest_and_default_name_inference_keep_native_binding_behavior() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result;async function f(){try{throw [[],undefined,undefined,11,12]}catch([[x=await 7],fn=function(){},anon=await(()=>{}),...tail]){return x===7&&fn.name==='fn'&&anon.name===''&&tail.join(',')==='11,12'}}f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result===true");
}

#[test]
fn parameter_tdz_errors_replace_the_throw_and_reach_finally_then_the_outer_handler() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],entered=0,result;async function f(){try{try{throw []}catch([x=await x]){entered++}finally{await 0;events.push('finally')}}catch(error){events.push('outer');return error instanceof ReferenceError}}f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===true&&entered===0&&events.join(',')==='finally,outer'",
    );
}

#[test]
fn rejected_defaults_close_the_iterator_before_finally_without_reentering_the_handler() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let reason={},events=[],entered=0,result;
        let source={[Symbol.iterator](){return this},next(){return{done:false,value:undefined}},return(){events.push('close');throw 0}};
        async function f(){try{try{throw source}catch([x=await Promise.reject(reason)]){entered++}finally{await 0;events.push('finally')}}catch(error){events.push('outer');return error===reason}}f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===true&&entered===0&&events.join(',')==='close,finally,outer'",
    );
}

#[test]
fn iterator_acquisition_step_and_close_errors_skip_the_body_and_unwind_the_catch_scope() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let reason={},entered=0,finalized=0,results=[];
        function source(kind){if(kind==='null')return null;return{get [Symbol.iterator](){if(kind==='iterator')throw reason;return function(){return this}},get next(){if(kind==='nextGetter')throw reason;return function(){if(kind==='next')throw reason;return{get done(){if(kind==='done')throw reason;return false},get value(){if(kind==='value')throw reason;return 1}}}},get return(){if(kind==='closeGetter')throw reason;return function(){throw reason}}}}
        async function f(kind){try{try{throw source(kind)}catch([x=await 7]){entered++}finally{await 0;finalized++}}catch(error){return kind==='null'?error instanceof TypeError:error===reason}}
        for(const kind of ['null','iterator','nextGetter','next','done','value','closeGetter','close'])f(kind).then(value=>results.push(value));
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "entered===0&&finalized===8&&results.length===8&&results.every(x=>x===true)",
    );
}

#[test]
fn suspension_retains_cached_next_then_releases_it_while_escaped_catch_bindings_stay_live() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,gate=new Promise(resolve=>{release=resolve}),weak,nextWeak,iteratorWeak,result,get;
        function source(){let keep={};weak=new WeakRef(keep);let next=()=>{keep;return{done:false,value:undefined}};nextWeak=new WeakRef(next);let iterator={next,return(){return{done:true}}};iteratorWeak=new WeakRef(iterator);return{[Symbol.iterator](){return iterator}}}
        async function f(){try{throw source()}catch([x=await gate]){get=()=>x;return x}}f().then(value=>{result=value});iteratorWeak.deref().next=null;
    "#).unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "weak.deref()!==undefined&&nextWeak.deref()!==undefined&&iteratorWeak.deref()!==undefined&&result===undefined",
    );
    realm.eval("release(7)").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "result===7&&get()===7&&weak.deref()===undefined&&nextWeak.deref()===undefined&&iteratorWeak.deref()===undefined",
    );
}

#[test]
fn host_abort_in_parameters_skips_close_disposal_finally_body_and_rejection_handlers() {
    for limited in [false, true] {
        let mut realm = Realm::new(Limits {
            max_steps: limited.then_some(10_000),
            ..Default::default()
        });
        realm.eval("let state={},closed=0,disposed=0,finalized=0,entered=0,caught=false,proceed,gate=new Promise(resolve=>{proceed=resolve});const dispose=Symbol.dispose;").unwrap();
        realm.eval("let resource={[dispose](){disposed++}};state.weak=new WeakRef(resource[dispose]);function spin(){while(true){}};let source={[Symbol.iterator](){return this},next(){return{done:false,value:undefined}},return(){closed++;return{done:true}}}").unwrap();
        let tail = if limited {
            "spin()"
        } else {
            "new SharedArrayBuffer(0)"
        };
        realm.eval(&format!("async function f(){{using r=resource;try{{throw source}}catch([x=(await gate,{tail})]){{entered++}}finally{{finalized++}}}}f().catch(()=>{{caught=true}})")).unwrap();
        realm.eval("delete resource[dispose];proceed()").unwrap();
        let mut reported = false;
        loop {
            match realm.run_next_promise_job().unwrap() {
                PromiseJobOutcome::NoJob => break,
                PromiseJobOutcome::Completed => (),
                PromiseJobOutcome::Reported(error) => {
                    assert!(matches!(error, Error::Limit { .. }) == limited, "{error}");
                    if !limited {
                        assert!(matches!(error, Error::Unsupported { .. }), "{error}");
                    }
                    reported = true;
                }
            }
        }
        assert!(reported);
        collect(&mut realm);
        check(
            &mut realm,
            "closed===0&&disposed===0&&finalized===0&&entered===0&&!caught&&state.weak.deref()===undefined",
        );
    }
}

#[test]
fn private_method_context_and_dynamic_async_function_bodies_share_catch_binding_execution() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let results=[];class C{#x=7;async f(){try{throw []}catch([x=await this.#x]){return x}}}
        new C().f().then(value=>results.push(value));let AsyncFunction=(async function(){}).constructor;
        AsyncFunction('try{throw []}catch([x=await 13]){return x}')().then(value=>results.push(value));
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results.length===2&&results.includes(7)&&results.includes(13)",
    );
}
