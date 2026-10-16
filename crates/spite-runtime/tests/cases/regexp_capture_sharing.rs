//! Equal capture ranges share immutable strings and retain logical output limits.

use spite_runtime::{Error, Limits, Realm, Value};

#[test]
fn many_enclosing_long_captures_use_unlimited_defaults_and_survive_collection() {
    let mut realm = Realm::default();
    realm.eval("let s='a'.repeat(200000)+'b',r=new RegExp('^'+'('.repeat(100000)+'a+b'+')'.repeat(100000)+'$','dg'),copy=new RegExp(r),a=copy.exec(s)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("a.length===100001&&a[0]===s&&a[1]===s&&a[100000]===s&&a.indices.length===100001&&a.indices[1][0]===0&&a.indices[100000][1]===200001&&a.indices[1]!==a.indices[100000]&&copy.lastIndex===200001&&r.lastIndex===0&&copy.source===r.source"),Ok(Value::Boolean(true)));
}

#[test]
fn shared_ranges_preserve_utf16_values_descriptors_undefined_and_distinct_index_arrays() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r"let a=/((\uD800))()((ab))|(z)/d.exec('\uD800ab'),p=Object.getOwnPropertyDescriptor(a,'2');a[0]==='\uD800ab'&&a[1]==='\uD800'&&a[2]==='\uD800'&&a[3]===''&&a[4]==='ab'&&a[5]==='ab'&&a[6]===undefined&&p.writable&&p.enumerable&&p.configurable&&a.indices[1]!==a.indices[2]&&a.indices[4]!==a.indices[5]&&a.indices[6]===undefined&&Object.hasOwn(a.indices,'6')"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("a[1]='changed';a[2]==='\\uD800'&&a[4]==='ab'&&a[5]==='ab'&&a[0]==='\\uD800ab'"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn optional_logical_capture_work_still_aborts_after_last_index_without_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(100000),
        ..Limits::default()
    });
    realm.eval("let r=new RegExp('('.repeat(1000)+'a+'+')'.repeat(1000),'g'),s='a'.repeat(1000),flag=0").unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(s)}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0&&r.lastIndex===1000"),
        Ok(Value::Boolean(true))
    );
}
