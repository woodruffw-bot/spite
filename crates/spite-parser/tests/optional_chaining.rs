//! Optional-chain grouping, references, and early errors (13.3.10).

use spite_core::{DiagnosticKind, Span};
use spite_parser::parse_script;
use test_case::test_case;

#[test]
fn optional_chain_syntax_snapshot() {
    insta::assert_debug_snapshot!(parse_script("obj?.a[key]?.(x,...ys).b; (obj?.a)(x); delete obj?.[key]; new (obj?.Ctor)(x); true?.30:false;").unwrap());
}

#[test_case("a?.b=1", Span::new(0, 4), "invalid assignment target"; "assignment")]
#[test_case("a?.b++", Span::new(0, 4), "invalid update target"; "postfix_update")]
#[test_case("--a?.b", Span::new(2, 6), "invalid update target"; "prefix_update")]
#[test_case(
    "new a?.b()",
    Span::new(4, 8),
    "optional chain is not a constructor expression";
    "constructor"
)]
#[test_case(
    "a?.`x`",
    Span::new(3, 6),
    "optional chains cannot be tagged templates";
    "tagged_template"
)]
#[test_case(
    "a?.b\n`x`",
    Span::new(5, 8),
    "optional chains cannot be tagged templates";
    "tagged_template_after_newline"
)]
#[test_case(
    "'use strict'; a?.[eval=1]",
    Span::new(18, 24),
    "invalid assignment in strict mode";
    "strict_assignment_in_computed_key"
)]
fn invalid_optional_chain_syntax(source: &str, expected_span: Span, expected_message: &str) {
    let diagnostic = parse_script(source).expect_err("invalid syntax must fail parsing");
    assert_eq!(diagnostic.kind, DiagnosticKind::Syntax);
    assert_eq!(diagnostic.span, expected_span);
    assert_eq!(diagnostic.message, expected_message);
}

#[test]
fn grouping_and_syntax_parameters_preserve_the_chain_boundary() {
    for source in [
        "a?.b.c(d)?.[e]",
        "a?.(b)?.c",
        "a\n?.b",
        "a?.\n[b]",
        "a?.true",
        "a?.null",
        "a?.\\u0061",
        "(a?.tag)`x`",
        "new (a?.Ctor)()",
        "(a?.b).c=1",
        "delete a?.b",
        "a?.[b in c]",
        "for(a?.b;;)break;",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    for source in [
        "a?.",
        "a?.[]",
        "a?.(,)",
        "(a?.b)=1",
        "a?.b.c+=1",
        "for(a?.b in c){}",
        "for(a?.b of c){}",
        "new a?.()",
        "a?.b()`x`",
        "'use strict'; a?.[yield]",
        "'use strict'; a?.(arguments=1)",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn flat_chain_steps_do_not_consume_recursive_syntax_depth() {
    let source = format!("a?.b{}", ".b".repeat(1000));
    assert!(parse_script(&source).is_ok());
}
