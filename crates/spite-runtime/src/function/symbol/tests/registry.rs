use super::*;

#[test]
fn registered_symbols_are_shared_by_key_and_separate_from_fresh_and_well_known_symbols() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let S=s.constructor,a=S.for('registry.basic');a===S.for('registry.basic') && a!==S('registry.basic') && a.description==='registry.basic' && S.keyFor(a)==='registry.basic'",
    );
    check(
        &mut realm,
        "S.for()===S.for(undefined) && S.for()===S.for('undefined') && S.keyFor(S.for())==='undefined' && S.keyFor(S.for(''))===''",
    );
    check(
        &mut realm,
        "S.keyFor(s)===undefined && S.keyFor(S.iterator)===undefined && S.for('Symbol.iterator')!==S.iterator && S.keyFor(S.for('Symbol.iterator'))==='Symbol.iterator'",
    );
    check(
        &mut realm,
        "S.for(1n)===S.for(1) && S.for(1)===S.for('1') && S.for(null)===S.for('null')",
    );
    check(
        &mut realm,
        "let rawKey='\\uD800\\u0000\\uDC00',rawSymbol=S.for(rawKey);S.keyFor(rawSymbol)===rawKey && rawSymbol.description===rawKey && rawSymbol!==S.for('\\uFFFD\\u0000\\uFFFD') && S.for('é')!==S.for('e\\u0301')",
    );
    realm.collect(30_000).unwrap();
    check(
        &mut realm,
        "a===S.for('registry.basic') && S.keyFor(a)==='registry.basic'",
    );
}

#[test]
fn registry_conversion_is_reentrant_and_key_for_never_coerces_its_argument() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let S=s.constructor,log='';let o={[convert]:(hint)=>{log+=hint;return S.keyFor(S.for('registry.reentrant'));}};S.for(o)===S.for('registry.reentrant') && log==='string'",
    );
    assert_eq!(
        realm.eval("S.for({toString:()=>{throw 7;}})"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    assert!(matches!(
        realm.eval("S.for(s)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    for argument in [
        "undefined",
        "null",
        "false",
        "0",
        "1n",
        "'x'",
        "Object(s)",
        "{[convert]:()=>{throw 9;}}",
    ] {
        assert!(
            matches!(
                realm.eval(&format!("S.keyFor({argument})")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{argument}"
        );
    }
    check(
        &mut realm,
        "S.for.call(null,'registry.receiver')===S.for('registry.receiver') && S.keyFor.call({},S.for('registry.receiver'))==='registry.receiver'",
    );
}

#[test]
fn registered_identity_survives_realm_destruction_and_is_atomic_across_threads() {
    fn registered(barrier: Option<std::sync::Arc<std::sync::Barrier>>) -> JsSymbol {
        let mut realm = realm_with_symbols();
        if let Some(barrier) = barrier {
            barrier.wait();
        }
        let Value::Symbol(symbol) = realm
            .eval("s.constructor.for('registry.thread-sharing')")
            .unwrap()
        else {
            panic!("symbol");
        };
        symbol
    }
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(8));
    let mut threads: Vec<_> = (0..8)
        .map(|_| {
            let barrier = barrier.clone();
            std::thread::spawn(move || registered(Some(barrier)))
        })
        .collect();
    let first = threads.remove(0).join().unwrap();
    for thread in threads {
        assert_eq!(thread.join().unwrap(), first);
    }
    assert_eq!(registered(None), first);
    let mut realm = realm_with_symbols();
    assert_eq!(
        realm.symbol_key_for(Value::Symbol(first), Span::new(0, 0)),
        Ok(Value::String(JsString::from("registry.thread-sharing")))
    );
}

#[test]
fn registry_methods_have_standard_metadata_and_complete_constructor_enumeration() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let S=s.constructor,ds=Object.getOwnPropertyDescriptors(S);ds.for.value===S.for && ds.keyFor.value===S.keyFor && ds.iterator.value===S.iterator && ds.prototype.value===S.prototype",
    );
    for name in ["for", "keyFor"] {
        check(
            &mut realm,
            &format!(
                "let d{name}=ds.{name};d{name}.writable && !d{name}.enumerable && d{name}.configurable && S.{name}.name==='{name}' && S.{name}.length===1 && !Object.hasOwn(S.{name},'prototype')"
            ),
        );
        assert!(matches!(
            realm.eval(&format!("new S.{name}")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
}

#[test]
fn recursive_registry_coercion_and_key_output_are_bounded_without_lock_reentry() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let mut realm = realm_with_symbols();
            realm
                .eval("let S=s.constructor,o={toString:()=>S.for(o)},flag=0")
                .unwrap();
            assert!(matches!(
                realm.eval("try{S.for(o);}catch{flag=1;}finally{flag=2;}"),
                Err(Error::Limit { .. })
            ));
            check(
                &mut realm,
                "flag===0 && S.for('registry.recovered')===S.for('registry.recovered')",
            );
        })
        .unwrap()
        .join()
        .unwrap();
    let mut realm = realm_with_symbols();
    realm
        .eval("let S=s.constructor,large=S.for('registry.output-limit'),flag=0")
        .unwrap();
    realm.limits.max_string_units = 8;
    assert!(matches!(
        realm.eval("try{S.keyFor(large);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    check(&mut realm, "flag===0");
    realm.limits.max_string_units = 1_048_576;
    check(
        &mut realm,
        "S.keyFor(large)==='registry.output-limit' && S.for('registry.output-limit')===large",
    );
}
