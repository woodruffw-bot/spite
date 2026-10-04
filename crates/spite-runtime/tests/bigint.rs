//! BigInt numeric types, coercion, evaluation order, and host resource boundaries.

use spite_bigint::{BigInt, Budget};
use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn integer(text: &str) -> Value {
    let mut budget = Budget::new(4096, 1_000_000);
    let value = BigInt::parse_digits(text.trim_start_matches('-'), 10, &mut budget).unwrap();
    Value::BigInt(if text.starts_with('-') {
        value.neg(&mut budget).unwrap()
    } else {
        value
    })
}
fn evaluates(source: &str, expected: Value) {
    for prefix in ["", "'use strict'; "] {
        assert_eq!(
            Realm::default().eval(&format!("{prefix}{source}")),
            Ok(expected.clone()),
            "{source}"
        );
    }
}
fn bigint(source: &str, expected: &str) {
    evaluates(source, integer(expected));
}
fn boolean(source: &str, expected: bool) {
    evaluates(source, Value::Boolean(expected));
}

#[test]
fn literal_radices_and_primitive_identity() {
    for source in ["255n", "2_55n", "0xffn", "0XFFn", "0b1111_1111n", "0o377n"] {
        bigint(source, "255");
    }
    bigint("9007199254740993n", "9007199254740993");
    bigint("-0n", "0");
    evaluates("typeof 0n", Value::String(JsString::from("bigint")));
    for (source, expected) in [
        ("!0n", true),
        ("!1n", false),
        ("!-1n", false),
        ("0n === -0n", true),
        ("1n === 1", false),
        ("1n !== '1'", true),
        ("1n === 0x1n", true),
    ] {
        boolean(source, expected);
    }
    bigint("0n || 7n", "7");
    bigint("0n ?? 7n", "0");
    bigint("3n && 7n", "7");
    bigint("0n ? 1n : 2n", "2");
    assert!(integer("0").same_value(&integer("-0")));
    assert!(!integer("1").same_value(&Value::Number(1.0)));
    assert_eq!(
        integer("1").to_number(),
        Err(spite_runtime::ConversionError::BigIntToNumber)
    );
}

#[test]
fn arithmetic_bitwise_and_shift_operators_preserve_bigint_type() {
    for (source, expected) in [
        ("9007199254740993n + 2n", "9007199254740995"),
        ("0n - 9007199254740993n", "-9007199254740993"),
        ("4294967297n * 4294967297n", "18446744082299486209"),
        ("-7n / 3n", "-2"),
        ("7n / -3n", "-2"),
        ("-7n % 3n", "-1"),
        ("7n % -3n", "1"),
        ("-6n % 3n", "0"),
        ("0n ** 0n", "1"),
        ("(-3n) ** 3n", "-27"),
        ("2n ** 64n", "18446744073709551616"),
        ("~0n", "-1"),
        ("~-1n", "0"),
        ("4294967296n | 1n", "4294967297"),
        ("-1n & 4294967296n", "4294967296"),
        ("-1n ^ 4294967296n", "-4294967297"),
        ("1n << 64n", "18446744073709551616"),
        ("-9n >> 1n", "-5"),
        ("-9n << -1n", "-5"),
        ("3n >> -3n", "24"),
        ("1n >> (2n ** 128n)", "0"),
        ("-1n >> (2n ** 128n)", "-1"),
        ("0n << (2n ** 128n)", "0"),
        ("(-1n) ** (2n ** 128n)", "1"),
    ] {
        bigint(source, expected);
    }
}

