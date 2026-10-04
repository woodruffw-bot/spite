//! Math inverse functions, cube roots, and ordered atan2 quadrants.

mod common;
use common::REALM_ENTRIES;
use spite_runtime::{Error, Realm, Value};

const UNARY: [&str; 7] = ["acos", "acosh", "asin", "asinh", "atan", "atanh", "cbrt"];

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn unary_domains_endpoints_and_signed_zeros_are_correct() {
    for method in UNARY {
        check(&format!(
            "Number.isNaN(Math.{method}(NaN)) && Number.isNaN(Math.{method}()) && Number.isNaN(Math.{method}(undefined))"
        ));
    }
    for method in ["acos", "asin", "atanh"] {
        check(&format!(
            "Number.isNaN(Math.{method}(1+Number.EPSILON)) && Number.isNaN(Math.{method}(-1-Number.EPSILON)) && Number.isNaN(Math.{method}(Infinity)) && Number.isNaN(Math.{method}(-Infinity))"
        ));
    }
    check(
        "Object.is(Math.acos(1),0) && Object.is(Math.acosh(1),0) && Number.isNaN(Math.acosh(1-Number.EPSILON)) && Number.isNaN(Math.acosh(0)) && Number.isNaN(Math.acosh(-Infinity)) && Math.acosh(Infinity)===Infinity && Math.asin(1)===Math.PI/2 && Math.asin(-1)===-Math.PI/2 && Math.atan(Infinity)===Math.PI/2 && Math.atan(-Infinity)===-Math.PI/2 && Math.atanh(1)===Infinity && Math.atanh(-1)===-Infinity",
    );
    for method in ["asin", "asinh", "atan", "atanh", "cbrt"] {
        check(&format!(
            "Object.is(Math.{method}(0),0) && Object.is(Math.{method}(-0),-0)"
        ));
    }
    for method in ["asinh", "cbrt"] {
        check(&format!(
            "Math.{method}(Infinity)===Infinity && Math.{method}(-Infinity)===-Infinity"
        ));
    }
}

#[test]
fn finite_approximations_handle_ordinary_extreme_and_subnormal_inputs() {
    check(
        "Math.abs(Math.acos(0)-Math.PI/2)<1e-15 && Math.abs(Math.asin(0.5)-Math.PI/6)<1e-15 && Math.abs(Math.atan(1)-Math.PI/4)<1e-15 && Math.abs(Math.acosh(2)-1.3169578969248166)<1e-15 && Math.abs(Math.asinh(-1)+0.881373587019543)<1e-15 && Math.abs(Math.atanh(0.5)-0.5493061443340548)<1e-15 && Math.abs(Math.cbrt(-8)+2)<1e-14",
    );
    check(
        "Number.isFinite(Math.acosh(Number.MAX_VALUE)) && Number.isFinite(Math.asinh(Number.MAX_VALUE)) && Number.isFinite(Math.asinh(-Number.MAX_VALUE)) && Number.isFinite(Math.cbrt(Number.MAX_VALUE)) && Number.isFinite(Math.cbrt(-Number.MAX_VALUE)) && Math.cbrt(Number.MIN_VALUE)===2**-358 && Math.cbrt(-Number.MIN_VALUE)===-(2**-358)",
    );
    for method in ["asin", "asinh", "atan", "atanh"] {
        check(&format!(
            "Math.{method}(Number.MIN_VALUE)===Number.MIN_VALUE && Math.{method}(-Number.MIN_VALUE)===-Number.MIN_VALUE"
        ));
    }
}

#[test]
fn atan2_distinguishes_every_zero_and_infinity_quadrant() {
    let x = ["0", "-0", "Infinity", "-Infinity"];
    for (y, expected) in [
        ("0", ["0", "Math.PI", "0", "Math.PI"]),
        ("-0", ["-0", "-Math.PI", "-0", "-Math.PI"]),
        (
            "Infinity",
            ["Math.PI/2", "Math.PI/2", "Math.PI/4", "3*Math.PI/4"],
        ),
        (
            "-Infinity",
            ["-Math.PI/2", "-Math.PI/2", "-Math.PI/4", "-3*Math.PI/4"],
        ),
    ] {
        for (x, expected) in x.into_iter().zip(expected) {
            check(&format!("Object.is(Math.atan2({y},{x}),{expected})"));
        }
    }
    check(
        "Math.atan2(1,1)===Math.PI/4 && Math.atan2(1,-1)===3*Math.PI/4 && Math.atan2(-1,-1)===-3*Math.PI/4 && Math.atan2(-1,1)===-Math.PI/4 && Math.atan2(1,0)===Math.PI/2 && Math.atan2(-1,-0)===-Math.PI/2",
    );
    check(
        "Math.atan2(Number.MIN_VALUE,1)===Number.MIN_VALUE && Math.atan2(-Number.MIN_VALUE,1)===-Number.MIN_VALUE && Object.is(Math.atan2(-Number.MAX_VALUE,Infinity),-0) && Math.atan2(1,Number.MIN_VALUE)===Math.PI/2 && Math.atan2(1,-Infinity)===Math.PI",
    );
}

