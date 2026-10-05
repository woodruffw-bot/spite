//! For-in headers, declaration inventories, and static semantics (14.7.5).

use spite_core::DiagnosticKind;
use spite_parser::{MAX_DEPTH, parse_script};

#[test]
fn syntax_and_diagnostics_snapshots() {
    insta::assert_debug_snapshot!(parse_script("for(obj[key()] in source) ; for(var x in (a,b)) x; a:for(let y in object) continue a; for(const z in null) {z;}").unwrap());
    insta::assert_debug_snapshot!(parse_script("for(let x=1 in {}) ;").unwrap_err());
    insta::assert_debug_snapshot!(parse_script("'use strict';for(var x=1 in {}) ;").unwrap_err());
}

#[test]
fn references_bindings_and_comma_rhs_are_supported() {
    for source in [
        "for(x in {}) ;",
        "for((x) in {}) ;",
        "for(obj.x in {}) ;",
        "for(obj[key()] in {}) ;",
        "for(async in {}) ;",
        "for(let in {}) ;",
        "for(let instanceof Object;false;) ;",
        "for(let.x in {}) ;",
        "for(var let in {}) ;",
        "for(let of in {}) ;",
        "for(const async in {}) ;",
        "for(let x in a,b) ;",
        "for(const x in 'p' in obj) ;",
        "for(var x in obj) {var x;}",
        "for(let x in obj) {let x;}",
        "a:b:for(const x in obj) continue a;",
        "for(let\nx in obj) ;",
        r"for(\u0061 in obj) ;",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    let script = parse_script("for(var x in {}) {var y;} for(let z in {}) {var a;}").unwrap();
    let names: Vec<_> = script.var_declarations().iter().map(|b| b.name).collect();
    assert_eq!(names, ["x", "y", "a"]);
}

#[test]
fn invalid_targets_declarations_and_lexical_conflicts_are_syntax_errors() {
    for source in [
        "for(1 in {}) ;",
        "for(x=1 in {}) ;",
        "for(x,y in {}) ;",
        "for(this in {}) ;",
        "for(var x,y in {}) ;",
        "for(let x,y in {}) ;",
        "for(let x=1 in {}) ;",
        "for(const x=1 in {}) ;",
        "for(let x in {}) {var x;}",
        "let x;for(var x in {}) ;",
        "'use strict';for(eval in {}) ;",
        "'use strict';for(var arguments in {}) ;",
        "'use strict';for(var arguments=1 in {}) ;",
        "'use strict';for(var eval=1 in {}) ;",
        "'use strict';for(var x=1 in {}) ;",
        "for(x in {}) ;continue;",
        "a:{for(x in {}) continue a;}",
        "for(x in {}) const y=1;",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    for name in ["arguments", "eval"] {
        let source = format!("'use strict';for(var {name}=1 in {{}}) ;");
        let diagnostic = parse_script(&source).unwrap_err();
        assert_eq!(diagnostic.message, "invalid binding in strict mode");
        assert_eq!(&source[diagnostic.span.start..diagnostic.span.end], name);
    }
}

#[test]
fn patterns_parse_and_annex_b_initializers_remain_unsupported() {
    assert!(parse_script("for({x} in {}) ;").is_ok());
    assert_eq!(
        parse_script("for(var x=1 in {}) ;").unwrap_err().kind,
        DiagnosticKind::Unsupported
    );
    let source = format!("{};", "for(let x in {}) ".repeat(MAX_DEPTH * 2));
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}
