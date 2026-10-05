//! Synchronous for-of grammar and early errors (14.7.5).

use spite_core::DiagnosticKind;
use spite_parser::{MAX_DEPTH, parse_script};

#[test]
fn syntax_and_diagnostics_snapshots() {
    insta::assert_debug_snapshot!(parse_script(
        "for(x of source) x; for(var v of values) ; a:b:for(let y of iterable) continue a; for(const z of 'ab') {z;}"
    ).unwrap());
    insta::assert_debug_snapshot!(parse_script("for(let x=1 of []) ;").unwrap_err());
    insta::assert_debug_snapshot!(parse_script("'use strict';for(eval of []) ;").unwrap_err());
}

#[test]
fn supported_targets_bindings_and_rhs_grammar() {
    for source in [
        r"for(\u0061sync of []) ;",
        r"for(l\u0065t.x of []) ;",
        "for(of of []) ;",
        "for(let of of []) ;",
        "for(const of of []) ;",
        "for(var let of []) ;",
        "for((let) of []) ;",
        "for((async) of []) ;",
        "for(async.x of []) ;",
        "for(let async of []) ;",
        "for(obj[key()] of []) ;",
        "for(obj.x of (a,b)) ;",
        "for(x of 'key' in obj) ;",
        "for(x of a=b) ;",
        "for(let x of []) {let x;}",
        "for(let x of []) ; let x;",
        "a:b:for(const x of []) continue a;",
        "for(var x of []) {var x;}",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}

#[test]
fn invalid_targets_initializers_scopes_and_controls_are_syntax_errors() {
    for source in [
        r"for(var x o\u0066 []) ;",
        "for(let of []) ;",
        "for(let.x of []) ;",
        "for(async of []) ;",
        "for(1 of []) ;",
        "for(x=1 of []) ;",
        "for(x,y of []) ;",
        "for(var x=1 of []) ;",
        "for(let x=1 of []) ;",
        "for(const x=1 of []) ;",
        "for(var x,y of []) ;",
        "for(let x,y of []) ;",
        "for(const x,y of []) ;",
        "for(x of a,b) ;",
        "for(let x of []) {var x;}",
        "for(const x of []) {var x;}",
        "let x;for(var x of []) ;",
        "for(let x of []) {if(false){var x;}}",
        "'use strict';for(eval of []) ;",
        "'use strict';for((arguments) of []) ;",
        "'use strict';for(var eval of []) ;",
        "'use strict';for(let arguments of []) ;",
        "for(x of []) let y;",
        "for(x of []) ; continue;",
        "a:{for(x of []) continue a;}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn patterns_parse_and_async_iteration_stays_unsupported() {
    assert!(parse_script("for([x] of []) ;").is_ok());
    assert!(parse_script("for({x} of []) ;").is_ok());
    assert_eq!(
        parse_script("for await(x of []) ;").unwrap_err().kind,
        DiagnosticKind::Unsupported
    );
    let source = format!("{};", "for(let x of []) ".repeat(MAX_DEPTH * 2));
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}

#[test]
fn var_declarations_include_headers_and_nested_bodies() {
    let script = parse_script("for(var x of []) {var y;} for(const a of []) {var z;}").unwrap();
    let names: Vec<_> = script.var_declarations().iter().map(|b| b.name).collect();
    assert_eq!(names, ["x", "y", "z"]);
}
