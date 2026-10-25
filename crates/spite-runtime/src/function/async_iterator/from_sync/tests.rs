//! Async-from-Sync forwarding, awaited values, and abrupt cleanup.

use crate::{Error, PromiseJobOutcome, Realm, Value};
use spite_core::Span;

fn new_realm(source: &str) -> Realm {
    let mut realm = Realm::default();
    let Value::Object(source) = realm.eval(source).unwrap() else {
        panic!("iterator");
    };
    let span = Span::new(0, 0);
    let record = realm.get_iterator_direct(source, span).unwrap();
    let wrapper = realm
        .create_async_from_sync_iterator(record, span)
        .unwrap()
        .into_wrapper()
        .iterator;
    let global = realm.global_object();
    realm
        .define_builtin_property(&global, "ai", Value::Object(wrapper), true, span)
        .unwrap();
    realm
}

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
fn metadata_and_forwarding_cache_next_but_lookup_return_and_throw_live() {
    let mut realm = new_realm(
        r#"
        let gets=0,log=[],source={get next(){gets++;return function(v){if(this!==source)throw 1;log.push('n'+arguments.length);return{value:v,done:false}}}};source
    "#,
    );
    realm.eval(r#"
        let p=Object.getPrototypeOf(ai),parent=Object.getPrototypeOf(p),seen=[];
        if(Object.getOwnPropertyNames(p).join(',')!=='next,return,throw'||Object.getPrototypeOf(parent)!==Object.prototype||ai[Symbol.asyncIterator]()!==ai)throw 2;
        for(const name of['next','return','throw']){let d=Object.getOwnPropertyDescriptor(p,name),f=d.value;if(!d.writable||d.enumerable||!d.configurable||f.name!==name||f.length!==1||Object.hasOwn(f,'prototype'))throw 3}
        Object.defineProperty(source,'next',{get(){throw 4}});
        ai.next().then(r=>{seen.push(r.value===undefined&&!r.done)});ai.next(undefined,2).then(r=>{seen.push(r.value===undefined&&!r.done)});
        source.return=function(v){if(this!==source)throw 5;log.push('r'+arguments.length);return{value:v,done:true}};
        ai.return(7,8).then(r=>{seen.push(r.value===7&&r.done)});
        source.throw=function(){if(this!==source)throw 6;log.push('t'+arguments.length);return{value:9,done:false}};
        ai.throw().then(r=>{seen.push(r.value===9&&!r.done)});
    "#).unwrap();
    check(
        &mut realm,
        "gets===1&&log.join(',')==='n0,n1,r1,t0'&&seen.length===0",
    );
    drain(&mut realm);
    check(&mut realm, "seen.length===4&&seen.every(v=>v)");
}

#[test]
fn continuation_reads_done_then_value_even_when_done_and_awaits_value() {
    let mut realm = new_realm(
        r#"
        let log='',release,gate=new Promise(r=>{release=r}),source={next(){return{get done(){log+='d';return true},get value(){log+='v';return gate}}}};source
    "#,
    );
    realm
        .eval("let result;ai.next().then(r=>{result=r})")
        .unwrap();
    check(&mut realm, "log==='dv'&&result===undefined");
    drain(&mut realm);
    check(&mut realm, "result===undefined");
    realm.eval("release(42)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result.value===42&&result.done&&Object.getPrototypeOf(result)===Object.prototype&&Object.keys(result).join(',')==='value,done'&&['value','done'].every(k=>{let d=Object.getOwnPropertyDescriptor(result,k);return d.writable&&d.enumerable&&d.configurable})",
    );
}

#[test]
fn rejected_values_close_only_next_or_throw_before_done_and_preserve_reason() {
    for member in ["next", "return", "throw"] {
        for done in [false, true] {
            let mut realm = new_realm(&r#"
                let marker={},closed=0,source={next(){return{value:Promise.reject(marker),done:DONE}},throw(){return this.next()},return(){closed++;return{value:Promise.reject(marker),done:DONE}}};source
            "#.replace("DONE", &done.to_string()));
            realm
                .eval(&format!(
                    "let rejected=false;ai.{member}().catch(e=>{{rejected=e===marker}})"
                ))
                .unwrap();
            // For return(), count the forwarding call separately from closing.
            drain(&mut realm);
            let expected = if member == "return" || !done { 1 } else { 0 };
            check(&mut realm, &format!("rejected&&closed==={expected}"));
        }
    }
    let mut realm = new_realm(
        "let marker={},source={next(){return{value:Promise.reject(marker),done:false}},get return(){throw 9}};source",
    );
    realm
        .eval("let ok=false;ai.next().catch(e=>{ok=e===marker})")
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "ok");
}

#[test]
fn getter_failures_do_not_close_but_promise_resolve_failure_does() {
    for field in ["done", "value"] {
        let mut realm = new_realm(&format!(
            "let marker={{}},closed=0,source={{next(){{return{{get {field}(){{throw marker}}}}}},return(){{closed++;return{{}}}}}};source"
        ));
        realm
            .eval("let ok=false;ai.next().catch(e=>{ok=e===marker})")
            .unwrap();
        drain(&mut realm);
        check(&mut realm, "ok&&closed===0");
    }
    let mut realm = new_realm(
        "let marker={},closed=0,p=Promise.resolve(1);Object.defineProperty(p,'constructor',{get(){throw marker}});let source={next(){return{value:p,done:false}},return(){closed++;return{}}};source",
    );
    realm
        .eval("let ok=false;ai.next().catch(e=>{ok=e===marker})")
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "ok&&closed===1");
}

#[test]
fn missing_return_embeds_argument_and_missing_throw_performs_normal_close() {
    let mut realm = new_realm("let source={next(){return{done:true}}};source");
    realm.eval("let argument=Promise.resolve(1),ok=false;ai.return(argument).then(r=>{ok=r.done&&r.value===argument})").unwrap();
    drain(&mut realm);
    check(&mut realm, "ok");
    for throws in [false, true] {
        let mut realm = new_realm(&format!(
            "let marker={{}},closed=0,source={{next(){{return{{}}}},return(){{closed++;{} }}}};source",
            if throws { "throw marker" } else { "return{}" }
        ));
        realm
            .eval("let reason;ai.throw(7).catch(e=>{reason=e})")
            .unwrap();
        drain(&mut realm);
        check(
            &mut realm,
            if throws {
                "closed===1&&reason===marker"
            } else {
                "closed===1&&reason instanceof TypeError"
            },
        );
    }
}

#[test]
fn invalid_results_and_receivers_reject_while_host_failures_bypass_handlers() {
    let mut realm = new_realm("let source={next(){return 7}};source");
    realm.eval("let errors=0;ai.next().catch(e=>{if(e instanceof TypeError)errors++});ai.next.call({}).catch(e=>{if(e instanceof TypeError)errors++})").unwrap();
    drain(&mut realm);
    check(&mut realm, "errors===2");
    let mut realm = new_realm("let source={next(){return new SharedArrayBuffer(0)}};source");
    realm.eval("let caught=false").unwrap();
    assert!(matches!(
        realm.eval("try{ai.next()}catch(e){caught=true}"),
        Err(Error::Unsupported { .. })
    ));
    check(&mut realm, "!caught");
}
