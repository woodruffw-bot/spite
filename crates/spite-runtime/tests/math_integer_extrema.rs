//! Math extrema and exact 32-bit operations, including ordered conversions.

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
fn extrema_handle_empty_calls_nan_infinity_and_both_zero_orders() {
    check(
        "Math.max()===-Infinity && Math.min()===Infinity && Math.max(-Infinity,Infinity)===Infinity && Math.min(Infinity,-Infinity)===-Infinity",
    );
    for method in ["max", "min"] {
        check(&format!(
            "Number.isNaN(Math.{method}(NaN)) && Number.isNaN(Math.{method}(Infinity,NaN,-Infinity)) && Number.isNaN(Math.{method}(undefined)) && Number.isNaN(Math.{method}({{}}))"
        ));
        check(&format!(
            "Object.is(Math.{method}(-0),-0) && Object.is(Math.{method}(0),0) && Object.is(Math.{method}(-0,-0),-0) && Object.is(Math.{method}(0,0),0)"
        ));
        let zero = if method == "max" { "0" } else { "-0" };
        check(&format!(
            "Object.is(Math.{method}(0,-0),{zero}) && Object.is(Math.{method}(-0,0),{zero}) && Object.is(Math.{method}(-0,0,-0),{zero})"
        ));
    }
    check(
        "Math.max(-Number.MAX_VALUE,Number.MAX_VALUE)===Number.MAX_VALUE && Math.min(Number.MAX_VALUE,-Number.MAX_VALUE)===-Number.MAX_VALUE && Math.max(-Number.MIN_VALUE,Number.MIN_VALUE)===Number.MIN_VALUE && Math.min(Number.MIN_VALUE,-Number.MIN_VALUE)===-Number.MIN_VALUE",
    );
    check(
        "Math.max.call(null,'3',true,null,-2)===3 && Math.min.call(Symbol(),'3',true,null,-2)===-2",
    );
}

#[test]
fn extrema_convert_all_arguments_once_before_nan_or_infinite_results() {
    for method in ["max", "min"] {
        check(&format!(
            "let log='',a={{[Symbol.toPrimitive](hint){{if(hint!=='number')throw 9;log+='a';return NaN;}}}},b={{valueOf(){{log+='b';return 7;}}}},c={{valueOf(){{log+='c';return -3;}}}};Number.isNaN(Math.{method}(a,b,c)) && log==='abc'"
        ));
        check(&format!(
            "let log='',a={{valueOf(){{log+='a';return 1;}}}},b={{valueOf(){{log+='b';throw 7;}}}},c={{valueOf(){{log+='c';return 9;}}}},caught=false;try{{Math.{method}(NaN,a,b,c);}}catch(e){{caught=e===7;}}caught && log==='ab'"
        ));
        check(&format!(
            "let calls=0,o={{valueOf(){{calls++;return 2;}}}};Math.{method}(Infinity,-Infinity,o);calls===1"
        ));
        for argument in ["1n", "Object(1n)", "Symbol()", "Object(Symbol())"] {
            check(&format!(
                "let caught=false;try{{Math.{method}(NaN,{argument});}}catch(e){{caught=e instanceof TypeError;}}caught"
            ));
        }
    }
}

#[test]
fn clz32_counts_bits_after_unsigned_truncation_and_wrapping() {
    check(
        "Math.clz32()===32 && Math.clz32(undefined)===32 && Math.clz32(null)===32 && Math.clz32(NaN)===32 && Math.clz32(Infinity)===32 && Math.clz32(-Infinity)===32 && Math.clz32(-0)===32 && Math.clz32(-0.9)===32",
    );
    check(
        "Math.clz32(1.9)===31 && Math.clz32(-1.9)===0 && Math.clz32(4294967296)===32 && Math.clz32(4294967297)===31 && Math.clz32(-4294967295)===31 && Math.clz32(Number.MAX_VALUE)===32 && Math.clz32(-Number.MAX_VALUE)===32 && Math.clz32('256')===23 && Math.clz32(true)===31",
    );
    check("let good=true;for(let i=0;i<32;i++){if(Math.clz32(2**i)!==31-i)good=false;}good");
    check(
        "let calls=0,o={[Symbol.toPrimitive](hint){if(hint!=='number')throw 9;calls++;return 256;}};Math.clz32.call(null,o,{valueOf(){throw 8;}})===23 && calls===1",
    );
}

