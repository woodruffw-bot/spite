//! Own-key reflection, enumerable value snapshots, and ordered getter effects.

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn own_names_and_symbols_use_distinct_key_types_and_standard_order() {
    check(
        "let a=Symbol('x'),b=Symbol('x'),o={[a]:1,b:2,2:3,'01':4,0:5,[b]:6,a:7};Object.defineProperty(o,'hidden',{value:8});Object.defineProperty(o,a,{enumerable:false});let names=Object.getOwnPropertyNames(o),symbols=Object.getOwnPropertySymbols(o);names.join(',')==='0,2,b,01,a,hidden' && symbols.length===2 && symbols[0]===a && symbols[1]===b && Object.keys(o).join(',')==='0,2,b,01,a'",
    );
    check(
        "let a=Symbol(),b=Symbol(),o={x:1,y:2,[a]:3,[b]:4};delete o.x;o.x=5;delete o[a];o[a]=6;Object.getOwnPropertyNames(o).join(',')==='y,x' && Object.getOwnPropertySymbols(o)[0]===b && Object.getOwnPropertySymbols(o)[1]===a",
    );
    check(
        "let o={'4294967295':1,'4294967294':2,'-0':3,'0':4};Object.getOwnPropertyNames(o).join(',')==='0,4294967294,4294967295,-0'",
    );
}

#[test]
fn key_reflection_never_gets_values_and_ignores_inherited_properties() {
    check(
        "let s=Symbol(),o=Object.create({inherited:1});Object.defineProperty(o,'x',{get:()=>{throw 7;},enumerable:true});Object.defineProperty(o,'hidden',{get:()=>{throw 8;}});Object.defineProperty(o,s,{get:()=>{throw 9;},enumerable:true});Object.getOwnPropertyNames(o).join(',')==='x,hidden' && Object.getOwnPropertySymbols(o)[0]===s && Object.keys(o).join(',')==='x'",
    );
    check(
        "let o={x:1,[Symbol()]:2};Object.defineProperty(o,Symbol(),{get:()=>{throw 7;},enumerable:true});Object.values(o).join(',')==='1' && Object.entries(o).length===1 && Object.entries(o)[0][0]==='x'",
    );
}

#[test]
fn enumerable_values_and_entries_recheck_live_descriptors_after_each_getter() {
    for method in ["values", "entries"] {
        let result = if method == "values" {
            "r[0]===1 && r[1]===4"
        } else {
            "r[0][0]==='a' && r[0][1]===1 && r[1][0]==='d' && r[1][1]===4"
        };
        check(&format!(
            "let log='',parent={{}},o=Object.create(parent);Object.defineProperty(parent,'b',{{get:()=>{{throw 7;}}}});Object.defineProperty(o,'a',{{get:()=>{{log+='a';delete o.b;Object.defineProperty(o,'c',{{enumerable:false}});Object.defineProperty(o,'d',{{enumerable:true}});o.e=5;return 1;}},enumerable:true}});Object.defineProperty(o,'b',{{value:2,writable:true,enumerable:true,configurable:true}});o.c=3;Object.defineProperty(o,'d',{{get:()=>{{log+='d';return 4;}},configurable:true}});let r=Object.{method}(o);log==='ad' && r.length===2 && {result}"
        ));
    }
    check(
        "let o={};Object.defineProperty(o,'a',{get:()=>{delete o.b;o.b=9;o[0]=7;return 1;},enumerable:true});o.b=2;let r=Object.entries(o);r.length===2 && r[0][0]==='a' && r[1][0]==='b' && r[1][1]===9 && Object.keys(o).join(',')==='0,a,b'",
    );
}

#[test]
fn value_reads_preserve_receivers_and_stop_at_abrupt_completion() {
    for method in ["values", "entries"] {
        check(&format!(
            "let log='',o={{}};Object.defineProperty(o,'a',{{get:function(){{'use strict';log+=this===o?'a':'!';return 1;}},enumerable:true}});Object.defineProperty(o,'b',{{get:()=>{{log+='b';throw 7;}},enumerable:true}});Object.defineProperty(o,'c',{{get:()=>{{log+='c';}},enumerable:true}});let caught=false;try{{Object.{method}(o);}}catch(e){{caught=e===7;}}caught && log==='ab'"
        ));
    }
}

