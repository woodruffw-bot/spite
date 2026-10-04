use super::*;
use crate::{
    ExceptionKind, Value,
    object::{Budget, DataDescriptor},
};

// The Symbol global stays unavailable until its wrappers and hooks are ready.
// Native injection lets us verify the primitive algorithms independently.
fn realm_with_symbols() -> Realm {
    let mut realm = Realm::default();
    realm.eval("").unwrap();
    let global = realm.global_object.as_ref().unwrap().clone();
    let symbol = JsSymbol::new(Some(JsString::from("name")));
    for (name, symbol) in [
        ("s", symbol.clone()),
        ("same", symbol),
        ("other", JsSymbol::new(Some(JsString::from("name")))),
        ("unnamed", JsSymbol::new(None)),
        ("empty", JsSymbol::new(Some(JsString::from("")))),
        (
            "raw",
            JsSymbol::new(Some(JsString::from_code_units(vec![0xD800, 0, 0xDC00]))),
        ),
    ] {
        realm
            .objects
            .define(
                &global,
                name,
                DataDescriptor {
                    value: Some(Value::Symbol(symbol)),
                    writable: Some(true),
                    enumerable: Some(true),
                    configurable: Some(true),
                },
                &mut Budget::new(10_000),
            )
            .unwrap();
    }
    realm
}

fn check(realm: &mut Realm, source: &str) {
    assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
}

#[test]
fn symbol_primitives_have_identity_equality_truthiness_and_typeof() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "typeof s==='symbol' && s===same && s==same && s!==other && s!=other && Object.is(s,same) && !Object.is(s,other)",
    );
    check(
        &mut realm,
        "Boolean(s) && !!s && s!=true && s!=false && s!=0 && s!='name' && s!=1n && s!=undefined && s!=null",
    );
    check(
        &mut realm,
        "({valueOf:()=>s})==s && s==({valueOf:()=>s}) && ({valueOf:()=>other})!=s",
    );
    check(
        &mut realm,
        "[s].includes(same) && [s].indexOf(same)===0 && [s].lastIndexOf(other)===-1",
    );
    check(
        &mut realm,
        "[s].toSorted()[0]===s && [undefined,s].sort()[0]===s",
    );
    realm.collect(10_000).unwrap();
    check(&mut realm, "s===same && s!==other");
}

#[test]
fn implicit_numeric_and_string_conversions_throw_type_errors_without_panicking() {
    for source in [
        "Number(s)",
        "+s",
        "-s",
        "s*1",
        "s+1",
        "s+1n",
        "1n+s",
        "s<0",
        "0<s",
        "s<1n",
        "1n<s",
        "s<=s",
        "s>s",
        "s>=s",
        "''+s",
        "s+''",
        "`${s}`",
        "parseInt(s)",
        "parseFloat(s)",
        "[s].join()",
        "[s,s].sort()",
        "[s,other].toSorted()",
        "[2,1].sort(()=>s)",
        "new String(s)",
        "String({toString:()=>s})",
    ] {
        assert!(
            matches!(
                realm_with_symbols().eval(source),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{source}"
        );
    }
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let caught=false;try{+s;}catch(e){caught=e instanceof TypeError;}caught",
    );
    check(
        &mut realm,
        "!Number.isNaN(s) && !Number.isFinite(s) && !Number.isInteger(s)",
    );
}

#[test]
fn explicit_string_calls_use_descriptive_utf16_strings() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "String(s)==='Symbol(name)' && String(same)==='Symbol(name)' && String(other)==='Symbol(name)'",
    );
    check(
        &mut realm,
        "String(unnamed)==='Symbol()' && String(empty)==='Symbol()' && unnamed!==empty",
    );
    check(&mut realm, "String(raw)==='Symbol(\\uD800\\u0000\\uDC00)'");
    check(
        &mut realm,
        "String.call(null,s)==='Symbol(name)' && String.bind(null,s)()==='Symbol(name)'",
    );
}

#[test]
fn comparison_coercion_order_and_abrupt_completions_are_preserved() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let log='',a={valueOf:()=>{log+='a';return s;}},b={valueOf:()=>{log+='b';return 1;}};let caught=false;try{a<b;}catch(e){caught=e instanceof TypeError;}caught && log==='ab'",
    );
    assert_eq!(
        realm.eval("s==({valueOf:()=>{throw 7;}})"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    assert_eq!(
        realm.eval("s<({valueOf:()=>{throw 7;}})"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
}

#[test]
fn descriptive_output_is_bounded_and_symbol_copies_do_not_copy_descriptions() {
    let mut realm = realm_with_symbols();
    realm.eval("let flag=0").unwrap();
    realm.limits.max_string_units = 8;
    assert!(matches!(
        realm.eval("try{String(s);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    check(&mut realm, "flag===0 && s===same");
    realm.limits.max_string_units = 1_048_576;
    let huge = JsSymbol::new(Some(JsString::from_code_units(vec![0x61; 100_000])));
    let mut budget = Budget::new(1);
    assert!(budget.value(&Value::Symbol(huge.clone())).is_ok());
    realm.remaining_steps = 10;
    assert!(matches!(
        realm.symbol_descriptive_string(&huge, Span::new(0, 0)),
        Err(Error::Limit { .. })
    ));
    realm.remaining_steps = 100_000;
    check(&mut realm, "String(s)==='Symbol(name)'");
}

#[test]
fn symbol_global_wrappers_and_realm_key_operations_remain_explicit_gaps() {
    for source in [
        "Symbol",
        "Object(s)",
        "s.description",
        "s.toString",
        "s.valueOf",
        "s.constructor",
        "Object.prototype.toString.call(s)",
        "({})[s]",
        "({[s]:1})",
        "Object.defineProperty({},s,{})",
        "s in {}",
        "({})[{toString:()=>s}]",
    ] {
        assert!(
            matches!(
                realm_with_symbols().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
}