#[test]
fn imul_returns_exact_signed_modular_products_and_positive_zero() {
    check(
        "Math.imul()===0 && Math.imul(3)===0 && Math.imul('3',true)===3 && Math.imul(-1,5)===-5 && Math.imul(4294967295,4294967295)===1 && Math.imul(2147483647,2147483647)===1 && Math.imul(2147483648,1)===-2147483648 && Math.imul(4294967297,3)===3",
    );
    check(
        "Math.imul(65535,65535)===-131071 && Math.imul(65536,65536)===0 && Math.imul(1073741824,7)===-1073741824 && Math.imul(-1073741824,7)===1073741824 && Math.imul(-1.9,7.9)===-7",
    );
    for zero in [
        "NaN",
        "Infinity",
        "-Infinity",
        "0",
        "-0",
        "0.9",
        "-0.9",
        "4294967296",
        "Number.MAX_VALUE",
    ] {
        check(&format!(
            "Object.is(Math.imul({zero},7),0) && Object.is(Math.imul(7,{zero}),0)"
        ));
    }
    check(
        "let log='',a={[Symbol.toPrimitive](hint){if(hint!=='number')throw 9;log+='a';return 4294967295;}},b={valueOf(){log+='b';return 3;}};Math.imul.call(Symbol(),a,b,{valueOf(){throw 8;}})===-3 && log==='ab'",
    );
    check(
        "let calls=0,caught=false;try{Math.imul({valueOf(){throw 7;}},{valueOf(){calls++;return 3;}});}catch(e){caught=e===7;}caught && calls===0",
    );
    check(
        "let calls=0,caught=false;try{Math.imul(NaN,{valueOf(){calls++;throw 7;}});}catch(e){caught=e===7;}caught && calls===1",
    );
}

#[test]
fn integer_conversions_reject_bigints_symbols_and_preserve_host_abort() {
    for method in ["clz32", "imul"] {
        for argument in ["1n", "Object(1n)", "Symbol()", "Object(Symbol())"] {
            check(&format!(
                "let caught=false;try{{Math.{method}({argument});}}catch(e){{caught=e instanceof TypeError;}}caught"
            ));
        }
    }
    for argument in ["1n", "Object(1n)", "Symbol()", "Object(Symbol())"] {
        check(&format!(
            "let caught=false;try{{Math.imul(0,{argument});}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    for expression in [
        "Math.max(NaN,o)",
        "Math.min(NaN,o)",
        "Math.clz32(o)",
        "Math.imul(0,o)",
    ] {
        let mut realm = Realm::default();
        realm.eval("let flag=0,o={valueOf(){Proxy;}};").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "try{{{expression};}}catch{{flag=1;}}finally{{flag=2;}}"
                )),
                Err(Error::Unsupported { .. })
            ),
            "{expression}"
        );
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
}

#[test]
fn default_limits_allow_large_extremum_argument_lists() {
    check(
        "let values=[];for(let i=0;i<4096;i++)values.push(i-2048);Math.max(...values)===2047 && Math.min(...values)===-2048",
    );
}

#[test]
fn metadata_and_intrinsic_roots_survive_deleted_public_properties() {
    for (method, length) in [("max", 2), ("min", 2), ("clz32", 1), ("imul", 2)] {
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(Math,'{method}'),n=Object.getOwnPropertyDescriptor(d.value,'name'),l=Object.getOwnPropertyDescriptor(d.value,'length');d.writable && !d.enumerable && d.configurable && n.value==='{method}' && !n.writable && !n.enumerable && n.configurable && l.value==={length} && !l.writable && !l.enumerable && l.configurable && !Object.hasOwn(d.value,'prototype')"
        ));
        check(&format!(
            "let caught=false;try{{new Math.{method}();}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    let mut realm = Realm::default();
    realm.eval("let max=Math.max,min=Math.min,clz32=Math.clz32,imul=Math.imul;delete Math.max;delete Math.min;delete Math.clz32;delete Math.imul;delete globalThis.Math;").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(realm.eval("max(1,2)===2 && min(1,2)===1 && clz32(1)===31 && imul(-1,2)===-2 && typeof Math==='undefined'"),Ok(Value::Boolean(true)));
}
