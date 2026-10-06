//! Iterative left-associated BinaryExpression syntax (13.6–13.13, 13.16).

use spite_core::DiagnosticKind;
use spite_parser::{
    ast::{ExprKind, StatementKind},
    parse_script,
};

#[test]
fn binary_grouping_and_prefix_spans_snapshot() {
    let mut rows = String::new();
    for source in [
        "1+2+3",
        "1+2*3+4",
        "2**3**2+4",
        "1+2-3<<4",
        "false&&a&&b||c",
        "null??a??b",
        "(a+b)+c",
        "a+(b+c)+d",
        "a,b,c",
        "a in b in c === d",
    ] {
        let script = parse_script(source).unwrap();
        rows.push_str(&format!("{source}\n{:#?}\n", script.statements()[0]));
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn long_flat_chains_parse_clone_compare_and_drop_without_native_recursion() {
    let source = format!("{}0", "0+".repeat(100_000));
    let script = parse_script(&source).unwrap();
    let StatementKind::Expression(expression) = &script.statements()[0].kind else {
        panic!("expression")
    };
    let ExprKind::BinaryChain { steps, .. } = &expression.kind else {
        panic!("chain")
    };
    assert_eq!(steps.len(), 100_000);
    assert_eq!(steps.last().unwrap().span.end, source.len());
    let clone = expression.clone();
    assert_eq!(&clone, expression);
    drop(clone);
    drop(script);
}

#[test]
fn nullish_mixing_and_strict_errors_still_inspect_flat_chain_operands() {
    for source in [
        "a||b||c??d",
        "a??b??c||d",
        "a&&b&&c??d",
        "'use strict';1+2+(delete x)",
        "'use strict';1+2+(eval=3)",
        "'use strict';1+2+(arguments++)",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    for source in ["(a||b||c)??d", "a??(b&&c&&d)", "(a??b??c)||d"] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}
