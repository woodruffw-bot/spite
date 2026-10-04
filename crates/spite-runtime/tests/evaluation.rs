//! Observable behavior of the first executable subset.

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn eval(source: &str) -> Value {
    Realm::default().eval(source).unwrap()
}
fn number(source: &str, expected: f64) {
    assert!(
        eval(source).same_value(&Value::Number(expected)),
        "{source}: expected {expected:?}"
    );
}
fn boolean(source: &str, expected: bool) {
    assert_eq!(eval(source), Value::Boolean(expected), "{source}");
}
fn exception(source: &str, kind: ExceptionKind) {
    assert!(
        matches!(Realm::default().eval(source), Err(Error::Exception { kind: actual, .. }) if actual == kind),
        "{source}"
    );
}

#[test]
fn arithmetic_precedence_and_signed_zero() {
    for (source, expected) in [
        ("1 + 2 * 3", 7.0),
        ("(1 + 2) * 3", 9.0),
        ("2 ** 3 ** 2", 512.0),
        ("2 ** -2", 0.25),
        ("(-2) ** 3", -8.0),
        ("-0", -0.0),
        ("1 / -0", f64::NEG_INFINITY),
        ("0 / 0", f64::NAN),
        ("-4 % 2", -0.0),
        ("-5 % 2", -1.0),
        ("5 % -2", 1.0),
        ("1 / 0", f64::INFINITY),
        ("NaN ** 0", 1.0),
        ("1 ** NaN", f64::NAN),
        ("1 ** Infinity", f64::NAN),
        ("(-1) ** Infinity", f64::NAN),
        ("(-0) ** -3", f64::NEG_INFINITY),
        ("(-0) ** 3", -0.0),
        ("(-Infinity) ** -3", -0.0),
        ("(-Infinity) ** 0.5", f64::INFINITY),
        ("(-2) ** 0.5", f64::NAN),
        ("2 ** -Infinity", 0.0),
        ("0.5 ** Infinity", 0.0),
    ] {
        number(source, expected);
    }
}

#[test]
fn coercions_follow_the_numeric_string_grammar() {
    for (source, expected) in [
        ("+null", 0.0),
        ("+true", 1.0),
        ("+false", 0.0),
        ("+undefined", f64::NAN),
        ("+''", 0.0),
        ("+'  '\n", 0.0),
        ("+'0xff'", 255.0),
        ("+'0b10'", 2.0),
        ("+'0o10'", 8.0),
        ("+'01'", 1.0),
        ("+'-0'", -0.0),
        ("+'+.5e2'", 50.0),
        ("+'1.'", 1.0),
        ("+'+Infinity'", f64::INFINITY),
        ("+'-Infinity'", f64::NEG_INFINITY),
        ("+'inf'", f64::NAN),
        ("+'infinity'", f64::NAN),
        ("+'1_0'", f64::NAN),
        ("+'+0x1'", f64::NAN),
        ("+'-0b1'", f64::NAN),
        ("+'1n'", f64::NAN),
        ("+'\\ud800'", f64::NAN),
        ("+'\\u0085'", f64::NAN),
        ("+'\\uFEFF1\\u2028'", 1.0),
        ("'3' * '4'", 12.0),
    ] {
        number(source, expected);
    }
}

#[test]
fn number_to_string_uses_ecmascript_not_rust_presentation() {
    for (source, expected) in [
        ("'' + -0", "0"),
        ("'' + NaN", "NaN"),
        ("'' + Infinity", "Infinity"),
        ("'' + -Infinity", "-Infinity"),
        ("'' + 1e21", "1e+21"),
        ("'' + 1e20", "100000000000000000000"),
        ("'' + 1e-6", "0.000001"),
        ("'' + 1e-7", "1e-7"),
        ("'' + 1.23e-7", "1.23e-7"),
        ("'' + 1.23", "1.23"),
        ("'' + 1000000000000000128", "1000000000000000100"),
        ("'' + 5e-324", "5e-324"),
        ("'' + 1e23", "1e+23"),
    ] {
        assert_eq!(
            eval(source),
            Value::String(JsString::from(expected)),
            "{source}"
        );
    }
}

#[test]
fn strings_compare_code_units_and_preserve_surrogates() {
    assert_eq!(
        eval("'\\ud800' + '\\udc00'"),
        Value::String(JsString::from("𐀀"))
    );
    assert_eq!(
        eval("'a' + true + null + undefined"),
        Value::String(JsString::from("atruenullundefined"))
    );
    boolean("'💩' < '\\uFFFF'", true);
    boolean("'10' < '2'", true);
    boolean("'10' < 2", false);
    boolean("'💩' === '\\ud83d\\udca9'", true);
}

