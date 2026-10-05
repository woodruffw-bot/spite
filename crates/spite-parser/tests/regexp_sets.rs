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
fn nested_classes_operators_and_property_gaps_cannot_receive_negative_credit() {
    for pattern in [
        "[[]]",
        "[a&&]",
        "[a--]",
        "[a&&b--c]",
        r"[\p{Invalid}]",
        r"[\P{Invalid}]",
    ] {
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
