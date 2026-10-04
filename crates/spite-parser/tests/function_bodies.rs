//! Function-body grammar, strict directives, and control/declaration boundaries.

use spite_core::DiagnosticKind;
use spite_parser::{
    MAX_DEPTH,
    ast::{ArrowBody, ExprKind, StatementKind},
    parse_script,
};

#[test]
fn block_arrows_and_returns_snapshot() {
    insta::assert_debug_snapshot!(
        parse_script("(x) => { 'use strict'; var y; if (x) return x; return; }").unwrap()
    );
}

#[test]
fn return_asi_and_body_in_parameter_are_independent_of_enclosing_expression() {
    for source in [
        "()=>{}",
        "()=>{return}",
        "()=>{return;}",
        "()=>{return 1,2}",
        "()=>{return\n1}",
        "()=>{return /*\n*/ 1}",
        "for(let f=()=>{return 'x' in o};false;);",
        "for(let f=()=>{let x = 'x' in o;};false;);",
        "()=>{return ()=>{return 1}}",
        "()=>{try{return}finally{return 2}}",
        "x=>{var x;{let x;} return x;}",
        "()=>{loop:while(true){break loop;} return 1;}",
        "outer:while(true){let f=()=>{outer:while(true){continue outer;}};break;}",
        "()=>{'use\\x20strict';var eval;}",
        "()=>{('use strict');var eval;}",
        "()=>{0;'use strict';var eval;}",
        "()=>{'use strict';return 1};var eval;",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
    let script = parse_script("()=>{return\n1}").unwrap();
    let StatementKind::Expression(expr) = &script.statements()[0].kind else {
        panic!()
    };
    let ExprKind::Arrow {
        body: ArrowBody::Block(body),
        source,
        ..
    } = &expr.kind
    else {
        panic!()
    };
    assert!(matches!(
        body.statements()[0].kind,
        StatementKind::Return(None)
    ));
    assert!(matches!(
        body.statements()[1].kind,
        StatementKind::Expression(_)
    ));
    assert_eq!(source.as_str(), "()=>{return\n1}");
}

#[test]
fn function_boundaries_reset_control_targets_and_return_permission() {
    for source in [
        "return;",
        "{return 1}",
        "()=>{};return 1",
        "while(true){let f=()=>{break;};}",
        "while(true){let f=()=>{continue;};}",
        "outer:while(true){let f=()=>{break outer;};}",
        "outer:while(true){let f=()=>{continue outer;};}",
        "()=>{while(true){let f=()=>{break;};}}",
        "()=>{switch(1){case 1:let f=()=>{break;};}}",
        "()=>{return 1 2}",
        "()=>{return",
        "()=>{return;",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn function_strictness_and_parameter_conflicts_are_early_errors() {
    for source in [
        "x=>{let x;}",
        "x=>{const x=1;}",
        "x=>{let y;var y;}",
        "x=>{let y;{var y;}}",
        "x=>{let y;let y;}",
        "eval=>{'use strict';}",
        "arguments=>{'use strict';}",
        "package=>{'use strict';}",
        "()=>{'use strict';eval=1;}",
        "()=>{'use strict';var arguments;}",
        "()=>{'use strict';010;}",
        "()=>{'\\1';'use strict';}",
        "()=>{'use strict';()=>010;}",
        "()=>{'use strict';()=>{var eval;}}",
        "'use strict';()=>{010;}",
        "()=>{'use strict';return ()=>{delete x;}}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn script_and_function_var_lists_exclude_nested_functions() {
    let script = parse_script("var a; ()=>{var b; if(0){var c;} ()=>{var d;};}").unwrap();
    assert_eq!(
        script
            .var_declarations()
            .iter()
            .map(|b| b.name.as_str())
            .collect::<Vec<_>>(),
        ["a"]
    );
    let StatementKind::Expression(expr) = &script.statements()[1].kind else {
        panic!()
    };
    let ExprKind::Arrow {
        body: ArrowBody::Block(body),
        ..
    } = &expr.kind
    else {
        panic!()
    };
    assert_eq!(
        body.var_declarations()
            .iter()
            .map(|b| b.name.as_str())
            .collect::<Vec<_>>(),
        ["b", "c"]
    );
}

#[test]
fn mixed_function_and_statement_nesting_is_bounded() {
    let source = format!("{}0{}", "()=>{{".repeat(MAX_DEPTH), "}}".repeat(MAX_DEPTH));
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}
