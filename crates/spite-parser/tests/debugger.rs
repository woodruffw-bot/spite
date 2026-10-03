//! Debugger statements use ordinary statement positions and ASI.

use spite_core::DiagnosticKind;
use spite_parser::parse_script;

#[test]
fn debugger_syntax_snapshot() {
    insta::assert_debug_snapshot!(parse_script("debugger; { debugger }").unwrap());
}

#[test]
fn debugger_accepts_asi_and_statement_positions() {
    for source in [
        "debugger",
        "debugger\n1",
        "debugger/*\n*/1",
        "if (true) debugger; else debugger;",
        "a: debugger;",
        "while (false) debugger;",
        "try { debugger; } finally { debugger; }",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}

#[test]
fn debugger_requires_a_semicolon_or_asi_and_an_unescaped_keyword() {
    for source in [
        "debugger 1",
        r"deb\u0075gger;",
        "let debugger;",
        "debugger: ;",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}
