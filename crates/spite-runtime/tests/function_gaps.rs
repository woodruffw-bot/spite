//! Primitive non-strict receivers and construction remain host gaps.

use spite_runtime::{Error, Realm, Value};

#[test]
fn missing_receivers_remain_uncatchable_host_gaps_and_never_execute_the_body() {
    let mut realm = Realm::default();
    realm.eval("let flag=0;function f(){flag=9;}").unwrap();
    for source in [
        "f.call(1)",
        "f.call(false)",
        "f.apply(1n,{length:0})",
        "f.bind('text')()",
    ] {
        assert!(matches!(
            realm.eval(&format!(
                "try{{{source};}}catch{{flag=1;}}finally{{flag=2;}}"
            )),
            Err(Error::Unsupported { .. })
        ));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
    assert!(matches!(
        realm.eval("f.call(1,flag=3)"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(3.0)));
    assert!(matches!(
        realm.eval("flag=1;(function(x,x){'use strict';})"),
        Err(Error::Parse(_))
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(3.0)));
    assert!(matches!(
        realm.eval("new f()"),
        Err(Error::Unsupported { .. })
    ));
}
