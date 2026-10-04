//! Stable sorting, comparison conversions, sparse writes, and dense copies.

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn default_comparison_uses_utf16_strings_and_never_compares_undefined() {
    for method in ["sort", "toSorted"] {
        check(&format!(
            "[3,20,1,undefined,11,null,true].{method}().join()==='1,11,20,3,,true,'"
        ));
        check(&format!(
            "['\\uE000','💩','\\uD800','a'].{method}().join()==='a,\\uD800,💩,\\uE000'"
        ));
        check(&format!("[2n,11n,1n].{method}().join()==='1,11,2'"));
        check(&format!(
            "let n=0,a=[undefined,3,1,undefined,2].{method}((x,y)=>{{if(x===undefined||y===undefined)throw 7;n++;return x-y;}});n>0 && a.join()==='1,2,3,,'"
        ));
        check(&format!(
            "let n=0,o={{toString:()=>{{n++;return 'x';}}}};[o,o].{method}();n===2"
        ));
        check(&format!(
            "let o={{toString:()=>{{throw 7;}}}};[o].{method}()[0]===o && [].{method}(()=>{{throw 8;}}).length===0"
        ));
    }
}

#[test]
fn stable_comparisons_preserve_ties_and_normalize_nan_and_signed_zero() {
    for method in ["sort", "toSorted"] {
        for result in [
            "0",
            "-0",
            "NaN",
            "undefined",
            "null",
            "false",
            "({valueOf:()=>NaN})",
        ] {
            check(&format!(
                "let a={{}},b={{}},c={{}},v=[a,b,c].{method}(()=>{result});v[0]===a && v[1]===b && v[2]===c"
            ));
        }
        check(&format!(
            "let a=[{{k:2,n:'a'}},{{k:1,n:'b'}},{{k:2,n:'c'}},{{k:1,n:'d'}},{{k:2,n:'e'}}];a.{method}((x,y)=>x.k-y.k).reduce((s,x)=>s+x.n,'')==='bdace'"
        ));
        check(&format!(
            "let seen=false;[3,1,2].{method}(function(x,y){{'use strict';seen=true;if(this!==undefined||arguments.length!==2)throw 7;return {{valueOf:()=>x-y}};}}).join()==='1,2,3' && seen"
        ));
    }
}

#[test]
fn odd_and_even_merge_ranges_preserve_every_value_in_stable_order() {
    for length in [0, 1, 2, 3, 5, 7, 8, 9, 15, 16, 17, 31, 32, 33, 63, 64, 65] {
        // Budget includes the scripted stability/permutation audit as well as sorting.
        let mut realm = Realm::new(Limits {
            max_steps: 1_000_000,
            ..Limits::default()
        });
        assert_eq!(realm.eval(&format!(
            "let a=[];for(let i=0;i<{length};i++)a.push({{key:(i*7)%5,id:i}});let b=a.toSorted((x,y)=>x.key-y.key),ok=b.length===a.length;for(let i=1;i<b.length;i++){{if(b[i-1].key>b[i].key||(b[i-1].key===b[i].key&&b[i-1].id>=b[i].id))ok=false;}}for(let i=0;i<b.length;i++){{if(b[i]!==a[b[i].id])ok=false;}}ok"
        )), Ok(Value::Boolean(true)));
    }
}

#[test]
fn sort_preserves_holes_while_to_sorted_defines_dense_intrinsic_elements() {
    check(
        "let a=[,3,undefined,,1],b=a.sort();b===a && a.length===5 && a[0]===1 && a[1]===3 && Object.hasOwn(a,'2') && a[2]===undefined && !Object.hasOwn(a,'3') && !Object.hasOwn(a,'4')",
    );
    check(
        "let a=[,3,undefined,,1],b=a.toSorted();b!==a && Array.isArray(b) && b.length===5 && b.join()==='1,3,,,' && Object.hasOwn(b,'3') && Object.hasOwn(b,'4') && !Object.hasOwn(a,'0') && a[1]===3",
    );
    check("Array.prototype[1]=2;let a=[3,,1];a.sort();a.join()==='1,2,3' && Object.hasOwn(a,'1')");
}

#[test]
fn to_sorted_ignores_constructors_and_bypasses_output_setters() {
    check(
        "let a=Object.freeze([3,1]),p=Array.prototype;Array=function(){throw 7;};let b=a.toSorted();b.join()==='1,3' && Object.getPrototypeOf(b)===p",
    );
    check(
        "let a=[3,1];Object.defineProperty(a,'constructor',{get:()=>{throw 7;}});Object.defineProperty(Array.prototype,'0',{set:()=>{throw 8;}});let b=a.toSorted(),d=Object.getOwnPropertyDescriptor(b,'0');d.value===1 && d.writable && d.enumerable && d.configurable",
    );
}

