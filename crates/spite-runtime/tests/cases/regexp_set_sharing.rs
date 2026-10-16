//! Repeated immutable set atoms retain independent captures and bounded preparation.

use spite_runtime::{Limits, Realm, Value};

#[test]
fn repeated_set_patterns_fit_opted_in_work_and_execute_after_collection() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(100_000),
        ..Limits::default()
    });
    realm
        .eval(&format!(
            "let r=/{}([a])([a])/dy,s='a'.repeat(1002)",
            "[a]".repeat(1000)
        ))
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=r.exec(s);m[0]===s&&m[1]==='a'&&m[2]==='a'&&m.indices[1][0]===1000&&m.indices[1][1]===1001&&m.indices[2][0]===1001&&m.indices[2][1]===1002&&r.lastIndex===1002"), Ok(Value::Boolean(true)));
}
