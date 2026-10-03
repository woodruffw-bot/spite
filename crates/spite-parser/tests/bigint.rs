//! Exact BigInt syntax and numeric-token boundaries (ECMA-262 12.9.3).

use spite_core::DiagnosticKind;
use spite_parser::{ast::*, parse_script};

#[test]
fn bigint_literals_preserve_exact_digits_and_radix() {
    for (source, digits, radix) in [
        ("0n", "0", 10),
        ("9007199254740993n", "9007199254740993", 10),
        ("1_000_000n", "1000000", 10),
        ("0b00_11n", "0011", 2),
        ("0B101n", "101", 2),
        ("0o00_77n", "0077", 8),
        ("0O123n", "123", 8),
        ("0x00_ff_ABn", "00ffAB", 16),
        ("0XABCn", "ABC", 16),
    ] {
        for prefix in ["", "'use strict'; "] {
            let script = parse_script(&format!("{prefix}{source}")).unwrap();
            let StatementKind::Expression(Expr {
                kind: ExprKind::Literal(literal),
                ..
            }) = &script.statements().last().unwrap().kind
            else {
                panic!("literal");
            };
            assert_eq!(
                *literal,
                Literal::BigInt {
                    digits: digits.into(),
                    radix
                },
                "{source}"
            );
        }
    }
    // Parsing stores digits and never rounds to Number or evaluates a large integer.
    assert!(parse_script(&format!("{}n", "9".repeat(100_000))).is_ok());
}

#[test]
fn bigint_invalid_forms_are_syntax_errors_in_both_modes() {
    for source in [
        "00n",
        "01n",
        "08n",
        "09n",
        "0_0n",
        "01_0n",
        "1.0n",
        "1.n",
        ".1n",
        "1e0n",
        "1e+1n",
        "1e-1n",
        "1_n",
        "1__0n",
        "0xn",
        "0x_1n",
        "0x1_n",
        "0b2n",
        "0o8n",
        "1nn",
        "1n0",
        "1na",
        "1n$",
        "1nπ",
        "1n\\u0061",
        "1N",
    ] {
        for prefix in ["", "'use strict'; "] {
            assert_eq!(
                parse_script(&format!("{prefix}{source}")).unwrap_err().kind,
                DiagnosticKind::Syntax,
                "{source}"
            );
        }
    }
}

#[test]
fn bigint_operators_and_diagnostics_snapshot() {
    insta::assert_debug_snapshot!(
        parse_script("let x = -0xF_Fn; x += 2n ** 64n; typeof x;").unwrap()
    );
    let errors: Vec<_> = ["00n", "1.0n", "1e0n", "0_1n", "1nπ"]
        .map(|s| parse_script(s).unwrap_err())
        .into();
    insta::assert_debug_snapshot!("bigint_diagnostics", errors);
}
