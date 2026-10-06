//! EscapeRegExpPattern preserves the shared Pattern grammar and literal boundary.

use spite_core::{DiagnosticKind, JsString, Span, regexp_pattern_source_units};
use spite_parser::{parse_script_utf16, validate_regexp_pattern};

#[test]
fn serialized_constructor_patterns_validate_with_the_same_capture_count_as_literals() {
    let span = Span::new(0, 1);
    let mut cases: Vec<JsString> = [
        "",
        "(?:)",
        "^a(b|c)+[d-f]$",
        "/",
        "[/]",
        "\n\r\u{2028}\u{2029}",
        "[\n\r\u{2028}\u{2029}]",
        r"(\d)\w\s\1",
        r"\/\\\n\u2028",
    ]
    .into_iter()
    .map(JsString::from)
    .collect();
    for units in [
        vec![0],
        vec![0xd800],
        vec![0xdc00],
        vec![0xd800, 0xdc00],
        vec![0xd800, 0xdc00, 0xdc00],
    ] {
        cases.push(JsString::from_code_units(units));
    }
    for body in &cases {
        for flags in ["", "u", "v"] {
            let flags = JsString::from(flags);
            if body == &JsString::from("[/]") && flags == JsString::from("v") {
                assert_eq!(
                    validate_regexp_pattern(body, &flags, span)
                        .unwrap_err()
                        .kind,
                    DiagnosticKind::Syntax
                );
            } else {
                let captures = validate_regexp_pattern(body, &flags, span).unwrap();
                let source = JsString::from_code_units(regexp_pattern_source_units(body).collect());
                assert_eq!(
                    validate_regexp_pattern(&source, &flags, span),
                    Ok(captures),
                    "{body:?} {flags:?}"
                );
                let literal = JsString::from("/")
                    .concat(&source)
                    .concat(&JsString::from("/"))
                    .concat(&flags);
                assert!(parse_script_utf16(&literal).is_ok(), "{literal:?}");
            }
        }
    }
    for terminator in [0x2f, 0x0a, 0x0d, 0x2028, 0x2029] {
        for count in 0..=6 {
            let mut units = vec![0x5c; count];
            units.push(terminator);
            let body = JsString::from_code_units(units);
            let flags = JsString::default();
            assert_eq!(validate_regexp_pattern(&body, &flags, span), Ok(0));
            let source = JsString::from_code_units(regexp_pattern_source_units(&body).collect());
            assert_eq!(validate_regexp_pattern(&source, &flags, span), Ok(0));
            let literal = JsString::from("/")
                .concat(&source)
                .concat(&JsString::from("/"));
            assert!(parse_script_utf16(&literal).is_ok(), "{literal:?}");
        }
    }
}
