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
        Some(&realm.record.intrinsics.as_ref().unwrap().string.prototype)
    );
}

#[test]
fn string_output_and_work_limits_skip_language_handlers() {
    let mut realm = Realm::default();
    realm
        .eval("let flag=0,s=Object('abcdef'),convert=String.prototype.valueOf")
        .unwrap();
    realm.limits.max_string_units = Some(3);
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
    realm.limits.max_string_units = Some(1000);
    let input = Value::String(JsString::from("x".repeat(100).as_str()));
    realm.remaining_steps = Some(50);
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
    realm.limits.max_string_units = Some(1);
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
        realm.remaining_steps = Some(50);
        assert!(matches!(
            realm.string_well_formed(input.clone(), replace, Span::new(0, 0)),
            Err(Error::Limit { .. })
        ));
    }
    realm.remaining_steps = Some(1000);
    realm.limits.max_string_units = Some(99);
    for limited in [
        input.clone(),
        Value::String(JsString::from("x".repeat(100).as_str())),
        Value::String(JsString::from("💩".repeat(50).as_str())),
    ] {
        realm.remaining_steps = Some(1000);
        assert!(matches!(
            realm.string_well_formed(limited, true, Span::new(0, 0)),
            Err(Error::Limit { .. })
        ));
    }
    realm.limits.max_string_units = Some(100);
    assert_eq!(
        realm.string_well_formed(input, true, Span::new(0, 0)),
        Ok(Value::String(JsString::from_code_units(vec![0xfffd; 100])))
    );
}

