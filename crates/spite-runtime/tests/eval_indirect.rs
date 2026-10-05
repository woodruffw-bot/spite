//! Global indirect PerformEval and its separate declaration algorithm (19.2.1.1–3).

mod common;

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn indirect_call_forms_use_globals_and_ignore_caller_strictness_and_receivers() {
    for call in [
        "indirect",
        "(0,eval)",
        "globalThis.eval",
        "eval.call.bind(eval,7)",
        "eval.bind(7)",
        "eval?.",
        "Reflect.apply.bind(null,eval,7)",
    ] {
        let argument = if call.starts_with("Reflect") {
            "['x+1']"
        } else {
            "'x+1'"
        };
        check(&format!(
            "let x=7,indirect=eval;function f(){{'use strict';let x=19;return {call}({argument});}}f()===8"
        ));
    }
    check(
        "function f(){'use strict';return eval.call(7,'this')===globalThis && eval.apply(7,['this'])===globalThis;}f()",
    );
    check(
        "let indirect=eval;function f(){let callerLocal=7;return indirect('typeof callerLocal')==='undefined' && indirect('typeof arguments')==='undefined';}f()",
    );
    check("let original=eval;eval=7;original('1+2')===3");
    check("let n=0;(0,eval)('1',n++)===1 && n===1");
    check("(0,eval)('#!comment\\n7;')===7");
}

#[test]
fn statement_completions_return_their_values_and_preserve_prior_effects() {
    for (body, value) in [
        ("", "undefined"),
        ("/*comment*/", "undefined"),
        (";", "undefined"),
        ("var x;", "undefined"),
        ("1;var x;", "1"),
        ("1;{2;}", "2"),
        ("1;if(false)2;", "undefined"),
        ("try{7;}finally{9;}", "7"),
        ("for(var i=0;i<2;i++){i;}", "1"),
    ] {
        check(&format!("(0,eval)('{body}')==={value}"));
    }
    check(
        "let error={},caught=false;try{(0,eval)('var effect=7;throw error;effect=9;');}catch(e){caught=e===error;}caught && effect===7",
    );
}

#[test]
fn new_global_declarations_are_configurable_and_existing_descriptors_are_preserved() {
    check(
        "(0,eval)('var x=7;function f(){return 1;}function f(){return 2;}');let xdesc=Object.getOwnPropertyDescriptor(globalThis,'x'),fdesc=Object.getOwnPropertyDescriptor(globalThis,'f');x===7 && f()===2 && xdesc.writable && xdesc.enumerable && xdesc.configurable && fdesc.writable && fdesc.enumerable && fdesc.configurable && delete x && delete f && typeof x==='undefined' && typeof f==='undefined'",
    );
    check(
        "var x=3;let original=Object;(0,eval)('var x=7;var Object;');x===7 && !Object.getOwnPropertyDescriptor(globalThis,'x').configurable && Object===original && !Object.getOwnPropertyDescriptor(globalThis,'Object').enumerable",
    );
    check(
        "Object.defineProperty(globalThis,'x',{value:3,writable:true,configurable:true});(0,eval)('var x=7;');x===7 && !Object.getOwnPropertyDescriptor(globalThis,'x').enumerable && Object.getOwnPropertyDescriptor(globalThis,'x').configurable",
    );
    check(
        "var f; (0,eval)('function f(){return 7;}');f()===7 && !Object.getOwnPropertyDescriptor(globalThis,'f').configurable",
    );
}

#[test]
fn lexical_declarations_are_fresh_and_escaped_closures_capture_them() {
    check(
        "let x=19;(0,eval)('let x=7;const y=2;function saved(){return x+y;}');(0,eval)('let x=33;');saved()===9 && x===19 && typeof y==='undefined' && !('x' in globalThis) && !('y' in globalThis)",
    );
    check(
        "(0,eval)('let NaN=7;const Infinity=9;globalThis.result=NaN+Infinity;');result===16 && Number.isNaN(NaN) && Infinity===1/0",
    );
    check(
        "let caught=false;try{(0,eval)('globalThis.marker=x;let x=7;');}catch(e){caught=e instanceof ReferenceError;}caught && !('marker' in globalThis)",
    );
    check(
        "let caught=false;try{(0,eval)('const local=1;local=2;');}catch(e){caught=e instanceof TypeError;}caught && typeof local==='undefined'",
    );
}

