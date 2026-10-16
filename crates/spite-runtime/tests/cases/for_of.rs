//! Synchronous for-of bindings, iteration, and abrupt completions (14.7.5).

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn arrays_strings_holes_and_live_lengths_are_iterated() {
    check(
        "let values=[];for(let x of [,2])values.push(x);values.length===2 && values[0]===undefined && values[1]===2",
    );
    check(
        "let values=[];for(const x of 'A\\uD834\\uDF06\\uD800')values.push(x);values.length===3 && values[1]==='\\uD834\\uDF06' && values[2]==='\\uD800'",
    );
    check("let a=[1,2],v=[];for(let x of a){v.push(x);if(x===1)a.push(3);}v.join(',')==='1,2,3'");
    check(
        "let caught=0;try{for(let x of null){};}catch(e){if(e instanceof TypeError)caught++;}try{for(let x of {}){};}catch(e){if(e instanceof TypeError)caught++;}caught===2",
    );
}

#[test]
fn next_is_cached_and_its_receiver_and_result_access_order_are_preserved() {
    check(
        "let log='',calls=0,it={get next(){log+='n';return function(){if(this!==it || arguments.length!==0)throw 7;log+='c';calls++;return {get done(){log+='d';return calls>1;},get value(){log+='v';return 8;}};}},return(){throw 9;}},source={get [Symbol.iterator](){log+='m';return function(){if(this!==source || arguments.length!==0)throw 6;log+='i';return it;}}};let value;for(value of source){log+='b';it.next=()=>{throw 5;};}log==='mincdvbcd' && value===8",
    );
}

#[test]
fn lhs_references_are_evaluated_after_each_value_and_assignment_errors_close() {
    check(
        "let log='',n=0,obj={},it={next(){log+='n';return {done:n++===2,get value(){log+='v';return n;}}}},source={[Symbol.iterator](){return it;}};function key(){log+='k';return n;}for(obj[key()] of source)log+='b';log==='nvkbnvkbn' && obj[1]===1 && obj[2]===2",
    );
    check(
        "let closes=0,source={[Symbol.iterator](){return {next(){return {value:7};},return(){closes++;return {};}}}};const x=1;let caught=false;try{for(x of source){throw 9;}}catch(e){caught=e instanceof TypeError;}caught && closes===1 && x===1",
    );
    check(
        "let marker={},closes=0,obj={set x(v){throw marker;}},source={[Symbol.iterator](){return {next(){return {value:7};},return(){closes++;throw 8;}}}};let caught=false;try{for(obj.x of source){};}catch(e){caught=e===marker;}caught && closes===1",
    );
}

#[test]
fn var_is_hoisted_and_uses_the_surrounding_binding() {
    check("let before=x;for(var x of [1,2]){};before===undefined && x===2 && globalThis.x===2");
    check(
        "function f(){let before=x;for(var x of [1,2]){};return before===undefined && x===2;}f() && !Object.hasOwn(globalThis,'x')",
    );
    check(
        "let closures=[];for(var x of [1,2])closures.push(()=>x);closures[0]()===2 && closures[1]()===2",
    );
}

#[test]
fn lexical_bindings_have_rhs_tdz_and_fresh_iteration_environments() {
    check(
        "let x=[1],caught=false;try{for(let x of x){};}catch(e){caught=e instanceof ReferenceError;}caught && x[0]===1",
    );
    check(
        "let x=7,closures=[];for(let x of [1,2]){closures.push(()=>x);x+=10;}x===7 && closures[0]()===11 && closures[1]()===12",
    );
    check(
        "let closures=[];for(const x of [1,2])closures.push(()=>x);closures[0]()===1 && closures[1]()===2 && typeof x==='undefined'",
    );
    check(
        "let rhs,inside;for(let x of (rhs=()=>x,[1]))inside=()=>x;let caught=false;try{rhs();}catch(e){caught=e instanceof ReferenceError;}caught && inside()===1",
    );
    check(
        "let closes=0,caught=false,source={[Symbol.iterator](){return {next(){return {value:1};},return(){closes++;return {};}}}};try{for(const x of source)x=2;}catch(e){caught=e instanceof TypeError;}caught && closes===1 && typeof x==='undefined'",
    );
}

#[test]
fn closures_from_iteration_bindings_survive_explicit_collection() {
    let mut realm = Realm::default();
    realm
        .eval("let closures=[];for(const x of [1,2])closures.push(()=>x)")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("closures[0]()+closures[1]()"),
        Ok(Value::Number(3.0))
    );
}

