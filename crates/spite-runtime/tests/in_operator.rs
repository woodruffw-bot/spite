//! Property presence, prototype traversal, and in-operator exception order.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

#[test]
fn in_distinguishes_absence_from_undefined_and_includes_inherited_properties() {
    for (source, expected) in [
        ("'x' in {}", false),
        ("'x' in {x: undefined}", true),
        ("'x' in {__proto__: {x: 1}}", true),
        ("let o = {x: 1}; delete o.x; 'x' in o", false),
        (
            "let o = {__proto__: {x: 1}, x: 2}; delete o.x; 'x' in o",
            true,
        ),
        ("'toString' in {}", true),
        ("'constructor' in {}", true),
        ("'toString' in {__proto__: null}", false),
        ("'__proto__' in {}", false),
        ("'__proto__' in {['__proto__']: 1}", true),
        ("'hasOwnProperty' in {__proto__: {}}", true),
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Ok(Value::Boolean(expected)),
            "{source}"
        );
    }
}

#[test]
fn primitive_keys_convert_and_in_retains_relational_precedence() {
    for source in [
        "16n in {16: 0}",
        "null in {null: 1}",
        "undefined in {undefined: 1}",
        "true in {true: 1}",
        "-0 in {0: 1}",
        "'x' in {x: 1} === true",
        "'x' in {x: 1} in {true: 1}",
        r"'\ud800' in {'\ud800': 1}",
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Ok(Value::Boolean(true)),
            "{source}"
        );
    }
}

#[test]
fn nonobject_rhs_throws_before_key_conversion_after_both_operands_evaluate() {
    for right in ["null", "undefined", "true", "0", "1n", "'abc'"] {
        for left in ["'x'", "({})"] {
            assert!(
                matches!(
                    Realm::default().eval(&format!("{left} in {right}")),
                    Err(Error::Exception {
                        kind: ExceptionKind::TypeError,
                        ..
                    })
                ),
                "{left} in {right}"
            );
        }
    }
    let mut realm = Realm::default();
    realm.eval("let flag = 0").unwrap();
    assert!(matches!(
        realm.eval("(flag = 1, {}) in (flag = 2, 0)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(2.0)));
    assert!(matches!(
        realm.eval("missing in (flag = 3, {})"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(2.0)));
    assert!(matches!(
        realm.eval("({}) in (flag = 4, {})"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(4.0)));
}

#[test]
fn for_header_nested_in_expressions_evaluate_normally() {
    for source in [
        "let n = 0; for (let present = ('x' in {x: 1}); present; present = false) n++; n",
        "let n = 0; for (let o = {x: 'x' in {x: 1}}; o.x; o.x = false) n++; n",
        "let n = 0; for (let present = true ? 'x' in {x: 1} : false; present; present = false) n++; n",
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Ok(Value::Number(1.0)),
            "{source}"
        );
    }
}
