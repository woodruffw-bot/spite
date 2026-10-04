//! Parameter TDZ, ordered defaults, separate variable scopes, names, and length.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn number(source: &str, expected: f64) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Number(expected)),
        "{source}"
    );
}

#[test]
fn defaults_run_only_for_undefined_and_execute_left_to_right_after_arguments() {
    number("((x=7)=>x)()", 7.0);
    number("((x=7)=>x)(undefined)", 7.0);
    number("((x=7)=>x)(3)", 3.0);
    number(
        "let n=0;let f=(a=(n=n*10+1),b=(n=n*10+2))=>n;f(undefined,(n=3,undefined),(n=4,99));n",
        412.0,
    );
    number("((x=3,y=x+1,z=y+x)=>z)()", 7.0);
    number("let n=0; let f=(a=(n++,1),b=(n++,2))=>a+b;f(3,4);n", 0.0);
    number(
        "let n=0; let f=(a=(n++,1))=>a;f(null);f(false);f(0);f('');n",
        0.0,
    );
    number("let x=1;let f=(a=x++)=>a;f()+f()+x", 6.0);
    number("let f=(a={value:1})=>a;f()===f()?0:1", 1.0);
    number("((a=2,b)=>b===undefined?1:0)()", 1.0);
    number(
        "let f=(a=2,b=3)=>a+b;f.call(null,undefined,5)+f.apply(null,{1:4,length:2})+f.bind(null,undefined)(1)",
        16.0,
    );
}

#[test]
fn uninitialized_parameter_bindings_shadow_outer_names_including_typeof() {
    for source in [
        "let x=9; ((x=x)=>x)()",
        "let y=9; ((x=y,y=2)=>x)()",
        "((x=typeof y,y)=>x)()",
        "((x=(y=1),y)=>x)()",
        "((x=x=1)=>x)()",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Exception {
                    kind: ExceptionKind::ReferenceError,
                    ..
                })
            ),
            "{source}"
        );
    }
    number("((x=1,y=(x=3))=>x+y)()", 6.0);
    number("((x=()=>y,y=3)=>x())()", 3.0);
    number("let f=(x=()=>y,y=3)=>x;f()()", 3.0);
}

#[test]
fn defaults_cannot_see_body_declarations_and_closures_keep_the_parameter_environment() {
    number("let x=7;let f=(a=x)=>{var x=1;return a+x;};f()", 8.0);
    number("let x=7;let f=(a=()=>x)=>{let x=1;return a()+x;};f()", 8.0);
    number("let f=(x=1,g=()=>x)=>{var x=9;return g()+x;};f()", 10.0);
    number("let f=(x=1,g=()=>x)=>{x=9;return g()+x;};f()", 18.0);
    number("let f=(x=1,g=()=>x)=>{var x;return g()+x;};f()", 2.0);
    number("let f=(x=1,g=()=>++x)=>{var x;return g()+x;};f()", 3.0);
    number(
        "let f=(x=1,g=()=>x)=>{var x=9;return ()=>g()+x;};f()()",
        10.0,
    );
    number(
        "'use strict'; let f=(x=1,g=()=>x)=>{var x=9;return g()+x;};f()",
        10.0,
    );
    number("let f=(x=1)=>{var y=2;return x+y;};f()", 3.0);
    number(
        "let f=(a=typeof local)=>{var local=3;return a==='undefined'?1:0;};f()",
        1.0,
    );
    assert!(matches!(
        Realm::default().eval("let f=(a=local)=>{var local=3;return a;};f()"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
}

#[test]
fn function_names_and_lengths_follow_default_parameter_rules() {
    let mut realm = Realm::default();
    for (source, expected) in [
        ("(()=>0).length", 0.0),
        ("((a,b)=>0).length", 2.0),
        ("((a,b=1,c)=>0).length", 1.0),
        ("((a=1,b)=>0).length", 0.0),
        ("((a,b,c=1)=>0).bind(null,1).length", 1.0),
    ] {
        assert_eq!(realm.eval(source), Ok(Value::Number(expected)), "{source}");
    }
    assert_eq!(
        realm.eval("((callback=()=>1)=>callback.name)()"),
        Ok(Value::String("callback".into()))
    );
    assert_eq!(
        realm.eval("((callback=(()=>1))=>callback.name)()"),
        Ok(Value::String("callback".into()))
    );
    assert_eq!(
        realm.eval("let named=()=>1;((callback=()=>1)=>callback.name)(named)"),
        Ok(Value::String("named".into()))
    );
    assert_eq!(
        realm.eval("((x = 1 /* keep */) => x).toString()"),
        Ok(Value::String("(x = 1 /* keep */) => x".into()))
    );
}

#[test]
fn abrupt_defaults_skip_later_initializers_and_body_and_restore_caller_state() {
    let mut realm = Realm::default();
    realm
        .eval("let n=0;let fail=()=>{throw 7;};let f=(a=fail(),b=n++)=>{n=9;}")
        .unwrap();
    assert_eq!(realm.eval("f()"), Err(Error::Thrown(Value::Number(7.0))));
    assert_eq!(realm.eval("n"), Ok(Value::Number(0.0)));
    assert_eq!(realm.eval("undeclared=3"), Ok(Value::Number(3.0)));
    realm
        .eval("let strict;{ 'unused'; strict=(()=>{'use strict';return (a=missing)=>1;})(); }")
        .unwrap();
    assert!(matches!(
        realm.eval("strict()"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
    assert_eq!(realm.eval("another=4"), Ok(Value::Number(4.0)));
    realm
        .eval("let flag=0; let recursive=(x=recursive())=>1;")
        .unwrap();
    assert!(matches!(
        realm.eval("try{recursive();}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}

#[test]
fn escaped_defaults_trace_partial_initialization_and_body_var_copies() {
    let mut realm = Realm::default();
    realm
        .eval("let escaped;let f=(a=(escaped=()=>b),b=missing)=>0;try{f();}catch{} f=null;")
        .unwrap();
    realm.collect(10000).unwrap();
    assert!(matches!(
        realm.eval("escaped()"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
    realm.eval("escaped=null").unwrap();
    assert_eq!(realm.collect(10000).unwrap().live, 11);
    realm.eval("let make=(x={value:1},g=()=>x)=>{var x={value:9};return ()=>g().value+x.value;};escaped=make();make=null").unwrap();
    realm.collect(10000).unwrap();
    assert_eq!(realm.eval("escaped()"), Ok(Value::Number(10.0)));
    realm.eval("escaped=null").unwrap();
    assert_eq!(realm.collect(10000).unwrap().live, 11);
}
