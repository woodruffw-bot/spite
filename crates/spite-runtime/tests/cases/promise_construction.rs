//! Promise construction, branded resolution and host-scheduled thenables (27.2).

use spite_runtime::{PromiseJobOutcome, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn constructor_validates_executor_before_prototype_and_invokes_it_synchronously() {
    check(
        "let calls=0,resolve,reject,p=new Promise(function(r,j){'use strict';if(this!==undefined||arguments.length!==2)throw 1;resolve=r;reject=j;calls++});calls===1&&p instanceof Promise&&Object.getOwnPropertyNames(p).length===0&&resolve.length===1&&reject.length===1&&resolve.name===''&&reject.name===''&&Object.getPrototypeOf(resolve)===Function.prototype&&!Object.hasOwn(resolve,'prototype')&&Object.getOwnPropertyNames(resolve).join(',')==='length,name'",
    );
    check(
        "let gets=0,calls=0,marker={},N=new Proxy(function(){},{get(t,p){if(p==='prototype'){gets++;throw marker}return t[p]}}),bad=false,abrupt=false;try{Reflect.construct(Promise,[null],N)}catch(e){bad=e instanceof TypeError}let before=gets;try{Reflect.construct(Promise,[()=>{calls++}],N)}catch(e){abrupt=e===marker}bad&&before===0&&abrupt&&gets===1&&calls===0",
    );
    check(
        "class S extends Promise{}let p=new S(()=>{});p instanceof S&&p instanceof Promise&&Object.getPrototypeOf(p)===S.prototype",
    );
    check(
        "let Saved=Promise,proto=Saved.prototype;function F(){}F.prototype=null;Promise=function(){throw 1};Object.getPrototypeOf(Reflect.construct(Saved,[()=>{}],F))===proto",
    );
}

#[test]
fn resolution_identity_requires_the_brand_and_constructor_identity_before_constructibility() {
    check(
        "let p=Promise.resolve(7);Object.defineProperty(p,'then',{get(){throw 1}});Promise.resolve(p)===p&&Promise.try(()=>p)===p",
    );
    check(
        "let p=Promise.resolve(7),C={};p.constructor=C;Promise.resolve.call(C,p)===p&&Promise.try.call(C,()=>p)===p",
    );
    check(
        "let fake={get constructor(){throw 1},[Symbol.toStringTag]:'Promise'},p=Promise.resolve(fake);p!==fake&&p instanceof Promise",
    );
    check(
        "let p=Promise.resolve(7),marker={},caught=0;Object.defineProperty(p,'constructor',{get(){throw marker}});try{Promise.resolve(p)}catch(e){if(e===marker)caught++}try{Promise.try(()=>p)}catch(e){if(e===marker)caught++}caught===2",
    );
}

#[test]
fn thenable_jobs_run_one_at_a_time_and_append_nested_jobs_in_fifo_order() {
    let mut realm = Realm::default();
    realm.eval("let log='';Promise.resolve({then(resolve){log+='a';resolve({then(resolve){log+='c';resolve(1)}})}});Promise.resolve({then(resolve){log+='b';resolve(2)}});").unwrap();
    assert_eq!(realm.eval("log"), Ok(Value::String("".into())));
    for expected in ["a", "ab", "abc"] {
        assert!(matches!(
            realm.run_next_promise_job(),
            Ok(PromiseJobOutcome::Completed)
        ));
        realm.collect(usize::MAX).unwrap();
        assert_eq!(realm.eval("log"), Ok(Value::String(expected.into())));
    }
    assert!(matches!(
        realm.run_next_promise_job(),
        Ok(PromiseJobOutcome::NoJob)
    ));
}
