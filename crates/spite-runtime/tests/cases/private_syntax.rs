//! Generator and async-generator private method objects share class early errors.

use spite_runtime::{Realm, Value};

#[test]
fn private_generator_function_kinds_instantiate_and_keep_early_errors() {
    for source in [
        "eval('class C{async *#m(){}}');",
        "eval('class C{*#m(){}}');",
        "eval('class C{static async *#m(){}}');",
    ] {
        let mut realm = Realm::default();
        realm.eval("let flag=0").unwrap();
        realm
            .eval(&format!("try{{{source}}}catch{{flag=1}}finally{{flag=2}}"))
            .unwrap();
        assert_eq!(realm.eval("flag"), Ok(Value::Number(2.0)), "{source}");
        assert_eq!(realm.eval("after=3"), Ok(Value::Number(3.0)));
    }
    assert_eq!(
        Realm::default().eval("let caught=false;try{eval('class C{#constructor;}');}catch(e){caught=e instanceof SyntaxError;}caught"),
        Ok(Value::Boolean(true)),
    );
}
