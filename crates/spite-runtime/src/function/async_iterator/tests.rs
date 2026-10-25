//! Hidden async iterator prototype contracts and Promise scheduling.

use crate::{Error, PromiseJobOutcome, Realm, Value};
use spite_core::{JsString, Span};

fn new_realm() -> Realm {
    let mut realm = Realm::default();
    realm.eval("0").unwrap();
    let prototype = realm
        .intrinsics
        .as_ref()
        .unwrap()
        .async_iterator
        .prototype
        .clone();
    let global = realm.global_object();
    realm
        .define_builtin_property(
            &global,
            "ap",
            Value::Object(prototype),
            true,
            Span::new(0, 0),
        )
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
            PromiseJobOutcome::Completed => {}
            PromiseJobOutcome::Reported(error) => panic!("{error}"),
        }
    }
}

#[test]
fn prototype_metadata_and_generic_identity_cover_every_receiver_kind() {
    check(
        &mut new_realm(),
        r#"
        let identity=ap[Symbol.asyncIterator],dispose=ap[Symbol.asyncDispose];
        if(Object.getPrototypeOf(ap)!==Object.prototype||Object.getOwnPropertyNames(ap).length!==0||Object.getOwnPropertySymbols(ap).length!==2||typeof AsyncIterator!=='undefined')throw 1;
        for(const [key,name]of[[Symbol.asyncIterator,'[Symbol.asyncIterator]'],[Symbol.asyncDispose,'[Symbol.asyncDispose]']]){
            let d=Object.getOwnPropertyDescriptor(ap,key),f=d.value,caught=false;
            try{new f()}catch(e){caught=e instanceof TypeError}
            if(!d.writable||d.enumerable||!d.configurable||f.name!==name||f.length!==0||Object.hasOwn(f,'prototype')||Object.getPrototypeOf(f)!==Function.prototype||!caught)throw 2;
            for(const k of['name','length']){let m=Object.getOwnPropertyDescriptor(f,k);if(m.writable||m.enumerable||!m.configurable)throw 3}
        }
        let revoked=Proxy.revocable({},{});revoked.revoke();
        for(const value of[undefined,null,0,false,'x',1n,Symbol(),{},revoked.proxy])if(!Object.is(identity.call(value),value))throw 4;
        true
    "#,
    );
}

#[test]
fn missing_return_fulfills_and_invalid_receivers_or_methods_reject_fresh_promises() {
    let mut realm = new_realm();
    realm.eval(r#"
        let dispose=ap[Symbol.asyncDispose],done=0,errors=0,first=dispose.call({}),second=dispose.call({});
        if(first===second||!(first instanceof Promise)||Object.getPrototypeOf(first)!==Promise.prototype)throw 1;
        first.then(v=>{if(v===undefined)done++});second.then(v=>{if(v===undefined)done++});
        for(const value of[null,undefined,{return:1},{return:false}])dispose.call(value).catch(e=>{if(e instanceof TypeError)errors++});
        let values=[];Number.prototype.return=function(){'use strict';if(this!==3||arguments.length!==0)throw 2;values.push(this);return 99};
        dispose.call(3,1,2).then(v=>{if(v===undefined)done++});delete Number.prototype.return;
    "#).unwrap();
    check(&mut realm, "done===0&&errors===0&&values.join(',')==='3'");
    drain(&mut realm);
    check(&mut realm, "done===3&&errors===4");
}

#[test]
fn return_lookup_is_cached_and_its_result_is_awaited_then_discarded() {
    let mut realm = new_realm();
    realm.eval(r#"
        let gets=0,calls=0,done=false,release,gate=new Promise(r=>{release=r});
        let receiver={get return(){gets++;return function(){'use strict';if(this!==receiver||arguments.length!==0)throw 1;calls++;return gate}}};
        let result=ap[Symbol.asyncDispose].call(receiver,1,2);
        Object.defineProperty(receiver,'return',{value(){throw 2}});result.then(v=>{done=v===undefined});
    "#).unwrap();
    check(&mut realm, "gets===1&&calls===1&&!done");
    drain(&mut realm);
    check(&mut realm, "!done");
    realm.eval("release(99)").unwrap();
    drain(&mut realm);
    check(&mut realm, "done&&gets===1&&calls===1");
}

#[test]
fn lookup_call_and_promise_resolution_errors_preserve_original_reasons() {
    let mut realm = new_realm();
    realm
        .eval(
            r#"
        let a={},b={},c={},values=[],dispose=ap[Symbol.asyncDispose],wait=Promise.resolve();
        Object.defineProperty(wait,'constructor',{get(){throw c}});
        dispose.call({get return(){throw a}}).catch(e=>{values.push(e===a)});
        dispose.call({return(){throw b}}).catch(e=>{values.push(e===b)});
        dispose.call({return(){return wait}}).catch(e=>{values.push(e===c)});
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "values.length===3&&values.every(v=>v)");
}

#[test]
fn intrinsic_await_ignores_public_then_species_and_replaced_promise_bindings() {
    let mut realm = new_realm();
    realm.eval(r#"
        let Saved=Promise,then=Promise.prototype.then,wait=Saved.resolve(1),done=false;
        Object.defineProperty(wait,'then',{get(){throw 1}});Object.defineProperty(Saved,Symbol.species,{get(){throw 2}});
        let result=ap[Symbol.asyncDispose].call({return(){return wait}});Promise=function(){throw 3};
        Object.defineProperty(result,'constructor',{value:undefined});Reflect.apply(then,result,[v=>{done=v===undefined}]);
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "done");
    let mut realm = new_realm();
    realm.eval("let flag=0").unwrap();
    assert!(matches!(realm.eval("try{ap[Symbol.asyncDispose].call({return(){return new SharedArrayBuffer(0)}})}catch(e){flag=1}finally{flag=2}"),Err(Error::Unsupported{..})));
    check(&mut realm, "flag===0");
}

#[test]
fn awaited_capabilities_survive_collection_without_retaining_unused_receivers() {
    let mut realm = new_realm();
    realm.eval(r#"
        let state={},done=false;state.gate=new Promise(r=>{state.release=r});
        (()=>{let receiver={return(){return state.gate}};state.weak=new WeakRef(receiver);state.result=ap[Symbol.asyncDispose].call(receiver)})();
        state.result.then(v=>{done=v===undefined});
    "#).unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(&mut realm, "state.weak.deref()===undefined&&!done");
    realm.eval("state.release(7)").unwrap();
    drain(&mut realm);
    check(&mut realm, "done");

    let mut realm = new_realm();
    let intrinsic = &realm.intrinsics.as_ref().unwrap().async_iterator;
    let handles = [
        intrinsic.prototype.clone(),
        intrinsic.identity.clone(),
        intrinsic.dispose.clone(),
    ];
    realm
        .eval("delete ap[Symbol.asyncIterator];delete ap[Symbol.asyncDispose];delete globalThis.ap")
        .unwrap();
    assert_eq!(
        realm.collect(usize::MAX).unwrap().live,
        crate::test_support::REALM_ENTRIES
    );
    for handle in handles {
        assert!(realm.inspect_object(&handle).is_ok());
    }
    assert_eq!(
        realm.eval("typeof ap"),
        Ok(Value::String(JsString::from("undefined")))
    );
}