#[test]
fn updates_assignments_switch_and_loops_use_bigint_semantics() {
    bigint("let x = 3n; x++", "3");
    bigint("let x = 3n; ++x", "4");
    bigint("let x = 0n; --x", "-1");
    bigint("let x = 0n; x--; x", "-1");
    bigint("let x = 2n; x **= 64n; x += 1n; x", "18446744073709551617");
    for (op, expected) in [
        ("-=", "4"),
        ("*=", "21"),
        ("/=", "2"),
        ("%=", "1"),
        ("&=", "3"),
        ("|=", "7"),
        ("^=", "4"),
        ("<<=", "56"),
        (">>=", "0"),
    ] {
        bigint(&format!("let x = 7n; x {op} 3n; x"), expected);
    }
    bigint("const x = 0n; x &&= missing", "0");
    bigint("let x = 0n; x ||= 3n; x ??= missing; x", "3");
    bigint(
        "let sum = 0n; for (let i = 0n; i < 5n; i++) sum += i; sum",
        "10",
    );
    bigint(
        "switch (1n) { case 1: 9n; break; case 1n: 7n; break; default: 0n; }",
        "7",
    );
}

#[test]
fn decimal_string_conversion_does_not_include_a_suffix() {
    for (source, expected) in [
        ("'' + 9007199254740993n", "9007199254740993"),
        ("-9n + 'x'", "-9x"),
        (
            "`${-0n}:${0xFFn}:${2n ** 64n}`",
            "0:255:18446744073709551616",
        ),
        ("let x = 1n; x += '2'; x", "12"),
    ] {
        evaluates(source, Value::String(JsString::from(expected)));
    }
}

#[test]
fn loose_equality_parses_integer_strings_and_compares_number_values_exactly() {
    for (left, right, equal) in [
        ("0n", "''", true),
        ("0n", "'  \\n'", true),
        ("0n", "'-0'", true),
        ("7n", "'+007'", true),
        ("-7n", "' -007 '", true),
        ("255n", "'0xff'", true),
        ("3n", "'0B11'", true),
        ("7n", "'0o7'", true),
        ("0n", "false", true),
        ("1n", "true", true),
        ("2n", "true", false),
        ("0n", "null", false),
        ("0n", "undefined", false),
        ("0n", "-0", true),
        ("9007199254740993n", "9007199254740992", false),
        ("9007199254740992n", "9007199254740992", true),
        ("1n", "1.5", false),
        ("1n", "Infinity", false),
        ("0n", "NaN", false),
    ] {
        for (a, b) in [(left, right), (right, left)] {
            boolean(&format!("{a} == {b}"), equal);
            boolean(&format!("{a} != {b}"), !equal);
        }
    }
    for invalid in [
        "'1n'",
        "'1.0'",
        "'1e0'",
        "'1_0'",
        "'+0x1'",
        "'-0b1'",
        "'0x'",
        "'+'",
        "'Infinity'",
        "'\\ud800'",
        "'\\u00851'",
    ] {
        boolean(&format!("1n == {invalid}"), false);
        boolean(&format!("{invalid} == 1n"), false);
    }
}

#[test]
fn relational_comparisons_handle_mixed_types_without_rounding() {
    for (a, b) in [
        ("9007199254740992", "9007199254740993n"),
        ("-9007199254740993n", "-9007199254740992"),
        ("1n", "1.5"),
        ("-1.5", "-1n"),
        ("0n", "5e-324"),
        ("-5e-324", "0n"),
        ("1n", "Infinity"),
        ("-Infinity", "0n"),
        ("2n", "'10'"),
        ("'2'", "10n"),
        ("0n", "true"),
        ("null", "1n"),
    ] {
        boolean(&format!("{a} < {b}"), true);
        boolean(&format!("{a} <= {b}"), true);
        boolean(&format!("{b} > {a}"), true);
        boolean(&format!("{b} >= {a}"), true);
        boolean(&format!("{a} >= {b}"), false);
        boolean(&format!("{b} <= {a}"), false);
    }
    for equal in ["0", "-0", "null", "false", "''", "'00'", "0n"] {
        for (a, b) in [("0n", equal), (equal, "0n")] {
            boolean(&format!("{a} < {b}"), false);
            boolean(&format!("{a} <= {b}"), true);
        }
    }
    for invalid in ["NaN", "undefined", "'1.5'", "'1e1'", "'1n'", "'\\ud800'"] {
        for op in ["<", "<=", ">", ">="] {
            boolean(&format!("1n {op} {invalid}"), false);
            boolean(&format!("{invalid} {op} 1n"), false);
        }
    }
}

