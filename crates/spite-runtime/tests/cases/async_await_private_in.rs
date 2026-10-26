//! PrivateIdentifier in ShiftExpression after Await (13.10.2).

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

#[test]
fn awaited_private_in_checks_instance_static_method_and_accessor_brands_without_coercion() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let results=[],getters=0,coerced=0,hasTraps=0;
        class C{#x=1;#m(){}get #a(){getters++;return 7}static #s=2;[Symbol.toPrimitive](){coerced++;throw 9}async check(value){return(#x in await value)&&(#m in await value)&&(#a in await value)}static async check(value){return #s in await value}}
        class Sub extends C{}let object=new C(),sub=new Sub(),proxy=new Proxy(object,{has(){hasTraps++;return true}});
        object.check(object).then(value=>{results.push(value)});object.check(sub).then(value=>{results.push(value)});object.check({}).then(value=>{results.push(!value)});object.check(proxy).then(value=>{results.push(!value)});C.check(C).then(value=>{results.push(value)});C.check(object).then(value=>{results.push(!value)});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results.length===6&&results.every(value=>value)&&getters===0&&coerced===0&&hasTraps===0",
    );
}

#[test]
fn private_in_rejects_resolved_primitives_and_preserves_await_rejection_reasons() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let results=[],reason={};class C{#x=1;async check(value){try{return #x in await value}catch(error){return error===reason||error instanceof TypeError}}}
        let object=new C();for(const value of [null,undefined,1,'x',Symbol('x'),1n])object.check(value).then(value=>{results.push(value)});object.check(Promise.reject(reason)).then(value=>{results.push(value)});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results.length===7&&results.every(value=>value)",
    );
}

#[test]
fn private_in_after_await_resolves_the_innermost_private_environment_and_survives_collection() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let results=[],weak;class Outer{#x=1;async read(value){return #x in await value}nested(){return class Inner{#x=2;async read(value){await 0;return #x in await value}}}}
        let outer=new Outer(),Inner=outer.nested(),inner=new Inner();outer.read(outer).then(value=>{results.push(value)});outer.read(inner).then(value=>{results.push(!value)});inner.read(inner).then(value=>{results.push(value)});inner.read(outer).then(value=>{results.push(!value)});
        function make(){const value=new Inner();weak=new WeakRef(value);return value}inner.read(make()).then(value=>{results.push(value)});
    "#).unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(&mut realm, "weak.deref()!==undefined&&results.length===0");
    drain(&mut realm);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "results.length===5&&results.every(value=>value)&&weak.deref()===undefined",
    );
}

#[test]
fn private_in_host_failure_after_await_skips_handlers_disposal_and_resource_captures_at_zero() {
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
        let code="class C{#x=0;async f(){using root=resource;try{return #x in(await 0,TAIL)}finally{finalized++}}}new C().f().catch(()=>{caught=true})".replace("TAIL",tail);
        realm.eval(&code).unwrap();
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
        realm.clear_kept_objects().unwrap();
        realm.collect(usize::MAX).unwrap();
        check(
            &mut realm,
            "disposed===0&&finalized===0&&!caught&&state.weak.deref()===undefined",
        );
    }
}
