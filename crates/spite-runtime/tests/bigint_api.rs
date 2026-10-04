//! BigInt constructor conversions, wrapper brands, prototype hooks, and quotas.

mod common;

use common::REALM_ENTRIES;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn constructor_converts_booleans_integral_numbers_and_integer_strings() {
    for (input, expected) in [
        ("true", "1n"),
        ("false", "0n"),
        ("-0", "0n"),
        ("42", "42n"),
        ("-42", "-42n"),
        ("9007199254740993", "9007199254740992n"),
        ("''", "0n"),
        ("'  +42  '", "42n"),
        ("'-42'", "-42n"),
        ("'0xFf'", "255n"),
        ("'0O10'", "8n"),
        ("'0B11'", "3n"),
        ("'\u{feff}\u{a0}'", "0n"),
        ("'010'", "10n"),
        ("42n", "42n"),
    ] {
        check(&format!("BigInt({input})==={expected}"));
    }
    check(
        "BigInt(Number.MAX_VALUE)===(2n**1024n-2n**971n) && BigInt(-Number.MAX_VALUE)===-(2n**1024n-2n**971n)",
    );
}

#[test]
fn constructor_uses_number_hint_once_and_preserves_abrupt_results() {
    check(
        "let calls=0,o={[Symbol.toPrimitive](hint){calls++;if(hint!=='number')throw 7;return 8;}};BigInt(o)===8n && calls===1",
    );
    check(
        "let log='',o={valueOf(){log+='v';return {};},toString(){log+='s';return '7';}};BigInt(o)===7n && log==='vs'",
    );
    check(
        "let called=0,o={valueOf(){called++;throw 7;}};let caught=false;try{BigInt(o);}catch(e){caught=e===7;}caught && called===1",
    );
    check(
        "let called=0,o={valueOf(){called++;return 1;}};let caught=false;try{new BigInt(o);}catch(e){caught=e instanceof TypeError;}caught && called===0",
    );
    check(
        "BigInt(Object(7n))===7n && BigInt(new Number(8))===8n && BigInt(new Boolean(true))===1n",
    );
}

