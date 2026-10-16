//! Edition-17 non-Intl locale methods use the host's fixed English conventions.

use spite_runtime::{Error, ExceptionKind, Realm, TimeZone, Value};

const METHODS: [(&str, &str); 3] = [
    ("toLocaleString", "toString"),
    ("toLocaleDateString", "toDateString"),
    ("toLocaleTimeString", "toTimeString"),
];

fn realm(zone: TimeZone) -> Realm {
    let mut r = Realm::default();
    r.set_time_zone(zone);
    r
}

#[test]
fn fallback_methods_use_the_local_calendar_and_ignore_reserved_arguments() {
    for zone in [
        "UTC",
        "America/New_York",
        "Australia/Lord_Howe",
        "Africa/Abidjan",
    ] {
        for (method, base) in METHODS {
            let mut r = realm(TimeZone::bundled(zone).unwrap());
            assert_eq!(r.eval(&format!("let touched=0,d=new Date(-1830383032001),poison={{get length(){{throw 7;}},get timeZone(){{throw 8;}},get localeMatcher(){{throw 9;}},[Symbol.toPrimitive](){{throw 10;}}}};let actual=d.{method}(poison,poison,touched++);actual===d.{base}() && touched===1")),Ok(Value::Boolean(true)),"{zone} {method}");
            assert_eq!(r.eval(&format!("d.{base}=function(){{throw 7;}};typeof Date.prototype.{method}.call(d,Symbol(),1n)==='string'")),Ok(Value::Boolean(true)));
        }
    }
}

#[test]
fn receiver_branding_frozen_slots_and_invalid_dates_follow_the_fallback_contract() {
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
        assert_eq!(realm(TimeZone::utc()).eval(&format!("let d=new Date(0);Object.setPrototypeOf(d,null);Object.freeze(d);typeof Date.prototype.{method}.call(d)==='string' && Object.isFrozen(d) && Date.prototype.getTime.call(d)===0")),Ok(Value::Boolean(true)));
        assert_eq!(
            realm(TimeZone::utc()).eval(&format!(
                "new Date(NaN).{method}(Symbol(),1n)==='Invalid Date'"
            )),
            Ok(Value::Boolean(true))
        );
    }
}
