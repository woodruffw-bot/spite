//! Identifier rest parameters, non-simple lists, and static semantics (15.2.3).

use spite_core::DiagnosticKind;
use spite_parser::{MAX_DEPTH, parse_script};

#[test]
fn syntax_and_diagnostics_snapshots() {
    insta::assert_debug_snapshot!(parse_script(
        "function f(a=1,...rest){return rest;} let g=(x,...xs)=>xs; ({m(...values){return values;}})"
    ).unwrap());
    insta::assert_debug_snapshot!(parse_script("(...rest,)=>rest").unwrap_err());
    insta::assert_debug_snapshot!(parse_script("function f(...rest){'use strict';}").unwrap_err());
}

#[test]
fn rest_is_final_has_no_initializer_and_obeys_binding_rules() {
    for source in [
        "(...r)=>r",
        "(x,...r)=>r",
        "(x=1,...r)=>r",
        "function f(...r){}",
        "function f(a,b,...r){}",
        "(function(...r){})",
        "({m(...r){}})",
        "'use strict';function f(...r){}",
        "function f(...arguments){}",
        "(...let)=>let",
        "function f(...yield){}",
        r"function f(...\u0061){}",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    for source in [
        "(...r,)=>r",
        "(...r,x)=>r",
        "(...r,...x)=>r",
        "(...r=[])=>r",
        "function f(...r,){}",
        "function f(...r,x){}",
        "function f(...r=[]){}",
        "function f(a,...a){}",
        "(a,...a)=>a",
        "(...r)=>{let r;}",
        "function f(...r){const r=1;}",
        "function f(...r){'use strict';}",
        "(...r)=>{'use strict';}",
        "({m(...r){'use strict';}})",
        "'use strict';(...eval)=>eval",
        "'use strict';function f(...arguments){}",
        "(...r)\n=>r",
        "...r=>r",
        "({set x(...r){}})",
        "({get x(...r){}})",
        "function f(...){}",
        "function f(......r){}",
        "function f(...this){}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn rest_patterns_remain_unsupported_and_nested_defaults_obey_depth_guards() {
    for source in ["(...[a])=>a", "function f(...{a}){}", "({m(...[a]){}})"] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
    let source = format!(
        "{}0{}",
        "(x=".repeat(MAX_DEPTH * 2),
        ",...r)=>r".repeat(MAX_DEPTH * 2)
    );
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}
