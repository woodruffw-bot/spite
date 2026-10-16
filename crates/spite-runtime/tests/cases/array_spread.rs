//! Array literal spread and iterator protocol (13.2.4.1–2).

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn yielded_undefined_is_dense_while_literal_elisions_remain_holes() {
    check(
        "let a=[,...[,2],,...[],4,];a.length===5 && !Object.hasOwn(a,'0') && Object.hasOwn(a,'1') && a[1]===undefined && a[2]===2 && !Object.hasOwn(a,'3') && a[4]===4",
    );
    check("let a=[...[]];a.length===0 && Object.keys(a).length===0");
    check(
        "let a=[...'A\\uD834\\uDF06\\uD800B'];a.length===4 && a[1].length===2 && a[2]==='\\uD800' && a.join('')==='A\\uD834\\uDF06\\uD800B'",
    );
    check(
        "let source=Object.create({1:7});source.length=3;source[Symbol.iterator]=Array.prototype.values;let a=[...source];a.length===3 && a[0]===undefined && a[1]===7 && Object.keys(a).length===3",
    );
}

#[test]
fn spread_is_exhausted_before_the_next_source_element_evaluates() {
    check(
        "let log='',i=0;function item(){log+='e';return 0;}let source={get [Symbol.iterator](){log+='g';return function(){log+='i';return {next(){log+='n';return ++i===3?{done:true}:{get done(){log+='d';return false;},get value(){log+='v';return i;}};}};};}};let a=[item(),...source,item()];a.join(',')==='0,1,2,0' && log==='egindvndvne'",
    );
    check(
        "let source=[1,2],i=0;Object.defineProperty(source,'0',{get(){if(i++===0)source.push(3);return 1;}});let a=[...source];a.join(',')==='1,2,3'",
    );
    check(
        "let source=[1,2,3];Object.defineProperty(source,'0',{get(){source.length=1;return 1;}});let a=[...source];a.join(',')==='1'",
    );
}

#[test]
fn next_is_cached_and_receivers_are_preserved() {
    check(
        "let gets=0,count=0,iterator={get next(){gets++;return function(){if(this!==iterator || arguments.length!==0)throw 8;return {done:++count===3,value:count};};}},source={[Symbol.iterator](){if(this!==source || arguments.length!==0)throw 9;return iterator;}};let a=[...source];gets===1 && count===3 && a.join(',')==='1,2'",
    );
    check(
        "Object.defineProperty(Number.prototype,Symbol.iterator,{value:function(){'use strict';let done=false,value=this;return {next(){let prior=done;done=true;return {done:prior,value};}};}});let a=[...7];a.length===1 && a[0]===7",
    );
}

#[test]
fn noniterables_and_invalid_protocol_results_throw_type_errors() {
    for source in [
        "undefined",
        "null",
        "true",
        "7",
        "7n",
        "Symbol()",
        "{}",
        "{length:1,0:7}",
        "{[Symbol.iterator]:null}",
        "{[Symbol.iterator]:7}",
        "{[Symbol.iterator](){return 7;}}",
        "{[Symbol.iterator](){return {next:7};}}",
        "{[Symbol.iterator](){return {next(){return 7;}};}}",
    ] {
        check(&format!(
            "let caught=false;try{{[...({source})];}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
}

#[test]
fn abrupt_iterator_steps_do_not_close_or_evaluate_later_elements() {
    for body in [
        "throw 7;",
        "return 7;",
        "return {get done(){throw 7;}};",
        "return {get value(){throw 7;}};",
    ] {
        check(&format!(
            "let closed=0,later=0,source={{[Symbol.iterator](){{return {{next(){{{body}}},return(){{closed++;return {{}};}}}};}}}};let caught=false;try{{[...source,++later];}}catch(e){{caught=true;}}caught && closed===0 && later===0"
        ));
    }
    check(
        "let gets=0,a=[...{[Symbol.iterator](){return {next(){return {done:true,get value(){gets++;throw 7;}};}};}}];a.length===0 && gets===0",
    );
    check(
        "let caught=false;try{[...{get [Symbol.iterator](){throw 7;}}];}catch(e){caught=e===7;}caught",
    );
}

#[test]
fn output_uses_intrinsic_arrays_data_definitions_and_no_name_inference() {
    check(
        "let source=[1],gets=0,setter=0;Object.defineProperty(source,'constructor',{get(){gets++;throw 7;}});Object.defineProperty(Array.prototype,'0',{set(v){setter++;},configurable:true});let a=[...source,()=>{},function(){}];a[0]===1 && a[1].name==='' && a[2].name==='' && Object.getPrototypeOf(a)===Array.prototype && gets===0 && setter===0",
    );
    check(
        "let source=Object.freeze([1,2]);let a=[...source];a!==source && a.join(',')==='1,2' && Object.getOwnPropertyDescriptor(a,'0').writable && Object.getOwnPropertyDescriptor(a,'0').enumerable && Object.getOwnPropertyDescriptor(a,'0').configurable",
    );
}

#[test]
fn opted_in_work_exhaustion_is_a_host_failure_without_cleanup() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let closed=0,flag=0;let source={[Symbol.iterator](){return {next(){return {value:1};},return(){closed++;return {};}};}}").unwrap();
    assert!(matches!(
        realm.eval("try{[...source];}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("closed===0 && flag===0"),
        Ok(Value::Boolean(true))
    );
}
