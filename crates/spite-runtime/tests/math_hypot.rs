//! Math.hypot: ordered conversion and range-safe compensated norms.

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
fn infinity_wins_over_nan_and_zero_results_are_positive() {
    check(
        "Object.is(Math.hypot(),0) && Object.is(Math.hypot(-0),0) && Object.is(Math.hypot(0,-0,-0),0) && Number.isNaN(Math.hypot(NaN,3)) && Number.isNaN(Math.hypot(3,undefined)) && Math.hypot(NaN,Infinity)===Infinity && Math.hypot(-Infinity,NaN)===Infinity && Math.hypot(3,-Infinity,4)===Infinity",
    );
}

#[test]
fn scaling_preserves_large_small_and_subnormal_norms() {
    check(
        "Math.hypot(3,4)===5 && Math.hypot(-3,-4,12)===13 && Math.hypot(Number.MAX_VALUE)===Number.MAX_VALUE && Math.hypot(Number.MAX_VALUE,Number.MAX_VALUE)===Infinity && Math.hypot(-Number.MIN_VALUE)===Number.MIN_VALUE && Math.hypot(Number.MIN_VALUE,Number.MIN_VALUE)===Number.MIN_VALUE && Math.hypot(3*Number.MIN_VALUE,4*Number.MIN_VALUE)===5*Number.MIN_VALUE",
    );
    check(
        "let large=Math.hypot(1e308,-1e308),small=Math.hypot(1e-308,-1e-308);Number.isFinite(large) && Math.abs(large/1e308-Math.SQRT2)<1e-15 && small>0 && Math.abs(small/1e-308-Math.SQRT2)<1e-15 && Math.hypot(1e308,1e-308)===1e308 && Math.hypot(1e-308,1e308)===1e308",
    );
}

#[test]
fn compensated_summation_retains_many_small_terms_with_default_limits() {
    check(
        "let small=2**-27,args=[1];for(let i=0;i<1024;i++)args.push(small);let a=Math.hypot.apply(null,args);args.reverse();let b=Math.hypot.apply(null,args);a===1+2**-45 && b===a",
    );
}

#[test]
fn conversions_are_ordered_once_and_precede_numeric_shortcuts() {
    check(
        "let log='',a={[Symbol.toPrimitive](hint){if(hint!=='number')throw 7;log+='a';return NaN;}},b={valueOf(){log+='b';return -Infinity;}},c={valueOf(){log+='c';return 3;}};Math.hypot.call(Symbol(),a,b,c)===Infinity && log==='abc' && Math.hypot('3','4',false,null)===5",
    );
    for first in ["NaN", "Infinity", "-Infinity", "0"] {
        check(&format!(
            "let calls=0,caught=false;try{{Math.hypot({first},{{valueOf(){{calls++;throw 7;}}}},{{valueOf(){{calls+=100;return 3;}}}});}}catch(e){{caught=e===7;}}caught && calls===1"
        ));
    }
    for argument in ["1n", "Object(1n)", "Symbol()", "Object(Symbol())"] {
        check(&format!(
            "let caught=false;try{{Math.hypot(Infinity,{argument});}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
}

#[test]
fn unsupported_conversion_remains_a_host_failure_after_infinity_or_nan() {
    for first in ["NaN", "Infinity"] {
        let mut realm = Realm::default();
        realm.eval("let flag=0,o={valueOf(){Proxy;}};").unwrap();
        assert!(matches!(
            realm.eval(&format!(
                "try{{Math.hypot({first},o);}}catch{{flag=1;}}finally{{flag=2;}}"
            )),
            Err(Error::Unsupported { .. })
        ));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
}

#[test]
fn standard_descriptors_non_constructibility_and_roots_are_preserved() {
    check(
        "let d=Object.getOwnPropertyDescriptor(Math,'hypot'),n=Object.getOwnPropertyDescriptor(d.value,'name'),l=Object.getOwnPropertyDescriptor(d.value,'length');d.writable && !d.enumerable && d.configurable && n.value==='hypot' && !n.writable && !n.enumerable && n.configurable && l.value===2 && !l.writable && !l.enumerable && l.configurable && !Object.hasOwn(d.value,'prototype')",
    );
    check("let caught=false;try{new Math.hypot();}catch(e){caught=e instanceof TypeError;}caught");
    let mut realm = Realm::default();
    realm
        .eval("let hypot=Math.hypot;delete Math.hypot;delete globalThis.Math;")
        .unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(
        realm.eval("hypot(3,4)===5 && typeof Math==='undefined'"),
        Ok(Value::Boolean(true))
    );
}
