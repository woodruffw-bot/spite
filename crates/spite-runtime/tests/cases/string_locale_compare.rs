//! Canonical equivalence, total ordering, and generic coercion without ECMA-402.

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn canonical_equivalence_returns_positive_zero_for_all_spec_examples() {
    for (left, right) in [
        ("\\u212B", "A\\u030A"),
        ("\\u2126", "\\u03A9"),
        ("\\u1E69", "s\\u0307\\u0323"),
        ("\\u1E0B\\u0323", "\\u1E0D\\u0307"),
        ("\\u1100\\u1161", "\\uAC00"),
        ("\\uAC01", "\\u1100\\u1161\\u11A8"),
        ("\\u{1D15E}", "\\u{1D157}\\u{1D165}"),
        ("\\u{2F82B}", "北"),
    ] {
        check(&format!(
            "Object.is('{left}'.localeCompare('{right}'),0) && Object.is('{right}'.localeCompare('{left}'),0)"
        ));
    }
    check("Object.is(''.localeCompare(''),0) && Object.is('x'.localeCompare('x'),0)");
}

#[test]
fn fixed_host_order_is_consistent_and_preserves_compatibility_distinctions() {
    // NFD makes canonically equivalent spellings interchangeable in comparisons.
    check(
        "let values=['','A','a','a\\u0301','á','b','ﬃ','\\uD800','😀','\\uE000'];let valid=true;for(let a of values){for(let b of values){let ab=a.localeCompare(b),ba=b.localeCompare(a);valid=valid && ab===-ba;for(let c of values){if(ab<=0 && b.localeCompare(c)<=0){valid=valid && a.localeCompare(c)<=0;}}}}valid",
    );
    check(
        "'é'.localeCompare('f')===-1 && 'e\\u0301'.localeCompare('f')===-1 && 'f'.localeCompare('é')===1 && 'a'.localeCompare('aa')===-1",
    );
    check(
        "'ﬃ'.localeCompare('ffi')!==0 && '①'.localeCompare('1')!==0 && 'Ａ'.localeCompare('A')!==0",
    );
    check(
        "'\\uD800'.localeCompare('\\uD801')===-1 && '😀'.localeCompare('\\uE000')===-1 && 'é\\uDC00'.localeCompare('e\\u0301\\uDC00')===0 && 'é\\uD800'.localeCompare('e\\uD800\\u0301')!==0",
    );
}

#[test]
fn coercion_is_ordered_once_with_string_hints_and_preserves_abrupt_results() {
    check(
        "let log='',s={[Symbol.toPrimitive](hint){log+='s'+hint;return 'é';}},t={[Symbol.toPrimitive](hint){log+='t'+hint;return 'e\\u0301';}};String.prototype.localeCompare.call(s,t)===0 && log==='sstringtstring'",
    );
    check(
        "let log='',caught=false;try{String.prototype.localeCompare.call({toString(){log+='s';throw 7;}},{toString(){log+='t';throw 8;}});}catch(e){caught=e===7;}caught && log==='s'",
    );
    check(
        "let log='',caught=false;try{String.prototype.localeCompare.call({toString(){log+='s';return 'x';}},{toString(){log+='t';throw 8;}});}catch(e){caught=e===8;}caught && log==='st'",
    );
    check(
        "let touched=false,caught=false;try{String.prototype.localeCompare.call(null,{toString(){touched=true;}});}catch(e){caught=e instanceof TypeError;}caught && !touched",
    );
    check(
        "String.prototype.localeCompare.call(7,7n)===0 && String.prototype.localeCompare.call(true,'true')===0 && new String('é').localeCompare(new String('e\\u0301'))===0 && 'undefined'.localeCompare()===0",
    );
    for source in [
        "String.prototype.localeCompare.call(undefined,'x')",
        "String.prototype.localeCompare.call(Symbol(),'x')",
        "'x'.localeCompare(Symbol())",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{source}"
        );
    }
}

#[test]
fn reserved_arguments_are_ignored_and_public_normalization_is_not_consulted() {
    check(
        "let log='',reserved={get length(){throw 7;},toString(){throw 8;}};'é'.localeCompare('e\\u0301',(log+='a',reserved),(log+='b',reserved))===0 && log==='ab'",
    );
    check(
        "Object.defineProperty(String.prototype,'normalize',{get(){throw 9;}});'é'.localeCompare('e\\u0301')===0",
    );
    check(
        "let caught=false;try{'x'.localeCompare('x',(()=>{throw 10;})());}catch(e){caught=e===10;}caught",
    );
}

#[test]
fn descriptors_nonconstruction_and_intrinsic_retention_are_standard() {
    check(
        "let d=Object.getOwnPropertyDescriptor(String.prototype,'localeCompare'),n=Object.getOwnPropertyDescriptor(d.value,'name'),l=Object.getOwnPropertyDescriptor(d.value,'length');d.writable && !d.enumerable && d.configurable && n.value==='localeCompare' && !n.writable && !n.enumerable && n.configurable && l.value===1 && !l.writable && !l.enumerable && l.configurable && !Object.hasOwn(d.value,'prototype')",
    );
    assert!(matches!(
        Realm::default().eval("new String.prototype.localeCompare()"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    let mut realm = Realm::default();
    let Value::Object(handle) = realm.eval("String.prototype.localeCompare").unwrap() else {
        panic!("native method");
    };
    realm.eval("let compare=String.prototype.localeCompare;delete String.prototype.localeCompare;delete globalThis.String;").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert!(realm.inspect_object(&handle).is_ok());
    assert_eq!(
        realm.eval("compare.call('é','e\\u0301')===0 && 'x'.localeCompare===undefined"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn large_default_inputs_work_and_opted_in_host_aborts_bypass_cleanup() {
    check(
        "'é'.repeat(10000).localeCompare('e\\u0301'.repeat(10000))===0 && ('x'+'\\u0301\\u0323'.repeat(5000)).localeCompare('x'+'\\u0323'.repeat(5000)+'\\u0301'.repeat(5000))===0",
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0;").unwrap();
    assert!(matches!(
        realm.eval(
            "try{'é'.repeat(1000).localeCompare('é'.repeat(1000));}catch{flag=1;}finally{flag=2;}"
        ),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::default();
    realm.eval("let flag=0;").unwrap();
    assert!(matches!(
        realm.eval(
            "try{'x'.localeCompare({toString(){Function('function* gap(){}');}});}catch{flag=1;}finally{flag=2;}"
        ),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
