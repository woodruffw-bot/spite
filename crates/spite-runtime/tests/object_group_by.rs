//! Object.groupBy: ordered callbacks, property keys, grouping, and cleanup.

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
fn groups_preserve_values_and_first_key_order_in_a_null_prototype_result() {
    check(
        "let a={x:1},b={x:2},source=[a,1,b,2,3],o=Object.groupBy(source,(v,i)=>i%2?'even':'odd');Object.getPrototypeOf(o)===null && Object.keys(o).join(',')==='odd,even' && o.odd.length===3 && o.odd[0]===a && o.odd[1]===b && o.odd[2]===3 && o.even.join(',')==='1,2' && Object.hasOwn(o.odd,'0') && Object.hasOwn(o.odd,'2')",
    );
    check(
        "let a=Object.groupBy([],()=>{throw 7;}),b=Object.groupBy([],()=>{throw 8;});a!==b && Object.getPrototypeOf(a)===null && Object.keys(a).length===0 && a.hasOwnProperty===undefined",
    );
    check(
        "let o=Object.groupBy([,1,,],v=>v===undefined?'hole':'value');o.hole.length===2 && Object.hasOwn(o.hole,'0') && Object.hasOwn(o.hole,'1') && o.hole[0]===undefined && o.hole[1]===undefined && o.value[0]===1",
    );
}

#[test]
fn property_key_conversion_preserves_symbol_identity_and_numeric_order() {
    check(
        "let a=Symbol('x'),b=Symbol('x'),source=[a,b,a,'x'],o=Object.groupBy(source,v=>v);Object.keys(o).join(',')==='x' && Object.getOwnPropertySymbols(o)[0]===a && Object.getOwnPropertySymbols(o)[1]===b && o[a].length===2 && o[b].length===1 && o.x[0]==='x'",
    );
    check(
        "let source=[10,2,10,-0,0,1n,'1',null,undefined,true,NaN],o=Object.groupBy(source,v=>v);Object.keys(o).join(',')==='0,1,2,10,null,undefined,true,NaN' && Object.is(o['0'][0],-0) && Object.is(o['0'][1],0) && o['1'][0]===1n && o['1'][1]==='1' && o['10'].join(',')==='10,10'",
    );
    check(
        "let log='',s=Symbol(),o=Object.groupBy([1,2],v=>({[Symbol.toPrimitive](hint){log+=hint;return s;}}));log==='stringstring' && o[s].join(',')==='1,2' && Object.getOwnPropertySymbols(o).length===1",
    );
    check(
        "let o=Object.groupBy([1,2,3],(v,i)=>i===0?'__proto__':i===1?'constructor':'\\uD800');Object.getPrototypeOf(o)===null && o.__proto__[0]===1 && o.constructor[0]===2 && o['\\uD800'][0]===3",
    );
}

#[test]
fn callbacks_receive_two_arguments_and_use_ordinary_receiver_rules() {
    check(
        "let source=[7,8],calls=0,o=Object.groupBy(source,function(v,i){'use strict';if(this!==undefined || arguments.length!==2 || v!==source[i])throw 7;calls++;return i;});calls===2 && o['0'][0]===7 && o['1'][0]===8",
    );
    check(
        "let source=[7],calls=0;Object.groupBy(source,function(v,i){if(this!==globalThis || arguments.length!==2)throw 7;calls++;return 'x';});calls===1",
    );
    check(
        "let context={x:'group'};function f(v,i){'use strict';if(this!==context || arguments.length!==2)throw 7;return this.x;}Object.groupBy([1,2],f.bind(context)).group.join(',')==='1,2'",
    );
}

#[test]
fn iteration_is_live_and_preserves_cached_next_and_conversion_order() {
    check(
        "let log='',n=0,i={get next(){log+='g';return function(){if(this!==i || arguments.length!==0)throw 7;log+='n';Object.defineProperty(i,'next',{value:7});return n++===0?{get done(){log+='d';return false;},get value(){log+='v';return 3;}}:{get done(){log+='d';return true;},get value(){throw 8;}};};}},source={[Symbol.iterator](){if(this!==source || arguments.length!==0)throw 9;log+='i';return i;}};let o=Object.groupBy(source,(v,k)=>{if(v!==3 || k!==0)throw 10;log+='c';return {[Symbol.toPrimitive](hint){if(hint!=='string')throw 11;log+='k';return 'x';}};});o.x[0]===3 && log==='igndvcknd'",
    );
    check(
        "let source=[1,2],calls=0,o=Object.groupBy(source,(v,i)=>{calls++;if(i===0){source[1]=7;source.push(3);}return 'x';});calls===3 && o.x.join(',')==='1,7,3'",
    );
    check(
        "let source=[1,2,3],o=Object.groupBy(source,(v,i)=>{source.length=1;return 'x';});o.x.length===1 && o.x[0]===1",
    );
    check(
        "let o=Object.groupBy('A\\uD834\\uDF06\\uD800A',v=>v);o.A.join('')==='AA' && o['\\uD834\\uDF06'].length===1 && o['\\uD800'].length===1",
    );
}