#[test]
fn reads_finish_before_comparisons_and_source_length_is_captured_once() {
    for method in ["sort", "toSorted"] {
        check(&format!(
            "let log='',o={{length:3}},data=[3,2,1];for(let i=0;i<3;i++)Object.defineProperty(o,i,{{get:()=>{{log+=i;return data[i];}},set:v=>{{log+='s';data[i]=v;}}}});let n=0,b=Array.prototype.{method}.call(o,(x,y)=>{{if(n===0&&log!=='012')throw 7;n++;return x-y;}});n>0 && b[0]===1"
        ));
    }
    check(
        "let a=[3,2,1];Object.defineProperty(a,'0',{get:()=>{delete a[1];a[3]=9;return 3;},set:()=>{}});let b=a.toSorted();b.length===3 && b[0]===1 && b[1]===3 && b[2]===undefined && a.length===4",
    );
    check(
        "let a=[3,2,1];Object.defineProperty(a,'0',{get:()=>{delete a[1];return 3;},set:()=>{}});a.sort();a[1]===3 && !Object.hasOwn(a,'2')",
    );
    check(
        "let a=[3,2,1],n=0;let b=a.toSorted((x,y)=>{if(n++===0){a[0]=9;a.length=0;}return x-y;});b.join()==='1,2,3' && a.length===0",
    );
    check(
        "let a=[3,2,1],n=0;a.sort((x,y)=>{if(n++===0)a.length=0;return x-y;});a.join()==='1,2,3'",
    );
}

#[test]
fn invalid_comparators_precede_length_and_collection_errors_precede_callbacks() {
    for method in ["sort", "toSorted"] {
        for cmp in ["null", "7", "{}", "'x'"] {
            check(&format!(
                "let n=0,caught=false,o={{}};Object.defineProperty(o,'length',{{get:()=>{{n++;throw 7;}}}});try{{Array.prototype.{method}.call(o,{cmp});}}catch(e){{caught=e instanceof TypeError;}}caught && n===0"
            ));
        }
        check(&format!(
            "let n=0,caught=false,a=[3,2,1];Object.defineProperty(a,'2',{{get:()=>{{throw 7;}}}});try{{a.{method}(()=>{{n++;return 0;}});}}catch(e){{caught=e===7;}}caught && n===0 && a[0]===3 && a[1]===2"
        ));
        assert!(matches!(
            Realm::default().eval(&format!("[2,1].{method}(()=>0n)")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        assert!(matches!(
            Realm::default().eval(&format!("Array.prototype.{method}.call({{length:1n}})")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        check(&format!(
            "let n=0,a=[3,2,1],caught=false;try{{a.{method}(()=>{{n++;throw 7;}});}}catch(e){{caught=e===7;}}caught && n===1 && a.join()==='3,2,1'"
        ));
    }
    check(
        "let n=0,caught=false,o={length:4294967296};Object.defineProperty(o,'0',{get:()=>{n++;throw 7;}});try{Array.prototype.toSorted.call(o);}catch(e){caught=e instanceof RangeError;}caught && n===0",
    );
}

#[test]
fn sort_write_and_delete_failures_leave_earlier_effects_and_never_set_length() {
    check(
        "let a=[3,2,1];Object.defineProperty(a,'1',{writable:false});let caught=false;try{a.sort();}catch(e){caught=e instanceof TypeError;}caught && a[0]===1 && a[1]===2 && a[2]===1",
    );
    check(
        "let a=[3,,1];Object.defineProperty(a,'2',{configurable:false});let caught=false;try{a.sort();}catch(e){caught=e instanceof TypeError;}caught && a[0]===1 && a[1]===3 && a[2]===1",
    );
    check(
        "let o={0:2,1:1};Object.defineProperty(o,'length',{get:()=>2,set:()=>{throw 7;}});Array.prototype.sort.call(o)===o && o[0]===1 && o[1]===2",
    );
    check(
        "let a=[2,1];Object.defineProperty(a,'length',{writable:false});a.sort()===a && a.join()==='1,2'",
    );
    check(
        "let a=[2,1],n=0;Object.defineProperty(a,'1',{get:()=>1,set:()=>{n++;throw 7;}});let caught=false;try{a.sort();}catch(e){caught=e===7;}caught && n===1 && a[0]===1",
    );
    check("Object.freeze([]).sort().length===0");
    assert!(matches!(
        Realm::default().eval("Object.freeze([1]).sort()"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn generic_receivers_and_method_metadata_are_standard() {
    check("Array.prototype.toSorted.call('ba').join()==='a,b'");
    check(
        "Array.prototype.sort.call(false) instanceof Boolean && Array.prototype.sort.call(7) instanceof Number",
    );
    assert!(matches!(
        Realm::default().eval("Array.prototype.sort.call('ba')"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    for method in ["sort", "toSorted"] {
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
        realm.eval(&format!("let f=Array.prototype.{method};delete Array.prototype.{method};let o={{}},a=[o],b=f.call(a);delete a[0]")).unwrap();
        realm.collect(10_000).unwrap();
        assert_eq!(
            realm.eval("f.call([2,1]).join()==='1,2'"),
            Ok(Value::Boolean(true))
        );
    }
}
