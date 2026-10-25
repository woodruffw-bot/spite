//! SuppressedError's uncoerced values and ordered initialization (20.5.8).

use spite_runtime::{Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn values_are_retained_without_hooks_and_options_are_ignored() {
    check(
        "let value=new Proxy({},{get(){throw 1}}),options=new Proxy({},{get(){throw 2},has(){throw 3}}),e=SuppressedError(value,value,'detail',options);e.error===value&&e.suppressed===value&&e.message==='detail'&&!Object.hasOwn(e,'cause')&&Reflect.ownKeys(e).join(',')==='message,error,suppressed'",
    );
    check(
        "let e=SuppressedError();Object.hasOwn(e,'error')&&Object.hasOwn(e,'suppressed')&&e.error===undefined&&e.suppressed===undefined&&!Object.hasOwn(e,'message')&&Reflect.ownKeys(e).join(',')==='error,suppressed'",
    );
    check(
        "for(const v of [undefined,null,false,0,-0,NaN,1n,Symbol(),'text',{}]){let e=SuppressedError(v,v);if(!Object.is(e.error,v)||!Object.is(e.suppressed,v))throw 1;}true",
    );
}

#[test]
fn message_conversion_follows_prototype_lookup_and_preserves_abrupt_completions() {
    check(
        "let log='',proto={},N=new Proxy(function(){},{get(t,p){if(p==='prototype'){log+='p';return proto}return t[p]}}),message={toString(){log+='m';return 'detail'}},e=Reflect.construct(SuppressedError,[1,2,message],N);log==='pm'&&Object.getPrototypeOf(e)===proto&&Error.isError(e)&&e.message==='detail'&&e.error===1&&e.suppressed===2",
    );
    check(
        "let marker={},calls=0,N=new Proxy(function(){},{get(){throw marker}}),message={toString(){calls++;throw 7}},caught=false;try{Reflect.construct(SuppressedError,[1,2,message],N)}catch(e){caught=e===marker}caught&&calls===0",
    );
    check(
        "let marker={},caught=false;try{SuppressedError(1,2,{toString(){throw marker}})}catch(e){caught=e===marker}caught",
    );
}

#[test]
fn instance_fields_bypass_inherited_setters_and_have_standard_descriptors() {
    check(
        "let p=SuppressedError.prototype;for(const key of ['message','error','suppressed'])Object.defineProperty(p,key,{set(){throw 1},configurable:true});let e=SuppressedError(1,2,'detail');for(const key of ['message','error','suppressed']){let d=Object.getOwnPropertyDescriptor(e,key);if(!d.writable||d.enumerable||!d.configurable)throw 2;}e.message==='detail'&&e.error===1&&e.suppressed===2&&Object.keys(e).length===0&&delete e.error&&delete e.suppressed&&!Object.hasOwn(e,'error')&&!Object.hasOwn(e,'suppressed')",
    );
}

#[test]
fn calls_bound_construction_and_subclasses_preserve_the_error_brand() {
    check(
        "let F=SuppressedError.bind({},1,2,'bound'),e=new F,f=SuppressedError.call(null,3,4,'call');e instanceof SuppressedError&&e instanceof Error&&Error.isError(e)&&e.toString()==='SuppressedError: bound'&&e.error===1&&e.suppressed===2&&f.error===3&&f.suppressed===4&&f.message==='call'",
    );
    check(
        "class S extends SuppressedError{}let e=new S(1,2,'sub');Object.getPrototypeOf(e)===S.prototype&&e instanceof S&&Error.isError(e)&&e.error===1&&e.suppressed===2&&e.toString()==='SuppressedError: sub'&&!Error.isError(SuppressedError.prototype)&&Object.prototype.toString.call(SuppressedError.prototype)==='[object Object]'&&Object.prototype.toString.call(e)==='[object Error]'",
    );
}

#[test]
fn retained_intrinsics_and_both_error_values_survive_collection() {
    let mut realm = Realm::default();
    realm.eval("let Saved=SuppressedError,e=Saved({message:'new'},{message:'old'});delete globalThis.SuppressedError;delete globalThis.Error;").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("Saved.isError(e)&&e.error.message==='new'&&e.suppressed.message==='old'&&Saved(1,2).suppressed===2"), Ok(Value::Boolean(true)));
}
