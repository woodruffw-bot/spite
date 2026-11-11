//! NewTarget early errors cross arrows and stop at ordinary function boundaries.

use spite_core::DiagnosticKind;
use spite_parser::parse_script;

#[test]
fn new_target_in_parameters_and_arrow_bodies_snapshot() {
    insta::assert_debug_snapshot!(parse_script("function F(x = new.target) { return ()=>new.target; }").unwrap(), @r#"
        Script {
            statements: [
                Statement {
                    kind: Function(
                        Function {
                            kind: Ordinary,
                            name: Some(
                                FunctionName {
                                    name: "F",
                                    span: Span {
                                        start: 9,
                                        end: 10,
                                    },
                                },
                            ),
                            parameters: [
                                Ordinary(
                                    BindingElement {
                                        pattern: BindingPattern {
                                            kind: Identifier(
                                                "x",
                                            ),
                                            span: Span {
                                                start: 11,
                                                end: 12,
                                            },
                                        },
                                        initializer: Some(
                                            Expr {
                                                depth: 1,
                                                kind: NewTarget,
                                                span: Span {
                                                    start: 15,
                                                    end: 25,
                                                },
                                            },
                                        ),
                                    },
                                ),
                            ],
                            body: FunctionBody {
                                statements: [
                                    Statement {
                                        kind: Return(
                                            Some(
                                                Expr {
                                                    depth: 2,
                                                    kind: Arrow {
                                                        kind: Ordinary,
                                                        parameters: [],
                                                        body: Expression(
                                                            Expr {
                                                                depth: 1,
                                                                kind: NewTarget,
                                                                span: Span {
                                                                    start: 40,
                                                                    end: 50,
                                                                },
                                                            },
                                                        ),
                                                        source: FunctionSource(
                                                            "()=>new.target",
                                                        ),
                                                    },
                                                    span: Span {
                                                        start: 36,
                                                        end: 50,
                                                    },
                                                },
                                            ),
                                        ),
                                        span: Span {
                                            start: 29,
                                            end: 51,
                                        },
                                    },
                                ],
                                strict: false,
                            },
                            source: FunctionSource(
                                "function F(x = new.target) { return ()=>new.target; }",
                            ),
                        },
                    ),
                    span: Span {
                        start: 0,
                        end: 53,
                    },
                },
            ],
            strict: false,
        }
        "#);
}

#[test]
fn ordinary_functions_enable_new_target_in_parameters_and_nested_arrows() {
    for source in [
        "function F(){return new.target;}",
        "'use strict';function F(){return new.target;}",
        "function F(x=new.target){}",
        "function F(x=()=>new.target){return ()=>new.target;}",
        "function F(){return (x=new.target)=>new.target;}",
        "()=>function F(){return new.target;}",
        "(x=function F(y=new.target){})=>x",
        "function F(){return new\n.\ntarget;}",
        "function F(){return new new.target();}",
        "function F(){new.target.prototype.x=1;return delete new.target;}",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
}

#[test]
fn top_level_and_arrow_only_contexts_reject_new_target() {
    for source in [
        "new.target",
        "(()=>new.target)",
        "(x=new.target)=>x",
        "()=>{return new.target;}",
        "function F(){}new.target",
        "(function F(){}) + new.target",
        "(x=function(){})=>new.target",
        "function F(){return new.target;}()=>new.target",
        "function F(){new.target=1;}",
        "function F(){new.target++;}",
        "function F(){++new.target;}",
        r"function F(){return new.\u0074arget;}",
        "function F(){return new.other;}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}
