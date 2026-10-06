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

#[test]
fn utc_string_output_uses_exact_year_width_for_opt_in_quotas() {
    let mut realm = Realm::default();
    realm
        .eval("let d=new Date(0),lo=new Date(-8640000000000000);let flag=0")
        .unwrap();
    realm.limits.max_string_units = Some(29);
    assert_eq!(
        realm.eval("d.toUTCString()"),
        Ok(Value::String(JsString::from(
            "Thu, 01 Jan 1970 00:00:00 GMT"
        )))
    );
    assert!(matches!(
        realm.eval("try{lo.toUTCString();}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(
        realm.eval("lo.getTime()"),
        Ok(Value::Number(-8640000000000000.0))
    );
    for (text, length) in [
        ("9999-01-01", 29),
        ("+010000-01-01", 30),
        ("+100000-01-01", 31),
        ("-000001-01-01", 30),
        ("-012345-01-01", 31),
        ("-123456-01-01", 32),
    ] {
        realm.limits.max_string_units = None;
        realm
            .eval(&format!("d.setTime(Date.parse('{text}'))"))
            .unwrap();
        realm.limits.max_string_units = Some(length - 1);
        assert!(
            matches!(realm.eval("d.toUTCString()"), Err(Error::Limit { .. })),
            "{text}"
        );
        realm.limits.max_string_units = Some(length);
        let Value::String(output) = realm.eval("d.toUTCString()").unwrap() else {
            panic!("UTC string");
        };
        assert_eq!(output.len(), length, "{text}");
    }
    realm.limits.max_string_units = None;
    assert_eq!(
        realm.eval("lo.toUTCString()"),
        Ok(Value::String(JsString::from(
            "Tue, 20 Apr -271821 00:00:00 GMT"
        )))
    );
}

#[test]
fn utc_number_calendar_work_ignores_bigint_value_quotas_and_preserves_opt_in_work_failures() {
    let mut realm = Realm::default();
    realm.limits.max_bigint_bits = Some(0);
    assert_eq!(
        realm.eval("Date.UTC(-Number.MAX_VALUE/12,Number.MAX_VALUE,1)"),
        Ok(Value::Number(-62146137600000.0))
    );
    realm.remaining_steps = Some(1000);
    assert!(matches!(
        realm.date_utc(
            vec![
                Value::Number(-f64::MAX / 12.0),
                Value::Number(f64::MAX),
                Value::Number(1.0)
            ]
            .into_iter(),
            Span { start: 0, end: 0 },
        ),
        Err(Error::Limit { .. })
    ));
    realm.remaining_steps = None;
    assert_eq!(
        realm.eval("Date.UTC(-Number.MAX_VALUE/12,Number.MAX_VALUE,1)"),
        Ok(Value::Number(-62146137600000.0))
    );
}

#[test]
fn utc_calendar_setter_work_failure_does_not_write_the_date_slot() {
    for method in [Method::SetUtcMonth, Method::SetUtcFullYear] {
        let mut realm = Realm::default();
        let Value::Object(object) = realm.eval("var d=new Date(7);d").unwrap() else {
            panic!("Date object");
        };
        let arguments = if matches!(method, Method::SetUtcMonth) {
            vec![Value::Number(f64::MAX), Value::Number(1.0)]
        } else {
            vec![
                Value::Number(-f64::MAX / 12.0),
                Value::Number(f64::MAX),
                Value::Number(1.0),
            ]
        };
        realm.remaining_steps = Some(1000);
        assert!(matches!(
            realm.date_set_utc_calendar(
                method,
                object.clone(),
                7.0,
                arguments.into_iter(),
                Span { start: 0, end: 0 },
            ),
            Err(Error::Limit { .. })
        ));
        realm.remaining_steps = None;
        assert_eq!(
            realm.inspect_object(&object).unwrap().date_value(),
            Some(7.0)
        );
        assert_eq!(realm.eval("d.getTime()"), Ok(Value::Number(7.0)));
        realm.limits.max_bigint_bits = Some(0);
        assert_eq!(
            realm.eval("d.setUTCFullYear(-Number.MAX_VALUE/12,Number.MAX_VALUE,1)"),
            Ok(Value::Number(-62146137599993.0))
        );
    }
}
