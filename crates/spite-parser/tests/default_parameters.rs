//! Default parameter cover grammar and non-simple parameter early errors.

use spite_core::DiagnosticKind;
use spite_parser::{MAX_DEPTH, parse_script};

#[test]
fn default_parameters_snapshot() {
    insta::assert_debug_snapshot!(parse_script("(x = 1, y = x, z,) => y + z").unwrap());
}

#[test]
fn defaults_accept_assignment_expressions_and_restore_in_context() {
    for source in [
        "(x=1)=>x",
        "(x=1,y)=>x+y",
        "(x,y=1,)=>x+y",
        "(x=(1,2))=>x",
        "(x=y=1)=>x",
        "(x=()=>1)=>x()",
        "(x=()=>{return 1})=>x()",
        "(x=(y=1)=>y)=>x()",
        "(x={a:1})=>x.a",
        "(x=`a${1}`)=>x",
        "(x=true?1:2)=>x",
        "(x='a' in o)=>x",
        "for(let f=(x='a' in o)=>x;false;);",
        "for(let f=(x=1)=>(x in o);false;);",
        "'use strict';(x=1)=>x",
        "'use strict';(x=1)=>{return x}",
        "(x=()=>{'use strict';return 1})=>x()",
        "(x=1)=>{('use strict');return x}",
        "(x=1)=>{'use\\x20strict';return x}",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
}

#[test]
fn defaults_require_parentheses_and_reject_duplicate_or_strictly_invalid_bindings() {
    for source in [
        "x=1=>x",
        "(x=)=>x",
        "(x=1,x)=>x",
        "(x,x=1)=>x",
        "(x=1,x=2)=>x",
        "(x=1)=>{'use strict';return x}",
        "(eval=1)=>{'use strict';}",
        "'use strict';(eval=1)=>1",
        "'use strict';(x=eval=1)=>x",
        "'use strict';(x=010)=>x",
        "()=>{'use strict';return (x=010)=>x;}",
        "(x=1)=>{let x;}",
        "(x=1)\n=>x",
        "(x=1)=>x in o; return;",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    for source in ["(...x)=>x", "([x]=[])=>x", "({x}={})=>x"] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
}

#[test]
fn defaults_participate_in_syntax_nesting_limits() {
    let source = format!("{}1{}", "(x=".repeat(MAX_DEPTH), ")=>x".repeat(MAX_DEPTH));
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}