#[test]
fn sequence_methods_bound_copying_and_output_lengths() {
    let mut realm = Realm::default();
    let input = Value::String(JsString::from("a".repeat(100).as_str()));
    let span = Span::new(0, 0);
    realm.remaining_steps = Some(50);
    assert!(matches!(
        realm.string_concat(input.clone(), vec![].into_iter(), span),
        Err(Error::Limit { .. })
    ));
    for relative in [false, true] {
        realm.remaining_steps = Some(50);
        assert!(matches!(
            realm.string_substring(
                input.clone(),
                Value::Number(0.0),
                Value::Undefined,
                relative,
                span
            ),
            Err(Error::Limit { .. })
        ));
        realm.remaining_steps = Some(1000);
        realm.limits.max_string_units = Some(99);
        assert!(matches!(
            realm.string_substring(
                input.clone(),
                Value::Number(0.0),
                Value::Undefined,
                relative,
                span
            ),
            Err(Error::Limit { .. })
        ));
        assert_eq!(
            realm.string_substring(
                input.clone(),
                Value::Number(0.0),
                Value::Number(1.0),
                relative,
                span
            ),
            Ok(Value::String(JsString::from("a")))
        );
        realm.limits.max_string_units = Some(100);
    }
    realm.remaining_steps = Some(1000);
    assert!(matches!(
        realm.string_concat(
            input.clone(),
            vec![Value::String(JsString::from("b"))].into_iter(),
            span
        ),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.string_concat(
            input.clone(),
            vec![Value::String(JsString::from(""))].into_iter(),
            span
        ),
        Ok(input)
    );
}

#[test]
fn whole_functional_replacement_retains_returned_storage_and_output_checks() {
    let mut realm = Realm::default();
    let span = Span::new(0, 0);
    let callback = realm
        .eval("let calls=0;const returned='x\\uD800\\u4E2D\\uDC00'.repeat(32);function replacement(){calls++;return returned}replacement")
        .unwrap();
    let Value::String(expected) = realm.eval("returned").unwrap() else {
        panic!("replacement text");
    };
    for input in ["a", ""] {
        let Value::String(result) = realm
            .string_replace(
                Value::String(JsString::from(input)),
                Value::String(JsString::from(input)),
                callback.clone(),
                span,
            )
            .unwrap()
        else {
            panic!("replace result");
        };
        assert_eq!(result, expected);
        assert_eq!(result.code_units().as_ptr(), expected.code_units().as_ptr());
    }
    realm.limits.max_string_units = Some(expected.len() - 1);
    assert!(matches!(
        realm.string_replace(
            Value::String(JsString::from("a")),
            Value::String(JsString::from("a")),
            callback,
            span,
        ),
        Err(Error::Limit { message, .. }) if message == "string length limit exceeded"
    ));
    realm.limits.max_string_units = None;
    assert_eq!(realm.eval("calls"), Ok(Value::Number(3.0)));
}

#[test]
fn splitting_without_matches_retains_source_storage_and_output_checks() {
    let mut realm = Realm::default();
    realm.eval("").unwrap();
    let span = Span::new(0, 0);
    let string = JsString::from_code_units(vec![0x61, 0xd800, 0x4e2d, 0xdc00]);
    for separator in [",", "long separator"] {
        realm.remaining_steps = Some(1000);
        let Value::Object(array) = realm
            .string_split(
                Value::String(string.clone()),
                Value::String(JsString::from(separator)),
                Value::Undefined,
                span,
            )
            .unwrap()
        else {
            panic!("split result");
        };
        let Value::String(part) = realm
            .get_property(&array, &JsString::from("0"), span)
            .unwrap()
        else {
            panic!("split substring");
        };
        assert_eq!(part, string);
        assert_eq!(part.code_units().as_ptr(), string.code_units().as_ptr());
    }
    realm.remaining_steps = Some(1000);
    realm.limits.max_string_units = Some(string.len() - 1);
    assert!(matches!(
        realm.string_split(
            Value::String(string),
            Value::String(JsString::from(",")),
            Value::Undefined,
            span,
        ),
        Err(Error::Limit { .. })
    ));
}

#[test]
fn trimming_bounds_scans_and_copies_only_the_result() {
    let mut realm = Realm::default();
    let span = Span::new(0, 0);
    let input = Value::String(JsString::from(" ".repeat(100).as_str()));
    for builtin in [
        Builtin::StringTrim,
        Builtin::StringTrimStart,
        Builtin::StringTrimEnd,
    ] {
        realm.remaining_steps = Some(50);
        assert!(matches!(
            realm.string_trim(builtin, input.clone(), span),
            Err(Error::Limit { .. })
        ));
        realm.remaining_steps = Some(1000);
        realm.limits.max_string_units = Some(0);
        assert_eq!(
            realm.string_trim(builtin, input.clone(), span),
            Ok(Value::String(JsString::from("")))
        );
        assert!(matches!(
            realm.string_trim(builtin, Value::String(JsString::from(" a ")), span),
            Err(Error::Limit { .. })
        ));
    }
}

#[test]
fn repetition_and_padding_charge_output_work_before_allocating() {
    let mut realm = Realm::default();
    let input = Value::String(JsString::from("a"));
    let span = Span::new(0, 0);
    realm.remaining_steps = Some(50);
    assert!(matches!(
        realm.string_repeat(input.clone(), Value::Number(100.0), span),
        Err(Error::Limit { .. })
    ));
    for at_start in [false, true] {
        realm.remaining_steps = Some(50);
        assert!(matches!(
            realm.string_pad(
                input.clone(),
                Value::Number(100.0),
                Value::Undefined,
                at_start,
                span
            ),
            Err(Error::Limit { .. })
        ));
    }
    // Platform capacity overflow must be a host limit, even if the embedding
    // disables practical work/length bounds. This cannot request real storage.
    realm.remaining_steps = None;
    realm.limits.max_string_units = None;
    let count = (usize::MAX / 2 + 1) as f64;
    assert!(matches!(
        realm.string_repeat(input, Value::Number(count), span),
        Err(Error::Limit { .. })
    ));
}

#[test]
fn substring_search_observes_agent_termination_without_a_work_quota() {
    let input = JsString::from("zzabzzab");
    let search = JsString::from("ab");
    for backwards in [false, true] {
        let mut realm = Realm::default();
        realm.agent_control().terminate();
        assert_eq!(realm.remaining_steps, None);
        assert!(matches!(
            realm.find_string(&input, &search, if backwards { 8 } else { 0 }, backwards, Span::new(0, 0)),
            Err(Error::Host { message, .. }) if message == "agent terminated by host"
        ));
    }
}

#[test]
fn substring_search_bounds_repeated_candidate_comparisons() {
    let mut realm = Realm::default();
    let input = Value::String(JsString::from("a".repeat(100).as_str()));
    let search = Value::String(JsString::from(format!("{}b", "a".repeat(39)).as_str()));
    for backwards in [false, true] {
        realm.remaining_steps = Some(500);
        assert!(matches!(
            realm.string_index_of(
                input.clone(),
                search.clone(),
                Value::Undefined,
                backwards,
                Span::new(0, 0)
            ),
            Err(Error::Limit { .. })
        ));
        realm.remaining_steps = Some(500);
        let expected = if backwards { 60.0 } else { 0.0 };
        assert_eq!(
            realm.string_index_of(
                input.clone(),
                Value::String(JsString::from("a".repeat(40).as_str())),
                Value::Undefined,
                backwards,
                Span::new(0, 0)
            ),
            Ok(Value::Number(expected))
        );
    }
}

#[test]
fn search_predicates_charge_comparisons_before_inspecting_code_units() {
    let mut realm = Realm::default();
    let input = Value::String(JsString::from("a".repeat(100).as_str()));
    for builtin in [
        Builtin::StringIncludes,
        Builtin::StringStartsWith,
        Builtin::StringEndsWith,
    ] {
        realm.remaining_steps = Some(50);
        assert!(matches!(
            realm.string_search_predicate(
                builtin,
                input.clone(),
                input.clone(),
                Value::Undefined,
                Span::new(0, 0)
            ),
            Err(Error::Limit { .. })
        ));
    }
}

#[test]
fn single_repetition_keeps_output_work_and_agent_checks() {
    let span = Span::new(0, 0);
    let input = Value::String(JsString::from_code_units(vec![0xd800, 0, 0x4e2d, 0xdc00]));
    for count in [1.0, 1.75] {
        let mut realm = Realm::default();
        realm.limits.max_string_units = Some(4);
        realm.remaining_steps = Some(5);
        assert_eq!(
            realm.string_repeat(input.clone(), Value::Number(count), span),
            Ok(input.clone())
        );
        assert_eq!(realm.remaining_steps, Some(0));
        realm.remaining_steps = Some(4);
        assert!(matches!(
            realm.string_repeat(input.clone(), Value::Number(count), span),
            Err(Error::Limit { .. })
        ));
        // Length rejection precedes work debit, as in repeated_string_buffer.
        realm.limits.max_string_units = Some(3);
        realm.remaining_steps = Some(5);
        assert!(
            matches!(realm.string_repeat(input.clone(), Value::Number(count), span), Err(Error::Limit { message, .. }) if message == "string allocation limit exceeded")
        );
        assert_eq!(realm.remaining_steps, Some(5));
        realm.limits.max_string_units = Some(4);
        realm.remaining_steps = None;
        realm.agent_control().terminate();
        assert!(
            matches!(realm.string_repeat(input.clone(), Value::Number(count), span), Err(Error::Host { message, .. }) if message == "agent terminated by host")
        );
    }
}