#[test]
fn constructor_distinguishes_range_type_and_syntax_errors() {
    for input in [
        "NaN",
        "Infinity",
        "-Infinity",
        "0.5",
        "-1.5",
        "Number.MIN_VALUE",
    ] {
        assert!(
            matches!(
                Realm::default().eval(&format!("BigInt({input})")),
                Err(Error::Exception {
                    kind: ExceptionKind::RangeError,
                    ..
                })
            ),
            "{input}"
        );
    }
    for input in [
        "",
        "undefined",
        "null",
        "Symbol()",
        "Object.create(null)",
        "{[Symbol.toPrimitive](){return {};}}",
    ] {
        assert!(
            matches!(
                Realm::default().eval(&format!("BigInt({input})")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{input}"
        );
    }
    for input in [
        "1.5",
        "1e3",
        "1n",
        "1_000",
        "+0x1",
        "-0b1",
        "0x",
        "+",
        "Infinity",
        "123\\uD800",
    ] {
        check(&format!(
            "let caught=false;try{{BigInt('{input}');}}catch(e){{caught=e instanceof SyntaxError;}}caught"
        ));
    }
}

#[test]
fn wrapper_brands_survive_prototype_changes_and_do_not_coerce_impostors() {
    check(
        "Object.getOwnPropertyNames(Object.getOwnPropertyDescriptors(1n)).length===0 && Object.values(1n).length===0",
    );
    check(
        "let a=Object(7n),b=new Object(7n);a!==b && a instanceof BigInt && a.valueOf()===7n && Object.keys(a).length===0 && Object.getOwnPropertyNames(a).length===0",
    );
    check(
        "let a=Object(7n);Object.setPrototypeOf(a,null);BigInt.prototype.valueOf.call(a)===7n && BigInt.prototype.toString.call(a)==='7' && Object.prototype.toString.call(a)==='[object Object]'",
    );
    for input in [
        "BigInt.prototype",
        "Object.create(BigInt.prototype)",
        "{}",
        "7",
        "'7'",
        "true",
        "null",
        "undefined",
        "Symbol()",
        "new Number(7)",
    ] {
        check(&format!(
            "let caught=false;try{{BigInt.prototype.valueOf.call({input});}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    check(
        "let calls=0,o={[Symbol.toPrimitive](){calls++;return 7n;}},r={valueOf(){calls++;return 10;}};let caught=false;try{BigInt.prototype.toString.call(o,r);}catch(e){caught=e instanceof TypeError;}caught && calls===0",
    );
}

#[test]
fn radix_formatting_and_non_402_locale_fallback_follow_brand_and_conversion_order() {
    for (radix, expected) in [
        ("undefined", "-255"),
        ("2", "-11111111"),
        ("8", "-377"),
        ("16", "-ff"),
        ("36", "-73"),
        ("16.9", "-ff"),
        ("'16'", "-ff"),
    ] {
        check(&format!("(-255n).toString({radix})==='{expected}'"));
    }
    check(
        "let calls=0,r={valueOf(){calls++;return 16;}};Object(255n).toString(r)==='ff' && calls===1 && (0n).toString(2)==='0'",
    );
    for radix in ["NaN", "0", "1", "37", "Infinity", "-Infinity", "null"] {
        check(&format!(
            "let caught=false;try{{(0n).toString({radix});}}catch(e){{caught=e instanceof RangeError;}}caught"
        ));
    }
    check(
        "let calls=0,o={valueOf(){calls++;throw 7;}};(-123n).toLocaleString(o,o)==='-123' && calls===0 && [1n,2n].toLocaleString()==='1,2'",
    );
    check(
        "let caught=false;try{(1n).toString(10n);}catch(e){caught=e instanceof TypeError;}caught",
    );
}

#[test]
fn non_strict_calls_box_bigints_and_strict_accessors_keep_primitive_receivers() {
    check(
        "function f(){return this;}let a=f.call(7n),b=f.call(7n);a!==b && a instanceof BigInt && a.valueOf()===7n && f.bind(8n)().valueOf()===8n",
    );
    check("function f(){'use strict';return this;}f.call(7n)===7n");
    check(
        "let received,assigned;Object.defineProperty(BigInt.prototype,'x',{get(){'use strict';return this;},set(v){'use strict';received=this;assigned=v;}});(7n).x=8; (7n).x===7n && received===7n && assigned===8",
    );
    check(
        "Object.defineProperty(BigInt.prototype,'x',{get(){return this;}});(7n).x instanceof BigInt && (7n).x.valueOf()===7n",
    );
}

#[test]
fn observable_tags_and_ordinary_coercions_use_the_exposed_prototype() {
    check(
        "Object.prototype.toString.call(7n)==='[object BigInt]' && Object.prototype.toString.call(Object(7n))==='[object BigInt]' && Object.prototype.toString.call(BigInt.prototype)==='[object BigInt]'",
    );
    check(
        "delete BigInt.prototype[Symbol.toStringTag];Object.prototype.toString.call(7n)==='[object Object]' && Object.prototype.toString.call(Object(7n))==='[object Object]'",
    );
    check(
        "let calls=0;Object.defineProperty(BigInt.prototype,Symbol.toStringTag,{get(){'use strict';calls++;if(typeof this!=='object' || this.valueOf()!==7n)throw 8;return 'Tag';}});Object.prototype.toString.call(7n)==='[object Tag]' && calls===1",
    );
    check(
        "Object(7n)+1n===8n && String(Object(7n))==='7' && Number(Object(7n))===7 && Object(7n)==7n",
    );
}

#[test]
fn intrinsics_have_standard_descriptors_and_survive_collection() {
    check(
        "BigInt.name==='BigInt' && BigInt.length===1 && BigInt.prototype.constructor===BigInt && BigInt.prototype.toString.length===1 && BigInt.prototype.valueOf.length===0 && BigInt.prototype.toLocaleString.length===0",
    );
    check(
        "let d=Object.getOwnPropertyDescriptor(BigInt,'prototype'),t=Object.getOwnPropertyDescriptor(BigInt.prototype,Symbol.toStringTag);!d.writable && !d.enumerable && !d.configurable && !t.writable && !t.enumerable && t.configurable && t.value==='BigInt'",
    );
    check(
        "Object.getOwnPropertyNames(BigInt.prototype).sort().join(',')==='constructor,toLocaleString,toString,valueOf' && Object.freeze(BigInt.prototype)===BigInt.prototype && Object.isFrozen(BigInt.prototype)",
    );
    let mut realm = Realm::default();
    realm
        .eval("let b=Object(12345678901234567890n);delete globalThis.BigInt")
        .unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES + 1);
    assert_eq!(realm.eval("b.valueOf()===12345678901234567890n && b.toString()==='12345678901234567890' && b instanceof b.constructor"),Ok(Value::Boolean(true)));
}

#[test]
fn opted_in_integer_string_and_heap_quotas_remain_host_failures() {
    for (limits, source) in [
        (
            Limits {
                max_bigint_bits: Some(8),
                ..Limits::default()
            },
            "BigInt(256)",
        ),
        (
            Limits {
                max_bigint_bits: Some(8),
                ..Limits::default()
            },
            "BigInt('256')",
        ),
        (
            Limits {
                max_string_units: Some(3),
                ..Limits::default()
            },
            "(1000n).toString()",
        ),
        (
            Limits {
                max_heap_entries: Some(REALM_ENTRIES),
                ..Limits::default()
            },
            "Object(1n)",
        ),
    ] {
        let mut realm = Realm::new(limits);
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
    }
}
