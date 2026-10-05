//! With grammar, strict early errors, and declaration/control-flow scope (14.11).

use spite_core::DiagnosticKind;
use spite_parser::parse_script;

#[test]
fn with_syntax_snapshot() {
    insta::assert_debug_snapshot!(parse_script("with (object) { var x = value; x; }").unwrap());
}

#[test]
fn strict_and_malformed_with_diagnostics_snapshot() {
    let errors: Vec<_> = [
        "'use strict'; with ({}) ;",
        "function f() { 'use strict'; with ({}) {} }",
        "with () ;",
        "with ({}) let x = 1;",
        "with ({}) function f() {}",
        "with ({}) class C {}",
        "with ({}) async function f() {}",
    ]
    .map(|source| parse_script(source).unwrap_err())
    .into();
    insta::assert_debug_snapshot!(errors);
}

#[test]
fn with_is_a_statement_and_preserves_enclosing_control_context() {
    for source in [
        "with ({}) ;",
        "if (true) with ({}) ; else ;",
        "with ('x' in {}) with ({}) {}",
        "with (0, {}) var x;",
        "a: with ({}) break a;",
        "a: while (true) with ({}) continue a;",
        "switch (0) { default: with ({}) break; }",
        "function f() { with ({}) return; }",
        "with ({}) { function f() { 'use strict'; return 1; } }",
        "with ({}) async\nfunction f() {}",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    for source in [
        "with ({}) break;",
        "with ({}) continue;",
        "with ({}) return;",
        "a: with ({}) while (true) continue a;",
        "with ({}) a: function f() {}",
        "with ({}) function* f() {}",
        "with ({}) async function* f() {}",
        r"wi\u0074h ({}) ;",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn strictness_and_var_conflicts_reach_through_with_bodies() {
    for source in [
        "'use strict'; if (false) with ({}) ;",
        "function f(){ 'use strict'; function g(){ with ({}) ; } }",
        "let f = () => { 'use strict'; with ({}) ; };",
        "let x; with ({}) { var x; }",
        "for (let x;;) with ({}) { var x; }",
        "with ({}) { let x; with ({}) var x; }",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    assert!(parse_script("with ({}) { let x; } var x;").is_ok());
    assert!(spite_parser::parse_dynamic_function("", "with ({}) return 7;").is_ok());
    assert_eq!(
        spite_parser::parse_dynamic_function("", "'use strict'; with ({}) return 7;")
            .unwrap_err()
            .kind,
        DiagnosticKind::Syntax,
    );
}