#[test]
fn atan2_converts_y_then_x_before_nan_and_skips_extra_arguments() {
    check(
        "let log='',y={[Symbol.toPrimitive](hint){if(hint!=='number')throw 8;log+='y';return NaN;}},x={valueOf(){log+='x';return 1;}};Number.isNaN(Math.atan2.call(Symbol(),y,x,{valueOf(){throw 9;}})) && log==='yx'",
    );
    check(
        "let calls=0,caught=false;try{Math.atan2(NaN,{valueOf(){calls++;throw 7;}});}catch(e){caught=e===7;}caught && calls===1",
    );
    check(
        "let calls=0,caught=false;try{Math.atan2({valueOf(){throw 7;}},{valueOf(){calls++;return 1;}});}catch(e){caught=e===7;}caught && calls===0",
    );
    check(
        "Number.isNaN(Math.atan2()) && Number.isNaN(Math.atan2(1)) && Number.isNaN(Math.atan2(0,NaN)) && Math.atan2('1','1')===Math.PI/4",
    );
    for argument in ["1n", "Object(1n)", "Symbol()", "Object(Symbol())"] {
        for expression in [
            format!("Math.atan2({argument},NaN)"),
            format!("Math.atan2(NaN,{argument})"),
        ] {
            check(&format!(
                "let caught=false;try{{{expression};}}catch(e){{caught=e instanceof TypeError;}}caught"
            ));
        }
    }
}

#[test]
fn unary_conversions_are_once_and_host_failures_skip_pending_handlers() {
    for method in UNARY {
        let input = if ["acos", "acosh"].contains(&method) {
            1
        } else {
            0
        };
        check(&format!(
            "let calls=0,o={{[Symbol.toPrimitive](hint){{if(hint!=='number')throw 7;calls++;return {input};}}}};Math.{method}.call(Symbol(),o,{{valueOf(){{throw 8;}}}})===0 && calls===1 && Math.{method}('{input}')===0"
        ));
        check(&format!(
            "let caught=false;try{{Math.{method}({{valueOf(){{throw 7;}}}});}}catch(e){{caught=e===7;}}caught"
        ));
        for argument in ["1n", "Object(1n)", "Symbol()", "Object(Symbol())"] {
            check(&format!(
                "let caught=false;try{{Math.{method}({argument});}}catch(e){{caught=e instanceof TypeError;}}caught"
            ));
        }
    }
    for expression in UNARY
        .map(|name| format!("Math.{name}(o)"))
        .into_iter()
        .chain(["Math.atan2(o,0)".into(), "Math.atan2(NaN,o)".into()])
    {
        let mut realm = Realm::default();
        realm.eval("let flag=0,o={valueOf(){Proxy;}};").unwrap();
        assert!(matches!(
            realm.eval(&format!(
                "try{{{expression};}}catch{{flag=1;}}finally{{flag=2;}}"
            )),
            Err(Error::Unsupported { .. })
        ));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
}

#[test]
fn standard_metadata_and_intrinsic_retention_survive_public_deletion() {
    for method in UNARY.into_iter().chain(["atan2"]) {
        let length = if method == "atan2" { 2 } else { 1 };
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(Math,'{method}'),n=Object.getOwnPropertyDescriptor(d.value,'name'),l=Object.getOwnPropertyDescriptor(d.value,'length');d.writable && !d.enumerable && d.configurable && n.value==='{method}' && !n.writable && !n.enumerable && n.configurable && l.value==={length} && !l.writable && !l.enumerable && l.configurable && !Object.hasOwn(d.value,'prototype')"
        ));
        check(&format!(
            "let caught=false;try{{new Math.{method}();}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    let mut realm = Realm::default();
    realm.eval("let acos=Math.acos,acosh=Math.acosh,asin=Math.asin,asinh=Math.asinh,atan=Math.atan,atanh=Math.atanh,cbrt=Math.cbrt,atan2=Math.atan2;delete Math.acos;delete Math.acosh;delete Math.asin;delete Math.asinh;delete Math.atan;delete Math.atanh;delete Math.cbrt;delete Math.atan2;delete globalThis.Math;").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(realm.eval("acos(1)===0 && acosh(1)===0 && asin(0)===0 && asinh(0)===0 && atan(0)===0 && atanh(0)===0 && cbrt(0)===0 && atan2(1,1)===0.7853981633974483 && typeof Math==='undefined'"),Ok(Value::Boolean(true)));
}
