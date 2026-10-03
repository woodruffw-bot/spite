//! Catch clauses without a binding parameter, including catch-finally.

use spite_core::DiagnosticKind;
use spite_parser::{MAX_DEPTH, parse_script};

#[test]
fn catch_syntax_snapshot() {
    insta::assert_debug_snapshot!(
        parse_script("try { throw 1; } catch { 2; } finally { 3; }").unwrap()
    );
}

#[test]
fn catch_blocks_preserve_statement_context_and_separate_scopes() {
    for source in [
        "try {} catch {}",
        "try\n{}\ncatch\n{}\nfinally\n{}",
        "if (true) try {} catch {} else ;",
        "let x; try { let x; } catch { let x; } finally { let x; }",
        "try { a: ; } catch { a: ; } finally { a: ; }",
        "a: while (false) try { continue a; } catch { break a; } finally { continue; }",
        "switch (0) { default: try {} catch { break; } }",
        "try { try {} catch {} } catch { try {} catch {} }",
        "try {} catch { 'use strict'; let eval; }",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}

#[test]
fn malformed_catch_and_block_early_errors_are_rejected() {
    for source in [
        "try {} catch",
        "try {} catch ;",
        "try {} catch 1;",
        "try {} catch {",
        "try {} catch {} catch {}",
        "try {} catch {} finally ;",
        "try {} catch { break; }",
        "try {} catch { continue; }",
        "try { a: ; } catch { break a; }",
        "try {} catch { a: ; } finally { break a; }",
        "a: try {} catch { a: ; }",
        "try {} catch { let x; let x; }",
        "'use strict'; try {} catch { let eval; }",
        "'use strict'; try {} catch { eval = 1; }",
        "'use strict'; try {} catch { yield; }",
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
fn nested_catch_blocks_obey_the_depth_limit() {
    let source = format!(
        "{};{}",
        "try {} catch {".repeat(MAX_DEPTH),
        "}".repeat(MAX_DEPTH)
    );
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}
