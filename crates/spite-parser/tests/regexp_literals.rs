//! Validated RegExp literal ASTs retain original UTF-16 text and source ranges.

use spite_core::{DiagnosticKind, JsString, Span};
use spite_parser::{
    EvalContext,
    ast::{ExprKind, Literal, StatementKind},
    parse_eval_utf16, parse_script, parse_script_utf16,
};
use std::fmt::Write;

fn literal(source: &JsString) -> (JsString, JsString, Span) {
    let script = parse_script_utf16(source).unwrap();
    assert_eq!(
        parse_eval_utf16(source, EvalContext::default()).unwrap(),
        script
    );
    let StatementKind::Expression(expr) = &script.statements()[0].kind else {
        panic!("expected a literal expression");
    };
    let ExprKind::Literal(Literal::RegExp { body, flags }) = &expr.kind else {
        panic!("expected a RegExp literal");
    };
    (body.clone(), flags.clone(), expr.span)
}

#[test]
fn original_patterns_flags_and_byte_spans_snapshot() {
    let cases = [
        ("/abc/ig", "abc", "ig"),
        (r"/a\/b/g", r"a\/b", "g"),
        (r"/=\}/", r"=\}", ""),
        ("/[}/]/", "[}/]", ""),
        ("/(?:)/", "(?:)", ""),
        ("/(?<x>a)|(?<x>b)/du", "(?<x>a)|(?<x>b)", "du"),
        (r"/\p{ASCII}/u", r"\p{ASCII}", "u"),
        (r"/[\q{ab|c}]/v", r"[\q{ab|c}]", "v"),
        (r"/\uD800/d", r"\uD800", "d"),
        ("/💩/", "💩", ""),
        ("\t /abc/y", "abc", "y"),
    ];
    let mut rows = String::new();
    for (source, expected_body, expected_flags) in cases {
        let value = JsString::from(source);
        assert_eq!(
            parse_script(source).unwrap(),
            parse_script_utf16(&value).unwrap()
        );
        let (body, flags, span) = literal(&value);
        assert_eq!(body, JsString::from(expected_body));
        assert_eq!(flags, JsString::from(expected_flags));
        writeln!(
            rows,
            "{value:?} -> body={body:?} flags={flags:?} bytes={}..{}",
            span.start, span.end
        )
        .unwrap();
    }
    for units in [
        vec![0xd800],
        vec![0xdc00],
        vec![0xd800, 0xdc00],
        vec![0xdc00, 0xd800],
    ] {
        let mut source = vec![u16::from(b'/')];
        source.extend(&units);
        source.extend("/d".encode_utf16());
        let source = JsString::from_code_units(source);
        let (body, flags, span) = literal(&source);
        assert_eq!(body.code_units(), units);
        assert_eq!(flags, JsString::from("d"));
        writeln!(
            rows,
            "{source:?} -> body={body:?} flags={flags:?} bytes={}..{}",
            span.start, span.end
        )
        .unwrap();
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn goal_sensitive_literals_parse_in_arrow_assignment_class_and_template_contexts() {
    for source in [
        "let f=(x=/[)]/)=>x;",
        "let x; ({x=/[}]/}={});",
        "let x; [x=/[\\]]/]=[];",
        "for(let x of [/a/])x;",
        "class C{get [/`\\}/](){return /[)]/;}}",
        "`a${/[}`]/}b${12/4}c`",
        "let a=/a/; a / 2;",
        "let x = /=x/;",
    ] {
        let script = parse_script(source).unwrap_or_else(|error| panic!("{source}: {error}"));
        assert_eq!(parse_script_utf16(&JsString::from(source)).unwrap(), script);
    }
}

#[test]
fn invalid_literals_keep_syntax_diagnostics_before_ast_construction() {
    for source in [
        "/(/",
        "/a/gg",
        "/(/z",
        "/a/uv",
        "/a\n/",
        "/[a/",
        "(x=/a/gg)=>x",
    ] {
        let error = parse_script(source).unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
        assert_eq!(
            parse_script_utf16(&JsString::from(source)).unwrap_err(),
            error
        );
    }
    assert_eq!(
        parse_script("/(/z").unwrap_err().message,
        "invalid regular expression flag"
    );
}
