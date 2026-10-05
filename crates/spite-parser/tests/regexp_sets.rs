//! UnicodeSetsMode ClassUnion and string-disjunction early errors.

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{
    EvalContext, parse_dynamic_function, parse_eval_utf16, parse_script, parse_script_utf16,
};

fn validates(pattern: &str) {
    let source = format!("/{pattern}/v");
    let error = parse_script(&source).unwrap_err();
    assert_eq!(error.kind, DiagnosticKind::Unsupported, "{source}");
    assert_eq!(
        error.message, "regular expression matching is not implemented",
        "{source}"
    );
}

#[test]
fn flat_class_unions_and_scalar_ranges_validate_in_unicode_sets_mode() {
    for pattern in [
        "[]",
        "[^]",
        "[^^]",
        "[abc]",
        "[^a-z]",
        "[a-zA-Z0-9]",
        "[&!#%,.:;<=>?@`~]",
        r"[\d\D\s\S\w\W]",
        r"[\b-\t]",
        r"[\cA-\cZ]",
        r"[\0-\x01]",
        r"[\-]",
        r"[\--a]",
        r"[\&\!\#\%\,\:\;\<\=\>\@\`\~]",
        r"[\(\)\[\]\{\}\/\-\|\\]",
        r"[\$\$\*\*\+\+\.\.\?\?\^\^]",
        r"[\&&]",
        r"[\$$]",
        "[😀-😁]",
        r"[\uD800\uDC00-\u{10001}]",
    ] {
        validates(pattern);
    }
    for pattern in ["[a-]", "[-a]", "[(){}|]"] {
        assert_eq!(
            parse_script(&format!("/{pattern}/v")).unwrap_err().kind,
            DiagnosticKind::Syntax
        );
    }
    // Escaped '&' is specifically permitted in v classes, unlike u classes.
    validates(r"[\&]");
    assert_eq!(
        parse_script(r"/[\&]/u").unwrap_err().kind,
        DiagnosticKind::Syntax
    );
}

#[test]
fn class_string_disjunctions_compute_empty_and_multi_character_early_errors() {
    for contents in [
        "",
        "a",
        "ab",
        "a|b",
        "a|",
        "|a",
        "a||b",
        "😀",
        r"\uD800\uDC00",
        r"\b|\-",
        r"\||\}",
    ] {
        validates(&format!(r"[\q{{{contents}}}]"));
    }
    for contents in ["a", "a|b", "😀", r"\uD800\uDC00", r"\b|\-"] {
        validates(&format!(r"[^\q{{{contents}}}]"));
    }
    for contents in ["", "ab", "a|", "|a", "a||b", "😀😀"] {
        let source = format!(r"/[^\q{{{contents}}}]/v");
        let error = parse_script(&source).unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
        assert_eq!(
            error.message,
            "negated regular expression class may contain strings"
        );
    }
    validates(&format!(r"[\q{{{}}}]", "a".repeat(100_000)));
}

#[test]
fn flat_class_grammar_errors_share_exact_diagnostics() {
    for pattern in [
        "[z-a]",
        r"[a-\d]",
        r"[\d-a]",
        r"[\q{a}-b]",
        "[a-]",
        "[|]",
        "[/]",
        r"[\q]",
        r"[\q{a]",
        r"[\q{\d}]",
        r"[\q{()}]",
        r"[\q{a**}]",
    ] {
        let source = format!("/{pattern}/v");
        let expected = parse_script(&source).unwrap_err();
        assert_eq!(expected.kind, DiagnosticKind::Syntax, "{source}");
        assert_eq!(
            parse_script_utf16(&JsString::from(source.as_str())).unwrap_err(),
            expected
        );
        assert_eq!(
            parse_eval_utf16(&JsString::from(source.as_str()), EvalContext::default()).unwrap_err(),
            expected
        );
    }
    for point in "!#$%*+,.:;<=>?@^`~".chars() {
        let source = format!("/[a{point}{point}]/v");
        let error = parse_script(&source).unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
        assert_eq!(
            error.message,
            "reserved double punctuation in regular expression Unicode class"
        );
    }
    for (parameters, body) in [(r"x = /[^\q{}]/v", "return x;"), ("x", "return /[a-]/v;")] {
        assert_eq!(
            parse_dynamic_function(parameters, body).unwrap_err().kind,
            DiagnosticKind::Syntax
        );
    }
    let errors: Vec<_> = [
        "/[z-a]/v",
        r"/[a-\d]/v",
        "/[a-]/v",
        "/[!!]/v",
        r"/[^\q{}]/v",
        r"/[\q{\d}]/v",
        r"/[\q]/v",
    ]
    .into_iter()
    .map(|source| parse_script(source).unwrap_err())
    .collect();
    insta::assert_debug_snapshot!(errors);
}

