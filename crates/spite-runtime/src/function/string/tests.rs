use super::*;

#[test]
fn new_target_prototype_is_observed_after_string_conversion_with_intrinsic_fallback() {
    let mut realm = Realm::default();
    let Value::Object(target)=realm.eval("let log='',p={},F=(function(){}).bind(null);Object.defineProperty(F,'prototype',{get:()=>{log+='p';return p;},configurable:true});F").unwrap() else {panic!("target");};
    let input = realm
        .eval("({toString:()=>{log+='v';return 'x';}})")
        .unwrap();
    let Value::Object(value) = realm
        .string_constructor(Some(target.clone()), Some(input), Span::new(0, 0))
        .unwrap()
    else {
        panic!("wrapper");
    };
    let Value::Object(prototype) = realm.eval("p").unwrap() else {
        panic!("prototype");
    };
    assert_eq!(
        realm.inspect_object(&value).unwrap().prototype(),
        Some(&prototype)
    );
    assert_eq!(
        realm.inspect_object(&value).unwrap().string_data(),
        Some(&JsString::from("x"))
    );
    assert_eq!(realm.eval("log"), Ok(Value::String(JsString::from("vp"))));
    realm
        .eval("Object.defineProperty(F,'prototype',{value:null})")
        .unwrap();
    let Value::Object(value) = realm
        .string_constructor(Some(target), None, Span::new(0, 0))
        .unwrap()
    else {
        panic!("wrapper");
    };
    assert_eq!(
        realm.inspect_object(&value).unwrap().prototype(),
        Some(&realm.intrinsics.as_ref().unwrap().string.prototype)
    );
}

#[test]
fn string_output_and_work_limits_skip_language_handlers() {
    let mut realm = Realm::default();
    realm
        .eval("let flag=0,s=Object('abcdef'),convert=String.prototype.valueOf")
        .unwrap();
    realm.limits.max_string_units = 3;
    for expression in [
        "String(1234)",
        "String(1234n)",
        "convert.call(s)",
        "new String(1234)",
    ] {
        assert!(
            matches!(
                realm.eval(&format!(
                    "try{{{expression};}}catch{{flag=1;}}finally{{flag=2;}}"
                )),
                Err(Error::Limit { .. })
            ),
            "{expression}"
        );
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
    assert_eq!(
        realm.eval("String(12)"),
        Ok(Value::String(JsString::from("12")))
    );
    realm.limits.max_string_units = 1000;
    let input = Value::String(JsString::from("x".repeat(100).as_str()));
    realm.remaining_steps = 50;
    assert!(matches!(
        realm.this_string_value(&input, Span::new(0, 0)),
        Err(Error::Limit { .. })
    ));
}

#[test]
fn code_construction_charges_each_utf16_unit_against_output_limits() {
    let mut realm = Realm::default();
    realm
        .eval("let flag=0,f=String.fromCodePoint,g=String.fromCharCode")
        .unwrap();
    realm.limits.max_string_units = 1;
    for expression in ["f(0x10000)", "g(65,66)"] {
        assert!(
            matches!(
                realm.eval(&format!(
                    "try{{{expression};}}catch{{flag=1;}}finally{{flag=2;}}"
                )),
                Err(Error::Limit { .. })
            ),
            "{expression}"
        );
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
    assert_eq!(realm.eval("g(65)"), Ok(Value::String(JsString::from("A"))));
}

#[test]
fn well_formedness_bounds_scans_and_replacement_allocation() {
    let mut realm = Realm::default();
    let input = Value::String(JsString::from_code_units(vec![0xd800; 100]));
    for replace in [false, true] {
        realm.remaining_steps = 50;
        assert!(matches!(
            realm.string_well_formed(input.clone(), replace, Span::new(0, 0)),
            Err(Error::Limit { .. })
        ));
    }
    realm.remaining_steps = 1000;
    realm.limits.max_string_units = 99;
    assert!(matches!(
        realm.string_well_formed(input.clone(), true, Span::new(0, 0)),
        Err(Error::Limit { .. })
    ));
    realm.limits.max_string_units = 100;
    assert_eq!(
        realm.string_well_formed(input, true, Span::new(0, 0)),
        Ok(Value::String(JsString::from_code_units(vec![0xfffd; 100])))
    );
}
