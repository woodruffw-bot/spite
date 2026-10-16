//! Reverse preserves holes and the order of observable reads and partial writes.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn reverses_in_place_preserving_length_identity_and_holes() {
    check("let a=[1,2,3,4];a.reverse()===a && a.join()==='4,3,2,1' && a.length===4");
    check(
        "let a=[1,,3,,];a.reverse()===a && a.length===4 && !(0 in a) && a[1]===3 && !(2 in a) && a[3]===1",
    );
    check("let a=[undefined,,];a.reverse();!(0 in a) && Object.hasOwn(a,'1') && a[1]===undefined");
    check("let a=[1,2,3];a.extra=7;a.reverse();a.join()==='3,2,1' && a.extra===7");
    check("let o={valueOf:()=>{throw 7;}},a=[o,1n];a.reverse();a[0]===1n && a[1]===o");
}

#[test]
fn empty_and_singleton_ranges_do_not_read_elements_or_write_length() {
    check("let a=Object.freeze([]),b=Object.freeze([1]);a.reverse()===a && b.reverse()===b");
    check(
        "let o={length:1};Object.defineProperty(o,'0',{get:()=>{throw 7;}});Array.prototype.reverse.call(o)===o",
    );
    check(
        "let a=[1,2,3];Object.defineProperty(a,'1',{get:()=>{throw 7;}});a.reverse();a[0]===3 && a[2]===1",
    );
    check(
        "let n=0,o={0:1,1:2};Object.defineProperty(o,'length',{get:()=>2,set:()=>{n++;throw 7;}});Array.prototype.reverse.call(o)===o && n===0 && o[0]===2 && o[1]===1",
    );
    check(
        "let a=[1,2];Object.defineProperty(a,'length',{writable:false});a.reverse();a[0]===2 && a[1]===1 && a.length===2",
    );
    check("let a=Object.preventExtensions([,,]);a.reverse()===a && !(0 in a) && !(1 in a)");
}

#[test]
fn inherited_values_and_setters_use_the_original_receiver() {
    check(
        "Array.prototype[1]=7;let a=[1,,];a.reverse();Object.hasOwn(a,'0') && Object.hasOwn(a,'1') && a[0]===7 && a[1]===1 && Array.prototype[1]===7",
    );
    check(
        "let p={1:7},o=Object.create(p);o.length=2;Array.prototype.reverse.call(o);o[0]===7 && o[1]===7 && !Object.hasOwn(o,'1') && p[1]===7",
    );
    check(
        "let valid=false,value,p={};Object.defineProperty(p,'0',{set:function(v){valid=this===o;value=v;}});let o=Object.create(p);o[1]=7;o.length=2;Array.prototype.reverse.call(o);valid && value===7 && Object.hasOwn(o,'1') && o[1]===undefined && !Object.hasOwn(o,'0')",
    );
}

#[test]
fn lower_get_precedes_upper_presence_check_and_both_reads_precede_writes() {
    check(
        "let a=[1,2];Object.defineProperty(a,'0',{get:()=>{a.length=0;return 1;},configurable:true});a.reverse();!(0 in a) && a[1]===1 && a.length===2",
    );
    check(
        "let a=[,,];Object.defineProperty(a,'0',{get:()=>{delete a[0];a[1]=2;return 1;},configurable:true});a.reverse();a[0]===2 && a[1]===1",
    );
    check(
        "let value,a=[1,2];Object.defineProperty(a,'1',{get:()=>{delete a[0];return 7;},set:v=>{value=v;}});a.reverse();a[0]===7 && value===1",
    );
    check(
        "let log='',o={};Object.defineProperty(o,'length',{get:()=>{log+='l';return {valueOf:()=>{log+='n';return 2;}};}});Object.defineProperty(o,'0',{get:()=>{log+='a';return 1;},set:v=>{log+='c'+v;}});Object.defineProperty(o,'1',{get:()=>{log+='b';return 2;},set:v=>{log+='d'+v;}});Array.prototype.reverse.call(o);log==='lnabc2d1'",
    );
}

