//! FindViaPredicate visits every index and retains the pre-predicate value.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

const METHODS: [&str; 4] = ["find", "findIndex", "findLast", "findLastIndex"];

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn predicates_find_the_first_matching_value_or_index_in_each_direction() {
    check(
        "[1,2,3,2].find(v=>v===2)===2 && [1,2,3,2].findIndex(v=>v===2)===1 && [1,2,3,2].findLast(v=>v===2)===2 && [1,2,3,2].findLastIndex(v=>v===2)===3",
    );
    check(
        "[].find(()=>true)===undefined && [].findIndex(()=>true)===-1 && [].findLast(()=>true)===undefined && [].findLastIndex(()=>true)===-1",
    );
    check(
        "[1,2].find(()=>false)===undefined && [1,2].findIndex(()=>false)===-1 && [1,2].findLast(()=>false)===undefined && [1,2].findLastIndex(()=>false)===-1",
    );
    check("Object.is([1].findIndex(()=>true),0) && Object.is([1].findLastIndex(()=>true),0)");
    check("let n=0;[1,2,3].find(v=>{n++;return v===2;})===2 && n===2");
    check("let n=0;[1,2,3].findLast(v=>{n++;return v===2;})===2 && n===2");
}

#[test]
fn holes_deleted_elements_and_inherited_values_are_visited() {
    for method in METHODS {
        check(&format!(
            "let n=0;[,,].{method}(v=>{{if(v!==undefined)throw 7;n++;return false;}});n===2"
        ));
    }
    check("[,1,,].findIndex(v=>v===undefined)===0 && [,1,,].findLastIndex(v=>v===undefined)===2");
    check(
        "let a=[1,2,3],log='';a.find((v,k)=>{log+=k+':'+v+';';delete a[1];return false;});log==='0:1;1:undefined;2:3;'",
    );
    check(
        "let a=[1,2,3],log='';a.findLast((v,k)=>{log+=k+':'+v+';';delete a[1];return false;});log==='2:3;1:undefined;0:1;'",
    );
    check("Array.prototype[1]=7;[, ,].find(v=>v===7)===7 && [, ,].findLastIndex(v=>v===7)===1");
}

#[test]
fn range_is_fixed_but_values_are_live_and_found_value_is_retained() {
    check("let a=[1,2],log='';a.find((v,k)=>{log+=v;a[1]=9;a[2]=7;return false;});log==='19'");
    check(
        "let a=[1,2,3],log='';a.findLast((v,k)=>{log+=v;a[0]=9;a[3]=7;return false;});log==='329'",
    );
    check(
        "let a=[1,2,3],log='';a.find((v,k)=>{log+=k+':'+v+';';a.length=0;return false;});log==='0:1;1:undefined;2:undefined;'",
    );
    for method in ["find", "findLast"] {
        check(&format!(
            "let o={{}},a=[o];a.{method}((v,k)=>{{a[k]=7;return true;}})===o && a[0]===7"
        ));
        check(&format!(
            "let a=['before'];a.{method}((v,k)=>{{delete a[k];return true;}})==='before' && !Object.hasOwn(a,'0')"
        ));
    }
}

#[test]
fn generic_receivers_and_full_safe_integer_range_are_supported() {
    check(
        "Array.prototype.find.call('💩x',v=>v==='\\uDCA9')==='\\uDCA9' && Array.prototype.findLastIndex.call('💩x',v=>v==='\\uDCA9')===1",
    );
    check(
        "let o={0:'a',1:'b',length:'2.9'};Array.prototype.find.call(o,v=>v==='b')==='b' && Array.prototype.findLast.call(o,()=>true)==='b'",
    );
    check(
        "let o={9007199254740990:'last',length:Infinity};Array.prototype.findLast.call(o,()=>true)==='last' && Array.prototype.findLastIndex.call(o,()=>true)===9007199254740990 && Array.prototype.findIndex.call(o,()=>true)===0",
    );
    check(
        "Array.prototype.find.call(false,()=>true)===undefined && Array.prototype.findLastIndex.call(7,()=>true)===-1",
    );
}

