//! In-place Array range methods retain ordered conversions and partial effects.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

fn type_error(source: &str) {
    assert!(
        matches!(
            Realm::default().eval(source),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ),
        "{source}"
    );
}

#[test]
fn fill_preserves_value_identity_and_creates_properties_for_holes() {
    check(
        "let a=[1,,,4],o={valueOf:()=>{throw 7;}};a.fill(o)===a && a.length===4 && a[0]===o && a[1]===o && a[2]===o && a[3]===o && Object.hasOwn(a,'1')",
    );
    check(
        "let a=[,,];a.fill();Object.hasOwn(a,'0') && Object.hasOwn(a,'1') && a[0]===undefined && a[1]===undefined",
    );
    check("[1,2].fill(7n)[1]===7n && ['x','y'].fill('\\uD800')[0]==='\\uD800'");
}

#[test]
fn fill_clamps_relative_ranges_and_distinguishes_undefined_end_from_nan() {
    for (args, expected) in [
        ("7", "7,7,7,7"),
        ("7,1,3", "0,7,7,3"),
        ("7,-2", "0,1,7,7"),
        ("7,-3,-1", "0,7,7,3"),
        ("7,-Infinity,Infinity", "7,7,7,7"),
        ("7,Infinity", "0,1,2,3"),
        ("7,1,undefined", "0,7,7,7"),
        ("7,0,NaN", "0,1,2,3"),
        ("7,-0,2.9", "7,7,2,3"),
        ("7,NaN,'2'", "7,7,2,3"),
        ("7,3,1", "0,1,2,3"),
        ("7,-100,-Infinity", "0,1,2,3"),
    ] {
        check(&format!("[0,1,2,3].fill({args}).join()==='{expected}'"));
    }
}

#[test]
fn copy_within_handles_overlap_in_both_directions_and_preserves_holes() {
    for (args, expected) in [
        ("1,0,4", "0,0,1,2,3"),
        ("0,1,4", "1,2,3,3,4"),
        ("-2,0,2", "0,1,2,0,1"),
        ("0,-2", "3,4,2,3,4"),
        ("2,0,-2", "0,1,0,1,2"),
        ("0,3,2", "0,1,2,3,4"),
        ("Infinity,0", "0,1,2,3,4"),
        ("0,Infinity", "0,1,2,3,4"),
        ("-Infinity,3,Infinity", "3,4,2,3,4"),
        ("NaN,'3.9'", "3,4,2,3,4"),
        ("0,1,NaN", "0,1,2,3,4"),
        ("0,1,undefined", "1,2,3,4,4"),
    ] {
        check(&format!(
            "[0,1,2,3,4].copyWithin({args}).join()==='{expected}'"
        ));
    }
    check(
        "let a=[0,,2,3];a.copyWithin(1,0,3)===a && a.length===4 && a[0]===0 && a[1]===0 && !(2 in a) && a[3]===2",
    );
    check("let a=[0,1,,3];a.copyWithin(0,1,3);a[0]===1 && !(1 in a) && !(2 in a) && a[3]===3");
    check("let a=[undefined,,];a.copyWithin(1,0,1);Object.hasOwn(a,'1') && a[1]===undefined");
}

#[test]
fn all_range_arguments_are_coerced_in_order_even_for_empty_ranges() {
    check(
        "let log='',o={};Object.defineProperty(o,'length',{get:()=>{log+='l';return {valueOf:()=>{log+='n';return 0;}};}});Array.prototype.fill.call(o,{}, {valueOf:()=>{log+='s';return Infinity;}},{valueOf:()=>{log+='e';return 0;}});log==='lnse'",
    );
    check(
        "let log='',o={};Object.defineProperty(o,'length',{get:()=>{log+='l';return {valueOf:()=>{log+='n';return 0;}};}});Array.prototype.copyWithin.call(o,{valueOf:()=>{log+='t';return Infinity;}},{valueOf:()=>{log+='s';return 0;}},{valueOf:()=>{log+='e';return 0;}});log==='lntse'",
    );
    for source in [
        "[].fill(7,1n)",
        "[].fill(7,0,1n)",
        "[].copyWithin(1n)",
        "[].copyWithin(0,1n)",
        "[].copyWithin(0,0,1n)",
    ] {
        type_error(source);
    }
    for source in [
        "[].fill(0,{valueOf:()=>{throw 7;}})",
        "[].fill(0,Infinity,{valueOf:()=>{throw 7;}})",
        "[].copyWithin({valueOf:()=>{throw 7;}})",
        "[].copyWithin(Infinity,{valueOf:()=>{throw 7;}})",
        "[].copyWithin(Infinity,Infinity,{valueOf:()=>{throw 7;}})",
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Err(Error::Thrown(Value::Number(7.0)))
        );
    }
}

