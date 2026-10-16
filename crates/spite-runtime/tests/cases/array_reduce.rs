//! Array reductions preserve accumulator identity and observable traversal order.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

const METHODS: [&str; 2] = ["reduce", "reduceRight"];

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn reductions_use_the_given_initial_value_or_first_present_element() {
    check("[1,2,3].reduce((a,v)=>a+v)===6 && [1,2,3].reduce((a,v)=>a+v,10)===16");
    check(
        "['a','b','c'].reduce((a,v)=>a+v)==='abc' && ['a','b','c'].reduceRight((a,v)=>a+v)==='cba'",
    );
    check("[1n,2n,3n].reduce((a,v)=>a+v)===6n");
    for method in METHODS {
        check(&format!(
            "let n=0;[,7,,].{method}(()=>{{n++;throw 7;}})===7 && n===0"
        ));
        check(&format!(
            "let n=0;[undefined].{method}(()=>{{n++;throw 7;}})===undefined && n===0"
        ));
        check(&format!(
            "let n=0;[].{method}(()=>{{n++;throw 7;}},undefined)===undefined && n===0"
        ));
        check(&format!(
            "let n=0;[7].{method}((a,v)=>{{n++;return a===undefined && v===7;}},undefined) && n===1"
        ));
        check(&format!(
            "let o={{valueOf:()=>{{throw 7;}}}};[].{method}(()=>0,o)===o && [,,].{method}(()=>0,o)===o && [o].{method}(()=>0)===o"
        ));
        check(&format!(
            "let n=0;[1,2,3].{method}((a,v)=>{{if(n>0 && a!==undefined)throw 7;n++;}},0)===undefined && n===3"
        ));
    }
}

