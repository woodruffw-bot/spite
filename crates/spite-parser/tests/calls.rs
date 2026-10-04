//! Call precedence, argument syntax, and retained callee references.

use spite_core::DiagnosticKind;
use spite_parser::{MAX_DEPTH, parse_script};

#[test]
fn call_syntax_snapshot() {
    insta::assert_debug_snapshot!(parse_script("(obj.method)(1, x = 2,).next()[key]();").unwrap());
}

#[test]
fn spread_call_and_constructor_arguments_snapshot() {
    insta::assert_debug_snapshot!(
        parse_script("obj.f(1,...items,...make(), x=y,);new F(...(a,b),...items,)").unwrap()
    );
    insta::assert_debug_snapshot!(parse_script("f(...,)").unwrap_err());
}

#[test]
fn calls_chain_with_members_and_accept_assignment_expressions() {
    for source in [
        "f()",
        "f(a, b,)",
        "f(...x)",
        "f(a,...x,b,...y,)",
        "f(...x=y)",
        "f(...(a,b))",
        "f((a, b))",
        "f(a = 1, b ? c : d)",
        "f()()",
        "f().x()",
        "f()[x]().y",
        "f().x = 1",
        "f().x++",
        "(f)()",
        "(0, obj.f)()",
        "(obj.f)()",
        "(x++)(y)",
        "1\n(2)",
        "f\n(a)",
        "-f().x",
        "f() ** 2",
        "for (f('x' in o); false; f()) ;",
        "'use strict'; f(eval, arguments, {yield: 1})",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
    assert_eq!(parse_script("x++\n(y)").unwrap().statements().len(), 2);
}

#[test]
fn malformed_arguments_and_call_targets_are_rejected() {
    for source in [
        "f(",
        "f(,)",
        "f(a,,b)",
        "f(a b)",
        "f(a; b)",
        "f() = 1",
        "f()++",
        "++f()",
        "'use strict'; f(eval = 1)",
        "'use strict'; f(yield)",
        "f(...)",
        "f(...,)",
        "f(...x,,y)",
        "f(...x ...y)",
        "'use strict'; f(...eval=1)",
        "'use strict'; f(...yield)",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    assert_eq!(
        parse_script("f?.()").unwrap_err().kind,
        DiagnosticKind::Unsupported
    );
}

#[test]
fn nested_and_flat_call_depth_is_bounded() {
    for source in [
        format!("f{}", "()".repeat(MAX_DEPTH * 2)),
        format!(
            "{}0{}",
            "f(".repeat(MAX_DEPTH * 2),
            ")".repeat(MAX_DEPTH * 2)
        ),
        format!(
            "{}[]{}",
            "f(...".repeat(MAX_DEPTH * 2),
            ")".repeat(MAX_DEPTH * 2)
        ),
    ] {
        assert_eq!(
            parse_script(&source).unwrap_err().kind,
            DiagnosticKind::Limit
        );
    }
}
