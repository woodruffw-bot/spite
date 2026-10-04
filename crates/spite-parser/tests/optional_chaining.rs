//! Optional-chain grouping, references, and early errors (13.3.10).

use spite_core::DiagnosticKind;
use spite_parser::parse_script;

#[test]
fn syntax_and_diagnostic_snapshots() {
    insta::assert_debug_snapshot!(parse_script("obj?.a[key]?.(x,...ys).b; (obj?.a)(x); delete obj?.[key]; new (obj?.Ctor)(x); true?.30:false;").unwrap());
    for source in [
        "a?.b=1",
        "a?.b++",
        "--a?.b",
        "new a?.b()",
        "a?.`x`",
        "a?.b\n`x`",
        "'use strict'; a?.[eval=1]",
    ] {
        insta::assert_debug_snapshot!(parse_script(source).unwrap_err());
    }
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
