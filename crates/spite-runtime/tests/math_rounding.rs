//! Math constants and basic rounding: coercion, IEEE special values, and ties.

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
fn fixed_constants_and_math_object_have_standard_descriptors_and_tag() {
    check(
        "typeof Math==='object' && Object.getPrototypeOf(Math)===Object.prototype && Math.toString()==='[object Math]' && Math.E===2.718281828459045 && Math.LN10===2.302585092994046 && Math.LN2===0.6931471805599453 && Math.LOG10E===0.4342944819032518 && Math.LOG2E===1.4426950408889634 && Math.PI===3.141592653589793 && Math.SQRT1_2===0.7071067811865476 && Math.SQRT2===1.4142135623730951",
    );
    for name in [
        "E", "LN10", "LN2", "LOG10E", "LOG2E", "PI", "SQRT1_2", "SQRT2",
    ] {
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(Math,'{name}');typeof d.value==='number' && !d.writable && !d.enumerable && !d.configurable && Reflect.set(Math,'{name}',7)===false && Reflect.deleteProperty(Math,'{name}')===false"
        ));
    }
    check(
        "let d=Object.getOwnPropertyDescriptor(globalThis,'Math'),t=Object.getOwnPropertyDescriptor(Math,Symbol.toStringTag);d.value===Math && d.writable && !d.enumerable && d.configurable && t.value==='Math' && !t.writable && !t.enumerable && t.configurable",
    );
    check(
        "'use strict';let caught=false;try{Math.PI=7;}catch(e){caught=e instanceof TypeError;}caught && Math.PI===3.141592653589793",
    );
    check(
        "let a=false,b=false;try{Math();}catch(e){a=e instanceof TypeError;}try{new Math();}catch(e){b=e instanceof TypeError;}a && b",
    );
}

#[test]
fn abs_and_sign_preserve_required_nan_infinity_and_zero_results() {
    check(
        "Number.isNaN(Math.abs(NaN)) && Number.isNaN(Math.sign(NaN)) && Object.is(Math.abs(-0),0) && Object.is(Math.abs(0),0) && Object.is(Math.sign(-0),-0) && Object.is(Math.sign(0),0)",
    );
    check(
        "Math.abs(-Infinity)===Infinity && Math.abs(Infinity)===Infinity && Math.sign(-Infinity)===-1 && Math.sign(Infinity)===1 && Math.abs(-Number.MIN_VALUE)===Number.MIN_VALUE && Math.sign(-Number.MIN_VALUE)===-1 && Math.sign(Number.MIN_VALUE)===1",
    );
    check(
        "Math.abs(-3.5)===3.5 && Math.abs(3.5)===3.5 && Math.sign(-3.5)===-1 && Math.sign(3.5)===1",
    );
}

#[test]
fn ceil_floor_and_trunc_retain_signs_at_zero_and_exact_integral_values() {
    for method in ["ceil", "floor", "trunc"] {
        check(&format!(
            "Number.isNaN(Math.{method}(NaN)) && Math.{method}(Infinity)===Infinity && Math.{method}(-Infinity)===-Infinity && Object.is(Math.{method}(0),0) && Object.is(Math.{method}(-0),-0) && Math.{method}(Number.MAX_VALUE)===Number.MAX_VALUE && Math.{method}(-Number.MAX_VALUE)===-Number.MAX_VALUE"
        ));
    }
    check(
        "Object.is(Math.ceil(-Number.MIN_VALUE),-0) && Object.is(Math.ceil(-0.9),-0) && Math.ceil(0.1)===1 && Math.floor(-0.1)===-1 && Object.is(Math.floor(Number.MIN_VALUE),0) && Object.is(Math.trunc(-0.9),-0) && Object.is(Math.trunc(0.9),0)",
    );
    check(
        "Math.ceil(1.1)===2 && Math.ceil(-1.1)===-1 && Math.floor(1.1)===1 && Math.floor(-1.1)===-2 && Math.trunc(1.9)===1 && Math.trunc(-1.9)===-1",
    );
}

