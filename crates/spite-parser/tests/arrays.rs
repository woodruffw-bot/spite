//! Array literals, elisions, and AssignmentExpression[+In] parsing (13.2.4).

use spite_core::DiagnosticKind;
use spite_parser::{
    MAX_DEPTH,
    ast::{ExprKind, StatementKind},
    parse_script,
};

#[test]
fn array_literal_snapshot() {
    insta::assert_debug_snapshot!(
        parse_script("[ , 1, , undefined, [2,], (a, b), x => x, ]").unwrap()
    );
}

#[test]
fn elisions_and_trailing_commas_have_exact_element_counts() {
    for (source, holes) in [
        ("[]", vec![]),
        ("[,]", vec![true]),
        ("[,,]", vec![true, true]),
        ("[1]", vec![false]),
        ("[1,]", vec![false]),
        ("[1,,]", vec![false, true]),
        ("[,1,,undefined,]", vec![true, false, true, false]),
    ] {
        let script = parse_script(source).unwrap();
        let StatementKind::Expression(expression) = &script.statements()[0].kind else {
            panic!("expression");
        };
        let ExprKind::Array(elements) = &expression.kind else {
            panic!("array");
        };
        assert_eq!(
            elements.iter().map(Option::is_none).collect::<Vec<_>>(),
            holes,
            "{source}"
        );
    }
}

#[test]
fn elements_accept_assignments_nested_literals_and_in_grammar() {
    for source in [
        "[a=1, b?c:d, x=>x, function(){}, {a:1}, [2]]",
        "[1][0];[].length;[]();new [];new [F][0](1)",
        "for(let a=['x' in {}];false;){}",
        "function f(a=[new.target]){return [a,()=>new.target];}",
        "[/*hole*/,/*value*/1,/*trailing*/];[\n,\n1\n,\n]",
        "[`${[1][0]}`, ...[]]",
    ] {
        if source.contains("...") {
            assert_eq!(
                parse_script(source).unwrap_err().kind,
                DiagnosticKind::Unsupported
            );
        } else {
            assert!(
                parse_script(source).is_ok(),
                "{source}: {:?}",
                parse_script(source)
            );
        }
    }
}

#[test]
fn malformed_elements_and_nested_strict_violations_are_syntax_errors() {
    for source in [
        "[",
        "[1",
        "[,",
        "[1 2]",
        "[;]",
        "[1;2]",
        "[1:2]",
        "[)]",
        "[]++",
        "++[]",
        "[]+=1",
        "([])=1",
        "'use strict';[eval=1]",
        "'use strict';[[arguments++]]",
        "'use strict';[yield]",
        "'use strict';[010]",
        "[new.target]",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    insta::assert_debug_snapshot!(parse_script("[1 2]").unwrap_err());
}

#[test]
fn spread_and_assignment_patterns_are_not_counted_as_syntax_errors() {
    for source in [
        "[...x]",
        "[1,...x]",
        "[, ...x,]",
        "[x]=a",
        "([]=a)",
        "let [x]=a",
        "([x])=>x",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
    insta::assert_debug_snapshot!(parse_script("[...x]").unwrap_err());
}

#[test]
fn nested_arrays_and_member_chains_obey_depth_limits() {
    for source in [
        format!(
            "{}0{}",
            "[".repeat(MAX_DEPTH * 2),
            "]".repeat(MAX_DEPTH * 2)
        ),
        format!("[0]{}", "[0]".repeat(MAX_DEPTH * 2)),
    ] {
        assert_eq!(
            parse_script(&source).unwrap_err().kind,
            DiagnosticKind::Limit
        );
    }
}
