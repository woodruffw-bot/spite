//! Member references, assignment targets, precedence, and line-terminator rules.

use spite_core::DiagnosticKind;
use spite_parser::{
    MAX_DEPTH,
    ast::{ExprKind, StatementKind},
    parse_script,
};

#[test]
fn member_reference_snapshot() {
    insta::assert_debug_snapshot!(
        parse_script("a.b[c].if += ++d[e]; delete (a.x); a[k] ??= 4;").unwrap()
    );
}

#[test]
fn identifier_names_computed_expressions_and_parenthesized_targets_parse() {
    for source in [
        "a.b",
        "a['b']",
        "a[b, c]",
        "a[b = 1]",
        "a[b ? c : d]",
        "a.if.null.true.false",
        r"a.\u0069f",
        "a.let.yield.await",
        "({x: 1}).x",
        "1..x",
        "1e0.x",
        "0x1.x",
        "1n.x",
        "'s'[0]",
        "null.x",
        "a.b = c.d = 1",
        "(a.b) = 1",
        "((a[k])) += 1",
        "a.b &&= 2",
        "a.b++",
        "--a[b]",
        "++a.b ** 2",
        "(a++).x",
        "(-a).x",
        "typeof a.b",
        "delete a[b]",
        "'use strict'; delete a.eval",
        "'use strict'; eval.x = 1; arguments.x++; a.implements = 2",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
}

#[test]
fn access_continues_across_newlines_when_grammar_allows_it() {
    for source in ["a\n.b", "a\n[b]", "a\n[b]++", "a.x\n[k]", "a/*\n*/.b"] {
        assert_eq!(
            parse_script(source).unwrap().statements().len(),
            1,
            "{source}"
        );
    }
    for source in [
        "a++\n(b)",
        "a--\n`x`",
        "-a++\n(b)",
        "a.b\n++c.d",
        "a++\n[0]",
    ] {
        assert_eq!(
            parse_script(source).unwrap().statements().len(),
            2,
            "{source}"
        );
    }
}

#[test]
fn member_access_binds_more_tightly_than_unary_update_and_binary_operators() {
    let script = parse_script("-a.b + c[d] ** e.x++").unwrap();
    let StatementKind::Expression(expr) = &script.statements()[0].kind else {
        panic!("expression")
    };
    let ExprKind::Binary(_, left, right) = &expr.kind else {
        panic!("addition")
    };
    let ExprKind::Unary(_, member) = &left.kind else {
        panic!("unary")
    };
    assert!(matches!(member.kind, ExprKind::Member(..)));
    let ExprKind::Binary(_, base, exponent) = &right.kind else {
        panic!("exponentiation")
    };
    assert!(matches!(base.kind, ExprKind::Member(..)));
    let ExprKind::Update { argument, .. } = &exponent.kind else {
        panic!("update")
    };
    assert!(matches!(argument.kind, ExprKind::Member(..)));
}

#[test]
fn malformed_access_and_invalid_targets_are_syntax_errors() {
    for source in [
        "a.",
        "a[]",
        "a[b",
        "a.'x'",
        "a.1",
        "a..b",
        "a[;]",
        "a++.x",
        "a++[x]",
        "a++(x)",
        "a++`x`",
        "a++\n.x",
        "(a.b + c) = 1",
        "a.b++ = 1",
        "++a.b++",
        "'use strict'; a[yield]",
        "'use strict'; (eval) = 1",
        "'use strict'; a[arguments = 1]",
        "'use strict'; delete (eval)",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn optional_chains_and_tagged_templates_remain_unsupported() {
    for source in ["a?.b", "a?.[b]", "a[b]`x`"] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
}

#[test]
fn flat_and_computed_member_depth_is_bounded() {
    for source in [
        format!("a{}", ".x".repeat(MAX_DEPTH * 2)),
        format!(
            "{}a{}",
            "a[".repeat(MAX_DEPTH * 2),
            "]".repeat(MAX_DEPTH * 2)
        ),
    ] {
        assert_eq!(
            parse_script(&source).unwrap_err().kind,
            DiagnosticKind::Limit
        );
    }
}
