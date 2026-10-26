//! Untagged template substitutions with Await and immediate ToString (13.2.8.6).

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
fn template_converts_each_substitution_before_evaluating_the_next_one() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],result,one={[Symbol.toPrimitive](hint){events.push('one:'+hint);return 'one'}},two={[Symbol.toPrimitive](hint){events.push('two:'+hint);return 'two'}};
        function first(){events.push('first');return one}function second(){events.push('second');return two}function third(){events.push('third');return 3}
        async function f(){return`A${first()}B${await second()}C${third()}D`}
        f().then(value=>{result=value});
    "#).unwrap();
    check(
        &mut realm,
        "events.join(',')==='first,one:string,second'&&result===undefined",
    );
    drain(&mut realm);
    check(
        &mut realm,
        "result==='AoneBtwoC3D'&&events.join(',')==='first,one:string,second,two:string,third'",
    );
}

#[test]
fn templates_support_nested_and_repeated_awaits_with_bigint_nullish_and_lossless_utf16() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let first,second;
        async function f(){return`x${await `inner${await 1}`}y${await null}z${await undefined}|${await 1n}`}
        async function g(){return`\uD800${await 'x'}\uDFFF${await 1n}`}
        f().then(value=>{first=value});g().then(value=>{second=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "first==='xinner1ynullzundefined|1'&&second.length===4&&second.charCodeAt(0)===55296&&second.charCodeAt(1)===120&&second.charCodeAt(2)===57343&&second.charCodeAt(3)===49",
    );
}

#[test]
fn rejected_or_throwing_substitutions_skip_later_evaluation_and_symbol_conversion_throws() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let results=[],later=0,reason={},bad={[Symbol.toPrimitive](){throw reason}};
        async function f(){try{return`${await Symbol('x')}${later++}`}catch(error){return error instanceof TypeError}}
        async function g(){try{return`${await bad}${later++}`}catch(error){return error===reason}}
        async function h(){try{return`${await Promise.reject(reason)}${later++}`}catch(error){return error===reason}}
        f().then(value=>{results.push(value)});g().then(value=>{results.push(value)});h().then(value=>{results.push(value)});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results.length===3&&results.every(value=>value)&&later===0",
    );
}

#[test]
fn templates_preserve_private_eval_this_arguments_and_dynamic_async_function_contexts() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let first,second;class C{#x=10;async f(a){return`${eval('this.#x')}:${await a}:${arguments.length}`}}
        let Constructor=(async()=>{}).constructor,dynamic=Constructor('a','return `${await a}:${arguments.length}`');
        new C().f(7).then(value=>{first=value});dynamic(8).then(value=>{second=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "first==='10:7:1'&&second==='8:1'");
}

#[test]
fn converted_template_values_release_objects_while_later_substitutions_are_pending() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,gate=new Promise(resolve=>{release=resolve}),weak,result;
        function make(){const value={[Symbol.toPrimitive](){return 'left'}};weak=new WeakRef(value);return value}
        async function f(){return`${await make()}${await gate}`}
        f().then(value=>{result=value});
    "#).unwrap();
    collect(&mut realm);
    check(&mut realm, "weak.deref()!==undefined&&result===undefined");
    drain(&mut realm);
    collect(&mut realm);
    check(&mut realm, "weak.deref()===undefined&&result===undefined");
    realm.eval("release('right')").unwrap();
    drain(&mut realm);
    check(&mut realm, "result==='leftright'");
}

#[test]
fn string_limits_abort_before_an_await_operand_and_after_resumed_concatenation() {
    let mut realm = Realm::new(Limits {
        max_string_units: Some(64),
        ..Default::default()
    });
    realm
        .eval("let operand=0,finalized=0,caught=false")
        .unwrap();
    let source="async function f(){try{return`PREFIX${await(operand++,0)}`}finally{finalized++}}f().catch(()=>{caught=true});".replace("PREFIX",&"a".repeat(65));
    let script = spite_parser::parse_script(&source).unwrap();
    assert!(matches!(realm.evaluate(&script), Err(Error::Limit { .. })));
    check(&mut realm, "operand===0&&finalized===0&&!caught");

    let mut realm = Realm::new(Limits {
        max_string_units: Some(64),
        ..Default::default()
    });
    realm
        .eval("let later=0,finalized=0,caught=false,head='a'.repeat(60),tail='b'.repeat(5)")
        .unwrap();
    realm.eval("async function f(){try{return`${await head}${await tail}${later++}`}finally{finalized++}}f().catch(()=>{caught=true})").unwrap();
    let mut reported = false;
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => (),
            PromiseJobOutcome::Reported(error) => {
                assert!(matches!(error, Error::Limit { .. }), "{error}");
                reported = true;
            }
        }
    }
    assert!(reported);
    check(&mut realm, "later===0&&finalized===0&&!caught");
}

#[test]
fn template_host_abort_skips_later_substitutions_handlers_and_disposal_at_budget_zero() {
    for limited in [false, true] {
        let mut realm = Realm::new(Limits {
            max_steps: limited.then_some(10_000),
            ..Default::default()
        });
        realm.eval("let state={},later=0,disposed=0,finalized=0,caught=false;const dispose=Symbol.dispose;").unwrap();
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
        let code="async function f(){using root=resource;try{return`${await 0}${TAIL}${later++}`}finally{finalized++}}".replace("TAIL",tail);
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
            "later===0&&disposed===0&&finalized===0&&!caught&&state.weak.deref()===undefined",
        );
    }
}
