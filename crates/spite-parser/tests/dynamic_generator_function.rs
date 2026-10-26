//! Separate dynamic GeneratorFunction grammar goals (20.2.1.1.1, 15.5).

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{
    MAX_DEPTH,
    ast::{ExprKind, FunctionKind, Literal, StatementKind},
    parse_dynamic_generator_function, parse_dynamic_generator_function_utf16,
};

#[test]
fn generator_dynamic_source_retains_kind_standard_name_and_yield_syntax() {
    let function = parse_dynamic_generator_function("x=1,...tail", "yield x;yield* tail").unwrap();
    assert_eq!(function.kind, FunctionKind::Generator);
    assert_eq!(function.name.as_ref().unwrap().name, "anonymous");
    assert_eq!(
        function.source.as_str(),
        Some("function* anonymous(x=1,...tail\n) {\nyield x;yield* tail\n}")
    );
    assert_eq!(function.parameters.len(), 2);
    for (statement, expected) in function.body.statements().iter().zip([false, true]) {
        let StatementKind::Expression(expression) = &statement.kind else {
            panic!("yield statement")
        };
        assert!(matches!(expression.kind,ExprKind::Yield{delegated,..} if delegated==expected));
    }
    insta::assert_debug_snapshot!((
        function.kind,
        function.source.to_js_string(),
        function.body.is_strict()
    ));
}

#[test]
fn parameter_and_body_goals_reject_yield_bindings_and_wrapper_escapes() {
    for (parameters, body) in [
        ("yield", "yield 1"),
        ("yi\\u0065ld", "yield 1"),
        ("x=yield 1", "yield x"),
        ("{[yield 1]:x}", "yield x"),
        ("x) {}", "yield x"),
        ("x/*", "*/yield x"),
        ("x", "yield x;} evil();{"),
        ("x", "let x"),
        ("", "var yield"),
    ] {
        assert_eq!(
            parse_dynamic_generator_function(parameters, body)
                .unwrap_err()
                .kind,
            DiagnosticKind::Syntax,
            "{parameters:?}/{body:?}"
        );
    }
}

#[test]
fn strictness_and_parameter_early_errors_are_checked_on_combined_source() {
    assert!(
        !parse_dynamic_generator_function("a,a", "yield a")
            .unwrap()
            .body
            .is_strict()
    );
    assert!(
        parse_dynamic_generator_function("a", "'use strict';yield a")
            .unwrap()
            .body
            .is_strict()
    );
    for (parameters, body) in [
        ("a,a", "'use strict';yield a"),
        ("a=1", "'use strict';yield a"),
        ("a=1,a", "yield a"),
        ("a", "'use strict';yield 010"),
    ] {
        assert_eq!(
            parse_dynamic_generator_function(parameters, body)
                .unwrap_err()
                .kind,
            DiagnosticKind::Syntax,
            "{parameters:?}/{body:?}"
        );
    }
    assert!(parse_dynamic_generator_function("a", "yield 010").is_ok());
}

#[test]
fn utf16_parameters_operands_and_retained_source_preserve_lone_surrogates() {
    let mut parameters: Vec<u16> = "x='".encode_utf16().collect();
    parameters.push(0xd800);
    parameters.extend("'".encode_utf16());
    let mut body: Vec<u16> = "yield '".encode_utf16().collect();
    body.push(0xdfff);
    body.extend("'".encode_utf16());
    let mut expected: Vec<u16> = "function* anonymous(".encode_utf16().collect();
    expected.extend_from_slice(&parameters);
    expected.extend("\n) {\n".encode_utf16());
    expected.extend_from_slice(&body);
    expected.extend("\n}".encode_utf16());
    let function = parse_dynamic_generator_function_utf16(
        &JsString::from_code_units(parameters),
        &JsString::from_code_units(body),
    )
    .unwrap();
    assert_eq!(
        function.source.to_js_string(),
        JsString::from_code_units(expected)
    );
    assert_eq!(function.source.as_str(), None);
    let StatementKind::Expression(expression) = &function.body.statements()[0].kind else {
        panic!("yield statement")
    };
    let ExprKind::Yield {
        argument: Some(argument),
        ..
    } = &expression.kind
    else {
        panic!("yield operand")
    };
    assert_eq!(
        argument.kind,
        ExprKind::Literal(Literal::String(JsString::from_code_units(vec![0xdfff])))
    );
    assert_eq!(
        function.parameters[0]
            .binding()
            .initializer
            .as_ref()
            .unwrap()
            .kind,
        ExprKind::Literal(Literal::String(JsString::from_code_units(vec![0xd800])))
    );
}

#[test]
fn nested_functions_methods_await_identifiers_and_regexp_goals_keep_their_contexts() {
    for body in [
        "yield /r/.test('r')",
        "yield\n/r/.test('r')",
        "yield* []",
        "function f(yield){return yield};yield f",
        "yield ()=>yield",
        "yield { *g(){yield super.x} }",
        "yield new.target",
        "yield await",
    ] {
        assert!(
            parse_dynamic_generator_function("await", body).is_ok(),
            "{body}: {:?}",
            parse_dynamic_generator_function("await", body)
        );
    }
    for body in ["yield super.x", "super()", "yield await value"] {
        assert_eq!(
            parse_dynamic_generator_function("", body).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{body}"
        );
    }
}

#[test]
fn nested_async_generators_parse_while_native_depth_limits_stay_distinct() {
    assert!(parse_dynamic_generator_function("", "async function*g(){}").is_ok());
    let body = format!(
        "yield {}0{}",
        "(".repeat(MAX_DEPTH + 1),
        ")".repeat(MAX_DEPTH + 1)
    );
    assert_eq!(
        parse_dynamic_generator_function("", &body)
            .unwrap_err()
            .kind,
        DiagnosticKind::Limit
    );
    let parameters = format!(
        "x={}0{}",
        "(".repeat(MAX_DEPTH + 1),
        ")".repeat(MAX_DEPTH + 1)
    );
    assert_eq!(
        parse_dynamic_generator_function(&parameters, "yield 1")
            .unwrap_err()
            .kind,
        DiagnosticKind::Limit
    );
}
