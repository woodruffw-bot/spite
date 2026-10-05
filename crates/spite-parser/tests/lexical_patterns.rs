//! Lexical binding patterns and declaration/loop early errors (14.3.1, 14.7).

use spite_core::DiagnosticKind;
use spite_parser::{MAX_DEPTH, ast::*, parse_script};

#[test]
fn lexical_binding_pattern_syntax_snapshot() {
    insta::assert_debug_snapshot!(parse_script("let {x:[y=1],...rest} = source;").unwrap());
}

#[test]
fn lexical_binding_pattern_diagnostics_snapshot() {
    insta::assert_debug_snapshot!(
        [
            "let {x:let} = {};",
            "let [x,{a:x}] = [];",
            "const {} ;",
            "{let {x}={};if(false)var x;}",
            "function f(x){let {x}={};}",
            "for(let [x] of []) {with({})var x;}",
            "'use strict';let {[delete name]:x}={};",
        ]
        .map(|source| parse_script(source).unwrap_err())
    );
}

#[test]
fn patterns_work_in_statement_lists_and_all_lexical_loop_headers() {
    for pattern in [
        "{}",
        "[]",
        "[,,]",
        "[a=1,...[b,c]]",
        "[...{length}]",
        "{a,b:[c],...rest}",
        "{default:x,let:y}",
        "{a:x,a:y}",
        "{[key in object]:x=key in object}",
        "{await,yield}",
    ] {
        for source in [
            format!("let {pattern} = source;"),
            format!("const {pattern} = source;"),
            format!("for(let {pattern} = source; false;) ;"),
            format!("for(const {pattern} = source; false;) ;"),
            format!("for(let {pattern} of source) ;"),
            format!("for(const {pattern} in source) ;"),
        ] {
            assert!(parse_script(&source).is_ok(), "{source}");
        }
    }
    assert!(parse_script("let a, {b}={}, [c]=[], d=1;").is_ok());
    assert!(parse_script("for(let [a]=(key in source);false;) ;").is_ok());
    assert!(parse_script("function f(a){{let [a]=[];}}").is_ok());
}

#[test]
fn all_bound_names_participate_in_declaration_and_loop_conflicts() {
    for source in [
        "let {} ;",
        "let [] ;",
        "const [a];",
        "for(let {};;) ;",
        "let {a:let}={};",
        r"let [\u006cet]=[];",
        "let [a,a]=[];",
        "let {a,...a}={};",
        "let {a}={},a;",
        "{var a;let {a}={};}",
        "{let [a]=[];with({})var a;}",
        "switch(0){case 0:let {a}={};case 1:let a;}",
        "function f(a){let [a]=[];}",
        "(a)=>{let {a}={};}",
        "try{}catch([a]){let {a}={};}",
        "for(let [a,a] of []) ;",
        "for(const {a,...a} in {}) ;",
        "for(let [a]=[];;){if(false)var a;}",
        "for(let {a} in {})var a;",
        "for(let [a]=[] of []) ;",
        "for(const {}={} in {}) ;",
        "for(let [a],b of []) ;",
        "for(const {a},b in {}) ;",
        "'use strict';let {eval}={};",
        "'use strict';let [arguments]=[];",
        "'use strict';for(let {a=delete name} of []) ;",
        "let {a:[...rest,]}={};",
        "let {...{a}}={};",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn bound_names_omit_property_keys_and_include_nested_and_rest_targets() {
    let script = parse_script("let {a:[x,,...{length:y}],...rest}={};").unwrap();
    let StatementKind::Lexical { bindings, .. } = &script.statements()[0].kind else {
        panic!("lexical");
    };
    assert_eq!(
        bindings[0]
            .pattern
            .bound_names()
            .iter()
            .map(|(name, _)| *name)
            .collect::<Vec<_>>(),
        ["x", "y", "rest"]
    );
    let script = parse_script("for(const {} of source) ;").unwrap();
    let StatementKind::ForOf {
        binding: ForBinding::Lexical { binding, .. },
        ..
    } = &script.statements()[0].kind
    else {
        panic!("for declaration");
    };
    assert!(binding.bound_names().is_empty());
}

#[test]
fn other_pattern_contexts_remain_explicitly_unsupported_and_depth_is_guarded() {
    for source in ["function f({a}){}", "({a}=source);", "for([a] of []) ;"] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
    let source = format!(
        "let {}a{}=[];",
        "[".repeat(MAX_DEPTH),
        "]".repeat(MAX_DEPTH)
    );
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}
