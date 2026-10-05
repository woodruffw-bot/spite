//! Direct PerformEval's caller and declaration semantics (19.2.1.1–3).
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn direct_reference_forms_use_caller_bindings_after_all_argument_effects() {
    for call in ["eval", "(eval)", "e\\u0076al"] {
        check(&format!(
            "let x=1;function f(){{let x=7,n=0;return {call}('x+n',n++)===8 && n===1;}}f()"
        ));
    }
    check("let x=1;function f(eval){let x=7;return eval('x')===7;}f(globalThis.eval)");
    check("let x=1;function f(){let x=7;with({eval:globalThis.eval}){return eval('x')===7;}}f()");
    check(
        "let x=1;function f(){let x=7,indirect=eval;return indirect('x')===1 && globalThis.eval('x')===1 && eval?.('x')===1 && eval('x')===7;}f()",
    );
    check("eval('')===undefined && eval('/*comment*/')===undefined && eval('1;var x;')===1");
}

#[test]
fn sloppy_declarations_enter_the_caller_variable_environment_and_are_deletable() {
    check(
        "function f(){eval('var x=7;function g(){return x;}function g(){return x+1;}');return x===7 && g()===8 && delete x && delete g && typeof x==='undefined' && typeof g==='undefined';}f() && typeof x==='undefined' && typeof g==='undefined'",
    );
    check(
        "function f(){var x=3;eval('var x;function g(){return 7;}');return x===3 && !delete x && delete g;}f()",
    );
    check(
        "function f(x){eval('var x=7;');if(arguments[0]!==7 || delete x)return false;eval('function x(){return 9;}');return x()===9 && arguments[0]===x && !delete x;}f(3)",
    );
    check("let x=19;function f(){eval('var x=7;');return x===7;}f() && x===19");
    check(
        "eval('var x=7;function g(){return x;}');Object.getOwnPropertyDescriptor(globalThis,'x').configurable && delete g && delete x",
    );
}

#[test]
fn eval_lexicals_are_fresh_with_tdz_and_function_closures_capture_them() {
    check(
        "function f(){let x=19;eval('let x=7;const y=2;function saved(){return x+y;}');eval('let x=33;');return x===19 && saved()===9 && typeof y==='undefined';}f()",
    );
    check(
        "function f(){let x=7,caught=false;try{eval('x;let x;');}catch(e){caught=e instanceof ReferenceError;}return caught && x===7;}f()",
    );
    check(
        "function f(){let caught=false;try{eval('const x=7;x=9;');}catch(e){caught=e instanceof TypeError;}return caught && typeof x==='undefined';}f()",
    );
    check("function f(){let x=7;return eval('let x=9;()=>x')()===9 && x===7;}f()");
}

#[test]
fn lexical_conflicts_are_checked_before_declarations_and_effects() {
    for body in [
        "var fresh;var x;",
        "function fresh(){};function x(){}",
        "var fresh;{var x;}",
    ] {
        check(&format!(
            "function f(){{let x=7,caught=false;try{{eval('{body}globalThis.effect=9;');}}catch(e){{caught=e instanceof SyntaxError;}}return caught && typeof fresh==='undefined' && !('effect' in globalThis);}}f()"
        ));
    }
    check(
        "let caught=false;{let x=7;try{eval('var x;');}catch(e){caught=e instanceof SyntaxError;}}caught && !('x' in globalThis)",
    );
    check(
        "function f(){try{throw 7;}catch(x){try{eval('var fresh;var x;');}catch(e){return e instanceof SyntaxError && typeof fresh==='undefined' && x===7;}}}f()",
    );
    check("function f(){try{eval('var x;');}catch(e){return e instanceof SyntaxError;}let x;}f()");
}

#[test]
fn with_environments_are_skipped_for_conflicts_and_initializers_resolve_live() {
    check(
        "function f(){let object={x:3};with(object){eval('var x=7;function g(){return x;}');}return object.x===7 && x===undefined && g()===7 && delete x && delete g;}f()",
    );
    check(
        "function f(){let count=0,object={get x(){count++;return 3;}};with(object){eval('var x;');}return count===0 && x===undefined && delete x;}f()",
    );
    check(
        "function f(){let object={x:3,[Symbol.unscopables]:{x:true}};with(object){eval('var x=7;');}return object.x===3 && x===7;}f()",
    );
}

