//! Contextual resource bindings, scope early errors and loop grammar.

use spite_core::{DiagnosticKind, JsString};
use spite_parser::ast::StatementKind;
use spite_parser::{parse_module, parse_script, parse_script_utf16};

#[test]
fn using_statements_and_both_loop_headers_snapshot() {
    insta::assert_debug_snapshot!(parse_script("{using first=null,second=resource;for(using all=null;test;update){}for(using item of source){}}function f(){using value=null;return value}").unwrap());
}

#[test]
fn resource_declarations_accept_identifier_bindings_in_lexical_scopes() {
    for source in [
        "{using x=null,y=undefined;}",
        "function f(){using x=null;return x}",
        "async function f(){using x=null;return x}",
        "class C{static{using x=null}m(){using x=null}}",
        "for(using x=null;false;){}",
        "for(using x of []){}",
        "for(using of=null;false;){}",
        r"for(using \u006ff of []){}",
        "{using \\u0061=null}",
        "function f(){using await=null;return await}",
        "switch(0){case 0:{using x=null;break}default:{using y=null}}",
        "{using x=/a/;for(using y=/b/;false;){}}",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
}

#[test]
fn binding_scope_script_switch_and_statement_position_errors_are_syntax() {
    for source in [
        "using x=null;",
        "'use strict';using x=null;",
        "{using x;}",
        "{using x=null,y;}",
        "{using x,y=null;}",
        "{using let=null;}",
        "{using x=null,x=null;}",
        "{let x;using x=null;}",
        "{using x=null;var x;}",
        "function f(x){using x=null;}",
        "'use strict';{using eval=null;}",
        "async function f(){using await=null;}",
        "class C{static{using await=null;}}",
        "{using {x}=null;}",
        "{using x=null,{y}=null;}",
        "{u\\u0073ing x=null;}",
        "if(true)using x=null;",
        "label:using x=null;",
        "switch(0){case 0:using x=null;}",
        "switch(0){default:using x=null;}",
        "for(using x in {}){}",
        "for(using x=1 of []){}",
        "for(using x,y of []){}",
        "for(using x=null;false;){var x;}",
        "for(using x of []){var x;}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn ordinary_using_identifiers_linebreaks_and_for_of_lookahead_remain_distinct() {
    for source in [
        "let using=1;using+=1;using;",
        "function using(){};using();",
        "let using={};using.x=1;using['x']=2;",
        "using in object;",
        "using instanceof Object;",
        "'use strict';using in object;using instanceof Object;",
        "for(using in object){}",
        "for(using instanceof Object;false;){}",
        "async function f(using){return await using in object}",
        "async function f(using){return await using instanceof Object}",
        "class C{m(using){using in this;return using instanceof Object}}",
        "{using\nx=null;}",
        "{using\nlet=1;var using,let;}",
        "var using,of=[[9],[8],[7]];for(using of of[0,1,2]){}",
        "let using=[];for(using[0] of []){}",
        "let using=[];for(using [0] of []){}",
        "{using [x]=null;}",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
        assert!(
            parse_script_utf16(&JsString::from(source)).is_ok(),
            "UTF-16: {source}"
        );
    }
    for (gap, newline) in [
        (" ", false),
        ("\t", false),
        ("/**/", false),
        ("/*x*/", false),
        ("//x\n", true),
        ("\n", true),
        ("\r", true),
        ("\r\n", true),
        ("\u{2028}", true),
        ("\u{2029}", true),
        ("/*\n*/", true),
        ("/*\r*/", true),
        ("/*\r\n*/", true),
        ("/*\u{2028}*/", true),
        ("/*\u{2029}*/", true),
    ] {
        let source = format!("function f(){{using{gap}x=null;}}");
        let script = parse_script(&source).unwrap();
        let module = parse_module(&source).unwrap();
        for statements in [script.statements(), module.statements()] {
            let StatementKind::Function(function) = &statements[0].kind else {
                panic!("function")
            };
            let body = function.body.statements();
            assert_eq!(body.len(), if newline { 2 } else { 1 }, "{gap:?}");
            assert_eq!(
                matches!(body[0].kind, StatementKind::Using(_)),
                !newline,
                "{gap:?}"
            );
            assert_eq!(function.source.as_str(), Some(source.as_str()));
        }
        let source = format!("function f(){{for(using{gap}x of []){{}}}}");
        if newline {
            assert_eq!(
                parse_script(&source).unwrap_err().kind,
                DiagnosticKind::Syntax
            );
            assert_eq!(
                parse_module(&source).unwrap_err().kind,
                DiagnosticKind::Syntax
            );
        } else {
            assert!(parse_script(&source).is_ok());
            assert!(parse_module(&source).is_ok());
        }
    }
}

#[test]
fn await_using_remains_unsupported_and_synchronous_using_accepts_await_initializers() {
    assert!(parse_script("async function f(){for await(using x of []){}}").is_ok());
    assert!(parse_script("async function f(){using x=await p;}").is_ok());
    for source in [
        "async function f(){await using x=null;}",
        "async function f(){for await(await using x of []){}}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
}
