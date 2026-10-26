//! ForAwaitOf grammar contexts, head restrictions and source AST (14.7.5).

use spite_core::DiagnosticKind;
use spite_parser::{MAX_DEPTH, parse_dynamic_async_function, parse_dynamic_function, parse_script};

#[test]
fn for_await_source_snapshot_distinguishes_synchronous_iteration() {
    insta::assert_debug_snapshot!(
        parse_script(
            "async function f(){for await(const x of source){await x}for(let y of source);}"
        )
        .unwrap()
    );
}

#[test]
fn for_await_accepts_assignment_pattern_declaration_and_using_heads() {
    for head in [
        "x",
        "object.x",
        "object[key]",
        "[x]",
        "{x}",
        "var x",
        "var [x]",
        "let x",
        "const {x}",
        "using x",
        "async",
        r"a\u0073ync",
    ] {
        let source = format!("async function f(){{for await({head} of source){{await 0}}}}");
        assert!(
            parse_script(&source).is_ok(),
            "{source}: {:?}",
            parse_script(&source)
        );
    }
    for source in [
        "async x=>{for await(x of source){}}",
        "async(x)=>{for await(x of source){}}",
        "let object={async f(){for await(this.x of source){}}}",
        "class C extends B{async f(){for await(super.x of source){}}}",
        "async function f(){for\nawait\n(var x of [])break;}",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
}

#[test]
fn for_await_requires_an_async_body_and_restores_enclosing_contexts() {
    for source in [
        "for await(x of []);",
        "function f(){for await(x of []);}",
        "async function f(){function g(){for await(x of []);}}",
        "async function f(){()=>{for await(x of []);}}",
        "async function f(){class C{static{for await(x of []);}}}",
        "async function f(){for await(x of []);}for await(y of []);",
        r"async function f(){for a\u0077ait(x of []);}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn for_await_rejects_in_three_clause_initializers_and_invalid_targets() {
    for head in [
        "x in []",
        ";;",
        "var x=0 of []",
        "let x=0 of []",
        "const x=0 of []",
        "using x=null of []",
        "var x,y of []",
        "let x,y of []",
        "const [x],y of []",
        "x+1 of []",
        "let of []",
        "x of [],[]",
        r"x o\u0066 []",
    ] {
        let source = format!("async function f(){{for await({head});}}");
        assert_eq!(
            parse_script(&source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn for_await_preserves_iteration_labels_lexical_conflicts_and_dynamic_function_contexts() {
    assert!(
        parse_script("async function f(){outer:inner:for await(const x of []){continue outer;}}")
            .is_ok()
    );
    for source in [
        "async function f(){for await(let x of []){var x;}}",
        "async function f(){label:{for await(x of [])continue label;}}",
        "async function f(){for await(const x of []);continue;}",
        "async function f(){for await(const [x,x] of []);}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    assert!(parse_dynamic_async_function("", "for await(var x of []){}").is_ok());
    assert_eq!(
        parse_dynamic_function("", "for await(var x of []){}")
            .unwrap_err()
            .kind,
        DiagnosticKind::Syntax
    );
}

#[test]
fn for_await_keeps_nested_native_depth_limits_and_remaining_gaps_explicit() {
    let source = format!(
        "async function f(){{{};}}",
        "for await(var x of [])".repeat(MAX_DEPTH * 2)
    );
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
    assert!(parse_script("async function* f(){for await(var x of []);}").is_ok());
    {
        let source = "async function f(){for await(await using x of []);}";
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
}