#[test]
fn strictness_is_inherited_and_eval_vars_remain_local() {
    check(
        r#"function f(){'use strict';let x=19;let read=eval('var x=7;function g(){return x;}g;');return x===19 && read()===7 && typeof g==='undefined';}f()"#,
    );
    check(
        r#"function f(){let x=19;let read=eval('"use strict";var x=7;()=>x;');return x===19 && read()===7;}f()"#,
    );
    for body in [
        "with({}){}",
        "var eval;",
        "var arguments;",
        "010;",
        "'\\1';",
        "delete x;",
    ] {
        check(&format!(
            "function f(){{'use strict';let caught=false;try{{eval({body:?});}}catch(e){{caught=e instanceof SyntaxError;}}return caught;}}f()"
        ));
    }
    check(r#"function f(){'use strict';return eval('this')===undefined;}f()"#);
    check(
        r#"function f(){'use strict';let caught=false;try{eval('missing=7;');}catch(e){caught=e instanceof ReferenceError;}return caught && typeof missing==='undefined';}f()"#,
    );
}

#[test]
fn parameter_default_eval_uses_separate_parameter_and_body_environments() {
    for function in ["function f", "let f="] {
        let head = if function == "function f" {
            "function f"
        } else {
            "let f="
        };
        let arrow = if function == "function f" { "" } else { "=>" };
        check(&format!(
            "{head}(a=eval('var x=7;'),read=()=>x){arrow}{{var x;return read()===7 && x===undefined && !delete x;}};f() && typeof x==='undefined'"
        ));
        check(&format!(
            "{head}(a=eval('var x=7;')){arrow}{{return x===7 && delete x && typeof x==='undefined';}};f()"
        ));
        check(&format!(
            "{head}(a=eval('var a;')){arrow}{{return false;}};let caught=false;try{{f();}}catch(e){{caught=e instanceof SyntaxError;}}caught"
        ));
    }
    check("function f(a=eval('var x=7;'),read=()=>x){var x=9;return read()===7 && x===9;}f()");
    check("function f(a=7,b=eval('a=9;')){var a;return a===9;}f()");
    check(
        "function f(a=eval('var arguments;')){}let caught=false;try{f();}catch(e){caught=e instanceof SyntaxError;}caught",
    );
    check("function f(...x){eval('var x;');return x[0]===7 && !delete x;}f(7)");
}

#[test]
fn this_and_new_target_follow_the_nearest_non_arrow_function() {
    check(
        "let object={x:7,m(){return eval('this.x')===7 && eval('new.target')===undefined;}};object.m()",
    );
    check(
        "function F(){this.ok=eval('new.target')===F && eval('(()=>new.target)()')===F && (()=>eval('new.target'))()===F;}new F().ok",
    );
    check(
        "function f(){return eval('new.target')===undefined && (()=>eval('new.target'))()===undefined;}f()",
    );
    check("let object={x:7,m(){return (()=>eval('this.x'))()===7;}};object.m()");
    for source in [
        "eval('new.target')",
        "(()=>eval('new.target'))()",
        "eval('return 7;')",
        "function f(){eval('return 7;');}f()",
    ] {
        check(&format!(
            "let caught=false;try{{{source};}}catch(e){{caught=e instanceof SyntaxError;}}caught"
        ));
    }
}

#[test]
fn method_super_context_is_inherited_only_through_arrows() {
    for source in [
        "({m(){return eval('super.x');}}).m()===undefined",
        "({m(){return (()=>eval('super.x'))();}}).m()===undefined",
        "({get x(){return eval('super.x');}}).x===undefined",
    ] {
        check(source);
    }
    for source in [
        "eval('super.x')",
        "function f(){eval('super.x');}f()",
        "({m(){return function(){eval('super.x');};}}).m()()",
        "({m(){eval('super()');}}).m()",
    ] {
        check(&format!(
            "let caught=false;try{{{source};}}catch(e){{caught=e instanceof SyntaxError;}}caught"
        ));
    }
}

#[test]
fn nested_eval_and_abrupt_completions_restore_caller_context() {
    check(
        r#"function f(){let x=7;eval('eval("var y=9;");');return x===7 && y===9 && delete y;}f() && typeof y==='undefined'"#,
    );
    check(
        r#"function f(){'use strict';let x=7;try{eval('throw 9;');}catch(e){if(e!==9)return false;}let caught=false;try{missing=1;}catch(e){caught=e instanceof ReferenceError;}return caught && x===7;}f()"#,
    );
    check(
        r#"function f(){let x=7;try{eval('"use strict";throw 9;');}catch(e){if(e!==9)return false;}eval('var y=3;');return x===7 && y===3 && delete y;}f()"#,
    );
    check(
        "function f(){let saved;try{eval('let x=7;saved=()=>x;throw 9;');}catch(e){if(e!==9)return false;}return saved;}f()()===7",
    );
    check(
        "function f(){let caught=false;try{eval('var effect=7;throw 9;');}catch(e){caught=e===9;}return caught && effect===7;}f()",
    );
}

#[test]
fn utf16_source_and_captured_environments_survive_collection() {
    let mut realm = Realm::default();
    realm.eval(r#"function make(){let text='\uD800',generated=eval("function f(){return '"+text+"';}f;");return generated;}let saved=make();delete make;let object={m(){return ()=>eval('new.target');}};let arrow=object.m();object=null;"#).unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("saved()"),
        Ok(Value::String(JsString::from_code_units(vec![0xd800])))
    );
    check(
        r#"let text='\uD800';let result=eval("function f(){return '"+text+"';}f;");result.toString()==="function f(){return '"+text+"';}""#,
    );
    assert_eq!(realm.eval("arrow()===undefined"), Ok(Value::Boolean(true)));
}

#[test]
fn direct_eval_shares_opt_in_limits_and_restores_state_after_host_failures() {
    check("eval('/*'+'a'.repeat(1050000)+'*/7;')===7");
    for (limits, source) in [
        (
            Limits {
                max_steps: Some(10000),
                ..Limits::default()
            },
            "try{eval('while(true){}');}catch{marker=1;}finally{marker=2;}",
        ),
        (
            Limits {
                max_source_bytes: Some(180),
                ..Limits::default()
            },
            "try{eval('/*'+'a'.repeat(200)+'*/7;');}catch{marker=1;}finally{marker=2;}",
        ),
        (
            Limits {
                max_heap_entries: Some(309),
                ..Limits::default()
            },
            "try{eval('7;');}catch{marker=1;}finally{marker=2;}",
        ),
    ] {
        let mut realm = Realm::new(limits);
        realm.eval("var marker=0;").unwrap();
        assert!(
            matches!(realm.eval(source), Err(Error::Limit { .. })),
            "{limits:?}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    }
    let mut realm = Realm::default();
    realm.eval("var marker=0,code='eval(code)';").unwrap();
    assert!(matches!(
        realm.eval("try{eval(code);}catch{marker=1;}finally{marker=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    assert_eq!(realm.eval("eval('7')"), Ok(Value::Number(7.0)));
}
