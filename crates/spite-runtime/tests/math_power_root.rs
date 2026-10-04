//! Math power/root conversion order, special values, exact sqrt, and roots.

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
fn power_special_values_follow_number_exponentiation() {
    check(
        "Number.isNaN(Math.pow()) && Number.isNaN(Math.pow(2)) && Number.isNaN(Math.pow(1,NaN)) && Math.pow(NaN,0)===1 && Math.pow(NaN,-0)===1 && Number.isNaN(Math.pow(NaN,1)) && Number.isNaN(Math.pow(1,Infinity)) && Number.isNaN(Math.pow(-1,-Infinity))",
    );
    check(
        "Math.pow(2,Infinity)===Infinity && Object.is(Math.pow(2,-Infinity),0) && Object.is(Math.pow(0.5,Infinity),0) && Math.pow(-0.5,-Infinity)===Infinity && Math.pow(-Infinity,3)===-Infinity && Math.pow(-Infinity,2)===Infinity && Object.is(Math.pow(-Infinity,-3),-0) && Object.is(Math.pow(-Infinity,-2),0)",
    );
    check(
        "Object.is(Math.pow(-0,3),-0) && Object.is(Math.pow(-0,2),0) && Math.pow(-0,-3)===-Infinity && Math.pow(-0,-2)===Infinity && Object.is(Math.pow(-0,0.5),0) && Math.pow(0,-0.5)===Infinity && Math.pow(-0,0)===1 && Number.isNaN(Math.pow(-2,0.5))",
    );
    check(
        "Math.pow(-1,9007199254740991)===-1 && Math.pow(-1,9007199254740992)===1 && Math.pow(-1,-9007199254740991)===-1 && Math.pow(1,-2147483648)===1 && Object.is(Math.pow(2,-2147483648),0) && Math.pow(2,1024)===Infinity && Math.pow(2,-1074)===Number.MIN_VALUE && Object.is(Math.pow(-2,-1075),-0) && Math.pow(-2,1025)===-Infinity",
    );
    check(
        "Math.pow.call(null,'2','3')===8 && Math.pow(true,null)===1 && Math.pow(-2,3)===-8 && Math.pow(-2,-3)===-0.125",
    );
}

#[test]
fn pow_converts_both_arguments_once_in_order_before_numeric_shortcuts() {
    check(
        "let log='',base={[Symbol.toPrimitive](hint){if(hint!=='number')throw 8;log+='b';return NaN;}},exp={valueOf(){log+='e';return 0;}};Math.pow.call(Symbol(),base,exp,{valueOf(){throw 9;}})===1 && log==='be'",
    );
    check(
        "let log='',base={valueOf(){log+='b';throw 7;}},exp={valueOf(){log+='e';return 0;}},caught=false;try{Math.pow(base,exp);}catch(e){caught=e===7;}caught && log==='b'",
    );
    check(
        "let calls=0,caught=false;try{Math.pow(NaN,{valueOf(){calls++;throw 7;}});}catch(e){caught=e===7;}caught && calls===1",
    );
    for argument in ["1n", "Object(1n)", "Symbol()", "Object(Symbol())"] {
        for expression in [
            format!("Math.pow({argument},0)"),
            format!("Math.pow(NaN,{argument})"),
        ] {
            check(&format!(
                "let caught=false;try{{{expression};}}catch(e){{caught=e instanceof TypeError;}}caught"
            ));
        }
    }
}

#[test]
fn sqrt_is_correctly_rounded_and_preserves_zero_and_range_endpoints() {
    check(
        "Number.isNaN(Math.sqrt()) && Number.isNaN(Math.sqrt(NaN)) && Number.isNaN(Math.sqrt(-1)) && Number.isNaN(Math.sqrt(-Number.MIN_VALUE)) && Number.isNaN(Math.sqrt(-Infinity)) && Math.sqrt(Infinity)===Infinity && Object.is(Math.sqrt(0),0) && Object.is(Math.sqrt(-0),-0)",
    );
    check(
        "Math.sqrt(2)===Math.SQRT2 && Math.sqrt(Number.MIN_VALUE)===2**-537 && Math.sqrt(Number.MAX_VALUE)===1.3407807929942596e154 && Math.sqrt(2.6364218206773287)===1.6237061990019404 && Math.sqrt(18.826149681104784)===4.33891111698601 && Math.sqrt(0.09916483871835496)===0.31490449142296295",
    );
    check(
        "let good=true;for(let i=-537;i<=511;i++){let power=2**i;if(Math.sqrt(power*power)!==power)good=false;}good",
    );
}

#[test]
fn sqrt_converts_once_and_skips_extra_argument_conversion() {
    check(
        "let calls=0,o={[Symbol.toPrimitive](hint){if(hint!=='number')throw 8;calls++;return 4;}};Math.sqrt.call(Symbol(),o,{valueOf(){throw 9;}})===2 && calls===1 && Math.sqrt('9')===3 && Math.sqrt(true)===1 && Math.sqrt(null)===0",
    );
    for argument in ["1n", "Object(1n)", "Symbol()", "Object(Symbol())"] {
        check(&format!(
            "let caught=false;try{{Math.sqrt({argument});}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    check("let caught=false;try{Math.sqrt({valueOf(){throw 7;}});}catch(e){caught=e===7;}caught");
}

#[test]
fn unsupported_conversions_abort_without_running_handlers_or_finalizers() {
    for expression in ["Math.pow(o,0)", "Math.pow(NaN,o)", "Math.sqrt(o)"] {
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
    for (method, length) in [("pow", 2), ("sqrt", 1)] {
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(Math,'{method}'),n=Object.getOwnPropertyDescriptor(d.value,'name'),l=Object.getOwnPropertyDescriptor(d.value,'length');d.writable && !d.enumerable && d.configurable && n.value==='{method}' && !n.writable && !n.enumerable && n.configurable && l.value==={length} && !l.writable && !l.enumerable && l.configurable && !Object.hasOwn(d.value,'prototype')"
        ));
        check(&format!(
            "let caught=false;try{{new Math.{method}();}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    let mut realm = Realm::default();
    realm.eval("let pow=Math.pow,sqrt=Math.sqrt;delete Math.pow;delete Math.sqrt;delete globalThis.Math;").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(
        realm.eval("pow(2,3)===8 && sqrt(4)===2 && typeof Math==='undefined'"),
        Ok(Value::Boolean(true))
    );
}
