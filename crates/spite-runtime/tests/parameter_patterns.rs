//! Formal binding initialization and function environment semantics (10.2.11, 15.1).

mod common;
use common::REALM_ENTRIES;
use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn functions_arrows_methods_setters_and_constructors_share_pattern_initialization() {
    check(
        "function f({a:[x=2]=[],...rest}={},[y=3]=[]){return x+y+Object.keys(rest).length;}f()===5 && f({a:[7],z:1},[8])===16",
    );
    check(
        "let f=({x=2},[y=3])=>x+y;f({},[])===5 && f.call(null,{x:7},[8])===15 && f.apply(null,[{},[]])===5 && f.bind(null,{x:1})([])===4",
    );
    check(
        "let object={m({x}){return this.y+x;},y:2,set value([x=3]){this.y=x;}};let a=object.m({x:1});object.value=[];a===3 && object.y===3",
    );
    check(
        "function C({x},[y=2]){this.value=x+y;this.target=new.target;}let c=new C({x:3},[]);c.value===5 && c.target===C",
    );
    check(
        "let x=8;let f=Function('{x=2}', '...[y=3]', 'return x+y;');f({})===5 && f({x:7},8)===15 && f.toString().includes('{x=2}')",
    );
    check(
        "let log=[];function f({[log.push('key')&&'a']:x=(log.push('default'),2)},[y]){log.push('body');return x+y;}let result=f((log.push('arg1'),{}),(log.push('arg2'),[3]));result===5 && log.join(',')==='arg1,arg2,key,default,body'",
    );
}

#[test]
fn rest_patterns_receive_dense_intrinsic_arrays_and_may_contain_defaults() {
    check("function f(...[x=2,...ys]){return x+ys.length;}f()===2 && f(3,4,5)===5");
    check("let f=(...[,,...{length:n}])=>n;f(1,2,3,4)===2 && f()===0");
    check("function f(...{length:n,0:first=7}){return n+first;}f()===7 && f(3,4)===5");
    check(
        "let original=Array;Array=function(){throw 1;};function f(...[x,...xs]){return xs;}let xs=f({},2,3);Array=original;xs.length===2 && xs[0]===2 && Object.getPrototypeOf(xs)===original.prototype",
    );
    check("let saved;function f(...[x=(saved=()=>x,2)]){var x=7;return saved()+x;}f()===9");
    check(
        "let original=Array.prototype[Symbol.iterator],calls=0;Array.prototype[Symbol.iterator]=function(){calls++;return{next(){return{done:false,value:7};},return(){return{};}};};function f(...[x]){return x;}let result=f(1,2);Array.prototype[Symbol.iterator]=original;result===7 && calls===1",
    );
}

