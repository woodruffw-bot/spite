//! Catch binding-pattern grammar and early errors (14.3.3, 14.15.1).

use spite_core::DiagnosticKind;
use spite_parser::{MAX_DEPTH, ast::StatementKind, parse_script};

#[test]
fn nested_catch_binding_pattern_syntax_snapshot() {
    insta::assert_debug_snapshot!(
        parse_script("try {} catch ({[key]: [x = 1, ...xs], ...rest}) {}").unwrap()
    );
}

#[test]
fn catch_binding_pattern_diagnostics_snapshot() {
    let errors: Vec<_> = [
        "try {} catch ({a, b: [a]}) {}",
        "try {} catch ([x]) { with ({}) var x; }",
        "'use strict'; try {} catch ({a: eval}) {}",
        "try {} catch ([...rest,]) {}",
        "try {} catch ({...{x}}) {}",
        "try {} catch ({[key]}) {}",
    ]
    .map(|source| parse_script(source).unwrap_err())
    .into();
    insta::assert_debug_snapshot!(errors);
}

#[test]
fn identifiers_patterns_defaults_elisions_and_rest_accept_the_full_catch_grammar() {
    for parameter in [
        "{}",
        "[]",
        "[,,]",
        "[x,]",
        "[x,,]",
        "[x=1,,...rest]",
        "[...[]]",
        "[...{length}]",
        "{a}",
        "{a=1}",
        "{a:x=1,b:y=x,...rest}",
        "{a:{b}=null}",
        "{a,}",
        "{...rest}",
        "{default:x,true:y,null:z}",
        "{0:x,'key':y,[a in b]:z}",
        "{a:a,b:b}",
        "{__proto__:x,__proto__:y}",
        r"{\u0061, b: \u0078}",
        "{await, yield, let}",
    ] {
        assert!(
            parse_script(&format!("try {{}} catch ({parameter}) {{}}")).is_ok(),
            "{parameter}"
        );
    }
    assert!(parse_script("'use strict'; try {} catch ({for:x,eval:y}) {}").is_ok());
}

#[test]
fn bound_names_include_nested_targets_and_rest_in_source_order() {
    let script = parse_script("try {} catch ({a:x,b:[y,,...{length:z}],...rest}) {}").unwrap();
    let StatementKind::Try {
        handler: Some(handler),
        ..
    } = &script.statements()[0].kind
    else {
        panic!("catch");
    };
    let parameter = handler.parameter.as_ref().unwrap();
    assert_eq!(
        parameter
            .bound_names()
            .iter()
            .map(|(name, _)| *name)
            .collect::<Vec<_>>(),
        ["x", "y", "z", "rest"]
    );
}

#[test]
fn duplicate_invalid_and_strict_binding_names_are_early_errors() {
    for parameter in [
        "[x,x]",
        r"{x,a:\u0078}",
        "{x,...x}",
        "{a:x,b:[x]}",
        "{for}",
        "{a:for}",
        "{...for}",
        "[...for]",
        "[...rest=1]",
        "{...rest=1}",
        "{...rest,}",
        "{...rest,a}",
        "[...rest,x]",
        "[(x)]",
        "[x.y]",
        "{a:x.y}",
        "{x(){}}",
        "{get x(){}}",
        "[x] = []",
        "{[a,b]:x}",
        "{x:y=}",
        "{,}",
        "[... ]",
    ] {
        let source = format!("try {{}} catch ({parameter}) {{}}");
        assert_eq!(
            parse_script(&source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    for parameter in [
        "{eval}",
        "[arguments]",
        "{x:{let}}",
        "{...yield}",
        "{x=delete name}",
        "{[delete name]:x}",
    ] {
        let source = format!("'use strict'; try {{}} catch ({parameter}) {{}}");
        assert_eq!(
            parse_script(&source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn all_bound_names_conflict_with_direct_lexicals_and_nested_vars() {
    for body in [
        "let x;",
        "const x=1;",
        "function x(){}",
        "with({}) var x;",
        "if(false)var x;",
        "try{}finally{var x;}",
    ] {
        let source = format!("try {{}} catch ({{a:[x]}}) {{ {body} }}");
        assert_eq!(
            parse_script(&source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    assert!(parse_script("try{}catch({x}){{let x;}function f(){var x;}}").is_ok());
}

#[test]
fn nested_patterns_share_the_parser_native_stack_guard() {
    let source = format!(
        "try{{}}catch({}x{}){{}}",
        "[".repeat(MAX_DEPTH),
        "]".repeat(MAX_DEPTH)
    );
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}
