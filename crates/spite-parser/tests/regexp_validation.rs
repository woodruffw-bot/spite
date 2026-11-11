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
    insta::assert_snapshot!(records, @r#"
        "" flags="" => captures=0
        "" flags="u" => captures=0
        "" flags="v" => captures=0
        "abc" flags="" => captures=0
        "abc" flags="u" => captures=0
        "abc" flags="v" => captures=0
        "/" flags="" => captures=0
        "/" flags="u" => captures=0
        "/" flags="v" => captures=0
        "\u000a\u000d\u2028\u2029" flags="" => captures=0
        "\u000a\u000d\u2028\u2029" flags="u" => captures=0
        "\u000a\u000d\u2028\u2029" flags="v" => captures=0
        "^a$" flags="" => captures=0
        "^a$" flags="u" => captures=0
        "^a$" flags="v" => captures=0
        "(a)(?:b)(c)" flags="" => captures=2
        "(a)(?:b)(c)" flags="u" => captures=2
        "(a)(?:b)(c)" flags="v" => captures=2
        "\u005c2(a)(b)" flags="" => captures=2
        "\u005c2(a)(b)" flags="u" => captures=2
        "\u005c2(a)(b)" flags="v" => captures=2
        "\u005c3(a)(b)" flags="" => Syntax@37..42: regular expression backreference exceeds the capture count
        "\u005c3(a)(b)" flags="u" => Syntax@37..42: regular expression backreference exceeds the capture count
        "\u005c3(a)(b)" flags="v" => Syntax@37..42: regular expression backreference exceeds the capture count
        "\u005c0" flags="" => captures=0
        "\u005c0" flags="u" => captures=0
        "\u005c0" flags="v" => captures=0
        "\u005c08" flags="" => Syntax@37..42: legacy regular expression octal escapes are not in the core grammar
        "\u005c08" flags="u" => Syntax@37..42: legacy regular expression octal escapes are not in the core grammar
        "\u005c08" flags="v" => Syntax@37..42: legacy regular expression octal escapes are not in the core grammar
        "\u005c8" flags="" => Syntax@37..42: regular expression backreference exceeds the capture count
        "\u005c8" flags="u" => Syntax@37..42: regular expression backreference exceeds the capture count
        "\u005c8" flags="v" => Syntax@37..42: regular expression backreference exceeds the capture count
        "[a-z]" flags="" => captures=0
        "[a-z]" flags="u" => captures=0
        "[a-z]" flags="v" => captures=0
        "[z-a]" flags="" => Syntax@37..42: regular expression class range is reversed
        "[z-a]" flags="u" => Syntax@37..42: regular expression class range is reversed
        "[z-a]" flags="v" => Syntax@37..42: regular expression Unicode class range is reversed
        "[\u005cd-a]" flags="" => Syntax@37..42: regular expression class range requires character endpoints
        "[\u005cd-a]" flags="u" => Syntax@37..42: regular expression class range requires character endpoints
        "[\u005cd-a]" flags="v" => Syntax@37..42: regular expression Unicode class range requires character endpoints
        "[a&&b]" flags="" => captures=0
        "[a&&b]" flags="u" => captures=0
        "[a&&b]" flags="v" => captures=0
        "[\u005cq{ab}]" flags="" => Syntax@37..42: invalid regular expression identity escape
        "[\u005cq{ab}]" flags="u" => Syntax@37..42: invalid regular expression identity escape
        "[\u005cq{ab}]" flags="v" => captures=0
        "(?i:abc)" flags="" => captures=0
        "(?i:abc)" flags="u" => captures=0
        "(?i:abc)" flags="v" => captures=0
        "(?ii:abc)" flags="" => Syntax@37..42: duplicate regular expression modifier
        "(?ii:abc)" flags="u" => Syntax@37..42: duplicate regular expression modifier
        "(?ii:abc)" flags="v" => Syntax@37..42: duplicate regular expression modifier
        "(?<a>x)|(?<a>y)" flags="" => captures=2
        "(?<a>x)|(?<a>y)" flags="u" => captures=2
        "(?<a>x)|(?<a>y)" flags="v" => captures=2
        "(?<a>x)(?<a>y)" flags="" => Syntax@37..42: regular expression capture names might both participate
        "(?<a>x)(?<a>y)" flags="u" => Syntax@37..42: regular expression capture names might both participate
        "(?<a>x)(?<a>y)" flags="v" => Syntax@37..42: regular expression capture names might both participate
        "\u005ck<a>(?<a>x)" flags="" => captures=1
        "\u005ck<a>(?<a>x)" flags="u" => captures=1
        "\u005ck<a>(?<a>x)" flags="v" => captures=1
        "a{3,2}" flags="" => Syntax@37..42: regular expression quantifier bounds are reversed
        "a{3,2}" flags="u" => Syntax@37..42: regular expression quantifier bounds are reversed
        "a{3,2}" flags="v" => Syntax@37..42: regular expression quantifier bounds are reversed
        "(" flags="" => Syntax@37..42: unterminated regular expression group
        "(" flags="u" => Syntax@37..42: unterminated regular expression group
        "(" flags="v" => Syntax@37..42: unterminated regular expression group
        "]" flags="" => Syntax@37..42: unexpected regular expression syntax character
        "]" flags="u" => Syntax@37..42: unexpected regular expression syntax character
        "]" flags="v" => Syntax@37..42: unexpected regular expression syntax character
        "\ud800" flags="" => captures=0
        "\ud800" flags="u" => captures=0
        "\ud800" flags="v" => captures=0
        "\udc00" flags="" => captures=0
        "\udc00" flags="u" => captures=0
        "\udc00" flags="v" => captures=0
        "\ud800\udc00" flags="" => captures=0
        "\ud800\udc00" flags="u" => captures=0
        "\ud800\udc00" flags="v" => captures=0
        "(" flags="z" => Syntax@37..42: invalid regular expression flag
        "(" flags="gg" => Syntax@37..42: duplicate regular expression flag
        "(" flags="uu" => Syntax@37..42: duplicate regular expression flag
        "(" flags="uv" => Syntax@37..42: regular expression flags u and v are mutually exclusive
        "(" flags="vu" => Syntax@37..42: regular expression flags u and v are mutually exclusive
        "(" flags="uvz" => Syntax@37..42: invalid regular expression flag
        "(" flags="vvu" => Syntax@37..42: duplicate regular expression flag
        "(" flags="gG" => Syntax@37..42: invalid regular expression flag
        "#);
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
