//! Generator declarations/expressions and YieldExpression grammar (15.5).

use spite_core::DiagnosticKind;
use spite_parser::{
    MAX_DEPTH,
    ast::{ExprKind, FunctionKind, StatementKind},
    parse_dynamic_function, parse_script,
};

#[test]
fn generator_kind_exact_source_and_yield_nodes_are_retained() {
    let source =
        "function* g(x=1,...tail){yield;yield x+2;yield* tail;return (x=yield /y/.test('y'));}";
    let script = parse_script(source).unwrap();
    let function = script.function_declarations()[0];
    assert_eq!(function.kind, FunctionKind::Generator);
    assert_eq!(function.source.as_str(), Some(source));
    let StatementKind::Expression(expression) = &function.body.statements()[0].kind else {
        panic!("yield statement")
    };
    assert!(matches!(
        expression.kind,
        ExprKind::Yield {
            delegated: false,
            argument: None
        }
    ));
    insta::assert_debug_snapshot!(script);
}

#[test]
fn assignment_precedence_commas_regexp_goals_and_restricted_linebreaks_parse() {
    for source in [
        "function*g(){yield x?y:z}",
        "function*g(){yield x=1}",
        "function*g(){yield a,b}",
        "function*g(){yield (yield 1)}",
        "function*g(){yield*\nsource}",
        "function*g(){yield\n/r/.test('r')}",
        "function*g(){return yield /r/.test('r')}",
        "function*g(){yield\n,1}",
        "function*g(){(yield x).value}",
        "function*g(){let[x=yield 1]=[];({[yield 'key']:x}=source)}",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
}

#[test]
fn yield_identifier_and_function_name_contexts_reset_at_nested_functions() {
    for source in [
        "let yield=1;yield++",
        "function* yield(){yield 1}",
        "function*g(){function f(yield){return yield}let nested=function yield(){};let a=()=>yield;let o={m(yield){return yield}}}",
        "function*g(x=function*(){yield 1}){yield x}",
        "function*g(x=()=>yield){yield x}",
        "async function f(){return function*g(){yield 1}}",
        "async function f(){return function*await(await){yield await}}",
        "function*g(){let f=async function yield(){await 1};yield f}",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
}

#[test]
fn generator_bindings_and_outer_declaration_names_cannot_use_yield() {
    for source in [
        "function*g(yield){}",
        "function*g({yield}){}",
        "function*g(...yield){}",
        "function*g(){var yield}",
        "function*g(){let {yield}=source}",
        "function*g(){yield:;}",
        "function*g(){function yield(){}}",
        "function*g(){class yield{}}",
        "(function* yield(){})",
        "async function f(){function*await(){}}",
        "function*g(){yi\\u0065ld 1}",
        "function*g(){let yi\\u0065ld}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn parameter_initializers_and_arrow_covers_reject_yield_expressions() {
    for source in [
        "function*g(x=yield 1){}",
        "function*g({[yield 1]:x}){}",
        "function*g(){(x=yield 1)=>x}",
        "function*g(){async(x=yield 1)=>x}",
        "function*g(){(yield)=>1}",
        "function*g(){async(yield)=>1}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn validation_reaches_operands_and_preserves_sloppy_and_strict_parameter_rules() {
    for source in [
        "function*g(){'use strict';yield delete x}",
        "function*g(){'use strict';yield (eval=1)}",
        "function*g(){yield object.#missing}",
        "function*g(a,a){'use strict'}",
        "function*g(a=1,a){}",
        "function*g(a=1){'use strict'}",
        "function*g(a){let a}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    assert!(parse_script("function*g(a,a){yield a}").is_ok());
}

#[test]
fn delegation_requires_an_operand_and_unparenthesized_yields_are_not_references_or_members() {
    for body in [
        "yield*;",
        "yield\n*source",
        "(yield x)=1",
        "(yield x)++",
        "yield.foo",
        "yield()",
        "new yield C",
        "yield ?x:y",
    ] {
        let source = format!("function*g(){{{body}}}");
        assert_eq!(
            parse_script(&source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn dynamic_ordinary_functions_retain_generator_syntax_and_depth_limits_stay_separate() {
    let function = parse_dynamic_function("", "function*g(){yield 1};return g").unwrap();
    assert_eq!(function.kind, FunctionKind::Ordinary);
    for source in ["async function*g(){}", "({*g(){}})", "class C{*g(){}}"] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
    let source = format!(
        "function*g(){{yield {}0{}}}",
        "(".repeat(MAX_DEPTH * 4),
        ")".repeat(MAX_DEPTH * 4)
    );
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}
