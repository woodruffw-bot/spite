//! Fresh intrinsic RegExp result initialization retains ordinary Array semantics.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn fresh_results_preserve_intrinsic_prototypes_all_descriptors_and_key_order() {
    check(
        r"let calls=0,P=Array.prototype;Object.defineProperty(P,'0',{set(){calls++},configurable:true});Object.defineProperty(P,'1',{set(){calls++},configurable:true});Array=function(){throw 7};let a=/()()a/d.exec('a'),p=Object.getOwnPropertyDescriptor(a,'1'),l=Object.getOwnPropertyDescriptor(a,'length');calls===0&&Object.getPrototypeOf(a)===P&&Object.getPrototypeOf(a.indices)===P&&Object.getPrototypeOf(a.indices[1])===P&&p.value===''&&p.writable&&p.enumerable&&p.configurable&&l.writable&&!l.enumerable&&!l.configurable&&Object.getOwnPropertyNames(a).join(',')==='0,1,2,length,index,input,groups,indices'&&Object.getOwnPropertyNames(a.indices).join(',')==='0,1,2,length,groups'&&a.indices[1]!==a.indices[2]",
    );
}

#[test]
fn initialized_results_remain_mutable_sparse_arrays_with_own_undefined_slots() {
    check(
        r"let a=/(a)|(b)/d.exec('a');a[2]===undefined&&Object.hasOwn(a,'2')&&a.indices[2]===undefined&&Object.hasOwn(a.indices,'2')&&delete a[1]&&a.length===3&&!(1 in a)&&Object.getOwnPropertyDescriptor(a,'2').configurable",
    );
    check(
        r"let a=/(a)(b)/d.exec('ab');a[1]='x';a.length=1;a[0]==='ab'&&a.length===1&&!Object.hasOwn(a,'1')&&!Object.hasOwn(a,'2')&&a.index===0&&a.input==='ab'&&a.indices.length===3",
    );
}

#[test]
fn large_capture_and_indices_results_use_unlimited_defaults_and_survive_collection() {
    let mut realm = Realm::default();
    realm
        .eval("let r=new RegExp('()'.repeat(100000)+'a','dg'),copy=new RegExp(r),a=copy.exec('a')")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("a.length===100001&&a[0]==='a'&&a[1]===''&&a[100000]===''&&a.indices.length===100001&&a.indices[100000][0]===0&&a.indices[100000][1]===0&&a.indices[1]!==a.indices[100000]&&copy.lastIndex===1&&r.lastIndex===0&&copy.source===r.source"),Ok(Value::Boolean(true)));
}

#[test]
fn opted_in_property_limits_abort_after_last_index_without_running_handlers() {
    let mut realm = Realm::new(Limits {
        max_properties: Some(200),
        ..Limits::default()
    });
    realm
        .eval("let r=new RegExp('()'.repeat(5000)+'a','g'),flag=0")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec('a')}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}
