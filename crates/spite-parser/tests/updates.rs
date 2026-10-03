//! Update target validation, precedence, and restricted line terminators.

use spite_core::DiagnosticKind;
use spite_parser::{MAX_DEPTH, parse_script};

#[test]
fn updates_syntax_snapshot() {
    insta::assert_debug_snapshot!(parse_script("++x ** y--; x\n++y; --(x);").unwrap());
}

#[test]
fn updates_accept_identifier_references_and_parentheses() {
    for source in [
        "++x",
        "--x",
        "x++",
        "x--",
        "++(x)",
        "((x))--",
        "++\nx",
        "x\n++y",
        "x/*\n*/--y",
        "x\u{2028}++y",
        "x\u{2029}--y",
        "x++\n++y",
        "x+++y",
        "x---y",
        "typeof x++",
        "void --x",
        "++x ** 2",
        "x++ ** --y",
        "for (let i = 0; i < 3; i++) ;",
        "for (;i++;) ;",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}

#[test]
fn updates_reject_invalid_and_strict_restricted_targets() {
    for source in [
        "1++",
        "++1",
        "--true",
        "(x + y)++",
        "++(x = 1)",
        "(x, y)--",
        "++x++",
        "x++++",
        "++ ++x",
        "(++x)++",
        "++!x",
        "x++ = 1",
        "x\n++",
        "for (;;x\n++) ;",
        "-x++ ** 2",
        "'use strict'; eval++",
        "'use strict'; ++(arguments)",
        "'use strict'; yield--",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn update_nesting_is_bounded() {
    let source = format!("{}x", "++".repeat(MAX_DEPTH * 2));
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}