#[test]
fn break_return_and_outer_continue_close_but_matching_continue_does_not() {
    check(
        "let closes=0,nexts=0,source={[Symbol.iterator](){return {next(){nexts++;return {value:nexts};},return(){closes++;return {};}}}};for(let x of source){if(x===1)continue;break;}closes===1 && nexts===2",
    );
    check(
        "let closes=0,n=0,source={[Symbol.iterator](){return {next(){return {value:++n,done:n>2};},return(){closes++;return {};}}}};a:b:for(let x of source)continue a;closes===0 && n===3",
    );
    check(
        "let log='',source={[Symbol.iterator](){return {next(){return {value:7};},get return(){log+='g';return function(){if(arguments.length!==0)throw 9;log+='r';return {};};}}}};function f(){for(let x of source){try{return x;}finally{log+='f';}}}f()===7 && log==='fgr'",
    );
    check(
        "let closes=0,source={[Symbol.iterator](){return {next(){return {value:1};},return(){closes++;return {};}}}};outer:for(let i=0;i<2;i++){for(let x of source)continue outer;}closes===2",
    );
    check(
        "let log='',source={[Symbol.iterator](){return {next(){return {value:1};},return(){log+='r';return {};}}}};outer:for(let x of source){for(let y of source)break outer;}log==='rr'",
    );
}

#[test]
fn normal_close_validates_return_and_can_replace_control_transfers() {
    check(
        "let caught=0;for(let method of [1,()=>1]){let source={[Symbol.iterator](){return {next(){return {value:7};},return:method};}};try{for(let x of source)break;}catch(e){if(e instanceof TypeError)caught++;}}caught===2",
    );
    check(
        "let marker={},source={[Symbol.iterator](){return {next(){return {value:7};},get return(){throw marker;}}}},caught=false;try{for(let x of source)break;}catch(e){caught=e===marker;}caught",
    );
    check(
        "let marker={},source={[Symbol.iterator](){return {next(){return {value:7};},return(){throw marker;}}}},caught=false;function f(){for(let x of source)return 7;}try{f();}catch(e){caught=e===marker;}caught",
    );
    check(
        "let source={[Symbol.iterator](){return {next(){return {value:7};},return:null};}},x;for(x of source)break;x===7",
    );
    check(
        "let source={[Symbol.iterator](){return {next(){return {value:7};},return(){return Object.freeze({get done(){throw 1;},get value(){throw 2;}});}}}};for(let x of source)break;true",
    );
}

#[test]
fn incoming_throws_override_cleanup_language_errors_and_non_object_results() {
    check(
        "let marker={},closes=0,caught=0;for(let method of [()=>{closes++;throw 8;},()=>{closes++;return 1;},1,null]){let source={[Symbol.iterator](){return {next(){return {value:7};},return:method};}};try{for(let x of source)throw marker;}catch(e){if(e===marker)caught++;}}caught===4 && closes===2",
    );
    check(
        "let marker={},gets=0,source={[Symbol.iterator](){return {next(){return {value:7};},get return(){gets++;throw 8;}}}},caught=false;try{for(let x of source)throw marker;}catch(e){caught=e===marker;}caught && gets===1",
    );
}

#[test]
fn iterator_step_failures_never_close_and_exhaustion_skips_value() {
    check(
        "let marker={},closes=0,caught=0;for(let next of [()=>{throw marker;},()=>({get done(){throw marker;}}),()=>({get value(){throw marker;}})]){let source={[Symbol.iterator](){return {next,return(){closes++;return {};}}}};try{for(let x of source){};}catch(e){if(e===marker)caught++;}}closes===0 && caught===3",
    );
    check(
        "let closes=0,caught=0;for(let next of [1,()=>1]){let source={[Symbol.iterator](){return {next,return(){closes++;return {};}}}};try{for(let x of source){};}catch(e){if(e instanceof TypeError)caught++;}}closes===0 && caught===2",
    );
    check(
        "let source={[Symbol.iterator](){return {next(){return {done:1,get value(){throw 7;}}},return(){throw 8;}}}};for(let x of source)throw 9;true",
    );
}

#[test]
fn loop_completion_values_and_scope_restoration_follow_abrupt_control() {
    let cases = [
        ("for(let x of []) {7;}", Value::Undefined),
        ("for(let x of [1,2]) {x;}", Value::Number(2.0)),
        ("for(let x of [1,2]) {x;break;}", Value::Number(1.0)),
        ("for(let x of [1,2]) {x;continue;}", Value::Number(2.0)),
        ("a:for(let x of [1]) {7;break a;}", Value::Number(7.0)),
    ];
    for (source, expected) in cases {
        assert_eq!(Realm::default().eval(source), Ok(expected), "{source}");
    }
    check("let x=7;try{for(let x of [1])throw 8;}catch{}x===7");
}

#[test]
fn host_failures_stop_without_cleanup_or_language_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0,closes=0,source={[Symbol.iterator](){return {next(){return {value:1};},return(){closes++;return {};}}}}").unwrap();
    assert!(matches!(
        realm.eval("try{for(let x of source){};}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag+closes"), Ok(Value::Number(0.0)));
    let mut realm = Realm::default();
    realm.eval("let flag=0,closes=0,source={[Symbol.iterator](){return {next(){return {value:1};},return(){closes++;return {};}}}}").unwrap();
    assert!(matches!(
        realm.eval("try{for(let x of source)Proxy;}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag+closes"), Ok(Value::Number(0.0)));
    realm
        .eval("source[Symbol.iterator]=()=>({next(){return {value:1};},return(){Proxy;}})")
        .unwrap();
    assert!(matches!(
        realm.eval("try{for(let x of source)throw 7;}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
