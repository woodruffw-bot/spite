//! Async lexical closures, intrinsic metadata and Promise-returning arrow calls.

use spite_runtime::{Error, PromiseJobOutcome, Realm, Value};

fn drain(realm: &mut Realm) {
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => {}
            PromiseJobOutcome::Reported(e) => panic!("{e}"),
        }
    }
}

#[test]
fn async_arrows_have_standard_metadata_source_and_no_construction_capability() {
    assert_eq!(Realm::default().eval("let f=async /*keep*/ (a,b=1)=>a+b,AF=(async function(){}).constructor,caught=false;try{new f()}catch(e){caught=e instanceof TypeError}caught&&f.name==='f'&&f.length===1&&!Object.hasOwn(f,'prototype')&&Object.getPrototypeOf(f)===AF.prototype&&f.toString()==='async /*keep*/ (a,b=1)=>a+b'&&Object.prototype.toString.call(f)==='[object AsyncFunction]'"),Ok(Value::Boolean(true)));
}

#[test]
fn expression_and_block_bodies_return_fresh_promises_and_reject_language_failures() {
    let mut realm = Realm::default();
    realm.eval("let seen=[],marker={},original=Promise.resolve(7);let f=async x=>x+1,g=async()=>{return 4},h=async()=>{throw marker},empty=async()=>{},params=async({x})=>x;f(1).then(v=>{seen.push(v)});g().then(v=>{seen.push(v)});h().catch(e=>{if(e!==marker)throw 1;seen.push('throw')});empty().then(v=>{if(v!==undefined)throw 2;seen.push('empty')});params(null).catch(e=>{if(!(e instanceof TypeError))throw 3;seen.push('params')});let p=(async()=>original)();if(p===original)throw 4;p.then(v=>{seen.push(v)})").unwrap();
    drain(&mut realm);
    assert_eq!(
        realm.eval("seen.join(',')"),
        Ok(Value::String("2,4,throw,empty,params,7".into()))
    );
}

#[test]
fn this_arguments_newtarget_and_super_remain_lexical_across_binding_and_jobs() {
    let mut realm = Realm::default();
    realm.eval("let values=[];function Parent(x){this.x=x;this.fn=async()=>[this,arguments[0],new.target]}let p=new Parent(9),fn=p.fn.bind({x:0});fn().then(a=>{if(a[0]!==p||a[1]!==9||a[2]!==Parent)throw 1;values.push('lexical')});class Base{m(){return this.x}}class Derived extends Base{m(){return async()=>super.m()}}let d=new Derived();d.x=7;d.m().call({x:0}).then(v=>{if(v!==7)throw 2;values.push('super')});let async=x=>x,await=8;if(async(await/2)!==4)throw 3").unwrap();
    drain(&mut realm);
    assert_eq!(
        realm.eval("values.join(',')"),
        Ok(Value::String("lexical,super".into()))
    );
}

#[test]
fn captured_objects_and_symbols_survive_collection_and_release_with_the_closure() {
    let mut realm = Realm::default();
    realm.eval("let state=(()=>{let value={},symbol=Symbol();return{fn:async()=>[value,symbol],object:new WeakRef(value),symbol:new WeakRef(symbol)}})()").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("state.object.deref()!==undefined&&state.symbol.deref()!==undefined"),
        Ok(Value::Boolean(true))
    );
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
fn dynamic_source_can_create_async_arrows_and_host_failures_bypass_rejection() {
    let mut realm = Realm::default();
    realm.eval("let AF=(async function(){}).constructor,result;AF('return async x=>x+1')().then(fn=>fn(4)).then(v=>{result=v})").unwrap();
    drain(&mut realm);
    assert_eq!(realm.eval("result"), Ok(Value::Number(5.0)));
    realm
        .eval("let flag=0,f=async()=>new SharedArrayBuffer(0)")
        .unwrap();
    assert!(matches!(
        realm.eval("try{f()}catch(e){flag=1}finally{flag=2}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
