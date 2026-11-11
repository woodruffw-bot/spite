//! With grammar, strict early errors, and declaration/control-flow scope (14.11).

use spite_core::DiagnosticKind;
use spite_parser::parse_script;

#[test]
fn with_syntax_snapshot() {
    insta::assert_debug_snapshot!(parse_script("with (object) { var x = value; x; }").unwrap(), @r#"
        Script {
            statements: [
                Statement {
                    kind: With {
                        object: Expr {
                            depth: 1,
                            kind: Identifier(
                                "object",
                            ),
                            span: Span {
                                start: 6,
                                end: 12,
                            },
                        },
                        body: Statement {
                            kind: Block(
                                [
                                    Statement {
                                        kind: Var(
                                            [
                                                BindingElement {
                                                    pattern: BindingPattern {
                                                        kind: Identifier(
                                                            "x",
                                                        ),
                                                        span: Span {
                                                            start: 20,
                                                            end: 21,
                                                        },
                                                    },
                                                    initializer: Some(
                                                        Expr {
                                                            depth: 1,
                                                            kind: Identifier(
                                                                "value",
                                                            ),
                                                            span: Span {
                                                                start: 24,
                                                                end: 29,
                                                            },
                                                        },
                                                    ),
                                                },
                                            ],
                                        ),
                                        span: Span {
                                            start: 16,
                                            end: 30,
                                        },
                                    },
                                    Statement {
                                        kind: Expression(
                                            Expr {
                                                depth: 1,
                                                kind: Identifier(
                                                    "x",
                                                ),
                                                span: Span {
                                                    start: 31,
                                                    end: 32,
                                                },
                                            },
                                        ),
                                        span: Span {
                                            start: 31,
                                            end: 33,
                                        },
                                    },
                                ],
                            ),
                            span: Span {
                                start: 14,
                                end: 35,
                            },
                        },
                    },
                    span: Span {
                        start: 0,
                        end: 35,
                    },
                },
            ],
            strict: false,
        }
        "#);
}

#[test]
fn strict_and_malformed_with_diagnostics_snapshot() {
    let errors: Vec<_> = [
        "'use strict'; with ({}) ;",
        "function f() { 'use strict'; with ({}) {} }",
        "with () ;",
        "with ({}) let x = 1;",
        "with ({}) function f() {}",
        "with ({}) class C {}",
        "with ({}) async function f() {}",
    ]
    .map(|source| parse_script(source).unwrap_err())
    .into();
    insta::assert_debug_snapshot!(errors, @r#"
        [
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 14,
                    end: 25,
                },
                message: "with statements are not allowed in strict mode",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 29,
                    end: 41,
                },
                message: "with statements are not allowed in strict mode",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 6,
                    end: 7,
                },
                message: "expected an expression",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 14,
                    end: 15,
                },
                message: "expected a semicolon or line terminator",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 10,
                    end: 18,
                },
                message: "function declaration requires a statement list",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 10,
                    end: 15,
                },
                message: "declaration requires a statement list",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 10,
                    end: 15,
                },
                message: "declaration requires a statement list",
            },
        ]
        "#);
}

#[test]
fn with_is_a_statement_and_preserves_enclosing_control_context() {
    for source in [
        "with ({}) ;",
        "if (true) with ({}) ; else ;",
        "with ('x' in {}) with ({}) {}",
        "with (0, {}) var x;",
        "a: with ({}) break a;",
        "a: while (true) with ({}) continue a;",
        "switch (0) { default: with ({}) break; }",
        "function f() { with ({}) return; }",
        "with ({}) { function f() { 'use strict'; return 1; } }",
        "with ({}) async\nfunction f() {}",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    for source in [
        "with ({}) break;",
        "with ({}) continue;",
        "with ({}) return;",
        "a: with ({}) while (true) continue a;",
        "with ({}) a: function f() {}",
        "with ({}) function* f() {}",
        "with ({}) async function* f() {}",
        r"wi\u0074h ({}) ;",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn strictness_and_var_conflicts_reach_through_with_bodies() {
    for source in [
        "'use strict'; if (false) with ({}) ;",
        "function f(){ 'use strict'; function g(){ with ({}) ; } }",
        "let f = () => { 'use strict'; with ({}) ; };",
        "let x; with ({}) { var x; }",
        "for (let x;;) with ({}) { var x; }",
        "with ({}) { let x; with ({}) var x; }",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    assert!(parse_script("with ({}) { let x; } var x;").is_ok());
    assert!(spite_parser::parse_dynamic_function("", "with ({}) return 7;").is_ok());
    assert_eq!(
        spite_parser::parse_dynamic_function("", "'use strict'; with ({}) return 7;")
            .unwrap_err()
            .kind,
        DiagnosticKind::Syntax,
    );
}
