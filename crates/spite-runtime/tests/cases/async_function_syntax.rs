//! Parsed async definitions remain host-unsupported until object/call support.

use spite_runtime::{Error, Realm, Value};

#[test]
fn async_objects_are_never_allocated_as_ordinary_functions() {
    for source in [
        "async function f(){}",
        "(async function(){})",
        "{async function f(){return 1}}",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    let mut realm = Realm::default();
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval("try{let f=async function(){}}catch(e){flag=1}finally{flag=2}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}

#[test]
fn dynamic_ordinary_functions_only_abort_when_nested_async_definitions_are_instantiated() {
    let mut realm = Realm::default();
    assert_eq!(
        realm.eval("let f=Function('return async function(){}');typeof f"),
        Ok(Value::String("function".into()))
    );
    assert!(matches!(realm.eval("f()"), Err(Error::Unsupported { .. })));
}
