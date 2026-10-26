//! DestructuringAssignmentEvaluation after an awaited RHS (13.15.5).

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
fn awaited_array_object_nested_and_rest_assignments_return_the_original_rhs() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result,a,b,rest,x,y,other,first=[1,{x:2},3,4],second={x:3,y:{z:4},tail:5};
        async function f(){let one=([a,{x:b},...rest]=await first)===first,two=({x,y:{z:y},...other}=await second)===second;return one&&two}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===true&&a===1&&b===2&&rest.join(',')==='3,4'&&x===3&&y===4&&other.tail===5",
    );
}

#[test]
fn destructuring_resolves_targets_and_computed_names_only_after_rhs_completion() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,releaseSecond,gate=new Promise(resolve=>{release=resolve}),second=new Promise(resolve=>{releaseSecond=resolve}),events=[],old={},current=old,newObject={},firstResult,secondResult;
        function target(){events.push('target');return current}function key(){events.push('key');return 'z'}
        async function f(){[target().x]=await gate;return 7}
        async function g(){({[key()]:target().y}=await second);return 8}
        f().then(value=>{firstResult=value});g().then(value=>{secondResult=value});
    "#).unwrap();
    check(
        &mut realm,
        "events.length===0&&firstResult===undefined&&secondResult===undefined",
    );
    realm
        .eval(
            "current=newObject;release([7]);releaseSecond({get z(){events.push('get');return 9}})",
        )
        .unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "firstResult===7&&secondResult===8&&newObject.x===7&&newObject.y===9&&old.x===undefined&&old.y===undefined&&events.join(',')==='target,key,target,get'",
    );
}

#[test]
fn awaited_rhs_patterns_close_synchronous_iterators_and_keep_yielded_promises_raw() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],promise=Promise.resolve(7),value,result,failed,reason={},closeReason={},target={set x(value){throw reason}};
        function source(name,bad){return{[Symbol.iterator](){return{next(){events.push(name+'-next');return{value:promise,done:false}},return(){events.push(name+'-return');if(bad)throw closeReason;return{}}}}}}
        async function f(){[value]=await source('first',false);return 7}
        async function g(){try{[target.x]=await source('second',true)}catch(error){return error===reason}}
        f().then(value=>{result=value});g().then(value=>{failed=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===7&&failed===true&&value===promise&&events.join(',')==='first-next,first-return,second-next,second-return'",
    );
}

#[test]
fn awaited_rhs_defaults_keep_name_inference_computed_keys_and_rest_exclusion() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let first,second,x,rest,result,keys=0,input={a:undefined,b:2};
        function key(){keys++;return 'a'}
        async function f(){[first=()=>7,(second)=()=>8]=await [undefined,undefined];({[key()]:x=9,...rest}=await input);return first.name==='first'&&second.name===''&&first()===7&&second()===8}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===true&&keys===1&&x===9&&rest.b===2&&!Object.hasOwn(rest,'a')",
    );
}

#[test]
fn rejected_rhs_does_not_resolve_or_write_pattern_targets() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let resolved=0,written=0,result,reason={},target={set x(value){written++}};
        function base(){resolved++;return target}
        async function f(){try{[base().x]=await Promise.reject(reason)}catch(error){return error===reason}}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result===true&&resolved===0&&written===0");
}

#[test]
fn awaited_patterns_preserve_mapped_arguments_private_context_and_super_receivers() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let mapped,result;async function f(x){[x]=await [7];return arguments[0]}
        class Base{set x(value){this.saved=value}}
        class C extends Base{#x=0;async f(x){[this.#x,super.x,arguments[0]]=await [1,2,3];return this.#x+this.saved+arguments[0]}}
        f(1).then(value=>{mapped=value});new C().f(0).then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "mapped===7&&result===6");
}

#[test]
fn destructuring_rhs_and_its_completion_value_remain_live_until_replaced() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,releaseSecond,gate=new Promise(resolve=>{release=resolve}),second=new Promise(resolve=>{releaseSecond=resolve}),weak,x,result;
        function make(){const value=[7];weak=new WeakRef(value);return value}
        async function f(){[x]=await make();const first=await gate;0;const next=await second;return 7}
        f().then(value=>{result=value});
    "#).unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "weak.deref()!==undefined&&x===undefined&&result===undefined",
    );
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "weak.deref()!==undefined&&x===7&&result===undefined",
    );
    realm.eval("release(0)").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(&mut realm, "weak.deref()===undefined&&result===undefined");
    realm.eval("releaseSecond(0)").unwrap();
    drain(&mut realm);
    check(&mut realm, "result===7");
}

#[test]
fn host_abort_in_a_post_await_default_skips_iterator_close_put_handlers_and_disposal() {
    for limited in [false, true] {
        let mut realm = Realm::new(Limits {
            max_steps: limited.then_some(10_000),
            ..Default::default()
        });
        realm.eval("let state={},writes=0,closed=0,disposed=0,finalized=0,caught=false;const dispose=Symbol.dispose;").unwrap();
        realm
            .eval("let object={set x(value){writes++}},resource={[dispose](){disposed++}};")
            .unwrap();
        realm
            .eval("state.weak=new WeakRef(resource[dispose]);")
            .unwrap();
        realm.eval("function spin(){while(true){}}").unwrap();
        realm.eval("let source={[Symbol.iterator](){return{next(){return{done:false,value:undefined}},return(){closed++;return{}}}}}").unwrap();
        let tail = if limited {
            "spin()"
        } else {
            "new SharedArrayBuffer(0)"
        };
        let code="async function f(){using root=resource;try{[object.x=TAIL]=await source}finally{finalized++}}".replace("TAIL",tail);
        realm.eval(&code).unwrap();
        realm.eval("f().catch(()=>{caught=true});").unwrap();
        realm.eval("delete resource[dispose]").unwrap();
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
            "writes===0&&closed===0&&disposed===0&&finalized===0&&!caught&&state.weak.deref()===undefined",
        );
    }
}
