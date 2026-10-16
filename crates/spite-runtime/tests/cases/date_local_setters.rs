//! Local setters retain captured fields and resolve zone transitions before clipping.

use spite_runtime::{Error, ExceptionKind, Limits, Realm, TimeZone, Value};

include!("fixtures/date_local_getters.rs");

const METHODS: [(&str, usize); 7] = [
    ("setDate", 1),
    ("setFullYear", 3),
    ("setMonth", 2),
    ("setHours", 4),
    ("setMinutes", 3),
    ("setSeconds", 2),
    ("setMilliseconds", 1),
];

fn realm(zone: TimeZone) -> Realm {
    let mut realm = Realm::default();
    realm.set_time_zone(zone);
    realm
}

fn check(zone: &str, source: &str) {
    assert_eq!(
        realm(TimeZone::bundled(zone).unwrap()).eval(source),
        Ok(Value::Boolean(true)),
        "{zone}: {source}"
    );
}

#[test]
fn defaults_truncation_rollover_and_literal_years_follow_the_local_calendar() {
    for (method, args, expected) in [
        ("setDate", "0.9", "2000-01-31T17:34:56.789Z"),
        ("setMonth", "2.9", "2000-03-29T17:34:56.789Z"),
        ("setFullYear", "2001.9", "2001-03-01T17:34:56.789Z"),
        ("setHours", "25.9", "2000-03-01T06:34:56.789Z"),
        ("setMinutes", "-1.9", "2000-02-29T16:59:56.789Z"),
        ("setSeconds", "60.9", "2000-02-29T17:35:00.789Z"),
        ("setMilliseconds", "-1.9", "2000-02-29T17:34:55.999Z"),
    ] {
        check(
            "America/New_York",
            &format!(
                "let d=new Date('2000-02-29T17:34:56.789Z');d.{method}({args})===d.getTime() && d.toISOString()==='{expected}'"
            ),
        );
    }
    check(
        "UTC",
        "let d=new Date('2000-02-29T12:34:56.789Z');d.setFullYear(0.9)===d.getTime() && d.toISOString()==='0000-02-29T12:34:56.789Z' && d.setFullYear(70)===d.getTime() && d.toISOString()==='0070-03-01T12:34:56.789Z'",
    );
    for (method, arity) in METHODS {
        check(
            "America/New_York",
            &format!("let d=new Date(0);Number.isNaN(d.{method}()) && Number.isNaN(d.getTime())"),
        );
        for i in 0..arity {
            let mut args = vec!["1"; i + 1];
            args[i] = "undefined";
            check(
                "America/New_York",
                &format!(
                    "let d=new Date(0);Number.isNaN(d.{method}({})) && Number.isNaN(d.getTime())",
                    args.join(",")
                ),
            );
        }
    }
    check(
        "UTC",
        "let d=new Date(0);1/d.setMilliseconds(-0.9)===Infinity && 1/d.getTime()===Infinity",
    );
}

#[test]
fn setters_choose_earlier_folds_and_preserve_gap_disambiguation_at_all_scales() {
    for (zone, original, method, args, expected) in [
        (
            "America/New_York",
            "2024-03-10T06:30:12.987Z",
            "setHours",
            "2",
            "2024-03-10T07:30:12.987Z",
        ),
        (
            "America/New_York",
            "2024-03-09T07:30:12.987Z",
            "setDate",
            "10",
            "2024-03-10T07:30:12.987Z",
        ),
        (
            "America/New_York",
            "2024-02-10T07:30:12.987Z",
            "setMonth",
            "2",
            "2024-03-10T07:30:12.987Z",
        ),
        (
            "America/New_York",
            "2023-03-10T07:30:12.987Z",
            "setFullYear",
            "2024",
            "2024-03-10T07:30:12.987Z",
        ),
        (
            "America/New_York",
            "2024-11-03T06:30:12.987Z",
            "setHours",
            "1",
            "2024-11-03T05:30:12.987Z",
        ),
        (
            "America/New_York",
            "2024-11-03T06:30:12.987Z",
            "setMinutes",
            "30",
            "2024-11-03T05:30:12.987Z",
        ),
        (
            "America/New_York",
            "2024-11-03T06:30:12.987Z",
            "setSeconds",
            "12",
            "2024-11-03T05:30:12.987Z",
        ),
        (
            "America/New_York",
            "2024-11-03T06:30:12.987Z",
            "setMilliseconds",
            "987",
            "2024-11-03T05:30:12.987Z",
        ),
        (
            "Australia/Lord_Howe",
            "2024-10-05T15:15:12.987Z",
            "setHours",
            "2",
            "2024-10-05T15:45:12.987Z",
        ),
        (
            "Australia/Lord_Howe",
            "2024-04-06T15:15:12.987Z",
            "setMinutes",
            "45",
            "2024-04-06T14:45:12.987Z",
        ),
        (
            "Pacific/Apia",
            "2011-12-29T22:00:00Z",
            "setDate",
            "30",
            "2011-12-30T22:00:00Z",
        ),
    ] {
        check(
            zone,
            &format!(
                "let d=new Date('{original}');d.{method}({args})===d.getTime() && d.getTime()===Date.parse('{expected}')"
            ),
        );
    }
}