#[test]
fn validation_precedes_iterator_lookup_and_acquisition_failures_do_not_close() {
    for callback in ["undefined", "null", "7", "{}", "'x'", "Symbol()", "1n"] {
        check(&format!(
            "let reads=0,caught=false,source={{get [Symbol.iterator](){{reads++;throw 7;}}}};try{{Object.groupBy(source,{callback});}}catch(e){{caught=e instanceof TypeError;}}caught && reads===0"
        ));
    }
    for source in [
        "null",
        "undefined",
        "7",
        "true",
        "1n",
        "Symbol()",
        "{}",
        "{[Symbol.iterator]:null}",
        "{[Symbol.iterator]:7}",
    ] {
        check(&format!(
            "let calls=0,caught=false;try{{Object.groupBy({source},()=>{{calls++;return 'x';}});}}catch(e){{caught=e instanceof TypeError;}}caught && calls===0"
        ));
    }
    check(
        "let closed=0,caught=false,i={get next(){throw 7;},return(){closed++;}};try{Object.groupBy({[Symbol.iterator](){return i;}},()=> 'x');}catch(e){caught=e===7;}caught && closed===0",
    );
}

#[test]
fn callback_and_conversion_failures_close_once_with_original_throw_precedence() {
    for callback in [
        "()=>{throw original;}",
        "()=>({[Symbol.toPrimitive](){throw original;}})",
    ] {
        for close in ["return {};", "return 7;", "return undefined;", "throw 8;"] {
            check(&format!(
                "let original={{}},closed=0,steps=0,i={{next(){{steps++;return {{value:7}};}},return(){{if(this!==i || arguments.length!==0)throw 9;closed++;{close}}}}},caught=false;try{{Object.groupBy({{[Symbol.iterator](){{return i;}}}},{callback});}}catch(e){{caught=e===original;}}caught && closed===1 && steps===1"
            ));
        }
    }
    check(
        "let gets=0,caught=false,i={next(){return {value:7};},get return(){gets++;throw 8;}};try{Object.groupBy({[Symbol.iterator](){return i;}},()=>{throw 9;});}catch(e){caught=e===9;}caught && gets===1",
    );
    check(
        "let closed=0,caught=false,i={next(){return {value:7};},return(){closed++;return {};}};try{Object.groupBy({[Symbol.iterator](){return i;}},()=>({[Symbol.toPrimitive](){return {};}}));}catch(e){caught=e instanceof TypeError;}caught && closed===1",
    );
}

#[test]
fn step_failures_and_normal_exhaustion_do_not_close() {
    for step in [
        "throw 7;",
        "return 7;",
        "return {get done(){throw 7;}};",
        "return {get value(){throw 7;}};",
    ] {
        check(&format!(
            "let closed=0,calls=0,caught=false,i={{next(){{{step}}},return(){{closed++;}}}};try{{Object.groupBy({{[Symbol.iterator](){{return i;}}}},()=>{{calls++;return 'x';}});}}catch(e){{caught=e===7 || e instanceof TypeError;}}caught && closed===0 && calls===0"
        ));
    }
    check(
        "let i={next(){return {done:true,get value(){throw 7;}};},get return(){throw 8;}};Object.keys(Object.groupBy({[Symbol.iterator](){return i;}},()=>{throw 9;})).length===0",
    );
}

#[test]
fn result_arrays_use_intrinsics_and_bypass_setters_and_public_constructors() {
    check(
        "let source=[1,2],P=Array.prototype;Object.defineProperty(P,'0',{set(){throw 7;}});Object.defineProperty(Object.prototype,'x',{set(){throw 8;}});let o=Object.groupBy(source,v=>{globalThis.Array=function(){throw 9;};return 'x';}),d=Object.getOwnPropertyDescriptor(o,'x'),e=Object.getOwnPropertyDescriptor(o.x,'0');Object.getPrototypeOf(o)===null && Object.getPrototypeOf(o.x)===P && d.writable && d.enumerable && d.configurable && e.writable && e.enumerable && e.configurable && o.x.join(',')==='1,2'",
    );
    check(
        "let source=Array.from({length:2000},(v,i)=>i),o=Object.groupBy(source,()=> 'all');o.all.length===2000 && o.all[1999]===1999 && Object.keys(o.all).length===2000",
    );
}

