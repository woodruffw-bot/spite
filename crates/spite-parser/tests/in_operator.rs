//! Relational in and the grammar's context-sensitive In parameter.

use spite_core::DiagnosticKind;
use spite_parser::{
    ast::{BinaryOp, ExprKind, StatementKind},
    parse_script,
};

#[test]
fn in_is_left_associative_at_relational_precedence() {
    let script = parse_script("'x' in a in b === true").unwrap();
    let StatementKind::Expression(expression) = &script.statements()[0].kind else {
        panic!("expression")
    };
    let ExprKind::Binary(BinaryOp::StrictEqual, left, _) = &expression.kind else {
        panic!("equality")
    };
    let ExprKind::Binary(BinaryOp::In, left, _) = &left.kind else {
        panic!("in")
    };
    assert!(matches!(left.kind, ExprKind::Binary(BinaryOp::In, ..)));
    insta::assert_debug_snapshot!(script);
}

#[test]
fn grammar_subexpressions_restore_in_inside_a_for_initializer() {
    for initializer in [
        "('x' in o)",
        "o['x' in other]",
        "{x: 'x' in o}",
        "{['x' in o]: 1}",
        "`${'x' in o}`",
        "true ? 'x' in o : false",
        "x = ('x' in o)",
    ] {
        for prefix in ["", "let x = ", "var x = ", "const x = "] {
            let source = format!("for ({prefix}{initializer}; false; 'x' in o) {{}}");
            assert!(
                parse_script(&source).is_ok(),
                "{source}: {:?}",
                parse_script(&source)
            );
        }
    }
    assert!(parse_script("for (;;'x' in o) ; 'x' in o").is_ok());
}

#[test]
fn for_in_headers_distinguish_reference_targets_from_initializers() {
    for source in [
        "for (x in o) ;",
        "for (var x in o) ;",
        "for (let x in o) ;",
        "for (const x in o) ;",
        "for ((x) in o) ;",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    for source in [
        "for (x = 'x' in o) ;",
        "for (let x = 'x' in o) ;",
        "for (true ? false : 'x' in o) ;",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn in_requires_operands_and_an_unescaped_terminal() {
    for source in ["'x' in", "'x' in ;", r"'x' \u0069n o", "('x' in o) = 1"] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    assert!(parse_script("o.in; o['in']; 'x'\nin\no").is_ok());
    assert_eq!(
        parse_script("x instanceof").unwrap_err().kind,
        DiagnosticKind::Syntax
    );
}
