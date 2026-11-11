//! ImportCall grammar, retained source operands and Script lexical goals (13.3.10).

use spite_core::DiagnosticKind;
use spite_parser::ast::{ExprKind, StatementKind};
use spite_parser::parse_script;

#[test]
fn specifier_options_and_exact_import_call_span_are_retained() {
    let source = "import(/x/, {with:{type:'json'}},)";
    let script = parse_script(source).unwrap();
    let StatementKind::Expression(expression) = &script.statements()[0].kind else {
        panic!("expression")
    };
    let ExprKind::ImportCall { specifier, options } = &expression.kind else {
        panic!("import call")
    };
    assert!(matches!(specifier.kind, ExprKind::Literal(_)));
    assert!(matches!(
        options.as_ref().unwrap().kind,
        ExprKind::Object(_)
    ));
    assert_eq!(&source[expression.span.start..expression.span.end], source);
    insta::assert_debug_snapshot!(script, @r#"
        Script {
            statements: [
                Statement {
                    kind: Expression(
                        Expr {
                            depth: 4,
                            kind: ImportCall {
                                specifier: Expr {
                                    depth: 1,
                                    kind: Literal(
                                        RegExp {
                                            body: "x",
                                            flags: "",
                                        },
                                    ),
                                    span: Span {
                                        start: 7,
                                        end: 10,
                                    },
                                },
                                options: Some(
                                    Expr {
                                        depth: 3,
                                        kind: Object(
                                            [
                                                Property(
                                                    ObjectProperty {
                                                        name: Literal(
                                                            String(
                                                                "with",
                                                            ),
                                                        ),
                                                        value: Expr {
                                                            depth: 2,
                                                            kind: Object(
                                                                [
                                                                    Property(
                                                                        ObjectProperty {
                                                                            name: Literal(
                                                                                String(
                                                                                    "type",
                                                                                ),
                                                                            ),
                                                                            value: Expr {
                                                                                depth: 1,
                                                                                kind: Literal(
                                                                                    String(
                                                                                        "json",
                                                                                    ),
                                                                                ),
                                                                                span: Span {
                                                                                    start: 24,
                                                                                    end: 30,
                                                                                },
                                                                            },
                                                                            kind: Data,
                                                                            span: Span {
                                                                                start: 19,
                                                                                end: 30,
                                                                            },
                                                                        },
                                                                    ),
                                                                ],
                                                            ),
                                                            span: Span {
                                                                start: 18,
                                                                end: 31,
                                                            },
                                                        },
                                                        kind: Data,
                                                        span: Span {
                                                            start: 13,
                                                            end: 31,
                                                        },
                                                    },
                                                ),
                                            ],
                                        ),
                                        span: Span {
                                            start: 12,
                                            end: 32,
                                        },
                                    },
                                ),
                            },
                            span: Span {
                                start: 0,
                                end: 34,
                            },
                        },
                    ),
                    span: Span {
                        start: 0,
                        end: 34,
                    },
                },
            ],
            strict: false,
        }
        "#);
}

#[test]
fn import_calls_allow_one_or_two_assignments_and_ordinary_call_suffixes() {
    for source in [
        "import('x')",
        "import('x',)",
        "import('x',undefined,)",
        "import((a,b),a?b:c)",
        "import(/x/.source).then(f)",
        "import(`x${1}`).x",
        "import('x')?.then(f)",
        "import('x')`tag`",
        "new (import('x'))()",
        "new (import('x').Ctor)()",
        "async function f(){return import(await source,await options)}",
        "function*f(){return import(yield 1,yield 2)}",
        "class C{#x;m(){return import(this.#x,{with:{value:this.#x}})}}",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
}

#[test]
fn malformed_arguments_constructor_forms_and_operand_early_errors_remain_syntax() {
    for source in [
        "import()",
        "import(,)",
        "import('x',,)",
        "import('x',1,2)",
        "import(...args)",
        "import('x',...args)",
        "new import('x')",
        "new import('x').Ctor()",
        "import('x')=1",
        "import('x')++",
        "import?.('x')",
        "imp\\u006frt('x')",
        "'use strict';import(delete x)",
        "'use strict';import('x',delete x)",
        "import(object.#missing)",
        "async function f(x=import(await 1)){}",
        "function*f(x=import(yield 1)){}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}
