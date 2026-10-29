//! Async resource registration, native suspension, suppression and collection.

use crate::common::UnavailableHostRealm;
use spite_runtime::{Error, PromiseJobOutcome, Realm, Value};

fn drain(realm: &mut Realm) {
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => {}
            PromiseJobOutcome::Reported(error) => panic!("{error}"),
        }
    }
}

fn check(realm: &mut Realm, source: &str) {
    assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
}

#[test]
fn constructor_metadata_descriptors_subclasses_and_prototype_selection() {
    check(
        &mut Realm::default(),
        r#"
        let C=AsyncDisposableStack,p=C.prototype;
        let d=Object.getOwnPropertyDescriptor(C,'prototype');
        if(C.name!=='AsyncDisposableStack'||C.length!==0||d.writable||d.enumerable||d.configurable)throw 1;
        if(Object.getPrototypeOf(C)!==Function.prototype||Object.getPrototypeOf(p)!==Object.prototype)throw 2;
        let alias=Object.getOwnPropertyDescriptor(p,Symbol.asyncDispose),tag=Object.getOwnPropertyDescriptor(p,Symbol.toStringTag);
        if(alias.value!==p.disposeAsync||!alias.writable||alias.enumerable||!alias.configurable||tag.value!=='AsyncDisposableStack'||tag.writable||tag.enumerable||!tag.configurable)throw 3;
        for(const [name,length]of[['adopt',2],['defer',1],['disposeAsync',0],['move',0],['use',1]]){
            let descriptor=Object.getOwnPropertyDescriptor(p,name),f=descriptor.value,caught=false;
            try{new f()}catch(e){caught=e instanceof TypeError}
            if(f.name!==name||f.length!==length||!descriptor.writable||descriptor.enumerable||!descriptor.configurable||!caught)throw 4;
        }
        let caught=false;try{C()}catch(e){caught=e instanceof TypeError}
        class S extends C{}let s=new S;
        let custom={},target=function(){};target.prototype=custom;
        let instance=Reflect.construct(C,[],target);target.prototype=1;
        caught&&s instanceof S&&!s.disposed&&Object.isExtensible(s)&&Object.getPrototypeOf(instance)===custom&&Object.getPrototypeOf(Reflect.construct(C,[],target))===p
    "#,
    );
}

#[test]
fn distinct_brands_and_state_checks_precede_observation_and_dispose_rejects() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let p=AsyncDisposableStack.prototype,get=Object.getOwnPropertyDescriptor(p,'disposed').get,traps=0,rejected=0;
        let proxy=new Proxy(new AsyncDisposableStack,{get(){traps++;throw 1}}),rev=Proxy.revocable(new AsyncDisposableStack,{});rev.revoke();
        let values=[undefined,null,1,'',true,1n,Symbol(),{},p,new DisposableStack,proxy,rev.proxy];
        for(const value of values){
            for(const f of[p.use,p.adopt,p.defer,p.move,get]){
                let caught=false;try{f.call(value)}catch(e){caught=e instanceof TypeError}if(!caught)throw 2;
            }
            let result=p.disposeAsync.call(value);
            if(!(result instanceof Promise))throw 3;
            result.catch(e=>{if(e instanceof TypeError)rejected++});
        }
        let s=new AsyncDisposableStack,calls=0,resource={get[Symbol.asyncDispose](){calls++;throw 4}};
        s.disposeAsync();let caught=0;
        for(const f of[()=>s.use(resource),()=>s.adopt(1,null),()=>s.defer(null),()=>s.move()]){
            try{f()}catch(e){if(e instanceof ReferenceError)caught++}
        }
        if(caught!==4||calls!==0||traps!==0)throw 5;
        let sync=new DisposableStack,async=new AsyncDisposableStack;
        try{DisposableStack.prototype.use.call(async,null);throw 6}catch(e){if(!(e instanceof TypeError))throw e}
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "rejected===values.length&&traps===0");
}

#[test]
fn cached_methods_preserve_receivers_arguments_and_async_precedence() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let s=new AsyncDisposableStack,log=[],gets=0,value={};
        let r={get[Symbol.asyncDispose](){gets++;return function(){'use strict';if(this!==r||arguments.length!==0)throw 1;log.push('use')}},get[Symbol.dispose](){throw 2}};
        if(s.use(r)!==r||s.use(null)!==null||s.use(undefined)!==undefined)throw 3;
        Object.defineProperty(r,Symbol.asyncDispose,{value(){throw 4}});
        if(s.adopt(value,function(v){'use strict';if(this!==undefined||arguments.length!==1||v!==value)throw 5;log.push('adopt')})!==value)throw 6;
        if(s.defer(function(){'use strict';if(this!==undefined||arguments.length!==0)throw 7;log.push('defer')})!==undefined)throw 8;
        let done=false;s[Symbol.asyncDispose]().then(v=>{done=v===undefined});
    "#).unwrap();
    check(&mut realm, "s.disposed&&log.join(',')==='defer'&&!done");
    drain(&mut realm);
    check(
        &mut realm,
        "done&&gets===1&&log.join(',')==='defer,adopt,use'",
    );
}