#[test]
fn equality_algorithms_are_distinct() {
    for (source, expected) in [
        ("NaN === NaN", false),
        ("0 === -0", true),
        ("null == undefined", true),
        ("null === undefined", false),
        ("false == 0", true),
        ("'' == false", true),
        ("false == null", false),
        ("'0' == 0", true),
        ("'0' === 0", false),
        ("true == true", true),
        ("true == false", false),
        ("null == 0", false),
        ("undefined != null", false),
        ("0 !== -0", false),
        ("NaN < 1", false),
        ("NaN >= 1", false),
        ("1 <= NaN", false),
    ] {
        boolean(source, expected);
    }
    assert!(Value::Number(f64::NAN).same_value(&Value::Number(-f64::NAN)));
    assert!(!Value::Number(0.0).same_value(&Value::Number(-0.0)));
}

#[test]
fn short_circuit_and_conditional_evaluation() {
    boolean("false && missing", false);
    boolean("true || missing", true);
    number("0 ?? missing", 0.0);
    number("null ?? 4", 4.0);
    number("undefined ?? 5", 5.0);
    number("true ? 2 : missing", 2.0);
    number("false ? missing : 3", 3.0);
    number(
        "let x = 0; false && (x = 1); true || (x = 2); 0 ?? (x = 3); x",
        0.0,
    );
    number("let x = 0; (x = 2) + (x = x + 1)", 5.0);
    number("let x = 0; (x = 1, x = x + 1, x)", 2.0);
    number("true?.3:0", 0.3);
}

#[test]
fn lexical_environments_and_temporal_dead_zones() {
    number("let x = 1; { let x = 2; x = 3; } x", 1.0);
    number("let x = 1; { x = 2; } x", 2.0);
    number("let x = 1, y = x + 2; y", 3.0);
    assert_eq!(eval("let x; x"), Value::Undefined);
    exception("x; let x = 1", ExceptionKind::ReferenceError);
    exception("let x = x", ExceptionKind::ReferenceError);
    exception(
        "let x = 1; { x; let x = 2; }",
        ExceptionKind::ReferenceError,
    );
    exception("x = 1; let x;", ExceptionKind::ReferenceError);
    exception("const x = 1; x = 2", ExceptionKind::TypeError);
    exception("let undefined", ExceptionKind::SyntaxError);
    exception("let NaN", ExceptionKind::SyntaxError);
    number("{ let undefined = 3; undefined; }", 3.0);
}

#[test]
fn completions_preserve_empty_and_undefined() {
    assert_eq!(eval(""), Value::Undefined);
    number("1; ; let x; {}", 1.0);
    number("1; {2; let x;}", 2.0);
    assert_eq!(eval("1; if (false) 2;"), Value::Undefined);
    assert_eq!(eval("1; if (true) {}"), Value::Undefined);
    number("if (false) 1; else 2", 2.0);
    assert_eq!(eval("1; void 0;"), Value::Undefined);
    assert_eq!(
        Realm::default().eval("throw 42"),
        Err(Error::Thrown(Value::Number(42.0)))
    );
}

#[test]
fn typeof_special_cases_only_unresolvable_references() {
    assert_eq!(
        eval("typeof missing"),
        Value::String(JsString::from("undefined"))
    );
    assert_eq!(
        eval("typeof (missing)"),
        Value::String(JsString::from("undefined"))
    );
    assert_eq!(eval("typeof null"), Value::String(JsString::from("object")));
    exception("typeof x; let x", ExceptionKind::ReferenceError);
    exception("typeof (0, missing)", ExceptionKind::ReferenceError);
    assert!(matches!(
        Realm::default().eval("typeof Math"),
        Err(Error::Unsupported { .. })
    ));
}

#[test]
fn global_bindings_persist_without_polluting_other_realms() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval("let x = 1; x"), Ok(Value::Number(1.0)));
    assert_eq!(realm.eval("x = 2; x"), Ok(Value::Number(2.0)));
    assert!(matches!(
        realm.eval("let y; let x"),
        Err(Error::Exception {
            kind: ExceptionKind::SyntaxError,
            ..
        })
    ));
    assert_eq!(
        realm.eval("typeof y"),
        Ok(Value::String(JsString::from("undefined")))
    );
    exception("x", ExceptionKind::ReferenceError);
    number("accidental = 1; accidental", 1.0);
    exception(
        "'use strict'; accidental = 1",
        ExceptionKind::ReferenceError,
    );
    number("Infinity = 1; Infinity", f64::INFINITY);
    exception("'use strict'; Infinity = 1", ExceptionKind::TypeError);
    assert_eq!(
        realm.eval("{ let x = 99; throw x; }"),
        Err(Error::Thrown(Value::Number(99.0)))
    );
    assert_eq!(realm.eval("x"), Ok(Value::Number(2.0)));
}

#[test]
fn syntax_errors_precede_all_execution() {
    let mut realm = Realm::default();
    realm.eval("let x = 0").unwrap();
    assert!(matches!(
        realm.eval("x = 1; let a; let a"),
        Err(Error::Parse(_))
    ));
    assert_eq!(realm.eval("x"), Ok(Value::Number(0.0)));
    assert!(matches!(
        realm.eval("x = 1; 'unterminated"),
        Err(Error::Parse(_))
    ));
    assert_eq!(realm.eval("x"), Ok(Value::Number(0.0)));
    assert_eq!(
        realm.eval("x = 1; throw 0"),
        Err(Error::Thrown(Value::Number(0.0)))
    );
    assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
}

