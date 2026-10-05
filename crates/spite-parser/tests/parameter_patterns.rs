//! Formal binding patterns, ContainsExpression, and non-simple early errors (15.1).

use spite_core::DiagnosticKind;
use spite_parser::{MAX_DEPTH, ast::StatementKind, parse_dynamic_function, parse_script};

#[test]
fn parameter_pattern_syntax_snapshot() {
    insta::assert_debug_snapshot!(
        parse_script("function f({[key]:[x=1],...rest}={},...[y]){return x+y;}").unwrap()
    );
}

#[test]
fn parameter_pattern_diagnostics_snapshot() {
    insta::assert_debug_snapshot!(
        [
            "function f([x],{a:x}){}",
            "([x])=>{'use strict';}",
            "function f({a:[x]}){let x;}",
            "function f(...[x],){}",
            "function f(...{x}={}){}",
            "'use strict';function f({a:eval}){}",
        ]
        .map(|source| parse_script(source).unwrap_err())
    );
}

#[test]
fn every_supported_function_form_accepts_nested_and_rest_patterns() {
    for parameters in [
        "{}",
        "[]",
        "{a:[x=key in source],...rest}",
        "[,,...{length:x}]",
        "[x]=[],{y}={}",
        "...[x,...rest]",
        "...{length,x}",
    ] {
        for source in [
            format!("function f({parameters}){{}}"),
            format!("(function({parameters}){{}})"),
            format!("({parameters})=>0"),
            format!("({{m({parameters}){{}}}})"),
            format!("'use strict';function f({parameters}){{}}"),
        ] {
            assert!(
                parse_script(&source).is_ok(),
                "{source}: {:?}",
                parse_script(&source)
            );
        }
        assert!(parse_dynamic_function(parameters, "return 1;").is_ok());
    }
    assert!(parse_script("({set x({a:[b=1]}={}){}})").is_ok());
    assert!(parse_script("function f([x]){var x;function x(){}} ").is_ok());
}

#[test]
fn all_bound_names_participate_in_duplicate_strict_and_body_conflicts() {
    for source in [
        "function f([x,x]){}",
        "function f({x,a:x}){}",
        "function f([x],x){}",
        "function f(x,...[x]){}",
        "([x],x)=>0",
        "({m({x},x){}})",
        "function f([x]){'use strict';}",
        "({set x([x]){'use strict';}})",
        "'use strict';function f({arguments}){}",
        "'use strict';([eval])=>0",
        "'use strict';function f({[eval=1]:x}){}",
        "'use strict';function f([x=delete y]){}",
        "function f(...[x]=[]){}",
        "function f(...{},x){}",
        "function f([...[x],]){}",
        "function f({a:[x],...rest}){const rest=1;}",
        "([x])=>{let {a:x}={};}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    assert!(parse_script("function f(x,x){}").is_ok());
}

#[test]
fn parameter_static_semantics_distinguish_nested_expressions_from_top_level_defaults() {
    let script = parse_script("function f(x,{},[y],{z=1},{[key]:w},...[r=2]){}").unwrap();
    let StatementKind::Function(function) = &script.statements()[0].kind else {
        panic!("function")
    };
    assert_eq!(
        function
            .parameters
            .iter()
            .map(|p| p.is_simple())
            .collect::<Vec<_>>(),
        [true, false, false, false, false, false]
    );
    assert_eq!(
        function
            .parameters
            .iter()
            .map(|p| p.contains_expression())
            .collect::<Vec<_>>(),
        [false, false, false, true, true, true]
    );
    assert_eq!(
        function
            .parameters
            .iter()
            .map(|p| p.counts_toward_length())
            .collect::<Vec<_>>(),
        [true, true, true, true, true, false]
    );
    assert_eq!(
        function
            .parameters
            .iter()
            .flat_map(|p| p.binding().pattern.bound_names())
            .map(|(name, _)| name)
            .collect::<Vec<_>>(),
        ["x", "y", "z", "w", "r"]
    );
}

#[test]
fn nested_parameter_patterns_share_the_native_stack_guard() {
    let source = format!(
        "function f({}x{}){{}}",
        "[".repeat(MAX_DEPTH),
        "]".repeat(MAX_DEPTH)
    );
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}
