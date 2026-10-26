//! PropertyDefinitionEvaluation with suspended keys, values and spread (13.2.5.5).

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
fn object_literals_evaluate_and_convert_awaited_keys_before_values_and_later_elements() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],result,keyObject={[Symbol.toPrimitive](hint){events.push('coerce:'+hint);return 'x'}};
        function key(){events.push('key');return keyObject}function rhs(){events.push('rhs');return 7}function spread(){events.push('spread');return{get y(){events.push('get');return 8}}}
        async function f(){return{first:(events.push('first'),1),[await key()]:await rhs(),...await spread(),last:(events.push('last'),9)}}
        f().then(value=>{result=value});
    "#).unwrap();
    check(
        &mut realm,
        "events.join(',')==='first,key'&&result===undefined",
    );
    drain(&mut realm);
    check(
        &mut realm,
        "result.first===1&&result.x===7&&result.y===8&&result.last===9&&events.join(',')==='first,key,coerce:string,rhs,spread,get,last'",
    );
}

#[test]
fn computed_awaited_method_keys_preserve_names_class_static_timing_accessors_and_home_objects() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let names=[],result,extra,symbol=Symbol('s');
        async function f(){return{[await 'fn']:function(){},[await symbol]:()=>7,[await 'K']:class{static{names.push(this.name)}},[await 'm'](){return super.x},get [await 'g'](){return this.fn.name},set [await 'g'](value){this.saved=value},[await 'async']:async function(value){return await value},__proto__:await {x:10},later:await 0}}
        f().then(value=>{result=value;value.async(8).then(v=>{extra=v})});
    "#).unwrap();
    drain(&mut realm);
    realm.eval("result.g=9").unwrap();
    check(
        &mut realm,
        "result.fn.name==='fn'&&result[symbol].name==='[s]'&&result.K.name==='K'&&names.join(',')==='K'&&result.m()===10&&result.g==='fn'&&result.saved===9&&result.async.name==='async'&&extra===8",
    );
}

#[test]
fn awaited_prototype_definitions_computed_proto_keys_shorthand_and_duplicates_keep_distinct_rules()
{
    let mut realm = Realm::default();
    realm.eval(r#"
        let result,ignored,shorthand=3;
        async function f(){return{__proto__:await null,['__proto__']:await 7,...await {x:1},x:await 2,shorthand}}
        async function g(){return{__proto__:await 17,x:1}}
        f().then(value=>{result=value});g().then(value=>{ignored=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "Object.getPrototypeOf(result)===null&&Object.hasOwn(result,'__proto__')&&result.__proto__===7&&result.x===2&&result.shorthand===3&&Object.keys(result).join(',')==='__proto__,x,shorthand'&&Object.getPrototypeOf(ignored)===Object.prototype&&ignored.x===1",
    );
}

#[test]
fn awaited_spread_snapshots_keys_reads_live_descriptors_and_keeps_values_unawaited() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],result,promise=Promise.resolve(7),symbol=Symbol('s'),source={a:1,get b(){events.push('b');delete source.c;source.late=4;return promise},c:3,[symbol]:5};
        Object.defineProperty(source,'a',{writable:false});Object.defineProperty(source,'hidden',{value:6,enumerable:false});
        async function f(){return{...await source,after:await 0}}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='b'&&result.a===1&&result.b===promise&&result[symbol]===5&&!Object.hasOwn(result,'c')&&!Object.hasOwn(result,'late')&&!Object.hasOwn(result,'hidden')&&Object.getOwnPropertyDescriptor(result,'a').writable===true",
    );
}

#[test]
fn key_value_and_spread_failures_skip_later_work_and_reject_with_the_original_reason() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let results=[],rhs=0,later=0,gets=0,reason={},badKey={[Symbol.toPrimitive](){throw reason}},badSpread={get x(){gets++;throw reason}};
        async function f(){try{return{[await badKey]:await(rhs++),after:later++}}catch(error){return error===reason}}
        async function g(){try{return{a:await Promise.reject(reason),...(later++,{})}}catch(error){return error===reason}}
        async function h(){try{return{...await badSpread,after:later++}}catch(error){return error===reason}}
        f().then(value=>{results.push(value)});g().then(value=>{results.push(value)});h().then(value=>{results.push(value)});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results.length===3&&results.every(value=>value)&&rhs===0&&later===0&&gets===1",
    );
}

#[test]
fn awaited_objects_work_in_return_using_dynamic_constructors_and_private_eval_contexts() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let flag=0,first,second,third,fourth,fifth,disposed=0;
        async function f(){return{x:await(flag=1)}}
        async function usingValue(){using value={x:await 7,[Symbol.dispose](){disposed++}};return value.x}
        class C{#x=10;async read(p){return{value:await p,private:eval('this.#x'),args:arguments.length}}}
        const Constructor=(async function(){}).constructor;
        f().then(value=>{first=value});Constructor('p','return {x:await p}')(7).then(value=>{second=value});(new Constructor('p','return {x:await p}'))(8).then(value=>{third=value});new C().read(9).then(value=>{fourth=value});usingValue().then(value=>{fifth=value});
    "#).unwrap();
    check(&mut realm, "flag===1&&first===undefined&&disposed===0");
    drain(&mut realm);
    check(
        &mut realm,
        "first.x===1&&second.x===7&&third.x===8&&fourth.value===9&&fourth.private===10&&fourth.args===1&&fifth===7&&disposed===1",
    );
}

#[test]
fn partial_objects_retain_earlier_values_but_converted_keys_release_their_input_objects() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,releaseSecond,gate=new Promise(resolve=>{release=resolve}),second=new Promise(resolve=>{releaseSecond=resolve}),childWeak,keyWeak,observed,result;
        function make(){const value={x:7};childWeak=new WeakRef(value);return value}
        function key(){const value={[Symbol.toPrimitive](){return 'saved'}};keyWeak=new WeakRef(value);return value}
        async function f(){let object={early:make(),[await key()]:await gate};observed=object.saved;object=null;0;await second;return 7}
        f().then(value=>{result=value});
    "#).unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "childWeak.deref()!==undefined&&keyWeak.deref()!==undefined&&result===undefined",
    );
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "childWeak.deref()!==undefined&&keyWeak.deref()===undefined&&result===undefined",
    );
    realm.eval("release(7)").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "childWeak.deref()===undefined&&observed===7&&result===undefined",
    );
    realm.eval("releaseSecond(0)").unwrap();
    drain(&mut realm);
    check(&mut realm, "result===7");
}

#[test]
fn object_literal_host_abort_releases_partial_state_and_resource_captures_without_cleanup() {
    for limited in [false, true] {
        let mut realm = Realm::new(Limits {
            max_steps: limited.then_some(10_000),
            ..Default::default()
        });
        realm
            .eval("let state={},disposed=0,finalized=0,caught=false;const dispose=Symbol.dispose;")
            .unwrap();
        realm
            .eval("let resource={[dispose](){disposed++}};")
            .unwrap();
        realm
            .eval("state.weak=new WeakRef(resource[dispose]);")
            .unwrap();
        realm.eval("function spin(){while(true){}}").unwrap();
        let tail = if limited {
            "spin()"
        } else {
            "new SharedArrayBuffer(0)"
        };
        let code="async function f(){using root=resource;try{return{a:await 0,b:TAIL}}finally{finalized++}}".replace("TAIL",tail);
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
            "disposed===0&&finalized===0&&!caught&&state.weak.deref()===undefined",
        );
    }
}
