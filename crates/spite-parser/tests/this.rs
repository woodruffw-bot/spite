//! The this primary expression and reserved-token restrictions.

use spite_core::DiagnosticKind;
use spite_parser::parse_script;

#[test]
fn this_is_a_primary_expression_but_not_an_assignment_or_binding_target() {
    for source in [
        "this",
        "this.x",
        "this['x']",
        "this()",
        "()=>this",
        "function f(x=this){return ()=>this;}",
        "typeof this",
        "delete this",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    for source in [
        "this=1",
        "this++",
        "++this",
        "let this=1",
        "this=>1",
        "function f(this){}",
        "\\u0074his",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}
