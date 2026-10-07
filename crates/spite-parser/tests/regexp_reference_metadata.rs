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
    insta::assert_snapshot!(rows);
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
