//! Three-clause for-loop grammar and static semantics.

use spite_core::DiagnosticKind;
use spite_parser::{MAX_DEPTH, parse_script};

#[test]
fn for_syntax_snapshot() {
    let script = parse_script("for (i = 0; i < 2; i = i + 1) continue; for (;;) break;").unwrap();
    insta::assert_debug_snapshot!(script);
}

#[test]
fn header_expressions_are_optional_and_accept_commas() {
    for source in [
        "for (;;) ;",
        "for (0;;) ;",
        "for (;false;) ;",
        "for (;;0) ;",
        "for (0;false;) ;",
        "for (0;;0) ;",
        "for (;false;0) ;",
        "for (0;false;0) ;",
        "for (a = 0, b = 0; a < 2, b < 3; a = a + 1, b = b + 1) ;",
        "for (let; false; let = 1) ;",
        "a: b: for (;;) continue a;",
        "for (;;) if (true) break; else continue;",
        "for (;false;) let\nx = 1;",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}

#[test]
fn asi_cannot_supply_header_semicolons_or_a_body() {
    for source in [
        "for (0\nfalse;0) ;",
        "for (0;false\n0) ;",
        "for (0\nfalse\n0) ;",
        "for () ;",
        "for (;) ;",
        "for (;;;) ;",
        "for (;;)",
        "for (;;)\n",
        "for (;;) let x;",
        "for (;;) const x = 1;",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn header_validation_and_label_contexts_are_preserved() {
    for source in [
        "'use strict'; for (yield;;) ;",
        "'use strict'; for (;yield;) ;",
        "'use strict'; for (;;arguments = 1) ;",
        "for (;false;) { let a; let a; }",
        "for (;;) break missing;",
        "a: { for (;;) continue a; }",
        "for (;false;) ; continue;",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    let source = format!("{};", "for (;;) ".repeat(MAX_DEPTH * 2));
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}

#[test]
fn unsupported_header_forms_are_not_syntax_error_passes() {
    for source in [
        "for (var x = 0;;) ;",
        "for (let x = 0;;) ;",
        "for (const x = 0;;) ;",
        "for (x in y) ;",
        "for (x of y) ;",
        "for await (x of y) ;",
        "for (i = 0; i < 3; i++) ;",
        "for (i += 1;;) ;",
        "for (;i++;) ;",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
}