#[test]
fn strict_eval_vars_are_local_and_indirect_eval_does_not_inherit_strictness() {
    check(
        r#"let x=19;let pair=(0,eval)('"use strict";var x=7;function f(){return x;}[f,()=>x];');pair[0]()===7 && pair[1]()===7 && x===19 && !('f' in globalThis)"#,
    );
    check(
        r#"function f(){'use strict';(0,eval)('var indirectGlobal=7;');}f();indirectGlobal===7 && Object.getOwnPropertyDescriptor(globalThis,'indirectGlobal').configurable"#,
    );
    check(
        r#"let caught=false;try{(0,eval)('"use strict";undeclared=7;');}catch(e){caught=e instanceof ReferenceError;}caught && typeof undeclared==='undefined'"#,
    );
    check(r#"(0,eval)('"use\\x20strict";var escapedDirective=7;');escapedDirective===7"#);
}

#[test]
fn global_declaration_checks_finish_before_creating_any_bindings() {
    check(
        "let conflict=7,caught=false;try{(0,eval)('var fresh;var conflict;');}catch(e){caught=e instanceof SyntaxError;}caught && !('fresh' in globalThis)",
    );
    check(
        "let conflict=7,caught=false;try{(0,eval)('var fresh;function conflict(){}');}catch(e){caught=e instanceof SyntaxError;}caught && !('fresh' in globalThis)",
    );
    check(
        "let caught=false;try{(0,eval)('var fresh;function NaN(){}');}catch(e){caught=e instanceof TypeError;}caught && !('fresh' in globalThis)",
    );
    check(
        "Object.preventExtensions(globalThis);let caught=false;try{(0,eval)('var fresh;');}catch(e){caught=e instanceof TypeError;}caught && typeof fresh==='undefined' && (0,eval)('let x=7;x;')===7",
    );
    check(r#"Object.preventExtensions(globalThis);(0,eval)('"use strict";var x=7;x;')===7"#);
}

#[test]
fn global_vars_preserve_accessors_and_initializers_use_live_references() {
    check(
        "let log=[];Object.defineProperty(globalThis,'x',{get(){log.push('get');return 3;},set(v){log.push(v);},configurable:true});(0,eval)('var x;');log.length===0 && (0,eval)('var x=7;')===undefined && log.join(',')==='7' && typeof Object.getOwnPropertyDescriptor(globalThis,'x').get==='function'",
    );
    check(
        "let object={x:3};(0,eval)('with(object){var [x]=[7];}');object.x===7 && globalThis.x===undefined && Object.getOwnPropertyDescriptor(globalThis,'x').configurable",
    );
    check(
        "Object.defineProperty(globalThis,'f',{get(){throw 1;},configurable:true});(0,eval)('function f(){return 7;}');f()===7 && Object.getOwnPropertyDescriptor(globalThis,'f').configurable",
    );
}

#[test]
fn script_grammar_and_utf16_source_remain_exact() {
    for body in [
        "return 7;",
        "break;",
        "continue;",
        "new.target",
        "()=>new.target",
        "let x;let x;",
        "let x;var x;",
        "super.x",
        "super()",
    ] {
        check(&format!(
            "let caught=false;try{{(0,eval)('{body}');}}catch(e){{caught=e instanceof SyntaxError;}}caught"
        ));
    }
    check(
        r#"let text='\uD800\uDFFF\uFFFD';(0,eval)("/*"+text+"*/'"+text+"';")===text && (0,eval)('`\uD800\r\n\uDFFF`;')==='\uD800\n\uDFFF'"#,
    );
    check(
        r#"let text='\uD800',body="function saved(){return '"+text+"';}saved;",f=(0,eval)(body);f()===text && f.toString()==="function saved(){return '"+text+"';}""#,
    );
    check("(0,eval)('(function(){return new.target;})()')===undefined");
    check(
        "let flag=0,caught=false;try{(0,eval)('flag=7;return 1;');}catch(e){caught=e instanceof SyntaxError;}caught && flag===0",
    );
}

#[test]
fn caller_scope_and_strictness_restore_after_success_and_exceptions() {
    check(
        r#"function f(){'use strict';let x=7;try{(0,eval)('"use strict";throw 9;');}catch(e){if(e!==9)return false;}let caught=false;try{callerUndefined=1;}catch(e){caught=e instanceof ReferenceError;}return caught && x===7;}f()"#,
    );
    check(
        r#"let caught=false;try{(0,eval)('"use strict";throw 9;');}catch(e){caught=e===9;}callerUndefined=7;caught && callerUndefined===7"#,
    );
    check("let x=7;function f(){let x=9;(0,eval)('with({x:33}){x;}');return x;}f()===9 && x===7");
    check(
        "let caught=false;try{(0,eval)('let x=7;function saved(){return x;}throw 9;');}catch(e){caught=e===9;}caught && saved()===7",
    );
}

#[test]
fn eval_closures_and_intrinsic_identity_survive_collection() {
    let mut realm = Realm::default();
    realm.eval("(0,eval)('let value={x:7};function saved(){return value;}');let original=eval;delete globalThis.eval;").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("saved().x===7 && original('1+2')===3"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn unlimited_sources_opt_in_quotas_and_native_recursion_keep_host_boundaries() {
    check("(0,eval)('/*'+'a'.repeat(1050000)+'*/7;')===7");
    let mut no_extra_heap = Realm::new(Limits {
        max_heap_entries: Some(common::REALM_ENTRIES),
        ..Limits::default()
    });
    assert_eq!(
        no_extra_heap.eval("(0,eval)('/*comment*/')"),
        Ok(Value::Undefined)
    );
    for (limits, source) in [
        (
            Limits {
                max_steps: Some(10000),
                ..Limits::default()
            },
            "try{(0,eval)('while(true){}');}catch{marker=1;}finally{marker=2;}",
        ),
        (
            Limits {
                max_source_bytes: Some(180),
                ..Limits::default()
            },
            "try{(0,eval)('/*'+'a'.repeat(200)+'*/7;');}catch{marker=1;}finally{marker=2;}",
        ),
        (
            Limits {
                max_heap_entries: Some(common::REALM_ENTRIES),
                ..Limits::default()
            },
            "try{(0,eval)('7;');}catch{marker=1;}finally{marker=2;}",
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
    realm.eval("var marker=0,code='(0,eval)(code)';").unwrap();
    assert!(matches!(
        realm.eval("try{(0,eval)(code);}catch{marker=1;}finally{marker=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    assert_eq!(realm.eval("(0,eval)('7')"), Ok(Value::Number(7.0)));
}
