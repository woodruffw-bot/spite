//! Calendar construction and local interchange parsing resolve before TimeClip.

use spite_runtime::{Error, ExceptionKind, Limits, Realm, TimeZone, Value};

include!("fixtures/date_local_getters.rs");

fn realm(zone: TimeZone) -> Realm {
    let mut r = Realm::default();
    r.set_time_zone(zone);
    r
}

fn check(zone: &str, source: &str) {
    assert_eq!(
        realm(TimeZone::bundled(zone).unwrap()).eval(source),
        Ok(Value::Boolean(true)),
        "{zone}: {source}"
    );
}

#[test]
fn numeric_components_normalize_defaults_short_years_and_ordered_truncation() {
    check(
        "UTC",
        "new Date(1970,0).getTime()===0 && new Date(0,0).getTime()===Date.UTC(1900,0) && new Date(99,0).getTime()===Date.UTC(1999,0) && new Date(100,0).getTime()===Date.UTC(100,0)",
    );
    check(
        "UTC",
        "new Date(2024,12,0,25,-1,61,-1).getTime()===Date.UTC(2024,12,0,25,-1,61,-1) && new Date(2024.9,2.9,10.9,2.9,30.9,12.9,987.9).getTime()===Date.UTC(2024,2,10,2,30,12,987)",
    );
    check(
        "UTC",
        "Number.isNaN(new Date(1970,0,undefined).getTime()) && Number.isNaN(new Date(1970,0,1,undefined).getTime()) && 1/new Date(1970,0,1,0,0,0,-0).getTime()===Infinity",
    );
    check(
        "America/New_York",
        "new Date(1970,0).getTime()===18000000 && new Date(2024,6).getTime()===Date.UTC(2024,6,1,4)",
    );
}

#[test]
fn local_inputs_choose_earlier_folds_and_preceding_offsets_for_skipped_times() {
    for (zone, year, month, day, hour, minute, utc) in [
        (
            "America/New_York",
            2024,
            2,
            10,
            2,
            30,
            "2024-03-10T07:30:00Z",
        ),
        (
            "America/New_York",
            2024,
            10,
            3,
            1,
            30,
            "2024-11-03T05:30:00Z",
        ),
        (
            "Australia/Lord_Howe",
            2024,
            9,
            6,
            2,
            15,
            "2024-10-05T15:45:00Z",
        ),
        (
            "Australia/Lord_Howe",
            2024,
            3,
            7,
            1,
            45,
            "2024-04-06T14:45:00Z",
        ),
        ("Pacific/Apia", 2011, 11, 30, 12, 0, "2011-12-30T22:00:00Z"),
    ] {
        let local = format!(
            "{year:04}-{:02}-{day:02}T{hour:02}:{minute:02}:00",
            month + 1
        );
        check(
            zone,
            &format!(
                "let expected=Date.parse('{utc}');new Date({year},{month},{day},{hour},{minute}).getTime()===expected && Date.parse('{local}')===expected && new Date('{local}').getTime()===expected"
            ),
        );
    }
    check(
        "America/New_York",
        "new Date(2024,2,10,2,30,12,987).getTime()===Date.UTC(2024,2,10,7,30,12,987) && Date.parse('2024-03-10T02:30:12.987')===Date.UTC(2024,2,10,7,30,12,987)",
    );
}

#[test]
fn interchange_dates_keep_utc_date_only_forms_literal_years_and_calendar_rollover() {
    check(
        "America/New_York",
        "Date.parse('1970')===0 && Date.parse('1970-01')===0 && Date.parse('1970-01-01')===0 && Date.parse('1970-01-01T00:00')===18000000 && Date.parse('1970-01-01T00:00Z')===0 && Date.parse('1970-01-01T00:00+01:00')===-3600000",
    );
    check(
        "UTC",
        "Date.parse('0000-01-01T00:00')===Date.parse('0000-01-01') && Date.parse('0001-01-01T00:00')===Date.parse('0001-01-01') && Date.parse('-000001-01-01T00:00')===Date.parse('-000001-01-01') && Date.parse('2024-02-31T24:00')===Date.UTC(2024,2,3)",
    );
    check(
        "America/New_York",
        "Date.parse('0000-01-01T00:00')===Date.parse('0000-01-01')+17762000 && Date.parse('+020000-01-01T00:00')===Date.parse('+020000-01-01T05:00Z') && Date.parse('+020000-07-01T00:00')===Date.parse('+020000-07-01T04:00Z')",
    );
}

#[test]
fn both_boundaries_round_trip_calendar_and_string_inputs_across_every_native_offset() {
    for (offset, time, fields) in LOCAL_GETTER_CASES {
        let mut r = realm(TimeZone::fixed(offset));
        let args = format!(
            "{},{},{},{},{},{},{}",
            fields[0], fields[1], fields[2], fields[4], fields[5], fields[6], fields[7]
        );
        assert_eq!(
            r.eval(&format!("new Date({args}).getTime()")),
            Ok(Value::Number(time as f64)),
            "{offset} {time} {args}"
        );
        let year = fields[0] as i32;
        let year = if (0..=9999).contains(&year) {
            format!("{year:04}")
        } else {
            format!(
                "{}{:06}",
                if year < 0 { '-' } else { '+' },
                year.unsigned_abs()
            )
        };
        let text = format!(
            "{year}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}",
            fields[1] as u8 + 1,
            fields[2] as u8,
            fields[4] as u8,
            fields[5] as u8,
            fields[6] as u8,
            fields[7] as u16
        );
        assert_eq!(
            r.eval(&format!("Date.parse('{text}')")),
            Ok(Value::Number(time as f64)),
            "{offset} {time} {text}"
        );
    }
    let mut r = realm(TimeZone::fixed(3600));
    assert_eq!(r.eval("Date.parse('+275760-09-13T01:00')===8640000000000000 && new Date(275760,8,13,1).getTime()===8640000000000000 && Number.isNaN(new Date(275760,8,13,1,0,0,1).getTime())"),Ok(Value::Boolean(true)));
    let mut r = realm(TimeZone::fixed(-3600));
    assert_eq!(r.eval("Date.parse('-271821-04-19T23:00')===-8640000000000000 && new Date(-271821,3,19,23).getTime()===-8640000000000000 && Number.isNaN(new Date(-271821,3,19,23,0,0,-1).getTime())"),Ok(Value::Boolean(true)));
}

