//! Independent dynamic AsyncFunction grammar goals and retained UTF-16 source.

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{
    MAX_DEPTH, ast::FunctionKind, parse_dynamic_async_function, parse_dynamic_async_function_utf16,
};

#[test]
fn async_dynamic_source_retains_kind_standard_name_and_independent_strictness() {
    let function = parse_dynamic_async_function("x=1", "return x").unwrap();
    assert_eq!(function.kind, FunctionKind::Async);
    assert_eq!(
        function.source.as_str(),
        Some("async function anonymous(x=1\n) {\nreturn x\n}")
    );
    assert!(!function.body.is_strict());
    assert!(parse_dynamic_async_function("a,a", "return a").is_ok());
    assert!(
        parse_dynamic_async_function("a", "'use strict';return a")
            .unwrap()
            .body
            .is_strict()
    );
    insta::assert_debug_snapshot!(function);
}

#[test]
fn separate_parameter_and_body_goals_enforce_await_early_errors_and_source_boundaries() {
    for (parameters, body) in [
        ("await", "return 1"),
        ("\\u0061wait", "return 1"),
        ("x=await p", "return x"),
        ("a,a", "'use strict';return a"),
        ("a=1", "'use strict';return a"),
        ("a) {}", "return a"),
        ("a/*", "*/ return a"),
        ("a", "return a; } evil(); {"),
        ("", "var await"),
        ("a", "let a"),
    ] {
        assert_eq!(
            parse_dynamic_async_function(parameters, body)
                .unwrap_err()
                .kind,
            DiagnosticKind::Syntax,
            "{parameters:?} / {body:?}"
        );
    }
}

#[test]
fn utf16_source_and_literal_contents_preserve_lone_surrogates() {
    let mut body: Vec<u16> = "return '".encode_utf16().collect();
    body.push(0xd800);
    body.extend("'".encode_utf16());
    let function =
        parse_dynamic_async_function_utf16(&JsString::from("x"), &JsString::from_code_units(body))
            .unwrap();
    let mut expected: Vec<u16> = "async function anonymous(x\n) {\nreturn '"
        .encode_utf16()
        .collect();
    expected.push(0xd800);
    expected.extend("'\n}".encode_utf16());
    assert_eq!(
        function.source.to_js_string(),
        JsString::from_code_units(expected)
    );
    assert_eq!(function.source.as_str(), None);
}

#[test]
fn awaited_bodies_parse_while_generators_and_native_depth_limits_remain_distinct() {
    for body in ["await p", "return await p", "return async()=>await p"] {
        assert!(parse_dynamic_async_function("", body).is_ok(), "{body}");
    }
    let body = "async function* g(){}";
    assert_eq!(
        parse_dynamic_async_function("", body).unwrap_err().kind,
        DiagnosticKind::Unsupported,
        "{body}"
    );
    let body = format!(
        "return {}0{};",
        "(".repeat(MAX_DEPTH + 1),
        ")".repeat(MAX_DEPTH + 1)
    );
    assert_eq!(
        parse_dynamic_async_function("", &body).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}
