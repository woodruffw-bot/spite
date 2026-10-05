//! Explicit RegExp boundaries must not acquire Pattern or execution credit.

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{
    EvalContext, parse_dynamic_function, parse_eval_utf16, parse_script, parse_script_utf16,
};

#[test]
fn literal_contents_do_not_substitute_unrelated_javascript_diagnostics() {
    let sources = [
        "/[}`]/g",
        "let x = /=}/;",
        "(x = /)/) => x",
        "({x: /`}/})",
        "class C { get [/`}/]() {} }",
        "`a${/[}`]/g}b`",
        "/(/",
    ];
    let errors: Vec<_> = sources
        .into_iter()
        .map(|source| parse_script(source).unwrap_err())
        .collect();
    assert!(
        errors
            .iter()
            .all(|error| error.kind == DiagnosticKind::Unsupported)
    );
    insta::assert_debug_snapshot!(errors);
}

#[test]
fn literal_boundaries_are_shared_by_script_eval_and_function_goals() {
    let source = "let x = /[}`]/g;";
    let expected = parse_script(source).unwrap_err();
    assert_eq!(expected.kind, DiagnosticKind::Unsupported);
    assert_eq!(
        parse_script_utf16(&JsString::from(source)).unwrap_err(),
        expected
    );
    assert_eq!(
        parse_eval_utf16(&JsString::from(source), EvalContext::default()).unwrap_err(),
        expected
    );
    for (parameters, body) in [("x = /[}`]/g", "return x;"), ("x", "return /[}`]/g;")] {
        assert_eq!(
            parse_dynamic_function(parameters, body).unwrap_err().kind,
            DiagnosticKind::Unsupported
        );
    }
}

#[test]
fn every_flag_subset_uses_exactly_one_unicode_mode() {
    let flags = b"dgimsuvy";
    for subset in 0u16..256 {
        let flags: String = flags
            .iter()
            .enumerate()
            .filter_map(|(index, flag)| (subset & (1 << index) != 0).then_some(char::from(*flag)))
            .collect();
        let source = format!("/a/{flags}");
        let error = parse_script(&source).unwrap_err();
        assert_eq!(
            error.kind,
            if flags.contains('u') && flags.contains('v') {
                DiagnosticKind::Syntax
            } else {
                DiagnosticKind::Unsupported
            },
            "{source}"
        );
    }
    for flags in ["yvsmigd", "yusmigd"] {
        assert_eq!(
            parse_script(&format!("/a/{flags}")).unwrap_err().kind,
            DiagnosticKind::Unsupported
        );
    }
}

#[test]
fn flag_early_errors_precede_unimplemented_pattern_validation() {
    let sources = [
        "/./G",
        "/./gig",
        "/a/qq",
        "/a/uv",
        "/a/vu",
        "/a/uvq",
        "/a/uvgg",
        "/(/q",
        "/./0",
        "/./$",
        "/./é",
        "/./𐐀",
        "let x = /a/gg;",
        "class C { get [/a/qq]() {} }",
        "`a${/a/uv}b`",
    ];
    let errors: Vec<_> = sources
        .into_iter()
        .map(|source| {
            let error = parse_script(source).unwrap_err();
            assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
            assert_eq!(
                parse_script_utf16(&JsString::from(source)).unwrap_err(),
                error
            );
            assert_eq!(
                parse_eval_utf16(&JsString::from(source), EvalContext::default()).unwrap_err(),
                error
            );
            error
        })
        .collect();
    insta::assert_debug_snapshot!(errors);
    for (parameters, body) in [("x = /./G", "return x;"), ("x", "return /./gig;")] {
        assert_eq!(
            parse_dynamic_function(parameters, body).unwrap_err().kind,
            DiagnosticKind::Syntax
        );
    }
}

#[test]
fn division_continuations_and_template_substitutions_keep_their_grammar() {
    for source in [
        "let x = 12; x /= 3; x\n/2;",
        "let f = (x = 12 / 3) => `${x / 2}tail`;",
        "`a${({x: 8 / 2}).x}b${`c${12 / 4}d`}e`;",
        "class C { get [`a${12 / 4}b`]() { return 6 / 2; } }",
        "// comment, not an empty literal\n/**/ 1;",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}

#[test]
fn computed_accessor_lookahead_preserves_field_asi_and_private_name_uses() {
    for source in [
        "class C { get\n[12 / 4] = 1; set\n[`x${class D { get [12 / 4]() {} }}`]; }",
        "class C { #x; get [()=>this.#x]() {} }",
        "class C { #x; m() { return class { get [()=>this.#x]() {} }; } }",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    for source in [
        "class C { get [this.#missing]() {} }",
        "class C { get\n[this.#missing]; }",
    ] {
        let error = parse_script(source).unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
        assert_eq!(&source[error.span.start..error.span.end], "#missing");
    }
    let mut name = "0".to_owned();
    for _ in 0..18 {
        name = format!("class {{ get [{name}]() {{}} }}");
    }
    assert!(parse_script(&format!("let C = {name};")).is_ok());
}

#[test]
fn malformed_literal_boundary_diagnostics_snapshot() {
    let sources = [
        "/",
        "/a\\",
        "/[a/",
        "/a\n/",
        "/[\\\u{2028}]/",
        "let x = /a\r/;",
    ];
    let errors: Vec<_> = sources
        .into_iter()
        .map(|source| parse_script(source).unwrap_err())
        .collect();
    assert!(
        errors
            .iter()
            .all(|error| error.kind == DiagnosticKind::Syntax)
    );
    insta::assert_debug_snapshot!(errors);
}
