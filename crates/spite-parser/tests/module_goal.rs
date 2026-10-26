//! Module goal strictness, lexical functions, Await and ImportMeta (16.2, 13.1.1).

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{
    ast::{ExprKind, Literal, StatementKind},
    parse_dynamic_function, parse_module, parse_module_utf16, parse_module_with_source_limit,
    parse_script,
};

#[test]
fn module_ast_retains_import_meta_top_level_await_and_source_spans() {
    let module = parse_module("const meta=import.meta;await meta").unwrap();
    let StatementKind::Lexical { bindings, .. } = &module.statements()[0].kind else {
        panic!("lexical")
    };
    assert!(matches!(
        bindings[0].initializer.as_ref().unwrap().kind,
        ExprKind::ImportMeta
    ));
    assert_eq!(bindings[0].initializer.as_ref().unwrap().span.start, 11);
    let StatementKind::Expression(expression) = &module.statements()[1].kind else {
        panic!("expression")
    };
    assert!(matches!(expression.kind, ExprKind::Await(_)));
    insta::assert_debug_snapshot!(module);
}

#[test]
fn strict_module_scope_treats_direct_functions_as_lexical_declarations() {
    for source in [
        "with({}){}",
        "var eval",
        "let arguments",
        "delete name",
        "010",
        "'\\1'",
        "function f(a,a){}",
        "function f(){}function f(){}",
        "var f;function f(){}",
        "function f(){}if(false){var f}",
        "let x;{var x}",
    ] {
        assert_eq!(
            parse_module(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    assert!(parse_script("function f(){}function f(){}").is_ok());
    for source in [
        "var x;var x",
        "function f(){};let x",
        "using resource=null",
        "",
    ] {
        assert!(parse_module(source).is_ok(), "{source}");
    }
}

#[test]
fn await_is_reserved_in_module_identifiers_including_nested_non_async_code() {
    for source in [
        "let await",
        "function await(){}",
        "(function await(){})",
        "function f(await){}",
        "function f(){var await}",
        "function f(){return await}",
        "class await{}",
        "class C{m(await){}}",
        "const f=await=>await",
        "await:;",
        "let a\\u0077ait",
    ] {
        assert_eq!(
            parse_module(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    for source in [
        "({await:1}).await",
        "const {await:x}={await:1}",
        "class C{await(){return import.meta}}",
    ] {
        assert!(parse_module(source).is_ok(), "{source}");
    }
    assert!(parse_script("'use strict';function f(){var await}").is_ok());
}

#[test]
fn module_context_reaches_nested_meta_while_function_context_restricts_await_and_returns() {
    for source in [
        "function f(){return import.meta}",
        "const f=()=>import.meta",
        "class C{field=import.meta;static value=import.meta}",
        "for await(const x of []){}",
        "function f(){return new.target}",
        "await import('source')",
    ] {
        assert!(
            parse_module(source).is_ok(),
            "{source}: {:?}",
            parse_module(source)
        );
    }
    for source in [
        "return 1",
        "new.target",
        "super.x",
        "function f(){await 1}",
        "import.meta=1",
        "++import.meta",
        "import.m\\u0065ta",
    ] {
        assert_eq!(
            parse_module(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    assert_eq!(
        parse_script("import.meta").unwrap_err().kind,
        DiagnosticKind::Syntax
    );
    assert_eq!(
        parse_dynamic_function("", "return import.meta")
            .unwrap_err()
            .kind,
        DiagnosticKind::Syntax
    );
}

#[test]
fn static_module_declarations_keep_their_explicit_boundary_only_at_top_level() {
    for source in [
        "import 'source'",
        "import value from 'source'",
        "export {}",
        "export default 1",
    ] {
        assert_eq!(
            parse_module(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
    for source in ["{import 'source'}", "function f(){export {}}"] {
        assert_eq!(
            parse_module(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn module_utf16_preserves_lone_surrogates_and_source_limits_remain_opted_in() {
    let prefix = "let x='";
    let suffix = "';import.meta";
    let units = prefix
        .encode_utf16()
        .chain([0xd800])
        .chain(suffix.encode_utf16())
        .collect();
    let module = parse_module_utf16(&JsString::from_code_units(units)).unwrap();
    let StatementKind::Lexical { bindings, .. } = &module.statements()[0].kind else {
        panic!("lexical")
    };
    assert!(
        matches!(&bindings[0].initializer.as_ref().unwrap().kind,ExprKind::Literal(Literal::String(value)) if value==&JsString::from_code_units(vec![0xd800]))
    );
    assert_eq!(
        module.statements()[1].span.end,
        prefix.len() + 3 + suffix.len()
    );
    assert!(parse_module_with_source_limit("await 1", 7).is_ok());
    assert_eq!(
        parse_module_with_source_limit("await 1", 6)
            .unwrap_err()
            .kind,
        DiagnosticKind::Limit
    );
}
