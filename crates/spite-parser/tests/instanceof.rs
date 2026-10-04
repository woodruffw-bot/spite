//! Instanceof uses relational precedence and is permitted in NoIn contexts.

use spite_core::DiagnosticKind;
use spite_parser::parse_script;

#[test]
fn instanceof_precedence_snapshot() {
    insta::assert_debug_snapshot!(parse_script("a instanceof F === true; a instanceof F instanceof G; new F instanceof G; for (x = a instanceof F; false;); ").unwrap());
}

#[test]
fn operands_obey_expression_and_strict_binding_rules() {
    for source in [
        "a instanceof F",
        "1 instanceof 2",
        "({}) instanceof (()=>1)",
        "a instanceof F()",
        "for (a instanceof F;false;);",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    for source in [
        "instanceof F",
        "a instanceof",
        "a instanceof F=1",
        "'use strict';a instanceof yield",
        r"a inst\u0061nceof F",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}
