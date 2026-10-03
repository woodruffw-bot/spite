//! ECMAScript identifier grammar and early errors.

use spite_core::{DiagnosticKind, Span};
use spite_parser::{ast::*, parse_script};

#[test]
fn unicode_identifier_names_and_byte_spans() {
    let source = "let π = 1; π;";
    let script = parse_script(source).unwrap();
    let StatementKind::Lexical { bindings, .. } = &script.statements()[0].kind else {
        panic!("expected declaration")
    };
    assert_eq!(bindings[0].name, "π");
    assert_eq!(bindings[0].span, Span::new(4, 6));
    let StatementKind::Expression(reference) = &script.statements()[1].kind else {
        panic!("expected reference")
    };
    assert_eq!(reference.kind, ExprKind::Identifier("π".into()));
    assert_eq!(reference.span, Span::new(12, 14));
    insta::assert_debug_snapshot!(script);
}

#[test]
fn literal_identifiers_use_id_properties_not_alphabetic_properties() {
    for name in [
        "π",
        "字",
        "𐐀",
        "℘",
        "℮",
        "゛",
        "ᢅ",
        "e\u{0301}",
        "a\u{00b7}",
        "a\u{0660}",
        "a\u{200c}",
        "a\u{200d}",
        "a\u{203f}",
        "a\u{e0100}",
        "ifπ",
        "null字",
        "true\u{200c}",
        "\u{18e00}",
        "\u{3d000}",
    ] {
        let source = format!("let {name} = 1; {name};");
        assert!(parse_script(&source).is_ok(), "{name:?}");
    }
    for name in [
        "💩",
        "\u{0300}a",
        "\u{0660}a",
        "\u{200c}a",
        "\u{200d}a",
        "\u{203f}a",
        "\u{2e2f}",
        "\u{0378}",
        "a💩",
        "a\u{10ffff}",
    ] {
        let source = format!("let {name} = 1;");
        assert_eq!(
            parse_script(&source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{name:?}"
        );
    }
}

#[test]
fn number_cannot_be_followed_by_an_identifier_start() {
    for source in ["1π", "1℘", "0b1字", "0x1𐐀", "1e2\u{18e00}"] {
        let error = parse_script(source).unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
        assert_eq!(
            error.message, "invalid character after numeric literal",
            "{source}"
        );
    }
}

#[test]
fn names_are_not_normalized() {
    assert!(parse_script("let é = 1; let e\u{0301} = 2;").is_ok());
    assert!(parse_script("let K = 1; let K = 2;").is_ok());
    assert_eq!(
        parse_script("let π; let π;").unwrap_err().kind,
        DiagnosticKind::Syntax
    );
}
