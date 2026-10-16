//! Local Date strings preserve exact offset round trips and ignore conversion hooks.

use spite_runtime::{Error, ExceptionKind, Limits, Realm, TimeZone, Value};

include!("fixtures/date_local_getters.rs");

const METHODS: [&str; 3] = ["toString", "toDateString", "toTimeString"];

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
fn local_strings_use_captured_calendar_fields_and_exact_historical_offsets() {
    check(
        "UTC",
        "let d=new Date(-1);d.toDateString()==='Wed Dec 31 1969' && d.toTimeString()==='23:59:59 GMT+0000' && d.toString()==='Wed Dec 31 1969 23:59:59 GMT+0000'",
    );
    check(
        "America/New_York",
        "let d=new Date('0000-01-01T00:00Z');d.toString()==='Fri Dec 31 -0001 19:03:58 GMT-0456 (UTC-04:56:02)' && Date.parse(d.toString())===d.getTime()",
    );
    check(
        "Africa/Abidjan",
        "let d=new Date(-1830383032001);d.toTimeString()==='23:59:59 GMT-0016 (UTC-00:16:08)' && Date.parse(d.toString())===Math.floor(d.getTime()/1000)*1000",
    );
    check(
        "America/New_York",
        "let d=new Date('2024-11-03T06:30Z');d.toString()==='Sun Nov 03 2024 01:30:00 GMT-0500' && Date.parse(d.toString())===d.getTime() && Date.parse(new Date('2024-11-03T05:30Z').toString())===Date.parse('2024-11-03T05:30Z')",
    );
    check(
        "Australia/Lord_Howe",
        "let d=new Date('2024-04-06T15:15Z');d.toTimeString()==='01:45:00 GMT+1030' && Date.parse(d.toString())===d.getTime()",
    );
}

#[test]
fn all_native_offset_boundary_strings_round_trip_without_reclipping_local_fields() {
    for (offset, time, _) in LOCAL_GETTER_CASES {
        let mut r = realm(TimeZone::fixed(offset));
        let expected = time.div_euclid(1000) * 1000;
        assert_eq!(r.eval(&format!("let d=new Date({time}),s=d.toString();s===d.toDateString()+' '+d.toTimeString() && Date.parse(s)==={expected} && new Date(s).getTime()==={expected}")),Ok(Value::Boolean(true)),"{offset} {time}");
        let text = r.eval("s").unwrap();
        // Parsing a saved offset string remains independent of the realm's
        // subsequently selected zone, including both endpoints and sub-seconds.
        r.set_time_zone(TimeZone::fixed(if offset == 0 { 3600 } else { 0 }));
        assert_eq!(
            r.eval("Date.parse(s)"),
            Ok(Value::Number(expected as f64)),
            "{offset}: {text:?}"
        );
    }
}

#[test]
fn branded_string_methods_ignore_arguments_and_receiver_conversion_properties() {
    for method in METHODS {
        check(
            "UTC",
            &format!(
                "let d=new Date(0),n=0;Object.defineProperty(d,Symbol.toPrimitive,{{get(){{throw 7;}}}});Object.defineProperty(d,'valueOf',{{get(){{throw 8;}}}});Object.defineProperty(d,'toISOString',{{get(){{throw 9;}}}});typeof Date.prototype.{method}.call(d,n++,{{[Symbol.toPrimitive](){{throw 10;}}}})==='string' && n===1 && Date.prototype.getTime.call(d)===0"
            ),
        );
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
                "{method} {receiver}"
            );
        }
        check(
            "UTC",
            &format!(
                "let d=new Date(0);Object.setPrototypeOf(d,null);Object.freeze(d);typeof Date.prototype.{method}.call(d)==='string' && Object.isFrozen(d)"
            ),
        );
        check("UTC", &format!("new Date(NaN).{method}()==='Invalid Date'"));
    }
}

#[test]
fn date_function_ignores_all_supplied_values_and_has_current_whole_second_output() {
    check(
        "UTC",
        "let touched=0,arg={[Symbol.toPrimitive](){touched++;throw 7;}},before=Date.now(),text=Date.call(arg,arg,arg,arg),after=Date.now(),parsed=Date.parse(text);typeof text==='string' && touched===0 && parsed>=Math.floor(before/1000)*1000 && parsed<=after && typeof Date.bind(arg,arg)()==='string'",
    );
    check(
        "America/New_York",
        "let d=new Date(0),text=d.toString();String(d)===text && d+1===text+'1' && new Date({[Symbol.toPrimitive](hint){if(hint!=='default')throw 7;return text;}}).getTime()===0",
    );
}

#[test]
fn opted_in_string_limits_use_exact_output_lengths_and_leave_slots_unchanged() {
    for (offset, time) in [
        (0, 0_i64),
        (-17762, 0),
        (i32::MIN, -8640000000000000),
        (i32::MAX, 8640000000000000),
    ] {
        for method in METHODS {
            let source = format!("new Date({time}).{method}()");
            let Value::String(expected) = realm(TimeZone::fixed(offset)).eval(&source).unwrap()
            else {
                panic!("string output")
            };
            for (length, succeeds) in [(expected.len(), true), (expected.len() - 1, false)] {
                let mut r = Realm::new(Limits {
                    max_string_units: Some(length),
                    ..Limits::default()
                });
                r.set_time_zone(TimeZone::fixed(offset));
                r.eval(&format!("var d=new Date({time});var marker=0;"))
                    .unwrap();
                let output = r.eval(&format!(
                    "try{{d.{method}();}}catch{{marker=1;}}finally{{marker=2;}}"
                ));
                if succeeds {
                    assert_eq!(output, Ok(Value::String(expected.clone())));
                } else {
                    assert!(
                        matches!(output, Err(Error::Limit { .. })),
                        "{offset} {time} {method} {length}: {output:?}"
                    );
                    assert_eq!(r.eval("marker"), Ok(Value::Number(0.0)));
                }
                assert_eq!(r.eval("d.getTime()"), Ok(Value::Number(time as f64)));
            }
        }
    }
}

#[test]
fn recursive_primitive_conversion_preserves_stack_guards_and_recovers() {
    let mut r = realm(TimeZone::utc());
    r.eval("var marker=0;var d=new Date(7);d.toString=function(){return String(d);};")
        .unwrap();
    assert!(matches!(
        r.eval("try{String(d);}catch{marker=1;}finally{marker=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        r.eval("marker===0 && d.getTime()===7"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(
        r.eval("delete d.toString;typeof String(d)==='string' && d.getTime()===7"),
        Ok(Value::Boolean(true))
    );
}
