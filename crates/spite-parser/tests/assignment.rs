//! Assignment operators share right associativity and target restrictions.

use spite_core::DiagnosticKind;
use spite_parser::{MAX_DEPTH, parse_script};

const OPERATORS: &[&str] = &[
    "=", "+=", "-=", "*=", "/=", "%=", "**=", "<<=", ">>=", ">>>=", "&=", "^=", "|=", "&&=", "||=",
    "??=",
];

#[test]
fn compound_assignment_syntax_snapshot() {
    insta::assert_debug_snapshot!(parse_script("x += y *= z; (x) ??= y || z;").unwrap());
}

#[test]
fn all_assignments_accept_parenthesized_targets_and_right_associativity() {
    for op in OPERATORS {
        for source in [
            format!("(x) {op} y {op} z"),
            format!("x\n{op}\ny"),
            format!("for (x {op} 1; x < 2; x {op} 1) ;"),
            format!("x {op} y ? z : w"),
            format!("x {op} y, z"),
        ] {
            assert!(parse_script(&source).is_ok(), "{source}");
        }
    }
    for source in [
        "x += ++y",
        "x *= y++",
        "x **= -y",
        "x ||= y ?? z",
        "x ??= y && z",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}

#[test]
fn all_assignments_reject_invalid_and_strict_restricted_targets() {
    for op in OPERATORS {
        for target in ["1", "(x + y)", "x++", "(x, y)", "++x", "x ? y : z"] {
            let source = format!("({target}) {op} 2");
            assert_eq!(
                parse_script(&source).unwrap_err().kind,
                DiagnosticKind::Syntax,
                "{source}"
            );
        }
        for target in ["eval", "arguments", "yield", "(eval)"] {
            let source = format!("'use strict'; {target} {op} 2");
            assert_eq!(
                parse_script(&source).unwrap_err().kind,
                DiagnosticKind::Syntax,
                "{source}"
            );
        }
    }
}

#[test]
fn assignment_chains_obey_the_depth_limit() {
    let source = format!("{}0", "x += ".repeat(MAX_DEPTH * 2));
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}
