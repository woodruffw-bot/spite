//! Standard function declarations without Annex B block/statement extensions.

use spite_core::DiagnosticKind;
use spite_parser::{MAX_DEPTH, parse_script};

#[test]
fn function_declaration_snapshot() {
    insta::assert_debug_snapshot!(
        parse_script("function f(x) { function g() { return x; } return g; }").unwrap()
    );
}

#[test]
fn direct_functions_are_var_scoped_and_allow_redeclarations() {
    for source in [
        "function f(){}",
        "function f(){}function f(){}",
        "'use strict';function f(){}function f(){}",
        "var f;function f(){}",
        "function f(){}var f;",
        "function f(a,a){return a;}",
        "function f(a=1,b=a){return b;}",
        "'use strict';function f(a=1){return a;}",
        "function f(){var g;function g(){}}",
        "function f(){function g(){}var g;}",
        "function f(g){function g(){}return g;}",
        "function f(g=1){function g(){}return g;}",
        "function f(){function g(){}function g(){}}",
        "()=>{function f(){}function f(){}}",
        "let f;{function f(){}}",
        "var f;{function f(){}}",
        "{function f(){}}var f;",
        "{function f(){}}let f;",
        "{function f(){} {function f(){}}}",
        "for(let f=0;false;){function f(){}}",
        "function f(){'use strict';return 1;}",
        "function \\u0066(\\u0061){return a;}",
        "function f(){} (f)()",
        "try{}catch(e){{function e(){}}}",
        "switch(1){case 1:function f(){}break;}",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
}

#[test]
fn block_functions_are_lexical_and_conflicts_do_not_receive_annex_b_exceptions() {
    for source in [
        "let f;function f(){}",
        "function f(){}let f;",
        "const f=1;function f(){}",
        "{function f(){}function f(){}}",
        "{let f;function f(){}}",
        "{function f(){}let f;}",
        "{var f;function f(){}}",
        "{function f(){}var f;}",
        "{function f(){}{var f;}}",
        "{{var f;}function f(){}}",
        "function f(){let g;function g(){}}",
        "function f(){function g(){}let g;}",
        "()=>{let f;function f(){}}",
        "()=>{function f(){}let f;}",
        "switch(1){case 1:function f(){}case 2:function f(){}}",
        "switch(1){case 1:let f;default:function f(){}}",
        "switch(1){case 1:function f(){}default:{var f;}}",
        "try{}catch(e){function e(){}}",
        "try{}catch(e){function \\u0065(){}}",
        "if(1)function f(){}",
        "if(0);else function f(){}",
        "while(0)function f(){}",
        "for(;;)function f(){}",
        "do function f(){}while(0);",
        "label:function f(){}",
        "a:b:function f(){}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn declaration_parameters_strictness_and_function_control_boundaries_are_validated() {
    for source in [
        "function(){}",
        "function f(a,a){'use strict';}",
        "'use strict';function f(a,a){}",
        "function f(a=1,a){}",
        "function f(a=1){'use strict';}",
        "function eval(){'use strict';}",
        "function arguments(){'use strict';}",
        "'use strict';function yield(){}",
        "function f(){'use strict';function eval(){}}",
        "function f(){'use strict';function g(a,a){}}",
        "function f(){'use strict';function g(){010;}}",
        "function f(){'\\1';'use strict';}",
        "function f(x){let x;}",
        "function f(){break;}",
        "while(1){function f(){continue;}}",
        "outer:while(1){function f(){break outer;}}",
        "function f(){}return;",
        "function f(){}()",
        "function f(x) return x;",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn declaration_inventories_distinguish_functions_from_vars_and_nested_scopes() {
    let script=parse_script("var a;function f(){var b;function g(){} {function h(){var hidden;}} function g(){}} {function block(){}} function f(){}").unwrap();
    assert_eq!(
        script
            .var_declarations()
            .iter()
            .map(|b| b.name)
            .collect::<Vec<_>>(),
        ["a"]
    );
    let functions = script.function_declarations();
    assert_eq!(
        functions
            .iter()
            .map(|f| f.name.as_ref().unwrap().name.as_str())
            .collect::<Vec<_>>(),
        ["f", "f"]
    );
    let body = &functions[0].body;
    assert_eq!(
        body.var_declarations()
            .iter()
            .map(|b| b.name)
            .collect::<Vec<_>>(),
        ["b"]
    );
    assert_eq!(
        body.function_declarations()
            .iter()
            .map(|f| f.name.as_ref().unwrap().name.as_str())
            .collect::<Vec<_>>(),
        ["g", "g"]
    );
    assert_eq!(
        functions[0].source.as_str().unwrap(),
        "function f(){var b;function g(){} {function h(){var hidden;}} function g(){}}"
    );
}

#[test]
fn deeply_nested_declarations_hit_the_parser_limit() {
    let source = format!(
        "{}{}",
        "function f(){".repeat(MAX_DEPTH),
        "}".repeat(MAX_DEPTH)
    );
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}
