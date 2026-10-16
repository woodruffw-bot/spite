//! Standard UTC Date formatting, parsing, and observable receiver/coercion rules.

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn utc_strings_use_standard_names_signed_years_and_containing_seconds() {
    for (time, text, parsed) in [
        (0_i64, "Thu, 01 Jan 1970 00:00:00 GMT", 0_i64),
        (-1, "Wed, 31 Dec 1969 23:59:59 GMT", -1000),
        (951827696789, "Tue, 29 Feb 2000 12:34:56 GMT", 951827696000),
        (
            -62167219200000,
            "Sat, 01 Jan 0000 00:00:00 GMT",
            -62167219200000,
        ),
        (-MAX_TIME, "Tue, 20 Apr -271821 00:00:00 GMT", -MAX_TIME),
        (MAX_TIME, "Sat, 13 Sep 275760 00:00:00 GMT", MAX_TIME),
    ] {
        check(&format!(
            "let d=new Date({time}); d.toUTCString()==='{text}' && Date.parse(d.toUTCString())==={parsed} && new Date('{text}').getTime()==={parsed} && d.getTime()==={time}"
        ));
    }
    check(
        "new Date(NaN).toUTCString()==='Invalid Date' && Number.isNaN(Date.parse('Invalid Date'))",
    );
}

const MAX_TIME: i64 = 8_640_000_000_000_000;

#[test]
fn utc_string_round_trips_preserve_literal_short_and_expanded_years() {
    for text in [
        "0000-01-01",
        "0001-01-01",
        "0012-01-01",
        "0099-01-01",
        "9999-01-01",
        "+010000-01-01",
        "+100000-01-01",
        "-000001-01-01",
        "-000012-01-01",
        "-000123-01-01",
        "-001234-01-01",
        "-012345-01-01",
        "-123456-01-01",
    ] {
        check(&format!(
            "let d=new Date('{text}'),s=d.toUTCString(); Date.parse(s)===d.getTime() && new Date(s).toISOString()===d.toISOString()"
        ));
    }
}

#[test]
fn formatting_checks_own_date_brand_and_ignores_conversion_hooks_and_arguments() {
    check(
        "let d=new Date(0);Object.defineProperty(d,Symbol.toPrimitive,{value(){throw 1;}});d.valueOf=()=>{throw 2;};Object.freeze(d);Object.setPrototypeOf(d,Date.prototype); Date.prototype.toUTCString.call(d,{toString(){throw 3;}})==='Thu, 01 Jan 1970 00:00:00 GMT'",
    );
    check(
        "let d=new Date(0);Object.setPrototypeOf(d,null);Date.prototype.toUTCString.call(d)==='Thu, 01 Jan 1970 00:00:00 GMT'",
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
        "new Number(0)",
    ] {
        assert!(
            matches!(
                Realm::default().eval(&format!(
                    "Date.prototype.toUTCString.call({receiver},{{toString(){{throw 7;}}}})"
                )),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{receiver}"
        );
    }
}

#[test]
fn parsing_converts_once_with_string_hint_and_propagates_abrupt_completions() {
    check(
        "let trace='',o={ [Symbol.toPrimitive](hint){trace+=hint;return 'Thu, 01 Jan 1970 00:00:00 GMT';},toString(){throw 1;}};Date.parse(o)===0 && trace==='string'",
    );
    check(
        "let trace='',o={toString(){trace+='s';return 'Thu, 01 Jan 1970 00:00:00 GMT';},valueOf(){throw 1;}};Date.parse(o)===0 && trace==='s'",
    );
    check("let caught=false;try{Date.parse({toString(){throw 7;}});}catch(e){caught=e===7;}caught");
    assert_eq!(
        Realm::default().eval("Date.parse({toString(){throw 7;}})"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    assert_eq!(
        Realm::default().eval("Date.parse('Thu, 01 Jan 1970 00:00:00 GMT')"),
        Ok(Value::Number(0.0))
    );
    check(
        "Number.isNaN(Date.parse('Wed, 01 Jan 1970 00:00:00 GMT')) && Number.isNaN(Date.parse('Wed, 31 Apr 2024 00:00:00 GMT')) && Number.isNaN(Date.parse('Thu, 01 Jan 1970 00:00:00 UTC'))",
    );
}

#[test]
fn recursive_parsing_hooks_respect_native_stack_guards_and_restore_state() {
    let mut realm = Realm::default();
    realm
        .eval("var flag=0;var o={toString(){return Date.parse(o);}}")
        .unwrap();
    assert!(matches!(
        realm.eval("try{Date.parse(o);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(
        realm.eval("new Date(0).toUTCString()"),
        Ok(Value::String(JsString::from(
            "Thu, 01 Jan 1970 00:00:00 GMT"
        )))
    );
}
