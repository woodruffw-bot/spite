//! For-in enumeration and binding state retained across source Await.

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
fn for_in_awaits_the_rhs_once_and_restores_lexical_tdz_before_the_body() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],result,tdz,key='outer',source={a:1,b:2};
        async function f(){let keys=[];for(let key in await(events.push('rhs'),source)){events.push(key);await 0;keys.push(key)}return keys.join(',')}
        async function g(){try{for(let key in await key){}}catch(error){return error instanceof ReferenceError}}
        f().then(value=>{result=value});g().then(value=>{tdz=value});
    "#).unwrap();
    check(
        &mut realm,
        "events.join(',')==='rhs'&&result===undefined&&key==='outer'",
    );
    drain(&mut realm);
    check(
        &mut realm,
        "result==='a,b'&&tdz===true&&key==='outer'&&events.join(',')==='rhs,a,b'",
    );
}

#[test]
fn for_in_skips_nullish_inputs_without_binding_and_boxes_primitives() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let writes=0,result,target={set x(value){writes++}};
        async function f(){let keys=[];for(target.x in await null){throw 1}for(target.x in await undefined){throw 2}for(const key in await 'ab'){await 0;keys.push(key)}return keys.join(',')}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "writes===0&&result==='0,1'");
}

#[test]
fn for_in_snapshots_own_keys_but_reads_descriptors_and_prototypes_after_suspension() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,gate=new Promise(resolve=>{release=resolve}),seen=[],result,getters=0;
        let source={first:1,deleted:2,get accessor(){getters++;return 3},hiddenLater:5,[Symbol('ignored')]:4};
        Object.defineProperty(source,'shadow',{value:1,enumerable:false,configurable:true});
        Object.setPrototypeOf(source,{old:1});
        async function f(){for(const key in source){seen.push(key);if(key==='first')await gate;else await 0}return 7}
        f().then(value=>{result=value});
    "#).unwrap();
    check(&mut realm, "seen.join(',')==='first'&&result===undefined");
    realm.eval("delete source.deleted;source.added=1;Object.defineProperty(source,'hiddenLater',{enumerable:false});Object.setPrototypeOf(source,{deleted:1,shadow:2,proto:3});release(0)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "seen.join(',')==='first,accessor,deleted,proto'&&getters===0&&result===7",
    );
}

#[test]
fn for_in_preserves_fresh_lexical_bindings_patterns_and_var_argument_aliases() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let closures=[],patterns=[],result,alias,assigned,holder={};
        async function f(){for(let key in await {a:1,b:2}){await 0;closures.push(()=>key)}for(const [head] in await {ab:1,cd:2}){await 0;patterns.push(head)}for(holder.key in await {x:1,y:2})await 0;return 7}
        async function g(x){for(var x in await {a:1,b:2})await 0;return arguments[0]}
        f().then(value=>{result=value});g(1).then(value=>{alias=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===7&&closures[0]()==='a'&&closures[1]()==='b'&&patterns.join(',')==='a,c'&&holder.key==='y'&&alias==='b'",
    );
}

#[test]
fn labelled_for_in_control_finishes_finalizers_and_closes_intervening_for_of() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],result,returns=0,source={a:1,b:2};
        Object.defineProperty(source,'return',{get(){returns++;throw 9}});
        let inner={[Symbol.iterator](){return{next(){return{value:0,done:false}},return(){events.push('close');return{}}}}};
        async function f(){outer:for(const key in source){for(const value of inner){try{await 0;if(key==='a')continue outer;break outer}finally{events.push(key);await 0}}}return 7}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='a,close,b,close'&&returns===0&&result===7",
    );
}

#[test]
fn for_in_binding_failure_rejection_and_return_propagate_without_return_lookup() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let returns=0,source={x:1},first,second,third,reason={};
        Object.defineProperty(source,'return',{get(){returns++;throw 9}});
        async function a(){try{for(const x in await source){await Promise.reject(reason)}}catch(error){return error===reason}}
        async function b(){for(const x in source){await 0;return x}}
        async function c(){try{for(const {x=missing} in await source){await 0}}catch(error){return error instanceof ReferenceError}}
        a().then(value=>{first=value});b().then(value=>{second=value});c().then(value=>{third=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "first===true&&second==='x'&&third===true&&returns===0",
    );
}

#[test]
fn for_in_traces_each_current_object_and_releases_previous_objects_and_finished_state() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,releaseSecond,releaseThird,gate=new Promise(resolve=>{release=resolve}),second=new Promise(resolve=>{releaseSecond=resolve}),third=new Promise(resolve=>{releaseThird=resolve}),weak,weakPrototype,result;
        function make(){const prototype={b:1},object=Object.create(prototype);object.a=1;weak=new WeakRef(object);weakPrototype=new WeakRef(prototype);return object}
        async function f(){for(const key in make()){if(key==='a')await gate;else await second}await third;return 7}
        f().then(value=>{result=value});
    "#).unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "weak.deref()!==undefined&&weakPrototype.deref()!==undefined&&result===undefined",
    );
    realm.eval("release(0)").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "weak.deref()===undefined&&weakPrototype.deref()!==undefined&&result===undefined",
    );
    realm.eval("releaseSecond(0)").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "weakPrototype.deref()===undefined&&result===undefined",
    );
    realm.eval("releaseThird(0)").unwrap();
    drain(&mut realm);
    check(&mut realm, "result===7");
}

#[test]
fn for_in_preserves_private_eval_this_and_arguments_through_body_await() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result;class C{#x=10;async read(){let total=0;for(const key in await {a:1,b:2}){await 0;total+=eval('this.#x')+arguments.length}return total}}
        new C().read('arg').then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result===22");
}

#[test]
fn for_in_host_abort_skips_handlers_and_releases_resource_captures_at_budget_zero() {
    for limited in [false, true] {
        let mut realm = Realm::new(Limits {
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
            "new SharedArrayBuffer(0)"
        };
        let code = "async function f(){using root=resource;try{for(const key in {a:1}){read=()=>key;await 0;TAIL}}finally{finalized++}}".replace("TAIL",tail);
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
            "disposed===0&&finalized===0&&!caught&&read()==='a'&&state.weak.deref()===undefined",
        );
    }
}
