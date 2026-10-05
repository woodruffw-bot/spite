//! Base classes and strict ClassDefinition early errors (15.7).

use spite_core::DiagnosticKind;
use spite_parser::{MAX_DEPTH, parse_script};

#[test]
fn base_class_syntax_snapshot() {
    insta::assert_debug_snapshot!(
        parse_script("class C { constructor(x=1){this.x=x;} static get v(){return 2;} [3](){} }")
            .unwrap()
    );
    for source in [
        "let C=class{};",
        "let C=(class Named{});",
        "class C { ; ; static(){} get(){} set(){} async(){} }",
        "class C { static constructor(){} ['constructor'](){} }",
        "class C { constructor(x=1){} m(x=1){} set x(v=1){} }",
        "class C { constructor(){super.x;} m(){return ()=>super.x;} }",
        "function f(){return class { [new.target](){} };}",
        "({m(){return class { [super.x](){} };}})",
        "class C { get x(){} set x({v}){} static ['prototype'](){} }",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}

#[test]
fn class_early_errors_snapshot() {
    let errors: Vec<_> = [
        "class {}",
        "class eval {}",
        "class let {}",
        "class C { constructor(){} constructor(){} }",
        "class C { get constructor(){} }",
        "class C { static prototype(){} }",
        "class C { m(a,a){} }",
        "class C { m(x=1){'use strict';} }",
        "class C { m(){with({}){} } }",
        "class C { [077](){} }",
        "class C { m(){return function(){return super.x;};} }",
        "class C { constructor(){super();} }",
        "class C{} var C;",
        "{ let C; class C{} }",
        "if(true) class C{}",
        "class C { [new.target](){} }",
    ]
    .into_iter()
    .map(|source| {
        let error = parse_script(source).unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
        (source, error)
    })
    .collect();
    insta::assert_debug_snapshot!(errors);
}

#[test]
fn incomplete_class_features_never_receive_negative_test_credit() {
    for source in [
        "class C{#x;}",
        "class C{m(){return this.#x;} #x;}",
        "class C{get #x(){}}",
        "class C{m(){return #x in this;} #x;}",
        "class C{*g(){}}",
        "class C{async m(){}}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
    let source = format!("let C={};", "class { [".repeat(MAX_DEPTH * 2));
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}
