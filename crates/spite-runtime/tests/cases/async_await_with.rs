//! WithStatement suspension and Object Environment references (14.11, 9.1.1.2).

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
fn awaited_with_objects_use_the_outer_scope_and_bodies_observe_live_properties() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let x=1,object={x:2},release,gate=new Promise(resolve=>{release=resolve}),events=[],result;
        async function f(){with(await(events.push(x),object)){events.push(x);await gate;{let x=4;events.push(x)}return x}}f().then(value=>{result=value});
    "#).unwrap();
    check(
        &mut realm,
        "events.join(',')==='1'&&x===1&&result===undefined",
    );
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='1,2'&&x===1&&result===undefined",
    );
    realm.eval("object.x=3;release()").unwrap();
    drain(&mut realm);
    check(&mut realm, "events.join(',')==='1,2,4'&&result===3&&x===1");
}

#[test]
fn awaited_var_initializers_save_object_references_before_rhs_and_patterns_resolve_after_rhs() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let first,second,a=new Promise(resolve=>{first=resolve}),b=new Promise(resolve=>{second=resolve}),result;
        let blocked={x:false,y:true},object={x:1,y:2,[Symbol.unscopables]:blocked};
        async function f(){with(object){var x=await a,{y}=await b;}return [x,y]}f().then(value=>{result=value});
    "#).unwrap();
    realm.eval("blocked.x=true;first(7)").unwrap();
    drain(&mut realm);
    check(&mut realm, "object.x===7&&object.y===2&&result===undefined");
    realm.eval("blocked.y=false;second({y:9})").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "object.x===7&&object.y===9&&result.length===2&&result[0]===undefined&&result[1]===undefined",
    );
}

#[test]
fn awaited_with_calls_keep_binding_receivers_and_direct_eval_uses_the_restored_scope() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result,object={x:7,method(value){return this===object&&value===9}},bare,dynamic;
        async function f(){with(await object){let called=method(await 9);await 0;eval('x=11');return called&&eval('x')===11}}f().then(value=>{result=value});
        async function g(){with(await {y:13})return await y}g().then(value=>{bare=value});
        (async function(){}).constructor('with(await {z:15})return await z')().then(value=>{dynamic=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===true&&object.x===11&&typeof x==='undefined'&&bare===13&&typeof y==='undefined'&&dynamic===15&&typeof z==='undefined'",
    );
}

#[test]
fn with_boxes_primitives_and_nullish_or_rejected_objects_do_not_enter_the_body() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let results=[],entered=0,reason={};async function f(value){with(await value){await 0;return length}}
        async function g(value){try{with(await value){entered++}}catch(error){return error===reason||error instanceof TypeError}}
        f('abc').then(value=>results.push(value));g(null).then(value=>results.push(value));g(undefined).then(value=>results.push(value));g(Promise.reject(reason)).then(value=>results.push(value));
        async function symbol(){with(Symbol('x')){await 0;return typeof valueOf()==='symbol'}}symbol().then(value=>results.push(value));
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "entered===0&&results.length===5&&results.includes(3)&&results.filter(x=>x===true).length===4",
    );
}

#[test]
fn dynamic_unscopables_changes_affect_later_lookups_without_replacing_cached_assignment_targets() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let x=1,release,gate=new Promise(resolve=>{release=resolve}),blocked={x:false},object={x:2,[Symbol.unscopables]:blocked},result;
        async function f(){with(object){x=await gate;return x}}f().then(value=>{result=value});
    "#).unwrap();
    realm.eval("blocked.x=true;release(9)").unwrap();
    drain(&mut realm);
    check(&mut realm, "object.x===9&&x===1&&result===1");
}

#[test]
fn with_restores_scope_after_labelled_control_awaited_finalizers_and_block_disposal() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let x=1,events=[],result;async function f(){outer:for(let i=0;i<2;i++){with({x:i+7}){using r={[Symbol.dispose](){events.push(x)}};try{await 0;if(i===0)continue outer;break outer}finally{await 0;events.push(x)}}}return x}f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===1&&x===1&&events.join(',')==='7,7,8,8'",
    );
    realm.eval(r#"
        let failure;async function g(){try{with(await{x:99}){throw await 0}}catch{await 0;return x}}g().then(value=>{failure=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "failure===1&&x===1");
}

#[test]
fn pending_with_environments_and_escaped_closures_keep_objects_live_until_released() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,gate=new Promise(resolve=>{release=resolve}),weak,result,get;
        function make(){const object={x:7};weak=new WeakRef(object);return object}
        async function f(){with(make()){await gate;get=()=>x;return x}}f().then(value=>{result=value});
    "#).unwrap();
    collect(&mut realm);
    check(&mut realm, "weak.deref()!==undefined&&result===undefined");
    realm.eval("release()").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "weak.deref()!==undefined&&get()===7&&result===7",
    );
    realm.eval("get=null").unwrap();
    collect(&mut realm);
    check(&mut realm, "weak.deref()===undefined");
}

#[test]
fn host_abort_restores_with_scope_and_skips_disposal_catch_and_finally() {
    for limited in [false, true] {
        let mut realm = Realm::new(Limits {
            max_steps: limited.then_some(10_000),
            ..Default::default()
        });
        realm.eval("let x=1,state={},disposed=0,finalized=0,caught=false,proceed,gate=new Promise(resolve=>{proceed=resolve});const dispose=Symbol.dispose;").unwrap();
        realm.eval("let resource={[dispose](){disposed++}};state.weak=new WeakRef(resource[dispose]);function spin(){while(true){}}").unwrap();
        let tail = if limited {
            "spin()"
        } else {
            "new SharedArrayBuffer(0)"
        };
        let code="async function f(){try{with(await {x:9}){using r=resource;await gate;TAIL}}catch{caught=true}finally{finalized++}}f().catch(()=>{caught=true})".replace("TAIL",tail);
        realm.eval(&code).unwrap();
        drain(&mut realm);
        realm.eval("delete resource[dispose];proceed()").unwrap();
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
            "x===1&&disposed===0&&finalized===0&&!caught&&state.weak.deref()===undefined",
        );
    }
}
