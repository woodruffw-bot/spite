//! Function.prototype call and the host's native source representation.

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn string(source: &str, expected: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::String(expected.into())),
        "{source}"
    );
}

#[test]
fn call_passes_receivers_without_boxing_or_global_substitution() {
    for (receiver, tag) in [
        ("undefined", "Undefined"),
        ("null", "Null"),
        ("true", "Boolean"),
        ("1", "Number"),
        ("1n", "BigInt"),
        ("'abc'", "String"),
        ("{}", "Object"),
        ("({}).toString", "Function"),
    ] {
        string(
            &format!("({{}}).toString.call({receiver})"),
            &format!("[object {tag}]"),
        );
    }
    string("({}).toString.call()", "[object Undefined]");
    string(
        "({}).toString.call.call(({}).toString, null)",
        "[object Null]",
    );
    assert_eq!(
        Realm::default().eval("let o = {}; ({}).valueOf.call(o) === o"),
        Ok(Value::Boolean(true))
    );
    for receiver in ["null", "undefined"] {
        assert!(matches!(
            Realm::default().eval(&format!("({{}}).valueOf.call({receiver})")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    // Observable primitive wrapper allocation is still a separate roadmap step.
    assert!(matches!(
        Realm::default().eval("({}).valueOf.call(1)"),
        Err(Error::Unsupported { .. })
    ));
}

#[test]
fn borrowed_call_checks_callability_after_arguments_and_preserves_callee_identity() {
    let mut realm = Realm::default();
    realm
        .eval("let flag = 0; let o = {call: ({}).toString.call}")
        .unwrap();
    assert!(matches!(
        realm.eval("o.call(null, flag = 1)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(1.0)));
    string(
        "let f = ({}).toString; f.call(null, f = 1)",
        "[object Null]",
    );
    string("let o = {}; (o.toString.call)(o)", "[object Object]");
    assert_eq!(
        Realm::default().eval("let f = ({}).valueOf; let o = {}; f.call(o, f = null) === o"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn native_source_uses_initial_name_and_callable_receiver() {
    for (source, expected) in [
        (
            "({}).toString.toString()",
            "function toString() { [native code] }",
        ),
        (
            "({}).valueOf.toString()",
            "function valueOf() { [native code] }",
        ),
        (
            "({}).toString.call.toString()",
            "function call() { [native code] }",
        ),
        (
            "let f = ({}).valueOf; delete f.name; f.toString()",
            "function valueOf() { [native code] }",
        ),
        (
            "({}).toString.toString.call(({}).valueOf)",
            "function valueOf() { [native code] }",
        ),
        ("'' + ({}).valueOf", "function valueOf() { [native code] }"),
    ] {
        string(source, expected);
    }
    for receiver in ["null", "undefined", "0", "1n", "''", "true", "{}"] {
        assert!(matches!(
            Realm::default().eval(&format!("({{}}).toString.toString.call({receiver})")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    assert!(matches!(
        Realm::default().eval("let source = ({}).toString.toString; source()"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn methods_have_standard_metadata_and_share_intrinsic_identity() {
    for source in [
        "({}).toString.call.name === 'call' && ({}).toString.call.length === 1",
        "({}).toString.toString.name === 'toString' && ({}).toString.toString.length === 0",
        "({}).toString.call === ({}).valueOf.call",
        "({}).toString.toString === ({}).valueOf.toString",
        "typeof ({}).toString.call === 'function'",
        "!('prototype' in ({}).toString.call)",
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Ok(Value::Boolean(true)),
            "{source}"
        );
    }
}

#[test]
fn generated_builtin_strings_obey_host_limits_even_during_coercion() {
    for source in [
        "({}).toString()",
        "+{}",
        "({}).valueOf.toString()",
        "+({}).toString",
    ] {
        let mut realm = Realm::new(Limits {
            max_string_units: 10,
            ..Limits::default()
        });
        realm.eval("let flag = 0").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "try {{ {source}; }} catch {{ flag = 1; }} finally {{ flag = 2; }}"
                )),
                Err(Error::Limit { .. })
            ),
            "{source}"
        );
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
}
