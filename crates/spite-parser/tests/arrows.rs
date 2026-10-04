//! Simple arrow parameters, assignment-expression bodies, and exact source text.

use spite_core::DiagnosticKind;
use spite_parser::{
    MAX_DEPTH,
    ast::{BinaryOp, ExprKind, StatementKind},
    parse_script,
};

#[test]
fn arrow_syntax_snapshot() {
    insta::assert_debug_snapshot!(
        parse_script("let f = (x, y,) => x + y; let g = x => y => ({x, y}); f(1, 2);").unwrap()
    );
}

#[test]
fn supported_parameters_bodies_and_in_contexts_parse() {
    for source in [
        "()=>1",
        "x=>x",
        "(x)=>x",
        "(x,y,)=>x+y",
        "x => y => x + y",
        "(x=>x)(1)",
        "let f = x => x = 1",
        "a ? x=>x : y=>y",
        "f(x=>x, y=>y)",
        "({a: x => x, [1]: () => 2})",
        "x=>({x})",
        "x=>`${x}`",
        "x => x ? y : z",
        "x => (a,b)",
        "x => x in o",
        "for (let f = x => (x in o); false;) ;",
        "for (f(x => x in o); false;) ;",
        "(async, await, yield, eval, arguments) => async",
        "async => async",
        "let => let",
        "(let) => let",
        "yield => yield",
        "await => await",
        "async\nx => x",
        "(\u{03c0}, \\u0061) => \u{03c0} + a",
        "(x /* newline\n */) => x",
        "x =>\nx",
        "x /*comment*/ => x",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
}

#[test]
fn bodies_bind_assignment_but_not_outer_comma_and_preserve_parameter_source() {
    let script = parse_script("x=>x, y").unwrap();
    let StatementKind::Expression(expression) = &script.statements()[0].kind else {
        panic!("expression")
    };
    let ExprKind::Binary(BinaryOp::Comma, left, _) = &expression.kind else {
        panic!("outer comma")
    };
    assert!(matches!(left.kind, ExprKind::Arrow { .. }));
    let input = String::from("let f = ((x /* retained */ , y) => (x + y));");
    let parsed = parse_script(&input).unwrap();
    drop(input);
    let StatementKind::Lexical { bindings, .. } = &parsed.statements()[0].kind else {
        panic!("declaration")
    };
    let ExprKind::Parenthesized(inner) = &bindings[0].initializer.as_ref().unwrap().kind else {
        panic!("parentheses")
    };
    let ExprKind::Arrow { source, .. } = &inner.kind else {
        panic!("arrow")
    };
    assert_eq!(source.as_str(), "(x /* retained */ , y) => (x + y)");
}

#[test]
fn invalid_parameters_duplicate_names_and_strict_bindings_are_early_errors() {
    for source in [
        "(x,x)=>x",
        "(a,\\u0061)=>a",
        "((x))=>x",
        "(x+y)=>x",
        "(,)=>1",
        "(x,,)=>x",
        "x\n=>x",
        "(x)\n=>x",
        "x /*\n*/ => x",
        "1=>1",
        "(true)=>1",
        "(for)=>1",
        "1 + x => x",
        "a || x => x",
        "(()=>1) = 2",
        "()=>",
        "x=>x +",
        "'use strict'; (eval)=>eval",
        "'use strict'; arguments=>1",
        "'use strict'; yield=>1",
        "'use strict'; x => eval = 1",
        "'use strict'; x => 010",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn unimplemented_parameter_and_body_forms_remain_explicit_gaps() {
    for source in [
        "(...[xs])=>xs",
        "([x])=>x",
        "({x})=>x",
        "async x=>x",
        "async (x)=>x",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
}

#[test]
fn arrow_diagnostics_snapshot() {
    let errors: Vec<_> = [
        "(x,x)=>x",
        "x\n=>x",
        "'use strict'; eval=>eval",
        "(...[xs])=>xs",
    ]
    .into_iter()
    .map(|source| parse_script(source).unwrap_err())
    .collect();
    insta::assert_debug_snapshot!(errors);
}

#[test]
fn nested_arrows_are_bounded() {
    let source = format!("{}1", "x=>".repeat(MAX_DEPTH * 2));
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}
