//! EvaluateImportCall validation, Promise rejection and operand suspension (13.3.10).

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
fn invalid_specifiers_reject_fresh_intrinsic_promises_without_public_constructor_hooks() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let NativePromise=Promise,errors=0,p=import(Symbol()),q=import(Symbol());
        globalThis.Promise=function(){throw 'public constructor'};NativePromise.resolve=()=>{throw 'public resolve'};
        let r=import(Symbol()),fresh=p!==q&&q!==r&&Object.getPrototypeOf(p)===NativePromise.prototype&&Object.getPrototypeOf(r)===NativePromise.prototype;
        for(const promise of [p,q,r])promise.catch(error=>{if(error instanceof TypeError)errors++});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "fresh&&errors===3");
}

#[test]
fn operand_abrupts_throw_synchronously_before_specifier_conversion() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let reason={},events=[],first=false,second=false;
        let specifier={toString(){events.push('coerce');return 'source'}};
        try{import((events.push('specifier'),specifier),(events.push('options'),(()=>{throw reason})()))}catch(error){first=error===reason}
        try{import((()=>{throw reason})(),(events.push('unreached'),{}))}catch(error){second=error===reason}
        first&&second&&events.join(',')==='specifier,options'
    "#,
    );
    assert!(matches!(
        realm.run_next_promise_job().unwrap(),
        PromiseJobOutcome::NoJob
    ));
}

#[test]
fn specifier_coercion_follows_both_operands_and_rejects_before_reading_options() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let reason={},events=[],same=false;
        let promise=import((events.push('specifier'),{[Symbol.toPrimitive](hint){events.push(hint);throw reason}}),(events.push('options'),{get with(){events.push('with');return{}}}));
        promise.catch(error=>{same=error===reason});
    "#).unwrap();
    check(
        &mut realm,
        "events.join(',')==='specifier,options,string'&&!same",
    );
    drain(&mut realm);
    check(
        &mut realm,
        "same&&events.join(',')==='specifier,options,string'",
    );
}

#[test]
fn primitive_options_reject_after_string_conversion() {
    for value in ["null", "false", "1", "1n", "Symbol()", "'options'"] {
        let mut realm = Realm::default();
        realm.eval(&format!(r#"
            let events=[],rejected=false;import({{toString(){{events.push('string');return'source'}}}},{value}).catch(error=>{{rejected=error instanceof TypeError}})
        "#)).unwrap();
        drain(&mut realm);
        check(&mut realm, "rejected&&events.join(',')==='string'");
    }
}

#[test]
fn with_getter_and_attribute_types_reject_without_synchronous_throws() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let reason={},same=false,errors=0;import('source',{get with(){throw reason}}).catch(error=>{same=error===reason});
        for(const value of [null,false,1,1n,Symbol(),'attributes'])import('source',{with:value}).catch(error=>{if(error instanceof TypeError)errors++});
        for(const value of [null,false,1,1n,Symbol(),{}])import('source',{with:{type:value}}).catch(error=>{if(error instanceof TypeError)errors++});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "same&&errors===12");
}

#[test]
fn attribute_getters_finish_before_type_validation_and_symbol_keys_are_ignored() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let reason={},events=[],same=false,typed=false;
        import('source',{with:{get first(){events.push('first');return 0},get second(){events.push('second');throw reason}}}).catch(error=>{same=error===reason});
        let key=Symbol(),attrs=new Proxy({a:0,[key]:1},{ownKeys(target){events.push('keys');return Reflect.ownKeys(target)},getOwnPropertyDescriptor(target,key){events.push(typeof key==='symbol'?'symbol-descriptor':'descriptor');return Reflect.getOwnPropertyDescriptor(target,key)},get(target,key,receiver){events.push(typeof key==='symbol'?'symbol-get':'get');return Reflect.get(target,key,receiver)}});
        import('source',{with:attrs}).catch(error=>{typed=error instanceof TypeError});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "same&&typed&&events.join(',')==='first,second,keys,descriptor,get'",
    );
}

