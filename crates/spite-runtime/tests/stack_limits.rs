//! Recursive execution must hit host limits before exhausting a modest native stack.

use spite_runtime::{Error, Realm, Value};

#[test]
fn recursion_limits_work_on_a_two_mebibyte_thread_stack() {
    for (index, (setup, call)) in [
        ("let f=(x=f())=>1;", "f()"),
        ("function f(x=f()){return 1;}", "f()"),
        ("function f(){'use strict';return f();}", "f()"),
        ("function F(){new F;}", "new F"),
        ("let o={valueOf:()=>+o};", "+o"),
        ("let o={valueOf:()=>Number(o)};", "Number(o)"),
        ("let o={valueOf:()=>isFinite(o)};", "isFinite(o)"),
        ("let o={valueOf:()=>isNaN(o)};", "isNaN(o)"),
        ("let o={toString:()=>Error(o)};", "Error(o)"),
        ("let o={toString:()=>String(o)};", "String(o)"),
        (
            "let d={};Object.defineProperty(d,'value',{get:()=>Object.defineProperty({},'x',d)});",
            "Object.defineProperty({},'x',d)",
        ),
        (
            "let d={};Object.defineProperty(d,'x',{enumerable:true,get:()=>Object.defineProperties({},d)});",
            "Object.defineProperties({},d)",
        ),
        (
            "let o={};Object.defineProperty(o,'x',{enumerable:true,get:()=>Object.assign({},o)});",
            "Object.assign({},o)",
        ),
        (
            "let o={name:{toString:()=>Error.prototype.toString.call(o)}};",
            "Error.prototype.toString.call(o)",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        std::thread::Builder::new()
            .name(format!("bounded-recursion-{index}"))
            .stack_size(2 * 1024 * 1024)
            .spawn(move || {
                let mut realm = Realm::default();
                realm.eval(&format!("let flag=0;{setup}")).unwrap();
                assert!(
                    matches!(
                        realm.eval(&format!("try{{{call};}}catch{{flag=1;}}finally{{flag=2;}}")),
                        Err(Error::Limit { .. })
                    ),
                    "{setup}"
                );
                assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
                assert_eq!(realm.eval("after=3"), Ok(Value::Number(3.0)));
                assert_eq!(realm.eval("Number('4')"), Ok(Value::Number(4.0)));
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
