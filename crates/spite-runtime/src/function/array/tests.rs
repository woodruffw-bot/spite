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

#[test]
fn recursive_push_setters_and_pop_getters_obey_the_native_stack_limit() {
    std::thread::Builder::new().stack_size(2*1024*1024).spawn(|| {
        for (setup,expression) in [
            ("let o={length:0,push:Array.prototype.push};Object.defineProperty(o,'0',{set:()=>o.push(1)})","o.push(1)"),
            ("let o={pop:Array.prototype.pop};Object.defineProperty(o,'length',{get:()=>o.pop()})","o.pop()"),
        ] {
            let mut realm=Realm::default();
            realm.eval(setup).unwrap();realm.eval("let flag=0").unwrap();
            assert!(matches!(realm.eval(&format!("try{{{expression};}}catch{{flag=1;}}finally{{flag=2;}}")),Err(Error::Limit{..})));
            check(&mut realm,"flag===0 && [1].pop()===1");
        }
    }).unwrap().join().unwrap();
}

#[test]
fn push_work_abort_retains_completed_elements_and_consistent_array_length() {
    let mut realm = Realm::default();
    let receiver = realm.eval("let a=[];a").unwrap();
    realm.remaining_steps = 300;
    let values = (0..20)
        .map(|n| Value::Number(f64::from(n)))
        .collect::<Vec<_>>();
    assert!(matches!(
        realm.array_push(receiver, values.into_iter(), Span::new(0, 0)),
        Err(Error::Limit { .. })
    ));
    check(
        &mut realm,
        "a.length>0 && a.length<20 && a[0]===0 && a[a.length-1]===a.length-1 && !Object.hasOwn(a,a.length)",
    );
}

#[test]
fn callback_iteration_work_and_this_arg_copying_are_bounded() {
    let mut realm = Realm::default();
    realm.eval("let flag=0").unwrap();
    realm.limits.max_steps = 500;
    for method in ["forEach", "every", "some"] {
        assert!(matches!(realm.eval(&format!("try{{Array.prototype.{method}.call({{length:Infinity}},()=>true);}}catch{{flag=1;}}finally{{flag=2;}}")),Err(Error::Limit{..})));
    }
    realm.limits.max_steps = 100_000;
    check(&mut realm, "flag===0");
    let receiver = realm.eval("[1]").unwrap();
    let callback = realm.eval("()=>0").unwrap();
    let this_arg = Value::String(JsString::from_code_units(vec![0x61; 1000]));
    realm.remaining_steps = 500;
    assert!(matches!(
        realm.array_callback(
            Builtin::ArrayForEach,
            receiver,
            callback,
            this_arg,
            Span::new(0, 0)
        ),
        Err(Error::Limit { .. })
    ));
}

