//! ArraySetLength preserves the descending first-failure rule (10.4.2.4).
use spite_runtime::{Limits, Realm, Value};
use std::fmt::Write;

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn array_linear_truncation_first_failure_snapshot() {
    let mut rows = String::new();
    for reversed in [false, true] {
        for mask in 0..32 {
            for length in [0, 3, 7] {
                for readonly in [false, true] {
                    let source = format!(
                        "let a=[],s=Symbol('marker'),indices=[0,1,2,5,9];a.note=7;a[s]=8;if({reversed})indices.reverse();for(let index of indices)Object.defineProperty(a,String(index),{{value:index,writable:true,enumerable:true,configurable:({mask}&(1<<[0,1,2,5,9].indexOf(index)))===0}});let ok=Reflect.defineProperty(a,'length',{{value:{length},writable:!{readonly}}});JSON.stringify([ok,a.length,Object.getOwnPropertyDescriptor(a,'length').writable,Reflect.ownKeys(a).map(k=>typeof k==='symbol'?'symbol':k),[0,1,2,5,9].map(i=>Object.hasOwn(a,String(i))),a.note,a[s]])"
                    );
                    let Value::String(result) = Realm::default().eval(&source).unwrap() else {
                        panic!("{source}");
                    };
                    writeln!(rows, "reverse={reversed} mask={mask} length={length} readonly={readonly}: {result:?}").unwrap();
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn array_linear_truncation_accessors_prototypes_and_maximum_index() {
    check(
        "let calls=0,a=[0,1,2,3];Object.defineProperty(a,'1',{get(){calls++;return 9},set(v){calls++},configurable:false});Object.defineProperty(a,'3',{get(){calls++;return 8},configurable:true});let ok=Reflect.defineProperty(a,'length',{value:0,writable:false});!ok&&calls===0&&a.length===2&&!Object.hasOwn(a,'2')&&!Object.hasOwn(a,'3')&&!Object.getOwnPropertyDescriptor(a,'length').writable&&Object.hasOwn(a,'0')&&Object.hasOwn(a,'1')",
    );
    check(
        "let p=Object.create(Array.prototype);Object.defineProperty(p,'9',{value:9,configurable:false});let a=[0,1,2];Object.setPrototypeOf(a,p);Reflect.defineProperty(a,'length',{value:0})&&a.length===0&&a[9]===9&&Reflect.ownKeys(a).join(',')==='length'",
    );
    check(
        "let a=[];a.note=1;a[4294967294]=4;a[4294967295]=5;a['01']=6;Object.defineProperty(a,'4294967293',{value:3,configurable:false});!Reflect.defineProperty(a,'length',{value:0,writable:false})&&a.length===4294967294&&!Object.hasOwn(a,'4294967294')&&a[4294967293]===3&&a[4294967295]===5&&a['01']===6&&a.note===1&&!Object.getOwnPropertyDescriptor(a,'length').writable",
    );
    check(
        "let a=[0,1,2];!Reflect.defineProperty(a,'length',{value:0,configurable:true})&&a.length===3&&a[2]===2&&Object.getOwnPropertyDescriptor(a,'length').writable",
    );
    check(
        "let a=[0,1,2];Object.defineProperty(a,'length',{writable:false});!Reflect.defineProperty(a,'length',{value:0})&&a.length===3&&a[2]===2",
    );
}

#[test]
fn array_linear_truncation_large_arrays_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    assert_eq!(realm.eval("let a=[];for(let i=0;i<100000;i++)a[i]=i;a.note=1;a.length=0;a.length===0&&Reflect.ownKeys(a).join(',')==='length,note'&&a.note===1"), Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("a[99999]===undefined&&a.length===0&&a.note===1"),
        Ok(Value::Boolean(true))
    );
}
