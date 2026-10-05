use super::*;

#[test]
fn date_slot_survives_collection_and_stays_independent_of_prototype_identity() {
    let mut realm = Realm::default();
    let value = realm.eval("class D extends Date {} new D(-1)").unwrap();
    let rooted = realm.root_value(value, usize::MAX).unwrap();
    let Value::Object(object) = rooted.value() else {
        panic!("Date object");
    };
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.inspect_object(object).unwrap().date_value(),
        Some(-1.0)
    );
    realm
        .objects
        .set_prototype(object, None, &mut crate::object::Budget::new(usize::MAX))
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.inspect_object(object).unwrap().date_value(),
        Some(-1.0)
    );
}

#[test]
fn intrinsic_property_graph_has_exact_names_lengths_and_descriptors() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval("Date.name==='Date' && Date.length===7 && Object.getPrototypeOf(Date)===Function.prototype && Object.getPrototypeOf(Date.prototype)===Object.prototype && Date.prototype.constructor===Date && Object.getOwnPropertyNames(Date).join(',')==='length,name,prototype,now,parse,UTC' && Object.keys(Date).length===0 && Object.keys(Date.prototype).length===0"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let p=Object.getOwnPropertyDescriptor(Date,'prototype'),g=Object.getOwnPropertyDescriptor(globalThis,'Date'); !p.writable && !p.enumerable && !p.configurable && p.value===Date.prototype && g.writable && !g.enumerable && g.configurable"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let d=Object.getOwnPropertyDescriptor(Date.prototype,Symbol.toPrimitive);!d.writable && !d.enumerable && d.configurable && d.value.name==='[Symbol.toPrimitive]' && d.value.length===1 && Object.getOwnPropertySymbols(Date.prototype).length===1 && Object.getOwnPropertyNames(Date.prototype).length===44"),Ok(Value::Boolean(true)));
    for method in Method::ALL {
        let key = if matches!(method, Method::ToPrimitive) {
            "Symbol.toPrimitive".to_owned()
        } else {
            format!("'{}'", method.name())
        };
        assert_eq!(realm.eval(&format!("(function(){{let f=Date.prototype[{key}],d=Object.getOwnPropertyDescriptor(Date.prototype,{key});return f.name==='{}' && f.length==={} && Object.getPrototypeOf(f)===Function.prototype && !Object.hasOwn(f,'prototype') && !d.enumerable && d.configurable;}})()",method.name(),method.length())),Ok(Value::Boolean(true)));
    }
    assert_eq!(
        realm.eval("delete globalThis.Date; typeof Date==='undefined' && !('Date' in globalThis)"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn fixed_size_iso_output_obeys_opt_in_quotas_and_restores_call_state() {
    let mut realm = Realm::default();
    realm
        .eval("let d=new Date(0),expanded=new Date(8640000000000000);let flag=0")
        .unwrap();
    realm.limits.max_string_units = Some(24);
    assert_eq!(
        realm.eval("d.toISOString()"),
        Ok(Value::String(JsString::from("1970-01-01T00:00:00.000Z")))
    );
    assert!(matches!(
        realm.eval("try{expanded.toISOString();}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(realm.eval("d.getTime()"), Ok(Value::Number(0.0)));
    realm.limits.max_string_units = Some(23);
    assert!(matches!(
        realm.eval("d.toISOString()"),
        Err(Error::Limit { .. })
    ));
    realm.limits.max_string_units = None;
    assert_eq!(
        realm.eval("expanded.toISOString()"),
        Ok(Value::String(JsString::from("+275760-09-13T00:00:00.000Z")))
    );
}
