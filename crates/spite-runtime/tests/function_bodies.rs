//! Function-local instantiation, return completions, strictness, and captures.

mod common;
use common::REALM_ENTRIES;

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn number(source: &str, expected: f64) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Number(expected)),
        "{source}"
    );
}

#[test]
fn return_values_and_normal_body_completion_are_distinct() {
    for source in [
        "(()=>{})()",
        "(()=>{9;})()",
        "(()=>{9;return;})()",
        "(()=>{return\n9})()",
        "(()=>{return /*\n*/ 9})()",
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Ok(Value::Undefined),
            "{source}"
        );
    }
    number("(()=>{return 1,2;})()", 2.0);
    number("let x=0; let f=()=>{return 1; x=2;}; f()+x", 1.0);
    number("let x={value:9}; (()=>{return x;})().value", 9.0);
    number("(()=>{return (()=>{return 4;})();})()", 4.0);
    number("((a,b)=>{return a-b;}).apply(null,{0:9,1:2,length:2})", 7.0);
}

#[test]
fn returns_propagate_through_all_control_flow_and_finally_can_override_them() {
    for statement in [
        "{return 3;}",
        "if(true)return 3;",
        "while(true){return 3;}",
        "do{return 3;}while(true)",
        "for(;;){return 3;}",
        "outer:for(let i=0;i<9;i++){return 3;}",
        "switch(1){case 1:return 3; default:return 7;}",
        "try{return 3;}finally{9;}",
        "try{throw 1;}catch{return 3;}",
        "try{throw 1;}catch(e){return e+2;}finally{9;}",
        "try{return 1;}finally{return 3;}",
        "try{throw 1;}finally{return 3;}",
        "outer:{try{return 1;}finally{break outer;}}return 3;",
    ] {
        number(&format!("(()=>{{{statement} return 99;}})()"), 3.0);
    }
    number(
        "let x=0; let f=()=>{try{return 3;}finally{x=2;}};f()+x",
        5.0,
    );
    assert_eq!(
        Realm::default().eval("(()=>{try{return 3;}finally{throw 7;}})()"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    assert_eq!(
        Realm::default().eval("(()=>{try{return 3;}finally{return;}})()"),
        Ok(Value::Undefined)
    );
}

#[test]
fn function_vars_are_hoisted_local_and_share_parameter_bindings() {
    number("let x=9; let f=()=>{var x=2;return x;}; f()+x", 11.0);
    number("((x)=>{var x;return x;})(7)", 7.0);
    number("((x)=>{var x=3;return x;})(7)", 3.0);
    number("((x)=>{'use strict';var x;let y=2;return x+y;})(7)", 9.0);
    number(
        "let f=()=>{if(false){var x=2;}return x===undefined;}; f()?1:0",
        1.0,
    );
    number("let f=()=>{for(var x=0;x<3;x++){} return x;};f()", 3.0);
    number("let f=()=>{return x;var x=9;}; f()===undefined?1:0", 1.0);
    number("let f=()=>{var x=1;{let x=3;x++;}return x;};f()", 1.0);
    number(
        "let f=()=>{var x=1; try{throw 3;}catch(e){var x=4;}return x;};f()",
        4.0,
    );
    number(
        "let f=()=>{return typeof x; ()=>{var x;};}; f()==='undefined'?1:0",
        1.0,
    );
    number("let f=()=>{var arguments=4;return arguments;};f()", 4.0);
    assert!(matches!(
        Realm::default().eval("(()=>{return x;let x=1;})()"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
    assert!(matches!(
        Realm::default().eval("(()=>{const x=1;x=2;})()"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn closures_capture_function_vars_lexicals_and_independent_invocations() {
    number(
        "let make=x=>{var y=x;return ()=>{return ++y;};};let a=make(1);let b=make(8);a()+a()+b()",
        14.0,
    );
    number(
        "let make=()=>{let x=1;return ()=>{return ++x;};};let f=make();f()+f()",
        5.0,
    );
    number("let f=n=>{if(n<2)return 1;return n*f(n-1);};f(5)", 120.0);
    number(
        "let f=()=>{let a,b;for(let i=0;i<2;i++){if(i===0)a=()=>i;else b=()=>i;}return a()+b();};f()",
        1.0,
    );
    number(
        "let f=()=>{try{throw 7;}catch(e){return ()=>e;}};f()()",
        7.0,
    );
    let mut realm = Realm::default();
    realm.eval("let make=x=>{let y={value:x};return ()=>{return ++y.value;};};let f=make(1);make=null;").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("f()+f()"), Ok(Value::Number(5.0)));
    realm.eval("f=null").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
}

#[test]
fn body_strictness_is_captured_and_caller_state_is_restored_after_abrupt_exits() {
    number(
        "let f=()=>{'use strict';return ()=>{try{undeclared=1;}catch{return 3;}};};f()()",
        3.0,
    );
    number(
        "let f=()=>{'use strict';return 1;};f(); undeclared=4;undeclared",
        4.0,
    );
    number(
        "let f=()=>{'use strict';throw 1;};try{f();}catch{} undeclared=4;undeclared",
        4.0,
    );
    number(
        "let f=()=>{return delete 1;};let g=()=>{'use strict';try{f();undeclared=1;}catch{return 2;}};g()",
        2.0,
    );
    let mut realm = Realm::default();
    realm
        .eval("let f=()=>{'use strict';missing;};let flag=0")
        .unwrap();
    assert!(matches!(
        realm.eval("f()"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
    assert_eq!(realm.eval("undeclared=7"), Ok(Value::Number(7.0)));
    assert!(matches!(
        realm.eval("flag=1; eval=>{'use strict';}"),
        Err(Error::Parse(_))
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}

#[test]
fn source_and_name_metadata_include_block_bodies() {
    let mut realm = Realm::default();
    realm
        .eval("let f=(x /*a*/) => { /*b*/ return x; }")
        .unwrap();
    assert_eq!(
        realm.eval("f.toString()"),
        Ok(Value::String("(x /*a*/) => { /*b*/ return x; }".into()))
    );
    assert_eq!(realm.eval("f.name"), Ok(Value::String("f".into())));
    assert_eq!(realm.eval("f.length"), Ok(Value::Number(1.0)));
    assert_eq!(realm.eval("'prototype' in f"), Ok(Value::Boolean(false)));
}

#[test]
fn host_limits_cannot_be_caught_or_replaced_by_return_completions() {
    let mut realm = Realm::default();
    realm.eval("let flag=0;let f=()=>{try{return f();}catch{flag=1;return 9;}finally{flag=2;return 7;}};").unwrap();
    assert!(matches!(realm.eval("f()"), Err(Error::Limit { .. })));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(realm.eval("(()=>{return 5;})()"), Ok(Value::Number(5.0)));
    // A deep body and recursive calls share one nesting budget.
    let source = format!(
        "let g=()=>{{{}return g();{}}};g()",
        "{".repeat(20),
        "}".repeat(20)
    );
    assert!(matches!(realm.eval(&source), Err(Error::Limit { .. })));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(2000),
        ..Limits::default()
    });
    realm
        .eval("let flag=0;let f=()=>{try{while(true){}}finally{flag=1;return 9;}}")
        .unwrap();
    assert!(matches!(realm.eval("f()"), Err(Error::Limit { .. })));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
