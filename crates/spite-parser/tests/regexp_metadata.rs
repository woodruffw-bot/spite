//! ParsePattern retains exact named capture slots and UTF-16 source ranges.

use spite_core::{DiagnosticKind, JsString, Span};
use spite_parser::{parse_regexp_pattern, validate_regexp_pattern};
use std::fmt::Write;

#[test]
fn named_capture_metadata_snapshot() {
    let mut rows = String::new();
    for text in [
        "",
        "()",
        "(?<x>a)",
        "(?<x>a)(b)(?<y>c)",
        "((?<x>a))",
        "(?:(?<x>a))",
        "(?<x>a)|(?<x>b)",
        "(?:(?<x>a)|(?<x>b))",
        "(?<x>a)|((?<y>b))",
        "(?<x>a)(?<x>b)",
        "(?<x>(?<x>a))",
        r"(?<\u0078>a)|(?<x>b)",
        r"(?<\u{78}>a)",
        "(?<µ>a)(?<Μ>b)",
        "(?<a\u{200c}>b)",
        "(?<𐐀>a)",
        r"(?<\uD801\uDC00>a)",
        r"(?<\u{10400}>a)",
        "💩(?<x>a)",
        "(?<𐐀>💩)(?<x>a)",
        r"\((?<x>a)\)",
        r"[(?<x>)](?<y>a)",
        r"(?<x>a)\1",
        r"(?<x>a)\k<x>",
        r"\k<x>(?<x>a)",
        r"(?<x>a)\k<y>",
        "(?<x>a)+",
        "(?<x>a)|(?<x>b)(?<x>c)",
        "(?<x>a)(?:(?<y>b)|(?<y>c))",
        "(?<__proto__>a)(?<constructor>b)",
        "(?=a)(?<x>a)",
        "(?<=a)(?<x>b)",
        "(?i:(?<x>a))",
        "(?<1x>a)",
        r"(?<\uD800>a)",
        "(?<x>a",
    ] {
        for flags in ["", "u", "v"] {
            let body = JsString::from(text);
            let flags = JsString::from(flags);
            let span = Span::new(7, 11);
            let parsed = parse_regexp_pattern(&body, &flags, span);
            assert_eq!(
                parsed.as_ref().map(|metadata| metadata.capture_count),
                validate_regexp_pattern(&body, &flags, span)
                    .as_ref()
                    .copied()
            );
            write!(rows, "{body:?} flags={flags:?}").unwrap();
            match parsed {
                Ok(metadata) => writeln!(rows, " {metadata:?}").unwrap(),
                Err(error) => writeln!(
                    rows,
                    " {:?} {:?}: {}",
                    error.kind, error.span, error.message
                )
                .unwrap(),
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn decoded_names_slots_and_specifier_units_agree_with_explicit_expectations() {
    for flags in ["", "u", "v"] {
        let body = JsString::from(r"💩()(?<𐐀>a)|(?<\u0078>b)(?<x>c)");
        // The repeated x in the same alternative is an early error.
        assert_eq!(
            parse_regexp_pattern(&body, &JsString::from(flags), Span::default())
                .unwrap_err()
                .kind,
            DiagnosticKind::Syntax
        );
        let body = JsString::from(r"💩()(?<𐐀>a)|(?<\u0078>b)");
        let metadata =
            parse_regexp_pattern(&body, &JsString::from(flags), Span::default()).unwrap();
        assert_eq!(metadata.capture_count, 3);
        let expected = [("𐐀", 2, "?<𐐀>"), ("x", 3, r"?<\u0078>")];
        assert_eq!(metadata.named_captures.len(), expected.len());
        for (capture, (name, index, specifier)) in metadata.named_captures.iter().zip(expected) {
            assert_eq!(capture.name, JsString::from(name));
            assert_eq!(capture.index, index);
            assert_eq!(
                &body.code_units()[capture.specifier.clone()],
                JsString::from(specifier).code_units()
            );
        }
        assert_eq!(metadata.named_captures[0].specifier, 5..10);
        assert_eq!(metadata.named_captures[1].specifier, 14..23);
        let duplicate = parse_regexp_pattern(
            &JsString::from(r"(?<\u0078>a)|(?<x>b)"),
            &JsString::from(flags),
            Span::default(),
        )
        .unwrap();
        assert_eq!(
            duplicate.named_captures[0].name,
            duplicate.named_captures[1].name
        );
        assert_eq!(duplicate.named_captures[0].index, 1);
        assert_eq!(duplicate.named_captures[1].index, 2);
    }
    for text in ["(?<x>a)(?<x>b)", r"(?<x>a)\k<y>", "(?<x>a"] {
        let body = JsString::from(text);
        let flags = JsString::from("gg");
        let error = parse_regexp_pattern(&body, &flags, Span::new(31, 47)).unwrap_err();
        assert_eq!(error.message, "duplicate regular expression flag");
        assert_eq!(error.span, Span::new(31, 47));
    }
    // Raw lone surrogates before and after a group keep their original offsets.
    let mut units = vec![0xd800];
    units.extend(JsString::from("(?<x>a)").code_units());
    units.push(0xdc00);
    for flags in ["", "u", "v"] {
        let metadata = parse_regexp_pattern(
            &JsString::from_code_units(units.clone()),
            &JsString::from(flags),
            Span::default(),
        )
        .unwrap();
        assert_eq!(metadata.named_captures[0].specifier, 2..6);
    }
}

#[test]
fn deeply_nested_metadata_uses_source_order_without_default_quotas() {
    let text = format!(
        "{}(?<x>a){}|(?<x>b)",
        "(".repeat(100_000),
        ")".repeat(100_000)
    );
    let metadata = parse_regexp_pattern(
        &JsString::from(text.as_str()),
        &JsString::default(),
        Span::default(),
    )
    .unwrap();
    assert_eq!(metadata.capture_count, 100_002);
    assert_eq!(metadata.named_captures.len(), 2);
    assert_eq!(metadata.named_captures[0].index, 100_001);
    assert_eq!(metadata.named_captures[0].specifier, 100_001..100_005);
    assert_eq!(metadata.named_captures[1].index, 100_002);
    let plain = JsString::from("()".repeat(100_000).as_str());
    let metadata = parse_regexp_pattern(&plain, &JsString::default(), Span::default()).unwrap();
    assert_eq!(metadata.capture_count, 100_000);
    assert!(metadata.named_captures.is_empty());
}
