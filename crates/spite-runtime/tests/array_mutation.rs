//! Generic Array push/pop and their required partial effects (23.1.3.22–23).

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}
fn type_error(realm: &mut Realm, source: &str) {
    assert!(
        matches!(
            realm.eval(source),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ),
        "{source}"
    );
}

#[test]
fn push_and_pop_preserve_element_identity_and_sparse_lengths() {
    check(
        "let a=[],o={};a.push(1,undefined,o)===3 && a.length===3 && Object.hasOwn(a,'1') && a.pop()===o && a.pop()===undefined && a.pop()===1 && a.pop()===undefined && a.length===0",
    );
    check(
        "let a=Array(4);a.push(7)===5 && !Object.hasOwn(a,'3') && a[4]===7 && a.pop()===7 && a.length===4 && a.pop()===undefined && a.length===3",
    );
    check("let a=[1];a.push()===1 && a[0]===1 && a.length===1");
    check(
        "let a=[{toString:()=>{throw 7;},valueOf:()=>{throw 8;}}],v=a.pop();a.push(v)===1 && a[0]===v",
    );
}

#[test]
fn generic_receivers_convert_length_and_preserve_safe_integer_keys() {
    check(
        "let o={length:'1.9',0:'a'};Array.prototype.push.call(o,'b')===2 && o.length===2 && o[0]==='a' && o[1]==='b' && Array.prototype.pop.call(o)==='b' && o.length===1 && !('1' in o)",
    );
    check("let o={};Array.prototype.push.call(o,7)===1 && o[0]===7 && o.length===1");
    check(
        "let o={length:9007199254740990};Array.prototype.push.call(o,'last')===9007199254740991 && o[9007199254740990]==='last' && Array.prototype.pop.call(o)==='last' && o.length===9007199254740990",
    );
    check(
        "let o={length:Infinity,9007199254740990:7};Array.prototype.pop.call(o)===7 && o.length===9007199254740990 && !('9007199254740990' in o)",
    );
    check("Array.prototype.push.call(true,7)===1 && Array.prototype.pop.call(7)===undefined");
    for length in [
        "undefined",
        "null",
        "false",
        "NaN",
        "-Infinity",
        "-3",
        "0.9",
        "-0",
    ] {
        check(&format!(
            "let o={{length:{length},0:7}};Array.prototype.pop.call(o)===undefined && Object.is(o.length,0) && o[0]===7"
        ));
    }
}

#[test]
fn push_invokes_inherited_setters_in_order_and_sets_length_last() {
    check(
        "let log='',p={};Object.defineProperty(p,'0',{set:function(v){log+='0'+v;this.saved=v;}});Object.defineProperty(p,'1',{set:function(v){log+='1'+v;}});let o=Object.create(p);Object.defineProperty(o,'length',{get:()=>{log+='l';return {valueOf:()=>{log+='n';return 0;}};},set:v=>{log+='L'+v;}});Array.prototype.push.call(o,'a','b')===2 && log==='ln0a1bL2' && o.saved==='a' && !Object.hasOwn(o,'0')",
    );
    check(
        "let a=[];Object.defineProperty(Array.prototype,'0',{set:function(v){this.saved=v;},configurable:true});a.push(7)===1 && a.saved===7 && !Object.hasOwn(a,'0') && a.length===1",
    );
    check(
        "let log='',o={};Object.defineProperty(o,'length',{get:()=>{log+='g';return 0;},set:v=>{log+='s'+v;}});Array.prototype.push.call(o)===0 && log==='gs0'",
    );
}

#[test]
fn pop_reads_inherited_elements_and_uses_the_snapshotted_length() {
    check(
        "let p={1:7},o=Object.create(p);o.length=2;Array.prototype.pop.call(o)===7 && o.length===1 && p[1]===7 && !Object.hasOwn(o,'1')",
    );
    check(
        "let a=[1,2,3];Object.defineProperty(a,'2',{get:()=>{a.length=0;return 7;},configurable:true});a.pop()===7 && a.length===2 && !Object.hasOwn(a,'0') && !Object.hasOwn(a,'2')",
    );
    check(
        "let log='',o={};Object.defineProperty(o,'length',{get:()=>{log+='l';return 1;},set:v=>{log+='L'+v;}});Object.defineProperty(o,'0',{get:()=>{log+='g';return 7;},configurable:true});Array.prototype.pop.call(o)===7 && log==='lgL0' && !Object.hasOwn(o,'0')",
    );
    check(
        "let log='',o={};Object.defineProperty(o,'length',{get:()=>{log+='g';return -1;},set:v=>{log+='s'+v;}});Array.prototype.pop.call(o)===undefined && log==='gs0'",
    );
}

#[test]
fn read_only_length_rejects_even_empty_operations_and_pop_can_delete_first() {
    for operation in ["a.push()", "a.pop()"] {
        let mut realm = Realm::default();
        realm
            .eval("let a=[];Object.defineProperty(a,'length',{writable:false})")
            .unwrap();
        type_error(&mut realm, operation);
    }
    let mut realm = Realm::default();
    realm
        .eval("let a=[1,2];Object.defineProperty(a,'length',{writable:false})")
        .unwrap();
    type_error(&mut realm, "a.push(3)");
    assert_eq!(
        realm.eval("a.length===2 && !('2' in a) && a[1]===2"),
        Ok(Value::Boolean(true))
    );
    type_error(&mut realm, "a.pop()");
    assert_eq!(
        realm.eval("a.length===2 && !('1' in a) && a[0]===1"),
        Ok(Value::Boolean(true))
    );
    for source in [
        "Array.prototype.push.call('',1)",
        "Array.prototype.push.call('x')",
        "Array.prototype.pop.call('x')",
        "Array.prototype.pop.call('')",
    ] {
        type_error(&mut Realm::default(), source);
    }
}