#[test]
fn numeric_errors_are_catchable_and_preserve_evaluation_order() {
    for (source, kind) in [
        ("+1n", ExceptionKind::TypeError),
        ("1n >>> 0n", ExceptionKind::TypeError),
        ("1n / 0n", ExceptionKind::RangeError),
        ("0n % 0n", ExceptionKind::RangeError),
        ("0n ** -1n", ExceptionKind::RangeError),
        ("(-1n) ** -1n", ExceptionKind::RangeError),
        ("const x = 1n; x++", ExceptionKind::TypeError),
    ] {
        for prefix in ["", "'use strict'; "] {
            assert!(
                matches!(Realm::default().eval(&format!("{prefix}{source}")), Err(Error::Exception {kind: actual, ..}) if actual == kind),
                "{source}"
            );
        }
    }
    for op in [
        "+", "-", "*", "/", "%", "**", "&", "|", "^", "<<", ">>", ">>>",
    ] {
        for source in [
            format!("1n {op} 1"),
            format!("1 {op} 1n"),
            format!("1n {op} true"),
        ] {
            assert!(
                matches!(
                    Realm::default().eval(&source),
                    Err(Error::Exception {
                        kind: ExceptionKind::TypeError,
                        ..
                    })
                ),
                "{source}"
            );
        }
    }
    bigint("try { 1n / 0n; } catch { 7n; } finally { 9n; }", "7");
    bigint("try { +1n; } catch { 8n; }", "8");
    let mut realm = Realm::default();
    realm.eval("let x = 0; let value = 1n").unwrap();
    assert!(matches!(
        realm.eval("value += (x = 7)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert_eq!(realm.eval("x"), Ok(Value::Number(7.0)));
    assert_eq!(realm.eval("value"), Ok(integer("1")));
    assert!(matches!(
        realm.eval("1n + missing"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
}

#[test]
fn bigint_size_work_and_string_limits_abort_language_handlers() {
    let mut realm = Realm::new(Limits {
        max_bigint_bits: Some(8),
        ..Limits::default()
    });
    realm.eval("let x = 255n; let flag = 0").unwrap();
    assert!(matches!(
        realm.eval("try { x++; } catch { x = 1n; } finally { flag = 1; }"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("x"), Ok(integer("255")));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert!(matches!(realm.eval("256n"), Err(Error::Limit { .. })));
    assert!(matches!(realm.eval("16n * 16n"), Err(Error::Limit { .. })));
    assert!(matches!(realm.eval("2n ** 8n"), Err(Error::Limit { .. })));
    let mut realm = Realm::new(Limits {
        max_steps: Some(100),
        ..Limits::default()
    });
    assert!(matches!(
        realm.eval("(1n << 1024n) / 3n"),
        Err(Error::Limit { .. })
    ));
    assert!(matches!(
        realm.eval(&format!("{}n", "9".repeat(1000))),
        Err(Error::Limit { .. })
    ));
    let mut realm = Realm::new(Limits {
        max_string_units: Some(3),
        ..Limits::default()
    });
    assert!(matches!(realm.eval("`${1000n}`"), Err(Error::Limit { .. })));
    assert!(matches!(realm.eval("'' + 1000n"), Err(Error::Limit { .. })));
    let mut realm = Realm::new(Limits {
        max_bigint_bits: Some(0),
        ..Limits::default()
    });
    assert_eq!(realm.eval("0n"), Ok(integer("0")));
    assert!(matches!(realm.eval("0n ** 0n"), Err(Error::Limit { .. })));
}
