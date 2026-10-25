//! Async function declarations/expressions before await suspension support.

use spite_core::DiagnosticKind;
use spite_parser::{ast::FunctionKind, parse_dynamic_function, parse_script};

#[test]
fn declarations_retain_async_kind_parameters_and_exact_source() {
    let source = "async /* retained */ function f(x=1,...rest){return [x,rest]}";
    let script = parse_script(source).unwrap();
    let function = script.function_declarations()[0];
    assert_eq!(function.kind, FunctionKind::Async);
    assert_eq!(function.source.as_str(), Some(source));
    assert_eq!(function.parameters.len(), 2);
    insta::assert_debug_snapshot!(script);
}

#[test]
fn async_functions_keep_sloppy_strict_and_nested_await_grammar_boundaries() {
    for source in [
        "async function f(a,a){return a}",
        "async function await(){}",
        "let f=async function named(x=1){return x}",
        "(async function(){return this})",
        "async function f(){function g(await){return await}return ()=>await}",
        "async function f(){let x={m(await){return await}};class C{m(await){return await}}}",
        "async function f(){let g=function await(){return 1};return g}",
        "async function f(){return{await:1}}",
        "async function f(){return class{static{let x=()=>await}}}",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
    let script = parse_script("async\nfunction f(){}").unwrap();
    assert_eq!(script.statements().len(), 2);
    assert_eq!(
        script.function_declarations()[0].kind,
        FunctionKind::Ordinary
    );
}

#[test]
fn binding_parameters_strictness_and_declaration_positions_report_syntax_errors() {
    for source in [
        "async function f(await){}",
        "async function f({await}){}",
        "async function f(x=await y){}",
        "async function f(){var await}",
        "async function f(){var \\u0061wait}",
        "async function f(){function await(){}}",
        "async function f(){async function await(){}}",
        "let f=async function await(){}",
        "'use strict';async function f(a,a){}",
        "async function f(a,a){'use strict'}",
        "async function f(a=1,a){}",
        "async function f(a=1){'use strict'}",
        "async function f(a){let a}",
        "if(true) async function f(){}",
        "async function(){}",
        "\\u0061sync function f(){}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn async_generators_remain_explicitly_unsupported() {
    for source in [
        "async function* f(){}",
        "async function f(){await using x=null;}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
}

#[test]
fn ordinary_dynamic_functions_can_retain_unexecuted_async_definitions() {
    for (parameters, body) in [
        ("", "async function f(){await p}"),
        ("x=(async function(){await p})", "return x"),
    ] {
        let function = parse_dynamic_function(parameters, body).unwrap();
        assert_eq!(function.kind, FunctionKind::Ordinary);
    }
}
