//! Array forEach/every/some: ordered live property visits and callback calls.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn callbacks_visit_present_elements_in_order_with_three_arguments() {
    check(
        "let a=[1,,3],log='',valid=true;let result=a.forEach(function(v,k,o){valid=valid && arguments.length===3 && o===a;log+=k+':'+v+';';return false;});result===undefined && valid && log==='0:1;2:3;'",
    );
    check(
        "[1,2,3].every(v=>v>0) && ![1,2,3].every(v=>v<3) && [1,2,3].some(v=>v===2) && ![1,2,3].some(v=>v===9)",
    );
    check(
        "[,,].every(()=>{throw 7;}) && ![,,].some(()=>{throw 7;}) && [,,].forEach(()=>{throw 7;})===undefined",
    );
    check(
        "let log='',a=[1,,3];Object.defineProperty(Array.prototype,'1',{get:function(){if(this!==a)throw 7;return 9;},configurable:true});a.forEach((v,k)=>{log+=k+':'+v+';';});log==='0:1;1:9;2:3;'",
    );
}

#[test]
fn predicates_short_circuit_without_converting_object_results() {
    check("let n=0;let result=[1,2,3].every(v=>{n++;return v<2;});!result && n===2");
    check("let n=0;let result=[1,2,3].some(v=>{n++;return v===2;});result && n===2");
    check(
        "let o={valueOf:()=>{throw 7;},toString:()=>{throw 8;}};[1].every(()=>o) && [1].some(()=>o)",
    );
    for value in ["undefined", "null", "false", "0", "-0", "NaN", "''", "0n"] {
        check(&format!(
            "![1].every(()=>{value}) && ![1].some(()=>{value})"
        ));
    }
    check(
        "Array.prototype.some.call({0:7,length:Infinity},()=>true) && !Array.prototype.every.call({0:7,length:Infinity},()=>false)",
    );
}

#[test]
fn length_is_snapshotted_but_presence_and_values_remain_live() {
    check(
        "let a=[1,2,3],log='';a.forEach((v,k)=>{log+=k+':'+v+';';if(k===0){delete a[1];a[2]=9;a[3]=4;}});log==='0:1;2:9;'",
    );
    check("let a=[1,,3],log='';a.forEach((v,k)=>{log+=v;if(k===0)a[1]=2;});log==='123'");
    check(
        "let a=[1,2,3],log='';Array.prototype[2]=7;a.forEach((v,k)=>{log+=k+':'+v+';';if(k===0)a.length=0;});log==='0:1;2:7;'",
    );
    check(
        "let a=[1,2],log='';Object.defineProperty(a,'0',{get:()=>{delete a[1];return 9;}});a.forEach(v=>{log+=v;});log==='9'",
    );
    check(
        "let log='',o={0:1,1:2};Object.defineProperty(o,'length',{get:()=>{log+='l';return {valueOf:()=>{log+='n';return 2;}};}});Array.prototype.forEach.call(o,v=>{log+=v;});log==='ln12'",
    );
}

#[test]
fn this_arg_follows_strict_sloppy_arrow_and_bound_function_rules() {
    for method in ["forEach", "every", "some"] {
        check(&format!(
            "let valid=false,o={{}};[1].{method}(function(){{'use strict';valid=this===o;}},o);valid"
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
            "let valid=false,o={{}},f=function(){{'use strict';valid=this===o;}}.bind(o);[1].{method}(f,7);valid"
        ));
        check(&format!(
            "let valid=false;[1].{method}(()=>{{valid=this===globalThis;}},7);valid"
        ));
    }
}

#[test]
fn methods_box_generic_receivers_and_preserve_utf16_units() {
    check(
        "let log='',object;Array.prototype.forEach.call('💩x',(v,k,o)=>{log+=v;object=o;});log==='💩x' && typeof object==='object' && object.valueOf()==='💩x'",
    );
    check(
        "Array.prototype.every.call('💩',v=>v.length===1) && Array.prototype.some.call('💩',v=>v==='\\uDCA9')",
    );
    check(
        "let n=0;Array.prototype.forEach.call(true,()=>{n++;});Array.prototype.every.call(7,()=>false) && !Array.prototype.some.call(false,()=>true) && n===0",
    );
    check(
        "let log='',o={0:7,1:8,length:'2.9'};Array.prototype.forEach.call(o,(v,k,a)=>{if(a!==o)throw 1;log+=v;});log==='78'",
    );
}

#[test]
fn callback_validation_follows_length_conversion_even_for_empty_receivers() {
    for method in ["forEach", "every", "some"] {
        for callback in ["undefined", "null", "7", "{}", "{valueOf:()=>{throw 8;}}"] {
            assert!(matches!(
                Realm::default().eval(&format!("[].{method}({callback})")),
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
        assert_eq!(Realm::default().eval(&format!("let o={{length:{{valueOf:()=>{{throw 7;}}}}}};Array.prototype.{method}.call(o,null)")),Err(Error::Thrown(Value::Number(7.0))));
        check(&format!(
            "let n=0,o={{length:1}};Object.defineProperty(o,'0',{{get:()=>{{n++;return 7;}}}});try{{Array.prototype.{method}.call(o,null);}}catch(e){{if(!(e instanceof TypeError))throw e;}}n===0"
        ));
    }
}

#[test]
fn abrupt_getters_and_callbacks_stop_later_visits() {
    for method in ["forEach", "every", "some"] {
        let mut realm = Realm::default();
        realm.eval("let n=0,a=[1,2,3]").unwrap();
        assert_eq!(
            realm.eval(&format!("a.{method}(()=>{{n++;throw 7;}})")),
            Err(Error::Thrown(Value::Number(7.0)))
        );
        assert_eq!(realm.eval("n"), Ok(Value::Number(1.0)));
        assert_eq!(Realm::default().eval(&format!("let a=[1];Object.defineProperty(a,'0',{{get:()=>{{throw 7;}}}});a.{method}(()=>true)")),Err(Error::Thrown(Value::Number(7.0))));
    }
}

#[test]
fn methods_have_standard_metadata_nonconstructibility_and_root_lifetimes() {
    for method in ["forEach", "every", "some"] {
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
    }
    let mut realm = Realm::default();
    realm.eval("let each=Array.prototype.forEach,every=Array.prototype.every,some=Array.prototype.some;delete Array.prototype.forEach;delete Array.prototype.every;delete Array.prototype.some").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let n=0;each.call([1,2],v=>{n+=v;});n===3 && every.call([1,2],v=>v>0) && some.call([1,2],v=>v===2)"),Ok(Value::Boolean(true)));
}
