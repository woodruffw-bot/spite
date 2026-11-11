//! Tagged call/member precedence, cooked escape handling, and site identity.

use spite_core::{DiagnosticKind, Span};
use spite_parser::ast::{ExprKind, StatementKind};
use spite_parser::{MAX_DEPTH, parse_script};
use std::rc::Rc;
use test_case::test_case;

#[test]
fn tagged_template_syntax_snapshot() {
    insta::assert_debug_snapshot!(
        parse_script("obj.tag`a${x}b`(y).z; new tag`constructor`(1); tag`\\x${nested`v`}tail`;")
            .unwrap()
    );
}

#[test_case("tag`a`=1", Span::new(0, 6), "invalid assignment target"; "assignment")]
#[test_case("tag`a`++", Span::new(0, 6), "invalid update target"; "postfix_update")]
#[test_case("tag`${}`", Span::new(6, 8), "expected an expression"; "missing_substitution")]
#[test_case(
    "a++`x`",
    Span::new(3, 6),
    "tagged template requires a left-hand-side expression";
    "update_as_tag"
)]
#[test_case(
    "'use strict'; tag`${arguments=1}`",
    Span::new(20, 31),
    "invalid assignment in strict mode";
    "strict_assignment_in_substitution"
)]
fn invalid_tagged_template_syntax(source: &str, expected_span: Span, expected_message: &str) {
    let diagnostic = parse_script(source).expect_err("invalid syntax must fail parsing");
    assert_eq!(diagnostic.kind, DiagnosticKind::Syntax);
    assert_eq!(diagnostic.span, expected_span);
    assert_eq!(diagnostic.message, expected_message);
}

#[test]
fn sites_survive_syntax_cloning_but_each_parse_and_literal_is_distinct() {
    let source = "tag`same`;tag`same`;";
    let script = parse_script(source).unwrap();
    let clone = script.clone();
    let parsed_again = parse_script(source).unwrap();
    let site = |script: &spite_parser::ast::Script, index: usize| {
        let StatementKind::Expression(expression) = &script.statements()[index].kind else {
            panic!("expression")
        };
        let ExprKind::TaggedTemplate { elements, .. } = &expression.kind else {
            panic!("tagged template")
        };
        elements.clone()
    };
    assert!(Rc::ptr_eq(&site(&script, 0), &site(&clone, 0)));
    assert!(!Rc::ptr_eq(&site(&script, 0), &site(&script, 1)));
    assert!(!Rc::ptr_eq(&site(&script, 0), &site(&parsed_again, 0)));
}

#[test]
fn tags_follow_member_call_and_new_precedence_and_cross_line_breaks() {
    for source in [
        "tag\n`a`",
        "obj.tag`x`.y",
        "obj['tag']`x`(1)",
        "fn()`x``y`",
        "new tag`x`",
        "new tag`x`(1)",
        "new F()`x`",
        "(x=>x)`x`",
        "`x``y`",
        "tag`a${(x,y)}b`",
        "tag`${'x' in obj}`",
        "for(tag`x`;;)break;",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    assert_eq!(parse_script("a++\n`x`").unwrap().statements().len(), 2);
    for source in [
        "'use strict'; tag`${eval=1}`",
        "'use strict'; tag`${yield}`",
        "tag`${let x}`",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    let source = format!("tag{}", "`x`".repeat(MAX_DEPTH * 2));
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}
