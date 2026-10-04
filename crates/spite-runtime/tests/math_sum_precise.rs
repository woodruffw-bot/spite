//! Math.sumPrecise: exact arithmetic, element validation, and iterator closing.

mod common;
use common::REALM_ENTRIES;
use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn signed_zero_and_nonfinite_results_follow_full_iteration() {
    check(
        "Object.is(Math.sumPrecise([]),-0) && Object.is(Math.sumPrecise([-0,-0]),-0) && Object.is(Math.sumPrecise([-0,0]),0) && Object.is(Math.sumPrecise([1,-1,-0]),0) && Number.isNaN(Math.sumPrecise([NaN,Infinity])) && Number.isNaN(Math.sumPrecise([Infinity,-Infinity])) && Math.sumPrecise([1,Infinity,2])===Infinity && Math.sumPrecise([-Infinity,-Infinity])===-Infinity",
    );
    check(
        "let calls=0,items={[Symbol.iterator](){return {next(){calls++;if(calls===1)return {value:NaN};if(calls===2)return {value:Infinity};return {done:true};},return(){throw 8;}};}};Number.isNaN(Math.sumPrecise(items)) && calls===3",
    );
}

#[test]
fn exact_cancellation_and_halfway_rounding_do_not_round_intermediate_sums() {
    check(
        "Math.sumPrecise([1e308,1e308,0.1,0.1,1e30,0.1,-1e30,-1e308,-1e308])===0.30000000000000004 && Math.sumPrecise([1e30,0.1,-1e30])===0.1 && Math.sumPrecise([Number.MAX_VALUE,Number.MAX_VALUE,-Number.MAX_VALUE])===Number.MAX_VALUE && Math.sumPrecise([Number.MIN_VALUE,-Number.MIN_VALUE,Number.MIN_VALUE])===Number.MIN_VALUE",
    );
    check(
        "Math.sumPrecise([1,2**-53])===1 && Math.sumPrecise([1,2**-53,Number.MIN_VALUE])===1+Number.EPSILON && Math.sumPrecise([1+Number.EPSILON,2**-53])===1+2*Number.EPSILON && Math.sumPrecise([Number.MAX_VALUE,2**970,-Number.MIN_VALUE])===Number.MAX_VALUE && Math.sumPrecise([Number.MAX_VALUE,2**970])===Infinity",
    );
}

