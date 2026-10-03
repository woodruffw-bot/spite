//! Catch binding identifiers and the non-browser early-error rules of 14.15.1.

use spite_core::DiagnosticKind;
use spite_parser::parse_script;

#[test]
fn catch_parameter_syntax_snapshot() {
    insta::assert_debug_snapshot!(
        parse_script(r"try { throw 7; } catch (\u0065) { e += 1; e; } finally { 0; }").unwrap()
    );
}

#[test]
fn catch_parameters_have_a_separate_binding_scope() {
    for source in [
        "let e; try {} catch (e) {}",
        "var e; try {} catch (e) {}",
        "try { var e; } catch (e) {} finally { var e; }",
        "try {} catch (e) { { let e; } }",
        "try {} catch (e) { try {} catch (e) {} }",
        "try {} catch (e) { var other; }",
        "try {} catch (e) { for (let e = 0; e < 1; e++) {} }",
        "try {} catch (e) { switch (0) { default: let e; } }",
        "loop: while (false) try {} catch (e) { continue loop; }",
    ] {
        for prefix in ["", "'use strict'; "] {
            assert!(
                parse_script(&format!("{prefix}{source}")).is_ok(),
                "{source}"
            );
        }
    }
    for name in ["let", "yield", "await", "eval", "arguments", "π"] {
        assert!(parse_script(&format!("try {{}} catch ({name}) {{}}")).is_ok());
    }
    let script =
        parse_script("try { var before; } catch (error) { var inside; } finally { var after; }")
            .unwrap();
    assert_eq!(
        script
            .var_declarations()
            .iter()
            .map(|b| b.name.as_str())
            .collect::<Vec<_>>(),
        ["before", "inside", "after"]
    );
}

#[test]
fn catch_conflicts_reject_nested_vars_without_the_optional_browser_exception() {
    for body in [
        "let e;",
        "const e = 0;",
        "var e;",
        "{ var e; }",
        "if (false) var e;",
        "for (var e = 0; false;) {}",
        "switch (0) { case 0: var e; }",
        "try {} catch { var e; }",
        "try {} finally { var e; }",
        "label: { var e; }",
    ] {
        for prefix in ["", "'use strict'; "] {
            let source = format!("{prefix}try {{}} catch (e) {{ {body} }}");
            assert_eq!(
                parse_script(&source).unwrap_err().kind,
                DiagnosticKind::Syntax,
                "{source}"
            );
        }
    }
    let errors: Vec<_> = [
        "try {} catch (e) { let e; }",
        "try {} catch (e) { { var e; } }",
        "'use strict'; try {} catch (eval) {}",
    ]
    .map(|s| parse_script(s).unwrap_err())
    .into();
    insta::assert_debug_snapshot!(errors);
}

#[test]
fn invalid_bindings_and_strict_reserved_names_are_early_errors() {
    for parameter in [
        "",
        "e, f",
        "e = 1",
        "...e",
        "e.x",
        "(e)",
        "1",
        "'e'",
        "null",
        "true",
        "var",
        "for",
        r"\u0066or",
    ] {
        let source = format!("try {{}} catch ({parameter}) {{}}");
        assert_eq!(
            parse_script(&source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    for parameter in [
        "eval",
        "arguments",
        "let",
        "yield",
        "implements",
        "private",
        r"\u0065val",
    ] {
        let source = format!("'use strict'; try {{}} catch ({parameter}) {{}}");
        assert_eq!(
            parse_script(&source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    for parameter in ["[e]", "{e}"] {
        assert_eq!(
            parse_script(&format!("try {{}} catch ({parameter}) {{}}"))
                .unwrap_err()
                .kind,
            DiagnosticKind::Unsupported
        );
    }
    for source in [
        "try {} catch (e) ;",
        "try {} catch (e)",
        "try {} catch (e {}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax
        );
    }
}
