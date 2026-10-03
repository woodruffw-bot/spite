//! Edition 17 legacy literals are required in non-strict code, not Annex B options.

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{ast::*, parse_script};

fn literal(source: &str) -> Literal {
    let script = parse_script(source).unwrap();
    let StatementKind::Expression(Expr {
        kind: ExprKind::Literal(value),
        ..
    }) = &script.statements()[0].kind
    else {
        panic!("literal");
    };
    value.clone()
}

#[test]
fn leading_zero_numbers_distinguish_octal_and_decimal_productions() {
    for (source, expected) in [
        ("00", 0.0),
        ("0007", 7.0),
        ("010", 8.0),
        ("077", 63.0),
        ("010000000001", 1073741825.0),
        ("08", 8.0),
        ("09", 9.0),
        ("00078", 78.0),
        ("0789", 789.0),
        ("08.5", 8.5),
        ("09.", 9.0),
        ("09.e2", 900.0),
        ("08e1", 80.0),
        ("08.1_5e+0_2", 815.0),
        ("08e999", f64::INFINITY),
    ] {
        assert_eq!(literal(source), Literal::Number(expected), "{source}");
        assert_eq!(
            parse_script(&format!("'use strict'; {source}"))
                .unwrap_err()
                .kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    assert_eq!(
        literal(&format!("0{}", "7".repeat(400))),
        Literal::Number(f64::INFINITY)
    );
}

#[test]
fn legacy_number_token_boundaries_and_separators_remain_restricted() {
    for source in [
        "00e1",
        "01e0",
        "00.1",
        "01.8",
        "0_1",
        "01_0",
        "08_0",
        "007_8",
        "08.1_",
        "08e_1",
        "01n",
        "08n",
        "01x",
        "08π",
        "00\\u0061",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    // A dot can still begin property syntax; the lexer must not eat it as a
    // decimal fraction after an octal integer.
    assert!(parse_script("010.foo").is_ok());
    assert!(parse_script("010\n.5").is_ok());
    assert!(parse_script("'use strict'; 0; 0.5; 0e1; 0o10; 0x08; 0n;").is_ok());
}

#[test]
fn octal_escape_length_and_decimal_escape_identity_follow_the_grammar() {
    for value in 0..=255u16 {
        let source = format!("'\\{value:03o}'");
        assert_eq!(
            literal(&source),
            Literal::String(JsString::from_code_units(vec![value]))
        );
        assert_eq!(
            parse_script(&format!("'use strict'; {source}"))
                .unwrap_err()
                .kind,
            DiagnosticKind::Syntax
        );
    }
    for (source, expected) in [
        (r"'\0'", "\0"),
        (r"'\08'", "\08"),
        (r"'\09'", "\09"),
        (r"'\8\9'", "89"),
        (r"'\18'", "\u{1}8"),
        (r"'\378'", "\u{1f}8"),
        (r"'\400'", " 0"),
        (r"'\777'", "?7"),
        (r"'\0000'", "\x000"),
        (r"'\3777'", "ÿ7"),
        (r"'\1234'", "S4"),
    ] {
        assert_eq!(
            literal(source),
            Literal::String(JsString::from(expected)),
            "{source}"
        );
    }
}

#[test]
fn strict_directives_retroactively_reject_legacy_escapes() {
    for source in [
        r"'\1'; 'use strict';",
        r"'\8'; 'other'; 'use strict';",
        r"'\00'; 'use strict';",
        r"'use strict'; '\09';",
        "'use strict'; if (false) { 010; }",
        r"'use strict'; `${'\1'}`",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    for source in [
        r"'\1'; 'use\x20strict';",
        r"('\1'); 'use strict';",
        r"0; 'use strict'; '\1';",
        r"'use strict'; '\0';",
        r"'use strict'; '\x00'; '\u0000';",
        "{ 'use strict'; 010; }",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    let errors: Vec<_> = [
        r"'\7'; 'use strict';",
        "'use strict'; 010",
        r"'use strict'; '\8'",
    ]
    .map(|s| parse_script(s).unwrap_err())
    .into();
    insta::assert_debug_snapshot!(errors);
}

#[test]
fn template_escape_restrictions_do_not_depend_on_script_strictness() {
    for source in [r"`\1`", r"`\08`", r"`\8`", r"`\377`"] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax
        );
    }
    assert!(parse_script(r"`\0`").is_ok());
}
