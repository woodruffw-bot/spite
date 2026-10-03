//! Variable statements, declaration collection, and lexical conflicts.

use spite_core::DiagnosticKind;
use spite_parser::parse_script;

#[test]
fn var_syntax_snapshot() {
    insta::assert_debug_snapshot!(
        parse_script("var x, y = 1; for (var i = 0; i < 2; i++) var z = i;").unwrap()
    );
}

#[test]
fn vars_are_statements_and_allow_redeclarations() {
    for source in [
        "var x; var x;",
        "var x, x = 1;",
        "if (true) var x; else var y;",
        "a: var x;",
        "while (false) var x;",
        "do var x; while (false)",
        "for (var x, y = 1;;) ;",
        "var let = 1;",
        "for (var let = 1;;) ;",
        "var x; { let x; }",
        "{ let x; } var x;",
        "for (var x = 0;;) { let x; }",
        "try { var x; } catch { var x; } finally { var x; }",
        "var x\nvar y",
        "var x = (1, 2), y = 3;",
        "'use strict'; var x, x;",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}

#[test]
fn var_names_conflict_with_enclosing_lexical_declarations() {
    for source in [
        "let x; var x;",
        "var x; const x = 1;",
        "let x; { var x; }",
        "{ var x; } let x;",
        "{ let x; if (false) var x; }",
        "{ while (false) var x; let x; }",
        "let x; for (var x;;) ;",
        "for (let x;;) var x;",
        "for (const x = 1;;) { if (false) var x; }",
        "for (let x;;) for (var x;;) ;",
        "for (let x;;) try {} catch { var x; }",
        "switch (0) { case 0: let x; default: var x; }",
        "switch (0) { case 0: var x; default: let x; }",
        "try { let x; { var x; } } catch {}",
        "let x; try {} finally { var x; }",
        r"let x; var \u0078;",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn var_identifiers_and_initializers_obey_strict_mode() {
    for source in [
        "var;",
        "var x,;",
        "var x =;",
        "var if;",
        "'use strict'; var eval;",
        "'use strict'; var arguments;",
        "'use strict'; var let;",
        "'use strict'; var x = yield;",
        "'use strict'; for (var eval;;) ;",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    for source in [
        "var [x] = y;",
        "var {x} = y;",
        "for (var x in y) ;",
        "for (var x of y) ;",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
}

#[test]
fn script_var_declarations_include_every_nested_statement_form_in_order() {
    let script = parse_script(
        "var a; { var b; } if (false) var c; else var d;
        a: while (false) var e; do var f; while (false);
        for (var g, h;;) var i;
        switch (0) { case 0: var j; default: var k; }
        try { var l; } catch { var m; } finally { var n; }
        var a; let excluded;",
    )
    .unwrap();
    let names: Vec<_> = script
        .var_declarations()
        .into_iter()
        .map(|b| b.name.as_str())
        .collect();
    assert_eq!(
        names,
        [
            "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "a"
        ]
    );
}
