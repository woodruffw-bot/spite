//! EvaluateSuperCall with suspended arguments in lexical async arrows (13.3.7.2).

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
fn super_constructor_selection_precedes_arguments_and_derived_fields_follow_this_binding() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,gate=new Promise(resolve=>{release=resolve}),oldCalls=0,newCalls=0,result;
        class Base{constructor(value){oldCalls++;this.value=value}}class Later{constructor(value){newCalls++;this.value=99}}
        class Derived extends Base{field=11;constructor(){return(async()=>super(await gate))()}}
        new Derived().then(value=>{result=value});Object.setPrototypeOf(Derived,Later);
    "#).unwrap();
    check(&mut realm, "oldCalls===0&&newCalls===0&&result===undefined");
    realm.eval("release(7)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "oldCalls===1&&newCalls===0&&result.value===7&&result.field===11&&result instanceof Derived&&result instanceof Base",
    );
}

#[test]
fn super_spread_keeps_raw_values_and_nonconstructors_fail_after_argument_evaluation() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result,failed,effects=0,calls=0,symbol=Symbol('raw'),promise=Promise.resolve(7);
        class Base{constructor(...values){calls++;this.values=values}}
        class Good extends Base{constructor(){return(async()=>super(symbol,...await [promise,8]))()}}
        class Bad extends Base{constructor(){return(async()=>super(await(effects++,1),(effects++,2)))()}}
        Object.setPrototypeOf(Bad,null);new Good().then(value=>{result=value});new Bad().then(undefined,error=>{failed=error instanceof TypeError});
    "#).unwrap();
    check(&mut realm, "effects===1&&calls===0");
    drain(&mut realm);
    check(
        &mut realm,
        "result.values.length===3&&result.values[0]===symbol&&result.values[1]===promise&&result.values[2]===8&&failed===true&&effects===2&&calls===1",
    );
}

#[test]
fn super_new_target_prototype_is_read_after_arguments_and_fields_use_the_active_constructor() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,gate=new Promise(resolve=>{release=resolve}),events=[],proto={marker:1},result;
        class Base{constructor(value){this.value=value}}
        class Derived extends Base{#x=9;constructor(){return(async()=>super(await gate))()}read(){return this.#x}}
        function Target(){}let target=new Proxy(Target,{get(object,key,receiver){if(key==='prototype'){events.push('prototype');return proto}return Reflect.get(object,key,receiver)}});
        Reflect.construct(Derived,[],target).then(value=>{result=value});
    "#).unwrap();
    check(&mut realm, "events.length===0&&result===undefined");
    realm.eval("proto={marker:2};release(7)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='prototype'&&Object.getPrototypeOf(result)===proto&&result.value===7&&Derived.prototype.read.call(result)===9",
    );
}

#[test]
fn a_repeated_super_call_constructs_before_bind_this_rejects_without_repeating_fields() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,gate=new Promise(resolve=>{release=resolve}),calls=0,fields=0,failed,observed;
        class Base{constructor(value){calls++;this.value=value}}
        class Derived extends Base{field=(fields++,1);constructor(){const pending=(async()=>super(await gate))();super(1);observed=this;return pending}}
        new Derived().then(undefined,error=>{failed=error instanceof ReferenceError});
    "#).unwrap();
    check(
        &mut realm,
        "calls===1&&fields===1&&observed.value===1&&observed.field===1&&failed===undefined",
    );
    realm.eval("release(2)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "calls===2&&fields===1&&failed===true&&observed.value===1&&observed.field===1",
    );
}

#[test]
fn a_throwing_derived_initializer_leaves_this_bound_for_the_resumed_catch() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let reason={},result,observed;function fail(){throw reason}class Base{}
        class Derived extends Base{field=fail();constructor(){return(async()=>{try{return super(await 0)}catch(error){observed=this;return error===reason}})()}}
        new Derived().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===true&&observed instanceof Derived&&!Object.hasOwn(observed,'field')",
    );
}

#[test]
fn pending_super_arguments_remain_live_and_rejection_releases_them_without_construction() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let reject,gate=new Promise((resolve,fail)=>{reject=fail}),weak,reason={},result,calls=0;
        function make(){const value={x:7};weak=new WeakRef(value);return value}
        class Base{constructor(value){calls++}}
        class Derived extends Base{constructor(){return(async()=>{try{return super(make(),await gate)}catch(error){return error===reason}})()}}
        new Derived().then(value=>{result=value});
    "#).unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "weak.deref()!==undefined&&calls===0&&result===undefined",
    );
    realm.eval("reject(reason)").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "weak.deref()===undefined&&calls===0&&result===true",
    );
}

#[test]
fn super_argument_limits_and_host_abort_skip_construction_fields_handlers_and_disposal() {
    let mut realm = Realm::with_unavailable_host_limits(Limits {
        max_arguments: Some(2),
        ..Default::default()
    });
    realm.eval("let effects=0,calls=0,fields=0,disposed=0,finalized=0,caught=false;const dispose=Symbol.dispose").unwrap();
    realm
        .eval("let resource={[dispose](){disposed++}};class Base{constructor(a,b,c){calls++}}")
        .unwrap();
    realm.eval("class Derived extends Base{field=(fields++,1);constructor(){return(async()=>{using root=resource;try{return super(await(effects++,1),(effects++,2),(effects++,3))}catch{caught=true}finally{finalized++}})()}}new Derived()").unwrap();
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
    check(
        &mut realm,
        "effects===3&&calls===0&&fields===0&&disposed===0&&finalized===0&&!caught",
    );
    for limited in [false, true] {
        let mut realm = Realm::with_unavailable_host_limits(Limits {
            max_steps: limited.then_some(10_000),
            ..Default::default()
        });
        realm.eval("let state={},calls=0,fields=0,disposed=0,finalized=0,caught=false;const dispose=Symbol.dispose;").unwrap();
        realm
            .eval("let resource={[dispose](){disposed++}};")
            .unwrap();
        realm
            .eval("state.weak=new WeakRef(resource[dispose]);")
            .unwrap();
        realm
            .eval("function spin(){while(true){}}class Base{constructor(){calls++}}")
            .unwrap();
        let tail = if limited {
            "spin()"
        } else {
            "spiteUnavailableHost()"
        };
        let code="class Derived extends Base{field=(fields++,1);constructor(){return(async()=>{using root=resource;try{return super(await 0,TAIL)}finally{finalized++}})()}}new Derived().catch(()=>{caught=true})".replace("TAIL",tail);
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
        collect(&mut realm);
        check(
            &mut realm,
            "calls===0&&fields===0&&disposed===0&&finalized===0&&!caught&&state.weak.deref()===undefined",
        );
    }
}