#[test]
fn round_uses_positive_halfway_ties_and_preserves_negative_zero() {
    check(
        "Object.is(Math.round(-0.5),-0) && Object.is(Math.round(-0.1),-0) && Object.is(Math.round(-Number.MIN_VALUE),-0) && Object.is(Math.round(-0),-0) && Object.is(Math.round(0),0) && Math.round(0.5)===1 && Math.round(-1.5)===-1 && Math.round(1.5)===2",
    );
    check(
        "Number.isNaN(Math.round(NaN)) && Math.round(Infinity)===Infinity && Math.round(-Infinity)===-Infinity && Math.round(Number.MAX_VALUE)===Number.MAX_VALUE && Math.round(-Number.MAX_VALUE)===-Number.MAX_VALUE",
    );
    check(
        "Object.is(Math.round(0.5-Number.EPSILON/4),0) && Math.round(0.5+Number.EPSILON/2)===1 && Math.round(-0.5-Number.EPSILON/2)===-1 && Object.is(Math.round(-0.5+Number.EPSILON/4),-0)",
    );
    check(
        "Math.round(4503599627370497)===4503599627370497 && Math.round(-4503599627370497)===-4503599627370497 && Math.round(9007199254740991)===9007199254740991 && Math.round(-9007199254740991)===-9007199254740991",
    );
    check(
        "let good=true;for(let i=-4096;i<=4096;i++){if(Math.round(i+0.25)!==i || Math.round(i+0.5)!==i+1 || Math.round(i+0.75)!==i+1)good=false;}good",
    );
}

#[test]
fn unary_operations_convert_once_ignore_receiver_and_skip_extra_argument_conversion() {
    for method in ["abs", "ceil", "floor", "round", "sign", "trunc"] {
        let expected = if method == "abs" { 1 } else { -1 };
        check(&format!(
            "let calls=0,o={{[Symbol.toPrimitive](hint){{if(hint!=='number')throw 7;calls++;return -1;}},valueOf(){{throw 8;}}}};Math.{method}.call(null,o,{{valueOf(){{throw 9;}}}})==={expected} && calls===1"
        ));
    }
}

#[test]
fn invalid_numeric_conversions_throw_and_host_gaps_skip_handlers() {
    for method in ["abs", "ceil", "floor", "round", "sign", "trunc"] {
        for argument in ["1n", "Object(1n)", "Symbol()", "Object(Symbol())"] {
            check(&format!(
                "let caught=false;try{{Math.{method}({argument});}}catch(e){{caught=e instanceof TypeError;}}caught"
            ));
        }
        check(&format!(
            "let caught=false;try{{Math.{method}({{valueOf(){{throw 7;}}}});}}catch(e){{caught=e===7;}}caught"
        ));
        let expected = if method == "sign" { 1 } else { 2 };
        check(&format!(
            "Number.isNaN(Math.{method}()) && Number.isNaN(Math.{method}(undefined)) && Math.{method}(null)===0 && Math.{method}(true)===1 && Math.{method}('2')==={expected}"
        ));
        let mut realm = Realm::default();
        realm.eval("let flag=0").unwrap();
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
fn method_metadata_and_intrinsic_retention_survive_public_deletion() {
    for method in ["abs", "ceil", "floor", "round", "sign", "trunc"] {
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(Math,'{method}'),n=Object.getOwnPropertyDescriptor(d.value,'name'),l=Object.getOwnPropertyDescriptor(d.value,'length');d.writable && !d.enumerable && d.configurable && n.value==='{method}' && !n.writable && !n.enumerable && n.configurable && l.value===1 && !l.writable && !l.enumerable && l.configurable && !Object.hasOwn(d.value,'prototype')"
        ));
        check(&format!(
            "let caught=false;try{{new Math.{method}();}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    let mut realm = Realm::default();
    realm.eval("let abs=Math.abs,ceil=Math.ceil,floor=Math.floor,round=Math.round,sign=Math.sign,trunc=Math.trunc;delete Math.abs;delete Math.ceil;delete Math.floor;delete Math.round;delete Math.sign;delete Math.trunc;delete globalThis.Math").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(realm.eval("abs(-1)===1 && ceil(1.1)===2 && floor(1.1)===1 && round(1.5)===2 && sign(-1)===-1 && trunc(1.9)===1 && typeof Math==='undefined'"),Ok(Value::Boolean(true)));
    for source in ["Math.pow", "Reflect.ownKeys(Math)"] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check("Reflect.has(Math,'pow') && Reflect.has(Math,'sumPrecise')");
}
