//! Ordinary function expression grammar, source retention, and early errors.

use spite_core::DiagnosticKind;
use spite_parser::{
    MAX_DEPTH,
    ast::{ExprKind, StatementKind},
    parse_script,
};

#[test]
fn ordinary_function_expression_snapshot() {
    insta::assert_debug_snapshot!(
        parse_script("let f = function named(x = 1) { return x; };").unwrap()
    );
}

#[test]
fn anonymous_named_and_default_parameter_functions_parse_in_expression_positions() {
    for source in [
        "(function(){})",
        "let f=function named(){}",
        "let f=function(a,b,){return a+b;}",
        "(function(a,a){return a;})",
        "(function named(named){return named;})",
        "(function eval(arguments){return arguments;})",
        "(function yield(await){return await;})",
        "(function(a=1,b=a){var a;return b;})",
        "'use strict';(function(a=1){return a;})",
        "(function(){'use strict';return 1;})",
        "(function(){return function(){return 1;};})",
        "()=>function(){return 1;}",
        "({f:function(){return 1;}})",
        "(function(){})()",
        "let f=function(){}.name",
        "let f=function(){}()",
        "for(let f=function(a='x' in o){return 'x' in o;};false;);",
        "(function(a=function(a,a){return a}){return a;})",
        "(function f(){let f; return f;})",
        "(function f(){var f;return f;})",
        "(function(){outer:while(true){continue outer;}})",
        "outer:while(true){let f=function(){outer:while(true){break outer;}};break;}",
        "(function(){'use\\x20strict';var eval;})",
        "(function(){('use strict');var eval;})",
        "(function(a,a){'use\\x20strict';})",
        "(function \\u0066(\\u0061){return a;})",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
}

#[test]
fn strict_and_non_simple_lists_reject_duplicate_parameters() {
    for source in [
        "'use strict';(function(a,a){})",
        "(function(a,a){'use strict';})",
        "(function(a=1,a){})",
        "(function(a,a=1){})",
        "(function(a=1,a=2){})",
        "(function(a,\\u0061){'use strict';})",
        "(function(a=1){'use strict';})",
        "(function eval(){'use strict';})",
        "(function arguments(){'use strict';})",
        "'use strict';(function eval(){})",
        "(function(eval){'use strict';})",
        "(function(arguments){'use strict';})",
        "(function(package){'use strict';})",
        "(function(){'use strict';var eval;})",
        "(function(){'use strict';010;})",
        "(function(){'\\1';'use strict';})",
        "'use strict';(function(a=010){})",
        "(function(){'use strict';return function(a,a){};})",
        "'use strict';(function(a=function(a,a){}){})",
        "(function(){'use strict';return function eval(){};})",
        "(function(x){let x;})",
        "(function(x){const x=1;})",
        "(function(){let x;var x;})",
        "(function(){var x;{let y;var y;}})",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn function_boundaries_reset_control_targets_and_grammar_restrictions() {
    for source in [
        "while(1){let f=function(){break;};}",
        "outer:while(1){let f=function(){continue outer;};}",
        "let f=function(){};return 1;",
        "(function(){return 1;});return 2;",
        "(function(){})=1",
        "(function(){})++",
        "(function enum(){})",
        "(function true(){})",
        "(function 1(){})",
        "(function(a,,){})",
        "(function(a=){})",
        "(function(a) return a)",
        "(function(){return;)",
        "(\\u0066unction(){})",
        "(function(a+b){})",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    for source in ["(function*(){})", "(function(...a){})", "(function([a]){})"] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
}

#[test]
fn source_retention_and_var_collection_respect_function_boundaries() {
    let source = String::from(
        "var outer; let f = (function /*a*/ \\u0066(x /*b*/) { var inner; return x; });",
    );
    let script = parse_script(&source).unwrap();
    drop(source);
    let StatementKind::Lexical { bindings, .. } = &script.statements()[1].kind else {
        panic!()
    };
    let ExprKind::Parenthesized(expr) = &bindings[0].initializer.as_ref().unwrap().kind else {
        panic!()
    };
    let ExprKind::Function(function) = &expr.kind else {
        panic!()
    };
    assert_eq!(function.name.as_ref().unwrap().name, "f");
    assert_eq!(
        function.source.as_str(),
        "function /*a*/ \\u0066(x /*b*/) { var inner; return x; }"
    );
    assert_eq!(
        script
            .var_declarations()
            .iter()
            .map(|b| b.name.as_str())
            .collect::<Vec<_>>(),
        ["outer"]
    );
    assert_eq!(
        function
            .body
            .var_declarations()
            .iter()
            .map(|b| b.name.as_str())
            .collect::<Vec<_>>(),
        ["inner"]
    );
}

#[test]
fn nested_function_bodies_and_defaults_are_bounded() {
    for source in [
        format!(
            "({}0{})",
            "function(){{return ".repeat(MAX_DEPTH),
            ";}}".repeat(MAX_DEPTH)
        ),
        format!(
            "({}1{})",
            "function(x=".repeat(MAX_DEPTH),
            "){return x;}".repeat(MAX_DEPTH)
        ),
    ] {
        assert_eq!(
            parse_script(&source).unwrap_err().kind,
            DiagnosticKind::Limit
        );
    }
}