#[test]
fn length_is_snapshotted_while_later_pairs_observe_mutations() {
    check(
        "let a=[1,2,3,4];Object.defineProperty(a,'0',{get:()=>1,set:v=>{a[1]=9;a[4]=7;}});a.reverse();a[1]===3 && a[2]===9 && a[3]===1 && a[4]===7 && a.length===5",
    );
    check(
        "let a=[1,2,3,4];Object.defineProperty(a,'0',{get:()=>{a.length=2;return 1;},configurable:true});a.reverse();a.length===4 && a[1]===undefined && a[2]===2 && a[3]===1 && !(0 in a)",
    );
}

#[test]
fn failing_strict_writes_or_deletions_keep_the_required_earlier_effects() {
    for (setup, expected) in [
        (
            "let a=[1,2];Object.defineProperty(a,'1',{writable:false})",
            "a[0]===2 && a[1]===2",
        ),
        (
            "let a=[,7];Object.defineProperty(a,'1',{configurable:false})",
            "a[0]===7 && a[1]===7",
        ),
        (
            "let a=[7,,];Object.defineProperty(a,'0',{configurable:false})",
            "a[0]===7 && !(1 in a)",
        ),
        (
            "let a=Object.preventExtensions([7,,])",
            "!(0 in a) && !(1 in a) && a.length===2",
        ),
        ("let a=Object.freeze([1,2])", "a[0]===1 && a[1]===2"),
    ] {
        let mut realm = Realm::default();
        realm.eval(setup).unwrap();
        assert!(
            matches!(
                realm.eval("a.reverse()"),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{setup}"
        );
        assert_eq!(realm.eval(expected), Ok(Value::Boolean(true)), "{setup}");
    }
    check(
        "let a=[1,2],seen=0;Object.defineProperty(a,'0',{get:()=>1,set:()=>{seen++;throw 7;}});try{a.reverse();}catch(e){if(e!==7)throw e;}seen===1 && a[1]===2",
    );
}

#[test]
fn generic_receivers_use_tolength_lengths_and_propagate_getter_failures() {
    check(
        "let o={0:'a',1:'b',length:'2.9'};Array.prototype.reverse.call(o)===o && o[0]==='b' && o[1]==='a' && o.length==='2.9'",
    );
    check(
        "Array.prototype.reverse.call(false).valueOf()===false && Array.prototype.reverse.call(7).valueOf()===7 && Array.prototype.reverse.call('x').valueOf()==='x'",
    );
    for receiver in ["null", "undefined", "'ab'", "'aa'", "{length:1n}"] {
        assert!(matches!(
            Realm::default().eval(&format!("Array.prototype.reverse.call({receiver})")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    for source in [
        "Array.prototype.reverse.call({length:{valueOf:()=>{throw 7;}}})",
        "let o={length:Infinity};Object.defineProperty(o,'9007199254740990',{get:()=>{throw 7;}});Array.prototype.reverse.call(o)",
        "let a=[1,2];Object.defineProperty(a,'0',{get:()=>{throw 7;}});a.reverse()",
        "let a=[1,2];Object.defineProperty(a,'1',{get:()=>{throw 7;}});a.reverse()",
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Err(Error::Thrown(Value::Number(7.0)))
        );
    }
}

#[test]
fn metadata_nonconstructibility_and_collection_are_standard() {
    check(
        "let f=Array.prototype.reverse,d=Object.getOwnPropertyDescriptor(Array.prototype,'reverse');f.name==='reverse' && f.length===0 && f.prototype===undefined && d.value===f && d.writable && !d.enumerable && d.configurable",
    );
    assert!(matches!(
        Realm::default().eval("new Array.prototype.reverse"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    let mut realm = Realm::default();
    realm
        .eval("let f=Array.prototype.reverse;delete Array.prototype.reverse")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("let a=[1,2];f.call(a)===a && a[0]===2 && a[1]===1"),
        Ok(Value::Boolean(true))
    );
}
