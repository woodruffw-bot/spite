//! Direct iteration followed by intrinsic list materialization (27.1.3.3.12).

mod common;
use common::REALM_ENTRIES;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn built_in_iterators_wrappers_and_helpers_produce_fresh_intrinsic_dense_arrays() {
    check(
        "let a=[1,2].values().toArray(),b=Iterator.concat([3],[],[4]).toArray(),c='a\u{1f600}'[Symbol.iterator]().toArray();Array.isArray(a) && Object.getPrototypeOf(a)===Array.prototype && a.join(',')==='1,2' && b.join(',')==='3,4' && c.length===2 && c[1]==='\u{1f600}' && (function(){return arguments[Symbol.iterator]().toArray().join(',')==='5,6';})(5,6)",
    );
    check(
        "let n=0,w=Iterator.from({next:()=>({value:n++,done:n>3})}),a=w.toArray(),b=w.toArray(),c=w.toArray();a.join(',')==='0,1,2' && b.length===0 && c.length===0 && b!==c",
    );
    check(
        "let value={},i=0,w=Iterator.from({next:()=>({value:i++===0?value:undefined,done:i>2})}),a=w.toArray();a.length===2 && a[0]===value && Object.hasOwn(a,'1') && a[1]===undefined && Reflect.ownKeys(a).join(',')==='0,1,length'",
    );
}

#[test]
fn direct_acquisition_caches_next_preserves_receiver_and_ignores_iterator_hooks() {
    check(
        "let gets=0,calls=0,receiver,argc,i={};Object.defineProperty(i,Symbol.iterator,{get:()=>{throw 7;}});Object.defineProperty(i,'next',{configurable:true,get:()=>{gets++;return function(){'use strict';receiver=this;argc=arguments.length;Object.defineProperty(i,'next',{value:()=>{throw 8;}});calls++;return {done:calls>2,value:calls};};}});let a=Iterator.prototype.toArray.call(i,{valueOf:()=>{throw 9;}});gets===1 && calls===3 && receiver===i && argc===0 && a.join(',')==='1,2'",
    );
    for primitive in ["undefined", "null", "true", "1", "1n", "'x'", "Symbol()"] {
        assert!(matches!(Realm::default().eval(&format!("Object.defineProperty(Number.prototype,'next',{{get:()=>{{throw 7;}}}});Iterator.prototype.toArray.call({primitive})")), Err(Error::Exception { kind: ExceptionKind::TypeError, .. })), "{primitive}");
    }
}

#[test]
fn step_errors_preserve_identity_do_not_close_and_done_skips_value() {
    for next in [
        "()=>{throw 7;}",
        "()=>7",
        "()=>({get done(){throw 7;}})",
        "()=>({done:false,get value(){throw 7;}})",
    ] {
        check(&format!(
            "let closed=0,i={{next:{next},return(){{closed++;throw 8;}}}},caught=false;try{{Iterator.prototype.toArray.call(i);}}catch(e){{caught=e===7 || e instanceof TypeError;}}caught && closed===0"
        ));
    }
    check(
        "let i={next:()=>({done:true,get value(){throw 7;}}),return:()=>{throw 8;}};Iterator.prototype.toArray.call(i).length===0",
    );
    assert_eq!(
        Realm::default().eval(
            "let i={get next(){throw 7;},return(){throw 8;}};Iterator.prototype.toArray.call(i)"
        ),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    check(
        "let calls=0,i={next(){calls++;if(calls===1)throw 7;return {done:true};}},caught=false;try{Iterator.prototype.toArray.call(i);}catch(e){caught=e===7;}caught && Iterator.prototype.toArray.call(i).length===0 && calls===2",
    );
}

#[test]
fn result_materialization_bypasses_species_and_inherited_element_setters() {
    check(
        "let I=Iterator.prototype.toArray,p=Array.prototype;Object.defineProperty(Array,Symbol.species,{get:()=>{throw 7;}});Object.defineProperty(p,'0',{set:()=>{throw 8;},configurable:true});let n=0,i={next:()=>({value:9,done:n++>0})},a=I.call(i),d=Object.getOwnPropertyDescriptor(a,'0');Object.getPrototypeOf(a)===p && a.length===1 && d.value===9 && d.writable && d.enumerable && d.configurable",
    );
    check(
        "let I=Iterator.prototype.toArray,p=Array.prototype;globalThis.Array=function(){throw 7;};let n=0,a=I.call({next:()=>({value:8,done:n++>0})});Object.getPrototypeOf(a)===p && a[0]===8",
    );
}

#[test]
fn metadata_constructor_rejection_and_intrinsic_retention_are_standard() {
    check(
        "let f=Iterator.prototype.toArray,d=Object.getOwnPropertyDescriptor(Iterator.prototype,'toArray');f.name==='toArray' && f.length===0 && Object.getPrototypeOf(f)===Function.prototype && !Object.hasOwn(f,'prototype') && d.value===f && d.writable && !d.enumerable && d.configurable",
    );
    assert!(matches!(
        Realm::default().eval("new Iterator.prototype.toArray()"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    let mut realm = Realm::default();
    realm
        .eval("let f=Iterator.prototype.toArray;delete Iterator.prototype.toArray;")
        .unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(
        realm.eval("f.call([7].values())[0]"),
        Ok(Value::Number(7.0))
    );
}

#[test]
fn large_default_inputs_work_and_opted_in_failures_skip_cleanup_and_handlers() {
    check(
        "let n=0,a=Iterator.prototype.toArray.call({next:()=>({done:n===10000,value:n++})});a.length===10000 && a[0]===0 && a[9999]===9999",
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm
        .eval("let flag=0,i={next:()=>({done:false,value:7}),return:()=>{flag=3;return {};}}")
        .unwrap();
    assert!(matches!(
        realm.eval("try{Iterator.prototype.toArray.call(i);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::default();
    realm
        .eval("let flag=0,i={next:()=>Function('class C{}'),return:()=>{flag=3;return {};}}")
        .unwrap();
    assert!(matches!(
        realm.eval("try{Iterator.prototype.toArray.call(i);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
