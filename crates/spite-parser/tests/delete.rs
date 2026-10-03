//! Delete grammar and strict-mode identifier early errors.

use spite_core::DiagnosticKind;
use spite_parser::parse_script;

#[test]
fn delete_syntax_snapshot() {
    insta::assert_debug_snapshot!(parse_script("delete (x); delete (0, x);").unwrap());
}

#[test]
fn delete_rejects_strict_identifier_references_but_accepts_values() {
    for operand in ["x", "(x)", "((x))", "undefined", "missing", "eval"] {
        assert!(parse_script(&format!("delete {operand}")).is_ok());
        assert_eq!(
            parse_script(&format!("'use strict'; delete {operand}"))
                .unwrap_err()
                .kind,
            DiagnosticKind::Syntax
        );
    }
    for operand in [
        "1",
        "null",
        "(0, x)",
        "(x = 1)",
        "x++",
        "++x",
        "(x ? y : z)",
        "typeof missing",
    ] {
        assert!(
            parse_script(&format!("'use strict'; delete {operand}")).is_ok(),
            "{operand}"
        );
    }
    assert_eq!(
        parse_script(r"del\u0065te x").unwrap_err().kind,
        DiagnosticKind::Syntax
    );
    assert_eq!(
        parse_script("delete x ** 2").unwrap_err().kind,
        DiagnosticKind::Syntax
    );
}