#[test]
fn failed_element_operations_keep_earlier_mutations() {
    let mut realm = Realm::default();
    realm
        .eval("let o={length:0};Object.defineProperty(o,'1',{value:9,writable:false})")
        .unwrap();
    type_error(&mut realm, "Array.prototype.push.call(o,1,2,3)");
    assert_eq!(
        realm.eval("o[0]===1 && o[1]===9 && !('2' in o) && o.length===0"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::default();
    realm.eval("let n=0,o={length:1};Object.defineProperty(o,'0',{get:()=>{n++;return 7;},configurable:false})").unwrap();
    type_error(&mut realm, "Array.prototype.pop.call(o)");
    assert_eq!(
        realm.eval("n===1 && o.length===1 && Object.hasOwn(o,'0')"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::default();
    realm
        .eval("let o={0:7};Object.defineProperty(o,'length',{value:1,writable:false})")
        .unwrap();
    type_error(&mut realm, "Array.prototype.pop.call(o)");
    assert_eq!(
        realm.eval("o.length===1 && !('0' in o)"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn safe_integer_overflow_precedes_writes_but_array_length_overflow_follows_them() {
    let mut realm = Realm::default();
    realm.eval("let n=0,o={length:9007199254740991};Object.defineProperty(o,'9007199254740991',{set:()=>{n++;}})").unwrap();
    type_error(&mut realm, "Array.prototype.push.call(o,1)");
    assert_eq!(
        realm.eval("n===0 && o.length===9007199254740991"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::default();
    realm.eval("let a=Array(4294967294)").unwrap();
    assert!(matches!(
        realm.eval("a.push('last','ordinary')"),
        Err(Error::Exception {
            kind: ExceptionKind::RangeError,
            ..
        })
    ));
    assert_eq!(
        realm.eval("a.length===4294967295 && a[4294967294]==='last' && a[4294967295]==='ordinary'"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(
        realm.eval("a.pop()==='last' && a.length===4294967294 && a[4294967295]==='ordinary'"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn throwing_length_setters_and_index_getters_preserve_order_and_identity() {
    let mut realm = Realm::default();
    realm
        .eval("let o={length:0};Object.defineProperty(o,'length',{get:()=>0,set:()=>{throw 7;}})")
        .unwrap();
    assert_eq!(
        realm.eval("Array.prototype.push.call(o,9)"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    assert_eq!(realm.eval("o[0]"), Ok(Value::Number(9.0)));
    let mut realm = Realm::default();
    realm
        .eval("let o={0:9};Object.defineProperty(o,'length',{get:()=>1,set:()=>{throw 7;}})")
        .unwrap();
    assert_eq!(
        realm.eval("Array.prototype.pop.call(o)"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    assert_eq!(
        realm.eval("!Object.hasOwn(o,'0')"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::default();
    realm
        .eval(
            "let o={length:1};Object.defineProperty(o,'0',{get:()=>{throw 7;},configurable:true})",
        )
        .unwrap();
    assert_eq!(
        realm.eval("Array.prototype.pop.call(o)"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    assert_eq!(
        realm.eval("o.length===1 && Object.hasOwn(o,'0')"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn invalid_receivers_and_bigint_lengths_throw_type_errors() {
    for method in ["push", "pop"] {
        for value in ["undefined", "null", "{length:1n}"] {
            type_error(
                &mut Realm::default(),
                &format!("Array.prototype.{method}.call({value})"),
            );
        }
    }
}

#[test]
fn methods_have_standard_metadata_and_survive_collection() {
    for (method, length) in [("push", 1), ("pop", 0)] {
        check(&format!(
            "let f=Array.prototype.{method},d=Object.getOwnPropertyDescriptor(Array.prototype,'{method}');f.name==='{method}' && f.length==={length} && f.prototype===undefined && d.value===f && d.writable && !d.enumerable && d.configurable"
        ));
        type_error(
            &mut Realm::default(),
            &format!("new Array.prototype.{method}"),
        );
    }
    let mut realm = Realm::default();
    realm.eval("let push=Array.prototype.push,pop=Array.prototype.pop;delete Array.prototype.push;delete Array.prototype.pop").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("let a=[];push.call(a,7)===1 && pop.call(a)===7 && a.length===0"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn user_code_can_make_length_read_only_during_an_element_operation() {
    for change in [
        "Object.freeze(a)",
        "Object.defineProperty(a,'length',{writable:false})",
    ] {
        let mut realm = Realm::default();
        realm.eval(&format!("let a=[],n=0;Object.defineProperty(Array.prototype,'0',{{set:()=>{{{change};n++;}}}})")).unwrap();
        type_error(&mut realm, "a.push(1)");
        assert_eq!(
            realm.eval("n===1 && a.length===0 && !Object.hasOwn(a,'0')"),
            Ok(Value::Boolean(true))
        );
        let mut realm = Realm::default();
        realm.eval(&format!("let a=Array(1),n=0;Object.defineProperty(Array.prototype,'0',{{get:()=>{{{change};n++;return 7;}}}})")).unwrap();
        type_error(&mut realm, "a.pop()");
        assert_eq!(
            realm.eval("n===1 && a.length===1 && !Object.hasOwn(a,'0')"),
            Ok(Value::Boolean(true))
        );
    }
}
