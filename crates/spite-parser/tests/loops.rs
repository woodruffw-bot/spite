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

#[test]
fn loop_control_syntax_snapshot() {
    let script = parse_script("while (true) { if (false) continue; break; }").unwrap();
    insta::assert_debug_snapshot!(script);
}

#[test]
fn loop_control_diagnostic_snapshot() {
    let errors: Vec<_> = [
        "break;",
        "{ continue; }",
        "if (false) break;",
        "while (false) ; continue;",
    ]
    .map(|source| parse_script(source).unwrap_err())
    .into();
    insta::assert_debug_snapshot!(errors);
}

#[test]
fn loop_control_requires_an_enclosing_iteration() {
    for keyword in ["break", "continue"] {
        for source in [
            format!("{keyword};"),
            format!("{{ {keyword}; }}"),
            format!("if (false) {keyword};"),
            format!("while (false) ; {keyword};"),
            format!("do ; while (false); {keyword};"),
        ] {
            let error = parse_script(&source).unwrap_err();
            assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
            assert_eq!(
                error.message,
                format!("{keyword} requires an enclosing loop")
            );
            assert_eq!(
                &source[error.span.start..error.span.end],
                format!("{keyword};")
            );
        }
        for source in [
            format!("while (false) {keyword};"),
            format!("do {keyword}; while (false);"),
            format!("while (false) {{ if (true) {{ {keyword}; }} }}"),
        ] {
            assert!(parse_script(&source).is_ok(), "{source}");
        }
    }
}

#[test]
fn line_terminators_prevent_loop_control_labels() {
    for keyword in ["break", "continue"] {
        for separator in [
            "\n",
            "\r",
            "\r\n",
            "\u{2028}",
            "\u{2029}",
            "/*\n*/",
            "// comment\n",
        ] {
            let source = format!("while (false) {{ {keyword}{separator}label; }}");
            let script = parse_script(&source).unwrap();
            let StatementKind::While { body, .. } = &script.statements()[0].kind else {
                panic!("expected while");
            };
            let StatementKind::Block(statements) = &body.kind else {
                panic!("expected block");
            };
            assert_eq!(statements.len(), 2, "{source}");
            assert!(matches!(statements[1].kind, StatementKind::Expression(_)));
        }
        for separator in [" ", "/*comment*/", "\u{a0}"] {
            let source = format!("while (false) {{ {keyword}{separator}label; }}");
            assert_eq!(
                parse_script(&source).unwrap_err().kind,
                DiagnosticKind::Unsupported,
                "{source}"
            );
        }
        for ending in [" 1;", " if;", " true;", " +1;"] {
            let source = format!("while (false) {{ {keyword}{ending} }}");
            assert_eq!(
                parse_script(&source).unwrap_err().kind,
                DiagnosticKind::Syntax,
                "{source}"
            );
        }
        assert!(parse_script(&format!("while (false) {{ {keyword} }}")).is_ok());
    }
    assert_eq!(
        parse_script("label: while (true) break label;")
            .unwrap_err()
            .kind,
        DiagnosticKind::Unsupported
    );
}

#[test]
fn let_expression_statement_and_declaration_lookahead_are_distinct() {
    for source in [
        "while (false) let\nx = 1;",
        "while (false) let\n{}",
        "if (false) let\nx = 1;",
        "if (false) let\n{}",
        "do let\nwhile (false)",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
        assert_eq!(
            parse_script(&format!("'use strict'; {source}"))
                .unwrap_err()
                .kind,
            DiagnosticKind::Syntax
        );
    }
    for source in [
        "while (false) let\n[x] = 0;",
        "if (false) let\n[x] = 0;",
        "do let\n[x] = 0; while (false);",
        "while (false) let x;",
        "while (false) let {}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    // In a StatementList, the same lookahead *does* introduce a declaration,
    // including across line terminators. No ASI is needed for `let\nx;`.
    let script = parse_script("let\nx;").unwrap();
    assert_eq!(script.statements().len(), 1);
    assert!(matches!(
        script.statements()[0].kind,
        StatementKind::Lexical { .. }
    ));
    // The `let [` lookahead restriction uses the terminal, not escaped names.
    assert_eq!(
        parse_script(r"while (false) l\u0065t[x];")
            .unwrap_err()
            .kind,
        DiagnosticKind::Unsupported
    );
}
