//! BigInt width APIs: ToIndex precedes ToBigInt (21.2.2.1–2).

mod common;

use common::REALM_ENTRIES;
use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn signed_and_unsigned_widths_reduce_exactly_across_word_boundaries() {
    for bits in [0, 1, 8, 16, 31, 32, 33, 63, 64, 65, 127, 128, 129] {
        let unsigned = if bits == 0 {
            "0n".into()
        } else {
            format!("2n**{bits}n-1n")
        };
        let signed = if bits == 0 { "0n" } else { "-1n" };
        check(&format!(
            "BigInt.asUintN({bits},-1n)===({unsigned}) && BigInt.asIntN({bits},-1n)==={signed} && BigInt.asIntN({bits},2n**{bits}n)===0n && BigInt.asUintN({bits},-(2n**{bits}n))===0n"
        ));
        if bits != 0 {
            check(&format!(
                "let min=-(2n**{}n);BigInt.asIntN({bits},-min)===min && BigInt.asIntN({bits},min)===min && BigInt.asIntN({bits},min-1n)===-min-1n",
                bits - 1
            ));
        }
    }
    check(
        "BigInt.asIntN(8,255n)===-1n && BigInt.asUintN(8,-255n)===1n && BigInt.asIntN(8,-255n)===1n",
    );
}

#[test]
fn widths_use_to_index_and_values_use_to_bigint() {
    for width in [
        "undefined",
        "NaN",
        "null",
        "false",
        "0",
        "-0",
        "-0.5",
        "0.9",
    ] {
        check(&format!(
            "BigInt.asIntN({width},7n)===0n && BigInt.asUintN({width},-7n)===0n"
        ));
    }
    for width in ["8.9", "'8'", "new Number(8)"] {
        check(&format!(
            "BigInt.asIntN({width},255n)===-1n && BigInt.asUintN({width},-1n)===255n"
        ));
    }
    for value in ["true", "'1'", "1n", "Object(1n)", "new Boolean(true)"] {
        check(&format!(
            "BigInt.asIntN(8,{value})===1n && BigInt.asUintN(8,{value})===1n"
        ));
    }
    for width in ["-1", "Infinity", "-Infinity", "9007199254740992"] {
        check(&format!(
            "let caught=false;try{{BigInt.asIntN({width},1n);}}catch(e){{caught=e instanceof RangeError;}}caught"
        ));
    }
    for width in ["1n", "Symbol()"] {
        check(&format!(
            "let caught=false;try{{BigInt.asUintN({width},1n);}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    for value in ["1", "new Number(1)", "undefined", "null", "Symbol()"] {
        check(&format!(
            "let caught=false;try{{BigInt.asIntN(0,{value});}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    check("let caught=false;try{BigInt.asUintN();}catch(e){caught=e instanceof TypeError;}caught");
    check(
        "let caught=false;try{BigInt.asUintN(0,'1.5');}catch(e){caught=e instanceof SyntaxError;}caught",
    );
}

#[test]
fn width_coercion_finishes_before_integer_coercion_even_for_zero_width() {
    check(
        "let log='',bits={[Symbol.toPrimitive](hint){log+='b'+hint;return 8;}},value={[Symbol.toPrimitive](hint){log+='v'+hint;return '-1';}};BigInt.asUintN(bits,value)===255n && log==='bnumbervnumber'",
    );
    check(
        "let calls=0,value={valueOf(){calls++;return 7n;}};BigInt.asIntN(0,value)===0n && calls===1",
    );
    check(
        "let calls=0,value={valueOf(){calls++;throw 7;}};let caught=false;try{BigInt.asUintN(0,value);}catch(e){caught=e===7;}caught && calls===1",
    );
    for width in ["-1", "Symbol()", "{valueOf(){throw 8;}}"] {
        check(&format!(
            "let calls=0,value={{valueOf(){{calls++;return 7n;}}}};let caught=false;try{{BigInt.asIntN({width},value);}}catch(e){{caught=true;}}caught && calls===0"
        ));
    }
}

#[test]
fn huge_widths_do_not_allocate_width_sized_storage_for_fitting_values() {
    check(
        "BigInt.asIntN(9007199254740991,-1n)===-1n && BigInt.asIntN(9007199254740991,1n)===1n && BigInt.asUintN(9007199254740991,1n)===1n && BigInt.asUintN(9007199254740991,0n)===0n",
    );
}

#[test]
fn static_descriptors_enumeration_deletion_and_collection_are_complete() {
    check(
        "BigInt.asIntN.name==='asIntN' && BigInt.asUintN.name==='asUintN' && BigInt.asIntN.length===2 && BigInt.asUintN.length===2 && Object.getOwnPropertyNames(BigInt).sort().join(',')==='asIntN,asUintN,length,name,prototype' && Object.keys(BigInt).length===0",
    );
    check(
        "let d=Object.getOwnPropertyDescriptor(BigInt,'asIntN');d.writable && !d.enumerable && d.configurable && d.value===BigInt.asIntN && Object.freeze(BigInt)===BigInt && Object.isFrozen(BigInt)",
    );
    check("delete BigInt.asIntN;BigInt.asIntN===undefined && !Object.hasOwn(BigInt,'asIntN')");
    check(
        "let calls=0,value={valueOf(){calls++;return 1n;}};let caught=false;try{new BigInt.asIntN(8,value);}catch(e){caught=e instanceof TypeError;}caught && calls===0",
    );
    let mut realm = Realm::default();
    realm
        .eval("let signed=BigInt.asIntN,unsigned=BigInt.asUintN;delete globalThis.BigInt")
        .unwrap();
    assert_eq!(realm.collect(10_000).unwrap().live, REALM_ENTRIES);
    assert_eq!(
        realm.eval("signed(8,255n)===-1n && unsigned(8,-1n)===255n"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn opted_in_result_size_quota_rejects_large_outputs_before_allocation() {
    for source in [
        "BigInt.asUintN(9,-1n)",
        "BigInt.asUintN(9007199254740991,-1n)",
    ] {
        let mut realm = Realm::new(Limits {
            max_bigint_bits: Some(8),
            ..Limits::default()
        });
        realm.eval("let flag=0").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "try{{{source};}}catch{{flag=1;}}finally{{flag=2;}}"
                )),
                Err(Error::Limit { .. })
            ),
            "{source}"
        );
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
        assert_eq!(
            realm.eval("BigInt.asIntN(9007199254740991,-1n)===-1n && BigInt.asUintN(8,-255n)===1n"),
            Ok(Value::Boolean(true))
        );
    }
}