#[test]
fn pending_cleanup_suspends_before_older_resources_and_reentry_is_independent() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let s=new AsyncDisposableStack,log=[],release,gate=new Promise(r=>{release=r});
        s.defer(()=>{log.push('last')});
        s.defer(()=>{if(!s.disposed)throw 1;log.push('first');return gate});
        let original=s.disposeAsync();original.then(()=>{log.push('done')});
        let again=s.disposeAsync();if(again===original)throw 2;again.then(()=>{log.push('again')});
    "#,
        )
        .unwrap();
    check(&mut realm, "s.disposed&&log.join(',')==='first'");
    drain(&mut realm);
    check(&mut realm, "log.join(',')==='first,again'");
    realm.eval("release(9)").unwrap();
    drain(&mut realm);
    check(&mut realm, "log.join(',')==='first,again,last,done'");
}

#[test]
fn synchronous_and_asynchronous_errors_form_intrinsic_suppression_chains() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let s=new AsyncDisposableStack,Saved=SuppressedError,a={},b={},c={},caught;
        s.defer(()=>{throw a});s.defer(()=>Promise.reject(b));s.defer(()=>{throw c});
        SuppressedError=function(){throw 9};s.disposeAsync().catch(e=>{caught=e});
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        r#"Object.getPrototypeOf(caught)===Saved.prototype&&caught.error===a&&caught.suppressed.error===b&&caught.suppressed.suppressed===c&&Error.isError(caught)&&Object.getOwnPropertyNames(caught).join(',')==='error,suppressed'"#,
    );
    let mut realm = Realm::default();
    realm.eval("let s=new AsyncDisposableStack,passed=false;s.defer(()=>{throw undefined});s.disposeAsync().catch(e=>{passed=e===undefined})").unwrap();
    drain(&mut realm);
    check(&mut realm, "passed");
}

#[test]
fn sync_fallback_ignores_results_and_turns_sync_throws_into_awaited_rejections() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let s=new AsyncDisposableStack,log=[],marker={},caught;
        s.defer(()=>{log.push('older')});
        s.use({[Symbol.dispose](){log.push('sync');throw marker}});
        s.use({[Symbol.dispose](){log.push('ignored');return{get then(){throw 1}}}});
        s.disposeAsync().catch(e=>{caught=e});
    "#,
        )
        .unwrap();
    check(&mut realm, "log.join(',')==='ignored'");
    assert!(matches!(
        realm.run_next_promise_job(),
        Ok(PromiseJobOutcome::Completed)
    ));
    check(&mut realm, "log.join(',')==='ignored,sync'");
    drain(&mut realm);
    check(
        &mut realm,
        "caught===marker&&log.join(',')==='ignored,sync,older'",
    );
}

#[test]
fn empty_and_nullish_resources_have_the_specified_job_order() {
    for registration in [
        "",
        "stack.use(null)",
        "stack.use(undefined)",
        "stack.use(null);stack.use(undefined)",
    ] {
        let mut realm = Realm::default();
        let prefix = if registration.is_empty() {
            ""
        } else {
            ".then(()=>0)"
        };
        realm.eval(&format!("let stack=new AsyncDisposableStack,sequence=[];{registration};Promise.all([Promise.resolve(){prefix}.then(()=>{{sequence.push('job 1')}}),stack.disposeAsync().then(()=>{{sequence.push('dispose')}}),Promise.resolve(){prefix}.then(()=>{{sequence.push('job 2')}})])")).unwrap();
        drain(&mut realm);
        check(&mut realm, "sequence.join(',')==='job 1,dispose,job 2'");
    }
}

