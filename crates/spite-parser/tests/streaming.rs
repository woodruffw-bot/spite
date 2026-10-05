//! Scanner diagnostics must survive every lazy parser entry point.

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{
    EvalContext, parse_dynamic_function, parse_dynamic_function_utf16, parse_eval_utf16,
    parse_script, parse_script_utf16,
};

#[test]
fn lexical_failure_cannot_be_accepted_as_end_of_input() {
    let source = "1; /*";
    let expected = parse_script(source).unwrap_err();
    assert_eq!(expected.kind, DiagnosticKind::Syntax);
    assert_eq!(expected.message, "unterminated comment");
    assert_eq!(
        parse_script_utf16(&JsString::from(source)).unwrap_err(),
        expected
    );
    assert_eq!(
        parse_eval_utf16(&JsString::from(source), EvalContext::default()).unwrap_err(),
        expected
    );
    for (parameters, body) in [("x /*", "return x;"), ("x", "return x; /*")] {
        let expected = parse_dynamic_function(parameters, body).unwrap_err();
        assert_eq!(expected.kind, DiagnosticKind::Syntax);
        assert_eq!(expected.message, "unterminated comment");
        assert_eq!(
            parse_dynamic_function_utf16(&JsString::from(parameters), &JsString::from(body))
                .unwrap_err(),
            expected
        );
    }
}

#[test]
fn lexical_diagnostics_survive_cover_and_computed_name_lookahead() {
    let sources = [
        "let x = 1; '\\u{110000}'",
        "(x = '\\u{110000}') => x",
        "({x: '\\u{110000}'})",
        "class C { get ['\\u{110000}']() {} }",
        "`head${({x: 1}).x}tail${'\\u{110000}'}`",
        "let x = 1; /*",
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

#[test]
fn unrequested_lexical_failure_cannot_mask_recognized_unsupported_syntax() {
    for source in [
        "function* g() { /*",
        "async function f() { '\\u{110000}'",
        "class C { *g() { /*",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
}