#[test]
fn captured_time_survives_hooks_and_full_year_revives_using_the_local_epoch_calendar() {
    for (method, args, expected) in [
        ("setDate", "1", "2000-02-01T17:34:56.789Z"),
        ("setFullYear", "2001", "2001-03-01T17:34:56.789Z"),
        ("setMonth", "2", "2000-03-29T17:34:56.789Z"),
        ("setHours", "2", "2000-02-29T07:34:56.789Z"),
        ("setMinutes", "2", "2000-02-29T17:02:56.789Z"),
        ("setSeconds", "2", "2000-02-29T17:34:02.789Z"),
        ("setMilliseconds", "2", "2000-02-29T17:34:56.002Z"),
    ] {
        check(
            "America/New_York",
            &format!(
                "let d=new Date('2000-02-29T17:34:56.789Z');d.{method}({{valueOf(){{d.setTime(NaN);return {args};}}}})===d.getTime() && d.toISOString()==='{expected}'"
            ),
        );
    }
    check(
        "America/New_York",
        "let d=new Date(NaN);d.setFullYear({valueOf(){d.setTime(9);return 2000;}})===d.getTime() && d.toISOString()==='2000-01-01T05:00:00.000Z'",
    );
    check(
        "America/New_York",
        "let d=new Date(NaN);d.setFullYear(0)===d.getTime() && d.toISOString()==='0000-01-01T04:56:02.000Z'",
    );
    check(
        "America/New_York",
        "let d=new Date(NaN);d.setFullYear(2000,1,29)===d.getTime() && d.toISOString()==='2000-02-29T05:00:00.000Z'",
    );
    check(
        "America/New_York",
        "let d=new Date(NaN);Number.isNaN(d.setFullYear({valueOf(){d.setTime(9);return NaN;}})) && Number.isNaN(d.getTime())",
    );
}

