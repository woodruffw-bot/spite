//! Switch grammar, lexical scope, and control-flow early errors.

use spite_core::DiagnosticKind;
use spite_parser::{MAX_DEPTH, parse_script};

#[test]
fn switch_syntax_snapshot() {
    let script = parse_script("switch (x) { case 0: ; default: 1; case 2: break; }").unwrap();
    insta::assert_debug_snapshot!(script);
}

#[test]
fn switch_diagnostics_snapshot() {
    let errors: Vec<_> = [
        "switch (0) { default: ; default: ; }",
        "switch (0) { case: ; }",
        "switch (0) { case 0: let x; default: const x = 1; }",
        "switch (0) { default: continue; }",
    ]
    .map(|s| parse_script(s).unwrap_err())
    .into();
    insta::assert_debug_snapshot!(errors);
}

#[test]
fn clauses_accept_expressions_empty_bodies_and_any_default_position() {
    for source in [
        "switch (0) {}",
        "switch (0) { default: }",
        "switch (0) { case 0: case 1: }",
        "switch (0) { default: case 0: case 1: }",
        "switch (0) { case 0: default: case 1: }",
        "switch (0) { case 0: case 1: default: }",
        "switch (0) { case 0: case 0: }",
        "switch (0, 1) { case true ? 0 : 1: case 0, 1: }",
        "switch (0) { case 0: switch (1) { default: break; } break; }",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    for source in [
        "switch () {}",
        "switch (0)",
        "switch (0) ;",
        "switch (0) { ; }",
        "switch (0) { case: }",
        "switch (0) { case 0 ; }",
        "switch (0) { default 0: }",
        "switch (0) { default: default: }",
        "switch (0) { case 0:",
        r"switch (0) { c\u0061se 0: }",
        r"switch (0) { def\u0061ult: }",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn clauses_share_one_lexical_scope() {
    for source in [
        "switch (0) { case 0: let x; case 1: let x; }",
        "switch (0) { case 0: let x; default: const x = 1; }",
        "switch (0) { default: let x; case 1: let x; }",
        r"switch (0) { case 0: let x; case 1: let \u0078; }",
    ] {
        let error = parse_script(source).unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Syntax);
        assert_eq!(error.message, "duplicate lexical binding");
    }
    for source in [
        "let x; switch (0) { case 0: let x; }",
        "switch (0) { case 0: { let x; } case 1: { let x; } }",
        "switch (0) { case 0: let x; case 1: { let x; } }",
        "switch (0) { case 0: let x; switch (1) { case 1: let x; } }",
        "switch (0) { case 0: let x; } let x;",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}

#[test]
fn switch_enables_break_without_enabling_continue() {
    for source in [
        "switch (0) { default: break; }",
        "switch (0) { case 0: if (false) { break; } }",
        "switch (0) { default: while (false) continue; }",
        "for (;;) switch (0) { default: continue; }",
        "a: while (false) switch (0) { default: continue a; }",
        "a: switch (0) { default: break a; }",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    for source in [
        "switch (0) { default: continue; }",
        "switch (0) { default: if (false) continue; }",
        "a: switch (0) { default: while (false) continue a; }",
        "while (false) a: switch (0) { default: continue a; }",
        "switch (0) { default: break missing; }",
        "switch (0) {} break;",
        "switch (0) {} continue;",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn strict_mode_and_asi_apply_inside_clauses() {
    for source in [
        "'use strict'; switch (yield) {}",
        "'use strict'; switch (0) { case yield: ; }",
        "'use strict'; switch (0) { default: let eval; }",
        "switch (0) { case 0: 1 case 1: 2 }",
        "switch (0) { case 0: break default: ; }",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    assert!(parse_script("switch (0) { case 0: 1\ncase 1: 2 }").is_ok());
    assert!(parse_script("switch (0) { case 0: break\ndefault: ; }").is_ok());
}

#[test]
fn nested_switches_obey_the_depth_limit() {
    let source = format!(
        "{};{}",
        "switch (0) { default: ".repeat(MAX_DEPTH * 2),
        "}".repeat(MAX_DEPTH * 2)
    );
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}
