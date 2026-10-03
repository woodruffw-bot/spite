//! Label grammar and control-target early errors.

use spite_core::DiagnosticKind;
use spite_parser::{MAX_DEPTH, parse_script};

#[test]
fn labelled_syntax_snapshot() {
    let script =
        parse_script(r"a: b: while (true) { if (false) continue \u0061; break b; }").unwrap();
    insta::assert_debug_snapshot!(script);
}

#[test]
fn labelled_diagnostics_snapshot() {
    let errors: Vec<_> = [
        "a: { a: ; }",
        "while (false) break missing;",
        "a: { while (false) continue a; }",
        "'use strict'; yield: ;",
        "a: function f() {}",
    ]
    .map(|s| parse_script(s).unwrap_err())
    .into();
    insta::assert_debug_snapshot!(errors);
}

#[test]
fn labels_are_scoped_and_separate_from_binding_names() {
    for source in [
        "a: ; a: ;",
        "a: { break a; } a: {}",
        "let a; a: { let a; break a; }",
        "a: b: while (false) continue a;",
        "a: b: do continue b; while (false);",
        "a: if (true) break a; else ;",
        "a\n: { break a; }",
        "'use strict'; eval: arguments: { break eval; }",
        "await: { break await; }",
        "yield: { break yield; }",
        "let: { break let; }",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    for source in [
        "a: a: ;",
        "a: { a: ; }",
        "a: if (true) a: ;",
        "a: while (false) a: ;",
        "a: ; break a;",
        "break a; a: ;",
        "a: { break b; }",
        "a: { break; }",
        "a: continue a;",
        "a: { while (false) continue a; }",
        "a: if (true) while (false) continue a;",
        "a: b: { while (false) continue b; }",
        "a: while (false) ; while (false) continue a;",
        "a: let x;",
        "a: const x = 1;",
        "a: function f() {}",
        "a: b: function f() {}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn label_names_are_decoded_without_normalization() {
    for source in [
        r"\u0061: { break a; }",
        r"a: { break \u0061; }",
        r"\u03c0: while (false) continue π;",
        r"𐐀: while (false) continue \u{10400};",
        "é: e\u{301}: { break é; }",
        "A: a: { break A; }",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    for source in [
        r"a: \u0061: ;",
        r"\u0061: a: ;",
        "é: { break e\u{301}; }",
        r"while (false) break \u0069f;",
        r"\u0069f: ;",
        "'use strict'; let: ;",
        "'use strict'; while (false) break yield;",
        "'use strict'; implements: ;",
        r"'use strict'; y\u0069eld: ;",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn labels_remain_subject_to_the_nesting_limit() {
    let source = (0..MAX_DEPTH * 2)
        .map(|i| format!("label{i}: "))
        .collect::<String>()
        + ";";
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}