#[test]
fn callback_arguments_receivers_and_boolean_results_follow_call_semantics() {
    for method in METHODS {
        check(&format!(
            "let a=[1,2],log='',valid=true,o={{}};a.{method}(function(v,k,r){{'use strict';valid=valid && this===o && r===a && arguments.length===3;log+=k;return false;}},o);valid && log==='{}'",
            if method.contains("Last") { "10" } else { "01" }
        ));
        check(&format!(
            "let valid=false;[1].{method}(function(){{'use strict';valid=this===undefined;}});valid"
        ));
        check(&format!(
            "let valid=false;[1].{method}(function(){{'use strict';valid=this===false;}},false);valid"
        ));
        check(&format!(
            "let valid=false;[1].{method}(function(){{valid=this===globalThis;}});valid"
        ));
        check(&format!(
            "let valid=false;[1].{method}(function(){{valid=typeof this==='object' && this.valueOf()===false;}},false);valid"
        ));
        check(&format!(
            "let n=0,o={{valueOf:()=>{{throw 7;}}}};[1,2].{method}(()=>{{n++;return o;}});n===1"
        ));
    }
}

#[test]
fn length_conversion_precedes_predicate_validation_and_index_access() {
    for method in METHODS {
        for predicate in ["undefined", "null", "1", "{}"] {
            assert!(matches!(
                Realm::default().eval(&format!("[].{method}({predicate})")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        }
        for receiver in ["null", "undefined", "{length:1n}"] {
            assert!(matches!(
                Realm::default().eval(&format!(
                    "Array.prototype.{method}.call({receiver},()=>true)"
                )),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        }
        assert_eq!(
            Realm::default().eval(&format!(
                "Array.prototype.{method}.call({{length:{{valueOf:()=>{{throw 7;}}}}}},null)"
            )),
            Err(Error::Thrown(Value::Number(7.0)))
        );
        check(&format!(
            "let n=0,o={{length:1}};Object.defineProperty(o,'0',{{get:()=>{{n++;return 7;}}}});try{{Array.prototype.{method}.call(o,null);}}catch(e){{if(!(e instanceof TypeError))throw e;}}n===0"
        ));
    }
    check(
        "let log='',o={};Object.defineProperty(o,'length',{get:()=>{log+='l';return {valueOf:()=>{log+='n';return 1;}};}});Object.defineProperty(o,'0',{get:()=>{log+='g';return 7;}});Array.prototype.find.call(o,v=>{log+='p';return true;})===7 && log==='lngp'",
    );
}

#[test]
fn getter_and_predicate_abrupt_completions_stop_traversal() {
    for method in METHODS {
        let mut realm = Realm::default();
        realm.eval("let n=0").unwrap();
        assert_eq!(
            realm.eval(&format!("[1,2,3].{method}(()=>{{n++;throw 7;}})")),
            Err(Error::Thrown(Value::Number(7.0)))
        );
        assert_eq!(realm.eval("n"), Ok(Value::Number(1.0)));
        assert_eq!(Realm::default().eval(&format!("let a=[1];Object.defineProperty(a,'0',{{get:()=>{{throw 7;}}}});a.{method}(()=>true)")),Err(Error::Thrown(Value::Number(7.0))));
    }
}

#[test]
fn metadata_nonconstructibility_and_collection_are_standard() {
    for method in METHODS {
        check(&format!(
            "let f=Array.prototype.{method},d=Object.getOwnPropertyDescriptor(Array.prototype,'{method}');f.name==='{method}' && f.length===1 && f.prototype===undefined && d.value===f && d.writable && !d.enumerable && d.configurable"
        ));
        assert!(matches!(
            Realm::default().eval(&format!("new Array.prototype.{method}")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        let mut realm = Realm::default();
        realm
            .eval(&format!(
                "let f=Array.prototype.{method};delete Array.prototype.{method}"
            ))
            .unwrap();
        realm.collect(10_000).unwrap();
        assert_eq!(
            realm.eval("f.call([7],()=>true)"),
            Ok(Value::Number(if method.ends_with("Index") {
                0.0
            } else {
                7.0
            }))
        );
    }
}
