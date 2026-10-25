//! Non-suspending async functions, intrinsic metadata and Promise-returning calls.

use spite_runtime::{Error, Limits, PromiseJobOutcome, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
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
fn async_objects_have_the_intrinsic_graph_metadata_and_no_construct_capability() {
    check(
        r#"
        async function f(a,b=2,...rest){return a}
        let proto=Object.getPrototypeOf(f),C=proto.constructor;
        let p=Object.getOwnPropertyDescriptor(C,'prototype');
        let c=Object.getOwnPropertyDescriptor(proto,'constructor');
        let tag=Object.getOwnPropertyDescriptor(proto,Symbol.toStringTag);
        let caught=0;
        for(const target of [f,f.bind(null),new Proxy(f,{construct(){throw 1}})]){
            try{new target();throw 2}catch(e){if(e instanceof TypeError)caught++;else throw e}
        }
        typeof AsyncFunction==='undefined'&&typeof f==='function'&&typeof proto==='object'&&
        Object.getPrototypeOf(proto)===Function.prototype&&Object.getPrototypeOf(C)===Function&&
        C.name==='AsyncFunction'&&C.length===1&&f.name==='f'&&f.length===1&&
        Reflect.ownKeys(f).join(',')==='length,name'&&caught===3&&
        !p.writable&&!p.enumerable&&!p.configurable&&p.value===proto&&
        !c.writable&&!c.enumerable&&c.configurable&&c.value===C&&
        !tag.writable&&!tag.enumerable&&tag.configurable&&tag.value==='AsyncFunction'&&
        Object.prototype.toString.call(f)==='[object AsyncFunction]'&&
        f.toString()==='async function f(a,b=2,...rest){return a}'
    "#,
    );
}

#[test]
fn calls_run_bodies_synchronously_but_return_fresh_promises_and_queue_thenable_assimilation() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval("let log='',value;async function f(){log+='b';return{get then(){log+='g';return resolve=>{log+='t';resolve(9)}}}}let p=f();log+='s';p.then(v=>{value=v});p instanceof Promise&&log==='bgs'"),Ok(Value::Boolean(true)));
    drain(&mut realm);
    assert_eq!(
        realm.eval("value===9&&log==='bgst'"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(realm.eval("let original=Promise.resolve(7);async function identity(){return original}let wrapped=identity(),adopted;wrapped.then(v=>{adopted=v});wrapped!==original&&wrapped instanceof Promise"),Ok(Value::Boolean(true)));
    drain(&mut realm);
    assert_eq!(realm.eval("adopted"), Ok(Value::Number(7.0)));
    realm.eval("let intrinsic=Promise,returned;Promise=function(){throw 1};async function plain(){return 3}let hidden=plain();hidden.then(v=>{returned=v});if(!(hidden instanceof intrinsic))throw 2").unwrap();
    drain(&mut realm);
    assert_eq!(realm.eval("returned"), Ok(Value::Number(3.0)));
}

#[test]
fn parameter_body_and_return_getter_language_failures_reject_instead_of_throwing_synchronously() {
    let mut realm = Realm::default();
    realm.eval("let marker={},seen=0,sync=false,body=false;async function params({x}){body=true}async function defaults(x=(()=>{throw marker})()){body=true}async function thrown(){throw marker}async function getter(){return{get then(){throw marker}}}try{params(null).catch(e=>{if(!(e instanceof TypeError))throw 1;seen++});for(const f of [defaults,thrown,getter])f().catch(e=>{if(e!==marker)throw 2;seen++})}catch(e){sync=true}").unwrap();
    drain(&mut realm);
    assert_eq!(
        realm.eval("!sync&&!body&&seen===4"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn closures_receivers_arguments_and_internal_kind_survive_binding_and_prototype_changes() {
    let mut realm = Realm::default();
    realm.eval("let receiver={},values=[];async function f(a,a){a=3;return[this,arguments[1],new.target]}f.call(receiver,1,2).then(a=>{if(a[0]!==receiver||a[1]!==3||a[2]!==undefined)throw 1;values.push('call')});f.bind(receiver)(1,2).then(a=>{if(a[0]!==receiver||a[1]!==3)throw 2;values.push('bound')});async function strict(){'use strict';return this}strict.call(7).then(v=>{if(v!==7)throw 3;values.push('strict')});let proto=Object.getPrototypeOf(f),ordinary=function(){return 5};Object.setPrototypeOf(ordinary,proto);Object.setPrototypeOf(f,Function.prototype);if(ordinary()!==5)throw 4;f(1,2).then(()=>{values.push('kind')});let make=Function('return async function(){return 6}');make()().then(v=>{if(v!==6)throw 5;values.push('dynamic')})").unwrap();
    drain(&mut realm);
    assert_eq!(
        realm.eval("values.join(',')"),
        Ok(Value::String("call,bound,strict,kind,dynamic".into()))
    );
}

#[test]
fn closures_results_and_hidden_intrinsics_survive_collection() {
    let mut realm = Realm::default();
    realm.eval("let state=(()=>{let value={},symbol=Symbol(),fn=async function(){return[value,symbol]};return{fn,object:new WeakRef(value),symbol:new WeakRef(symbol)}})();let proto=Object.getPrototypeOf(state.fn),ctor=proto.constructor,weakCtor=new WeakRef(ctor);delete proto.constructor;ctor=null").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("state.object.deref()!==undefined&&state.symbol.deref()!==undefined&&weakCtor.deref()!==undefined"),Ok(Value::Boolean(true)));
    realm
        .eval("state.promise=state.fn();state.fn=null")
        .unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("state.object.deref()!==undefined&&state.symbol.deref()!==undefined"),
        Ok(Value::Boolean(true))
    );
    realm.eval("state.promise=null").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("state.object.deref()===undefined&&state.symbol.deref()===undefined"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn host_failures_bypass_rejection_and_await_execution_stays_explicitly_unsupported() {
    let mut realm = Realm::default();
    realm
        .eval("let flag=0;async function f(){return new SharedArrayBuffer(0)}")
        .unwrap();
    assert!(matches!(
        realm.eval("try{f()}catch(e){flag=1}finally{flag=2}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    for source in [
        "(async function(){}).constructor('await p')()",
        "new (async function(){}).constructor('await p')()",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    let mut limited = Realm::new(Limits {
        max_steps: Some(100_000),
        ..Limits::default()
    });
    limited
        .eval("let flag=0;async function f(){while(true){}}")
        .unwrap();
    assert!(matches!(
        limited.eval("try{f()}catch(e){flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(limited.eval("flag"), Ok(Value::Number(0.0)));
}

#[test]
fn unexecuted_await_bodies_and_dynamic_async_compilation_preserve_source_and_metadata() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        async function declared(){return await missing}
        let arrow=async()=>await missing,object={async m(){return await missing}};
        class C{async m(){return await missing}}
        let AsyncFunction=Object.getPrototypeOf(declared).constructor;
        let dynamic=AsyncFunction('value','return await value');
        typeof declared==='function'&&typeof arrow==='function'&&typeof object.m==='function'&&typeof C.prototype.m==='function'&&Object.getPrototypeOf(dynamic)===AsyncFunction.prototype&&dynamic.toString().includes('return await value')&&declared.toString()==='async function declared(){return await missing}'
    "#),Ok(Value::Boolean(true)));
}

#[test]
fn unsupported_await_execution_skips_operand_rejection_and_pending_finalizers() {
    let mut realm = Realm::default();
    realm.eval("let before=0,operand=0,handled=0;async function f(){before++;try{return await (operand++)}catch(e){handled=1}finally{handled=2}}").unwrap();
    assert!(matches!(
        realm.eval("try{f().catch(()=>{handled=3})}catch(e){handled=4}finally{handled=5}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(
        realm.eval("before===1&&operand===0&&handled===0"),
        Ok(Value::Boolean(true))
    );
}
