//! Core Pattern validation precedes the separate matching implementation.

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{
    EvalContext, parse_dynamic_function, parse_eval_utf16, parse_script, parse_script_utf16,
};

fn validates(pattern: &str, flags: &str) {
    use spite_parser::ast::{ExprKind, Literal, StatementKind};
    let source = format!("/{pattern}/{flags}");
    let script = parse_script(&source).unwrap_or_else(|error| panic!("{source}: {error}"));
    let StatementKind::Expression(expr) = &script.statements()[0].kind else {
        panic!("expected an expression")
    };
    let ExprKind::Literal(Literal::RegExp {
        body,
        flags: actual_flags,
    }) = &expr.kind
    else {
        panic!("expected RegExp literal")
    };
    assert_eq!(body, &JsString::from(pattern));
    assert_eq!(actual_flags, &JsString::from(flags));
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
            validates(pattern, flags);
        }
    }
    for pattern in [
        r"\u{0}",
        r"\u{10ffff}",
        r"\u{d800}",
        r"\u{00000000000000000000000041}",
    ] {
        validates(pattern, "u");
        validates(pattern, "v");
    }
    for pattern in [r"\-\!\$", "\\😀", "\\𐐀", "\\\u{2000}"] {
        validates(pattern, "");
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
fn valid_properties_reach_matching_and_flags_are_checked_first() {
    for source in [r"/\p{Letter}/u", r"/\P{Letter}/v"] {
        parse_script(source).unwrap();
    }
    // Flags are validated before the Pattern grammar.
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
    validates(&format!("a{{{huge},{huge}}}"), "u");
    validates(&format!("a{{000{huge},1{huge}}}"), "");
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
    validates(r"\12()()()()()()()()()()()()", "u");
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
    validates(&pattern, "");
    validates(&pattern, "v");
    for flags in ["", "u", "v"] {
        for units in [[0xd800, 0xdc00], [0xd800, 0xd800], [0xdc00, 0xdc00]] {
            let mut source = vec![u16::from(b'/')];
            source.extend(units);
            source.push(u16::from(b'/'));
            source.extend(flags.encode_utf16());
            parse_script_utf16(&JsString::from_code_units(source)).unwrap();
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
            let result = parse_script(&source);
            if enabled | disabled == 0 || enabled & disabled != 0 {
                assert_eq!(result.unwrap_err().kind, DiagnosticKind::Syntax, "{source}");
            } else {
                result.unwrap_or_else(|error| panic!("{source}: {error}"));
            }
        }
    }
}

#[test]
fn ordinary_classes_preserve_dash_backspace_and_set_escape_grammar() {
    for flags in ["", "u", "dgimsy"] {
        for pattern in [
            "[]",
            "[^]",
            "[a-zA-Z0-9_]",
            "[-a]",
            "[a-]",
            "[--a]",
            "[---a]",
            "[a-c-e]",
            "[a-c--e]",
            "[a-c--]",
            "[^a-z]+?",
            "[[]",
            "[(){}|]",
            r"[\b-\t]",
            r"[\x00-\u007f]",
            r"[\cA-\cZ]",
            r"[\0-\x01]",
            r"[\d\D\s\S\w\W]",
            r"[-\d]",
            r"[\d-]",
            r"[\--a]",
            r"[\^\$\\\.\*\+\?\(\)\[\]\{\}\|\/]",
            r"[\uD800-\uDBFF]",
            r"[\uDC00-\uDFFF]",
        ] {
            validates(pattern, flags);
        }
    }
    // Ordinary classes do not acquire the UnicodeSetsMode grammar.
    for (pattern, expected) in [
        ("[a&&b]", None),
        ("[!!]", Some(DiagnosticKind::Syntax)),
        ("[{}]", Some(DiagnosticKind::Syntax)),
        ("[|]", Some(DiagnosticKind::Syntax)),
    ] {
        validates(pattern, "u");
        assert_eq!(
            parse_script(&format!("/{pattern}/v"))
                .err()
                .map(|error| error.kind),
            expected
        );
    }
    validates(r"[\!\$]", "");
    for pattern in [r"[\p{Letter}]", r"[\P{Letter}]"] {
        parse_script(&format!("/{pattern}/u")).unwrap();
    }
}

#[test]
fn class_range_errors_share_exact_diagnostics_across_grammar_goals() {
    for flags in ["", "u"] {
        for pattern in [
            "[z-a]",
            "[a--b]",
            r"[\d-a]",
            r"[a-\D]",
            r"[\s-\w]",
            r"[\t-\b]",
            r"[\cZ-\cA]",
            r"[\x80-\u007f]",
            r"[\B]",
            r"[\1]",
            r"[\k<a>]",
            r"[\01]",
            r"[\c0]",
            r"[\x0]",
            r"[\u000]",
            r"[\a]",
        ] {
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
    for (parameters, body) in [(r"x = /[a-\d]/", "return x;"), ("x", "return /[z-a]/u;")] {
        assert_eq!(
            parse_dynamic_function(parameters, body).unwrap_err().kind,
            DiagnosticKind::Syntax
        );
    }
    let errors: Vec<_> = [
        "/[z-a]/",
        r"/[\d-a]/u",
        r"/[a-\s]/",
        r"/[\t-\b]/u",
        r"/[\B]/u",
        r"/[\1]/",
        r"/[\!]/u",
        "class C { get [/[z-a]/u]() {} }",
    ]
    .into_iter()
    .map(|source| parse_script(source).unwrap_err())
    .collect();
    insta::assert_debug_snapshot!(errors);
}

#[test]
fn range_values_pair_only_adjacent_hex_surrogate_escapes_in_unicode_mode() {
    for (pattern, ordinary, unicode) in [
        (r"[\uD800\uDC00-\uD800\uDC01]", false, true),
        (r"[\uD800\uDC01-\uD800\uDC00]", false, false),
        (r"[\uD800\uDC00-\uFFFF]", true, false),
        (r"[\uFFFF-\uD800\uDC00]", false, true),
        (r"[\uD800\uD800\uDC00-\u{10001}]", false, true),
        (r"[\uD800\u{dc00}-\uDC01]", false, true),
        ("[😀-😁]", false, true),
    ] {
        for (flags, valid) in [("", ordinary), ("u", unicode)] {
            let result = parse_script(&format!("/{pattern}/{flags}"));
            if valid {
                result.unwrap();
            } else {
                assert_eq!(
                    result.unwrap_err().kind,
                    DiagnosticKind::Syntax,
                    "{pattern} {flags}"
                );
            }
        }
    }
    validates(r"[\u{10000}-\u{10ffff}]", "u");
    for flags in ["", "u"] {
        for (left, right, valid) in [
            (0xd800, 0xdbff, true),
            (0xdbff, 0xd800, false),
            (0xdc00, 0xdfff, true),
        ] {
            let mut units: Vec<u16> = "/[".encode_utf16().collect();
            units.extend([left, u16::from(b'-'), right]);
            units.extend("]/".encode_utf16());
            units.extend(flags.encode_utf16());
            let result = parse_script_utf16(&JsString::from_code_units(units));
            if valid {
                result.unwrap();
            } else {
                assert_eq!(result.unwrap_err().kind, DiagnosticKind::Syntax);
            }
        }
    }
    validates(&format!("[{}]", "a".repeat(100_000)), "u");
}
