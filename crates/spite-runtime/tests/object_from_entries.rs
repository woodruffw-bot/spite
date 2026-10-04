//! Object.fromEntries: entry order, data definitions, and iterator closing.

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
fn results_are_fresh_ordinary_objects_with_ordered_data_properties() {
    check(
        "let o=Object.fromEntries([['b',1],['10',2],['2',3],['a',4],['b',5]]);Object.getPrototypeOf(o)===Object.prototype && !Array.isArray(o) && Object.keys(o).join(',')==='2,10,b,a' && o.b===5 && o.a===4",
    );
    check(
        "let a=Object.fromEntries([]),b=Object.fromEntries([]);a!==b && Object.keys(a).length===0 && Object.getPrototypeOf(a)===Object.prototype",
    );
    check(
        "Object.defineProperty(Object.prototype,'x',{get(){throw 7;},set(){throw 8;}});let o=Object.fromEntries([['x',1],['__proto__',2]]),d=Object.getOwnPropertyDescriptor(o,'x');d.value===1 && d.writable && d.enumerable && d.configurable && Object.hasOwn(o,'__proto__') && o.__proto__===2 && Object.getPrototypeOf(o)===Object.prototype",
    );
    check(
        "let o=Object.fromEntries([['x',1],['x',2]]);Object.getOwnPropertyDescriptor(o,'x').writable && delete o.x && Object.keys(o).length===0",
    );
}

#[test]
fn entries_are_objects_read_by_index_without_length_or_iteration() {
    check(
        "let p={get 0(){if(this!==entry)throw 7;return 'x';},get 1(){if(this!==entry)throw 8;return 9;}},entry=Object.create(p);entry[Symbol.iterator]=()=>{throw 10;};Object.defineProperty(entry,'length',{get(){throw 11;}});let o=Object.fromEntries([entry]);o.x===9",
    );
    check(
        "let o=Object.fromEntries([Object('ab'),new String('cd'),{},['x'],[,'hole']]);o.a==='b' && o.c==='d' && o.x===undefined && o.undefined==='hole'",
    );
    check(
        "let o=Object.fromEntries([function(){},Object(1),Object(true),Object(1n),Object(Symbol())]);Object.hasOwn(o,'undefined') && o.undefined===undefined && Object.keys(o).length===1",
    );
    for entry in ["undefined", "null", "1", "true", "1n", "Symbol()", "'ab'"] {
        check(&format!(
            "let closed=0,i={{next(){{return {{value:{entry}}};}},return(){{closed++;return {{}};}}}},caught=false;try{{Object.fromEntries({{[Symbol.iterator](){{return i;}}}});}}catch(e){{caught=e instanceof TypeError;}}caught && closed===1"
        ));
    }
}

#[test]
fn iterator_receivers_cached_next_and_entry_coercions_follow_spec_order() {
    check(
        "let log='',n=0,key={[Symbol.toPrimitive](hint){if(hint!=='string')throw 7;log+='k';entry[1]=9;return 'x';}},entry={get 0(){log+='0';return key;},get 1(){log+='1';return 3;},set 1(v){log+='s';}};let i={get next(){log+='g';return function(){if(this!==i || arguments.length!==0)throw 8;log+='n';Object.defineProperty(i,'next',{value:7});return n++===0?{get done(){log+='d';return false;},get value(){log+='v';return entry;}}:{get done(){log+='d';return true;},get value(){throw 9;}};};}},source={[Symbol.iterator](){if(this!==source || arguments.length!==0)throw 10;log+='i';return i;}};let o=Object.fromEntries(source);o.x===3 && log==='igndv01ksnd'",
    );
    check(
        "let symbol=Symbol('key'),o=Object.fromEntries([[symbol,1],[{[Symbol.toPrimitive](){return symbol;}},2],[-0,3],[1n,4],[null,5],[true,6],['\\uD800',7]]);o[symbol]===2 && Object.getOwnPropertySymbols(o)[0]===symbol && o['0']===3 && o['1']===4 && o.null===5 && o.true===6 && o['\\uD800']===7",
    );
}