#[test]
fn empty_or_all_hole_receivers_without_an_initial_value_throw_type_error() {
    for method in METHODS {
        for receiver in ["[]", "[,,,]", "{length:0}", "{length:3}", "false"] {
            assert!(matches!(
                Realm::default().eval(&format!("Array.prototype.{method}.call({receiver},()=>1)")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        }
        for callback in ["undefined", "null", "1", "{}"] {
            assert!(matches!(
                Realm::default().eval(&format!("[].{method}({callback},7)")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        }
        check(&format!(
            "let o={{length:2}};Object.defineProperty(o,'1',{{set:function(v){{}}}});Array.prototype.{method}.call(o,()=>{{throw 7;}})===undefined"
        ));
    }
}

#[test]
fn callbacks_receive_four_arguments_and_undefined_this_with_ordered_indices() {
    for method in METHODS {
        check(&format!(
            "let a=[1,2,3],log='',valid=true;a.{method}(function(p,v,k,o){{'use strict';valid=valid && this===undefined && arguments.length===4 && o===a && o[k]===v;log+=k;return p+v;}},0)===6 && valid && log==='{}'",
            if method == "reduce" { "012" } else { "210" }
        ));
        check(&format!(
            "let valid=false;[1].{method}(function(){{valid=this===globalThis;}},0);valid"
        ));
        check(&format!(
            "let o={{}},valid=false;[1].{method}(function(){{valid=this===o;}}.bind(o),0);valid"
        ));
        check(&format!(
            "let valid=false;[1].{method}(()=>{{valid=this===globalThis;}},0);valid"
        ));
        check(&format!(
            "let a={{}},b={{}};[1,2].{method}((p,v)=>v===1?a:b,0)==={}",
            if method == "reduce" { "b" } else { "a" }
        ));
    }
}

#[test]
fn holes_are_skipped_and_inherited_elements_can_seed_or_update_the_accumulator() {
    for method in METHODS {
        check(&format!(
            "let n=0;[,1,,2,,].{method}((a,v)=>{{n++;return a+v;}})===3 && n===1"
        ));
        check(&format!(
            "Array.prototype[1]=7;[,,].{method}(()=>{{throw 7;}})===7"
        ));
        check(&format!(
            "Array.prototype[1]=7;[1,,3].{method}((a,v)=>a+v,0)===11"
        ));
    }
    check(
        "let a=[1,2,3];a.reduce((p,v,k)=>{if(k===0){delete a[1];Array.prototype[1]=8;}return p+v;},0)===12",
    );
    check(
        "let a=[1,2,3];a.reduceRight((p,v,k)=>{if(k===2){delete a[1];Array.prototype[1]=8;}return p+v;},0)===12",
    );
}

#[test]
fn initial_length_is_fixed_but_seed_getters_and_callbacks_can_change_later_values() {
    check("let a=[1,2,3];a.reduce((p,v,k)=>{a[1]=9;a[3]=7;return p+v;},0)===13");
    check("let a=[1,2,3];a.reduceRight((p,v,k)=>{a[0]=9;a[3]=7;return p+v;},0)===14");
    check("let a=[1,2,3],n=0;a.reduce((p,v)=>{n++;a.length=0;return p+v;},0)===1 && n===1");
    check(
        "let a=[1,2,3];Object.defineProperty(a,'0',{get:()=>{delete a[1];a[2]=9;a[3]=7;return 1;}});a.reduce((p,v)=>p+v)===10",
    );
    check(
        "let a=[1,2,3];Object.defineProperty(a,'2',{get:()=>{a.length=0;return 3;},configurable:true});a.reduceRight(()=>{throw 7;})===3",
    );
    check("let a=[1,,,];a.reduce((p,v,k)=>{a[2]=7;return p+v;},0)===8");
    check("let a=[,,,1];a.reduceRight((p,v,k)=>{a[1]=7;return p+v;},0)===8");
}

#[test]
fn generic_receivers_use_boxed_objects_and_utf16_code_unit_indices() {
    check(
        "Array.prototype.reduce.call('💩',(p,v)=>p+v,'')==='💩' && Array.prototype.reduceRight.call('💩',(p,v)=>p+v,'')==='\\uDCA9\\uD83D'",
    );
    check(
        "let valid=true;Array.prototype.reduce.call('ab',(p,v,k,o)=>{valid=valid && typeof o==='object' && o.valueOf()==='ab' && o[k]===v;return p+v;},'')==='ab' && valid",
    );
    check(
        "let o={0:1,1:2,2:9,length:'2.9'};Array.prototype.reduce.call(o,(p,v)=>p+v,0)===3 && Array.prototype.reduceRight.call(o,(p,v)=>p+v,0)===3",
    );
    check(
        "Array.prototype.reduce.call(false,()=>{throw 7;},9)===9 && Array.prototype.reduceRight.call(7,()=>{throw 7;},9)===9",
    );
    for method in METHODS {
        let index = if method == "reduce" {
            "0"
        } else {
            "9007199254740990"
        };
        assert_eq!(Realm::default().eval(&format!("let o={{length:Infinity}};Object.defineProperty(o,'{index}',{{get:()=>{{throw 7;}}}});Array.prototype.{method}.call(o,()=>0)")),Err(Error::Thrown(Value::Number(7.0))));
    }
}

#[test]
fn length_conversion_precedes_validation_and_abrupt_completions_stop_traversal() {
    for method in METHODS {
        for receiver in ["null", "undefined", "{length:1n}"] {
            assert!(matches!(
                Realm::default().eval(&format!(
                    "Array.prototype.{method}.call({receiver},()=>0,0)"
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
            "let n=0,o={{length:1}};Object.defineProperty(o,'0',{{get:()=>{{n++;throw 7;}}}});try{{Array.prototype.{method}.call(o,null);}}catch(e){{if(!(e instanceof TypeError))throw e;}}n===0"
        ));
        let mut realm = Realm::default();
        realm.eval("let n=0").unwrap();
        assert_eq!(
            realm.eval(&format!("[1,2,3].{method}(()=>{{n++;throw 7;}},0)")),
            Err(Error::Thrown(Value::Number(7.0)))
        );
        assert_eq!(realm.eval("n"), Ok(Value::Number(1.0)));
        check(&format!(
            "let log='',o={{}};Object.defineProperty(o,'length',{{get:()=>{{log+='l';return {{valueOf:()=>{{log+='n';return 1;}}}};}}}});Object.defineProperty(o,'0',{{get:()=>{{log+='g';return 7;}}}});Array.prototype.{method}.call(o,(p,v)=>{{log+='c';return p+v;}},0)===7 && log==='lngc'"
        ));
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
        realm.collect(usize::MAX).unwrap();
        assert_eq!(
            realm.eval("f.call([1,2,3],(p,v)=>p+v,0)"),
            Ok(Value::Number(6.0))
        );
    }
}