#[test]
fn huge_finite_calendar_cancellations_are_not_limited_by_js_bigint_value_quotas() {
    let mut r = Realm::new(Limits {
        max_bigint_bits: Some(1),
        ..Limits::default()
    });
    r.set_time_zone(TimeZone::utc());
    assert_eq!(r.eval("new Date(-Number.MAX_VALUE/12,Number.MAX_VALUE,1).getTime()===Date.parse('0000-09-01') && new Date(Number.MAX_VALUE/12,-Number.MAX_VALUE,1).getTime()===Date.parse('0000-05-01') && new Date(5000000000,0,1-1826211780472).getTime()===0 && new Date(1e200,0,-3.652425e202).getTime()===-86400000"),Ok(Value::Boolean(true)));
    check(
        "UTC",
        "Number.isNaN(new Date(Number.MAX_VALUE,0).getTime()) && Number.isNaN(new Date(1970,0,1,0,0,0,Number.MAX_VALUE).getTime()) && Number.isNaN(Date.parse('+999999-01-01T00:00'))",
    );
}

#[test]
fn all_present_conversions_precede_prototype_lookup_and_excess_arguments_are_ignored() {
    let args = (0..7)
        .map(|i| {
            format!(
                "{{valueOf(){{trace+='{i}';return {};}}}}",
                ["1970", "0", "1", "0", "0", "0", "0"][i]
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    check(
        "UTC",
        &format!(
            "var trace='',p={{}},C=(function(){{}}).bind(null);Object.defineProperty(C,'prototype',{{get(){{trace+='p';return p;}}}});var d=Reflect.construct(Date,[{args},{{valueOf(){{throw 9;}}}}],C);trace==='0123456p' && Object.getPrototypeOf(d)===p && Date.prototype.getTime.call(d)===0"
        ),
    );
    check(
        "UTC",
        "let n=0;new Date(1970,0,1,0,0,0,0,n++,{valueOf(){throw 9;}}).getTime()===0 && n===1",
    );
    check(
        "UTC",
        "class D extends Date{#x=7;read(){return this.#x;}}let d=new D(1970,0);d instanceof D && d instanceof Date && d.read()===7 && d.getTime()===0",
    );
    let mut r = realm(TimeZone::utc());
    for source in ["new Date(1970,Symbol())", "new Date(1970,0,1n)"] {
        assert!(matches!(
            r.eval(source),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    for failure in 0..7 {
        let args = (0..7)
            .map(|i| {
                format!(
                    "{{valueOf(){{trace+='{i}';{};}}}}",
                    if i == failure { "throw 7" } else { "return 0" }
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        r.eval("var trace='';").unwrap();
        assert_eq!(
            r.eval(&format!("new Date({args})")),
            Err(Error::Thrown(Value::Number(7.0)))
        );
        assert_eq!(
            r.eval(&format!(
                "trace==='{}'",
                (0..=failure).map(|i| i.to_string()).collect::<String>()
            )),
            Ok(Value::Boolean(true))
        );
    }
}

#[test]
fn string_constructor_and_parse_apply_their_existing_distinct_coercion_hints() {
    check(
        "America/New_York",
        "let a='',b='',x={ [Symbol.toPrimitive](h){a=h;return '1970-01-01T00:00';}},y={ [Symbol.toPrimitive](h){b=h;return '1970-01-01T00:00';}};new Date(x).getTime()===18000000 && Date.parse(y)===18000000 && a==='default' && b==='string'",
    );
    check(
        "UTC",
        "let d=new Date(7);d[Symbol.toPrimitive]=()=>{throw 9;};new Date(d).getTime()===7",
    );
}

#[test]
fn calendar_coercion_and_prototype_reentry_keep_default_stack_guards() {
    std::thread::Builder::new().stack_size(2*1024*1024).spawn(|| {
        let mut r=realm(TimeZone::utc());
        r.eval("var flag=0,arg={valueOf(){return new Date(arg,0);}};").unwrap();
        assert!(matches!(r.eval("try{new Date(arg,0);}catch{flag=1;}finally{flag=2;}"),Err(Error::Limit{..})));
        assert_eq!(r.eval("flag===0 && new Date(1970,0).getTime()===0"),Ok(Value::Boolean(true)));
        r.eval("var C=(function(){}).bind(null);Object.defineProperty(C,'prototype',{get(){return Reflect.construct(Date,[1970,0],C);}});").unwrap();
        assert!(matches!(r.eval("Reflect.construct(Date,[1970,0],C)"),Err(Error::Limit{..})));
        assert_eq!(r.eval("new Date(1970,0).getTime()===0"),Ok(Value::Boolean(true)));
    }).unwrap().join().unwrap();
}
