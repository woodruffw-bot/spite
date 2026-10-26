//! Generic NewPromiseCapability initialization and Promise static method ordering.

use spite_runtime::{Error, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn capability_executor_has_native_metadata_and_single_initialization() {
    check(
        "let saved,r=function(){},j=function(){},p={};function C(executor){saved=executor;if(arguments.length!==1)throw 1;executor(undefined,undefined);executor(r,j);return p}let cap=Promise.withResolvers.call(C);cap.promise===p&&cap.resolve===r&&cap.reject===j&&saved.name===''&&saved.length===2&&Object.getPrototypeOf(saved)===Function.prototype&&Object.getOwnPropertyNames(saved).join(',')==='length,name'&&!Object.hasOwn(saved,'prototype')&&saved.toString()==='function () { [native code] }'",
    );
    check(
        "let saved;function C(executor){saved=executor;executor(()=>{},()=>{})}Promise.withResolvers.call(C);let caught=0;try{saved(()=>{},()=>{})}catch(e){if(e instanceof TypeError)caught++}try{new saved()}catch(e){if(e instanceof TypeError)caught++}caught===2",
    );
}

#[test]
fn receiver_type_precedes_callback_but_constructor_and_capability_validation_follow_it() {
    check(
        "let calls=0;for(const C of [undefined,null,1,{},()=>{}]){let caught=false;try{Promise.try.call(C,()=>{calls++})}catch(e){caught=e instanceof TypeError}if(!caught)throw 1;}calls===2",
    );
    check(
        "let calls=0;for(const pair of [[undefined,()=>{}],[()=>{},undefined],[1,()=>{}],[()=>{},{}]]){function C(executor){executor(pair[0],pair[1])}let caught=false;try{Promise.try.call(C,()=>{calls++})}catch(e){caught=e instanceof TypeError}if(!caught)throw 1;}calls===4",
    );
    check(
        "let marker={},called=false;function C(){throw marker}let caught=false;try{Promise.try.call(C,()=>{called=true})}catch(e){caught=e===marker}caught&&called",
    );
}

#[test]
fn with_resolvers_creates_ordered_intrinsic_data_properties_without_setters() {
    check(
        "let r=()=>{},j=()=>{},p={};function C(executor){executor(r,j);return p}for(const key of ['promise','resolve','reject'])Object.defineProperty(Object.prototype,key,{set(){throw 1},configurable:true});let cap=Promise.withResolvers.call(C);for(const key of ['promise','resolve','reject']){let d=Object.getOwnPropertyDescriptor(cap,key);if(!d.writable||!d.enumerable||!d.configurable)throw 2;}Object.getPrototypeOf(cap)===Object.prototype&&Object.keys(cap).join(',')==='promise,resolve,reject'&&cap.promise===p&&cap.resolve===r&&cap.reject===j",
    );
}

#[test]
fn reject_and_try_call_exact_callbacks_with_undefined_receiver_and_original_values() {
    check(
        "let seen,receiver,args,reason=new Proxy({},{get(){throw 1}}),p={};function C(executor){executor(()=>{throw 2},function(v){'use strict';receiver=this;args=arguments.length;seen=v});return p}Promise.reject.call(C,reason)===p&&seen===reason&&receiver===undefined&&args===1",
    );
    check(
        "let log='',p={},result={},first={},second={};function C(executor){log+='c';executor(function(v){'use strict';if(this!==undefined||arguments.length!==1||v!==result)throw 1;log+='r'},()=>{throw 2});return p}let out=Promise.try.call(C,function(a,b){'use strict';if(this!==undefined||arguments.length!==2||a!==first||b!==second)throw 3;log+='f';return result},first,second);out===p&&log==='fcr'",
    );
}

#[test]
fn callback_errors_are_rejected_but_resolver_errors_propagate_unchanged() {
    check(
        "let marker={},seen,p={};function C(executor){executor(()=>{throw 1},v=>{seen=v});return p}Promise.try.call(C,()=>{throw marker})===p&&seen===marker",
    );
    check(
        "let seen,p={};function C(executor){executor(()=>{throw 1},v=>{seen=v});return p}Promise.try.call(C,undefined)===p&&seen instanceof TypeError&&Error.isError(seen)",
    );
    check(
        "let marker={},rejected=false;function C(executor){executor(()=>{throw marker},()=>{rejected=true})}let caught=false;try{Promise.try.call(C,()=>1)}catch(e){caught=e===marker}caught&&!rejected",
    );
    let mut realm = Realm::default();
    realm
        .eval(
            "let rejected=false,flag=0;function C(executor){executor(()=>{},()=>{rejected=true})}",
        )
        .unwrap();
    assert!(matches!(realm.eval("try{Promise.try.call(C,()=>{Function('async function* gap(){}')})}catch{flag=1}finally{flag=2}"),Err(Error::Unsupported{..})));
    assert_eq!(realm.eval("!rejected&&flag===0"), Ok(Value::Boolean(true)));
}

#[test]
fn intrinsic_try_runs_the_callback_and_preserves_native_promise_identity() {
    check(
        "let calls=0,sentinel=Promise.resolve(7),result=Promise.try(()=>{calls++;return sentinel});calls===1&&result===sentinel",
    );
}

#[test]
fn retained_executor_keeps_private_callback_captures_alive_across_collection() {
    let mut realm = Realm::default();
    realm.eval("let saved,state=(()=>{let value={},weak=new WeakRef(value);function C(executor){saved=executor;executor(()=>value,()=>{})}Promise.withResolvers.call(C);return weak})();").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("state.deref()!==undefined"),
        Ok(Value::Boolean(true))
    );
    realm.eval("saved=null").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("state.deref()===undefined"),
        Ok(Value::Boolean(true))
    );
}