#[test]
fn captured_length_controls_range_despite_conversion_and_setter_mutations() {
    check(
        "let a=[1,2,3];a.fill(7,{valueOf:()=>{a.length=0;return 0;}});a.join()==='7,7,7' && a.length===3",
    );
    check(
        "let a=[1,2,3];a.fill(7,0,{valueOf:()=>{a[3]=9;return undefined;}});a.join()==='1,2,3,9'",
    );
    check(
        "let a=[1,2,3];Object.defineProperty(a,'0',{set:v=>{a.length=1;}});a.fill(7);a[1]===7 && a[2]===7 && a.length===3",
    );
    check("let a=[1,2,3];a.copyWithin({valueOf:()=>{a[3]=9;return 0;}},1);a.join()==='2,3,3,9'");
    check(
        "let a=[1,2,3];a.copyWithin(0,{valueOf:()=>{a.length=1;return 1;}});!(0 in a) && a.length===1",
    );
    check(
        "let a=[1,2,3,4];Object.defineProperty(a,'1',{get:()=>{a[2]=9;return 2;},set:function(v){}});a.copyWithin(0,1,4);a[0]===2 && a[2]===4",
    );
}

#[test]
fn copy_reads_and_writes_live_in_the_required_direction_without_self_copy_shortcuts() {
    for (args, expected) in [
        ("1,0,3", "g2s3:2;g1s2:1;g0s1:0;"),
        ("0,1,4", "g1s0:1;g2s1:2;g3s2:3;"),
        ("0,0,1", "g0s0:0;"),
    ] {
        check(&format!(
            "let log='',o={{length:4}};function define(k){{Object.defineProperty(o,k,{{get:()=>{{log+='g'+k;return k;}},set:v=>{{log+='s'+k+':'+v+';'}}}});}}define(0);define(1);define(2);define(3);Array.prototype.copyWithin.call(o,{args});log==='{expected}'"
        ));
    }
    check(
        "let a=[1,2,3];Object.defineProperty(a,'0',{get:()=>{delete a[1];return 1;}});a.copyWithin(1,0,2);a[1]===1 && a[2]===2",
    );
    check(
        "let a=[1,2,3,4];Object.defineProperty(a,'2',{get:()=>{delete a[3];return 3;}});a.copyWithin(0,2,4);a[0]===3 && !(1 in a)",
    );
}

#[test]
fn inherited_sources_and_setters_work_and_hole_deletion_preserves_prototypes() {
    check(
        "let p={1:7},o=Object.create(p);o.length=2;Array.prototype.copyWithin.call(o,0,1);o[0]===7 && Object.hasOwn(o,'0') && !Object.hasOwn(o,'1')",
    );
    check(
        "let p={0:9},o=Object.create(p);o[0]=1;o.length=2;Array.prototype.copyWithin.call(o,0,1);!Object.hasOwn(o,'0') && o[0]===9 && p[0]===9",
    );
    check(
        "let value,valid=false,p={};Object.defineProperty(p,'0',{set:function(v){value=v;valid=this===o;}});let o=Object.create(p);o.length=1;Array.prototype.fill.call(o,7);valid && value===7 && !Object.hasOwn(o,'0')",
    );
    check(
        "let value,valid=false,p={};Object.defineProperty(p,'0',{set:function(v){value=v;valid=this===o;}});let o=Object.create(p);o.length=2;o[1]=7;Array.prototype.copyWithin.call(o,0,1);valid && value===7 && !Object.hasOwn(o,'0')",
    );
}

