//! Object-literal names, computed keys, shorthand, and prototype early errors.

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{
    MAX_DEPTH,
    ast::{ExprKind, Literal, PropertyKind, PropertyName, StatementKind},
    parse_script,
};

#[test]
fn object_literal_snapshot() {
    insta::assert_debug_snapshot!(
        parse_script("({plain: 1, shorthand, ['x' + n]: 2, 0x10n: 3, __proto__: null,})").unwrap()
    );
}

#[test]
fn property_names_accept_identifier_names_and_preserve_utf16() {
    let script =
        parse_script(r"({if: 1, null: 2, true: 3, false: 4, \u0069f: 5, '\ud800': 6})").unwrap();
    let StatementKind::Expression(expression) = &script.statements()[0].kind else {
        panic!("expression")
    };
    let ExprKind::Parenthesized(expression) = &expression.kind else {
        panic!("parentheses")
    };
    let ExprKind::Object(properties) = &expression.kind else {
        panic!("object")
    };
    let expected = [
        JsString::from("if"),
        JsString::from("null"),
        JsString::from("true"),
        JsString::from("false"),
        JsString::from("if"),
        JsString::from_code_units(vec![0xd800]),
    ];
    for (property, expected) in properties.iter().zip(expected) {
        assert_eq!(
            property.name,
            PropertyName::Literal(Literal::String(expected))
        );
        assert_eq!(property.kind, PropertyKind::Data);
    }
}

#[test]
fn literal_and_computed_keys_shorthand_and_trailing_commas_parse() {
    for source in [
        "({})",
        "({a: 1, a: 2,})",
        "let a = {a, b: {c: 2}};",
        "({0: 1, 1.5: 2, 1e100: 3, 0xff: 4, 2n: 5, 0b10n: 6})",
        "({[a = 1]: 2, [(a, b)]: c ? d : e})",
        "({get, set, async, get: 1, set: 2, async: 3})",
        "({let, yield, eval, arguments})",
        r"({\u0061, \u0069f: 1})",
        "`${{a: 1, b: {c: `x${n}`}}}`",
        "'use strict'; ({eval, arguments, yield: 1, let: 2})",
        "({__proto__: null, ['__proto__']: 1, __proto__})",
        "({__proto__, __proto__})",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
}

#[test]
fn duplicate_prototype_setters_are_early_errors_only_for_colon_literal_names() {
    for source in [
        "({__proto__: null, __proto__: null})",
        "({'__proto__': null, __proto__: 1})",
        r"({__pr\u006fto__: null, '__proto__': null})",
        "'use strict'; ({__proto__: null, __proto__: null})",
    ] {
        let error = parse_script(source).unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
        assert_eq!(
            error.message,
            "duplicate prototype setter in object literal"
        );
    }
    insta::assert_debug_snapshot!(
        parse_script("({__proto__: null, '__proto__': null})").unwrap_err()
    );
}

#[test]
fn malformed_properties_and_nested_strict_violations_are_rejected() {
    for source in [
        "({,})",
        "({a:})",
        "({a: 1,,})",
        "({a: 1 b: 2})",
        "({a",
        "({a = 1})",
        "({get = 1})",
        "({if})",
        "({true})",
        "({null})",
        "({1})",
        "({'x'})",
        "({[a]})",
        "({[]: 1})",
        "({[a, b]: 1})",
        "({-1: 2})",
        "'use strict'; ({yield})",
        "'use strict'; ({let})",
        "'use strict'; ({[eval = 1]: 2})",
        "'use strict'; ({a: arguments = 1})",
        "'use strict'; ({010: 1})",
        "'use strict'; ({'\\1': 1})",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn methods_accessors_and_spread_remain_explicitly_unsupported() {
    for source in [
        "({m() {}})",
        "({get x() {}})",
        "({set x(v) {}})",
        "({get 1() {}})",
        "({['x']() {}})",
        "({*g() {}})",
        "({async m() {}})",
        "({...x})",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
}

#[test]
fn nested_object_values_and_computed_names_obey_depth_limits() {
    for source in [
        format!(
            "({}0{})",
            "{x:".repeat(MAX_DEPTH * 2),
            "}".repeat(MAX_DEPTH * 2)
        ),
        format!(
            "({}0{})",
            "{[".repeat(MAX_DEPTH * 2),
            "]: 0}".repeat(MAX_DEPTH * 2)
        ),
    ] {
        assert_eq!(
            parse_script(&source).unwrap_err().kind,
            DiagnosticKind::Limit
        );
    }
}
