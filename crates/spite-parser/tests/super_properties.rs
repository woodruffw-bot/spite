//! Method-scoped SuperProperty grammar and reference targets (13.3.7).

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{EvalContext, parse_eval_utf16, parse_script};

#[test]
fn super_properties_retain_names_and_reference_targets() {
    let context = EvalContext {
        in_function: true,
        in_method: true,
        ..Default::default()
    };
    let script = parse_eval_utf16(&JsString::from("super.x = super[1];"), context).unwrap();
    insta::assert_debug_snapshot!(script, @r#"
        Script {
            statements: [
                Statement {
                    kind: Expression(
                        Expr {
                            depth: 3,
                            kind: Assign(
                                Expr {
                                    depth: 1,
                                    kind: SuperProperty(
                                        Literal(
                                            String(
                                                "x",
                                            ),
                                        ),
                                    ),
                                    span: Span {
                                        start: 0,
                                        end: 7,
                                    },
                                },
                                Expr {
                                    depth: 2,
                                    kind: SuperProperty(
                                        Computed(
                                            Expr {
                                                depth: 1,
                                                kind: Literal(
                                                    Number(
                                                        1.0,
                                                    ),
                                                ),
                                                span: Span {
                                                    start: 16,
                                                    end: 17,
                                                },
                                            },
                                        ),
                                    ),
                                    span: Span {
                                        start: 10,
                                        end: 18,
                                    },
                                },
                            ),
                            span: Span {
                                start: 0,
                                end: 18,
                            },
                        },
                    ),
                    span: Span {
                        start: 0,
                        end: 19,
                    },
                },
            ],
            strict: false,
        }
        "#);
    for source in [
        "({m(k){return super[k in {}];}})",
        "({m(){return super.null + super.true + super.\\u0078;}})",
        "({m(){super.x++; ++super.x; super.x ||= 7;}})",
        "({m(){[super.x]=[1]; ({x:super.x}={x:2});}})",
        "({m(){for(super.x of []){} for(super.x in {}){}}})",
        "({m(x=super.x){return ()=>super[x];}})",
        "({get x(){return super.x;},set x(v){super.x=v;}})",
        "({m(){return super.m?.() + super.tag`x`;}})",
        "({m(){return new super.F();}})",
        "({m(){delete (super.x);}})",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}

#[test]
fn super_grammar_and_context_errors_are_syntax_errors() {
    let errors: Vec<_> = [
        "super.x",
        "()=>super.x",
        "function f(){return super.x;}",
        "({m(){return function(){return ()=>super.x;};}})",
        "({m(){super();}})",
        "({m(){super;}})",
        "({m(){super?.x;}})",
        "({m(){super.`x`;}})",
        "({m(){super[ ];}})",
        "({m(){super.#x;}})",
        "({m(){'use strict';super[public];}})",
    ]
    .into_iter()
    .map(|source| {
        let error = parse_script(source).unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
        error
    })
    .collect();
    insta::assert_debug_snapshot!(errors, @r#"
        [
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
                    start: 4,
                    end: 9,
                },
                message: "super property access outside a method",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 20,
                    end: 25,
                },
                message: "super property access outside a method",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 35,
                    end: 40,
                },
                message: "super property access outside a method",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 6,
                    end: 11,
                },
                message: "super call requires a derived constructor",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 6,
                    end: 11,
                },
                message: "super call requires a derived constructor",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 6,
                    end: 11,
                },
                message: "super call requires a derived constructor",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 12,
                    end: 15,
                },
                message: "expected an identifier name after dot",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 13,
                    end: 14,
                },
                message: "expected an expression",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 12,
                    end: 13,
                },
                message: "super cannot access private properties",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 25,
                    end: 31,
                },
                message: "reserved identifier in strict mode",
            },
        ]
        "#);
}

#[test]
fn computed_super_names_remain_subject_to_parser_depth_guards() {
    let source = format!(
        "({{m(){{return super[{}0{}];}}}})",
        "(".repeat(80),
        ")".repeat(80)
    );
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}
