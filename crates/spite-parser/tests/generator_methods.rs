//! Synchronous GeneratorMethod grammar and context boundaries (15.5).

use spite_core::DiagnosticKind;
use spite_parser::{
    MAX_DEPTH,
    ast::{ClassElement, ExprKind, FunctionKind, ObjectElement, ObjectProperty, StatementKind},
    parse_script,
};

fn summary(property: &ObjectProperty) -> (FunctionKind, String, bool) {
    let ExprKind::Function(function) = &property.value.kind else {
        panic!("method function")
    };
    let StatementKind::Expression(expression) = &function.body.statements()[0].kind else {
        panic!("yield statement")
    };
    let ExprKind::Yield { delegated, .. } = &expression.kind else {
        panic!("yield expression")
    };
    (
        function.kind,
        function.source.as_str().unwrap().to_owned(),
        *delegated,
    )
}

#[test]
fn object_class_static_and_private_methods_retain_generator_kind_and_exact_source() {
    let script = parse_script(
        "({*g(x){yield x},*['h'](){yield* tail}});class C{*#g(){yield 1}static *h(){yield 2}}",
    )
    .unwrap();
    let StatementKind::Expression(expression) = &script.statements()[0].kind else {
        panic!("object statement")
    };
    let ExprKind::Parenthesized(expression) = &expression.kind else {
        panic!("parenthesized object")
    };
    let ExprKind::Object(properties) = &expression.kind else {
        panic!("object")
    };
    let mut summaries = Vec::new();
    for property in properties {
        let ObjectElement::Property(property) = property else {
            panic!("property")
        };
        summaries.push(summary(property));
    }
    let StatementKind::Class(class) = &script.statements()[1].kind else {
        panic!("class")
    };
    for element in &class.elements {
        let ClassElement::Method { property, .. } = element else {
            panic!("method")
        };
        summaries.push(summary(property));
    }
    assert!(
        summaries
            .iter()
            .all(|entry| entry.0 == FunctionKind::Generator)
    );
    insta::assert_debug_snapshot!(summaries);
}

#[test]
fn contextual_names_static_modifiers_asi_and_computed_names_parse() {
    for source in [
        "({*get(){yield 1},*set(){yield 2},*async(){yield 3},*__proto__(){}})",
        "class C{*static(){yield 1}static *constructor(){yield 2}*[Symbol.iterator](){yield this}*#m(){yield 1}static *#s(){yield 2}}",
        "class C{async\n*g(){yield 1}}",
        "function*outer(){({*[yield 1](){yield 2}});class C{*[yield 3](){yield 4}}}",
        "async function f(){return {*[await 'g'](await){yield await}}}",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
}

#[test]
fn parameters_forbid_yield_and_duplicate_names_even_in_sloppy_objects() {
    for source in [
        "({*g(yield){}})",
        "({*g(x=yield 1){}})",
        "({*g(a,a){}})",
        "class C{*#g({yield}){}}",
        "class C{static *g(x=yield 1){}}",
        "({*g(){yield:;}})",
        "({*g(x=1){'use strict'}})",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn super_private_and_nested_function_contexts_follow_method_boundaries() {
    for source in [
        "({*g(){yield super.x;yield ()=>super.x;yield new.target}})",
        "class C extends B{#x;*g(){yield this.#x;yield super.x}}",
        "({*g(){function f(yield){return yield}yield ()=>yield;yield {m(yield){return yield}}}})",
        "function*outer(){({*[yield 1](x=function*(){yield 2}){yield x}});yield 3}",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
    for source in [
        "({*g(){super()}})",
        "class C{*g(){yield object.#missing}}",
        "class C{#x;*g(){yield delete this.#x}}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn generator_constructors_fields_and_accessor_combinations_are_syntax_errors() {
    for source in [
        "class C{*constructor(){}}",
        "class C{static *prototype(){}}",
        "class C{*#constructor(){}}",
        "class C{*g;}",
        "({*g:1})",
        "({*g=1})",
        "class C{get *g(){}}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn async_generator_methods_and_native_depth_limits_keep_distinct_boundaries() {
    for source in [
        "({async *g(){}})",
        "class C{async *g(){}}",
        "class C{static async *#g(){}}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
    let source = format!(
        "({{*g(){{yield {}0{}}}}})",
        "(".repeat(MAX_DEPTH * 4),
        ")".repeat(MAX_DEPTH * 4)
    );
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}
