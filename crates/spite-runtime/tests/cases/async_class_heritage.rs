//! ClassDeclaration heritage continuations through Await and Yield (15.7.14).

use spite_runtime::{Error, PromiseJobOutcome, Realm, Value, object::PromiseState};

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
fn heritage_waits_without_replaying_effects_and_preserves_internal_class_tdz() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let release,read,outerRead,events=[],result;
        let gate=new Promise(r=>release=r);
        async function f(){
            outerRead=()=>C;
            class C extends (events.push('heritage'),read=()=>C,await gate){static ready=C;value=7}
            let internal=read();C=99;
            return [internal,new internal().value,internal.ready,outerRead()];
        }
        f().then(v=>result=v);
    "#,
        )
        .unwrap();
    check(
        &mut realm,
        "events.join(',')==='heritage'&&(()=>{try{read()}catch(e){return e instanceof ReferenceError}})()&&(()=>{try{outerRead()}catch(e){return e instanceof ReferenceError}})()",
    );
    realm.collect(usize::MAX).unwrap();
    realm.eval("release(Object)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "events.length===1&&result[1]===7&&result[0]===result[2]&&result[3]===99&&read()===result[0]",
    );
}

#[test]
fn resumed_heritage_validates_superclass_and_reads_its_prototype_once() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],seen=[];
        async function f(value){class C extends (await value){static initialized=events.push('static')}return C}
        let base=new Proxy(function(){},{get(t,k,r){if(k==='prototype')events.push('prototype');return Reflect.get(t,k,r)}});
        f(base).then(C=>seen.push(Object.getPrototypeOf(C)===base));
        f(null).then(C=>seen.push(Object.getPrototypeOf(C.prototype)===null));
        for(let bad of [0,{},()=>{}])f(bad).catch(e=>seen.push(e instanceof TypeError));
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "seen.length===5&&seen.every(Boolean)&&events.filter(x=>x==='prototype').length===1&&events.filter(x=>x==='static').length===2",
    );
}

#[test]
fn heritage_closures_keep_outer_private_names_before_body_shadowing() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result;
        class Outer{#x=7;async make(){let read;class C extends (read=o=>o.#x,await Object){#x=9;own(){return this.#x}}return [new C().own(),read(this)]}}
        new Outer().make().then(v=>result=v);
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result.join(',')==='9,7'");
}

#[test]
fn heritage_rejection_restores_scope_and_sloppy_context_before_catch_and_finally() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let marker={},events=[],result;
        async function f(){
            try{class C extends (await Promise.reject(marker)){};events.push('wrong')}
            catch(e){heritageSloppy=7;events.push(e===marker,typeof C==='undefined')}
            finally{events.push(heritageSloppy===7)}
            return 9;
        }
        f().then(v=>result=v);
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===9&&events.length===3&&events.every(Boolean)&&heritageSloppy===7",
    );
}

#[test]
fn yielded_heritage_handles_resume_throw_and_return_with_context_cleanup() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let events=[],marker={};
        function* g(){try{class C extends (yield 1){value=8}return new C().value}catch(e){events.push(e===marker)}finally{generatorSloppy=4;events.push('finally')}}
        let success=g();if(success.next().value!==1||success.next(Object).value!==8)throw 1;
        let thrown=g();thrown.next();if(!thrown.throw(marker).done)throw 2;
        let returned=g();returned.next();let end=returned.return(99);
        end.done&&end.value===99&&generatorSloppy===4&&events.join(',')==='finally,true,finally,finally'
    "#,
    );
}

#[test]
fn module_default_class_awaits_heritage_before_static_initialization_and_naming() {
    let mut realm = Realm::default();
    realm.eval("let observed=[]").unwrap();
    let promise=realm.eval_module("export default class extends (await Object){static label=observed.push(this.name);static value=7};observed.push('body')").unwrap();
    let Value::Object(promise) = promise else {
        panic!("evaluation promise")
    };
    assert!(matches!(
        realm.inspect_object(&promise).unwrap().promise_state(),
        Some(PromiseState::Pending)
    ));
    check(&mut realm, "observed.length===0");
    drain(&mut realm);
    assert!(matches!(
        realm.inspect_object(&promise).unwrap().promise_state(),
        Some(PromiseState::Fulfilled(Value::Undefined))
    ));
    check(&mut realm, "observed.join(',')==='default,body'");
}

#[test]
fn heritage_host_failure_skips_language_handlers_and_restores_embedding_context() {
    let mut realm = Realm::default();
    realm.eval("let flag=0;async function f(){try{class C extends (await Object,import('unavailable')){}}catch{flag=1}finally{flag=2}};f()").unwrap();
    assert!(matches!(
        realm.run_next_promise_job(),
        Ok(PromiseJobOutcome::Reported(Error::Unsupported { .. }))
    ));
    check(&mut realm, "flag===0&&(heritageEmbeddingSloppy=7)===7");
}
