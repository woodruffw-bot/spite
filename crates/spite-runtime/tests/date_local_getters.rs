//! Date local calendar reads use immutable per-realm zone histories.

use spite_runtime::{Error, ExceptionKind, Realm, TimeZone, Value};

const GETTERS: [&str; 9] = [
    "getFullYear",
    "getMonth",
    "getDate",
    "getDay",
    "getHours",
    "getMinutes",
    "getSeconds",
    "getMilliseconds",
    "getTimezoneOffset",
];

include!("fixtures/date_local_getters.rs");

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
fn local_fields_keep_historical_seconds_and_utc_timezone_offset_is_positive_zero() {
    check(
        "UTC",
        "let d=new Date(-1);d.getFullYear()===1969 && d.getMonth()===11 && d.getDate()===31 && d.getDay()===3 && d.getHours()===23 && d.getMinutes()===59 && d.getSeconds()===59 && d.getMilliseconds()===999 && 1/d.getTimezoneOffset()===Infinity",
    );
    check(
        "America/New_York",
        "let d=new Date(0);d.getFullYear()===1969 && d.getMonth()===11 && d.getDate()===31 && d.getDay()===3 && d.getHours()===19 && d.getMinutes()===0 && d.getSeconds()===0 && d.getMilliseconds()===0 && d.getTimezoneOffset()===300",
    );
    check(
        "America/New_York",
        "let d=new Date(Date.UTC(1800,0,1));d.getFullYear()===1799 && d.getMonth()===11 && d.getDate()===31 && d.getHours()===19 && d.getMinutes()===3 && d.getSeconds()===58 && d.getTimezoneOffset()===17762/60",
    );
    check(
        "Africa/Abidjan",
        "let before=new Date(-1830383032001),at=new Date(-1830383032000);before.getHours()===23 && before.getMinutes()===59 && before.getSeconds()===59 && before.getMilliseconds()===999 && before.getDate()===31 && before.getTimezoneOffset()===968/60 && at.getHours()===0 && at.getMinutes()===16 && at.getSeconds()===8 && at.getDate()===1 && 1/at.getTimezoneOffset()===Infinity",
    );
}

#[test]
fn calendar_reads_follow_exact_spring_and_autumn_transitions() {
    check(
        "America/New_York",
        "let before=new Date(Date.UTC(2024,2,10,6,59,59,999)),at=new Date(Date.UTC(2024,2,10,7));before.getHours()===1 && before.getMinutes()===59 && before.getMilliseconds()===999 && before.getTimezoneOffset()===300 && at.getHours()===3 && at.getMinutes()===0 && at.getTimezoneOffset()===240",
    );
    check(
        "America/New_York",
        "let early=new Date(Date.UTC(2024,10,3,5,30)),late=new Date(Date.UTC(2024,10,3,6,30));early.getHours()===1 && early.getMinutes()===30 && early.getTimezoneOffset()===240 && late.getHours()===1 && late.getMinutes()===30 && late.getTimezoneOffset()===300",
    );
    check(
        "Australia/Lord_Howe",
        "let before=new Date(Date.UTC(2024,9,5,15,29,59,999)),at=new Date(Date.UTC(2024,9,5,15,30));before.getHours()===1 && before.getMinutes()===59 && before.getTimezoneOffset()===-630 && at.getHours()===2 && at.getMinutes()===30 && at.getTimezoneOffset()===-660",
    );
}

#[test]
fn local_calendar_intermediates_remain_unclipped_at_every_native_offset_extreme() {
    for (offset, time, expected) in LOCAL_GETTER_CASES {
        let mut r = realm(TimeZone::fixed(offset));
        r.eval(&format!("var d=new Date({time});")).unwrap();
        for (method, expected) in GETTERS.into_iter().zip(expected) {
            assert_eq!(
                r.eval(&format!("d.{method}()")),
                Ok(Value::Number(expected)),
                "{offset} {time} {method}"
            );
        }
    }
    let mut r = realm(TimeZone::fixed(3600));
    assert_eq!(
        r.eval("new Date(8640000000000000).getHours()===1"),
        Ok(Value::Boolean(true))
    );
    let mut r = realm(TimeZone::fixed(-3600));
    assert_eq!(
        r.eval("let d=new Date(-8640000000000000);d.getDate()===19 && d.getHours()===23"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn getters_check_the_internal_slot_and_ignore_arguments_after_caller_evaluation() {
    let mut r = realm(TimeZone::utc());
    for method in GETTERS {
        for receiver in ["Date.prototype", "{}", "0", "null", "undefined"] {
            assert!(
                matches!(
                    r.eval(&format!("Date.prototype.{method}.call({receiver})")),
                    Err(Error::Exception {
                        kind: ExceptionKind::TypeError,
                        ..
                    })
                ),
                "{method} {receiver}"
            );
        }
        assert_eq!(
            r.eval(&format!(
                "var n=0,d=new Date(0);d.{method}(n++,{{valueOf(){{throw 7;}}}});n===1"
            )),
            Ok(Value::Boolean(true))
        );
        assert_eq!(
            r.eval(&format!("var d=new Date(NaN);Number.isNaN(d.{method}())")),
            Ok(Value::Boolean(true))
        );
    }
    check(
        "UTC",
        "class D extends Date{}let d=new D(0);Object.setPrototypeOf(d,null);Date.prototype.getFullYear.call(d)===1970",
    );
}

#[test]
fn switching_one_realms_zone_changes_existing_dates_without_mutating_their_time_value() {
    let mut first = realm(TimeZone::utc());
    let mut second = realm(TimeZone::fixed(3600));
    first.eval("var d=new Date(0)").unwrap();
    second.eval("var d=new Date(0)").unwrap();
    assert_eq!(first.eval("d.getHours()"), Ok(Value::Number(0.0)));
    assert_eq!(second.eval("d.getHours()"), Ok(Value::Number(1.0)));
    first.set_time_zone(TimeZone::fixed(-3600));
    assert_eq!(
        first.eval("d.getHours()===23 && d.getTime()===0"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(
        second.eval("d.getHours()===1 && d.getTime()===0"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn local_reads_remain_safe_under_recursive_calls_on_the_default_native_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let mut r = realm(TimeZone::bundled("America/New_York").unwrap());
            r.eval("var d=new Date(0);function f(n){return n?f(n-1):d.getFullYear();}")
                .unwrap();
            assert_eq!(r.eval("f(12)"), Ok(Value::Number(1969.0)));
            assert!(matches!(r.eval("f(300)"), Err(Error::Limit { .. })));
            assert_eq!(
                r.eval("d.getTime()===0 && d.getHours()===19"),
                Ok(Value::Boolean(true))
            );
        })
        .unwrap()
        .join()
        .unwrap();
}
