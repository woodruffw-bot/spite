//! Named-reference metadata preserves decoded names and complete UTF-16 escapes.

use spite_core::{DiagnosticKind, JsString, Span};
use spite_parser::{parse_regexp_pattern, validate_regexp_pattern};
use std::fmt::Write;

#[test]
fn named_reference_metadata_snapshot() {
    let mut rows = String::new();
    for text in [
        "",
        r"(?<x>a)\k<x>",
        r"\k<x>(?<x>a)",
        r"(?<x>a\k<x>)",
        r"(?<x>a)\k<x>\k<x>",
        r"(?<\u0078>a)\k<x>",
        r"(?<x>a)\k<\u0078>",
        r"(?<\u{78}>a)\k<\u{78}>",
        r"💩(?<𐐀>a)\k<𐐀>",
        r"\k<\u{10400}>(?<𐐀>a)",
        r"(?<µ>a)\k<µ>",
        r"(?<a\u200c>b)\k<a\u200c>",
        r"(?<__proto__>a)\k<__proto__>",
        r"(?<x>a)|(?<x>b)\k<x>",
        r"\k<x>(?:(?<x>a)|(?<x>b))",
        r"(?<x>a)\\k<x>",
        r"[k<>](?<x>a)\k<x>",
        r"(?<x>a)\1",
        r"(?<x>a)\k<y>",
        r"(?<x>a)\k<X>",
        r"(?<x>a)\k<1x>",
        r"(?<x>a)\k<\uD800>",
        r"(?<x>a)\k<x",
        r"(?<x>a)\k",
    ] {
        for flags in ["", "u", "v"] {
            let body = JsString::from(text);
            let flags = JsString::from(flags);
            let span = Span::new(13, 29);
            let result = parse_regexp_pattern(&body, &flags, span);
            assert_eq!(
                result.as_ref().map(|m| m.capture_count),
                validate_regexp_pattern(&body, &flags, span)
                    .as_ref()
                    .copied()
            );
            writeln!(rows, "{body:?} flags={flags:?} {result:?}").unwrap();
        }
    }
    insta::assert_snapshot!(rows, @r#"
        "" flags="" Ok(RegExpPatternMetadata { capture_count: 0, named_captures: [], named_references: [] })
        "" flags="u" Ok(RegExpPatternMetadata { capture_count: 0, named_captures: [], named_references: [] })
        "" flags="v" Ok(RegExpPatternMetadata { capture_count: 0, named_captures: [], named_references: [] })
        "(?<x>a)\u005ck<x>" flags="" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..5 }], named_references: [RegExpNamedReference { name: "x", escape: 7..12 }] })
        "(?<x>a)\u005ck<x>" flags="u" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..5 }], named_references: [RegExpNamedReference { name: "x", escape: 7..12 }] })
        "(?<x>a)\u005ck<x>" flags="v" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..5 }], named_references: [RegExpNamedReference { name: "x", escape: 7..12 }] })
        "\u005ck<x>(?<x>a)" flags="" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 6..10 }], named_references: [RegExpNamedReference { name: "x", escape: 0..5 }] })
        "\u005ck<x>(?<x>a)" flags="u" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 6..10 }], named_references: [RegExpNamedReference { name: "x", escape: 0..5 }] })
        "\u005ck<x>(?<x>a)" flags="v" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 6..10 }], named_references: [RegExpNamedReference { name: "x", escape: 0..5 }] })
        "(?<x>a\u005ck<x>)" flags="" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..5 }], named_references: [RegExpNamedReference { name: "x", escape: 6..11 }] })
        "(?<x>a\u005ck<x>)" flags="u" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..5 }], named_references: [RegExpNamedReference { name: "x", escape: 6..11 }] })
        "(?<x>a\u005ck<x>)" flags="v" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..5 }], named_references: [RegExpNamedReference { name: "x", escape: 6..11 }] })
        "(?<x>a)\u005ck<x>\u005ck<x>" flags="" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..5 }], named_references: [RegExpNamedReference { name: "x", escape: 7..12 }, RegExpNamedReference { name: "x", escape: 12..17 }] })
        "(?<x>a)\u005ck<x>\u005ck<x>" flags="u" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..5 }], named_references: [RegExpNamedReference { name: "x", escape: 7..12 }, RegExpNamedReference { name: "x", escape: 12..17 }] })
        "(?<x>a)\u005ck<x>\u005ck<x>" flags="v" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..5 }], named_references: [RegExpNamedReference { name: "x", escape: 7..12 }, RegExpNamedReference { name: "x", escape: 12..17 }] })
        "(?<\u005cu0078>a)\u005ck<x>" flags="" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..10 }], named_references: [RegExpNamedReference { name: "x", escape: 12..17 }] })
        "(?<\u005cu0078>a)\u005ck<x>" flags="u" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..10 }], named_references: [RegExpNamedReference { name: "x", escape: 12..17 }] })
        "(?<\u005cu0078>a)\u005ck<x>" flags="v" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..10 }], named_references: [RegExpNamedReference { name: "x", escape: 12..17 }] })
        "(?<x>a)\u005ck<\u005cu0078>" flags="" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..5 }], named_references: [RegExpNamedReference { name: "x", escape: 7..17 }] })
        "(?<x>a)\u005ck<\u005cu0078>" flags="u" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..5 }], named_references: [RegExpNamedReference { name: "x", escape: 7..17 }] })
        "(?<x>a)\u005ck<\u005cu0078>" flags="v" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..5 }], named_references: [RegExpNamedReference { name: "x", escape: 7..17 }] })
        "(?<\u005cu{78}>a)\u005ck<\u005cu{78}>" flags="" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..10 }], named_references: [RegExpNamedReference { name: "x", escape: 12..22 }] })
        "(?<\u005cu{78}>a)\u005ck<\u005cu{78}>" flags="u" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..10 }], named_references: [RegExpNamedReference { name: "x", escape: 12..22 }] })
        "(?<\u005cu{78}>a)\u005ck<\u005cu{78}>" flags="v" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..10 }], named_references: [RegExpNamedReference { name: "x", escape: 12..22 }] })
        "\ud83d\udca9(?<\ud801\udc00>a)\u005ck<\ud801\udc00>" flags="" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "\ud801\udc00", index: 1, specifier: 3..8 }], named_references: [RegExpNamedReference { name: "\ud801\udc00", escape: 10..16 }] })
        "\ud83d\udca9(?<\ud801\udc00>a)\u005ck<\ud801\udc00>" flags="u" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "\ud801\udc00", index: 1, specifier: 3..8 }], named_references: [RegExpNamedReference { name: "\ud801\udc00", escape: 10..16 }] })
        "\ud83d\udca9(?<\ud801\udc00>a)\u005ck<\ud801\udc00>" flags="v" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "\ud801\udc00", index: 1, specifier: 3..8 }], named_references: [RegExpNamedReference { name: "\ud801\udc00", escape: 10..16 }] })
        "\u005ck<\u005cu{10400}>(?<\ud801\udc00>a)" flags="" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "\ud801\udc00", index: 1, specifier: 14..19 }], named_references: [RegExpNamedReference { name: "\ud801\udc00", escape: 0..13 }] })
        "\u005ck<\u005cu{10400}>(?<\ud801\udc00>a)" flags="u" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "\ud801\udc00", index: 1, specifier: 14..19 }], named_references: [RegExpNamedReference { name: "\ud801\udc00", escape: 0..13 }] })
        "\u005ck<\u005cu{10400}>(?<\ud801\udc00>a)" flags="v" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "\ud801\udc00", index: 1, specifier: 14..19 }], named_references: [RegExpNamedReference { name: "\ud801\udc00", escape: 0..13 }] })
        "(?<\u00b5>a)\u005ck<\u00b5>" flags="" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "\u00b5", index: 1, specifier: 1..5 }], named_references: [RegExpNamedReference { name: "\u00b5", escape: 7..12 }] })
        "(?<\u00b5>a)\u005ck<\u00b5>" flags="u" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "\u00b5", index: 1, specifier: 1..5 }], named_references: [RegExpNamedReference { name: "\u00b5", escape: 7..12 }] })
        "(?<\u00b5>a)\u005ck<\u00b5>" flags="v" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "\u00b5", index: 1, specifier: 1..5 }], named_references: [RegExpNamedReference { name: "\u00b5", escape: 7..12 }] })
        "(?<a\u005cu200c>b)\u005ck<a\u005cu200c>" flags="" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "a\u200c", index: 1, specifier: 1..11 }], named_references: [RegExpNamedReference { name: "a\u200c", escape: 13..24 }] })
        "(?<a\u005cu200c>b)\u005ck<a\u005cu200c>" flags="u" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "a\u200c", index: 1, specifier: 1..11 }], named_references: [RegExpNamedReference { name: "a\u200c", escape: 13..24 }] })
        "(?<a\u005cu200c>b)\u005ck<a\u005cu200c>" flags="v" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "a\u200c", index: 1, specifier: 1..11 }], named_references: [RegExpNamedReference { name: "a\u200c", escape: 13..24 }] })
        "(?<__proto__>a)\u005ck<__proto__>" flags="" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "__proto__", index: 1, specifier: 1..13 }], named_references: [RegExpNamedReference { name: "__proto__", escape: 15..28 }] })
        "(?<__proto__>a)\u005ck<__proto__>" flags="u" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "__proto__", index: 1, specifier: 1..13 }], named_references: [RegExpNamedReference { name: "__proto__", escape: 15..28 }] })
        "(?<__proto__>a)\u005ck<__proto__>" flags="v" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "__proto__", index: 1, specifier: 1..13 }], named_references: [RegExpNamedReference { name: "__proto__", escape: 15..28 }] })
        "(?<x>a)|(?<x>b)\u005ck<x>" flags="" Ok(RegExpPatternMetadata { capture_count: 2, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..5 }, RegExpNamedCapture { name: "x", index: 2, specifier: 9..13 }], named_references: [RegExpNamedReference { name: "x", escape: 15..20 }] })
        "(?<x>a)|(?<x>b)\u005ck<x>" flags="u" Ok(RegExpPatternMetadata { capture_count: 2, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..5 }, RegExpNamedCapture { name: "x", index: 2, specifier: 9..13 }], named_references: [RegExpNamedReference { name: "x", escape: 15..20 }] })
        "(?<x>a)|(?<x>b)\u005ck<x>" flags="v" Ok(RegExpPatternMetadata { capture_count: 2, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..5 }, RegExpNamedCapture { name: "x", index: 2, specifier: 9..13 }], named_references: [RegExpNamedReference { name: "x", escape: 15..20 }] })
        "\u005ck<x>(?:(?<x>a)|(?<x>b))" flags="" Ok(RegExpPatternMetadata { capture_count: 2, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 9..13 }, RegExpNamedCapture { name: "x", index: 2, specifier: 17..21 }], named_references: [RegExpNamedReference { name: "x", escape: 0..5 }] })
        "\u005ck<x>(?:(?<x>a)|(?<x>b))" flags="u" Ok(RegExpPatternMetadata { capture_count: 2, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 9..13 }, RegExpNamedCapture { name: "x", index: 2, specifier: 17..21 }], named_references: [RegExpNamedReference { name: "x", escape: 0..5 }] })
        "\u005ck<x>(?:(?<x>a)|(?<x>b))" flags="v" Ok(RegExpPatternMetadata { capture_count: 2, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 9..13 }, RegExpNamedCapture { name: "x", index: 2, specifier: 17..21 }], named_references: [RegExpNamedReference { name: "x", escape: 0..5 }] })
        "(?<x>a)\u005c\u005ck<x>" flags="" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..5 }], named_references: [] })
        "(?<x>a)\u005c\u005ck<x>" flags="u" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..5 }], named_references: [] })
        "(?<x>a)\u005c\u005ck<x>" flags="v" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..5 }], named_references: [] })
        "[k<>](?<x>a)\u005ck<x>" flags="" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 6..10 }], named_references: [RegExpNamedReference { name: "x", escape: 12..17 }] })
        "[k<>](?<x>a)\u005ck<x>" flags="u" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 6..10 }], named_references: [RegExpNamedReference { name: "x", escape: 12..17 }] })
        "[k<>](?<x>a)\u005ck<x>" flags="v" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 6..10 }], named_references: [RegExpNamedReference { name: "x", escape: 12..17 }] })
        "(?<x>a)\u005c1" flags="" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..5 }], named_references: [] })
        "(?<x>a)\u005c1" flags="u" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..5 }], named_references: [] })
        "(?<x>a)\u005c1" flags="v" Ok(RegExpPatternMetadata { capture_count: 1, named_captures: [RegExpNamedCapture { name: "x", index: 1, specifier: 1..5 }], named_references: [] })
        "(?<x>a)\u005ck<y>" flags="" Err(Diagnostic { kind: Syntax, span: Span { start: 13, end: 29 }, message: "regular expression named backreference has no capture" })
        "(?<x>a)\u005ck<y>" flags="u" Err(Diagnostic { kind: Syntax, span: Span { start: 13, end: 29 }, message: "regular expression named backreference has no capture" })
        "(?<x>a)\u005ck<y>" flags="v" Err(Diagnostic { kind: Syntax, span: Span { start: 13, end: 29 }, message: "regular expression named backreference has no capture" })
        "(?<x>a)\u005ck<X>" flags="" Err(Diagnostic { kind: Syntax, span: Span { start: 13, end: 29 }, message: "regular expression named backreference has no capture" })
        "(?<x>a)\u005ck<X>" flags="u" Err(Diagnostic { kind: Syntax, span: Span { start: 13, end: 29 }, message: "regular expression named backreference has no capture" })
        "(?<x>a)\u005ck<X>" flags="v" Err(Diagnostic { kind: Syntax, span: Span { start: 13, end: 29 }, message: "regular expression named backreference has no capture" })
        "(?<x>a)\u005ck<1x>" flags="" Err(Diagnostic { kind: Syntax, span: Span { start: 13, end: 29 }, message: "invalid regular expression group name identifier" })
        "(?<x>a)\u005ck<1x>" flags="u" Err(Diagnostic { kind: Syntax, span: Span { start: 13, end: 29 }, message: "invalid regular expression group name identifier" })
        "(?<x>a)\u005ck<1x>" flags="v" Err(Diagnostic { kind: Syntax, span: Span { start: 13, end: 29 }, message: "invalid regular expression group name identifier" })
        "(?<x>a)\u005ck<\u005cuD800>" flags="" Err(Diagnostic { kind: Syntax, span: Span { start: 13, end: 29 }, message: "invalid regular expression group name identifier" })
        "(?<x>a)\u005ck<\u005cuD800>" flags="u" Err(Diagnostic { kind: Syntax, span: Span { start: 13, end: 29 }, message: "invalid regular expression group name identifier" })
        "(?<x>a)\u005ck<\u005cuD800>" flags="v" Err(Diagnostic { kind: Syntax, span: Span { start: 13, end: 29 }, message: "invalid regular expression group name identifier" })
        "(?<x>a)\u005ck<x" flags="" Err(Diagnostic { kind: Syntax, span: Span { start: 13, end: 29 }, message: "unterminated regular expression group name" })
        "(?<x>a)\u005ck<x" flags="u" Err(Diagnostic { kind: Syntax, span: Span { start: 13, end: 29 }, message: "unterminated regular expression group name" })
        "(?<x>a)\u005ck<x" flags="v" Err(Diagnostic { kind: Syntax, span: Span { start: 13, end: 29 }, message: "unterminated regular expression group name" })
        "(?<x>a)\u005ck" flags="" Err(Diagnostic { kind: Syntax, span: Span { start: 13, end: 29 }, message: "regular expression named backreference requires a group name" })
        "(?<x>a)\u005ck" flags="u" Err(Diagnostic { kind: Syntax, span: Span { start: 13, end: 29 }, message: "regular expression named backreference requires a group name" })
        "(?<x>a)\u005ck" flags="v" Err(Diagnostic { kind: Syntax, span: Span { start: 13, end: 29 }, message: "regular expression named backreference requires a group name" })
        "#);
}