#[test]
fn valid_properties_reach_matching_and_unicode_modes_remain_exclusive() {
    for pattern in [r"[\p{Letter}]", r"[\P{Letter}]"] {
        assert_eq!(
            parse_script(&format!("/{pattern}/v")).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{pattern}"
        );
    }
    assert_eq!(
        parse_script("/[a-z]/uv").unwrap_err().message,
        "regular expression flags u and v are mutually exclusive"
    );
}

#[test]
fn nested_classes_and_set_operators_obey_operand_and_range_grammar() {
    for pattern in [
        "[[]]",
        "[[a]b]",
        "[[^a]&&[b]]",
        "[a&&b&&c]",
        "[a--b--c]",
        "[[a-z]&&[a-m]]",
        "[a&&[b--c]]",
        r"[\d--[a-z]]",
        r"[\q{ab}&&a]",
        r"[^\q{ab}&&a]",
        r"[^\q{a}--\q{bc}]",
        r"[\q{ab}--\q{ab}]",
        r"[\q{ab}&&\q{cd}]",
        r"[a&&\&]",
    ] {
        validates(pattern);
    }
    for pattern in [
        "[ab&&c]",
        "[a&&bc]",
        "[a--bc]",
        "[a&&b--c]",
        "[a--b&&c]",
        "[a-b&&c]",
        "[a&&b-c]",
        "[a-b--c]",
        "[[a]-b]",
        "[a-[b]]",
        "[a--]",
        "[--a]",
        "[a&&]",
        "[&&a]",
        "[a&&&b]",
        "[a&&[b]c]",
        "[a-b-c]",
        r"[a-\q{b}]",
        r"[[^\q{ab}]&&a]",
    ] {
        let source = format!("/{pattern}/v");
        let expected = parse_script(&source).unwrap_err();
        assert_eq!(expected.kind, DiagnosticKind::Syntax, "{source}");
        assert_eq!(
            parse_script_utf16(&JsString::from(source.as_str())).unwrap_err(),
            expected
        );
        assert_eq!(
            parse_eval_utf16(&JsString::from(source.as_str()), EvalContext::default()).unwrap_err(),
            expected
        );
    }
    let errors: Vec<_> = [
        "/[a&&]/v",
        "/[a&&&b]/v",
        "/[a&&b--c]/v",
        "/[a-b&&c]/v",
        "/[[a]-b]/v",
        r"/[[^\q{ab}]&&a]/v",
        "/[a&&bc]/v",
    ]
    .into_iter()
    .map(|source| parse_script(source).unwrap_err())
    .collect();
    insta::assert_debug_snapshot!(errors);
}

#[test]
fn string_containment_follows_union_intersection_and_subtraction_static_rules() {
    for (left, left_strings) in [("a", false), ("", true), ("ab", true)] {
        for (right, right_strings) in [("b", false), ("", true), ("cd", true)] {
            for (operator, strings) in [
                ("", left_strings || right_strings),
                ("&&", left_strings && right_strings),
                ("--", left_strings),
            ] {
                let union = format!(r"[\q{{{left}}}{operator}\q{{{right}}}]");
                validates(&union);
                let inverted = format!(r"/[^\q{{{left}}}{operator}\q{{{right}}}]/v");
                let error = parse_script(&inverted).unwrap_err();
                assert_eq!(
                    error.kind,
                    if strings {
                        DiagnosticKind::Syntax
                    } else {
                        DiagnosticKind::Unsupported
                    },
                    "{inverted}"
                );
                assert_eq!(
                    error.message,
                    if strings {
                        "negated regular expression class may contain strings"
                    } else {
                        "regular expression matching is not implemented"
                    }
                );
            }
        }
    }
    // Subtraction retains the left operand's conservative result even if the
    // two string sets could cancel completely during actual matching.
    assert_eq!(
        parse_script(r"/[^\q{ab}--\q{ab}]/v").unwrap_err().kind,
        DiagnosticKind::Syntax
    );
    validates(r"[^[\q{ab}]&&[a]]");
    validates(r"[^[\q{ab}]&&[\q{cd}]&&a]");
    assert_eq!(
        parse_script(r"/[^[[\q{ab}]&&[\q{cd}]]]/v")
            .unwrap_err()
            .kind,
        DiagnosticKind::Syntax
    );
}

#[test]
fn nested_unicode_classes_are_iterative_without_a_default_depth_quota() {
    let pattern = format!("{}a{}", "[".repeat(20_000), "]".repeat(20_000));
    validates(&pattern);
    let mut pattern = "a".to_owned();
    for _ in 0..20_000 {
        pattern.push_str("&&a");
    }
    validates(&format!("[{pattern}]"));
    assert_eq!(
        parse_dynamic_function("x", "return /[a&&]/v;")
            .unwrap_err()
            .kind,
        DiagnosticKind::Syntax
    );
}