#[test]
fn strict_mutation_failures_retain_only_the_completed_earlier_effects() {
    for (setup, call, expected) in [
        (
            "let a=[1,2,3];Object.defineProperty(a,'1',{writable:false})",
            "a.fill(7)",
            "a.join()==='7,2,3'",
        ),
        (
            "let a=[1,,];Object.preventExtensions(a)",
            "a.fill(7)",
            "a[0]===7 && !(1 in a)",
        ),
        (
            "let a=[1,2,3];Object.defineProperty(a,'1',{writable:false})",
            "a.copyWithin(1,0,2)",
            "a.join()==='1,2,2'",
        ),
        (
            "let a=[1,2,,];Object.defineProperty(a,'1',{configurable:false})",
            "a.copyWithin(0,1,3)",
            "a[0]===2 && a[1]===2 && !(2 in a)",
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
    type_error("Object.freeze([1]).copyWithin(0,0)");
    check("let a=Object.freeze([]);a.fill(7)===a && a.copyWithin(0,0)===a");
    check(
        "let a=[1,2],n=0;Object.defineProperty(a,'1',{set:()=>{n++;throw 7;}});try{a.fill(9);}catch(e){if(e!==7)throw e;}a[0]===9 && n===1",
    );
}

#[test]
fn generic_receivers_keep_length_untouched_and_support_safe_integer_indices() {
    for method in ["fill", "copyWithin"] {
        check(&format!(
            "let o={{0:1,1:2}},n=0;Object.defineProperty(o,'length',{{get:()=>2,set:()=>{{n++;throw 7;}}}});Array.prototype.{method}.call(o,0,1)===o && n===0"
        ));
        check(&format!(
            "Array.prototype.{method}.call(false).valueOf()===false && Array.prototype.{method}.call(7).valueOf()===7 && Array.prototype.{method}.call('').valueOf()===''"
        ));
        type_error(&format!("Array.prototype.{method}.call('aa',0,0)"));
    }
    check(
        "let o={length:Infinity};Array.prototype.fill.call(o,7,-2);o[9007199254740989]===7 && o[9007199254740990]===7 && !(9007199254740991 in o) && o.length===Infinity",
    );
    check(
        "let o={9007199254740989:7,0:1,1:2,length:Infinity};Array.prototype.copyWithin.call(o,0,-2);o[0]===7 && !(1 in o) && o.length===Infinity",
    );
    check(
        "let a=[1,2];Object.defineProperty(a,'length',{writable:false});a.fill(7);a.copyWithin(0,1);a.join()==='7,7'",
    );
}

#[test]
fn receiver_and_length_errors_precede_index_conversion() {
    for method in ["fill", "copyWithin"] {
        for receiver in ["null", "undefined", "{length:1n}"] {
            type_error(&format!("Array.prototype.{method}.call({receiver})"));
        }
        assert_eq!(Realm::default().eval(&format!("Array.prototype.{method}.call({{length:{{valueOf:()=>{{throw 7;}}}}}},0,{{valueOf:()=>{{throw 8;}}}})")),Err(Error::Thrown(Value::Number(7.0))));
    }
    assert_eq!(
        Realm::default()
            .eval("let a=[1];Object.defineProperty(a,'0',{get:()=>{throw 7;}});a.copyWithin(0,0)"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
}

#[test]
fn metadata_nonconstructibility_and_collection_are_standard() {
    for (method, length) in [("fill", 1), ("copyWithin", 2)] {
        check(&format!(
            "let f=Array.prototype.{method},d=Object.getOwnPropertyDescriptor(Array.prototype,'{method}');f.name==='{method}' && f.length==={length} && f.prototype===undefined && d.value===f && d.writable && !d.enumerable && d.configurable"
        ));
        type_error(&format!("new Array.prototype.{method}"));
        let mut realm = Realm::default();
        realm
            .eval(&format!(
                "let f=Array.prototype.{method};delete Array.prototype.{method}"
            ))
            .unwrap();
        realm.collect(10_000).unwrap();
        assert_eq!(
            realm.eval("let a=[];f.call(a,0,0)===a"),
            Ok(Value::Boolean(true))
        );
    }
}
