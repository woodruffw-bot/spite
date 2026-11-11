//! Explicit RegExp goals preserve literal ASTs and precise early errors.

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{
    EvalContext, parse_dynamic_function, parse_eval_utf16, parse_script, parse_script_utf16,
};

#[test]
fn literal_contents_do_not_substitute_unrelated_javascript_diagnostics() {
    let sources = [
        "/[}`]/g",
        "let x = /=\\}/;",
        "(x = /\\)/) => x",
        "({x: /`\\}/})",
        "class C { get [/`\\}/]() {} }",
        "`a${/[}`]/g}b`",
        "/(?:)/",
    ];
    for source in sources {
        parse_script(source).unwrap_or_else(|error| panic!("{source}: {error}"));
    }
}

#[test]
fn literal_boundaries_are_shared_by_script_eval_and_function_goals() {
    let source = "let x = /[}`]/g;";
    let expected = parse_script(source).unwrap();
    assert_eq!(
        parse_script_utf16(&JsString::from(source)).unwrap(),
        expected
    );
    assert_eq!(
        parse_eval_utf16(&JsString::from(source), EvalContext::default()).unwrap(),
        expected
    );
    for (parameters, body) in [("x = /[}`]/g", "return x;"), ("x", "return /[}`]/g;")] {
        parse_dynamic_function(parameters, body).unwrap();
    }
}

#[test]
fn every_flag_subset_uses_exactly_one_unicode_mode() {
    let flags = b"dgimsuvy";
    for subset in 0u16..256 {
        let flags: String = flags
            .iter()
            .enumerate()
            .filter_map(|(index, flag)| (subset & (1 << index) != 0).then_some(char::from(*flag)))
            .collect();
        let source = format!("/a/{flags}");
        let result = parse_script(&source);
        if flags.contains('u') && flags.contains('v') {
            assert_eq!(result.unwrap_err().kind, DiagnosticKind::Syntax, "{source}");
        } else {
            result.unwrap_or_else(|error| panic!("{source}: {error}"));
        }
    }
    for flags in ["yvsmigd", "yusmigd"] {
        parse_script(&format!("/a/{flags}")).unwrap();
    }
}

#[test]
fn flag_early_errors_precede_pattern_validation() {
    let sources = [
        "/./G",
        "/./gig",
        "/a/qq",
        "/a/uv",
        "/a/vu",
        "/a/uvq",
        "/a/uvgg",
        "/(/q",
        "/./0",
        "/./$",
        "/./é",
        "/./𐐀",
        "let x = /a/gg;",
        "class C { get [/a/qq]() {} }",
        "`a${/a/uv}b`",
    ];
    let errors: Vec<_> = sources
        .into_iter()
        .map(|source| {
            let error = parse_script(source).unwrap_err();
            assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
            assert_eq!(
                parse_script_utf16(&JsString::from(source)).unwrap_err(),
                error
            );
            assert_eq!(
                parse_eval_utf16(&JsString::from(source), EvalContext::default()).unwrap_err(),
                error
            );
            error
        })
        .collect();
    insta::assert_debug_snapshot!(errors);
    for (parameters, body) in [("x = /./G", "return x;"), ("x", "return /./gig;")] {
        assert_eq!(
            parse_dynamic_function(parameters, body).unwrap_err().kind,
            DiagnosticKind::Syntax
        );
    }
}