#[test]
fn recursive_array_callbacks_stop_on_a_two_mebibyte_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            for method in ["forEach", "every", "some"] {
                let mut realm = Realm::default();
                realm
                    .eval(&format!("let a=[1],flag=0,f=()=>a.{method}(f)"))
                    .unwrap();
                assert!(matches!(
                    realm.eval(&format!(
                        "try{{a.{method}(f);}}catch{{flag=1;}}finally{{flag=2;}}"
                    )),
                    Err(Error::Limit { .. })
                ));
                check(&mut realm, "flag===0 && [1].every(v=>v===1)");
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn recursive_find_predicates_stop_on_a_two_mebibyte_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            for method in ["find", "findIndex", "findLast", "findLastIndex"] {
                let mut realm = Realm::default();
                realm
                    .eval(&format!("let a=[1],flag=0,f=()=>a.{method}(f)"))
                    .unwrap();
                assert!(matches!(
                    realm.eval(&format!(
                        "try{{a.{method}(f);}}catch{{flag=1;}}finally{{flag=2;}}"
                    )),
                    Err(Error::Limit { .. })
                ));
                check(&mut realm, "flag===0 && [1].find(v=>v===1)===1");
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn find_bounds_hole_traversal_and_copying_of_retained_values() {
    let mut realm = Realm::default();
    realm.eval("let flag=0").unwrap();
    realm.limits.max_steps = 500;
    for method in ["find", "findIndex", "findLast", "findLastIndex"] {
        assert!(matches!(realm.eval(&format!("try{{Array.prototype.{method}.call({{length:Infinity}},()=>false);}}catch{{flag=1;}}finally{{flag=2;}}")),Err(Error::Limit{..})));
    }
    realm.limits.max_steps = 100_000;
    check(&mut realm, "flag===0");
    let receiver = realm.eval("['a'.repeat(1000)]").unwrap();
    let predicate = realm.eval("()=>true").unwrap();
    realm.remaining_steps = 1500;
    assert!(matches!(
        realm.array_find(
            Builtin::ArrayFind,
            receiver,
            predicate,
            Value::Undefined,
            Span::new(0, 0)
        ),
        Err(Error::Limit { .. })
    ));
}

#[test]
fn recursive_array_search_getters_stop_on_a_two_mebibyte_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            for method in ["includes", "indexOf", "lastIndexOf"] {
                let mut realm = Realm::default();
                realm
                    .eval(&format!(
                        "let a=[1],flag=0;Object.defineProperty(a,'0',{{get:()=>a.{method}(1)}})"
                    ))
                    .unwrap();
                assert!(matches!(
                    realm.eval(&format!(
                        "try{{a.{method}(1);}}catch{{flag=1;}}finally{{flag=2;}}"
                    )),
                    Err(Error::Limit { .. })
                ));
                check(&mut realm, "flag===0 && [1].includes(1)");
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn array_search_bounds_hole_traversal_and_each_element_comparison() {
    let mut realm = Realm::default();
    realm.eval("let flag=0").unwrap();
    realm.limits.max_steps = 500;
    for method in ["includes", "indexOf", "lastIndexOf"] {
        assert!(matches!(realm.eval(&format!("try{{Array.prototype.{method}.call({{length:Infinity}},7);}}catch{{flag=1;}}finally{{flag=2;}}")),Err(Error::Limit{..})));
    }
    realm.limits.max_steps = 100_000;
    check(
        &mut realm,
        "flag===0 && Array.prototype.includes.call({length:Infinity},undefined)",
    );
    let receiver = realm.eval("['a'.repeat(1000)]").unwrap();
    let search = realm.eval("'a'.repeat(1000)").unwrap();
    realm.remaining_steps = 2500;
    assert!(matches!(
        realm.array_search(
            Builtin::ArrayIncludes,
            receiver,
            search,
            None,
            Span::new(0, 0)
        ),
        Err(Error::Limit { .. })
    ));
}

#[test]
fn recursive_reducers_stop_on_a_two_mebibyte_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            for method in ["reduce", "reduceRight"] {
                let mut realm = Realm::default();
                realm
                    .eval(&format!("let a=[1],flag=0,f=()=>a.{method}(f,0)"))
                    .unwrap();
                assert!(matches!(
                    realm.eval(&format!(
                        "try{{a.{method}(f,0);}}catch{{flag=1;}}finally{{flag=2;}}"
                    )),
                    Err(Error::Limit { .. })
                ));
                check(&mut realm, "flag===0 && [1].reduce((p,v)=>p+v,0)===1");
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn reduction_bounds_seed_search_and_callback_traversal() {
    let mut realm = Realm::default();
    realm.eval("let flag=0").unwrap();
    realm.limits.max_steps = 500;
    for method in ["reduce", "reduceRight"] {
        for initial in ["", ",0"] {
            assert!(matches!(realm.eval(&format!("try{{Array.prototype.{method}.call({{length:Infinity}},()=>0{initial});}}catch{{flag=1;}}finally{{flag=2;}}")),Err(Error::Limit{..})));
        }
        assert!(matches!(realm.eval(&format!("try{{[1,2,3].{method}(()=>{{while(true){{}}}},0);}}catch{{flag=1;}}finally{{flag=2;}}")),Err(Error::Limit{..})));
    }
    realm.limits.max_steps = 100_000;
    check(&mut realm, "flag===0");
}

#[test]
fn recursive_reverse_getters_and_setters_stop_on_a_two_mebibyte_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            for descriptor in ["{get:()=>a.reverse()}", "{get:()=>1,set:()=>a.reverse()}"] {
                let mut realm = Realm::default();
                realm
                    .eval(&format!(
                        "let a=[1,2],flag=0;Object.defineProperty(a,'0',{descriptor})"
                    ))
                    .unwrap();
                assert!(matches!(
                    realm.eval("try{a.reverse();}catch{flag=1;}finally{flag=2;}"),
                    Err(Error::Limit { .. })
                ));
                check(&mut realm, "flag===0 && [1,2].reverse()[0]===2");
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn reverse_bounds_sparse_traversal_and_keeps_completed_mutations_on_host_abort() {
    let mut realm = Realm::default();
    realm.eval("let o={0:7,length:Infinity},flag=0").unwrap();
    // Allow the first pair to complete before the remaining huge range aborts.
    realm.limits.max_steps = 10_000;
    assert!(matches!(
        realm.eval("try{Array.prototype.reverse.call(o);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    realm.limits.max_steps = 100_000;
    check(
        &mut realm,
        "flag===0 && !(0 in o) && o[9007199254740990]===7 && o.length===Infinity",
    );
    let receiver = realm
        .eval("let a=['a'.repeat(1000),'b'.repeat(1000)];a")
        .unwrap();
    realm.remaining_steps = 1500;
    assert!(matches!(
        realm.array_reverse(receiver, Span::new(0, 0)),
        Err(Error::Limit { .. })
    ));
    check(&mut realm, "a[0].charAt(0)==='a' && a[1].charAt(0)==='b'");
}

#[test]
fn recursive_range_conversions_and_writes_stop_on_a_two_mebibyte_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            for (setup, call) in [
                ("let a=[1],i={valueOf:()=>a.fill(0,i)}", "a.fill(0,i)"),
                (
                    "let a=[1],i={valueOf:()=>a.copyWithin(i,0)}",
                    "a.copyWithin(i,0)",
                ),
                (
                    "let a=[1];Object.defineProperty(a,'0',{set:()=>a.fill(0)})",
                    "a.fill(0)",
                ),
                (
                    "let a=[1];Object.defineProperty(a,'0',{get:()=>a.copyWithin(0,0)})",
                    "a.copyWithin(0,0)",
                ),
            ] {
                let mut realm = Realm::default();
                realm.eval(setup).unwrap();
                realm.eval("let flag=0").unwrap();
                assert!(matches!(
                    realm.eval(&format!("try{{{call};}}catch{{flag=1;}}finally{{flag=2;}}")),
                    Err(Error::Limit { .. })
                ));
                check(&mut realm, "flag===0 && [1].fill(7)[0]===7");
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn range_mutations_bound_traversal_and_charge_fill_value_copies() {
    let mut realm = Realm::default();
    realm.eval("let o={length:Infinity},flag=0").unwrap();
    realm.limits.max_steps = 10_000;
    assert!(matches!(
        realm.eval("try{Array.prototype.fill.call(o,7);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    realm.limits.max_steps = 100_000;
    check(&mut realm, "flag===0 && o[0]===7 && o.length===Infinity");
    realm.limits.max_steps = 500;
    assert!(matches!(realm.eval("try{Array.prototype.copyWithin.call({length:Infinity},0,0);}catch{flag=1;}finally{flag=2;}"),Err(Error::Limit{..})));
    realm.limits.max_steps = 100_000;
    check(&mut realm, "flag===0");
    let receiver = realm.eval("let a=[0,0];a").unwrap();
    let value = realm.eval("'a'.repeat(1000)").unwrap();
    realm.remaining_steps = 500;
    assert!(matches!(
        realm.array_fill(
            receiver,
            value,
            Value::Undefined,
            Value::Undefined,
            Span::new(0, 0)
        ),
        Err(Error::Limit { .. })
    ));
    check(&mut realm, "a[0]===0 && a[1]===0");
}

#[test]
fn recursive_front_mutations_stop_on_a_two_mebibyte_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            for (setup,call) in [
                ("let a=[1];Object.defineProperty(a,'0',{get:()=>a.shift()})", "a.shift()"),
                ("let a={length:0};Object.defineProperty(a,'length',{get:()=>0,set:()=>Array.prototype.unshift.call(a)})", "Array.prototype.unshift.call(a)"),
                ("let a=[1];Object.defineProperty(a,'0',{get:()=>a.unshift(7)})", "a.unshift(7)"),
            ] {
                let mut realm=Realm::default();realm.eval(setup).unwrap();realm.eval("let flag=0").unwrap();
                assert!(matches!(realm.eval(&format!("try{{{call};}}catch{{flag=1;}}finally{{flag=2;}}")),Err(Error::Limit{..})));
                check(&mut realm,"flag===0 && [1].shift()===1");
            }
        }).unwrap().join().unwrap();
}

#[test]
fn front_movement_bounds_huge_ranges_and_retains_completed_effects_on_host_abort() {
    for (setup, call, expected) in [
        (
            "let o={0:1,1:7,length:Infinity}",
            "Array.prototype.shift.call(o)",
            "o[0]===7 && o.length===Infinity",
        ),
        (
            "let o={9007199254740989:7,length:9007199254740990}",
            "Array.prototype.unshift.call(o,1)",
            "o[9007199254740990]===7 && o.length===9007199254740990 && !Object.hasOwn(o,'0')",
        ),
    ] {
        let mut realm = Realm::default();
        realm.eval(setup).unwrap();
        realm.eval("let flag=0").unwrap();
        realm.limits.max_steps = 10_000;
        assert!(matches!(
            realm.eval(&format!("try{{{call};}}catch{{flag=1;}}finally{{flag=2;}}")),
            Err(Error::Limit { .. })
        ));
        realm.limits.max_steps = 100_000;
        check(&mut realm, &format!("flag===0 && ({expected})"));
    }
}

#[test]
fn recursive_copy_getters_and_index_conversions_stop_on_a_two_mebibyte_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            for (setup, call) in [
                (
                    "let a=[1];Object.defineProperty(a,'0',{get:()=>a.toReversed()})",
                    "a.toReversed()",
                ),
                (
                    "let a=[1,2];Object.defineProperty(a,'0',{get:()=>a.with(1,7)})",
                    "a.with(1,7)",
                ),
                ("let a=[1],i={valueOf:()=>a.with(i,7)}", "a.with(i,7)"),
                (
                    "let a=[1];Object.defineProperty(a,'0',{get:()=>a.toSpliced()})",
                    "a.toSpliced()",
                ),
                ("let a=[1],i={valueOf:()=>a.toSpliced(i)}", "a.toSpliced(i)"),
                (
                    "let a=[1],i={valueOf:()=>a.toSpliced(0,i)}",
                    "a.toSpliced(0,i)",
                ),
            ] {
                let mut realm = Realm::default();
                realm.eval(setup).unwrap();
                realm.eval("let flag=0").unwrap();
                assert!(matches!(
                    realm.eval(&format!("try{{{call};}}catch{{flag=1;}}finally{{flag=2;}}")),
                    Err(Error::Limit { .. })
                ));
                check(&mut realm, "flag===0 && [1].with(0,7)[0]===7");
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn array_copies_bound_dense_output_and_preserve_the_receiver_on_host_abort() {
    for call in ["toReversed()", "with(0,7)", "toSpliced(0,0)"] {
        let mut realm = Realm::default();
        realm.eval("let a=Array(4294967295),flag=0").unwrap();
        realm.limits.max_steps = 500;
        assert!(matches!(
            realm.eval(&format!(
                "try{{a.{call};}}catch{{flag=1;}}finally{{flag=2;}}"
            )),
            Err(Error::Limit { .. })
        ));
        realm.limits.max_steps = 100_000;
        check(
            &mut realm,
            "flag===0 && a.length===4294967295 && !Object.hasOwn(a,'0')",
        );
        realm.collect(10_000).unwrap();
    }
    let mut realm = Realm::default();
    let receiver = realm.eval("['x'.repeat(1000)]").unwrap();
    realm.remaining_steps = 500;
    assert!(matches!(
        realm.array_to_reversed(receiver, Span::new(0, 0)),
        Err(Error::Limit { .. })
    ));
}

#[test]
fn array_of_constructor_and_length_setter_reentry_are_bounded_on_small_stacks() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            for setup in [
                "function C(){return Array.of.call(C);}",
                "function C(){}Object.defineProperty(C.prototype,'length',{set:()=>Array.of.call(C)});",
            ] {
                let mut realm = Realm::default();
                realm.eval(setup).unwrap();
                realm.eval("let flag=0").unwrap();
                assert!(matches!(
                    realm.eval("try{Array.of.call(C);}catch{flag=1;}finally{flag=2;}"),
                    Err(Error::Limit { .. })
                ));
                check(&mut realm, "flag===0 && Array.of(7)[0]===7");
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn array_of_bounds_property_work_and_retains_partial_custom_object_definitions() {
    let mut realm = Realm::default();
    let constructor = realm.eval("let o={};function C(){return o;}C").unwrap();
    realm.remaining_steps = 500;
    assert!(matches!(
        realm.array_of(
            constructor,
            vec![Value::Number(7.0); 1000].into_iter(),
            Span::new(0, 0)
        ),
        Err(Error::Limit { .. })
    ));
    check(
        &mut realm,
        "o[0]===7 && !Object.hasOwn(o,'999') && !Object.hasOwn(o,'length')",
    );
    realm.collect(10_000).unwrap();
}

#[test]
fn recursive_array_locale_calls_and_getters_stop_on_a_two_mebibyte_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            for setup in [
                "let a=[];a[0]=a;",
                "let a=[1];Object.defineProperty(a,'0',{get:()=>a.toLocaleString()});",
                "let o={},a=[o];Object.defineProperty(o,'toLocaleString',{get:()=>a.toLocaleString()});",
                "let a=[{toLocaleString:()=>a.toLocaleString()}];",
                "let a=[{toLocaleString:()=>({toString:()=>a.toLocaleString()})}];",
            ] {
                let mut realm=Realm::default();
                realm.eval(setup).unwrap();
                realm.eval("let flag=0").unwrap();
                assert!(matches!(realm.eval("try{a.toLocaleString();}catch{flag=1;}finally{flag=2;}"),Err(Error::Limit{..})));
                check(&mut realm,"flag===0 && [1,2].toLocaleString()==='1,2'");
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn array_locale_output_limits_precede_later_gets_and_large_lengths_are_bounded() {
    let mut realm = Realm::default();
    realm
        .eval("let flag=0,n=0,a=['abc',2];Object.defineProperty(a,'1',{get:()=>{n++;return 2;}})")
        .unwrap();
    realm.limits.max_string_units = 3;
    assert!(matches!(
        realm.eval("try{a.toLocaleString();}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    check(&mut realm, "n===0 && flag===0");
    realm.limits.max_string_units = 1_048_576;
    realm.limits.max_steps = 500;
    assert!(matches!(realm.eval("try{Array.prototype.toLocaleString.call({length:Infinity});}catch{flag=1;}finally{flag=2;}"),Err(Error::Limit{..})));
    realm.limits.max_steps = 100_000;
    check(&mut realm, "flag===0 && [1].toLocaleString()==='1'");
    assert_eq!(realm.eval("Array.prototype.toLocaleString.call({0:{toLocaleString:()=>{throw 7;}},length:Infinity})"),Err(Error::Thrown(Value::Number(7.0))));
}

#[test]
fn recursive_array_sort_getters_comparators_and_conversions_are_stack_bounded() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            for method in ["sort", "toSorted"] {
                for setup in [
                    format!("let a=[2,1];Object.defineProperty(a,'0',{{get:()=>a.{method}()}});"),
                    format!("let a=[{{toString:()=>a.{method}()}},1];"),
                    format!("let a=[2,1],cmp=()=>a.{method}(cmp);"),
                    format!("let a=[2,1],cmp=()=>({{valueOf:()=>a.{method}(cmp)}});"),
                ] {
                    let mut realm=Realm::default();
                    let comparator=if setup.contains("cmp=") {"cmp"}else{""};
                    realm.eval(&setup).unwrap();
                    realm.eval("let flag=0").unwrap();
                    assert!(matches!(realm.eval(&format!("try{{a.{method}({comparator});}}catch{{flag=1;}}finally{{flag=2;}}")),Err(Error::Limit{..})));
                    check(&mut realm,"flag===0 && [2,1].toSorted().join()==='1,2'");
                }
            }
            let mut realm=Realm::default();
            realm.eval("let a=[2,1];Object.defineProperty(a,'0',{get:()=>2,set:()=>a.sort()});let flag=0").unwrap();
            assert!(matches!(realm.eval("try{a.sort();}catch{flag=1;}finally{flag=2;}"),Err(Error::Limit{..})));
            check(&mut realm,"flag===0");
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn array_sort_collection_and_comparison_work_abort_without_implicit_writes() {
    for (setup, call, expected) in [
        (
            "let a={0:2,1:1,length:Infinity}",
            "Array.prototype.sort.call(a)",
            "a[0]===2 && a[1]===1 && a.length===Infinity",
        ),
        (
            "let a=Array(4294967295)",
            "a.toSorted()",
            "a.length===4294967295 && !Object.hasOwn(a,'0')",
        ),
        (
            "let a=[2,1]",
            "a.sort(()=>{while(true){}})",
            "a.join()==='2,1'",
        ),
    ] {
        let mut realm = Realm::default();
        realm.eval(setup).unwrap();
        realm.eval("let flag=0").unwrap();
        realm.limits.max_steps = 500;
        assert!(matches!(
            realm.eval(&format!("try{{{call};}}catch{{flag=1;}}finally{{flag=2;}}")),
            Err(Error::Limit { .. })
        ));
        realm.limits.max_steps = 100_000;
        check(&mut realm, &format!("flag===0 && ({expected})"));
        realm.collect(10_000).unwrap();
    }
}