#[test]
fn move_uses_intrinsics_and_getter_reentry_retains_the_original_resource_list() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let Saved=AsyncDisposableStack,proto=Saved.prototype,log=[];class S extends Saved{}
        let source=new S,moved;source.defer(()=>{log.push('old')});
        let resource={get[Symbol.asyncDispose](){moved=source.move();return()=>{log.push('new')}}};
        if(source.use(resource)!==resource||!source.disposed||moved.disposed||Object.getPrototypeOf(moved)!==proto||moved instanceof S)throw 1;
        Object.defineProperty(moved,'constructor',{get(){throw 2}});AsyncDisposableStack=function(){throw 3};
        let target=moved.move();if(!moved.disposed||target.disposed||Object.getPrototypeOf(target)!==proto)throw 4;
        source.disposeAsync();moved.disposeAsync();target.disposeAsync();
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "log.join(',')==='new,old'");
}

#[test]
fn internal_await_bypasses_then_and_species_and_uses_intrinsic_promises() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let Saved=Promise,then=Promise.prototype.then,s=new AsyncDisposableStack,wait=Saved.resolve(1),done=false,log=[];
        Object.defineProperty(wait,'then',{get(){throw 1}});
        Object.defineProperty(Saved,Symbol.species,{get(){throw 2}});
        s.defer(()=>{log.push('older')});s.defer(()=>wait);
        let result=s.disposeAsync();Promise=function(){throw 3};
        Object.defineProperty(result,'constructor',{value:undefined});
        Reflect.apply(then,result,[()=>{done=true}]);
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "done&&log.join(',')==='older'&&Object.getPrototypeOf(s.disposeAsync())===Saved.prototype",
    );
}

#[test]
fn promise_constructor_lookup_errors_and_thenable_jobs_follow_await_semantics() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let s=new AsyncDisposableStack,marker={},caught,log=[],wait=Promise.resolve();
        Object.defineProperty(wait,'constructor',{get(){throw marker}});
        s.defer(()=>{log.push('older')});s.defer(()=>wait);
        s.disposeAsync().catch(e=>{caught=e});
    "#,
        )
        .unwrap();
    check(&mut realm, "log.join(',')==='older'");
    drain(&mut realm);
    check(&mut realm, "caught===marker");
    let mut realm = Realm::default();
    realm.eval(r#"
        let s=new AsyncDisposableStack,log=[],done=false,thenable={get then(){log.push('get');return function(resolve,reject){if(this!==thenable)throw 1;log.push('then');resolve(1);reject(2)}}};
        s.defer(()=>{log.push('older')});s.defer(()=>thenable);s.disposeAsync().then(()=>{done=true});
    "#).unwrap();
    check(&mut realm, "log.join(',')==='get'");
    drain(&mut realm);
    check(&mut realm, "done&&log.join(',')==='get,then,older'");
}

#[test]
fn resources_and_suppressed_failures_are_traced_across_suspension_and_released() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},completed=false;
        (()=>{let object={},symbol=Symbol(),error={};state.stack=new AsyncDisposableStack;
            state.object=new WeakRef(object);state.symbol=new WeakRef(symbol);state.error=new WeakRef(error);
            state.stack.adopt(object,()=>{});state.stack.adopt(symbol,()=>{});
            state.gate=new Promise(r=>{state.release=r});state.stack.defer(()=>state.gate);state.stack.defer(()=>{throw error});
        })();
        state.result=state.stack.disposeAsync();state.stack=null;
        state.result.catch(e=>{completed=e===state.error.deref()});
    "#).unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "state.object.deref()!==undefined&&state.symbol.deref()!==undefined&&state.error.deref()!==undefined",
    );
    realm
        .eval("state.release();state.release=null;state.gate=null")
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "completed");
    realm.eval("state.result=null").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "state.object.deref()===undefined&&state.symbol.deref()===undefined&&state.error.deref()===undefined",
    );
}

#[test]
fn host_failures_abort_initial_and_resumed_cleanup_without_js_rejection() {
    let mut realm = Realm::with_unavailable_host();
    realm.eval("let flag=0,s=new AsyncDisposableStack;s.defer(()=>{flag=3});s.defer(()=>spiteUnavailableHost())").unwrap();
    assert!(matches!(
        realm.eval("try{s.disposeAsync()}catch(e){flag=1}finally{flag=2}"),
        Err(Error::Unsupported { .. })
    ));
    check(&mut realm, "flag===0&&s.disposed");
    let mut realm = Realm::with_unavailable_host();
    realm.eval("let flag=0,s=new AsyncDisposableStack;s.defer(()=>{flag=3});s.defer(()=>spiteUnavailableHost());s.defer(()=>undefined);let result=s.disposeAsync();result.catch(()=>{flag=1})").unwrap();
    assert!(matches!(
        realm.run_next_promise_job(),
        Ok(PromiseJobOutcome::Reported(Error::Unsupported { .. }))
    ));
    drain(&mut realm);
    check(&mut realm, "flag===0&&s.disposed");
}
