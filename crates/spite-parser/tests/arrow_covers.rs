//! Arrow probes refine the parameter grammar with expression-selected goals.

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{
    EvalContext, MAX_DEPTH, parse_dynamic_function, parse_eval_utf16, parse_script,
    parse_script_utf16,
};

#[test]
fn grammar_selected_literal_boundaries_survive_nested_arrow_covers() {
    let sources = [
        "(x = /[)]/) => x",
        "(x = /[(]/) => x",
        "(x = /[}\\]`]/) => x",
        "(x = /\\)/g) => x",
        "(x = /[)]/ / 2) => x",
        "({x = /[)]/} = {}) => x",
        "([x = /[)]/] = []) => x",
        "(x = (y = /[)]/) => y) => x",
        "(x = `${/[}`]/}tail`) => x",
        "(x = class { get [/[(]/]() {} }) => x",
        "(x = /[(]/) + 1",
        "([x = /a/])",
        "(x = (y = /[)]/) => y) + 1",
    ];
    for source in sources {
        let expected = parse_script(source).unwrap_or_else(|error| panic!("{source}: {error}"));
        assert_eq!(
            parse_script_utf16(&JsString::from(source)).unwrap(),
            expected
        );
        assert_eq!(
            parse_eval_utf16(&JsString::from(source), EvalContext::default()).unwrap(),
            expected
        );
    }
    for body in ["return (x = /[)]/) => x;", "return (x = /[(]/) + 1;"] {
        parse_dynamic_function("", body).unwrap();
    }
}

#[test]
fn invalid_patterns_and_arrow_line_terminators_remain_syntax_errors() {
    let sources = [
        "(x = /[)]/)\n=> x",
        "(x = /[)]/u) /*\n*/ => x",
        "(x = /a/gg) => x",
        "(x = /[/) => x",
        "(x = /a\n/) => x",
        "(x = /(/) => x",
        "({x=1})",
        "({__proto__:1,__proto__:2})",
        "(for)=>1",
        "(return)=>1",
        "(enum)=>1",
        "(...[x],)=>x",
        "(...x=[])=>1",
        "([...[x]=[]])=>1",
        "([...x,y])=>1",
        "({bre\\u0061k})=>1",
        "let f=import=>1",
        "(import)=>1",
        "({x=/a/})",
    ];
    let diagnostics: Vec<_> = sources
        .into_iter()
        .map(|source| {
            let error = parse_script(source).unwrap_err();
            assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
            (source, error)
        })
        .collect();
    insta::assert_debug_snapshot!(diagnostics);
    assert_eq!(
        parse_script("import('x')").unwrap_err().kind,
        DiagnosticKind::Unsupported
    );
}

#[test]
fn fallback_expressions_and_selected_heads_preserve_private_names_and_context() {
    for source in [
        "class C { #x; m() { return (x = this.#x) => x; } }",
        "class C { #x; m() { return (x = this.#x); } }",
        "class C { #x; m() { return (x = class { m() { return this.#x; } }) => x; } }",
        "class C { #x; m() { return (x = class { get [()=>this.#x]() {} }); } }",
        "class C { static { let f = (x = () => await) => x; } }",
        "function f() { return (x = new.target) => x; }",
        "for (let f = (x = 1 in {}) => x; false;) ;",
        "((x = 1), x)",
        "([...x,y])",
        "([...x=[]])",
        "({bre\\u0061k:1})",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
    for source in [
        "class C { m() { return (x = this.#missing) => x; } }",
        "class C { m() { return (x = this.#missing); } }",
        "class C { m() { return (x = class { get [()=>this.#missing]() {} }); } }",
        "class C { static { let f = (await) => 1; } }",
        "(x = new.target) => x",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn nested_failed_heads_and_default_arrows_use_the_existing_stack_guard() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            for n in [8, 16] {
                let ordinary = format!("{}1{}", "(x=".repeat(n), ")".repeat(n));
                assert!(parse_script(&ordinary).is_ok(), "{ordinary}");
                let arrows = format!("{}1{}", "(x=".repeat(n), ")=>x".repeat(n));
                assert!(parse_script(&arrows).is_ok(), "{arrows}");
            }
            for suffix in [")", ")=>x"] {
                let source = format!(
                    "{}1{}",
                    "(x=".repeat(MAX_DEPTH * 2),
                    suffix.repeat(MAX_DEPTH * 2)
                );
                assert_eq!(
                    parse_script(&source).unwrap_err().kind,
                    DiagnosticKind::Limit
                );
            }
        })
        .unwrap()
        .join()
        .unwrap();
}