#[test]
fn element_types_are_validated_without_coercion_even_after_nonfinite_values() {
    for value in [
        "'3'",
        "true",
        "null",
        "undefined",
        "1n",
        "Symbol()",
        "Object(3)",
    ] {
        check(&format!(
            "let caught=false;try{{Math.sumPrecise([NaN,Infinity,{value}]);}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    check(
        "let calls=0,caught=false,o={[Symbol.toPrimitive](){calls++;throw 7;},valueOf(){calls++;throw 8;}};try{Math.sumPrecise([o]);}catch(e){caught=e instanceof TypeError;}caught && calls===0",
    );
    for input in [
        "undefined",
        "null",
        "7",
        "{}",
        "'3'",
        "{[Symbol.iterator]:7}",
    ] {
        check(&format!(
            "let caught=false;try{{Math.sumPrecise({input});}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
}

#[test]
fn iterator_method_and_next_are_captured_with_exact_receivers_and_live_steps() {
    check(
        "let log='',steps=0,iterator={get next(){log+='n';return function(){if(this!==iterator)throw 7;steps++;return steps<3?{value:steps}:{done:true,get value(){throw 8;}};};}},items={get [Symbol.iterator](){log+='i';return function(){if(this!==items)throw 9;log+='c';return iterator;};}};Math.sumPrecise.call(Symbol(),items,{valueOf(){throw 10;}})===3 && steps===3 && log==='icn'",
    );
    check(
        "let a=[1,2];a[Symbol.iterator]=function(){let i=0;return {next(){i++;return i===1?{value:7}:{done:true};}};};Math.sumPrecise(a)===7",
    );
}

#[test]
fn element_errors_close_but_step_errors_leave_the_iterator_open() {
    for cleanup in ["return {};", "return 7;", "throw 8;"] {
        check(&format!(
            "let closed=0,steps=0,caught=false,items={{[Symbol.iterator](){{return {{next(){{steps++;return {{value:{{}}}};}},return(){{closed++;{cleanup}}}}};}}}};try{{Math.sumPrecise(items);}}catch(e){{caught=e instanceof TypeError;}}caught && steps===1 && closed===1"
        ));
    }
    check(
        "let closed=0,caught=false,items={[Symbol.iterator](){return {next(){return {value:{}};},get return(){closed++;throw 8;}};}};try{Math.sumPrecise(items);}catch(e){caught=e instanceof TypeError;}caught && closed===1",
    );
    for result in [
        "throw 7;",
        "return {get done(){throw 7;}};",
        "return {done:false,get value(){throw 7;}};",
    ] {
        check(&format!(
            "let closed=0,caught=false,items={{[Symbol.iterator](){{return {{next(){{{result}}},return(){{closed++;return {{}};}}}};}}}};try{{Math.sumPrecise(items);}}catch(e){{caught=e===7;}}caught && closed===0"
        ));
    }
    check(
        "let closed=0,caught=false,items={[Symbol.iterator](){return {next(){return 7;},return(){closed++;return {};}};}};try{Math.sumPrecise(items);}catch(e){caught=e instanceof TypeError;}caught && closed===0",
    );
}

#[test]
fn host_failures_skip_cleanup_and_handlers_and_cannot_be_masked_during_close() {
    let mut realm = Realm::default();
    realm.eval("let flag=0,closed=0,items={[Symbol.iterator](){return {next(){Proxy;},return(){closed++;return {};}};}};").unwrap();
    assert!(matches!(
        realm.eval("try{Math.sumPrecise(items);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(
        realm.eval("flag===0 && closed===0"),
        Ok(Value::Boolean(true))
    );
    realm.eval("items={[Symbol.iterator](){return {next(){return {value:{}};},return(){closed++;Proxy;}};}};").unwrap();
    assert!(matches!(
        realm.eval("try{Math.sumPrecise(items);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(
        realm.eval("flag===0 && closed===1"),
        Ok(Value::Boolean(true))
    );
    let mut limited = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    limited.eval("let flag=0,closed=0,items={[Symbol.iterator](){return {next(){return {value:1};},return(){closed++;return {};}};}};").unwrap();
    assert!(matches!(
        limited.eval("try{Math.sumPrecise(items);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        limited.eval("flag===0 && closed===0"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn number_summation_does_not_use_the_host_bigint_magnitude_quota() {
    let mut realm = Realm::new(Limits {
        max_bigint_bits: Some(0),
        ..Limits::default()
    });
    assert_eq!(
        realm.eval("Math.sumPrecise([Number.MAX_VALUE,1,-Number.MAX_VALUE])"),
        Ok(Value::Number(1.0))
    );
}

#[test]
fn standard_descriptors_non_constructibility_and_intrinsic_roots_are_preserved() {
    check(
        "let d=Object.getOwnPropertyDescriptor(Math,'sumPrecise'),n=Object.getOwnPropertyDescriptor(d.value,'name'),l=Object.getOwnPropertyDescriptor(d.value,'length');d.writable && !d.enumerable && d.configurable && n.value==='sumPrecise' && !n.writable && !n.enumerable && n.configurable && l.value===1 && !l.writable && !l.enumerable && l.configurable && !Object.hasOwn(d.value,'prototype')",
    );
    check(
        "let caught=false;try{new Math.sumPrecise();}catch(e){caught=e instanceof TypeError;}caught",
    );
    let mut realm = Realm::default();
    realm
        .eval("let sum=Math.sumPrecise;delete Math.sumPrecise;delete globalThis.Math;")
        .unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(
        realm.eval("sum([1,2])===3 && typeof Math==='undefined'"),
        Ok(Value::Boolean(true))
    );
}
