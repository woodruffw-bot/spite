//! Sparse front mutations preserve direction, inherited values, and partial effects.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn shift_returns_the_original_first_value_and_moves_holes_left() {
    check("let a=[1,2,3];a.shift()===1 && a.join()==='2,3' && a.length===2 && !(2 in a)");
    check(
        "let a=[,1,,3];a.shift()===undefined && a.length===3 && a[0]===1 && !(1 in a) && a[2]===3",
    );
    check("let o={},a=[o,7];a.shift()===o && a[0]===7 && a.length===1");
    check("let a=[undefined];a.shift()===undefined && a.length===0 && !Object.hasOwn(a,'0')");
    check("let a=[];a.shift()===undefined && Object.is(a.length,0)");
}

#[test]
fn unshift_moves_elements_right_before_inserting_arguments_in_order() {
    check("let a=[2,3];a.unshift(0,1)===4 && a.join()==='0,1,2,3'");
    check(
        "let a=[,1,,3];a.unshift(7,8)===6 && a[0]===7 && a[1]===8 && !(2 in a) && a[3]===1 && !(4 in a) && a[5]===3",
    );
    check("let o={valueOf:()=>{throw 7;}},a=[];a.unshift(o,1n)===2 && a[0]===o && a[1]===1n");
    check("let a=[1,2];a.unshift()===2 && a.join()==='1,2'");
}

#[test]
fn generic_length_conversion_and_zero_argument_paths_still_write_length() {
    check(
        "let o={0:1,1:2,length:'2.9'};Array.prototype.shift.call(o)===1 && o[0]===2 && !(1 in o) && o.length===1",
    );
    check(
        "let o={0:1,length:'1.9'};Array.prototype.unshift.call(o,7)===2 && o[0]===7 && o[1]===1 && o.length===2",
    );
    check(
        "let n=0,o={};Object.defineProperty(o,'length',{get:()=>-7,set:function(v){if(this!==o || !Object.is(v,0))throw 7;n++;}});Array.prototype.shift.call(o)===undefined && Array.prototype.unshift.call(o)===0 && n===2",
    );
    check(
        "let o={length:Infinity};Object.defineProperty(o,'0',{get:()=>{throw 7;}});Array.prototype.unshift.call(o)===9007199254740991 && o.length===9007199254740991",
    );
    check(
        "Array.prototype.shift.call(false)===undefined && Array.prototype.unshift.call(7)===0 && Array.prototype.unshift.call(false,7)===1",
    );
}

#[test]
fn inherited_values_and_setters_are_observed_and_prototype_properties_survive() {
    check(
        "Array.prototype[0]=7;let a=[,1];a.shift()===7 && a[0]===1 && a.length===1 && Array.prototype[0]===7",
    );
    check(
        "Array.prototype[1]=7;let a=[1,,];a.unshift(9);a[0]===9 && a[1]===1 && a[2]===7 && Array.prototype[1]===7",
    );
    check(
        "let p={0:9},o=Object.create(p);o[0]=1;o.length=2;Array.prototype.shift.call(o)===1 && !Object.hasOwn(o,'0') && o[0]===9 && o.length===1",
    );
    check(
        "let p={},valid=false,value;Object.defineProperty(p,'0',{set:function(v){valid=this===o;value=v;}});let o=Object.create(p);o.length=0;Array.prototype.unshift.call(o,7)===1 && valid && value===7 && !Object.hasOwn(o,'0')",
    );
}

#[test]
fn captured_length_and_live_values_survive_getter_and_setter_mutations() {
    check(
        "let a=[1,2,3];Object.defineProperty(a,'0',{get:()=>{a.length=0;return 1;},configurable:true});a.shift()===1 && a.length===2 && !(0 in a) && !(1 in a)",
    );
    check(
        "let a=[1,2,3];Object.defineProperty(a,'1',{get:()=>{a[2]=9;return 2;},set:function(v){Object.defineProperty(a,'1',{value:v,writable:true});},configurable:true});a.shift()===1 && a[0]===2 && a[1]===9 && a.length===2",
    );
    check(
        "let a=[1,2];Object.defineProperty(a,'1',{get:()=>{a[0]=9;return 2;},set:function(v){Object.defineProperty(a,'1',{value:v,writable:true});},configurable:true});a.unshift(7)===3 && a.join()==='7,9,2'",
    );
    check(
        "let a=[1,2];Object.defineProperty(a,'1',{get:()=>{a.length=0;return 2;},configurable:true});a.unshift(7)===3 && a[0]===7 && !(1 in a) && a[2]===2",
    );
}

#[test]
fn getter_setter_and_final_length_order_follow_each_algorithm() {
    for (call, expected) in [
        ("shift.call(o)", "lng0g1s0:1;g2s1:2;L2;"),
        ("unshift.call(o,7)", "lng2s3:2;g1s2:1;g0s1:0;s0:7;L4;"),
    ] {
        check(&format!(
            "let log='',o={{}};Object.defineProperty(o,'length',{{get:()=>{{log+='l';return {{valueOf:()=>{{log+='n';return 3;}}}};}},set:v=>{{log+='L'+v+';'}}}});function define(k){{Object.defineProperty(o,k,{{get:()=>{{log+='g'+k;return k;}},set:v=>{{log+='s'+k+':'+v+';'}},configurable:true}});}}define(0);define(1);define(2);Object.defineProperty(o,'3',{{set:v=>{{log+='s3:'+v+';'}}}});Array.prototype.{call};log==='{expected}'"
        ));
    }
}