#[test]
fn invalid_inputs_and_iterator_acquisition_failures_do_not_close() {
    for input in [
        "undefined",
        "null",
        "7",
        "true",
        "1n",
        "Symbol()",
        "{}",
        "{[Symbol.iterator]:null}",
        "{[Symbol.iterator]:7}",
    ] {
        check(&format!(
            "let caught=false;try{{Object.fromEntries({input});}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    check("Object.keys(Object.fromEntries('')).length===0");
    check(
        "let caught=false;try{Object.fromEntries({get [Symbol.iterator](){throw 7;}});}catch(e){caught=e===7;}caught",
    );
    check(
        "let closed=0,i={get next(){throw 7;},return(){closed++;}},caught=false;try{Object.fromEntries({[Symbol.iterator](){return i;}});}catch(e){caught=e===7;}caught && closed===0",
    );
}

#[test]
fn stepping_failures_and_normal_exhaustion_do_not_close() {
    for step in [
        "throw 7;",
        "return 7;",
        "return {get done(){throw 7;}};",
        "return {get value(){throw 7;}};",
    ] {
        check(&format!(
            "let closed=0,caught=false,i={{next(){{{step}}},return(){{closed++;}}}};try{{Object.fromEntries({{[Symbol.iterator](){{return i;}}}});}}catch(e){{caught=e===7 || e instanceof TypeError;}}caught && closed===0"
        ));
    }
    check(
        "let closed=0,i={next:7,return(){closed++;}},caught=false;try{Object.fromEntries({[Symbol.iterator](){return i;}});}catch(e){caught=e instanceof TypeError;}caught && closed===0",
    );
    check(
        "let i={next(){return {done:true,get value(){throw 7;}};},get return(){throw 8;}};Object.keys(Object.fromEntries({[Symbol.iterator](){return i;}})).length===0",
    );
}

#[test]
fn entry_failures_close_once_and_preserve_incoming_throws() {
    for entry in [
        "{get 0(){throw original;},get 1(){throw 9;}}",
        "{get 0(){return 'x';},get 1(){throw original;}}",
        "{0:{[Symbol.toPrimitive](){throw original;}},1:3}",
    ] {
        for close in ["return {};", "return 7;", "return undefined;", "throw 8;"] {
            check(&format!(
                "let original={{}},closed=0,steps=0,i={{next(){{steps++;return {{value:{entry}}};}},return(){{if(this!==i || arguments.length!==0)throw 9;closed++;{close}}}}},caught=false;try{{Object.fromEntries({{[Symbol.iterator](){{return i;}}}});}}catch(e){{caught=e===original;}}caught && closed===1 && steps===1"
            ));
        }
    }
    check(
        "let gets=0,caught=false,i={next(){return {value:{get 0(){throw 7;}}};},get return(){gets++;throw 8;}};try{Object.fromEntries({[Symbol.iterator](){return i;}});}catch(e){caught=e===7;}caught && gets===1",
    );
    check(
        "let caught=false,i={next(){return {value:{get 1(){throw 7;}}};},return:8};try{Object.fromEntries({[Symbol.iterator](){return i;}});}catch(e){caught=e===7;}caught",
    );
    check(
        "let log='',entry={0:{[Symbol.toPrimitive](){log+='k';return {}; }},get 1(){log+='v';return 3;}},i={next(){log+='n';return {value:entry};},return(){log+='r';return {}; }},caught=false;try{Object.fromEntries({[Symbol.iterator](){return i;}});}catch(e){caught=e instanceof TypeError;}caught && log==='nvkr'",
    );
}

#[test]
fn metadata_receiver_independence_deletion_and_collection_are_correct() {
    check(
        "let d=Object.getOwnPropertyDescriptor(Object,'fromEntries'),n=Object.getOwnPropertyDescriptor(Object.fromEntries,'name'),l=Object.getOwnPropertyDescriptor(Object.fromEntries,'length');d.writable && !d.enumerable && d.configurable && n.value==='fromEntries' && !n.writable && !n.enumerable && n.configurable && l.value===1 && !l.writable && !l.enumerable && l.configurable && !Object.hasOwn(Object.fromEntries,'prototype')",
    );
    check(
        "let touched=0,caught=false,o={get [Symbol.iterator](){touched++;throw 7;}};try{new Object.fromEntries(o);}catch(e){caught=e instanceof TypeError;}caught && touched===0",
    );
    check(
        "function C(){throw 7;}let o=Object.fromEntries.call(C,[['x',1]],{get ignored(){throw 8;}});Object.getPrototypeOf(o)===Object.prototype && o.x===1 && Object.fromEntries.call(null,[]).constructor===Object",
    );
    check(
        "delete Object.fromEntries;Object.fromEntries===undefined && !Object.hasOwn(Object,'fromEntries')",
    );
    let mut realm = Realm::default();
    realm
        .eval("let make=Object.fromEntries;delete Object.fromEntries;delete globalThis.Object")
        .unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    realm.eval("let o=make([['x',{v:7}]])").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("o.x.v===7 && make([])!==o"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn host_failures_skip_cleanup_and_pending_handlers() {
    let mut realm = Realm::new(Limits {
        max_properties: Some(100),
        ..Limits::default()
    });
    realm.eval("let closed=0,flag=0,n=0,source={[Symbol.iterator](){return {next(){return {value:[n++,7]};},return(){closed++;return {};}};}};").unwrap();
    assert!(matches!(
        realm.eval("try{Object.fromEntries(source);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("closed===0 && flag===0 && n===101"),
        Ok(Value::Boolean(true))
    );

    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let closed=0,flag=0,source={[Symbol.iterator](){return {next(){return {value:['same',7]};},return(){closed++;return {};}};}};").unwrap();
    assert!(matches!(
        realm.eval("try{Object.fromEntries(source);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("closed===0 && flag===0"),
        Ok(Value::Boolean(true))
    );

    let mut realm = Realm::default();
    realm.eval("let flag=0,closed=0").unwrap();
    assert!(matches!(realm.eval("try{Object.fromEntries({[Symbol.iterator](){return {next(){return {value:{get 0(){Math;}}};},return(){closed++;return {};}};}});}catch{flag=1;}finally{flag=2;}"), Err(Error::Unsupported { .. })));
    assert_eq!(
        realm.eval("closed===0 && flag===0"),
        Ok(Value::Boolean(true))
    );
    assert!(matches!(realm.eval("try{Object.fromEntries({[Symbol.iterator](){return {next(){return {value:null};},return(){Math;}};}});}catch{flag=1;}finally{flag=2;}"), Err(Error::Unsupported { .. })));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