#[test]
fn bitwise_operations_use_modulo_conversion() {
    for (source, expected) in [
        ("~0", -1.0),
        ("4294967297 | 0", 1.0),
        ("-1 >>> 0", 4294967295.0),
        ("1 << 31", -2147483648.0),
        ("-8 >> 2", -2.0),
        ("1 << 32", 1.0),
        ("NaN | 0", 0.0),
        ("Infinity | 0", 0.0),
        ("-1.9 | 0", -1.0),
        ("7 & 3", 3.0),
        ("7 ^ 3", 4.0),
    ] {
        number(source, expected);
    }
}

#[test]
fn execution_work_limits_are_opt_in() {
    for source in [
        "function f(){var n=0;while(n<20000)n++;return n;}f()===20000",
        "'a'.repeat(2000).indexOf('a'.repeat(999)+'b')===-1",
        "`${2n**8192n}`.length===2467",
        "let o={length:4000};o[Symbol.isConcatSpreadable]=true;[].concat(o).length===4000",
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Ok(Value::Boolean(true)),
            "{source}"
        );
        assert!(
            matches!(
                Realm::new(Limits {
                    max_steps: Some(100_000),
                    ..Limits::default()
                })
                .eval(source),
                Err(Error::Limit { .. })
            ),
            "{source}"
        );
    }
}

#[test]
fn resource_failures_are_host_errors_and_restore_scopes() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(1_000),
        max_string_units: Some(16),
        ..Limits::default()
    });
    assert!(matches!(
        realm.eval("{ let hidden = 2; while (true) {} }"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("let x = 1"), Ok(Value::Undefined));
    assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(100),
        max_string_units: Some(4),
        ..Limits::default()
    });
    assert!(matches!(
        realm.eval("'abc' + 'def'"),
        Err(Error::Limit { .. })
    ));
    assert!(matches!(realm.eval("'abcde'"), Err(Error::Limit { .. })));
    assert!(matches!(
        realm.eval("typeof missing"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("'ab' + 'cd'"),
        Ok(Value::String(JsString::from("abcd")))
    );
}

#[test]
fn unsupported_features_cannot_masquerade_as_runtime_exceptions() {
    assert!(matches!(
        Realm::default().eval("Math"),
        Err(Error::Unsupported { .. })
    ));
    assert!(matches!(
        Realm::default().eval("BigInt.asIntN"),
        Err(Error::Unsupported { .. })
    ));
    for name in ["console", "process", "require", "setTimeout", "fetch"] {
        exception(name, ExceptionKind::ReferenceError);
    }
}

#[test]
fn unicode_bindings_preserve_code_point_identity() {
    number("let π = 2; { let π = 3; π = 4; } π", 2.0);
    number("let 字 = 6; 字 = 字 * 7; 字", 42.0);
    number("let 𐐀 = 3; let a\u{200c} = 4; 𐐀 + a\u{200c}", 7.0);
    number(
        "let é = 1; let e\u{0301} = 2; let K = 3; let K = 4; é + e\u{0301} + K + K",
        10.0,
    );
    exception("π; let π = 1", ExceptionKind::ReferenceError);
    let mut realm = Realm::default();
    realm.eval("let π = 3").unwrap();
    assert_eq!(realm.eval("π + 1"), Ok(Value::Number(4.0)));
}

#[test]
fn escaped_identifiers_resolve_the_same_binding() {
    number(r"let \u0061 = 1; a = 2; \u{61}", 2.0);
    number(r"let \u03c0 = 3; π = π + 1; \u{3c0}", 4.0);
    number(r"let \u{10400} = 5; 𐐀", 5.0);
    number(r"let a\u200c = 1; a\u{200c} = 2; a\u200c", 2.0);
    number(r"let \u00e9 = 1; let e\u0301 = 2; é + e\u{301}", 3.0);
    exception(r"\u03c0; let π = 1", ExceptionKind::ReferenceError);
    exception(r"const a = 1; \u0061 = 2", ExceptionKind::TypeError);
    assert_eq!(
        eval(r"typeof \u0075ndefined"),
        Value::String(JsString::from("undefined"))
    );
    number(r"\u0049nfinity", f64::INFINITY);
    number(r"l\u0065t = 2; let", 2.0);
    let mut realm = Realm::default();
    realm.eval("let π = 3").unwrap();
    assert_eq!(realm.eval(r"\u03c0"), Ok(Value::Number(3.0)));
    assert!(matches!(
        realm.eval(r"let \u03c0"),
        Err(Error::Exception {
            kind: ExceptionKind::SyntaxError,
            ..
        })
    ));
    assert!(matches!(
        realm.eval(r"π = 4; let tr\u0075e"),
        Err(Error::Parse(_))
    ));
    assert_eq!(realm.eval("π"), Ok(Value::Number(3.0)));
}
