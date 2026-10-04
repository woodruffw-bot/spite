//! Constructor precedence, optional arguments, and early-error traversal.

use spite_core::DiagnosticKind;
use spite_parser::{MAX_DEPTH, parse_script};

#[test]
fn construction_precedence_snapshot() {
    insta::assert_debug_snapshot!(parse_script("new F; new F(); new F.x(a).y(); new F(a)(b); new new F(a)(b); new (f())(x); new new F().x;").unwrap());
}

#[test]
fn construction_accepts_member_callees_and_assignment_arguments() {
    for source in [
        "new F",
        "new F()",
        "new F(a, b,)",
        "new F((a,b))",
        "new F(a=1,b?c:d)",
        "new F[a](b)",
        "new F().x",
        "new F().x=1",
        "new F().x++",
        "-new F()",
        "new F ** 2",
        "new new F",
        "new new F()",
        "new new F()()",
        "new new F().x()",
        "new (f())",
        "new function(){}",
        "new function(a){this.a=a;}(1)",
        "new (()=>1)",
        "new 1",
        "new null",
        "new this",
        "new {a:1}",
        "new `text`",
        "for(new F('x' in o);false;);",
        "for(new F['x' in o];false;);",
        "new\nF\n(1)",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
}

#[test]
fn invalid_targets_and_strict_errors_in_callees_or_arguments_are_rejected() {
    for source in [
        "new",
        "new;",
        "new F(",
        "new F(,)",
        "new F(a,,b)",
        "new F(a b)",
        "new -1",
        "new ++x",
        "new typeof x",
        "new F=1",
        "new F()=1",
        "new F++",
        "++new F()",
        "new .other",
        "'use strict';new yield",
        "'use strict';new F(yield)",
        "'use strict';new F(eval=1)",
        "'use strict';new F[arguments=1]",
        r"n\u0065w F",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    for source in ["new F(...x)", "new F(a,...x)"] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}

#[test]
fn deeply_nested_new_expressions_and_member_chains_are_bounded() {
    for source in [
        format!("{}F", "new ".repeat(MAX_DEPTH * 2)),
        format!(
            "{}F{}",
            "new ".repeat(MAX_DEPTH * 2),
            "()".repeat(MAX_DEPTH * 2)
        ),
        format!("new F(){}", ".x".repeat(MAX_DEPTH * 2)),
    ] {
        assert_eq!(
            parse_script(&source).unwrap_err().kind,
            DiagnosticKind::Limit
        );
    }
}
