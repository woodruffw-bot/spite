//! Array construction and exotic length/element behavior (23.1.1–3, 10.4.2).

mod common;
use common::REALM_ENTRIES;
use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn constructors_distinguish_lengths_from_single_elements() {
    check(
        "let a=Array(),b=new Array;a!==b && a.length===0 && b.length===0 && a instanceof Array && Object.getPrototypeOf(a)===Array.prototype",
    );
    check("let a=Array(3);a.length===3 && !('0' in a) && !('1' in a) && !('2' in a)");
    check(
        "let a=new Array(4294967295);a.length===4294967295 && !('4294967294' in a) && Object.is(Array(-0).length,0)",
    );
    for value in [
        "undefined",
        "null",
        "true",
        "'3'",
        "3n",
        "{}",
        "new Number(3)",
    ] {
        check(&format!(
            "let v={value},a=Array(v);a.length===1 && a[0]===v && Object.hasOwn(a,'0')"
        ));
    }
    check(
        "let o={valueOf:()=>{throw 1;},toString:()=>{throw 2;}},a=new Array(o);a[0]===o && a.length===1",
    );
    check(
        "let a=Array(1,undefined,3);a.length===3 && a[0]===1 && a[1]===undefined && Object.hasOwn(a,'1') && a[2]===3",
    );
    check(
        "let n=0;function f(){return ++n;}let a=new Array(f(),f());n===2 && a[0]===1 && a[1]===2",
    );
}

#[test]
fn invalid_numeric_lengths_throw_range_errors() {
    for value in ["-1", "0.5", "NaN", "Infinity", "-Infinity", "4294967296"] {
        for prefix in ["", "new "] {
            let source = format!("{prefix}Array({value})");
            assert!(
                matches!(
                    Realm::default().eval(&source),
                    Err(Error::Exception {
                        kind: ExceptionKind::RangeError,
                        ..
                    })
                ),
                "{source}"
            );
        }
    }
    check("let ok=false;try{Array(0.5);}catch(e){ok=e instanceof RangeError;}ok");
}

#[test]
fn calls_apply_and_bound_construction_preserve_array_identity() {
    check("let a=Array.call({x:1},'x');a[0]==='x' && a.length===1 && a instanceof Array");
    check("let a=Array.apply(null,{0:3,1:4,length:2});a.length===2 && a[0]===3 && a[1]===4");
    check(
        "let F=Array.bind(null,'a');F.prototype={};let a=new F('b');a[0]==='a' && a[1]==='b' && a.length===2 && Object.getPrototypeOf(a)===Array.prototype && a instanceof F",
    );
    check("let F=Array.bind(null,4);new F().length===4 && F().length===4");
}

#[test]
fn constructor_uses_own_data_definitions_without_inherited_setters() {
    check(
        "let n=0;Object.defineProperty(Array.prototype,'0',{set:()=>{n++;},configurable:true});let a=Array('x'),b=Array(1,2);n===0 && a[0]==='x' && b[0]===1 && b[1]===2 && Object.hasOwn(a,'0')",
    );
    check(
        "Object.defineProperty(Array.prototype,'0',{value:9,writable:false,configurable:true});let a=Array(1,2);a[0]===1 && a[1]===2",
    );
    check(
        "Object.defineProperty(Array.prototype,'length',{writable:false});let a=Array(2,3);a.length===2 && a[0]===2 && a[1]===3",
    );
}

#[test]
fn array_brand_survives_prototype_changes_and_rejects_impostors() {
    check(
        "Array.isArray(Array.prototype) && Array.prototype.length===0 && Object.getPrototypeOf(Array.prototype)===Object.prototype",
    );
    check(
        "let a=Array(2);Object.setPrototypeOf(a,null);Array.isArray(a) && Object.prototype.toString.call(a)==='[object Array]'",
    );
    check(
        "!Array.isArray(Object.create(Array.prototype)) && !Array.isArray({length:0}) && !Array.isArray(Array) && !Array.isArray()",
    );
    for value in [
        "undefined",
        "null",
        "1",
        "1n",
        "true",
        "'x'",
        "new String('x')",
        "(function(){return arguments;})()",
    ] {
        check(&format!("!Array.isArray({value})"));
    }
    check("let o={valueOf:()=>{throw 1;},toString:()=>{throw 2;}};!Array.isArray(o)");
    check(
        "Object.prototype.toString.call(Array.prototype)==='[object Array]' && Object.prototype.toString.call(Object.create(Array.prototype))==='[object Object]'",
    );
}

