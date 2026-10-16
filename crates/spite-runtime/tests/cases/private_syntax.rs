//! Unimplemented async/generator private methods remain host failures.

use spite_runtime::{Error, Realm, Value};

#[test]
fn unsupported_private_function_kinds_are_distinct_from_early_errors() {
    for source in [
        "eval('class C{async #m(){}}');",
        "eval('class C{*#m(){}}');",
        "eval('class C{static async #m(){}}');",
    ] {
        let mut realm = Realm::default();
        realm.eval("let flag=0;").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "try{{{source}}}catch{{flag=1;}}finally{{flag=2;}}"
                )),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
        assert_eq!(realm.eval("after=3"), Ok(Value::Number(3.0)));
    }
    assert_eq!(
        Realm::default().eval("let caught=false;try{eval('class C{#constructor;}');}catch(e){caught=e instanceof SyntaxError;}caught"),
        Ok(Value::Boolean(true)),
    );
}
