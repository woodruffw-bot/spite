//! Parsed generators retain an explicit runtime boundary before body execution.

use spite_runtime::{Error, Realm, Value};

#[test]
fn generator_object_creation_is_a_host_boundary_before_any_generator_body_effects() {
    let mut realm = Realm::default();
    assert_eq!(
        realm.eval("typeof Function('function* gap(){}')==='function'"),
        Ok(Value::Boolean(true))
    );
    for (source, before) in [
        ("before++;function*g(){operand++;yield 1}", 0.0),
        (
            "before++;try{(function*(){operand++;yield 1})}catch{handled=1}finally{handled=2}",
            1.0,
        ),
    ] {
        let mut realm = Realm::default();
        realm.eval("let before=0,operand=0,handled=0").unwrap();
        assert!(
            matches!(realm.eval(source), Err(Error::Unsupported { .. })),
            "{source}"
        );
        assert_eq!(realm.eval("before"), Ok(Value::Number(before)));
        assert_eq!(
            realm.eval("operand===0&&handled===0"),
            Ok(Value::Boolean(true))
        );
    }
}

#[test]
fn generator_syntax_errors_are_language_errors_before_script_effects() {
    for source in [
        "before++;function*g(x=yield 1){}",
        "before++;function*g(){let yield}",
    ] {
        let mut realm = Realm::default();
        realm.eval("let before=0").unwrap();
        assert!(
            matches!(realm.eval(source),Err(Error::Parse(error)) if error.kind==spite_core::DiagnosticKind::Syntax),
            "{source}"
        );
        assert_eq!(realm.eval("before"), Ok(Value::Number(0.0)));
    }
}
