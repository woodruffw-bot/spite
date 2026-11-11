//! Var binding patterns, BoundNames, and header grammar (14.3.2, 14.7).

use spite_core::DiagnosticKind;
use spite_parser::{MAX_DEPTH, parse_script};

#[test]
fn var_binding_pattern_syntax_snapshot() {
    insta::assert_debug_snapshot!(parse_script("var {a:[x=1],...rest}=source;").unwrap(), @r#"
        Script {
            statements: [
                Statement {
                    kind: Var(
                        [
                            BindingElement {
                                pattern: BindingPattern {
                                    kind: Object {
                                        properties: [
                                            BindingProperty {
                                                key: Literal(
                                                    String(
                                                        "a",
                                                    ),
                                                ),
                                                element: BindingElement {
                                                    pattern: BindingPattern {
                                                        kind: Array {
                                                            elements: [
                                                                Some(
                                                                    BindingElement {
                                                                        pattern: BindingPattern {
                                                                            kind: Identifier(
                                                                                "x",
                                                                            ),
                                                                            span: Span {
                                                                                start: 8,
                                                                                end: 9,
                                                                            },
                                                                        },
                                                                        initializer: Some(
                                                                            Expr {
                                                                                depth: 1,
                                                                                kind: Literal(
                                                                                    Number(
                                                                                        1.0,
                                                                                    ),
                                                                                ),
                                                                                span: Span {
                                                                                    start: 10,
                                                                                    end: 11,
                                                                                },
                                                                            },
                                                                        ),
                                                                    },
                                                                ),
                                                            ],
                                                            rest: None,
                                                        },
                                                        span: Span {
                                                            start: 7,
                                                            end: 12,
                                                        },
                                                    },
                                                    initializer: None,
                                                },
                                            },
                                        ],
                                        rest: Some(
                                            BindingPattern {
                                                kind: Identifier(
                                                    "rest",
                                                ),
                                                span: Span {
                                                    start: 16,
                                                    end: 20,
                                                },
                                            },
                                        ),
                                    },
                                    span: Span {
                                        start: 4,
                                        end: 21,
                                    },
                                },
                                initializer: Some(
                                    Expr {
                                        depth: 1,
                                        kind: Identifier(
                                            "source",
                                        ),
                                        span: Span {
                                            start: 22,
                                            end: 28,
                                        },
                                    },
                                ),
                            },
                        ],
                    ),
                    span: Span {
                        start: 0,
                        end: 29,
                    },
                },
            ],
            strict: false,
        }
        "#);
}

#[test]
fn var_binding_pattern_diagnostics_snapshot() {
    insta::assert_debug_snapshot!([
            "var [x];",
            "let x;var {a:x}={};",
            "try{}catch([x]){var {x}={};}",
            "'use strict';var {a:eval}={};",
            "for(var [x]=[] in {}) ;",
            "for(var [x,...rest,] of []) ;",
        ]
        .map(|source| parse_script(source).unwrap_err()), @r#"
        [
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 7,
                    end: 8,
                },
                message: "binding pattern requires an initializer",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 13,
                    end: 14,
                },
                message: "var declaration conflicts with lexical binding",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 21,
                    end: 22,
                },
                message: "catch parameter conflicts with var declaration",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 20,
                    end: 24,
                },
                message: "invalid binding in strict mode",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 12,
                    end: 14,
                },
                message: "for-in requires one binding without an initializer",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 18,
                    end: 19,
                },
                message: "expected ]",
            },
        ]
        "#);
}

#[test]
fn patterns_allow_repeated_var_names_and_every_declaration_header() {
    for pattern in [
        "{}",
        "[]",
        "[x,x]",
        "{x,a:x,...x}",
        "{let,await,yield}",
        "[x=key in source,...{length}]",
        "{[key in source]:x}",
    ] {
        for source in [
            format!("var {pattern}=source;"),
            format!("for(var {pattern}=source;false;) ;"),
            format!("for(var {pattern} of source) ;"),
            format!("for(var {pattern} in source) ;"),
        ] {
            assert!(parse_script(&source).is_ok(), "{source}");
        }
    }
    assert!(parse_script("var a,{x}={},[y]=[],z;").is_ok());
    assert!(parse_script("function f(a){var [a]=[];}").is_ok());
    assert!(parse_script("function f(){var {arguments}={};}").is_ok());
    assert!(parse_script("for(let x of []) {function f(){var {x}={};}}").is_ok());
}

#[test]
fn all_nested_bound_names_conflict_with_enclosing_lexical_and_catch_names() {
    for source in [
        "var {} ;",
        "for(var [];;) ;",
        "let x;var {a:[x]}={};",
        "{let x;if(false)var [x]=[];}",
        "try{}catch(x){var [x]=[];}",
        "for(let x;;)var {x}={};",
        "for(const x of [])for(var [x] in {}) ;",
        "'use strict';var [arguments]=[];",
        "'use strict';var {a=delete name}={};",
        "for(var [x]=[] of []) ;",
        "for(var {x}=source in {}) ;",
        "for(var [x],y of []) ;",
        "for(var {x},y in {}) ;",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    assert_eq!(
        parse_script("for(var x=1 in {}) ;").unwrap_err().kind,
        DiagnosticKind::Syntax
    );
}

#[test]
fn var_declared_names_traverse_every_pattern_in_source_order_and_stop_at_functions() {
    let script=parse_script("if(false)var {a:[x,x],...rest}={};with({})var {b:y}={};for(var [z] of [])var tail;function f(){var [hidden]=[];}").unwrap();
    assert_eq!(
        script
            .var_declarations()
            .iter()
            .map(|name| name.name)
            .collect::<Vec<_>>(),
        ["x", "x", "rest", "y", "z", "tail"]
    );
    for name in script.var_declarations() {
        assert_eq!(
            &"if(false)var {a:[x,x],...rest}={};with({})var {b:y}={};for(var [z] of [])var tail;function f(){var [hidden]=[];}"
                [name.span.start..name.span.end],
            name.name
        );
    }
}

#[test]
fn nested_patterns_use_the_existing_native_depth_guard() {
    let source = format!(
        "var {}x{}=[];",
        "[".repeat(MAX_DEPTH),
        "]".repeat(MAX_DEPTH)
    );
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
    for source in ["([x]=source);", "({x}=source);", "for([x] of []) ;"] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}
