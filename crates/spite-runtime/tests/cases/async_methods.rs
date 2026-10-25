//! Async method metadata, receivers, home objects, private brands and collection.

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
fn object_and_class_methods_have_async_metadata_standard_descriptors_and_no_construct() {
    assert_eq!(Realm::default().eval("let key=Symbol('k'),o={async m(a,b=1){return a},async[key](){return 1}},AF=(async function(){}).constructor;class C{async m(){return 1}static async n(){return 2}}let caught=0;for(const f of[o.m,o[key],C.prototype.m,C.n]){try{new f()}catch(e){if(e instanceof TypeError)caught++}}let a=Object.getOwnPropertyDescriptor(o,'m'),b=Object.getOwnPropertyDescriptor(C.prototype,'m');caught===4&&a.writable&&a.enumerable&&a.configurable&&b.writable&&!b.enumerable&&b.configurable&&o.m.name==='m'&&o.m.length===1&&o[key].name==='[k]'&&Object.getPrototypeOf(o.m)===AF.prototype&&!Object.hasOwn(o.m,'prototype')&&o.m.toString()==='async m(a,b=1){return a}'"),Ok(Value::Boolean(true)));
}

#[test]
fn receivers_arguments_and_parameter_body_errors_follow_async_call_semantics() {
    let mut realm = Realm::default();
    realm.eval("let marker={},values=[],o={async m(x){return[this,x,arguments.length,new.target]},async bad({x}){return x},async thrown(){throw marker}};o.m(7).then(a=>{if(a[0]!==o||a[1]!==7||a[2]!==1||a[3]!==undefined)throw 1;values.push('object')});o.bad(null).catch(e=>{if(!(e instanceof TypeError))throw 2;values.push('param')});o.thrown().catch(e=>{if(e!==marker)throw 3;values.push('throw')});class C{async m(){return this}}C.prototype.m.call(9).then(v=>{if(v!==9)throw 4;values.push('strict')})").unwrap();
    drain(&mut realm);
    assert_eq!(
        realm.eval("values.join(',')"),
        Ok(Value::String("object,param,throw,strict".into()))
    );
}

#[test]
fn super_in_parameters_and_bodies_uses_live_home_objects_and_original_receivers() {
    let mut realm = Realm::default();
    realm.eval("let values=[],base={get x(){return this.value}},o={__proto__:base,value:7,async m(x=super.x){return x}};o.m().then(v=>{values.push(v)});o.m.call({value:9}).then(v=>{values.push(v)});class Base{m(){return this.value}}class Derived extends Base{async m(){return super.m()+1}static async tag(){return super.name}}let d=new Derived();d.value=4;d.m().then(v=>{values.push(v)});Derived.tag().then(v=>{values.push(v)});Object.setPrototypeOf(o,{get x(){return this.value+10}});o.m().then(v=>{values.push(v)})").unwrap();
    drain(&mut realm);
    assert_eq!(
        realm.eval("values.join(',')"),
        Ok(Value::String("7,9,5,Base,17".into()))
    );
}

#[test]
fn instance_and_static_private_async_methods_enforce_brands_through_rejections() {
    let mut realm = Realm::default();
    realm.eval("let values=[];class C{#x=7;async #m(v){return this.#x+v}get(){return this.#m}static #y=9;static async #n(){return this.#y}static get(){return this.#n}}let c=new C(),m=c.get(),s=C.get();m.call(c,1).then(v=>{values.push(v)});m.call({}).catch(e=>{if(!(e instanceof TypeError))throw 1;values.push('brand')});s.call(C).then(v=>{values.push(v)});class D extends C{}s.call(D).catch(e=>{if(!(e instanceof TypeError))throw 2;values.push('static')})").unwrap();
    drain(&mut realm);
    assert_eq!(
        realm.eval("values.join(',')"),
        Ok(Value::String("8,brand,9,static".into()))
    );
}

#[test]
fn async_method_captures_and_home_objects_survive_collection_and_release() {
    let mut realm = Realm::default();
    realm.eval("let state=(()=>{let value={},symbol=Symbol(),home={__proto__:{get read(){return value}},async m(){return[super.read,symbol]}};return{method:home.m,object:new WeakRef(value),symbol:new WeakRef(symbol)}})()").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("state.object.deref()!==undefined&&state.symbol.deref()!==undefined"),
        Ok(Value::Boolean(true))
    );
    realm
        .eval("state.promise=state.method();state.method=null")
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
fn dynamic_source_supports_async_methods_and_host_failures_skip_handlers() {
    let mut realm = Realm::default();
    realm.eval("let AF=(async function(){}).constructor,result;AF('return {async m(){return 5}}')().then(o=>o.m()).then(v=>{result=v})").unwrap();
    drain(&mut realm);
    assert_eq!(realm.eval("result"), Ok(Value::Number(5.0)));
    realm
        .eval("let flag=0,o={async m(){return new SharedArrayBuffer(0)}}")
        .unwrap();
    assert!(matches!(
        realm.eval("try{o.m()}catch(e){flag=1}finally{flag=2}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