#[test]
fn nested_names_begin_in_tdz_and_defaults_name_only_identifier_targets() {
    for source in [
        "function f([x=x]){}f([])",
        "let y=7;function f({x=y},[y]){}f({},[2])",
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
    check(
        "let result;function f({[typeof y]:x},[y]){}try{f({},[]);}catch(e){result=e instanceof ReferenceError;}result",
    );
    check("function f([x=2,y=x+1],{z=y}){return x+y+z;}f([],{})===8");
    check("function f({a=()=>1,b=function(){}}){return a.name==='a' && b.name==='b';}f({})");
    check("function f({x} = function(){}){return arguments[0]===undefined && x===undefined;}f()");
}

#[test]
fn computed_keys_and_nested_defaults_separate_parameter_closures_from_body_vars() {
    for prefix in ["", "'use strict';"] {
        check(&format!(
            "{prefix}let saved;function f({{[(saved=()=>x,'a')]:x}}){{var x=9;return saved()+x;}}f({{a:2}})===11"
        ));
        check(&format!(
            "{prefix}function f([x=2,g=()=>x]){{var x=9;return g()+x;}}f([])===11"
        ));
        check(&format!(
            "{prefix}let saved;let f=({{[(saved=()=>x,'a')]:x}})=>{{var x=9;return saved()+x;}};f({{a:2}})===11"
        ));
    }
    check("let saved;function f([x]){var x=9;return()=>x;}saved=f([2]);saved()===9");
    check("let x=7;function f([y=x]){var x=2;return x+y;}f([])===9");
    check(
        "let saved;function f({[(saved=()=>x,'a')]:x}){function x(){return 3;}return saved()===2 && x()===3;}f({a:2})",
    );
}

#[test]
fn patterns_select_unmapped_arguments_and_all_names_control_arguments_binding() {
    check(
        "function f([x],y){y=9;arguments[1]=7;return x===1 && y===9 && arguments[1]===7;}f([1],2)",
    );
    check("function f({arguments}){return arguments;}f({arguments:7})===7");
    check("function f([x]){function arguments(){return 7;}return arguments();}f([1])===7");
    check(
        "function f({[(arguments[0].a=3,'a')]:x}){let arguments=7;return x+arguments;}f({a:1})===10",
    );
    check("let arguments=8;let f=([x=arguments])=>x;f([])===8");
    check("function f(x,x){return x===undefined && arguments[0]===1;}f(1)");
}

#[test]
fn length_counts_patterns_until_a_top_level_default_or_rest_and_source_is_exact() {
    check("function f({},[x=1],{[key]:y},z=1){}f.length===3 && f.bind(null,{}).length===2");
    check("(({},[x=1],{[key]:y},...rest)=>0).length===3");
    check("(({}={},[x])=>0).length===0 && (function(...{}){}).length===0");
    check("let f=({ a: [x = 1] } /*keep*/) => x;f.toString()==='({ a: [x = 1] } /*keep*/) => x'");
    check(
        "let object={set x({y=1}){}},setter=Object.getOwnPropertyDescriptor(object,'x').set;setter.length===1",
    );
}

#[test]
fn iterator_closing_preserves_errors_and_skips_later_parameters_and_body() {
    check(
        "let log=[],source={next(){log.push('step');return{done:false,value:undefined};},return(){log.push('close');throw 8;},[Symbol.iterator](){return this;}};function fail(){throw 7;}function f([x=fail()],y=(log.push('later'),2)){log.push('body');}let result;try{f(source);}catch(e){result=e===7;}result && log.join(',')==='step,close'",
    );
    check(
        "let log=[],source={next(){log.push('step');return{done:false,value:2};},return(){log.push('close');return{};},[Symbol.iterator](){return this;}};function f([x],y=(log.push('later'),3)){log.push('body');return x+y;}f(source)===5 && log.join(',')==='step,close,later,body'",
    );
    check(
        "let closed=0,source={next(){throw 7;},return(){closed++;return{};},[Symbol.iterator](){return this;}};function f([x]){}let result;try{f(source);}catch(e){result=e===7;}result && closed===0",
    );
}

#[test]
fn escaped_partial_parameters_survive_collection_and_failed_calls_restore_state() {
    let mut realm = Realm::default();
    realm
        .eval("let saved;function f([x=(saved=()=>y,1),y=missing]){}try{f([]);}catch{}f=null;")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert!(matches!(
        realm.eval("saved()"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
    realm.eval("saved=null").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    realm.eval("function make({[(saved=()=>x,'a')]:x}){var x={value:9};return()=>saved().value+x.value;}let escaped=make({a:{value:2}});make=null;").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("escaped()"), Ok(Value::Number(11.0)));
    realm.eval("escaped=null;saved=null").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(realm.eval("undeclared=3"), Ok(Value::Number(3.0)));
}

#[test]
fn flat_parameter_patterns_have_no_default_work_or_heap_quota() {
    let names = (0..12000)
        .map(|i| format!("p{i}"))
        .collect::<Vec<_>>()
        .join(",");
    check(&format!(
        "function f({{{names}}}){{return p11999===undefined;}}f({{}})"
    ));
}
