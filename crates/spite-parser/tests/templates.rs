//! Template components, restricted escapes, and substitution syntax.

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{
    MAX_DEPTH,
    ast::{Expr, ExprKind, StatementKind},
    parse_script,
};

#[test]
fn template_syntax_snapshot() {
    insta::assert_debug_snapshot!(parse_script("`a\\n${x}b${`c${y}`}d`").unwrap());
}

#[test]
fn template_components_preserve_raw_and_cooked_utf16() {
    for (source, cooked, raw) in [
        ("`abc`", "abc", "abc"),
        ("`a\r\nb\rc`", "a\nb\nc", "a\nb\nc"),
        (r"`\n\x41\u{1f4a9}`", "\nA💩", r"\n\x41\u{1f4a9}"),
        ("`a\\\r\nb`", "ab", "a\\\nb"),
        ("`a\\\u{2028}b`", "ab", "a\\\u{2028}b"),
        (r"`\`\${x}\q`", "`${x}q", r"\`\${x}\q"),
        ("`\u{2028}\u{2029}`", "\u{2028}\u{2029}", "\u{2028}\u{2029}"),
    ] {
        let script = parse_script(source).unwrap();
        let StatementKind::Expression(Expr {
            kind:
                ExprKind::Template {
                    elements,
                    substitutions,
                },
            ..
        }) = &script.statements()[0].kind
        else {
            panic!("template");
        };
        assert!(substitutions.is_empty());
        assert_eq!(elements[0].cooked, Some(JsString::from(cooked)), "{source}");
        assert_eq!(elements[0].raw, JsString::from(raw), "{source}");
        assert_eq!(
            &source[elements[0].span.start..elements[0].span.end],
            &source[1..source.len() - 1]
        );
    }
    let script = parse_script(r"`\ud800\u{dfff}`").unwrap();
    let StatementKind::Expression(Expr {
        kind: ExprKind::Template { elements, .. },
        ..
    }) = &script.statements()[0].kind
    else {
        panic!("template");
    };
    assert_eq!(
        elements[0].cooked,
        Some(JsString::from_code_units(vec![0xd800, 0xdfff]))
    );
}

#[test]
fn templates_do_not_introduce_or_extend_directive_prologues() {
    for source in [
        "`use strict`; let eval;",
        "``; 'use strict'; let eval;",
        "'other'; ``; 'use strict'; let eval;",
    ] {
        assert!(!parse_script(source).unwrap().is_strict(), "{source}");
    }
    assert!(parse_script("'use strict'; `x`").unwrap().is_strict());
    assert_eq!(
        parse_script("'use strict'; `${eval = 1}`")
            .unwrap_err()
            .kind,
        DiagnosticKind::Syntax
    );
}

#[test]
fn substitutions_accept_expressions_comments_and_nested_templates() {
    for source in [
        "``",
        "`${x}`",
        "`${x}${y}`",
        "`${x, y}`",
        "`${x ? y : z}`",
        "`${`a${`b`}c`}`",
        "`${'}'}`",
        "`${x /* } ` */}`",
        "`${x // } `\n}`",
        "`${x += 1}`",
        "`a` + `b`",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}

#[test]
fn malformed_untagged_templates_are_syntax_errors() {
    for source in [
        "`",
        "`a",
        "`${x",
        "`${}`",
        "`${;}`",
        "`${x y}`",
        "`x` = 1",
        "`x`++",
        r"`\1`",
        r"`\01`",
        r"`\8`",
        r"`\x`",
        r"`\x1`",
        r"`\u`",
        r"`\u{}`",
        r"`\u{110000}`",
        r"`\u{1_0}`",
        r"`\u00z0`",
        r"`\x${x}`",
        r"`${x}\9`",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn tagged_templates_remain_unsupported_even_with_invalid_cooked_escapes() {
    for source in [
        "tag`x`",
        "tag\n`x`",
        "tag`${x}`",
        r"tag`\x`",
        r"tag`\u{110000}`",
        "`x``y`",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
}

#[test]
fn template_nesting_is_bounded() {
    let source = format!(
        "{}0{}",
        "`${".repeat(MAX_DEPTH * 2),
        "}`".repeat(MAX_DEPTH * 2)
    );
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}
