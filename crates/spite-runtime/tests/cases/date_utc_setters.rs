//! UTC setters retain the original timestamp through observable coercion.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

const METHODS: [(&str, usize); 4] = [
    ("setUTCHours", 4),
    ("setUTCMinutes", 3),
    ("setUTCSeconds", 2),
    ("setUTCMilliseconds", 1),
];

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn utc_day_setter_normalizes_days_and_preserves_the_original_time_of_day() {
    for (original, date, expected) in [
        (
            "2000-02-29T12:34:56.789Z",
            "1.9",
            "2000-02-01T12:34:56.789Z",
        ),
        ("2000-02-29T12:34:56.789Z", "0", "2000-01-31T12:34:56.789Z"),
        ("2000-02-29T12:34:56.789Z", "30", "2000-03-01T12:34:56.789Z"),
        ("1900-02-28T12:34:56.789Z", "29", "1900-03-01T12:34:56.789Z"),
        (
            "2000-01-01T00:00:00.001Z",
            "-1.9",
            "1999-12-30T00:00:00.001Z",
        ),
        ("1969-12-31T23:59:59.999Z", "0", "1969-11-30T23:59:59.999Z"),
        (
            "0000-01-01T00:00:00.000Z",
            "0",
            "-000001-12-31T00:00:00.000Z",
        ),
    ] {
        check(&format!(
            "let d=new Date('{original}'); d.setUTCDate({date})===d.getTime() && d.toISOString()==='{expected}'"
        ));
    }
    for date in ["0", "-0", "0.9", "-0.9", "null", "false"] {
        check(&format!(
            "let d=new Date(0);d.setUTCDate({date})===-86400000 && d.getTime()===-86400000"
        ));
    }
    for date in [
        "",
        "undefined",
        "NaN",
        "Infinity",
        "-Infinity",
        "Number.MAX_VALUE",
        "-Number.MAX_VALUE",
    ] {
        check(&format!(
            "let d=new Date(0);Number.isNaN(d.setUTCDate({date})) && Number.isNaN(d.getTime())"
        ));
    }
}

#[test]
fn utc_day_setter_clips_only_after_month_day_and_time_of_day_are_combined() {
    check(
        "let d=new Date(-8640000000000000); d.setUTCDate(20)===-8640000000000000 && d.setUTCDate(21)===-8639999913600000 && Number.isNaN(d.setUTCDate(19)) && Number.isNaN(d.getTime())",
    );
    check(
        "let d=new Date(8640000000000000); d.setUTCDate(13.9)===8640000000000000 && d.setUTCDate(12)===8639999913600000 && Number.isNaN(d.setUTCDate(14)) && Number.isNaN(d.getTime())",
    );
    check(
        "let d=new Date(-8640000000000000+1);d.setUTCDate(20.9)===-8640000000000000+1 && d.getUTCMilliseconds()===1",
    );
    check(
        "let d=new Date(8640000000000000-1);d.setUTCDate(12)===8640000000000000-1 && Number.isNaN(d.setUTCDate(13))",
    );
}

#[test]
fn utc_day_setter_captures_the_month_and_time_before_converting_the_day() {
    check(
        "let d=new Date('2000-02-29T12:34:56.789Z'),calls=0,hint='';let arg={[Symbol.toPrimitive](h){calls++;hint=h;d.setTime(0);return 31;}};let result=d.setUTCDate(arg);result===d.getTime() && calls===1 && hint==='number' && d.toISOString()==='2000-03-02T12:34:56.789Z'",
    );
    check(
        "let d=new Date(0),calls=0;d.setUTCDate({valueOf(){calls++;d.setTime(NaN);return 2;}})===86400000 && d.getTime()===86400000 && calls===1",
    );
    check(
        "let d=new Date(NaN),calls=0;Number.isNaN(d.setUTCDate({valueOf(){calls++;d.setTime(9);return 1;}})) && calls===1 && d.getTime()===9",
    );
    check(
        "let d=new Date(7),caught=false;try{d.setUTCDate({valueOf(){d.setTime(77);throw 9;}});}catch(e){caught=e===9;}caught && d.getTime()===77",
    );
}

