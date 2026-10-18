//! Loop scope restoration under an explicitly enabled work quota.

use crate::{Error, Realm, Value};

#[test]
fn infinite_loops_exhaust_the_host_budget_and_restore_scopes() {
    for source in [
        // The original while/do quota cases.
        "while (true) ;",
        "do ; while (true)",
        "while (true) { let x = 2; }",
        "do { let x = 2; } while (true)",
        // The original for-loop quota cases.
        "for (;;) ;",
        "for (;;) continue;",
        "for (;;) { let x = 2; }",
        "a: for (;;) { let x = 2; continue a; }",
        // The original continue/restoration quota cases.
        "while (true) { let x = 2; continue; }",
        "do { let x = 2; continue; } while (true)",
    ] {
        // Global declaration work grows with the intrinsic inventory. Prepare
        // the fixture before enabling the original loop-execution quota.
        let mut realm = Realm::default();
        realm.eval("let x = 1").unwrap();
        realm.limits.max_steps = Some(100);
        assert!(
            matches!(realm.eval(source), Err(Error::Limit { .. })),
            "{source}"
        );
        assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
    }

    // Preserve the original labeled-loop abort and subsequent throw checks in
    // the same realm, so both host and language failures must unwind scopes.
    let mut realm = Realm::default();
    realm.eval("let x = 1").unwrap();
    realm.limits.max_steps = Some(100);
    assert!(matches!(
        realm.eval("a: while (true) { let x = 2; while (true) continue a; }"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
    assert_eq!(
        realm.eval("a: { let x = 2; throw x; }"),
        Err(Error::Thrown(Value::Number(2.0)))
    );
    assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
}
