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
    assert_eq!(
        bindings[0].pattern.kind,
        BindingPatternKind::Identifier("π".into())
    );
    assert_eq!(bindings[0].pattern.span, Span::new(4, 6));
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

#[test]
fn identifier_escapes_decode_names_and_preserve_source_spans() {
    let source = r"let \u{10400} = 1; \uD801;";
    let error = parse_script(source).unwrap_err();
    assert_eq!(&source[error.span.start..error.span.end], r"\uD801");
    assert_eq!(error.message, "invalid identifier start escape");

    let source = r"let \u{10400} = 1; \u{010400};";
    let script = parse_script(source).unwrap();
    let StatementKind::Lexical { bindings, .. } = &script.statements()[0].kind else {
        panic!("expected declaration")
    };
    assert_eq!(
        bindings[0].pattern.kind,
        BindingPatternKind::Identifier("𐐀".into())
    );
    assert_eq!(
        &source[bindings[0].pattern.span.start..bindings[0].pattern.span.end],
        r"\u{10400}"
    );
    let StatementKind::Expression(reference) = &script.statements()[1].kind else {
        panic!("expected reference")
    };
    assert_eq!(reference.kind, ExprKind::Identifier("𐐀".into()));
    assert_eq!(
        &source[reference.span.start..reference.span.end],
        r"\u{010400}"
    );
    insta::assert_debug_snapshot!(script);
}

#[test]
fn valid_start_and_part_escapes() {
    for name in [
        r"\u0061",
        r"\u{000000000000000061}",
        r"\u0024",
        r"\u005f",
        r"\u03c0",
        r"\u{10400}",
        r"a\u0301",
        r"a\u200C",
        r"a\u200D",
        r"a\u{E0100}",
        r"a\u0062c",
        r"\u{18e00}",
        r"\u{3d000}",
    ] {
        assert!(
            parse_script(&format!("let {name} = 1; {name};")).is_ok(),
            "{name}"
        );
    }
}

#[test]
fn invalid_escapes_are_syntax_errors() {
    for name in [
        r"\x61",
        r"\U0061",
        r"\u",
        r"\u123",
        r"\u00gg",
        r"\u{}",
        r"\u{61",
        r"\u{110000}",
        r"\u{1_0000}",
        r"\uD800",
        r"\u{DFFF}",
        r"\uD801\uDC00",
        r"\u0030",
        r"\u0300",
        r"\u200C",
        r"\u200D",
        r"\u0020",
        r"a\u0020",
        r"a\u002d",
        r"a\u{1f4a9}",
        r"a\uFEFF",
        r"a\q",
        "a\\\n",
        "a\\",
    ] {
        let source = format!("let {name} = 1;");
        assert_eq!(
            parse_script(&source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{name}"
        );
    }
    let source = r"\u{110000}; a\u0020; \u0030;";
    let errors: Vec<_> = source
        .split(';')
        .filter(|s| !s.trim().is_empty())
        .map(|s| parse_script(s.trim()).unwrap_err())
        .collect();
    insta::assert_debug_snapshot!(errors);
}

#[test]
fn escaped_reserved_words_never_become_keywords_or_literals() {
    for keyword in [
        "break",
        "case",
        "catch",
        "class",
        "const",
        "continue",
        "debugger",
        "default",
        "delete",
        "do",
        "else",
        "enum",
        "export",
        "extends",
        "finally",
        "for",
        "function",
        "if",
        "import",
        "in",
        "instanceof",
        "new",
        "return",
        "super",
        "switch",
        "this",
        "throw",
        "try",
        "typeof",
        "var",
        "void",
        "while",
        "with",
        "null",
        "true",
        "false",
    ] {
        let first = keyword.chars().next().unwrap() as u32;
        for escaped in [
            format!("\\u{first:04x}{}", &keyword[1..]),
            format!("\\u{{{first:x}}}{}", &keyword[1..]),
        ] {
            for source in [
                format!("{escaped};"),
                format!("let {escaped} = 1;"),
                format!("{escaped} = 1;"),
            ] {
                assert_eq!(
                    parse_script(&source).unwrap_err().kind,
                    DiagnosticKind::Syntax,
                    "{source}"
                );
            }
        }
    }
    for source in [
        r"\u0069f (true) 1;",
        r"\u0074hrow 1;",
        r"\u0074ypeof missing;",
        r"\u0076oid 0;",
        r"\u0063onst a = 1;",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn contextual_identifiers_and_strict_early_errors_use_decoded_names() {
    for name in [
        r"aw\u0061it",
        r"y\u0069eld",
        r"\u0065val",
        r"arg\u0075ments",
        r"st\u0061tic",
    ] {
        assert!(
            parse_script(&format!("let {name} = 1; {name};")).is_ok(),
            "{name}"
        );
    }
    assert!(parse_script(r"'use strict'; let aw\u0061it = 1; aw\u0061it;").is_ok());
    assert!(parse_script(r"l\u0065t;").is_ok());
    for source in [
        r"l\u0065t a = 1;",
        r"let l\u0065t = 1;",
        r"let a; let \u0061;",
        r"let π; let \u03c0;",
        r"'use strict'; l\u0065t;",
        r"'use strict'; y\u0069eld;",
        r"'use strict'; let \u0065val;",
        r"'use strict'; arg\u0075ments = 1;",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    for name in [
        "implements",
        "interface",
        "let",
        "package",
        "private",
        "protected",
        "public",
        "static",
        "yield",
    ] {
        let escaped = format!(
            "\\u{:04x}{}",
            name.chars().next().unwrap() as u32,
            &name[1..]
        );
        for source in [
            format!("'use strict'; {escaped};"),
            format!("'use strict'; let {escaped};"),
            format!("'use strict'; {escaped} = 1;"),
        ] {
            assert_eq!(
                parse_script(&source).unwrap_err().kind,
                DiagnosticKind::Syntax,
                "{source}"
            );
        }
    }
}
