//! Primitive receiver boxing and early errors before Script execution.

use spite_runtime::{Error, Realm, Value};

#[test]
fn bigint_receivers_box_and_run_bodies_after_argument_evaluation() {
    let mut realm = Realm::default();
    realm
        .eval("let flag=0;function f(){flag=9;return this;}")
        .unwrap();
    for source in ["f.call(1n)", "f.apply(1n,{length:0})", "f.bind(1n)()"] {
        realm.eval("flag=0").unwrap();
        assert_eq!(
            realm.eval(&format!("({source}).valueOf()===1n && flag===9")),
            Ok(Value::Boolean(true))
        );
    }
    assert_eq!(
        realm.eval("f.call(1n,flag=3).valueOf()===1n && flag===9"),
        Ok(Value::Boolean(true))
    );
    assert!(matches!(
        realm.eval("flag=1;(function(x,x){'use strict';})"),
        Err(Error::Parse(_))
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(9.0)));
}
