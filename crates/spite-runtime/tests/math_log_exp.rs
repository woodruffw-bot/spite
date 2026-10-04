//! Exponential/logarithmic domains, small inputs, exact powers, and coercion.

mod common;
use common::REALM_ENTRIES;
use spite_runtime::{Error, Realm, Value};

const METHODS: [&str; 6] = ["exp", "expm1", "log", "log1p", "log2", "log10"];

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn exponential_endpoints_and_signed_zero_are_correct() {
    check(
        "Math.exp(0)===1 && Math.exp(-0)===1 && Math.exp(Infinity)===Infinity && Object.is(Math.exp(-Infinity),0) && Math.expm1(Infinity)===Infinity && Math.expm1(-Infinity)===-1 && Object.is(Math.expm1(0),0) && Object.is(Math.expm1(-0),-0)",
    );
    check(
        "Math.exp(1000)===Infinity && Object.is(Math.exp(-1000),0) && Math.expm1(1000)===Infinity && Math.expm1(-1000)===-1 && Math.abs(Math.exp(1)-Math.E)<1e-15 && Math.abs(Math.expm1(1)-(Math.E-1))<1e-15",
    );
    for method in METHODS {
        check(&format!(
            "Number.isNaN(Math.{method}(NaN)) && Number.isNaN(Math.{method}()) && Number.isNaN(Math.{method}(undefined))"
        ));
    }
}

#[test]
fn logarithm_domain_endpoints_preserve_required_infinities_and_zeros() {
    for method in ["log", "log2", "log10"] {
        check(&format!(
            "Math.{method}(0)===-Infinity && Math.{method}(-0)===-Infinity && Object.is(Math.{method}(1),0) && Math.{method}(Infinity)===Infinity && Number.isNaN(Math.{method}(-1)) && Number.isNaN(Math.{method}(-Number.MIN_VALUE)) && Number.isNaN(Math.{method}(-Infinity))"
        ));
    }
    check(
        "Object.is(Math.log1p(0),0) && Object.is(Math.log1p(-0),-0) && Math.log1p(-1)===-Infinity && Math.log1p(Infinity)===Infinity && Number.isNaN(Math.log1p(-1-Number.EPSILON)) && Number.isNaN(Math.log1p(-Infinity))",
    );
    check(
        "Math.abs(Math.log(Math.E)-1)<1e-15 && Math.abs(Math.log(2)-Math.LN2)<1e-15 && Math.log10(10)===1 && Math.log10(100)===2 && Math.log10(1000)===3 && Math.abs(Math.log1p(-1+2**-53)+53*Math.LN2)<1e-14",
    );
}

#[test]
fn log2_is_exact_for_every_representable_binary_power() {
    check(
        "let good=true;for(let i=-1074;i<=1023;i++){if(Math.log2(2**i)!==i)good=false;}good && Math.log2(Number.MIN_VALUE)===-1074 && Math.log2(2**-1022)===-1022 && Math.log2(2**1023)===1023",
    );
    check(
        "Math.abs(Math.log2(3)-1.584962500721156)<1e-15 && Math.abs(Math.log2(0.3)+1.7369655941662063)<1e-15 && Math.log2(Number.MAX_VALUE)===1024",
    );
}

#[test]
fn near_zero_operations_avoid_loss_from_intermediate_addition_or_subtraction() {
    check(
        "let tiny=Number.EPSILON/4;Math.expm1(tiny)===tiny && Math.expm1(-tiny)===-tiny && Math.log1p(tiny)===tiny && Math.log1p(-tiny)===-tiny && Math.exp(tiny)-1===0 && Math.log(1+tiny)===0",
    );
    check(
        "Math.expm1(Number.MIN_VALUE)===Number.MIN_VALUE && Math.expm1(-Number.MIN_VALUE)===-Number.MIN_VALUE && Math.log1p(Number.MIN_VALUE)===Number.MIN_VALUE && Math.log1p(-Number.MIN_VALUE)===-Number.MIN_VALUE",
    );
    check(
        "Math.abs(Math.expm1(1e-6)-0.0000010000005000001665)<1e-21 && Math.abs(Math.log1p(1e-6)-0.0000009999995000003334)<1e-21",
    );
}

#[test]
fn methods_convert_once_ignore_receivers_and_skip_extra_argument_conversion() {
    for (method, input, expected) in [
        ("exp", 0, 1),
        ("expm1", 0, 0),
        ("log", 1, 0),
        ("log1p", 0, 0),
        ("log2", 2, 1),
        ("log10", 10, 1),
    ] {
        check(&format!(
            "let calls=0,o={{[Symbol.toPrimitive](hint){{if(hint!=='number')throw 7;calls++;return {input};}},valueOf(){{throw 8;}}}};Math.{method}.call(Symbol(),o,{{valueOf(){{throw 9;}}}})==={expected} && calls===1 && Math.{method}('{input}')==={expected}"
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
    for method in METHODS {
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(Math,'{method}'),n=Object.getOwnPropertyDescriptor(d.value,'name'),l=Object.getOwnPropertyDescriptor(d.value,'length');d.writable && !d.enumerable && d.configurable && n.value==='{method}' && !n.writable && !n.enumerable && n.configurable && l.value===1 && !l.writable && !l.enumerable && l.configurable && !Object.hasOwn(d.value,'prototype')"
        ));
        check(&format!(
            "let caught=false;try{{new Math.{method}();}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    let mut realm = Realm::default();
    realm.eval("let exp=Math.exp,expm1=Math.expm1,log=Math.log,log1p=Math.log1p,log2=Math.log2,log10=Math.log10;delete Math.exp;delete Math.expm1;delete Math.log;delete Math.log1p;delete Math.log2;delete Math.log10;delete globalThis.Math;").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(realm.eval("exp(0)===1 && expm1(0)===0 && log(1)===0 && log1p(0)===0 && log2(2)===1 && log10(10)===1 && typeof Math==='undefined'"),Ok(Value::Boolean(true)));
}
