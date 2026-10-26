//! Async method grammar, names, boundaries and class/private forms.

use spite_core::DiagnosticKind;
use spite_parser::parse_script;

#[test]
fn async_object_class_and_private_method_syntax_snapshot() {
    insta::assert_debug_snapshot!(parse_script("let o={async m(x=1){return x},async ['n'](){return super.n()}};class C extends B{async #m(){return this}static async m(){return super.m()}}").unwrap());
}

#[test]
fn names_fields_newlines_and_nested_ordinary_contexts_remain_distinct() {
    for source in [
        "({async(){},async:1})",
        "({async await(){return 1},async get(){return 2}})",
        "class C{async=1;async(){};async\nm(){}}",
        "class C{static async=1;static async(){}}",
        "({async m(){return function await(){return await}}})",
        "({async m(){return()=>await}})",
        "class C{async ['constructor'](){return 1}}",
        "({async [key](x=super.m()){return x}})",
        "class C{static async #m(x){return x}}",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
}

#[test]
fn constructor_await_duplicate_parameter_and_linebreak_early_errors_are_syntax() {
    for source in [
        "class C{async constructor(){}}",
        "class C{async 'constructor'(){}}",
        "({async\nm(){}})",
        "({\\u0061sync m(){}})",
        "({async m(a,a){}})",
        "class C{async m(a,a){}}",
        "({async m(await){}})",
        "({async m(){var await}})",
        "({async m(a=1){'use strict'}})",
        "({async m(){super()}})",
        "({async m(a=super()){}})",
        "class C{async m(){super()}}",
        "({async get m(){}})",
        "class C{static async prototype(){}}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn async_generator_method_forms_share_supported_method_grammar() {
    for source in ["({async *m(){}})", "class C{static async *m(){}}"] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}
