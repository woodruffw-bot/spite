use super::*;

#[test]
fn scanner_and_integer_failures_abort_and_restore_call_state() {
    let mut realm = Realm::default();
    realm
        .eval("let flag=0;let text='12345678901234567890'")
        .unwrap();
    realm.limits.max_bigint_bits = 8;
    assert!(matches!(
        realm.eval("try{parseInt(text);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(realm.eval("parseInt('12')"), Ok(Value::Number(12.0)));
    realm.remaining_steps = 0;
    assert!(matches!(
        realm.parse_float(Value::String(JsString::from("123")), Span::new(0, 0)),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("parseFloat('1.5')"), Ok(Value::Number(1.5)));
}

#[test]
fn internally_converted_strings_obey_the_string_limit() {
    let mut realm = Realm::default();
    realm.eval("let flag=0").unwrap();
    realm.limits.max_string_units = 3;
    for function in ["parseInt", "parseFloat"] {
        assert!(matches!(
            realm.eval(&format!(
                "try{{{function}(1234n);}}catch{{flag=1;}}finally{{flag=2;}}"
            )),
            Err(Error::Limit { .. })
        ));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
}
