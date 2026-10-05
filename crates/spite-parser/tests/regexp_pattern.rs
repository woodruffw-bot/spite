//! Core Pattern validation precedes the separate matching implementation.

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{
    EvalContext, parse_dynamic_function, parse_eval_utf16, parse_script, parse_script_utf16,
};

fn matching_gap(pattern: &str, flags: &str) {
    let source = format!("/{pattern}/{flags}");
    let error = parse_script(&source).unwrap_err();
    assert_eq!(error.kind, DiagnosticKind::Unsupported, "{source}");
    assert_eq!(
        error.message, "regular expression matching is not implemented",
        "{source}"
    );
}

#[test]
fn core_groups_assertions_quantifiers_and_escapes_validate_without_matching() {
    for flags in ["", "u", "v", "dgimsy"] {
        for pattern in [
            "a|",
            "|a",
            "||",
            "()",
            "(?:)",
            "(a|b(c))",
            "(?=a)",
            "(?!a)",
            "(?<=a)",
            "(?<!a)",
            "^a$",
            r"\bword\B",
            "a*?",
            "a+?",
            "a??",
            "(?:a){0001,0002}?",
            "a{0}",
            "a{001,}",
            "(a)\\1",
            r"\1(a)",
            r"\2(a)(b)",
            r"\d\D\s\S\w\W",
            r"\f\n\r\t\v\cA\cz\0",
            r"\x00\xAf\u0000\uD800\uDC00",
            r"\^\$\\\.\*\+\?\(\)\[\]\{\}\|\/",
            "(?ims:a)",
            "(?-ims:a)",
            "(?i-ms:a)*",
            "(?ims-:a)",
            "😀*",
            "💩|é",
        ] {
            matching_gap(pattern, flags);
        }
    }
    for pattern in [
        r"\u{0}",
        r"\u{10ffff}",
        r"\u{d800}",
        r"\u{00000000000000000000000041}",
    ] {
        matching_gap(pattern, "u");
        matching_gap(pattern, "v");
    }
    for pattern in [r"\-\!\$", "\\😀", "\\𐐀", "\\\u{2000}"] {
        matching_gap(pattern, "");
    }
}

