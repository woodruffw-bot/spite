//! Shared ParsePattern validation for literal and constructor input (22.2.3.2).

use spite_core::{DiagnosticKind, JsString, Span};
use spite_parser::{parse_script_utf16, validate_regexp_pattern};

#[test]
fn standalone_utf16_patterns_share_flag_grammar_and_capture_diagnostics() {
    let mut records = String::new();
    let span = Span::new(37, 42);
    for body in [
        "",
        "abc",
        "/",
        "\n\r\u{2028}\u{2029}",
        "^a$",
        "(a)(?:b)(c)",
        r"\2(a)(b)",
        r"\3(a)(b)",
        r"\0",
        r"\08",
        r"\8",
        "[a-z]",
        "[z-a]",
        r"[\d-a]",
        "[a&&b]",
        r"[\q{ab}]",
        "(?i:abc)",
        "(?ii:abc)",
        "(?<a>x)|(?<a>y)",
        "(?<a>x)(?<a>y)",
        r"\k<a>(?<a>x)",
        "a{3,2}",
        "(",
        "]",
    ] {
        for flags in ["", "u", "v"] {
            let body = JsString::from(body);
            let flags = JsString::from(flags);
            let result = validate_regexp_pattern(&body, &flags, span);
            records.push_str(&format!("{body:?} flags={flags:?} => "));
            match result {
                Ok(count) => records.push_str(&format!("captures={count}\n")),
                Err(error) => records.push_str(&format!(
                    "{:?}@{}..{}: {}\n",
                    error.kind, error.span.start, error.span.end, error.message
                )),
            }
        }
    }
    for units in [vec![0xd800], vec![0xdc00], vec![0xd800, 0xdc00]] {
        let body = JsString::from_code_units(units);
        for flags in ["", "u", "v"] {
            let flags = JsString::from(flags);
            assert_eq!(validate_regexp_pattern(&body, &flags, span), Ok(0));
            records.push_str(&format!("{body:?} flags={flags:?} => captures=0\n"));
        }
    }
    for flags in ["z", "gg", "uu", "uv", "vu", "uvz", "vvu", "gG"] {
        let flags = JsString::from(flags);
        let error = validate_regexp_pattern(&JsString::from("("), &flags, span).unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Syntax);
        assert!(error.message.contains("flag"));
        records.push_str(&format!(
            "\"(\" flags={flags:?} => {:?}@{}..{}: {}\n",
            error.kind, error.span.start, error.span.end, error.message
        ));
    }
    insta::assert_snapshot!("standalone_utf16_patterns", records);
}

#[test]
fn constructor_patterns_skip_literal_token_boundaries_and_report_exact_capture_counts() {
    let span = Span::new(0, 1);
    let body = JsString::from("/\n\r\u{2028}\u{2029}");
    for flags in ["", "u", "v"] {
        assert_eq!(
            validate_regexp_pattern(&body, &JsString::from(flags), span),
            Ok(0)
        );
    }
    assert_eq!(
        parse_script_utf16(&JsString::from("/\n/"))
            .unwrap_err()
            .kind,
        DiagnosticKind::Syntax
    );
    assert_eq!(
        validate_regexp_pattern(
            &JsString::from("(?<a>x)|(?<a>y)(z)"),
            &JsString::from("u"),
            span
        ),
        Ok(3)
    );
    assert!(parse_script_utf16(&JsString::from("/(a)(b)/u")).is_ok());
}

#[test]
fn group_depth_and_alternative_count_use_iterative_storage_without_default_quotas() {
    let body = JsString::from(format!("{}a{}", "(".repeat(100_000), ")".repeat(100_000)).as_str());
    assert_eq!(
        validate_regexp_pattern(&body, &JsString::from("u"), Span::new(0, 1)),
        Ok(100_000)
    );
    let body = JsString::from(format!("{}a", "a|".repeat(100_000)).as_str());
    assert_eq!(
        validate_regexp_pattern(&body, &JsString::from("v"), Span::new(0, 1)),
        Ok(0)
    );
}
