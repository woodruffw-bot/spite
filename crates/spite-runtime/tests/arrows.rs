//! Arrow execution remains explicit until closure instantiation is integrated.

use spite_runtime::{Error, Realm, Value};

#[test]
fn arrow_execution_is_a_host_gap_and_early_errors_precede_all_effects() {
    let mut realm = Realm::default();
    realm.eval("let flag = 0").unwrap();
    assert!(matches!(
        realm.eval("try { x => (flag = 1); } catch { flag = 2; } finally { flag = 3; }"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert!(matches!(
        realm.eval("flag = 1; (x, x) => x"),
        Err(Error::Parse(_))
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
