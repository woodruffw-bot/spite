//! Awaited SuperProperty keys and live MakeSuperPropertyReference (13.3.7.1/3).

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
fn super_base_is_read_after_the_awaited_key_and_before_key_coercion() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,gate=new Promise(resolve=>{release=resolve}),events=[],result,receiver,before={get x(){throw 9}},after={get x(){events.push('after');receiver=this;return this.marker+7}},latest={get x(){throw 9}};
        let home={__proto__:before,marker:10,async read(){return super[await gate]}},key={[Symbol.toPrimitive](){events.push('key');Object.setPrototypeOf(home,latest);return 'x'}};
        home.read().then(value=>{result=value});Object.setPrototypeOf(home,after);
    "#).unwrap();
    check(&mut realm, "result===undefined&&events.length===0");
    realm.eval("release(key)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===17&&receiver===home&&Object.getPrototypeOf(home)===latest&&events.join(',')==='key,after'",
    );
}

#[test]
fn computed_super_calls_optional_calls_tags_and_constructor_values_keep_their_receivers() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let first,second,third,fourth,proto={m(value){return this.marker+value},tag(strings,value){return this.marker+value},Ctor:function(value){this.x=value}};
        let home={__proto__:proto,marker:10,async f(){return super[await 'm'](await 7)},async g(){return super[await 'm']?.(await 8)},async h(){return(super[await 'tag'])`x${await 9}`},async j(){return new(super[await 'Ctor'])(await 11)}};
        home.f().then(value=>{first=value});home.g().then(value=>{second=value});home.h().then(value=>{third=value});home.j().then(value=>{fourth=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "first===17&&second===18&&third===19&&fourth.x===11&&fourth instanceof proto.Ctor",
    );
}

#[test]
fn super_assignments_and_updates_keep_the_selected_base_through_a_later_rhs_await() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,gate=new Promise(resolve=>{release=resolve}),first,second,third,events=[],proto={get x(){events.push('get');return 3},set x(value){events.push('set');this.savedX=value},set y(value){this.savedY=value},get z(){return '4'},set z(value){this.savedZ=value}};
        let home={__proto__:proto,async f(){return super[await 'x']+=await gate},async g(){return super[await 'y']=await 7},async h(){return(super[await 'z'])++}};
        home.f().then(value=>{first=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "events.join(',')==='get'&&first===undefined");
    realm
        .eval("Object.setPrototypeOf(home,{set x(value){throw 9}});release(4)")
        .unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "first===7&&home.savedX===7&&events.join(',')==='get,set'",
    );
    realm.eval("Object.setPrototypeOf(home,proto);home.g().then(value=>{second=value});home.h().then(value=>{third=value})").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "second===7&&home.savedY===7&&third===4&&home.savedZ===5",
    );
}

#[test]
fn deleting_awaited_super_references_throws_without_getting_or_converting_the_key() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let first,second,keys=0,coerced=0,gets=0,key={[Symbol.toPrimitive](){coerced++;return 'x'}},proto={get x(){gets++;throw 9}};
        let home={__proto__:proto,async f(){try{return delete super[await(keys++,key)]}catch(error){return error instanceof ReferenceError}}},empty={__proto__:null,async f(){try{return delete super[await(keys++,key)]}catch(error){return error instanceof ReferenceError}}};
        home.f().then(value=>{first=value});empty.f().then(value=>{second=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "first===true&&second===true&&keys===2&&coerced===0&&gets===0",
    );
}

#[test]
fn super_reads_this_before_the_key_and_rejection_skips_later_super_getters() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let effects=0,failed,rejected,gets=0,reason={};class Base{}class Derived extends Base{constructor(){return(async()=>super[await(++effects,'x')])()}}
        let home={__proto__:{get x(){gets++;return 7}},async f(){try{return super[await Promise.reject(reason)]}catch(error){return error===reason}}};
        new Derived().then(undefined,error=>{failed=error instanceof ReferenceError});home.f().then(value=>{rejected=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "effects===0&&failed===true&&rejected===true&&gets===0",
    );
}

#[test]
fn pending_super_keys_do_not_retain_old_prototypes_and_host_abort_releases_resource_captures() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,gate=new Promise(resolve=>{release=resolve}),proto={x:7},weak=new WeakRef(proto),result,home={__proto__:proto,async f(){return super[await gate]}};
        home.f().then(value=>{result=value});Object.setPrototypeOf(home,{x:8});proto=null;
    "#).unwrap();
    collect(&mut realm);
    check(&mut realm, "weak.deref()===undefined&&result===undefined");
    realm.eval("release('x')").unwrap();
    drain(&mut realm);
    check(&mut realm, "result===8");
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
        let code="let home={async f(){using root=resource;try{return super[(await 0,TAIL)]}finally{finalized++}}};home.f().catch(()=>{caught=true})".replace("TAIL",tail);
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
            "disposed===0&&finalized===0&&!caught&&state.weak.deref()===undefined",
        );
    }
}