#[test]
fn malformed_core_patterns_have_shared_syntax_diagnostics() {
    let patterns = [
        "(",
        ")",
        "a)",
        "(?:a",
        "(?q:a)",
        "(?i)",
        "(?ii:a)",
        "(?i-mm:a)",
        "(?-:a)",
        "(?i-i:a)",
        "*a",
        "a**",
        "a???",
        "a{",
        "a{}",
        "a{,2}",
        "a{1,2,3}",
        "a{2,1}",
        "a{0002,1}",
        "a{1}*",
        "(?=a)+",
        "(?<!a){2}",
        "^*",
        r"\b?",
        "]",
        "}",
        r"\1",
        r"(a)\2",
        r"\01",
        r"\00",
        r"\c0",
        r"\x0",
        r"\u000",
        r"\a",
        r"\_",
        r"\é",
    ];
    for flags in ["", "u", "v"] {
        for pattern in patterns {
            let source = format!("/{pattern}/{flags}");
            let expected = parse_script(&source).unwrap_err();
            assert_eq!(expected.kind, DiagnosticKind::Syntax, "{source}");
            assert_eq!(
                parse_script_utf16(&JsString::from(source.as_str())).unwrap_err(),
                expected
            );
            assert_eq!(
                parse_eval_utf16(&JsString::from(source.as_str()), EvalContext::default())
                    .unwrap_err(),
                expected
            );
        }
    }
    for (parameters, body) in [("x = /(/", "return x;"), ("x", r"return /\1/;")] {
        assert_eq!(
            parse_dynamic_function(parameters, body).unwrap_err().kind,
            DiagnosticKind::Syntax
        );
    }
    for source in [
        "let x = /(/;",
        "(x = /(/) => x",
        "class C { get [/(/]() {} }",
        "`a${/(/}b`",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    for pattern in [
        r"\u{}",
        r"\u{110000}",
        r"\u{100000000000000000000}",
        r"\-",
        "\\😀",
    ] {
        assert_eq!(
            parse_script(&format!("/{pattern}/u")).unwrap_err().kind,
            DiagnosticKind::Syntax
        );
    }
    assert_eq!(
        parse_script(r"/\u{41}/").unwrap_err().kind,
        DiagnosticKind::Syntax
    );
    let errors: Vec<_> = [
        "/(/",
        "/a{2,1}/",
        r"/(a)\2/u",
        "/(?i-i:a)/",
        r"/\é/u",
        "/(?=a)*/",
    ]
    .into_iter()
    .map(|source| parse_script(source).unwrap_err())
    .collect();
    insta::assert_debug_snapshot!(errors);
}

#[test]
fn unsupported_pattern_productions_do_not_receive_negative_credit() {
    for source in [
        "/[z-a]/u",
        "/[a&&]/v",
        "/(?<a>a)(?<a>b)/u",
        r"/\k<missing>/",
        r"/\p{Invalid}/u",
        r"/\P{Invalid}/v",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
    // Flags are validated before every unsupported Pattern production.
    for pattern in ["[z-a]", "(?<a>a)", r"\p{Invalid}"] {
        assert_eq!(
            parse_script(&format!("/{pattern}/uv")).unwrap_err().kind,
            DiagnosticKind::Syntax
        );
    }
}

#[test]
fn arbitrary_decimal_bounds_and_forward_references_are_compared_exactly() {
    let huge = "9".repeat(20_000);
    matching_gap(&format!("a{{{huge},{huge}}}"), "u");
    matching_gap(&format!("a{{000{huge},1{huge}}}"), "");
    assert_eq!(
        parse_script(&format!("/a{{1{huge},{huge}}}/"))
            .unwrap_err()
            .kind,
        DiagnosticKind::Syntax
    );
    assert_eq!(
        parse_script(&format!("/\\{huge}(a)/")).unwrap_err().kind,
        DiagnosticKind::Syntax
    );
    matching_gap(r"\12()()()()()()()()()()()()", "u");
    assert_eq!(
        parse_script(r"/\13()()()()()()()()()()()()/")
            .unwrap_err()
            .kind,
        DiagnosticKind::Syntax
    );
}

#[test]
fn nested_groups_and_raw_surrogates_do_not_require_a_default_depth_quota() {
    let pattern = format!("{}a{}", "(".repeat(20_000), ")".repeat(20_000));
    matching_gap(&pattern, "");
    matching_gap(&pattern, "v");
    for flags in ["", "u", "v"] {
        for units in [[0xd800, 0xdc00], [0xd800, 0xd800], [0xdc00, 0xdc00]] {
            let mut source = vec![u16::from(b'/')];
            source.extend(units);
            source.push(u16::from(b'/'));
            source.extend(flags.encode_utf16());
            let error = parse_script_utf16(&JsString::from_code_units(source)).unwrap_err();
            assert_eq!(error.kind, DiagnosticKind::Unsupported);
            assert_eq!(
                error.message,
                "regular expression matching is not implemented"
            );
        }
    }
}

#[test]
fn scoped_modifier_lists_obey_their_complete_early_errors() {
    for modifiers in ["-q", "-I", "-ſ", r"-\u0069", "g-", "d", "i\u{200d}"] {
        let error = parse_script(&format!("/(?{modifiers}:a)/")).unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Syntax);
        assert_eq!(error.message, "invalid regular expression group prefix");
    }
    for enabled in 0..8 {
        for disabled in 0..8 {
            let list = |bits| {
                (*b"ims")
                    .into_iter()
                    .enumerate()
                    .filter_map(|(index, byte)| {
                        (bits & (1 << index) != 0).then_some(char::from(byte))
                    })
                    .collect::<String>()
            };
            let source = format!("/(?{}-{}:a)/", list(enabled), list(disabled));
            let error = parse_script(&source).unwrap_err();
            assert_eq!(
                error.kind,
                if enabled | disabled == 0 || enabled & disabled != 0 {
                    DiagnosticKind::Syntax
                } else {
                    DiagnosticKind::Unsupported
                },
                "{source}"
            );
        }
    }
}
