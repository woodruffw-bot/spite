//! Named Pattern grammar and alternative-sensitive static semantics.

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{
    EvalContext, parse_dynamic_function, parse_eval_utf16, parse_script, parse_script_utf16,
};

fn validates(pattern: &str, flags: &str) {
    let source = format!("/{pattern}/{flags}");
    let error = parse_script(&source).unwrap_err();
    assert_eq!(error.kind, DiagnosticKind::Unsupported, "{source}");
    assert_eq!(
        error.message, "regular expression matching is not implemented",
        "{source}"
    );
}

#[test]
fn named_captures_decode_identifiers_and_resolve_forward_and_numbered_references() {
    for flags in ["", "u", "v", "dgimsy"] {
        for pattern in [
            r"(?<a>x)\k<a>",
            r"\k<a>(?<a>x)",
            r"\2(?<a>x)(y)",
            r"(?<\u0061>x)\k<a>",
            r"(?<a>x)\k<\u{61}>",
            r"(?<\u{10400}>x)\k<𐐀>",
            r"(?<𐐀>x)\k<\uD801\uDC00>",
            "(?<$>x)(?<_>y)(?<a𐒤>z)",
            "(?<a\u{200c}\u{200d}>x)",
            "(?<é>x)(?<e\u{301}>y)",
            "(?=a)(?<a>x)(?<=b)",
        ] {
            validates(pattern, flags);
        }
    }
}

#[test]
fn duplicate_names_require_a_separating_disjunction() {
    for flags in ["", "u", "v"] {
        for pattern in [
            r"(?<a>x)|(?<a>y)",
            r"(?:(?<a>x)|(?<a>y))\k<a>",
            "(?<a>x)|(?:(?<a>y)|(?<a>z))",
            "(?:(?<a>x)|y)|(?<a>z)",
            "(?<a>x)|(?<b>y)(?<a>z)",
            "(?:(?<a>x)|(?<a>y))*",
            r"(?<a>x)|(?<\u0061>y)",
            "(?!(?<a>x)|(?<a>y))",
            "(?:(?<a>x)|(?<b>y))(?<c>z)",
        ] {
            validates(pattern, flags);
        }
        for pattern in [
            "(?<a>x)(?<a>y)",
            "(?<a>(?<a>x)|y)",
            "(?<a>x|(?<a>y))",
            "(?:(?<a>x)|y)(?:z|(?<a>w))",
            "(?<a>x)(?:(?<a>y)|z)",
            "(?:(?<a>x)|y)(?<a>z)",
            "(?!(?<a>x))(?<a>y)",
            r"(?<a>x)(?<\u{61}>y)",
            "(?<a>x)|(?<a>y)(?<a>z)",
        ] {
            let source = format!("/{pattern}/{flags}");
            let error = parse_script(&source).unwrap_err();
            assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
            assert_eq!(
                error.message,
                "regular expression capture names might both participate"
            );
        }
    }
}

#[test]
fn malformed_names_and_missing_references_share_grammar_entry_diagnostics() {
    for flags in ["", "u", "v"] {
        for pattern in [
            "(?<>x)",
            "(?<42>x)",
            "(?<❤>x)",
            "(?<a:>x)",
            "(?<a)",
            "(?<𐒤>x)",
            r"(?<\uD800>x)",
            r"(?<a\uDC00>x)",
            r"(?<\u{110000}>x)",
            r"(?<\>x)",
            r"\k",
            r"\k<>",
            r"\k<a",
            r"\k<a>",
            r"(?<a>x)\k<b>",
            r"\3(?<a>x)(y)",
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
    for (parameters, body) in [
        ("x = /(?<>x)/", "return x;"),
        ("x", r"return /\k<missing>/;"),
    ] {
        assert_eq!(
            parse_dynamic_function(parameters, body).unwrap_err().kind,
            DiagnosticKind::Syntax
        );
    }
    let errors: Vec<_> = [
        "/(?<>x)/",
        "/(?<42>x)/u",
        r"/(?<\>x)/",
        "/(?<a)/",
        r"/\k/u",
        r"/\k<a>/",
        "/(?<a>x)(?<a>y)/v",
        r"/(?<a>x)\k<b>/",
        "class C { get [/(?<a>x)(?<a>y)/]() {} }",
    ]
    .into_iter()
    .map(|source| parse_script(source).unwrap_err())
    .collect();
    insta::assert_debug_snapshot!(errors);
}

#[test]
fn raw_lone_surrogate_names_and_valid_properties_keep_distinct_outcomes() {
    for flags in ["", "u", "v"] {
        for unit in [0xd800, 0xdc00] {
            let mut source: Vec<_> = "/(?<a".encode_utf16().collect();
            source.push(unit);
            source.extend(">x)/".encode_utf16());
            source.extend(flags.encode_utf16());
            let error = parse_script_utf16(&JsString::from_code_units(source)).unwrap_err();
            assert_eq!(error.kind, DiagnosticKind::Syntax);
            assert_eq!(
                error.message,
                "invalid regular expression group name identifier"
            );
        }
    }
    for source in [r"/(?<a>\p{Letter})/u", r"/(?<a>[\p{Letter}])/v"] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported
        );
    }
    assert_eq!(
        parse_script("/(?<a>x)/qq").unwrap_err().message,
        "invalid regular expression flag"
    );
}

#[test]
fn deep_names_and_wide_alternatives_do_not_need_default_limits() {
    let mut pattern = String::new();
    for index in 0..20_000 {
        pattern.push_str(&format!("(?<a{index}>"));
    }
    pattern.push('x');
    pattern.push_str(&")".repeat(20_000));
    validates(&pattern, "u");
    validates(&"(?<a>x)|".repeat(20_000), "");
}