#[test]
fn await_and_yield_operands_preserve_order_and_the_original_specifier() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],asyncRejected=false,generatorRejected=false;
        async function f(){return import((events.push('specifier'),await Promise.resolve(Symbol())),(events.push('options'),await Promise.resolve({})))}f().catch(error=>{asyncRejected=error instanceof TypeError});
        function*g(){return import(yield'specifier',yield'options')}let iterator=g(),a=iterator.next(),b=iterator.next(Symbol()),c=iterator.next({});
        let valid=a.value==='specifier'&&b.value==='options'&&c.done;c.value.catch(error=>{generatorRejected=error instanceof TypeError});
    "#).unwrap();
    check(&mut realm, "events.join(',')==='specifier'&&valid");
    drain(&mut realm);
    check(
        &mut realm,
        "asyncRejected&&generatorRejected&&events.join(',')==='specifier,options'",
    );
}

#[test]
fn host_failures_during_conversion_bypass_rejection_handlers_and_disposal() {
    for limited in [false, true] {
        let mut realm = Realm::new(Limits {
            max_steps: limited.then_some(10_000),
            ..Default::default()
        });
        realm.reserve_unsupported_global("unsupportedHost");
        let tail = if limited {
            "while(true){}"
        } else {
            "Function('unsupportedHost')()"
        };
        realm.eval(&format!("let disposed=0,caught=0,finalized=0;let resource={{[Symbol.dispose](){{disposed++}}}},weak=new WeakRef(resource[Symbol.dispose]);let source={{toString(){{{tail}}}}}")).unwrap();
        let error = realm.eval("function f(){using r=resource;delete resource[Symbol.dispose];try{import(source,{})}catch{caught++}finally{finalized++}}f()").unwrap_err();
        assert!(
            if limited {
                matches!(error, Error::Limit { .. })
            } else {
                matches!(error, Error::Unsupported { .. })
            },
            "{error}"
        );
        collect(&mut realm);
        check(
            &mut realm,
            "disposed===0&&caught===0&&finalized===0&&weak.deref()===undefined",
        );
    }
}

#[test]
fn valid_requests_run_validation_before_the_module_loading_host_boundary() {
    let mut realm = Realm::default();
    realm.eval("let events=[],caught=0,finalized=0").unwrap();
    let error=realm.eval(r#"
        try{import((events.push('specifier'),{toString(){events.push('string');return'source'}}),(events.push('options'),{get with(){events.push('with');return{get type(){events.push('value');return'json'}}}}))}catch{caught++}finally{finalized++}
    "#).unwrap_err();
    assert!(matches!(error, Error::Unsupported { .. }), "{error}");
    check(
        &mut realm,
        "caught===0&&finalized===0&&events.join(',')==='specifier,options,string,with,value'",
    );
    assert!(matches!(
        realm.run_next_promise_job().unwrap(),
        PromiseJobOutcome::NoJob
    ));
}

#[test]
fn specifiers_remain_alive_across_await_or_yield_then_release_after_completion() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let weak,release,reason={},rejected=false,gate=new Promise(resolve=>{release=resolve});
        async function f(){return import((()=>{let value={[Symbol.toPrimitive](){throw reason}};weak=new WeakRef(value);return value})(),await gate)}f().catch(error=>{rejected=error===reason});
    "#).unwrap();
    collect(&mut realm);
    check(&mut realm, "weak.deref()!==undefined");
    realm.eval("release({})").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(&mut realm, "rejected&&weak.deref()===undefined");
    realm.eval(r#"
        function*g(){return import((()=>{let value={};weak=new WeakRef(value);return value})(),yield'options')}let iterator=g();iterator.next()
    "#).unwrap();
    collect(&mut realm);
    check(&mut realm, "weak.deref()!==undefined");
    realm.eval("iterator.return(7)").unwrap();
    collect(&mut realm);
    check(&mut realm, "weak.deref()===undefined");
}
