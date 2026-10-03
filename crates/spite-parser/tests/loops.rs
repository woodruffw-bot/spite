//! Iteration grammar and early-error regressions.

use spite_core::DiagnosticKind;
use spite_parser::{MAX_DEPTH, ast::StatementKind, parse_script};

#[test]
fn loop_syntax_snapshot() {
    let script = parse_script("while (false) ; do { 1; } while (false) 2;").unwrap();
    insta::assert_debug_snapshot!(script);
}

#[test]
fn do_while_has_a_special_asi_rule() {
    for source in [
        "do {} while (false)",
        "do {} while (false);",
        "do {} while (false) 1;",
        "do {} while (false) (1);",
        "do {} while (false) let x;",
        "do {} while (false) do {} while (false)",
        "{ do {} while (false) }",
        "do ; while (false)",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    // Only the terminating semicolon is optional; an empty loop body needs one.
    for source in ["do while (false)", "while (false)", "while (false)\n"] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax
        );
    }
}

#[test]
fn loop_bodies_are_statements_not_lexical_declarations() {
    for source in [
        "while (false) let x;",
        "while (false) const x = 1;",
        "do let x; while (false);",
        "do const x = 1; while (false);",
        "while () ;",
        "do ; while ();",
        "do ; (false);",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    for source in [
        "while (false) { let x; }",
        "do { const x = 1; } while (false);",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}

#[test]
fn validation_reaches_conditions_and_bodies() {
    for source in [
        "'use strict'; while (yield) ;",
        "'use strict'; do ; while (arguments = 1);",
        "'use strict'; while (false) { let eval; }",
        "'use strict'; do { yield; } while (false);",
        "while (false) { let x; let x; }",
        "do { const x = 1; let x; } while (false);",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn dangling_else_stays_with_the_nearest_if() {
    let script = parse_script("if (true) while (false) if (false) ; else ;").unwrap();
    let StatementKind::If {
        consequent,
        alternate: None,
        ..
    } = &script.statements()[0].kind
    else {
        panic!("else must not attach to the outer if");
    };
    let StatementKind::While { body, .. } = &consequent.kind else {
        panic!("expected while");
    };
    assert!(matches!(
        body.kind,
        StatementKind::If {
            alternate: Some(_),
            ..
        }
    ));
    assert!(parse_script("if (true) do ; while (false) else ;").is_ok());
}

#[test]
fn nested_loops_obey_parser_depth_limits() {
    for source in [
        format!("{};", "while (false) ".repeat(MAX_DEPTH * 2)),
        format!(
            "{};{}",
            "do ".repeat(MAX_DEPTH * 2),
            " while (false);".repeat(MAX_DEPTH * 2)
        ),
    ] {
        assert_eq!(
            parse_script(&source).unwrap_err().kind,
            DiagnosticKind::Limit
        );
    }
}
