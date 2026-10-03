//! Parser regression tests.

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{MAX_DEPTH, MAX_SOURCE_BYTES, ast::*, parse_script};

#[test]
fn syntax_snapshot() {
    let script =
        parse_script("let x = 1 + 2 * 3; x = x ** 2; if (x > 4) { x; } else { null; }").unwrap();
    insta::assert_debug_snapshot!(script);
}

#[test]
fn diagnostic_snapshot() {
    let sources = [
        "const a;",
        "let a; let a;",
        "throw\n1",
        "1 ?? 2 || 3",
        "-2 ** 2",
        "'\\u{110000}'",
        "/*",
    ];
    let errors: Vec<_> = sources
        .iter()
        .map(|source| parse_script(source).unwrap_err())
        .collect();
    insta::assert_debug_snapshot!(errors);
}

#[test]
fn asi_observes_line_terminators_and_continuations() {
    assert_eq!(
        parse_script("1\n2\u{2028}3\u{2029}4\r5")
            .unwrap()
            .statements()
            .len(),
        5
    );
    assert_eq!(parse_script("1/*\n*/2").unwrap().statements().len(), 2);
    assert_eq!(parse_script("1\n+2").unwrap().statements().len(), 1);
    assert_eq!(parse_script("{1}").unwrap().statements().len(), 1);
    for source in ["1 2", "throw /*\n*/ 1", "throw\r\n1"] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    for source in ["1\n(2)", "1\n[2]", "x.foo"] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
}

#[test]
fn strict_directives_and_early_errors() {
    assert!(parse_script("'use strict';").unwrap().is_strict());
    assert!(!parse_script("'use\\x20strict';").unwrap().is_strict());
    assert!(!parse_script("('use strict');").unwrap().is_strict());
    assert!(!parse_script("0; 'use strict';").unwrap().is_strict());
    assert!(parse_script("'other'; 'use strict';").unwrap().is_strict());
    for source in [
        "'use strict'; let eval;",
        "'use strict'; arguments = 1;",
        "'use strict'; yield;",
        "let a, a;",
        "let a; {let a; let a;}",
        "if (true) let x = 1;",
        "let let = 1;",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    assert!(parse_script("let a; { let a; }").is_ok());
    assert!(parse_script("let eval; let yield;").is_ok());
    assert!(parse_script("let; let + 1;").is_ok());
}

#[test]
fn operator_grammar() {
    for source in [
        "-2 ** 2",
        "!2 ** 2",
        "2 ** -2 ** 2",
        "1 ?? 2 && 3",
        "1 || 2 ?? 3",
        "1 = 2",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    for source in [
        "(-2) ** 2",
        "2 ** -2",
        "1 ?? (2 || 3)",
        "(1 ?? 2) && 3",
        "(a) = 2",
        "a = b = 1",
        "1 ? 2 : 3 ? 4 : 5",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}

fn literal(source: &str) -> Literal {
    let script = parse_script(source).unwrap();
    let StatementKind::Expression(Expr {
        kind: ExprKind::Literal(literal),
        ..
    }) = &script.statements()[0].kind
    else {
        panic!("expected a literal")
    };
    literal.clone()
}

#[test]
fn numeric_literals_round_once() {
    for (source, expected) in [
        (".5", 0.5),
        ("1.", 1.0),
        ("1.e2", 100.0),
        ("0b1010", 10.0),
        ("0o17", 15.0),
        ("0x1f", 31.0),
        ("1_000.0_5e+1", 10000.5),
        ("1e9999", f64::INFINITY),
        ("0x20000000000001", 9007199254740992.0),
        ("0x20000000000003", 9007199254740996.0),
    ] {
        assert_eq!(literal(source), Literal::Number(expected), "{source}");
    }
    for source in [
        "0x", "0b2", "1_", "1__0", "1._0", "1e+", "123abc", "0x1g", "0b1_", "1e_2",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    let enormous = format!("0x1{}", "0".repeat(300));
    assert_eq!(literal(&enormous), Literal::Number(f64::INFINITY));
}

#[test]
fn strings_preserve_utf16_and_escapes() {
    assert_eq!(
        literal("'\\ud800'"),
        Literal::String(JsString::from_code_units(vec![0xd800]))
    );
    assert_eq!(
        literal("'💩\\u{1f4a9}'"),
        Literal::String(JsString::from("💩💩"))
    );
    assert_eq!(literal("'a\\\r\nb'"), Literal::String(JsString::from("ab")));
    assert_eq!(
        literal("'\\0\\v\\x41\\u0042'"),
        Literal::String(JsString::from("\0\u{b}AB"))
    );
    assert_eq!(
        literal("'\u{2028}\u{2029}'"),
        Literal::String(JsString::from("\u{2028}\u{2029}"))
    );
    for source in ["'\n'", "'\\x0g'", "'\\u{}'", "'\\u{110000}'", "'unfinished"] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn unsupported_features_remain_distinct() {
    for source in ["tag`x`", "function f() {}", "'\\1'", "012"] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
}

#[test]
fn limits_cover_nested_and_flat_expression_trees() {
    let nested = format!(
        "{}0{}",
        "(".repeat(MAX_DEPTH * 4),
        ")".repeat(MAX_DEPTH * 4)
    );
    let chain = "1+".repeat(MAX_DEPTH * 4) + "1";
    let huge = " ".repeat(MAX_SOURCE_BYTES + 1);
    for source in [nested, chain, huge] {
        assert_eq!(
            parse_script(&source).unwrap_err().kind,
            DiagnosticKind::Limit
        );
    }
}

#[test]
fn comments_whitespace_and_hashbang() {
    assert!(parse_script("#!/usr/bin/env spite\n/* comment */ // comment\n1").is_ok());
    assert!(parse_script("\u{feff}\u{a0}\u{1680}1\u{2007}\u{3000}").is_ok());
    assert!(parse_script("\u{85}1").is_err());
    assert!(parse_script("1\n#!invalid").is_err());
}
