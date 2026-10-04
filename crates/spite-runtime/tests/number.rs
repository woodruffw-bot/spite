//! Number conversion, wrapper identity, predicates, and decimal formatting.

mod common;
use common::REALM_ENTRIES;

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn constructor_distinguishes_absence_from_undefined_and_preserves_number_bits() {
    for (source, expected) in [
        ("Number()", 0.0),
        ("Number(null)", 0.0),
        ("Number(false)", 0.0),
        ("Number(true)", 1.0),
        ("Number('  12.5  ')", 12.5),
        ("Number('0xff')", 255.0),
        ("Number(-0)", -0.0),
        ("Number('-0')", -0.0),
        ("Number(Infinity)", f64::INFINITY),
        ("Number(-Infinity)", f64::NEG_INFINITY),
    ] {
        let Value::Number(value) = Realm::default().eval(source).unwrap() else {
            panic!("Number")
        };
        assert_eq!(value.to_bits(), expected.to_bits(), "{source}");
    }
    check(
        "Number.isNaN(Number(undefined)) && Number.isNaN(Number('bad')) && Number.isNaN(Number(NaN))",
    );
    check("let count=0; Number('4',count=1)===4 && count===1 && Number.call(null,'3')===3");
}

#[test]
fn explicit_bigint_conversion_rounds_once_and_does_not_change_implicit_conversion() {
    check(
        "Number(9007199254740993n)===9007199254740992 && Number(9007199254740995n)===9007199254740996 && Number(-9007199254740995n)===-9007199254740996",
    );
    check("Number(2n**1024n)===Infinity && Number(-(2n**1024n))===-Infinity && Number(0n)===0");
    check("Number({valueOf:()=>9007199254740993n})===9007199254740992");
    for source in ["+1n", "+{valueOf:()=>1n}", "1n-1"] {
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
}

#[test]
fn object_conversion_calls_valueof_then_tostring_and_propagates_abrupt_results() {
    check(
        "let order=''; let value=Number({valueOf:()=>{order+='v';return {};},toString:()=>{order+='s';return '7';}}); value===7 && order==='vs'",
    );
    check(
        "let called=false; new Number({valueOf:()=>3,toString:()=>{called=true;return '4';}}).valueOf()===3 && !called",
    );
    let mut realm = Realm::default();
    realm.eval("let flag=0").unwrap();
    assert_eq!(
        realm.eval("new Number({valueOf:()=>{flag=1;throw 9;},toString:()=>{flag=2;return '1';}})"),
        Err(Error::Thrown(Value::Number(9.0)))
    );
    assert_eq!(realm.eval("flag"), Ok(Value::Number(1.0)));
    assert!(matches!(
        realm.eval("Number({valueOf:()=>({}),toString:()=>({})})"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn wrappers_have_own_slots_and_bound_construction_ignores_bound_this() {
    check(
        "let a=new Number(-0), b=new Number; a!==b && a instanceof Number && a.constructor===Number && 1/a.valueOf()===-Infinity && 1/b.valueOf()===Infinity && 1/Number.prototype.valueOf()===Infinity",
    );
    check(
        "Number.isNaN(new Number(undefined).valueOf()) && Number.isNaN(new Number(NaN).valueOf())",
    );
    check(
        "let N=Number.bind({},'7'); N()===7 && new N(8).valueOf()===7 && new N instanceof N && new N instanceof Number",
    );
    check(
        "let n=new Number(2); +n===2 && n+3===5 && `${n}`==='2' && n==2 && n!==2 && !!new Number(0)",
    );
    check(
        "({}).toString.call(new Number)==='[object Number]' && ({}).toString.call({__proto__:Number.prototype})==='[object Object]'",
    );
    for receiver in [
        "undefined",
        "null",
        "true",
        "'1'",
        "1n",
        "{}",
        "({__proto__:Number.prototype})",
        "new Boolean(true)",
    ] {
        for method in ["valueOf", "toString"] {
            let source = format!("Number.prototype.{method}.call({receiver})");
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
    }
}

#[test]
fn non_strict_receivers_box_and_primitive_data_writes_do_not_persist() {
    check(
        "function f(){return this;} let a=f.call(-0),b=f.call(-0); a!==b && a instanceof Number && 1/a.valueOf()===-Infinity",
    );
    check("function f(){'use strict';return this;} f.call(7)===7 && 1/f.call(-0)===-Infinity");
    check("let a=({}).valueOf.call(NaN); a instanceof Number && Number.isNaN(a.valueOf())");
    check(
        "Number.prototype.extra=8; (1).extra===8 && (2).extra===8 && delete (1).extra && (2).extra===8",
    );
    check(
        "Number.prototype.extra=8; (1).extra=9; (1).other=9; (1).extra===8 && (1).other===undefined",
    );
    assert!(matches!(
        Realm::default().eval("'use strict'; (1).extra=9"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    check(
        "let N=Number; delete Number; typeof Number==='undefined' && (1).constructor===N && ({}).valueOf.call(2) instanceof N",
    );
}

#[test]
fn numeric_predicates_never_coerce_and_distinguish_integer_boundaries() {
    for method in ["isFinite", "isNaN", "isInteger", "isSafeInteger"] {
        for value in [
            "undefined",
            "null",
            "false",
            "'1'",
            "1n",
            "new Number(1)",
            "({valueOf:()=>{throw 1;}})",
        ] {
            check(&format!("Number.{method}({value})===false"));
        }
        check(&format!("Number.{method}()===false"));
    }
    check("Number.isNaN(NaN) && !Number.isNaN(Infinity) && !Number.isNaN(0)");
    check(
        "Number.isFinite(0) && Number.isFinite(Number.MAX_VALUE) && !Number.isFinite(NaN) && !Number.isFinite(Infinity) && !Number.isFinite(-Infinity)",
    );
    check(
        "Number.isInteger(-0) && Number.isInteger(-7) && Number.isInteger(Number.MAX_VALUE) && !Number.isInteger(0.5) && !Number.isInteger(Number.MIN_VALUE) && !Number.isInteger(NaN) && !Number.isInteger(Infinity)",
    );
    check(
        "Number.isSafeInteger(-0) && Number.isSafeInteger(Number.MAX_SAFE_INTEGER) && Number.isSafeInteger(Number.MIN_SAFE_INTEGER) && !Number.isSafeInteger(9007199254740992) && !Number.isSafeInteger(-9007199254740992) && !Number.isSafeInteger(0.5)",
    );
}

#[test]
fn decimal_formatting_and_radix_validation_follow_specification_order() {
    for (value, expected) in [
        ("-0", "0"),
        ("NaN", "NaN"),
        ("Infinity", "Infinity"),
        ("-Infinity", "-Infinity"),
        ("1e-7", "1e-7"),
        ("1e-6", "0.000001"),
        ("1e21", "1e+21"),
        ("123.5", "123.5"),
    ] {
        check(&format!(
            "({value}).toString()==='{expected}' && new Number({value}).toString(10)==='{expected}'"
        ));
    }
    check(
        "(12).toString('10')==='12' && (12).toString(10.9)==='12' && (12).toString(undefined)==='12'",
    );
    check(
        "NaN.toString(2)==='NaN' && Infinity.toString(36)==='Infinity' && (-0).toString(2)==='0'",
    );
    for radix in ["null", "false", "NaN", "-0", "1", "37", "Infinity", "'bad'"] {
        for value in ["1", "NaN", "Infinity"] {
            let source = format!("({value}).toString({radix})");
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
    let mut realm = Realm::default();
    realm
        .eval("let flag=0; let radix={valueOf:()=>{flag=1;return 10;}};")
        .unwrap();
    assert!(matches!(
        realm.eval("Number.prototype.toString.call({},radix)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(
        realm.eval("(4).toString(radix)"),
        Ok(Value::String(JsString::from("4")))
    );
    assert_eq!(realm.eval("flag"), Ok(Value::Number(1.0)));
    assert!(matches!(
        realm.eval("(4).toString(10n)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn constants_and_builtin_metadata_have_standard_values_and_attributes() {
    let mut realm = Realm::default();
    let Value::Object(constructor) = realm.eval("Number").unwrap() else {
        panic!("constructor")
    };
    let object = realm.inspect_object(&constructor).unwrap();
    assert!(object.is_constructor());
    for (name, expected) in [
        ("EPSILON", f64::EPSILON),
        ("MAX_VALUE", f64::MAX),
        ("MIN_VALUE", f64::from_bits(1)),
        ("MAX_SAFE_INTEGER", 9_007_199_254_740_991.0),
        ("MIN_SAFE_INTEGER", -9_007_199_254_740_991.0),
        ("NaN", f64::NAN),
        ("POSITIVE_INFINITY", f64::INFINITY),
        ("NEGATIVE_INFINITY", f64::NEG_INFINITY),
    ] {
        let property = object
            .own_property(&JsString::from(name))
            .unwrap()
            .as_data()
            .unwrap();
        assert!(
            property.value.same_value(&Value::Number(expected)),
            "{name}"
        );
        assert!(!property.writable && !property.enumerable && !property.configurable);
    }
    let prototype = object
        .own_property(&JsString::from("prototype"))
        .unwrap()
        .as_data()
        .unwrap();
    assert!(!prototype.writable && !prototype.enumerable && !prototype.configurable);
    for (method, length) in [
        ("Number", 1.0),
        ("Number.prototype.valueOf", 0.0),
        ("Number.prototype.toString", 1.0),
        ("Number.isFinite", 1.0),
        ("Number.isNaN", 1.0),
        ("Number.isInteger", 1.0),
        ("Number.isSafeInteger", 1.0),
    ] {
        let Value::Object(handle) = realm.eval(method).unwrap() else {
            panic!("function")
        };
        let object = realm.inspect_object(&handle).unwrap();
        assert_eq!(object.is_constructor(), method == "Number");
        for (key, value) in [
            ("length", Value::Number(length)),
            (
                "name",
                Value::String(JsString::from(method.rsplit('.').next().unwrap())),
            ),
        ] {
            let property = object
                .own_property(&JsString::from(key))
                .unwrap()
                .as_data()
                .unwrap();
            assert_eq!(property.value, value);
            assert!(!property.writable && !property.enumerable && property.configurable);
        }
    }
    check("Number.MAX_VALUE=0; Number.MAX_VALUE>0 && !delete Number.MAX_VALUE");
}

#[test]
fn incomplete_formatting_paths_remain_visible_host_gaps() {
    for source in [
        "(1).toString(2)",
        "new Number(0.5).toString(16)",
        "Number.parseInt",
        "Number.parseFloat",
        "Number.prototype.toFixed",
        "(1).toPrecision",
    ] {
        let mut realm = Realm::default();
        realm.eval("let flag=0").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "try{{{source};}}catch{{flag=1;}}finally{{flag=2;}}"
                )),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
    check("'toFixed' in Number.prototype && 'parseInt' in Number");
}

#[test]
fn escaped_numeric_receivers_survive_collection_and_then_release() {
    let mut realm = Realm::default();
    realm
        .eval("let f=(function(){return ()=>this;}).call(-0)")
        .unwrap();
    realm.collect(10_000).unwrap();
    assert_eq!(
        realm.eval("1/f().valueOf()"),
        Ok(Value::Number(f64::NEG_INFINITY))
    );
    realm.eval("f=null").unwrap();
    assert_eq!(realm.collect(10_000).unwrap().live, REALM_ENTRIES);
}
