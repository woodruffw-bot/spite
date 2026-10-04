//! Non-strict calls with object receivers and mapped-argument aliasing.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn number(source: &str, expected: f64) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Number(expected)),
        "{source}"
    );
}

#[test]
fn mapped_indices_and_parameters_share_values_until_deletion() {
    number("function f(a){a=7;return arguments[0];}f.call({},1)", 7.0);
    number("function f(a){arguments[0]=7;return a;}f.call({},1)", 7.0);
    number("function f(a){++arguments[0];return a;}f.call({},1)", 2.0);
    number(
        "function f(a){delete arguments[0];a=7;return arguments[0]===undefined?1:0;}f.call({},1)",
        1.0,
    );
    number(
        "function f(a){delete arguments[0];arguments[0]=7;return a;}f.call({},1)",
        1.0,
    );
    number(
        "function f(a){arguments.length=0;a=7;return arguments[0];}f.call({},1)",
        7.0,
    );
    number("function f(a){arguments[1]=7;return a;}f.call({},1,2)", 1.0);
    number(
        "function f(a){var a=7;return arguments[0];}f.call({},1)",
        7.0,
    );
    number(
        "function f(a){return arguments.callee===f?1:0;}f.call({},1)",
        1.0,
    );
    number(
        "function f(){arguments.callee=7;return arguments.callee;}f.call({})",
        7.0,
    );
    number(
        "function f(){return delete arguments.callee?1:0;}f.call({})",
        1.0,
    );
    number(
        "function f(a){let o={__proto__:arguments};a=7;return o[0];}f.call({},1)",
        7.0,
    );
    number(
        "function f(a){let o={__proto__:arguments};o[0]=7;return a;}f.call({},1)",
        1.0,
    );
}

#[test]
fn duplicate_parameter_mapping_uses_only_the_last_occurrence_even_if_missing() {
    number(
        "function f(a,a){a=7;return arguments[0]+arguments[1];}f.call({},1,2)",
        8.0,
    );
    number(
        "function f(a,a){arguments[0]=7;return a;}f.call({},1,2)",
        2.0,
    );
    number(
        "function f(a,a){arguments[1]=7;return a;}f.call({},1,2)",
        7.0,
    );
    number(
        "function f(a,a){return a===undefined?1:0;}f.call({},1)",
        1.0,
    );
    number("function f(a,a){a=7;return arguments[0];}f.call({},1)", 1.0);
    number(
        "function f(a,a){arguments[0]=7;return a===undefined?1:0;}f.call({},1)",
        1.0,
    );
    number(
        "function f(a,b,a){arguments[0]=7;arguments[1]=8;return a===undefined?b:99;}f.call({},1,2)",
        8.0,
    );
    number(
        "function f(a,a){arguments[1]=7;return a===undefined?1:0;}f.call({})",
        1.0,
    );
}

#[test]
fn arguments_bindings_follow_parameter_and_body_declaration_rules() {
    number("function f(arguments){return arguments;}f.call({},7)", 7.0);
    number("function f(arguments=7){return arguments;}f.call({})", 7.0);
    number(
        "function f(){let arguments=7;return arguments;}f.call({})",
        7.0,
    );
    number(
        "function f(){var arguments;return arguments[0];}f.call({},7)",
        7.0,
    );
    number("function f(){arguments=7;return arguments;}f.call({})", 7.0);
    number(
        "function f(){var arguments=7;return arguments;}f.call({})",
        7.0,
    );
    number(
        "function f(){return arguments.length;function arguments(a,b){} }f.call({})",
        2.0,
    );
    number(
        "function f(a=arguments[1]){let arguments=3;return a+arguments;}f.call({},undefined,9)",
        12.0,
    );
    number(
        "function f(a=arguments[1]){function arguments(x){}return a+arguments.length;}f.call({},undefined,9)",
        10.0,
    );
    number("function f(a=1){a=7;return arguments[0];}f.call({},2)", 2.0);
    number("function f(a=1){arguments[0]=7;return a;}f.call({},2)", 2.0);
    assert!(matches!(
        Realm::default().eval("function f(){return arguments;let arguments;}f.call({})"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
    assert!(matches!(
        Realm::default().eval("function f(a=1){return arguments.callee;}f.call({})"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn object_receivers_sloppy_self_bindings_and_lexical_captures_work() {
    number(
        "function f(){return this.value;}let o={value:7,f};o.f()",
        7.0,
    );
    number(
        "function f(a,b){return this.value+a+b;}f.bind({value:1},2)(4)",
        7.0,
    );
    number(
        "function f(){return ()=>this.value;}f.call({value:7})()",
        7.0,
    );
    number(
        "let f=function local(){local=7;return typeof local==='function'?1:0;};f.call({})",
        1.0,
    );
    number(
        "let f=function local(){return (()=>{'use strict';try{local=7;}catch{return 1;}})();};f.call({})",
        1.0,
    );
    number(
        "function f(n){return n<2?1:n*f.call(this,n-1);}f.call({},5)",
        120.0,
    );
    number(
        "function f(){'use strict';function g(){return this;}return g.call({});}typeof f()==='object'?1:0",
        1.0,
    );
}

#[test]
fn escaped_arguments_keep_the_parameter_environment_alive_and_mutable() {
    let mut realm = Realm::default();
    realm.eval("let saved;function f(a){saved={args:arguments,get:()=>a,set:x=>a=x};}f.call({},1);f=null;").unwrap();
    realm.collect(10000).unwrap();
    assert_eq!(
        realm.eval("saved.set(7);saved.args[0]"),
        Ok(Value::Number(7.0))
    );
    assert_eq!(
        realm.eval("saved.args[0]=9;saved.get()"),
        Ok(Value::Number(9.0))
    );
    assert_eq!(
        realm.eval("delete saved.args[0];saved.set(3);saved.args[0]"),
        Ok(Value::Undefined)
    );
    realm.eval("saved=null").unwrap();
    assert_eq!(realm.collect(10000).unwrap().live, 11);
}

#[test]
fn removing_the_last_alias_releases_the_invocation_environment() {
    let mut realm = Realm::default();
    realm
        .eval("let args;function f(a){args=arguments;}f.call({payload:{}},1);f=null;")
        .unwrap();
    assert_eq!(realm.collect(10000).unwrap().live, 17);
    realm.eval("delete args[0]").unwrap();
    assert_eq!(realm.collect(10000).unwrap().live, 14);
    realm.eval("args=null").unwrap();
    assert_eq!(realm.collect(10000).unwrap().live, 11);
}