#[test]
fn strict_failures_keep_earlier_moves_insertions_and_deletions() {
    for (setup, call, expected) in [
        (
            "let a=[1,2];Object.defineProperty(a,'length',{writable:false})",
            "a.shift()",
            "a.length===2 && a[0]===2 && !(1 in a)",
        ),
        (
            "let a=[1,2];Object.defineProperty(a,'1',{configurable:false})",
            "a.shift()",
            "a[0]===2 && a[1]===2 && a.length===2",
        ),
        (
            "let a=[1,,];Object.defineProperty(a,'0',{configurable:false})",
            "a.shift()",
            "a[0]===1 && a.length===2",
        ),
        (
            "let a=[1,2];Object.defineProperty(a,'length',{writable:false})",
            "a.unshift(7)",
            "a.join()==='1,2' && a.length===2",
        ),
        (
            "let a={0:1,length:1};Object.defineProperty(a,'length',{writable:false})",
            "Array.prototype.unshift.call(a,7)",
            "a[0]===7 && a[1]===1 && a.length===1",
        ),
        (
            "let a=[1,2];Object.defineProperty(a,'1',{writable:false})",
            "a.unshift(7)",
            "a[0]===1 && a[1]===2 && a[2]===2 && a.length===3",
        ),
        (
            "let a={0:1,length:2};Object.defineProperty(a,'2',{value:9,configurable:false})",
            "Array.prototype.unshift.call(a,7)",
            "a[0]===1 && a[2]===9 && a.length===2",
        ),
    ] {
        let mut realm = Realm::default();
        realm.eval(setup).unwrap();
        assert!(
            matches!(
                realm.eval(call),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{call}"
        );
        assert_eq!(realm.eval(expected), Ok(Value::Boolean(true)), "{call}");
    }
}

#[test]
fn maximum_safe_length_is_checked_before_movement_and_large_keys_stay_exact() {
    check(
        "let n=0,o={length:Infinity};Object.defineProperty(o,'9007199254740990',{get:()=>{n++;throw 7;}});try{Array.prototype.unshift.call(o,1);}catch(e){if(!(e instanceof TypeError))throw e;}n===0 && o.length===Infinity && !Object.hasOwn(o,'0')",
    );
    let mut realm = Realm::default();
    realm.eval("let o={9007199254740989:7,length:9007199254740990};Object.defineProperty(o,'9007199254740988',{get:()=>{throw 7;}})").unwrap();
    assert_eq!(
        realm.eval("Array.prototype.unshift.call(o,1)"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    assert_eq!(
        realm.eval(
            "o[9007199254740990]===7 && o.length===9007199254740990 && !Object.hasOwn(o,'0')"
        ),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::default();
    realm.eval("let a=Array(4294967295);a[4294967294]=7;Object.defineProperty(a,'4294967293',{get:()=>{throw 7;}})").unwrap();
    assert_eq!(
        realm.eval("a.unshift(1)"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    assert_eq!(
        realm.eval("a[4294967295]===7 && a.length===4294967295"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn receiver_conversion_getters_and_readonly_lengths_propagate_errors() {
    for method in ["shift", "unshift"] {
        for receiver in [
            "null",
            "undefined",
            "''",
            "'ab'",
            "{length:1n}",
            "Object.freeze([])",
        ] {
            assert!(
                matches!(
                    Realm::default().eval(&format!("Array.prototype.{method}.call({receiver})")),
                    Err(Error::Exception {
                        kind: ExceptionKind::TypeError,
                        ..
                    })
                ),
                "{method}: {receiver}"
            );
        }
        assert_eq!(
            Realm::default().eval(&format!(
                "Array.prototype.{method}.call({{length:{{valueOf:()=>{{throw 7;}}}}}})"
            )),
            Err(Error::Thrown(Value::Number(7.0)))
        );
        assert_eq!(
            Realm::default().eval(&format!(
                "let a=[1];Object.defineProperty(a,'0',{{get:()=>{{throw 7;}}}});a.{method}(7)"
            )),
            Err(Error::Thrown(Value::Number(7.0)))
        );
    }
}

#[test]
fn metadata_nonconstructibility_and_collection_are_standard() {
    for (method, length) in [("shift", 0), ("unshift", 1)] {
        check(&format!(
            "let f=Array.prototype.{method},d=Object.getOwnPropertyDescriptor(Array.prototype,'{method}');f.name==='{method}' && f.length==={length} && f.prototype===undefined && d.value===f && d.writable && !d.enumerable && d.configurable"
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
        assert_eq!(realm.eval("f.call([1])"), Ok(Value::Number(1.0)));
    }
}
