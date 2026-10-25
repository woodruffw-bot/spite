//! Async arrow cover selection, lexical contexts and source retention.

use spite_core::DiagnosticKind;
use spite_parser::parse_script;

#[test]
fn async_arrow_syntax_snapshot_retains_prefix_defaults_and_expression_bodies() {
    insta::assert_debug_snapshot!(
        parse_script("let f=async /*keep*/ (x=1,...rest)=>({x,rest});").unwrap()
    );
}

#[test]
fn arrow_heads_preserve_async_identifier_calls_newlines_and_lexical_context() {
    for source in [
        "async x=>x",
        "async()=>1",
        "async(a,b=1,...rest)=>a+b",
        "async ({await:x})=>x",
        "async=>async",
        "async\nx=>x",
        "let async=x=>x,await=8;async(await/2)",
        "let async=x=>x;async(x=>x)",
        "async()=>()=>await",
        "function f(){return async()=>[this,arguments,new.target]}",
        "({m(){return async()=>super.m()}})",
        "class C extends B{m(){return async()=>super.m()}static{let f=async()=>1}}",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
}

#[test]
fn await_parameters_duplicates_and_line_terminators_are_syntax_errors() {
    for source in [
        "async await=>1",
        "async(await)=>1",
        "async(\\u0061wait)=>1",
        "async(x=await p)=>x",
        "async(a,a)=>a",
        "async(a,a)=>{'use strict'}",
        "async(a=1)=>{'use strict'}",
        "async x\n=>x",
        "async(x)\n=>x",
        "async()=>{var await}",
        "async()=>{var \\u0061wait}",
        "async()=>new.target",
        "async()=>super.x",
        "class C{static{let f=async()=>arguments}}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn async_arrow_await_bodies_parse_and_report_later_lexical_errors() {
    for source in ["async x=>await x", "async()=>{return await p}"] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    assert_eq!(
        parse_script("async()=>{await p; '\\u{110000}'")
            .unwrap_err()
            .kind,
        DiagnosticKind::Syntax
    );
}
