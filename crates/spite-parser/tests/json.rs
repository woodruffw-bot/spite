//! ECMA-404 grammar and source fidelity for ParseJSON and raw JSON.

use spite_core::JsString;
use spite_parser::json::{JsonKind, parse_json, parse_json_with_work};

#[test]
fn nested_values_duplicate_names_and_utf16_source_ranges() {
    let source = JsString::from(
        r#" {"😀":[true,false,null,-0,1.25e+2,"\uD800"],"__proto__":1,"__proto__":2} "#,
    );
    insta::assert_debug_snapshot!(parse_json(&source).unwrap());
}

#[test]
fn json_rejects_ecmascript_extensions_and_incomplete_tokens() {
    let cases = [
        "",
        " ",
        "undefined",
        "NaN",
        "Infinity",
        "+1",
        "01",
        "-01",
        "0x10",
        ".1",
        "1.",
        "-",
        "1e",
        "1e+",
        "1e-",
        "tru",
        "True",
        "null false",
        "[1,]",
        "[,1]",
        "[1 2]",
        "[",
        "{",
        "{a:1}",
        "{'a':1}",
        "{\"a\" 1}",
        "{\"a\":}",
        "{\"a\":1,}",
        "//x\n1",
        "/*x*/1",
        "\u{feff}0",
        "\u{a0}0",
        "\u{2028}0",
        "0\u{b}",
        "\"unterminated",
        "\"line\nfeed\"",
        "\"\\v\"",
        "\"\\x41\"",
        "\"\\u{41}\"",
        "\"\\u123\"",
    ];
    let diagnostics: Vec<_> = cases
        .into_iter()
        .map(|source| (source, parse_json(&JsString::from(source)).unwrap_err()))
        .collect();
    assert!(diagnostics.iter().all(|(_, error)| !error.limit));
    insta::assert_debug_snapshot!(diagnostics);
}

#[test]
fn string_escapes_and_raw_utf16_preserve_every_noncontrol_unit() {
    let source = JsString::from(r#""\"\\\/\b\f\n\r\t\u0000\uD800\udc00""#);
    let document = parse_json(&source).unwrap();
    assert_eq!(
        document.nodes[document.root].kind,
        JsonKind::String(JsString::from_code_units(vec![
            0x22, 0x5c, 0x2f, 0x08, 0x0c, 0x0a, 0x0d, 0x09, 0, 0xd800, 0xdc00
        ]))
    );
    for unit in 0x20..=u16::MAX {
        if matches!(unit, 0x22 | 0x5c) {
            continue;
        }
        let source = JsString::from_code_units(vec![0x22, unit, 0x22]);
        let document = parse_json(&source).unwrap();
        assert_eq!(
            document.nodes[document.root].kind,
            JsonKind::String(JsString::from_code_units(vec![unit]))
        );
    }
    for unit in 0..=0x1f {
        assert!(
            !parse_json(&JsString::from_code_units(vec![0x22, unit, 0x22]))
                .unwrap_err()
                .limit
        );
    }
    assert!(parse_json(&JsString::from("\t\r\n [ ] \t\r\n")).is_ok());
}

#[test]
fn decimal_rounding_overflow_underflow_and_negative_zero_match_binary64() {
    for (source, expected) in [
        ("-0", -0.0),
        ("-0.0e0", -0.0),
        ("9007199254740993", 9007199254740992.0),
        ("1e400", f64::INFINITY),
        ("-1e400", f64::NEG_INFINITY),
        ("-1e-400", -0.0),
        ("5e-324", f64::from_bits(1)),
        ("2.4703282292062327e-324", 0.0),
        ("2.4703282292062328e-324", f64::from_bits(1)),
        ("1.7976931348623157e308", f64::MAX),
    ] {
        let document = parse_json(&JsString::from(source)).unwrap();
        let JsonKind::Number(actual) = document.nodes[document.root].kind else {
            panic!("number");
        };
        assert_eq!(actual.to_bits(), expected.to_bits(), "{source}");
    }
}

#[test]
fn deeply_nested_input_and_tree_drop_are_iterative_without_default_quotas() {
    let depth = 20000;
    for (open, close) in [("[", "]"), ("{\"x\":", "}")] {
        let source =
            JsString::from(format!("{}null{}", open.repeat(depth), close.repeat(depth)).as_str());
        let document = parse_json(&source).unwrap();
        assert_eq!(document.nodes.len(), depth + 1);
        assert_eq!(document.root, depth);
        for (index, node) in document.nodes.iter().enumerate() {
            match &node.kind {
                JsonKind::Array(children) => assert_eq!(children, &[index - 1]),
                JsonKind::Object(children) => assert_eq!(children[0].1, index - 1),
                JsonKind::Null => assert_eq!(index, 0),
                _ => panic!("nested containers"),
            }
        }
        drop(document);
    }
}

#[test]
fn caller_opted_in_work_abort_is_distinct_from_invalid_json() {
    let mut remaining = 10;
    let error = parse_json_with_work(&JsString::from("[true,false,null]"), |work| {
        if work > remaining {
            false
        } else {
            remaining -= work;
            true
        }
    })
    .unwrap_err();
    assert!(error.limit);
    assert!(error.offset < 17);
    assert!(
        !parse_json(&JsString::from("[true,false,null,]"))
            .unwrap_err()
            .limit
    );
}
