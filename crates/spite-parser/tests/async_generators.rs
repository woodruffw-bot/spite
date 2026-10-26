//! Combined Await/Yield grammar contexts and early errors (15.6).

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{
    MAX_DEPTH,
    ast::{FunctionKind, StatementKind},
    parse_dynamic_async_generator_function, parse_dynamic_async_generator_function_utf16,
    parse_module, parse_script,
};

#[test]
fn async_generator_declarations_retain_kind_source_and_await_yield_nodes() {
    let source = "async /* retained */ function* g(first=1,...rest){yield await first;yield* rest;return await 0}";
    let script = parse_script(source).unwrap();
    let function = script.function_declarations()[0];
    assert_eq!(function.kind, FunctionKind::AsyncGenerator);
    assert_eq!(function.source.as_str(), Some(source));
    assert_eq!(function.parameters.len(), 2);
    insta::assert_debug_snapshot!(script);
}

#[test]
fn object_class_static_computed_and_private_async_generator_methods_parse() {
    for source in [
        "({async *g(){yield await 1}})",
        "({async *['g'](){yield* source}})",
        "class C{async *g(){yield await 1}}",
        "class C{static async *g(){for await(const x of source)yield x}}",
        "class C{async *#g(){yield this.#g}static async *#s(){yield await 1}}",
        "class C extends B{async *g(x=super.x){yield super.g();return new.target}}",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
    for source in [
        "({async *g(a,a){}})",
        "class C{async *constructor(){}}",
        "class C{static async *prototype(){}}",
        "class C{async *g;}",
        "({async *g(){super()}})",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn async_generator_parameters_forbid_both_await_and_yield_while_strictness_stays_independent() {
    for source in [
        "async function*g(await){}",
        "async function*g(yield){}",
        "async function*g(x=await 1){}",
        "async function*g(x=yield 1){}",
        "async function*g({x=await 1}){}",
        "async function*g([x=yield 1]){}",
        "async function*g(a,a){'use strict'}",
        "'use strict';async function*g(a,a){}",
        "async function*g(x=1){'use strict'}",
        "async function*g(a){let a}",
        "(async function* await(){})",
        "(async function* yield(){})",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    // Edition 17 applies UniqueFormalParameters to declarations/expressions
    // only in strict code; methods always use UniqueFormalParameters (15.6.1).
    for source in [
        "async function*g(a,a){yield a}",
        "async function*await(){}",
        "async function*yield(){}",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}

#[test]
fn await_and_yield_contexts_restore_across_nested_functions_and_restricted_productions() {
    for source in [
        "async function*g(){yield await 1;await(yield 2);yield*await source}",
        "async function*g(){function f(await,yield){return await+yield}yield await 1}",
        "async function*g(){const f=async()=>await 1;const g=function*(){yield 2};yield await f()}",
        "async function*g(){function f(){return new.target}yield {await:1,yield:2}}",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
    for source in [
        "async function*g(){await yield 1}",
        "async function*g(){function f(){await 1}}",
        "async function*g(){function f(){yield 1}}",
        "({async\n*g(){}})",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    let script = parse_script("async\nfunction*g(){}").unwrap();
    assert_eq!(
        script.function_declarations()[0].kind,
        FunctionKind::Generator
    );
    assert!(parse_script("class C{async\n*g(){}}").is_ok());
}

#[test]
fn dynamic_async_generator_goals_keep_parameters_body_and_exact_source_separate() {
    let function =
        parse_dynamic_async_generator_function("first=1,...rest", "yield await first;yield*rest")
            .unwrap();
    assert_eq!(function.kind, FunctionKind::AsyncGenerator);
    assert_eq!(
        function.source.as_str(),
        Some("async function* anonymous(first=1,...rest\n) {\nyield await first;yield*rest\n}")
    );
    for (parameters, body) in [
        ("await", ""),
        ("yield", ""),
        ("x=await 1", ""),
        ("x=yield 1", ""),
        ("x=1", "'use strict'"),
        ("/*", "*/ yield 1"),
        ("x)", "yield 1"),
    ] {
        assert_eq!(
            parse_dynamic_async_generator_function(parameters, body)
                .unwrap_err()
                .kind,
            DiagnosticKind::Syntax,
            "{parameters} / {body}"
        );
    }
    assert!(parse_dynamic_async_generator_function("x// comment", "yield await x").is_ok());
}

#[test]
fn dynamic_async_generator_utf16_source_and_native_depth_limits_remain_lossless_and_explicit() {
    let mut units: Vec<_> = "yield '".encode_utf16().collect();
    units.push(0xd800);
    units.extend("';await 1".encode_utf16());
    let body = JsString::from_code_units(units);
    let function =
        parse_dynamic_async_generator_function_utf16(&JsString::from("x"), &body).unwrap();
    assert_eq!(function.kind, FunctionKind::AsyncGenerator);
    assert!(function.source.as_str().is_none());
    assert!(
        function
            .source
            .to_js_string()
            .code_units()
            .contains(&0xd800)
    );
    let source = format!(
        "async function*g(){{yield {}0{}}}",
        "(".repeat(MAX_DEPTH * 4),
        ")".repeat(MAX_DEPTH * 4)
    );
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}

#[test]
fn module_named_and_default_async_generator_exports_retain_strict_bindings() {
    let module=parse_module("export async function*g(){yield await 1};export default async function*(){for await(const x of source)yield x}").unwrap();
    assert!(
        module
            .statements()
            .iter()
            .filter_map(|s| match &s.kind {
                StatementKind::Function(f) => Some(f),
                _ => None,
            })
            .all(|f| f.kind == FunctionKind::AsyncGenerator)
    );
    let entries = module.entries();
    assert_eq!(entries.local_export_entries[0].local, JsString::from("g"));
    assert_eq!(
        entries.local_export_entries[1].local,
        JsString::from("*default*")
    );
    assert_eq!(
        parse_module("export default async function*await(){}")
            .unwrap_err()
            .kind,
        DiagnosticKind::Syntax
    );
}
