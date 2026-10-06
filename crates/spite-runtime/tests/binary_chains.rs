//! Iterative binary evaluation preserves per-operation ordering (13.6–13.13, 13.16).

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn operators_keep_precedence_grouping_and_right_associated_exponentiation() {
    check(
        "1+2*3+4===11 && 2**3**2+4===516 && 2**3+4-1===11 && (1+2)*3+4===13 && (1+2<<1&7^2|1)===5 && (10-3-2)===5 && (8/2/2)===2",
    );
    check("let a={x:1},b={true:1};('x' in a in b)===true");
}

#[test]
fn right_operands_run_before_each_primitive_conversion_and_before_later_terms() {
    check(
        "let trace=[];function v(name,n){trace.push(name);return {valueOf(){trace.push(name+'v');return n;}}}let result=v('a',1)+v('b',2)+v('c',3);result===6 && trace.join(',')==='a,b,av,bv,c,cv'",
    );
    check(
        "let n=0,a={valueOf(){return n}};function b(){n=2;return 1}function c(){n=10;return 1}a+b()+c()===4 && n===10",
    );
    check(
        "let calls=0;function late(){calls++;return 0}let caught=false;try{1n+2+late()}catch(e){caught=e instanceof TypeError}caught && calls===0",
    );
}

#[test]
fn short_circuit_decisions_apply_to_each_accumulated_value() {
    check(
        "let calls=[];function v(n){calls.push(n);return n}let a=0&&v(1)&&v(2)||v(3);a===3 && calls.join(',')==='3' && (null??undefined??false)===false && ('x'||v(4)||v(5))==='x' && calls.join(',')==='3'",
    );
    check("let calls=0;function v(){calls++;return 4}(((0&&1)||2)||v())===2 && calls===0");
}

#[test]
fn comma_results_keep_unbound_receivers_and_indirect_eval() {
    check("'use strict';let o={m(){return this}};(0,0,o.m)()===undefined");
    check("function f(){let local=1;return (0,0,eval)('typeof local')}f()==='undefined'");
}

#[test]
fn primitive_string_and_bigint_steps_preserve_each_intermediate_result() {
    check(
        r"1+2+'3'+'4'==='334' && 'x'+1+2==='x12' && '\uD800'+''+'\uDC00'==='\uD800\uDC00' && 1n+2n+3n===6n",
    );
}

#[test]
fn large_flat_arithmetic_and_string_chains_use_ordinary_unlimited_defaults() {
    let mut realm = Realm::default();
    let source = format!("{}0", "0+".repeat(100_000));
    assert_eq!(realm.eval(&source), Ok(Value::Number(0.0)));
    let source = format!("{}'x'", "'x'+".repeat(1000));
    assert_eq!(
        realm.eval(&format!("({source}).length")),
        Ok(Value::Number(1001.0))
    );
}

#[test]
fn opted_in_abort_preserves_host_failure_and_skips_javascript_cleanup() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(20_000),
        ..Limits::default()
    });
    realm
        .eval("let flag=0;function mark(){flag=1;return 0}")
        .unwrap();
    let source = format!(
        "try{{mark(){};}}catch{{flag=2}}finally{{flag=3}}",
        "+0".repeat(40_000)
    );
    assert!(matches!(realm.eval(&source), Err(Error::Limit { .. })));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
