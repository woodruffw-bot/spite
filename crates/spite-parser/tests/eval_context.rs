//! Direct eval's inherited Script early errors (19.2.1.1).
use spite_core::{DiagnosticKind, JsString};
use spite_parser::{EvalContext, parse_eval_utf16, parse_eval_utf16_with_private_names};
use std::collections::BTreeSet;

#[test]
fn direct_eval_inherits_strictness_and_constructor_context_without_allowing_return() {
    let context = EvalContext {
        strict: true,
        in_function: true,
        in_method: false,
        in_derived_constructor: false,
        in_class_field_initializer: false,
    };
    let script = parse_eval_utf16(
        &JsString::from("let target=new.target;(()=>target);"),
        context,
    )
    .unwrap();
    assert!(script.is_strict());
    insta::assert_debug_snapshot!(script, @r#"
        Script {
            statements: [
                Statement {
                    kind: Lexical {
                        mutable: true,
                        bindings: [
                            BindingElement {
                                pattern: BindingPattern {
                                    kind: Identifier(
                                        "target",
                                    ),
                                    span: Span {
                                        start: 4,
                                        end: 10,
                                    },
                                },
                                initializer: Some(
                                    Expr {
                                        depth: 1,
                                        kind: NewTarget,
                                        span: Span {
                                            start: 11,
                                            end: 21,
                                        },
                                    },
                                ),
                            },
                        ],
                    },
                    span: Span {
                        start: 0,
                        end: 22,
                    },
                },
                Statement {
                    kind: Expression(
                        Expr {
                            depth: 3,
                            kind: Parenthesized(
                                Expr {
                                    depth: 2,
                                    kind: Arrow {
                                        kind: Ordinary,
                                        parameters: [],
                                        body: Expression(
                                            Expr {
                                                depth: 1,
                                                kind: Identifier(
                                                    "target",
                                                ),
                                                span: Span {
                                                    start: 27,
                                                    end: 33,
                                                },
                                            },
                                        ),
                                        source: FunctionSource(
                                            "()=>target",
                                        ),
                                    },
                                    span: Span {
                                        start: 23,
                                        end: 33,
                                    },
                                },
                            ),
                            span: Span {
                                start: 22,
                                end: 34,
                            },
                        },
                    ),
                    span: Span {
                        start: 22,
                        end: 35,
                    },
                },
            ],
            strict: true,
        }
        "#);
    let diagnostics: Vec<_> = [
        "with({}){}",
        "var eval;",
        "010;",
        "'\\1';",
        "delete x;",
        "return 7;",
        "super.x",
        "super()",
    ]
    .into_iter()
    .map(|source| {
        let error = parse_eval_utf16(&JsString::from(source), context).unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
        error
    })
    .collect();
    insta::assert_debug_snapshot!(diagnostics, @r#"
        [
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 0,
                    end: 10,
                },
                message: "with statements are not allowed in strict mode",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 4,
                    end: 8,
                },
                message: "invalid binding in strict mode",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 0,
                    end: 3,
                },
                message: "legacy numeric literals and escapes are forbidden in strict code",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 0,
                    end: 4,
                },
                message: "legacy numeric literals and escapes are forbidden in strict code",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 7,
                    end: 8,
                },
                message: "cannot delete an identifier in strict mode",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 0,
                    end: 6,
                },
                message: "return outside a function body",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 0,
                    end: 5,
                },
                message: "super property access outside a method",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 0,
                    end: 5,
                },
                message: "super call requires a derived constructor",
            },
        ]
        "#);
}

#[test]
fn method_context_does_not_cross_ordinary_functions() {
    let context = EvalContext {
        in_function: true,
        in_method: true,
        ..Default::default()
    };
    for source in ["super.x", "()=>super.x"] {
        assert!(parse_eval_utf16(&JsString::from(source), context).is_ok());
    }
    for source in [
        "function f(){super.x;}",
        "super()",
        "function f(){return ()=>super.x;}",
    ] {
        assert_eq!(
            parse_eval_utf16(&JsString::from(source), context)
                .unwrap_err()
                .kind,
            DiagnosticKind::Syntax
        );
    }
    assert!(
        parse_eval_utf16(
            &JsString::from("function f(){return new.target;}"),
            EvalContext::default()
        )
        .is_ok()
    );
    assert_eq!(
        parse_eval_utf16(&JsString::from("()=>new.target"), EvalContext::default())
            .unwrap_err()
            .kind,
        DiagnosticKind::Syntax
    );
}

#[test]
fn eval_private_names_cross_functions_and_allow_class_body_shadowing() {
    let names = BTreeSet::from(["#x".into(), "#y".into()]);
    for source in [
        "this.#x; #y in this;",
        "function f(o){return o.#x;} (()=>this.#y);",
        "class C extends (this.#x){#x;read(o){return o.#x;}}",
        "class C{read(o){return o.#y;}#x;}",
        "class C{#x;make(){return class{read(o){return o.#x;}};}}",
    ] {
        assert!(
            parse_eval_utf16_with_private_names(
                &JsString::from(source),
                EvalContext::default(),
                &names
            )
            .is_ok(),
            "{source}"
        );
    }
    let diagnostics: Vec<_> = [
        "o.#missing",
        "function f(o){return #missing in o;}",
        "class C extends (o.#missing){#missing;}",
        "class C{read(o){return o.#missing;}}",
        "delete o.#x",
    ]
    .into_iter()
    .map(|source| {
        let error = parse_eval_utf16_with_private_names(
            &JsString::from(source),
            EvalContext::default(),
            &names,
        )
        .unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
        error
    })
    .collect();
    insta::assert_debug_snapshot!(diagnostics, @r#"
        [
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 2,
                    end: 10,
                },
                message: "private identifier is not declared in an enclosing class",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 21,
                    end: 29,
                },
                message: "private identifier is not declared in an enclosing class",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 19,
                    end: 27,
                },
                message: "private identifier is not declared in an enclosing class",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 25,
                    end: 33,
                },
                message: "private identifier is not declared in an enclosing class",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 7,
                    end: 11,
                },
                message: "cannot delete a private element",
            },
        ]
        "#);
    assert_eq!(
        parse_eval_utf16(&JsString::from("o.#x"), EvalContext::default())
            .unwrap_err()
            .kind,
        DiagnosticKind::Syntax
    );
}