#[test]
fn primitive_wrappers_arrays_and_arguments_expose_their_actual_own_keys() {
    check(
        "let text='\\uD83D\\uDE00',r=Object.values(text);Object.keys(text).join(',')==='0,1' && Object.getOwnPropertyNames(text).join(',')==='0,1,length' && Object.getOwnPropertySymbols(text).length===0 && r.length===2 && r[0]==='\\uD83D' && r[1]==='\\uDE00'",
    );
    for value in ["3", "true", "Symbol('x')"] {
        for method in [
            "keys",
            "values",
            "entries",
            "getOwnPropertyNames",
            "getOwnPropertySymbols",
        ] {
            check(&format!("Object.{method}({value}).length===0"));
        }
    }
    check(
        "let a=[,7,,9];Object.getOwnPropertyNames(a).join(',')==='1,3,length' && Object.keys(a).join(',')==='1,3' && Object.values(a).join(',')==='7,9' && Object.getOwnPropertyNames(a.values()).length===0",
    );
    check(
        "function f(a,b){a=7;let names=Object.getOwnPropertyNames(arguments),symbols=Object.getOwnPropertySymbols(arguments);return names.join(',')==='0,1,length,callee' && symbols.length===1 && symbols[0]===Symbol.iterator && Object.values(arguments).join(',')==='7,2';}f(1,2)",
    );
    check(
        "function f(a){'use strict';a=7;return Object.values(arguments)[0]===1 && Object.keys(arguments).join(',')==='0';}f(1)",
    );
}

#[test]
fn nullish_arguments_throw_without_coercion_and_builtins_are_not_constructors() {
    for method in [
        "keys",
        "values",
        "entries",
        "getOwnPropertyNames",
        "getOwnPropertySymbols",
    ] {
        for source in [
            format!("Object.{method}(null)"),
            format!("Object.{method}()"),
            format!("new Object.{method}({{}})"),
        ] {
            assert!(
                matches!(
                    Realm::default().eval(&source),
                    Err(Error::Exception {
                        kind: ExceptionKind::TypeError,
                        ..
                    })
                ),
                "{source}"
            );
        }
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(Object,'{method}'),fn=d.value;fn.name==='{method}' && fn.length===1 && d.writable && !d.enumerable && d.configurable && fn.call(null,{{x:1}}).length==={}",
            if method == "getOwnPropertySymbols" {
                0
            } else {
                1
            }
        ));
    }
}

#[test]
fn result_arrays_use_intrinsics_and_define_dense_data_elements() {
    check(
        "let entries=Object.entries,proto=Array.prototype,getProto=Object.getPrototypeOf;Object.defineProperty(Array,Symbol.species,{get:()=>{throw 7;}});Array=function(){throw 8;};Object.defineProperty(proto,'0',{set:()=>{throw 9;}});let o={x:{v:7}},r=entries(o),d=Object.getOwnPropertyDescriptor(r,'0');getProto(r)===proto && getProto(r[0])===proto && r!==entries(o) && r[0][0]==='x' && r[0][1]===o.x && d.writable && d.enumerable && d.configurable",
    );
}

#[test]
fn copied_symbol_keys_and_object_values_survive_collection() {
    let mut realm = Realm::default();
    realm.eval("let o={[Symbol('x')]:1,a:{x:7}},symbols=Object.getOwnPropertySymbols(o),values=Object.values(o);o=null").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("symbols[0].description==='x' && values[0].x===7"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn incomplete_intrinsic_key_lists_remain_explicit_gaps() {
    for source in [
        "Object.keys(String.prototype)",
        "Object.getOwnPropertySymbols(globalThis)",
        "Object.keys(Array)",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
}

#[test]
fn enumeration_work_and_reentrant_getters_obey_host_limits() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(1000),
        ..Limits::default()
    });
    realm.eval("let flag=0,o={}").unwrap();
    for index in 0..64 {
        realm.eval(&format!("o[{index}]={index}")).unwrap();
    }
    assert!(matches!(
        realm.eval("try{Object.entries(o);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    std::thread::Builder::new().stack_size(2*1024*1024).spawn(|| {
        let mut realm=Realm::default();
        realm.eval("let flag=0,o={};Object.defineProperty(o,'x',{get:()=>Object.values(o),enumerable:true})").unwrap();
        assert!(matches!(realm.eval("try{Object.values(o);}catch{flag=1;}finally{flag=2;}"),Err(Error::Limit{..})));
        assert_eq!(realm.eval("flag===0 && Object.values({x:7})[0]===7"),Ok(Value::Boolean(true)));
    }).unwrap().join().unwrap();
}
