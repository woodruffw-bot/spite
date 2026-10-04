use super::*;
// Exercise the public constructor and real Script operations on its result.
fn realm_with_array(length: u32) -> Realm {
    let mut realm = Realm::default();
    realm.eval(&format!("let a=Array({length})")).unwrap();
    realm
}

fn check(realm: &mut Realm, source: &str) {
    assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
}

#[test]
fn assignments_convert_strings_booleans_and_null_to_uint32_lengths() {
    for (input, expected) in [
        ("'3'", 3.0),
        ("true", 1.0),
        ("false", 0.0),
        ("null", 0.0),
        ("-0", 0.0),
        ("4294967295", 4294967295.0),
    ] {
        let mut realm = realm_with_array(5);
        realm.eval(&format!("a.length={input}")).unwrap();
        assert_eq!(realm.eval("a.length"), Ok(Value::Number(expected)));
    }
    check(
        &mut realm_with_array(1),
        "a.length=-0;Object.is(a.length,0)",
    );
}

#[test]
fn invalid_lengths_throw_range_errors_and_bigints_throw_type_errors() {
    for input in [
        "undefined",
        "NaN",
        "-1",
        "0.5",
        "Infinity",
        "-Infinity",
        "4294967296",
    ] {
        let mut realm = realm_with_array(5);
        assert!(
            matches!(
                realm.eval(&format!("a.length={input}")),
                Err(Error::Exception {
                    kind: ExceptionKind::RangeError,
                    ..
                })
            ),
            "{input}"
        );
        assert_eq!(realm.eval("a.length"), Ok(Value::Number(5.0)));
    }
    let mut realm = realm_with_array(5);
    assert!(matches!(
        realm.eval("a.length=0n"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert!(matches!(
        realm.eval("Object.defineProperty(a,'length',{value:0n})"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn array_set_length_converts_the_same_original_value_twice() {
    check(
        &mut realm_with_array(5),
        "let n=0,v={valueOf:()=>{n++;return n===1?1.9:1;}};a.length=v;n===2 && a.length===1",
    );
    check(
        &mut realm_with_array(5),
        "let n=0,v={valueOf:()=>{n++;return n===1?-1:4294967295;}};Object.defineProperty(a,'length',{value:v});n===2 && a.length===4294967295",
    );
    check(
        &mut realm_with_array(5),
        "let n=0,v={toString:()=>{n++;return '2';}};Object.defineProperties(a,{length:{value:v}});n===2 && a.length===2",
    );
    check(
        &mut realm_with_array(5),
        "let n=0,v={valueOf:()=>{n++;return 2;}};Object.assign(a,{length:v});n===2 && a.length===2",
    );
    let mut realm = realm_with_array(5);
    realm.eval("let n=0,v={valueOf:()=>++n}").unwrap();
    assert!(matches!(
        realm.eval("a.length=v"),
        Err(Error::Exception {
            kind: ExceptionKind::RangeError,
            ..
        })
    ));
    check(&mut realm, "n===2 && a.length===5");
}

#[test]
fn abrupt_conversion_stops_before_length_changes() {
    for failure in [1, 2] {
        let mut realm = realm_with_array(5);
        realm
            .eval(&format!(
                "let n=0,v={{valueOf:()=>{{n++;if(n==={failure})throw 7;return 2;}}}}"
            ))
            .unwrap();
        assert_eq!(
            realm.eval("a.length=v"),
            Err(Error::Thrown(Value::Number(7.0)))
        );
        check(&mut realm, &format!("n==={failure} && a.length===5"));
    }
}

#[test]
fn assignment_checks_writability_before_coercion_but_definitions_convert_first() {
    let mut realm = realm_with_array(0);
    realm.eval("Object.defineProperty(a,'length',{writable:false});let n=0,v={valueOf:()=>{n++;return 0;}}").unwrap();
    realm.eval("a.length=v").unwrap();
    check(&mut realm, "n===0 && a.length===0");
    assert!(matches!(
        realm.eval("'use strict';a.length=v"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    check(&mut realm, "n===0");
    check(
        &mut realm,
        "Object.defineProperty(a,'length',{value:v})===a && n===2",
    );
    assert_eq!(
        realm.eval("Object.defineProperty(a,'length',{value:{valueOf:()=>{throw 9;}}})"),
        Err(Error::Thrown(Value::Number(9.0)))
    );
    assert!(matches!(
        realm.eval("Object.defineProperty(a,'length',{value:NaN})"),
        Err(Error::Exception {
            kind: ExceptionKind::RangeError,
            ..
        })
    ));
    let mut realm = realm_with_array(5);
    realm
        .eval("let n=0,v={valueOf:()=>{n++;return 2;}}")
        .unwrap();
    assert!(matches!(
        realm.eval("Object.defineProperty(a,'length',{value:v,configurable:true})"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    check(&mut realm, "n===2 && a.length===5");
}

#[test]
fn storage_observes_length_changes_made_during_both_conversions() {
    check(
        &mut realm_with_array(5),
        "let n=0,v={valueOf:()=>{n++;a.length=n===1?4:3;return 2;}};a[4]='old';a.length=v;n===2 && a.length===2 && !('4' in a)",
    );
    check(
        &mut realm_with_array(5),
        "let n=0,v={valueOf:()=>{n++;if(n===2)Object.defineProperty(a,'length',{value:1,writable:false});return 2;}};a.length=v;n===2 && a.length===1 && !Object.getOwnPropertyDescriptor(a,'length').writable",
    );
    let mut realm = realm_with_array(5);
    realm.eval("let n=0,v={valueOf:()=>{n++;if(n===2)Object.defineProperty(a,'length',{value:1,writable:false});return 2;}}").unwrap();
    assert!(matches!(
        realm.eval("Object.defineProperty(a,'length',{value:v})"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    check(&mut realm, "n===2 && a.length===1");
}

#[test]
fn partial_truncation_rejection_follows_strict_and_descriptor_throw_rules() {
    for expression in [
        "a.length=1",
        "'use strict';a.length=1",
        "Object.defineProperty(a,'length',{value:1,writable:false})",
    ] {
        let mut realm = realm_with_array(5);
        realm
            .eval("Object.defineProperty(a,'3',{value:3,configurable:false});a[4]=4")
            .unwrap();
        let result = realm.eval(expression);
        if expression == "a.length=1" {
            assert_eq!(result, Ok(Value::Number(1.0)));
        } else {
            assert!(matches!(
                result,
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        }
        check(&mut realm, "a.length===4 && a[3]===3 && !('4' in a)");
        if expression.starts_with("Object") {
            check(
                &mut realm,
                "!Object.getOwnPropertyDescriptor(a,'length').writable",
            );
        }
    }
}

#[test]
fn truncation_work_abort_precedes_mutation_and_skips_language_handlers() {
    let mut realm = realm_with_array(10);
    realm
        .eval("let flag=0;for(let i=0;i<10;i++)a[i]=i")
        .unwrap();
    realm.limits.max_steps = 1000;
    assert!(matches!(
        realm.eval("try{a.length=0;}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    check(&mut realm, "flag===0 && a.length===10 && a[9]===9");
}

#[test]
fn recursive_length_conversion_is_bounded_on_a_two_mebibyte_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let mut realm = realm_with_array(0);
            realm
                .eval("let flag=0,v={valueOf:()=>{a.length=v;return 0;}}")
                .unwrap();
            assert!(matches!(
                realm.eval("try{a.length=v;}catch{flag=1;}finally{flag=2;}"),
                Err(Error::Limit { .. })
            ));
            check(&mut realm, "flag===0 && a.length===0");
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn construction_observes_new_target_prototype_before_length_validation() {
    let mut realm = Realm::default();
    let Value::Object(target) = realm.eval("let n=0,p={},F=(function(){}).bind(null);Object.defineProperty(F,'prototype',{get:()=>{n++;return p;},configurable:true});F").unwrap() else { panic!("target"); };
    let span = Span::new(0, 0);
    let Value::Object(array) = realm
        .array_constructor(
            Some(target.clone()),
            vec![Value::Number(2.0)].into_iter(),
            span,
        )
        .unwrap()
    else {
        panic!("array");
    };
    let Value::Object(prototype) = realm.eval("p").unwrap() else {
        panic!("prototype");
    };
    assert_eq!(
        realm.inspect_object(&array).unwrap().prototype(),
        Some(&prototype)
    );
    assert!(realm.inspect_object(&array).unwrap().is_array());
    assert!(matches!(
        realm.array_constructor(
            Some(target.clone()),
            vec![Value::Number(-1.0)].into_iter(),
            span
        ),
        Err(Error::Exception {
            kind: ExceptionKind::RangeError,
            ..
        })
    ));
    check(&mut realm, "n===2");
    realm
        .eval("Object.defineProperty(F,'prototype',{value:null})")
        .unwrap();
    let Value::Object(array) = realm
        .array_constructor(Some(target.clone()), vec![].into_iter(), span)
        .unwrap()
    else {
        panic!("array");
    };
    assert_eq!(
        realm.inspect_object(&array).unwrap().prototype(),
        Some(&realm.intrinsics.as_ref().unwrap().array.prototype)
    );
    realm
        .eval("Object.defineProperty(F,'prototype',{get:()=>{throw 9;}})")
        .unwrap();
    assert_eq!(
        realm.array_constructor(Some(target), vec![Value::Number(-1.0)].into_iter(), span),
        Err(Error::Thrown(Value::Number(9.0)))
    );
}

#[test]
fn join_checks_output_before_next_get_and_bounds_empty_output_work() {
    let mut realm = Realm::default();
    realm.eval("let n=0,flag=0,a=['a','b'],sep='xxx';Object.defineProperty(a,'1',{get:()=>{n++;return 'b';}})").unwrap();
    realm.limits.max_string_units = 3;
    assert!(matches!(
        realm.eval("try{a.join(sep);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    check(&mut realm, "n===0 && flag===0");
    realm.limits.max_string_units = 1_048_576;
    realm.limits.max_steps = 500;
    assert!(matches!(
        realm.eval(
            "try{Array.prototype.join.call({length:Infinity},'');}catch{flag=1;}finally{flag=2;}"
        ),
        Err(Error::Limit { .. })
    ));
    realm.limits.max_steps = 100_000;
    check(&mut realm, "flag===0 && [1,2].join()==='1,2'");
    // A huge logical length does not skip earlier observable operations.
    assert_eq!(
        realm.eval("Array.prototype.join.call({0:{toString:()=>{throw 9;}},length:Infinity})"),
        Err(Error::Thrown(Value::Number(9.0)))
    );
}

#[test]
fn recursive_join_and_to_string_are_bounded_on_a_two_mebibyte_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            for source in ["let a=[];a[0]=a;", "let a=[];a.join=()=>a.toString();"] {
                let mut realm = Realm::default();
                realm.eval(source).unwrap();
                realm.eval("let flag=0").unwrap();
                assert!(matches!(
                    realm.eval("try{a.toString();}catch{flag=1;}finally{flag=2;}"),
                    Err(Error::Limit { .. })
                ));
                check(&mut realm, "flag===0 && [1,2].join()==='1,2'");
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn recursive_at_index_conversion_is_bounded_on_a_two_mebibyte_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let mut realm = Realm::default();
            realm
                .eval("let a=[1],flag=0,i={valueOf:()=>a.at(i)}")
                .unwrap();
            assert!(matches!(
                realm.eval("try{a.at(i);}catch{flag=1;}finally{flag=2;}"),
                Err(Error::Limit { .. })
            ));
            check(&mut realm, "flag===0 && a.at(0)===1");
        })
        .unwrap()
        .join()
        .unwrap();
}