#[test]
fn decoded_reference_names_and_original_spelling_ranges_are_independent() {
    let body = JsString::from(r"💩\k<\u{10400}>(?<𐐀>a)(?<\u0078>b)\k<x>\k<\u0078>");
    for flags in ["", "u", "v"] {
        let metadata =
            parse_regexp_pattern(&body, &JsString::from(flags), Span::default()).unwrap();
        let expected = [
            ("𐐀", r"\k<\u{10400}>"),
            ("x", r"\k<x>"),
            ("x", r"\k<\u0078>"),
        ];
        assert_eq!(metadata.named_references.len(), expected.len());
        for (reference, (name, spelling)) in metadata.named_references.iter().zip(expected) {
            assert_eq!(reference.name, JsString::from(name));
            assert_eq!(
                &body.code_units()[reference.escape.clone()],
                JsString::from(spelling).code_units()
            );
        }
        assert_eq!(metadata.named_references[0].escape, 2..15);
        assert_eq!(metadata.named_captures[0].index, 1);
        assert_eq!(metadata.named_captures[1].index, 2);
        assert_eq!(
            metadata.named_references[1].name,
            metadata.named_captures[1].name
        );
        let mut units = vec![0xd800];
        units.extend(JsString::from(r"\k<x>(?<x>a)").code_units());
        units.push(0xdc00);
        let raw = parse_regexp_pattern(
            &JsString::from_code_units(units),
            &JsString::from(flags),
            Span::default(),
        )
        .unwrap();
        assert_eq!(raw.named_references[0].escape, 1..6);
    }
    let escaped = parse_regexp_pattern(
        &JsString::from(r"(?<x>a)\\k<x>"),
        &JsString::default(),
        Span::default(),
    )
    .unwrap();
    assert!(escaped.named_references.is_empty());
    for body in [r"(?<x>a)\k<missing>", r"(?<x>a)\k<x", r"(?<x>a)\k<1>"] {
        let span = Span::new(31, 47);
        let error =
            parse_regexp_pattern(&JsString::from(body), &JsString::from("gg"), span).unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Syntax);
        assert_eq!(error.span, span);
        assert_eq!(error.message, "duplicate regular expression flag");
    }
}

#[test]
fn deep_groups_and_repeated_references_keep_linear_metadata_without_default_quotas() {
    let text = format!(
        "{}\\k<x>(?<x>a){}{}",
        "(".repeat(100000),
        ")".repeat(100000),
        r"\k<\u0078>".repeat(100000)
    );
    let body = JsString::from(text.as_str());
    let metadata = parse_regexp_pattern(&body, &JsString::default(), Span::default()).unwrap();
    assert_eq!(metadata.capture_count, 100001);
    assert_eq!(metadata.named_captures.len(), 1);
    assert_eq!(metadata.named_captures[0].index, 100001);
    assert_eq!(metadata.named_references.len(), 100001);
    assert_eq!(metadata.named_references[0].escape, 100000..100005);
    for reference in &metadata.named_references[1..] {
        assert_eq!(reference.name, JsString::from("x"));
        assert_eq!(
            &body.code_units()[reference.escape.clone()],
            JsString::from(r"\k<\u0078>").code_units()
        );
    }
    assert_eq!(
        metadata.named_references.last().unwrap().escape.end,
        body.len()
    );
    assert_eq!(
        validate_regexp_pattern(&body, &JsString::default(), Span::default()),
        Ok(100001)
    );
}
