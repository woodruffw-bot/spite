//! Circular and hyperbolic Math functions: domains, endpoints, and coercion.

mod common;
use common::REALM_ENTRIES;
use spite_runtime::{Error, Realm, Value};

const METHODS: [&str; 6] = ["sin", "cos", "tan", "sinh", "cosh", "tanh"];

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn required_zero_nan_and_infinity_results_are_exact() {
    for method in METHODS {
        check(&format!(
            "Number.isNaN(Math.{method}(NaN)) && Number.isNaN(Math.{method}()) && Number.isNaN(Math.{method}(undefined))"
        ));
    }
    for method in ["sin", "tan", "sinh", "tanh"] {
        check(&format!(
            "Object.is(Math.{method}(0),0) && Object.is(Math.{method}(-0),-0)"
        ));
    }
    for method in ["cos", "cosh"] {
        check(&format!("Math.{method}(0)===1 && Math.{method}(-0)===1"));
    }
    for method in ["sin", "cos", "tan"] {
        check(&format!(
            "Number.isNaN(Math.{method}(Infinity)) && Number.isNaN(Math.{method}(-Infinity))"
        ));
    }
    check(
        "Math.sinh(Infinity)===Infinity && Math.sinh(-Infinity)===-Infinity && Math.cosh(Infinity)===Infinity && Math.cosh(-Infinity)===Infinity && Math.tanh(Infinity)===1 && Math.tanh(-Infinity)===-1",
    );
}

#[test]
fn finite_approximations_cover_symmetry_extreme_and_subnormal_inputs() {
    check(
        "Math.abs(Math.sin(Math.PI/6)-0.5)<1e-15 && Math.abs(Math.cos(Math.PI/3)-0.5)<1e-15 && Math.abs(Math.tan(Math.PI/4)-1)<1e-15 && Math.abs(Math.sinh(1)-1.1752011936438014)<1e-15 && Math.abs(Math.cosh(1)-1.5430806348152437)<1e-15 && Math.abs(Math.tanh(1)-0.7615941559557649)<1e-15",
    );
    for method in ["sin", "tan", "sinh", "tanh"] {
        check(&format!(
            "Math.{method}(Number.MIN_VALUE)===Number.MIN_VALUE && Math.{method}(-Number.MIN_VALUE)===-Number.MIN_VALUE && Math.{method}(-0.125)===-Math.{method}(0.125)"
        ));
    }
    for method in ["cos", "cosh"] {
        check(&format!(
            "Math.{method}(Number.MIN_VALUE)===1 && Math.{method}(-0.125)===Math.{method}(0.125)"
        ));
    }
    check(
        "Number.isFinite(Math.sin(Number.MAX_VALUE)) && Math.abs(Math.sin(Number.MAX_VALUE))<=1 && Number.isFinite(Math.cos(-Number.MAX_VALUE)) && Math.abs(Math.cos(-Number.MAX_VALUE))<=1 && Number.isFinite(Math.tan(Number.MAX_VALUE)) && Math.sinh(Number.MAX_VALUE)===Infinity && Math.sinh(-Number.MAX_VALUE)===-Infinity && Math.cosh(-Number.MAX_VALUE)===Infinity && Math.tanh(Number.MAX_VALUE)===1 && Math.tanh(-Number.MAX_VALUE)===-1",
    );
}

#[test]
fn conversion_is_once_with_number_hint_and_no_receiver_or_extra_coercion() {
    for method in METHODS {
        let result = usize::from(["cos", "cosh"].contains(&method));
        check(&format!(
            "let calls=0,o={{[Symbol.toPrimitive](hint){{if(hint!=='number')throw 7;calls++;return -0;}},valueOf(){{throw 8;}}}};Object.is(Math.{method}.call(Symbol(),o,{{valueOf(){{throw 9;}}}}),{expected}) && calls===1 && Math.{method}('0')==={result} && Math.{method}(null)==={result}",
            expected = if result == 1 { "1" } else { "-0" },
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
}

#[test]
fn host_gaps_during_conversion_skip_javascript_handlers() {
    for method in METHODS {
        let mut realm = Realm::default();
        realm.eval("let flag=0,o={valueOf(){Proxy;}};").unwrap();
        assert!(matches!(
            realm.eval(&format!(
                "try{{Math.{method}(o);}}catch{{flag=1;}}finally{{flag=2;}}"
            )),
            Err(Error::Unsupported { .. })
        ));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
}

#[test]
fn standard_function_descriptors_and_non_constructibility_are_preserved() {
    for method in METHODS {
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(Math,'{method}'),n=Object.getOwnPropertyDescriptor(d.value,'name'),l=Object.getOwnPropertyDescriptor(d.value,'length');d.writable && !d.enumerable && d.configurable && n.value==='{method}' && !n.writable && !n.enumerable && n.configurable && l.value===1 && !l.writable && !l.enumerable && l.configurable && !Object.hasOwn(d.value,'prototype')"
        ));
        check(&format!(
            "let caught=false;try{{new Math.{method}();}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
}

#[test]
fn intrinsic_roots_retain_functions_after_deleting_public_properties() {
    let mut realm = Realm::default();
    realm.eval("let sin=Math.sin,cos=Math.cos,tan=Math.tan,sinh=Math.sinh,cosh=Math.cosh,tanh=Math.tanh;delete Math.sin;delete Math.cos;delete Math.tan;delete Math.sinh;delete Math.cosh;delete Math.tanh;delete globalThis.Math;").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(realm.eval("sin(0)===0 && cos(0)===1 && tan(0)===0 && sinh(0)===0 && cosh(0)===1 && tanh(0)===0 && typeof Math==='undefined'"),Ok(Value::Boolean(true)));
}
