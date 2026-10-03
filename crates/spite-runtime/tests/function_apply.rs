//! Function.prototype.apply and ordered, bounded CreateListFromArrayLike.

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn string(source: &str, expected: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::String(expected.into())),
        "{source}"
    );
}

#[test]
fn nullish_lists_are_empty_and_receivers_pass_through_unchanged() {
    for source in [
        "({}).toString.apply()",
        "({}).toString.apply(undefined)",
        "({}).toString.apply(undefined, null)",
        "({}).toString.apply(undefined, undefined)",
    ] {
        string(source, "[object Undefined]");
    }
    string("({}).toString.apply(null, {})", "[object Null]");
    string("({}).toString.apply(1n, {length: 0})", "[object BigInt]");
    assert_eq!(
        Realm::default().eval("let o = {}; ({}).valueOf.apply(o, {}) === o"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn lists_include_inherited_indices_and_undefined_holes() {
    for list in [
        "{0: null, length: 1}",
        "{__proto__: {0: null, length: 1}}",
        "{0: null, length: '1.9'}",
    ] {
        string(
            &format!("({{}}).toString.call.apply(({{}}).toString, {list})"),
            "[object Null]",
        );
    }
    string(
        "({}).toString.call.apply(({}).toString, {length: 1})",
        "[object Undefined]",
    );
    string(
        "({}).toString.call.apply(({}).toString, {0: true, 1: false, length: 2})",
        "[object Boolean]",
    );
    string(
        "({}).toString.apply.call(({}).toString, 'x', {length: 0})",
        "[object String]",
    );
}

#[test]
fn length_uses_tolength_and_does_not_coerce_index_values() {
    for length in [
        "undefined",
        "null",
        "NaN",
        "-Infinity",
        "-1",
        "-0",
        "0.9",
        "''",
        "'invalid'",
        "{}",
    ] {
        string(
            &format!("({{}}).toString.call.apply(({{}}).toString, {{0: null, length: {length}}})"),
            "[object Undefined]",
        );
    }
    for length in ["true", "1.9", "'1'", "{valueOf: ({}).toString.call}"] {
        let source =
            format!("({{}}).toString.call.apply(({{}}).toString, {{0: null, length: {length}}})");
        if length.starts_with('{') {
            assert!(matches!(
                Realm::default().eval(&source),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        } else {
            string(&source, "[object Null]");
        }
    }
    for length in ["1n", "{__proto__: null}"] {
        assert!(matches!(
            Realm::default().eval(&format!(
                "({{}}).toString.apply(null, {{length: {length}}})"
            )),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    string(
        "({}).toString.call.apply(({}).toString, {0: {__proto__: null}, length: 1})",
        "[object Object]",
    );
}

#[test]
fn nonobjects_are_rejected_and_callable_check_precedes_list_inspection() {
    for list in ["0", "1n", "true", "'abc'"] {
        assert!(matches!(
            Realm::default().eval(&format!("({{}}).toString.apply(null, {list})")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    assert!(matches!(
        Realm::default()
            .eval("let o = {apply: ({}).toString.apply}; o.apply(null, {length: Infinity})"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    let mut realm = Realm::default();
    realm
        .eval("let flag = 0; let o = {apply: ({}).toString.apply}")
        .unwrap();
    assert!(matches!(
        realm.eval("o.apply(null, (flag = 1, {}))"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(1.0)));
}

#[test]
fn list_lengths_and_direct_calls_have_explicit_host_argument_limits() {
    for length in ["4", "Infinity", "9007199254740991", "1e100"] {
        let mut realm = Realm::new(Limits {
            max_arguments: 3,
            ..Limits::default()
        });
        realm.eval("let flag = 0").unwrap();
        assert!(matches!(realm.eval(&format!("try {{ ({{}}).toString.apply(null, {{length: {length}}}); }} catch {{ flag = 1; }} finally {{ flag = 2; }}")), Err(Error::Limit { .. })));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
    let mut realm = Realm::new(Limits {
        max_arguments: 1,
        ..Limits::default()
    });
    assert_eq!(
        realm.eval("({}).toString(null)"),
        Ok(Value::String("[object Object]".into()))
    );
    assert!(matches!(
        realm.eval("({}).toString(null, null)"),
        Err(Error::Limit { .. })
    ));
}

#[test]
fn apply_metadata_and_native_source_are_standard() {
    assert_eq!(Realm::default().eval("let a = ({}).toString.apply; a.name === 'apply' && a.length === 2 && typeof a === 'function' && !('prototype' in a)"), Ok(Value::Boolean(true)));
    string(
        "({}).toString.apply.toString()",
        "function apply() { [native code] }",
    );
}
