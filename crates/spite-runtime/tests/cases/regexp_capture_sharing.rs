//! Equal capture ranges share immutable strings and retain logical output limits.

use spite_runtime::{Realm, Value};

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