#[test]
fn metadata_deletion_and_collected_group_edges_remain_correct() {
    check(
        "let d=Object.getOwnPropertyDescriptor(Object,'groupBy'),n=Object.getOwnPropertyDescriptor(Object.groupBy,'name'),l=Object.getOwnPropertyDescriptor(Object.groupBy,'length');d.writable && !d.enumerable && d.configurable && n.value==='groupBy' && !n.writable && !n.enumerable && n.configurable && l.value===2 && !l.writable && !l.enumerable && l.configurable && !Object.hasOwn(Object.groupBy,'prototype')",
    );
    check(
        "let called=0,caught=false,o={get [Symbol.iterator](){called++;throw 7;}};try{new Object.groupBy(o,()=> 'x');}catch(e){caught=e instanceof TypeError;}caught && called===0",
    );
    check("delete Object.groupBy;Object.groupBy===undefined && !Object.hasOwn(Object,'groupBy')");
    let mut realm = Realm::default();
    realm.eval("let group=Object.groupBy;delete Object.groupBy;delete globalThis.Object;delete globalThis.Array").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    realm
        .eval("let value={x:7},o=group([value],()=> 'x');value=null")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("o.x[0].x===7 && group([1,2],()=> 'x').x.length===2"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn complete_object_constructor_supports_reflection_copying_and_integrity() {
    check(
        "let names=Object.getOwnPropertyNames(Object);names.length===26 && names.includes('fromEntries') && names.includes('groupBy') && Object.getOwnPropertySymbols(Object).length===0 && Object.keys(Object).length===0 && Object.values(Object).length===0 && Object.entries(Object).length===0 && Object.keys({...Object}).length===0 && Object.keys(Object.assign({},Object)).length===0 && Object.keys(Object.defineProperties({},Object)).length===0 && Object.getOwnPropertyDescriptors(Object).groupBy.value===Object.groupBy",
    );
    check(
        "Object.freeze(Object);Object.isFrozen(Object) && Object.isSealed(Object) && !Object.isExtensible(Object) && !Object.getOwnPropertyDescriptor(Object,'groupBy').writable && Object.groupBy([1,2],()=> 'x').x.length===2 && Object.fromEntries([['x',3]]).x===3",
    );
    check(
        "Object.seal(Object);Object.isSealed(Object) && !Object.isFrozen(Object) && Object.getOwnPropertyDescriptor(Object,'groupBy').writable && !Object.getOwnPropertyDescriptor(Object,'groupBy').configurable",
    );
}

#[test]
fn host_failures_during_grouping_materialization_or_cleanup_skip_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let closed=0,flag=0,source={[Symbol.iterator](){return {next(){return {value:7};},return(){closed++;return {};}};}}").unwrap();
    assert!(matches!(
        realm.eval("try{Object.groupBy(source,()=> 'x');}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("closed===0 && flag===0"),
        Ok(Value::Boolean(true))
    );

    let mut realm = Realm::new(Limits {
        max_properties: Some(100),
        ..Limits::default()
    });
    realm.eval("let closed=0,flag=0,steps=0,calls=0,source={[Symbol.iterator](){return {next(){let n=steps++;return n<101?{value:n}:{done:true};},return(){closed++;return {};}};}}").unwrap();
    assert!(matches!(
        realm.eval(
            "try{Object.groupBy(source,v=>{calls++;return 'x';});}catch{flag=1;}finally{flag=2;}"
        ),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("closed===0 && flag===0 && steps===102 && calls===101"),
        Ok(Value::Boolean(true))
    );

    let mut realm = Realm::default();
    realm.eval("let closed=0,flag=0,source={[Symbol.iterator](){return {next(){return {value:7};},return(){closed++;return {};}};}}").unwrap();
    assert!(matches!(
        realm.eval("try{Object.groupBy(source,()=>{Math;});}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(
        realm.eval("closed===0 && flag===0"),
        Ok(Value::Boolean(true))
    );
    assert!(matches!(realm.eval("try{Object.groupBy({[Symbol.iterator](){return {next(){return {value:7};},return(){Math;}};}},()=>{throw 8;});}catch{flag=1;}finally{flag=2;}"),Err(Error::Unsupported{..})));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
