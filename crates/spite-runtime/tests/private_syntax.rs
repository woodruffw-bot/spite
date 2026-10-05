//! Parsed private syntax must preserve host failures until execution is implemented.

use spite_runtime::{Error, Realm, Value};

#[test]
fn incomplete_private_execution_is_distinct_from_early_errors() {
    for source in [
        "class C{#x;}",
        "class C{#m(){}}",
        "class C{get #x(){}set #x(v){}}",
        "class C{[({m(o){return o.#x;}}).m({})](){}#x;}",
        "class C{[#x in {}](){}#x;}",
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