#[test]
fn utc_day_setter_checks_brand_before_coercion_and_can_mutate_frozen_slots() {
    for receiver in [
        "undefined",
        "null",
        "0",
        "'1970'",
        "1n",
        "Symbol()",
        "{}",
        "Date.prototype",
        "Object.create(new Date(0))",
    ] {
        assert!(
            matches!(
                Realm::default().eval(&format!(
                    "Date.prototype.setUTCDate.call({receiver},{{valueOf(){{throw 7;}}}})"
                )),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{receiver}"
        );
    }
    for time in ["0", "NaN"] {
        for value in ["1n", "Symbol()"] {
            assert!(matches!(
                Realm::default().eval(&format!("new Date({time}).setUTCDate({value})")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        }
    }
    check(
        "let d=new Date(0);Object.setPrototypeOf(d,null);Object.freeze(d);Date.prototype.setUTCDate.call(d,2)===86400000 && Object.isFrozen(d) && Date.prototype.getTime.call(d)===86400000",
    );
    check(
        "let d=new Date(0),count=0;d.setUTCDate(1,count++,{valueOf(){throw 7;}})===0 && count===1 && d.getTime()===0",
    );
}

#[test]
fn recursive_day_coercion_uses_existing_stack_guards_and_restores_call_state() {
    let mut realm = Realm::default();
    realm
        .eval("var flag=0;let d=new Date(7),arg={valueOf(){return d.setUTCDate(arg);}}")
        .unwrap();
    assert!(matches!(
        realm.eval("try{d.setUTCDate(arg);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(realm.eval("d.getTime()"), Ok(Value::Number(7.0)));
    assert_eq!(realm.eval("d.setUTCDate(1)"), Ok(Value::Number(7.0)));
}

#[test]
fn omitted_fields_retain_the_original_utc_components_and_explicit_undefined_is_nan() {
    for (method, expected) in [
        ("setUTCHours", "2000-02-29T01:34:56.789Z"),
        ("setUTCMinutes", "2000-02-29T12:01:56.789Z"),
        ("setUTCSeconds", "2000-02-29T12:34:01.789Z"),
        ("setUTCMilliseconds", "2000-02-29T12:34:56.001Z"),
    ] {
        check(&format!(
            "let d=new Date('2000-02-29T12:34:56.789Z');let result=d.{method}(1);result===d.getTime() && d.toISOString()==='{expected}'"
        ));
        check(&format!(
            "let d=new Date(0);Number.isNaN(d.{method}()) && Number.isNaN(d.getTime())"
        ));
    }
    for (method, arity) in METHODS {
        for index in 0..arity {
            let mut args = vec!["1"; index + 1];
            args[index] = "undefined";
            check(&format!(
                "let d=new Date(0); Number.isNaN(d.{method}({})) && Number.isNaN(d.getTime())",
                args.join(",")
            ));
        }
    }
}

#[test]
fn negative_and_oversized_components_roll_across_days_and_leap_boundaries() {
    for (method, value, expected) in [
        ("setUTCHours", -1, "1999-12-31T23:00:00.005Z"),
        ("setUTCMinutes", -1, "1999-12-31T23:59:00.005Z"),
        ("setUTCSeconds", -1, "1999-12-31T23:59:59.005Z"),
        ("setUTCMilliseconds", -1, "1999-12-31T23:59:59.999Z"),
        ("setUTCHours", 24, "2000-01-02T00:00:00.005Z"),
        ("setUTCMinutes", 60, "2000-01-01T01:00:00.005Z"),
        ("setUTCSeconds", 60, "2000-01-01T00:01:00.005Z"),
        ("setUTCMilliseconds", 1000, "2000-01-01T00:00:01.000Z"),
    ] {
        check(&format!(
            "let d=new Date('2000-01-01T00:00:00.005Z');d.{method}({value})===d.getTime() && d.toISOString()==='{expected}'"
        ));
    }
    check(
        "let d=new Date('2000-02-29T12:34:56.789Z');d.setUTCHours(24,0,0,0)===951868800000 && d.toISOString()==='2000-03-01T00:00:00.000Z'",
    );
    check(
        "let d=new Date(-1);d.setUTCHours(0,0,0,0)===-86400000 && d.toISOString()==='1969-12-31T00:00:00.000Z'",
    );
}

#[test]
fn arithmetic_truncates_fields_canonicalizes_zero_and_preserves_float_operation_order() {
    check(
        "let d=new Date(0);d.setUTCHours(1.9,2.9,3.9,4.9)===3723004 && d.getTime()===3723004 && d.setUTCHours(-1.9,-2.9,-3.9,-4.9)===-3723004",
    );
    check("let d=new Date(0);d.setUTCHours(2**52,-60*2**52,1,1)===1001 && d.getTime()===1001");
    for (method, _) in METHODS {
        check(&format!(
            "let d=new Date(0);1/d.{method}(-0.9)===Infinity && 1/d.getTime()===Infinity"
        ));
        for value in [
            "NaN",
            "Infinity",
            "-Infinity",
            "Number.MAX_VALUE",
            "-Number.MAX_VALUE",
        ] {
            check(&format!(
                "let d=new Date(0);Number.isNaN(d.{method}({value})) && Number.isNaN(d.getTime())"
            ));
        }
    }
}

#[test]
fn timeclip_endpoints_accept_unchanged_components_and_reject_outside_rollover() {
    for (method, value) in [
        ("setUTCHours", 0),
        ("setUTCMinutes", 0),
        ("setUTCSeconds", 0),
        ("setUTCMilliseconds", 0),
    ] {
        for time in ["8640000000000000", "-8640000000000000"] {
            check(&format!(
                "let d=new Date({time});d.{method}({value})==={time} && d.getTime()==={time}"
            ));
        }
        check(&format!(
            "let hi=new Date(8640000000000000),lo=new Date(-8640000000000000);Number.isNaN(hi.{method}(1)) && Number.isNaN(hi.getTime()) && Number.isNaN(lo.{method}(-1)) && Number.isNaN(lo.getTime())"
        ));
    }
}

#[test]
fn branding_precedes_conversion_and_frozen_dates_can_mutate_internal_slots() {
    for (method, _) in METHODS {
        for receiver in [
            "null",
            "undefined",
            "0",
            "'text'",
            "Symbol()",
            "{}",
            "Date.prototype",
            "Object.create(new Date(0))",
        ] {
            let source =
                format!("Date.prototype.{method}.call({receiver},{{valueOf(){{throw 7;}}}})");
            assert!(
                matches!(
                    Realm::default().eval(&source),
                    Err(Error::Exception {
                        kind: ExceptionKind::TypeError,
                        ..
                    })
                ),
                "{source}"
            );
        }
        check(&format!(
            "let d=new Date(0);Object.freeze(d);d.{method}(1)===d.getTime() && d.getTime()>0 && Object.isFrozen(d)"
        ));
    }
}

#[test]
fn conversion_hooks_cannot_replace_the_captured_day_or_omitted_fields() {
    for (method, expected) in [
        ("setUTCHours", "1969-12-31T00:59:59.999Z"),
        ("setUTCMinutes", "1969-12-31T23:00:59.999Z"),
        ("setUTCSeconds", "1969-12-31T23:59:00.999Z"),
        ("setUTCMilliseconds", "1969-12-31T23:59:59.000Z"),
    ] {
        check(&format!(
            "let d=new Date(-1),calls=0,arg={{valueOf(){{calls++;if(this!==arg || arguments.length!==0)throw 1;d.setTime(0);return 0;}}}};let result=d.{method}(arg);calls===1 && result===d.getTime() && d.toISOString()==='{expected}'"
        ));
    }
}

#[test]
fn previously_invalid_dates_convert_arguments_then_return_nan_without_writing() {
    for (method, arity) in METHODS {
        check(&format!(
            "let d=new Date(NaN),calls=0;Number.isNaN(d.{method}({{valueOf(){{calls++;d.setTime(9);return 0;}}}})) && calls===1 && d.getTime()===9"
        ));
        let args = (0..arity)
            .map(|i| format!("{{valueOf(){{trace+='{i}';d.setTime({i});return 0;}}}}"))
            .collect::<Vec<_>>()
            .join(",");
        let expected = (0..arity).map(|i| i.to_string()).collect::<String>();
        check(&format!(
            "let trace='',d=new Date(NaN);Number.isNaN(d.{method}({args})) && trace==='{expected}' && d.getTime()==={} ",
            arity - 1
        ));
        assert!(matches!(
            Realm::default().eval(&format!("new Date(NaN).{method}(1n)")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        assert!(matches!(
            Realm::default().eval(&format!("new Date(NaN).{method}(Symbol())")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
}

#[test]
fn ordered_conversions_continue_after_nan_and_abrupt_completion_preserves_hook_mutations() {
    for (method, arity) in METHODS {
        for failure in 0..arity {
            let args = (0..arity)
                .map(|i| {
                    format!(
                        "{{valueOf(){{trace+='{i}';d.setTime(9);{} }}}}",
                        if i == failure {
                            "throw 7;"
                        } else {
                            "return NaN;"
                        }
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            let expected = (0..=failure).map(|i| i.to_string()).collect::<String>();
            check(&format!(
                "let trace='',d=new Date(0),caught=false;try{{d.{method}({args});}}catch(e){{caught=e===7;}}caught && trace==='{expected}' && d.getTime()===9"
            ));
        }
    }
}

#[test]
fn extra_arguments_are_evaluated_by_the_caller_and_never_coerced_by_setters() {
    for (method, arity) in METHODS {
        let args = vec!["0"; arity].join(",");
        check(&format!(
            "let d=new Date(0),count=0;d.{method}({args},count++,{{valueOf(){{throw 7;}}}})===0 && d.getTime()===0 && count===1"
        ));
    }
}

#[test]
fn recursive_conversion_uses_existing_stack_guards_and_restores_native_call_state() {
    let mut realm = Realm::default();
    realm
        .eval("var marker=0;let d=new Date(7),arg={valueOf(){return d.setUTCHours(arg);}}")
        .unwrap();
    assert!(matches!(
        realm.eval("try{d.setUTCHours(arg);}catch{marker=1;}finally{marker=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    assert_eq!(realm.eval("d.getTime()"), Ok(Value::Number(7.0)));
    assert_eq!(realm.eval("d.setUTCHours(0)"), Ok(Value::Number(7.0)));
}
