//! Ephemeral primitive wrappers and String exotic own properties.

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn value(source: &str, expected: Value) {
    assert_eq!(Realm::default().eval(source), Ok(expected), "{source}");
}

#[test]
fn string_length_and_indices_count_utf16_code_units() {
    value("'💩x'.length", Value::Number(3.0));
    value("''.length", Value::Number(0.0));
    for (index, unit) in [(0, 0xd83d), (1, 0xdca9), (2, 0x78)] {
        value(
            &format!("'💩x'[{index}]"),
            Value::String(JsString::from_code_units(vec![unit])),
        );
    }
    value(
        r"'\ud800'[0]",
        Value::String(JsString::from_code_units(vec![0xd800])),
    );
    value("'💩'[0].length", Value::Number(1.0));
    value("'abc'[1n]", Value::String(JsString::from("b")));
    value("'abc'[-0]", Value::String(JsString::from("a")));
}

#[test]
fn only_canonical_in_range_integral_indices_are_own_properties() {
    for key in [
        "-0",
        "-1",
        "00",
        "01",
        "+0",
        "1.0",
        "1e0",
        " 1",
        "NaN",
        "Infinity",
        "0.5",
        "3",
        "4294967295",
        "9007199254740993",
        "999999999999999999999999999999",
        "",
    ] {
        value(&format!("'abc'['{key}']"), Value::Undefined);
        value(
            &format!("'use strict'; delete 'abc'['{key}']"),
            Value::Boolean(true),
        );
    }
    value("''[0]", Value::Undefined);
    value("'abc'[false]", Value::Undefined);
    value("'abc'[null]", Value::Undefined);
    value("'abc'[undefined]", Value::Undefined);
}

#[test]
fn primitive_writes_do_not_persist_and_sloppy_assignments_return_rhs_values() {
    for base in ["'abc'", "1", "true", "1n"] {
        value(&format!("({base}).x = 7"), Value::Number(7.0));
        value(&format!("let p = {base}; p.x = 7; p.x"), Value::Undefined);
        value(&format!("delete ({base}).x"), Value::Boolean(true));
        value(&format!("delete ({base}).toString"), Value::Boolean(true));
    }
    value("'abc'[0] = 'z'", Value::String(JsString::from("z")));
    value(
        "let s = 'abc'; s[0] = 'z'; s",
        Value::String(JsString::from("abc")),
    );
    value("let s = 'abc'; s.length = 99; s.length", Value::Number(3.0));
}

#[test]
fn strict_primitive_writes_throw_after_evaluating_the_rhs() {
    for target in [
        "'abc'[0]",
        "'abc'.length",
        "'abc'.x",
        "(1).x",
        "true.x",
        "1n.x",
        "'abc'.toString",
    ] {
        assert!(
            matches!(
                Realm::default().eval(&format!("'use strict'; {target} = 7")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{target}"
        );
        value(
            &format!(
                "'use strict'; let effect = 0; try {{ {target} = (effect = 1); }} catch {{ effect += 2; }} effect"
            ),
            Value::Number(3.0),
        );
    }
}

#[test]
fn string_own_properties_are_nonconfigurable_but_inherited_methods_are_not_deleted() {
    for target in ["'abc'[0]", "'abc'[2]", "'abc'.length"] {
        value(&format!("delete {target}"), Value::Boolean(false));
        assert!(
            matches!(
                Realm::default().eval(&format!("'use strict'; delete {target}")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{target}"
        );
    }
    for target in [
        "'abc'[3]",
        "''.x",
        "'abc'.charAt",
        "(1).toString",
        "true.valueOf",
        "1n.toLocaleString",
    ] {
        value(
            &format!("'use strict'; delete {target}"),
            Value::Boolean(true),
        );
    }
}

#[test]
fn string_updates_return_numeric_values_and_respect_strict_putvalue() {
    value("'5'[0]++", Value::Number(5.0));
    value("++'5'[0]", Value::Number(6.0));
    value(
        "let s = '5'; ++s[0]; s[0]",
        Value::String(JsString::from("5")),
    );
    value("'abc'.length += 4", Value::Number(7.0));
    let Value::Number(number) = Realm::default().eval("'abc'[0]++").unwrap() else {
        panic!("Number")
    };
    assert!(number.is_nan());
    for source in [
        "'use strict'; '5'[0]++",
        "'use strict'; ++'5'[0]",
        "'use strict'; 'abc'.length += 4",
    ] {
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
}

#[test]
fn logical_assignment_can_skip_a_forbidden_primitive_write() {
    value("'use strict'; 'abc'.length ||= missing", Value::Number(3.0));
    value("'use strict'; ''.length &&= missing", Value::Number(0.0));
    value("'use strict'; 'abc'.length ??= missing", Value::Number(3.0));
    value(
        "'use strict'; 'a'[0] ||= missing",
        Value::String(JsString::from("a")),
    );
    assert!(matches!(
        Realm::default().eval("'use strict'; 'abc'.length &&= 7"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn missing_standard_methods_are_distinct_from_absent_and_annex_b_properties() {
    for source in [
        "'s'.charAt",
        "'s'.toWellFormed",
        "'s'.trimStart",
        "'s'.replaceAll",
        "(1).toFixed",
        "(1).toPrecision",
        "true.toString",
        "1n.toLocaleString",
        "'s'.hasOwnProperty",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    for source in [
        "'s'.substr",
        "'s'.bold",
        "'s'.trimLeft",
        "'s'.trimRight",
        "'s'.__proto__",
        "true.toFixed",
        "1n.length",
        "(1).charAt",
        "true.missing",
    ] {
        value(source, Value::Undefined);
    }
}
