//! Try-finally grammar and the early errors of its two block scopes.

use spite_core::DiagnosticKind;
use spite_parser::{MAX_DEPTH, parse_script};

#[test]
fn try_finally_syntax_snapshot() {
    let script = parse_script("try { let x = 1; throw x; } finally { let x = 2; x; }").unwrap();
    insta::assert_debug_snapshot!(script);
}

#[test]
fn try_finally_diagnostics_snapshot() {
    let errors: Vec<_> = [
        "try ; finally {}",
        "try {}",
        "try {} finally ;",
        "try {} finally { break; }",
        "try {} finally { let x; let x; }",
        "try {} catch {}",
    ]
    .map(|s| parse_script(s).unwrap_err())
    .into();
    insta::assert_debug_snapshot!(errors);
}

#[test]
fn both_parts_require_blocks_and_unescaped_keywords() {
    for source in [
        "try {} finally {}",
        "try\n{}\nfinally\n{}",
        "if (true) try {} finally {} else ;",
        "a: try {} finally {}",
        "try { try {} finally {} } finally { try {} finally {} }",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    for source in [
        "try ; finally {}",
        "try 1; finally {}",
        "try {}",
        "try {} ; finally {}",
        "try {} finally",
        "try {} finally ;",
        "try {} finally if (true) {}",
        "try {} finally {",
        "try {} finally {} finally {}",
        "try {} finally {} catch {}",
        "catch {}",
        "finally {}",
        r"tr\u0079 {} finally {}",
        r"try {} fin\u0061lly {}",
        r"try {} c\u0061tch {}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn each_block_has_its_own_lexical_and_label_scope() {
    for source in [
        "let x; try { let x; } finally { let x; }",
        "try { a: ; } finally { a: ; }",
        "try { let x; } finally {} let x;",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    for source in [
        "try { let x; let x; } finally {}",
        "try {} finally { let x; let x; }",
        "try { a: ; } finally { break a; }",
        "try { break a; } finally { a: ; }",
        "a: try { a: ; } finally {}",
        "a: try {} finally { a: ; }",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn control_targets_are_inherited_without_creating_a_new_target() {
    for source in [
        "while (false) try { break; } finally { continue; }",
        "for (;;) try { continue; } finally { break; }",
        "a: while (false) try { continue a; } finally { break a; }",
        "a: try { break a; } finally { break a; }",
        "switch (0) { default: try { break; } finally { break; } }",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    for source in [
        "try { break; } finally {}",
        "try {} finally { break; }",
        "try { continue; } finally {}",
        "try {} finally { continue; }",
        "try { while (false) ; } finally { continue; }",
        "a: try {} finally { while (false) continue a; }",
        "switch (0) { default: try {} finally { continue; } }",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn strict_mode_and_asi_apply_to_both_blocks() {
    for source in [
        "'use strict'; try { let eval; } finally {}",
        "'use strict'; try {} finally { let arguments; }",
        "'use strict'; try { eval = 1; } finally {}",
        "'use strict'; try {} finally { yield; }",
        "try { throw\n1; } finally {}",
        "try {} finally { throw\n1; }",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    assert!(parse_script("try { 1 } finally { 2 }").is_ok());
    // A block cannot introduce a Script directive prologue.
    assert!(parse_script("try { 'use strict'; let eval; } finally { let arguments; }").is_ok());
    assert!(parse_script("try {} finally { 'use strict'; let eval; }").is_ok());
}

#[test]
fn catch_remains_explicitly_unsupported() {
    for source in [
        "try {} catch {}",
        "try {} catch (e) {}",
        "try {} catch (e) {} finally {}",
        "try {} catch {} finally {}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
}

#[test]
fn nested_try_and_finally_blocks_obey_the_depth_limit() {
    for source in [
        format!(
            "{};{}",
            "try {".repeat(MAX_DEPTH),
            "} finally {}".repeat(MAX_DEPTH)
        ),
        format!(
            "{};{}",
            "try {} finally {".repeat(MAX_DEPTH),
            "}".repeat(MAX_DEPTH)
        ),
    ] {
        assert_eq!(
            parse_script(&source).unwrap_err().kind,
            DiagnosticKind::Limit
        );
    }
}
