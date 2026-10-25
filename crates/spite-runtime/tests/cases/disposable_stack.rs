//! Synchronous DisposableStack captures, reentry and cleanup completions.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn cached_disposers_preserve_receivers_arguments_and_reverse_registration_order() {
    check(
        "let s=new DisposableStack,log='',gets=0,value={},r={get [Symbol.dispose](){gets++;return function(){'use strict';if(this!==r||arguments.length!==0)throw 1;log+='u';return {get then(){throw 2}}}}};s.use(r);Object.defineProperty(r,Symbol.dispose,{value(){throw 3}});s.adopt(value,function(v){'use strict';if(this!==undefined||arguments.length!==1||v!==value)throw 4;log+='a'});s.defer(function(){'use strict';if(this!==undefined||arguments.length!==0)throw 5;log+='d'});s.dispose()===undefined&&log==='dau'&&gets===1&&s.disposed&&s.dispose()===undefined",
    );
}

#[test]
fn receiver_and_state_validation_precede_resource_and_callback_observation() {
    check(
        "let p=DisposableStack.prototype,get=Object.getOwnPropertyDescriptor(p,'disposed').get,traps=0,proxy=new Proxy(new DisposableStack,{get(){traps++;throw 1}}),rev=Proxy.revocable(new DisposableStack,{});rev.revoke();for(const value of [undefined,null,1,'',true,1n,Symbol(),{},p,proxy,rev.proxy]){for(const f of [p.use,p.adopt,p.defer,p.dispose,p.move,get]){let caught=false;try{f.call(value)}catch(e){caught=e instanceof TypeError}if(!caught)throw 2;}}traps===0",
    );
    check(
        "let s=new DisposableStack,calls=0,r={get [Symbol.dispose](){calls++;throw 1}};s.dispose();let caught=0;for(const f of [()=>s.use(r),()=>s.adopt(1,null),()=>s.defer(null),()=>s.move()]){try{f()}catch(e){if(e instanceof ReferenceError)caught++}}caught===4&&calls===0",
    );
}

#[test]
fn resource_lookup_keeps_the_original_list_when_the_stack_is_moved_reentrantly() {
    check(
        "let s=new DisposableStack,moved,log='';s.defer(()=>{log+='old'});let resource={get [Symbol.dispose](){moved=s.move();return function(){log+='new'}}};s.use(resource)===resource&&s.disposed&&!moved.disposed&&s.dispose()===undefined&&log===''&&moved.dispose()===undefined&&log==='newold'",
    );
    check(
        "let s=new DisposableStack,calls=0,r={get [Symbol.dispose](){s.dispose();return function(){calls++}}};s.use(r)===r&&s.disposed&&s.dispose()===undefined&&calls===0",
    );
}

#[test]
fn disposal_marks_state_before_callbacks_and_ignores_returned_thenables() {
    check(
        "let s=new DisposableStack,calls=0;s.defer(()=>{calls++});s.defer(()=>{if(!s.disposed||s.dispose()!==undefined)throw 1;let caught=false;try{s.defer(()=>{})}catch(e){caught=e instanceof ReferenceError}if(!caught)throw 2;return {get then(){throw 3}}});s[Symbol.dispose]()===undefined&&s.disposed&&calls===1&&s.dispose()===undefined",
    );
}

#[test]
fn cleanup_errors_form_intrinsic_suppression_chains_and_preserve_single_throws() {
    check(
        "let s=new DisposableStack,marker={},calls=0;s.defer(()=>{calls++});s.defer(()=>{throw marker});let caught=false;try{s.dispose()}catch(e){caught=e===marker}caught&&calls===1&&s.disposed&&s.dispose()===undefined",
    );
    check(
        "let s=new DisposableStack,Saved=SuppressedError,a={},b={},c={};s.defer(()=>{throw a});s.defer(()=>{throw b});s.defer(()=>{throw c});SuppressedError=function(){throw 9};let caught;try{s.dispose()}catch(e){caught=e}Object.getPrototypeOf(caught)===Saved.prototype&&caught.error===a&&caught.suppressed.error===b&&caught.suppressed.suppressed===c&&Error.isError(caught)&&Object.getOwnPropertyNames(caught).join(',')==='error,suppressed'",
    );
    check(
        "let s=new DisposableStack;s.defer(()=>{null.x});s.defer(()=>{throw undefined});let caught;try{s.dispose()}catch(e){caught=e}caught instanceof SuppressedError&&caught.error instanceof TypeError&&Object.hasOwn(caught,'suppressed')&&caught.suppressed===undefined",
    );
}

#[test]
fn move_uses_intrinsics_transfers_ownership_and_ignores_species_and_constructor_hooks() {
    check(
        "let Saved=DisposableStack,proto=Saved.prototype;class S extends Saved{}let s=new S,calls=0;s.defer(()=>{calls++});Object.defineProperty(s,'constructor',{get(){throw 1}});DisposableStack=function(){throw 2};let target=s.move();s.disposed&&!target.disposed&&Object.getPrototypeOf(target)===proto&&!(target instanceof S)&&s.dispose()===undefined&&calls===0&&target.dispose()===undefined&&calls===1",
    );
}

#[test]
fn resource_objects_and_symbols_survive_collection_and_are_released_after_disposal() {
    let mut realm = Realm::default();
    realm.eval("let state=(()=>{let value={},symbol=Symbol(),stack=new DisposableStack;stack.adopt(value,()=>{});stack.adopt(symbol,()=>{});return{stack,object:new WeakRef(value),symbol:new WeakRef(symbol)}})();").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("state.object.deref()!==undefined&&state.symbol.deref()!==undefined"),
        Ok(Value::Boolean(true))
    );
    realm.eval("state.stack.dispose()").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("state.object.deref()===undefined&&state.symbol.deref()===undefined"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn unreachable_resource_symbols_do_not_keep_weak_map_cycles_alive() {
    let mut realm = Realm::default();
    realm.eval("let state=(()=>{let symbol=Symbol(),stack=new DisposableStack;stack.adopt(symbol,()=>{});return{map:new WeakMap([[symbol,stack]]),weak:new WeakRef(symbol)}})();").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("state.weak.deref()===undefined"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn default_resource_lists_are_unrestricted_and_host_failures_skip_remaining_cleanup() {
    check(
        "let s=new DisposableStack,n=0;for(let i=0;i<5000;i++)s.defer(()=>{n++});s.dispose();n===5000&&s.disposed",
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let s=new DisposableStack,called=false,flag=0;s.defer(()=>{called=true});s.defer(()=>{while(true){}});").unwrap();
    assert!(matches!(
        realm.eval("try{s.dispose()}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("s.disposed&&!called&&flag===0&&s.dispose()===undefined"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::default();
    realm.eval("let s=new DisposableStack,called=false,flag=0;s.defer(()=>{called=true});s.defer(()=>{Function('function* gap(){}')});").unwrap();
    assert!(matches!(
        realm.eval("try{s.dispose()}catch{flag=1}finally{flag=2}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(
        realm.eval("s.disposed&&!called&&flag===0&&s.dispose()===undefined"),
        Ok(Value::Boolean(true))
    );
}