#[test]
fn every_present_field_converts_in_order_after_nan_and_excess_values_are_ignored() {
    for (method, arity) in METHODS {
        let args=(0..arity).map(|i| format!("{{[Symbol.toPrimitive](h){{if(h!=='number')throw 1;trace+='{i}';return {};}}}}",if i==0 {"NaN"} else {"0"})).collect::<Vec<_>>().join(",");
        let trace = (0..arity).map(|i| i.to_string()).collect::<String>();
        for time in ["0", "NaN"] {
            check(
                "America/New_York",
                &format!(
                    "let d=new Date({time}),trace='';Number.isNaN(d.{method}({args})) && trace==='{trace}'"
                ),
            );
        }
        let args = vec!["1"; arity].join(",");
        check(
            "UTC",
            &format!(
                "let count=0,d=new Date(0);d.{method}({args},count++,{{valueOf(){{throw 7;}}}})===d.getTime() && count===1"
            ),
        );
        for failure in 0..arity {
            let args = (0..arity)
                .map(|i| {
                    format!(
                        "{{valueOf(){{trace+='{i}';d.setTime(77);{}}}}}",
                        if i == failure {
                            "throw 7;"
                        } else {
                            "return NaN;"
                        }
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            let trace = (0..=failure).map(|i| i.to_string()).collect::<String>();
            for time in ["0", "NaN"] {
                check(
                    "America/New_York",
                    &format!(
                        "let d=new Date({time}),trace='',caught=false;try{{d.{method}({args});}}catch(e){{caught=e===7;}}caught && trace==='{trace}' && d.getTime()===77"
                    ),
                );
            }
        }
        for failure in 0..arity {
            for value in ["1n", "Symbol()"] {
                let mut args = vec!["NaN"; arity];
                args[failure] = value;
                assert!(matches!(
                    realm(TimeZone::utc())
                        .eval(&format!("new Date(0).{method}({})", args.join(","))),
                    Err(Error::Exception {
                        kind: ExceptionKind::TypeError,
                        ..
                    })
                ));
            }
        }
    }
}

#[test]
fn all_native_offset_boundary_calendars_round_trip_without_clipping_local_fields() {
    for (offset, time, fields) in LOCAL_GETTER_CASES {
        let mut r = realm(TimeZone::fixed(offset));
        for (method, args) in [
            ("setDate", fields[2].to_string()),
            (
                "setFullYear",
                format!("{},{},{}", fields[0], fields[1], fields[2]),
            ),
            ("setMonth", format!("{},{}", fields[1], fields[2])),
            (
                "setHours",
                format!("{},{},{},{}", fields[4], fields[5], fields[6], fields[7]),
            ),
            (
                "setMinutes",
                format!("{},{},{}", fields[5], fields[6], fields[7]),
            ),
            ("setSeconds", format!("{},{}", fields[6], fields[7])),
            ("setMilliseconds", fields[7].to_string()),
        ] {
            assert_eq!(
                r.eval(&format!(
                    "{{let d=new Date({time});d.{method}({args})==={time} && d.getTime()==={time}}}"
                )),
                Ok(Value::Boolean(true)),
                "{offset} {time} {method}({args})"
            );
        }
    }
    for (time, delta) in [(8640000000000000_i64, 1), (-8640000000000000, -1)] {
        check(
            "UTC",
            &format!(
                "let d=new Date({time});Number.isNaN(d.setMilliseconds({delta})) && Number.isNaN(d.getTime())"
            ),
        );
    }
}

#[test]
fn enormous_calendar_components_can_cancel_without_javascript_bigint_value_limits() {
    let mut r = Realm::new(Limits {
        max_bigint_bits: Some(1),
        ..Limits::default()
    });
    r.set_time_zone(TimeZone::utc());
    assert_eq!(r.eval("let d=new Date(0);d.setFullYear(-Number.MAX_VALUE/12,Number.MAX_VALUE,1)===Date.parse('0000-09-01') && d.setFullYear(Number.MAX_VALUE/12,-Number.MAX_VALUE,1)===Date.parse('0000-05-01') && d.setFullYear(5000000000,0,1-1826211780472)===0 && d.setFullYear(1e200,0,-3.652425e202)===-86400000"),Ok(Value::Boolean(true)));
}

#[test]
fn brand_checks_precede_coercion_and_frozen_slots_still_mutate() {
    for (method, _) in METHODS {
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
                    realm(TimeZone::utc()).eval(&format!(
                        "Date.prototype.{method}.call({receiver},{{valueOf(){{throw 7;}}}})"
                    )),
                    Err(Error::Exception {
                        kind: ExceptionKind::TypeError,
                        ..
                    })
                ),
                "{method}: {receiver}"
            );
        }
        check(
            "UTC",
            &format!(
                "let d=new Date(0);Object.setPrototypeOf(d,null);Object.freeze(d);Date.prototype.{method}.call(d,1)===Date.prototype.getTime.call(d) && Object.isFrozen(d)"
            ),
        );
    }
}

#[test]
fn recursive_coercion_preserves_stack_guards_slot_and_realm_recovery() {
    for (method, _) in METHODS {
        let mut r = realm(TimeZone::utc());
        r.eval(&format!(
            "var flag=0;var d=new Date(7),arg={{valueOf(){{return d.{method}(arg);}}}};"
        ))
        .unwrap();
        assert!(
            matches!(
                r.eval(&format!(
                    "try{{d.{method}(arg);}}catch{{flag=1;}}finally{{flag=2;}}"
                )),
                Err(Error::Limit { .. })
            ),
            "{method}"
        );
        assert_eq!(
            r.eval("flag===0 && d.getTime()===7"),
            Ok(Value::Boolean(true))
        );
        assert_eq!(r.eval("new Date(7).getTime()"), Ok(Value::Number(7.0)));
    }
}
