//! Parsed ordinary functions retain an explicit runtime gap until instantiation.

use spite_runtime::{Error, Realm, Value};

#[test]
fn function_instantiation_is_an_uncatchable_host_gap_without_body_effects() {
    let mut realm = Realm::default();
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval("try{(function(){flag=1;});}catch{flag=2;}finally{flag=3;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert!(matches!(
        realm.eval("flag=1;(function(x,x){'use strict';})"),
        Err(Error::Parse(_))
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(
        realm.eval("if(false){(function(){return 1;});} 7"),
        Ok(Value::Number(7.0))
    );
}

#[test]
fn declarations_report_the_gap_during_scope_instantiation() {
    let mut realm = Realm::default();
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval("flag=1;function f(){}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert!(matches!(
        realm.eval("try{flag=1;{flag=2;function f(){}}}finally{flag=3;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(1.0)));
    assert!(matches!(
        realm.eval("(()=>{flag=2;function f(){}})()"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(1.0)));
    assert_eq!(
        realm.eval("if(false){function f(){}} 7"),
        Ok(Value::Number(7.0))
    );
}
