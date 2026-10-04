//! Direct binary32/binary16 rounding, coercion, and intrinsic lifetime.

mod common;
use common::REALM_ENTRIES;
use spite_runtime::{Error, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn special_values_and_signed_underflow_are_preserved() {
    for method in ["fround", "f16round"] {
        check(&format!(
            "Number.isNaN(Math.{method}(NaN)) && Number.isNaN(Math.{method}()) && Number.isNaN(Math.{method}(undefined)) && Math.{method}(Infinity)===Infinity && Math.{method}(-Infinity)===-Infinity && Object.is(Math.{method}(0),0) && Object.is(Math.{method}(-0),-0) && Object.is(Math.{method}(Number.MIN_VALUE),0) && Object.is(Math.{method}(-Number.MIN_VALUE),-0) && Math.{method}(Number.MAX_VALUE)===Infinity && Math.{method}(-Number.MAX_VALUE)===-Infinity"
        ));
    }
}

#[test]
fn binary32_uses_even_halfway_ties_and_handles_range_boundaries() {
    check(
        "Math.fround(1+2**-24)===1 && Math.fround(1+3*2**-24)===1+2**-22 && Math.fround(1+2**-24+Number.EPSILON)===1+2**-23 && Math.fround(1+2**-24-Number.EPSILON)===1 && Math.fround(-1-3*2**-24)===-1-2**-22",
    );
    check(
        "Math.fround(0.1)===0.10000000149011612 && Math.fround(4294967295)===4294967296 && Math.fround(2**-149)===2**-149 && Object.is(Math.fround(2**-150),0) && Object.is(Math.fround(-(2**-150)),-0) && Math.fround(3*2**-150)===2**-148 && Math.fround(2**-126-2**-150)===2**-126",
    );
    check(
        "Math.fround(3.4028234663852886e38)===3.4028234663852886e38 && Math.fround(3.4028235677973366e38)===Infinity && Math.fround(-3.4028235677973366e38)===-Infinity && Math.fround(3.4028235677973362e38)===3.4028234663852886e38",
    );
}

#[test]
fn binary16_rounds_directly_without_a_binary32_intermediate() {
    check(
        "Math.f16round(0.1)===0.0999755859375 && Math.f16round(1.1)===1.099609375 && Math.f16round(2049)===2048 && Math.f16round(2051)===2052 && Math.f16round(-2049)===-2048 && Math.f16round(-2051)===-2052",
    );
    check(
        "let mid=1+2**-11;Math.f16round(mid)===1 && Math.f16round(mid+Number.EPSILON)===1+2**-10 && Math.f16round(mid-Number.EPSILON)===1 && Math.f16round(Math.fround(mid+Number.EPSILON))===1 && Math.f16round(-mid-Number.EPSILON)===-1-2**-10 && Math.f16round(1+3*2**-11)===1+2**-9",
    );
    check(
        "Math.f16round(2**-24)===2**-24 && Object.is(Math.f16round(2**-25),0) && Object.is(Math.f16round(-(2**-25)),-0) && Math.f16round(2.980232238769532e-8)===2**-24 && Math.f16round(3*2**-25)===2**-23 && Math.f16round(5*2**-25)===2**-23 && Math.f16round(1.490116119384766e-7)===3*2**-24",
    );
    check(
        "Math.f16round(2**-14)===2**-14 && Math.f16round(2**-14-2**-25)===2**-14 && Math.f16round(0.0000610053539276123)===0.00006097555160522461 && Math.f16round(65504)===65504 && Math.f16round(65519.99999999999)===65504 && Math.f16round(65520)===Infinity && Math.f16round(-65520)===-Infinity",
    );
}

#[test]
fn numeric_conversion_is_once_and_ignores_receiver_and_extra_arguments() {
    for method in ["fround", "f16round"] {
        check(&format!(
            "let calls=0,o={{[Symbol.toPrimitive](hint){{if(hint!=='number')throw 7;calls++;return 1;}},valueOf(){{throw 8;}}}};Math.{method}.call(Symbol(),o,{{valueOf(){{throw 9;}}}})===1 && calls===1 && Math.{method}('1')===1 && Math.{method}(null)===0 && Math.{method}(true)===1"
        ));
        check(&format!(
            "let caught=false;try{{Math.{method}({{valueOf(){{throw 7;}}}});}}catch(e){{caught=e===7;}}caught"
        ));
        for argument in ["1n", "Object(1n)", "Symbol()", "Object(Symbol())"] {
            check(&format!(
                "let caught=false;try{{Math.{method}({argument});}}catch(e){{caught=e instanceof TypeError;}}caught"
            ));
        }
        let mut realm = Realm::default();
        realm.eval("let flag=0;").unwrap();
        assert!(matches!(
            realm.eval(&format!(
                "try{{Math.{method}({{valueOf(){{Proxy;}}}});}}catch{{flag=1;}}finally{{flag=2;}}"
            )),
            Err(Error::Unsupported { .. })
        ));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
}

#[test]
fn standard_metadata_and_intrinsic_retention_survive_public_deletion() {
    for method in ["fround", "f16round"] {
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(Math,'{method}'),n=Object.getOwnPropertyDescriptor(d.value,'name'),l=Object.getOwnPropertyDescriptor(d.value,'length');d.writable && !d.enumerable && d.configurable && n.value==='{method}' && !n.writable && !n.enumerable && n.configurable && l.value===1 && !l.writable && !l.enumerable && l.configurable && !Object.hasOwn(d.value,'prototype')"
        ));
        check(&format!(
            "let caught=false;try{{new Math.{method}();}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    let mut realm = Realm::default();
    realm.eval("let fround=Math.fround,f16round=Math.f16round;delete Math.fround;delete Math.f16round;delete globalThis.Math;").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(realm.eval("fround(0.1)===0.10000000149011612 && f16round(0.1)===0.0999755859375 && typeof Math==='undefined'"),Ok(Value::Boolean(true)));
}