#[test]
fn division_continuations_and_template_substitutions_keep_their_grammar() {
    for source in [
        "let x = 12; x /= 3; x\n/2;",
        "let f = (x = 12 / 3) => `${x / 2}tail`;",
        "`a${({x: 8 / 2}).x}b${`c${12 / 4}d`}e`;",
        "class C { get [`a${12 / 4}b`]() { return 6 / 2; } }",
        "// comment, not an empty literal\n/**/ 1;",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}

#[test]
fn computed_accessor_lookahead_preserves_field_asi_and_private_name_uses() {
    for source in [
        "class C { get\n[12 / 4] = 1; set\n[`x${class D { get [12 / 4]() {} }}`]; }",
        "class C { #x; get [()=>this.#x]() {} }",
        "class C { #x; m() { return class { get [()=>this.#x]() {} }; } }",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    for source in [
        "class C { get [this.#missing]() {} }",
        "class C { get\n[this.#missing]; }",
    ] {
        let error = parse_script(source).unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
        assert_eq!(&source[error.span.start..error.span.end], "#missing");
    }
    let mut name = "0".to_owned();
    for _ in 0..18 {
        name = format!("class {{ get [{name}]() {{}} }}");
    }
    assert!(parse_script(&format!("let C = {name};")).is_ok());
}

#[test]
fn goal_rescans_keep_property_diagnostics_across_cover_and_template_contexts() {
    let sources = [
        r"(x = /[\p{Invalid}]/u) => x",
        r"({x: /\p{WSpace}/v})",
        r"`outer${ { key: /\p{Invalid}/u } }tail`",
        r"class C { get [/\p{Invalid}/u]() {} }",
        r"(x = /\p{gc=Latin}/u)",
    ];
    let errors: Vec<_> = sources
        .into_iter()
        .map(|source| {
            let error = parse_script(source).unwrap_err();
            assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
            assert!(
                error.message.contains("Unicode property"),
                "{source}: {error:?}"
            );
            let utf16 = JsString::from(source);
            assert_eq!(parse_script_utf16(&utf16).unwrap_err(), error);
            assert_eq!(
                parse_eval_utf16(&utf16, EvalContext::default()).unwrap_err(),
                error
            );
            assert_eq!(
                parse_dynamic_function("x", &format!("return {source};"))
                    .unwrap_err()
                    .kind,
                DiagnosticKind::Syntax
            );
            error
        })
        .collect();
    insta::assert_debug_snapshot!(errors, @r#"
        [
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 5,
                    end: 21,
                },
                message: "unknown regular expression Unicode property",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 5,
                    end: 18,
                },
                message: "unknown regular expression Unicode property",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 16,
                    end: 30,
                },
                message: "unknown regular expression Unicode property",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 15,
                    end: 29,
                },
                message: "unknown regular expression Unicode property",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 5,
                    end: 20,
                },
                message: "unknown regular expression Unicode property value",
            },
        ]
        "#);
}

#[test]
fn malformed_literal_boundary_diagnostics_snapshot() {
    let sources = [
        "/",
        "/a\\",
        "/[a/",
        "/a\n/",
        "/[\\\u{2028}]/",
        "let x = /a\r/;",
    ];
    let errors: Vec<_> = sources
        .into_iter()
        .map(|source| parse_script(source).unwrap_err())
        .collect();
    assert!(
        errors
            .iter()
            .all(|error| error.kind == DiagnosticKind::Syntax)
    );
    insta::assert_debug_snapshot!(errors, @r#"
        [
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 0,
                    end: 1,
                },
                message: "unterminated regular expression literal",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 0,
                    end: 3,
                },
                message: "unterminated regular expression escape",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 0,
                    end: 4,
                },
                message: "unterminated regular expression literal",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 0,
                    end: 3,
                },
                message: "line terminator in regular expression literal",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 0,
                    end: 6,
                },
                message: "line terminator in regular expression literal",
            },
            Diagnostic {
                kind: Syntax,
                span: Span {
                    start: 8,
                    end: 11,
                },
                message: "line terminator in regular expression literal",
            },
        ]
        "#);
}

#[test]
fn required_numeric_digits_highlight_complete_characters_and_preserve_eof() {
    for (source, start, end) in [
        ("0b2n", 2, 3),
        ("0o8", 2, 3),
        ("0xg", 2, 3),
        ("0xπ", 2, 4),
        ("0b😀", 2, 6),
        ("1e+π", 3, 5),
        ("0x", 2, 2),
        ("1e+", 3, 3),
    ] {
        let error = parse_script(source).unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
        assert_eq!(error.span, spite_core::Span::new(start, end), "{source}");
        assert_eq!(error.message, "expected a digit", "{source}");
    }
}
