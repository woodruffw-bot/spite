//! Realm-level OrdinaryToPrimitive dispatch and conversion ordering.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn type_error(source: &str) {
    assert!(
        matches!(
            Realm::default().eval(source),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ),
        "{source}"
    );
}

#[test]
fn objects_without_callable_conversion_methods_produce_typeerror() {
    for object in [
        "({__proto__: null})",
        "({toString: 1, valueOf: 2})",
        "({toString: null, valueOf: undefined})",
        "({toString: {}, valueOf: {}})",
        "({__proto__: {toString: 1, valueOf: 2}})",
    ] {
        for expression in [
            format!("+{object}"),
            format!("-{object}"),
            format!("~{object}"),
            format!("{object} + 1"),
            format!("1 + {object}"),
            format!("'' + {object}"),
            format!("{object} - 1n"),
            format!("1n * {object}"),
            format!("{object} >>> 0"),
            format!("{object} ** 0"),
            format!("`${{{object}}}`"),
            format!("{object} == 1"),
            format!("true == {object}"),
            format!("{object} != 'x'"),
            format!("{object} < 1"),
            format!("1 >= {object}"),
            format!("{object} in {{}}"),
            format!("({{[{object}]: 1}})"),
            format!("({{}})[{object}]"),
            format!("delete ({{}})[{object}]"),
        ] {
            type_error(&expression);
        }
    }
}

#[test]
fn truthiness_identity_and_nullish_equality_never_request_conversion() {
    for source in [
        "let o = {__proto__: null}; o === o",
        "let o = {__proto__: null}; o == o",
        "let o = {__proto__: null}; o != null && o != undefined",
        "let o = {__proto__: null}; !!o",
        "let o = {__proto__: null}; (o || missing) === o",
        "let o = {__proto__: null}; (o ?? missing) === o",
        "let o = {__proto__: null}; (o && 1) === 1",
        "let o = {__proto__: null}; typeof o === 'object'",
        "let o = {__proto__: null}; delete (0, o)",
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Ok(Value::Boolean(true)),
            "{source}"
        );
    }
}

#[test]
fn conversions_run_left_to_right_after_both_operand_expressions() {
    for operator in ["+", "-", "*", "/", "<", ">", "<=", ">="] {
        let mut realm = Realm::default();
        realm.eval("let flag = 0").unwrap();
        let source =
            format!("(flag = 1, {{__proto__: null}}) {operator} (flag = 2, ({{}}).toString)");
        assert!(
            matches!(
                realm.eval(&source),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{operator}"
        );
        assert_eq!(realm.eval("flag"), Ok(Value::Number(2.0)));
        let source = format!("({{}}).toString {operator} ({{__proto__: null}})");
        assert!(
            matches!(realm.eval(&source), Err(Error::Unsupported { .. })),
            "{operator}"
        );
    }
}

#[test]
fn deferred_reference_conversion_observes_rhs_mutation_of_the_key_object() {
    let mut realm = Realm::default();
    realm
        .eval("let o = {}; let key = {}; let flag = 0")
        .unwrap();
    assert!(matches!(
        realm.eval("o[key] = (key.toString = 1, key.valueOf = 2, flag = 1)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(1.0)));
    realm.eval("key = {}; flag = 0").unwrap();
    assert!(matches!(
        realm.eval("o[key] += (key.toString = 1, key.valueOf = 2, flag = 1)"),
        Ok(Value::Number(n)) if n.is_nan()
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(1.0)));
    assert!(matches!(realm.eval("o['[object Object]']"), Ok(Value::Number(n)) if n.is_nan()));
}

#[test]
fn failed_conversion_is_catchable_and_prevents_later_values_or_writes() {
    for source in [
        "let flag = 0; try { ({[{__proto__: null}]: flag = 1}); } catch { flag = 2; } flag",
        "let flag = 0; try { `${{__proto__: null}}${flag = 1}`; } catch { flag = 2; } flag",
        "let o = {x: {__proto__: null}}; let old = o.x; try { o.x++; } catch {} o.x === old ? 2 : 0",
        "let o = {x: {__proto__: null}}; let old = o.x; try { o.x += 1; } catch {} o.x === old ? 2 : 0",
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Ok(Value::Number(2.0)),
            "{source}"
        );
    }
}

#[test]
fn missing_callable_intrinsics_remain_unsupported_and_string_hook_names_are_ordinary() {
    for source in ["+({}).toString", "`${({}).toString}`"] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    assert_eq!(
        Realm::default().eval("({valueOf: 1}) + 2"),
        Ok(Value::String("[object Object]2".into()))
    );
    type_error("({toString: 1}) + 2");
    type_error("+({__proto__: null, '@@toPrimitive': 7})");
}