#[test]
fn intrinsic_names_lengths_descriptors_and_key_order_are_exact() {
    check(
        "Array.name==='Array' && Array.length===1 && Array.isArray.name==='isArray' && Array.isArray.length===1 && Array.prototype.constructor===Array && Array.isArray.prototype===undefined",
    );
    check(
        "let d=Object.getOwnPropertyDescriptor(Array,'prototype');d.value===Array.prototype && !d.writable && !d.enumerable && !d.configurable",
    );
    check(
        "let d=Object.getOwnPropertyDescriptor(Array.prototype,'length');d.value===0 && d.writable && !d.enumerable && !d.configurable",
    );
    check(
        "let d=Object.getOwnPropertyDescriptor(Array,'isArray');d.value===Array.isArray && d.writable && !d.enumerable && d.configurable",
    );
    check(
        "let d=Object.getOwnPropertyDescriptor(globalThis,'Array');d.value===Array && d.writable && !d.enumerable && d.configurable",
    );
    check(
        "let d=Object.getOwnPropertyDescriptor(Array(1,2),'0');d.value===1 && d.writable && d.enumerable && d.configurable",
    );
    check(
        "Array.toString()==='function Array() { [native code] }' && Array.isArray.toString()==='function isArray() { [native code] }'",
    );
    assert!(matches!(
        Realm::default().eval("new Array.isArray"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    let mut realm = Realm::default();
    let Value::Object(array) = realm.eval("let a=Array(2);a.x=1;a[10]=3;a[1]=2;a").unwrap() else {
        panic!("array");
    };
    assert_eq!(
        realm.inspect_object(&array).unwrap().own_keys(),
        ["1", "10", "length", "x"].map(JsString::from)
    );
}

#[test]
fn index_growth_readonly_lengths_and_integrity_operations_use_array_rules() {
    check("let a=Array();a[3]=9;a['01']=8;a[4294967295]=7;a.length===4 && a[3]===9 && !('0' in a)");
    check("let a=Array(1,2,3);delete a[2];a.length===3 && !('2' in a)");
    check("let a=Array(1,2,3);a.length='1';a.length===1 && !('1' in a) && !('2' in a)");
    check(
        "let a=Array(1,2);Object.freeze(a);a[0]=8;a[2]=9;a.length=0;Object.isFrozen(a) && a[0]===1 && a.length===2 && !('2' in a)",
    );
    check(
        "let a=Array(1,2);Object.seal(a);a[0]=8;a.length=0;Object.isSealed(a) && a[0]===8 && a.length===2",
    );
    check(
        "let a=Array(1);Object.preventExtensions(a);a.length=5;a[0]=8;!Object.isExtensible(a) && a.length===5 && !('0' in a)",
    );
}

#[test]
fn missing_array_intrinsics_remain_explicit_host_gaps() {
    for source in [
        "Array.from",
        "Array.fromAsync",
        "Array().map",
        "Array().values",
        "Object.getOwnPropertyDescriptor(Array.prototype,'map')",
        "Object.defineProperty(Array,'from',{})",
        "Object.freeze(Array.prototype)",
        "Object.getOwnPropertyDescriptors(Array)",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check("let a=Array();a.join=7;a.join===7 && a.random===undefined");
}

#[test]
fn explicit_collection_traces_arrays_elements_and_intrinsic_roots() {
    let mut realm = Realm::default();
    let value = realm.eval("Array({x:1})").unwrap();
    let root = realm.root_value(value, 100).unwrap();
    assert_eq!(realm.collect(10_000).unwrap().live, REALM_ENTRIES + 2);
    drop(root);
    assert_eq!(realm.collect(10_000).unwrap().live, REALM_ENTRIES);
    realm
        .eval("let a=Array({x:1});delete globalThis.Array")
        .unwrap();
    realm.collect(10_000).unwrap();
    assert_eq!(
        realm.eval("a.constructor.isArray(a) && a[0].x===1"),
        Ok(Value::Boolean(true))
    );
    realm.eval("a.length=0").unwrap();
    assert_eq!(realm.collect(10_000).unwrap().live, REALM_ENTRIES + 1);
    realm.eval("a=null").unwrap();
    assert_eq!(realm.collect(10_000).unwrap().live, REALM_ENTRIES);
}

#[test]
fn array_allocation_limits_skip_javascript_handlers() {
    let mut realm = Realm::new(Limits {
        max_heap_entries: REALM_ENTRIES,
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval("try{Array();}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
