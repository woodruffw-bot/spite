//! Direct eval's inherited Script early errors (19.2.1.1).
use spite_core::{DiagnosticKind, JsString};
use spite_parser::{EvalContext, parse_eval_utf16};

#[test]
fn direct_eval_inherits_strictness_and_constructor_context_without_allowing_return() {
    let context = EvalContext {
        strict: true,
        in_function: true,
        in_method: false,
        in_derived_constructor: false,
    };
    let script = parse_eval_utf16(
        &JsString::from("let target=new.target;(()=>target);"),
        context,
    )
    .unwrap();
    assert!(script.is_strict());
    insta::assert_debug_snapshot!(script);
    let diagnostics: Vec<_> = [
        "with({}){}",
        "var eval;",
        "010;",
        "'\\1';",
        "delete x;",
        "return 7;",
        "super.x",
        "super()",
    ]
    .into_iter()
    .map(|source| {
        let error = parse_eval_utf16(&JsString::from(source), context).unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
        error
    })
    .collect();
    insta::assert_debug_snapshot!(diagnostics);
}

#[test]
fn method_context_does_not_cross_ordinary_functions() {
    let context = EvalContext {
        in_function: true,
        in_method: true,
        ..Default::default()
    };
    for source in ["super.x", "()=>super.x"] {
        assert!(parse_eval_utf16(&JsString::from(source), context).is_ok());
    }
    for source in [
        "function f(){super.x;}",
        "super()",
        "function f(){return ()=>super.x;}",
    ] {
        assert_eq!(
            parse_eval_utf16(&JsString::from(source), context)
                .unwrap_err()
                .kind,
            DiagnosticKind::Syntax
        );
    }
    assert!(
        parse_eval_utf16(
            &JsString::from("function f(){return new.target;}"),
            EvalContext::default()
        )
        .is_ok()
    );
    assert_eq!(
        parse_eval_utf16(&JsString::from("()=>new.target"), EvalContext::default())
            .unwrap_err()
            .kind,
        DiagnosticKind::Syntax
    );
}
