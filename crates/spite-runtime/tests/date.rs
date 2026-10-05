//! Date timestamps, UTC fields, receiver branding, and observable conversion.

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

fn type_error(source: &str) {
    assert!(
        matches!(
            Realm::default().eval(source),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ),
        "{source}"
    );
}

#[test]
fn timestamp_inputs_clip_preserve_brand_and_distinguish_absent_undefined() {
    check(
        "let d=new Date; Number.isFinite(d.getTime()) && Number.isInteger(d.getTime()) && Math.abs(d.getTime())<=8640000000000000 && Number.isFinite(Date.now()) && Number.isInteger(Date.now())",
    );
    check(
        "new Date(null).getTime()===0 && new Date(false).getTime()===0 && new Date(true).getTime()===1 && new Date(1.9).valueOf()===1 && new Date(-1.9).getTime()===-1 && 1/new Date(-0).getTime()===Infinity && 1/new Date(-0.9).getTime()===Infinity",
    );
    for value in [
        "undefined",
        "NaN",
        "Infinity",
        "-Infinity",
        "8640000000000001",
        "-8640000000000001",
    ] {
        check(&format!(
            "let d=new Date({value}); Number.isNaN(d.getTime()) && ({{}}).toString.call(d)==='[object Date]'"
        ));
    }
    for source in ["new Date(Symbol())", "new Date(1n)"] {
        type_error(source);
    }
    check(
        "let D=Date.bind(null,-1); new D().getTime()===-1 && new D instanceof Date && new D instanceof D",
    );
}

#[test]
fn interchange_strings_parse_with_offsets_rollover_and_full_domain_clipping() {
    for (text, value) in [
        ("1970", 0.0),
        ("1970T00:00Z", 0.0),
        ("1970-01-01T00:00:00.001Z", 1.0),
        ("1970-01-01T00:01+00:01", 0.0),
        ("1970-01-01T00:00-00:01", 60000.0),
        ("2000-02-30T24:00:00.000Z", 951955200000.0),
        ("0000-01-01", -62167219200000.0),
        ("-271821-04-20T00:00:00.000Z", -8640000000000000.0),
        ("+275760-09-13T00:00:00.000Z", 8640000000000000.0),
    ] {
        check(&format!(
            "Date.parse('{text}')==={value} && new Date('{text}').getTime()==={value}"
        ));
    }
    for text in [
        "bad",
        "-000000-01-01",
        "1970-13-01",
        "1970-01-32",
        "1970-01-01T24:01Z",
        "1970-01-01T00:00:00.0000Z",
        "+275760-09-13T00:00:00.001Z",
        "-271821-04-19T23:59:59.999Z",
    ] {
        check(&format!(
            "Number.isNaN(Date.parse('{text}')) && Number.isNaN(new Date('{text}').getTime())"
        ));
    }
    check(
        "let hint='',calls=0; Date.parse({[Symbol.toPrimitive](h){hint=h;calls++;return '1970';}})===0 && hint==='string' && calls===1",
    );
    type_error("Date.parse(Symbol())");
}

#[test]
fn copying_dates_bypasses_hooks_but_other_inputs_observe_default_hint() {
    check(
        "let original=new Date(7); Object.defineProperty(original,Symbol.toPrimitive,{value(){throw 1;}}); original.valueOf=()=>{throw 2;}; let copied=new Date(original); copied!==original && copied.getTime()===7",
    );
    check(
        "let original=new Date(NaN); Object.defineProperty(original,Symbol.toPrimitive,{value(){throw 1;}}); Number.isNaN(new Date(original).getTime())",
    );
    check(
        "let trace=''; let d=new Date({[Symbol.toPrimitive](hint){trace+=hint;return '1970';},valueOf(){throw 1;}}); d.getTime()===0 && trace==='default'",
    );
    check(
        "let trace=''; new Date({valueOf(){trace+='v';return {};},toString(){trace+='s';return '1970';}}).getTime()===0 && trace==='vs'",
    );
    check(
        "let original=new Date(7), inherited=Object.create(original); Object.defineProperty(inherited,Symbol.toPrimitive,{value:()=>9});new Date(inherited).getTime()===9",
    );
    type_error("new Date({[Symbol.toPrimitive](){return {};}})");
}

#[test]
fn constructor_converts_before_selecting_newtarget_prototype_and_supports_subclasses() {
    check(
        "let trace='',p={}; function C(){} let input={valueOf(){trace+='v'; C.prototype=p;return 7;}}; let d=Reflect.construct(Date,[input],C); trace==='v' && Object.getPrototypeOf(d)===p && Date.prototype.getTime.call(d)===7 && ({}).toString.call(d)==='[object Date]'",
    );
    check(
        "function C(){} C.prototype=3; Object.getPrototypeOf(Reflect.construct(Date,[0],C))===Date.prototype",
    );
    check(
        "class D extends Date { #x=3; read(){return this.#x+this.getTime();} } let d=new D(7); d instanceof Date && d instanceof D && d.read()===10",
    );
    let mut realm = Realm::default();
    let value = realm.eval("class D extends Date {} new D(9)").unwrap();
    let root = realm.root_value(value, usize::MAX).unwrap();
    realm.collect(usize::MAX).unwrap();
    let method = realm
        .read_property(root.value(), &JsString::from("getTime"))
        .unwrap();
    assert!(matches!(method, Value::Object(_)));
}

#[test]
fn utc_getters_decompose_epoch_negative_milliseconds_leap_days_and_extremes() {
    check(
        "let d=new Date(-1); d.getUTCFullYear()===1969 && d.getUTCMonth()===11 && d.getUTCDate()===31 && d.getUTCDay()===3 && d.getUTCHours()===23 && d.getUTCMinutes()===59 && d.getUTCSeconds()===59 && d.getUTCMilliseconds()===999",
    );
    check(
        "let d=new Date('2000-02-29T12:34:56.789Z'); d.getUTCFullYear()===2000 && d.getUTCMonth()===1 && d.getUTCDate()===29 && d.getUTCDay()===2 && d.getUTCHours()===12 && d.getUTCMinutes()===34 && d.getUTCSeconds()===56 && d.getUTCMilliseconds()===789",
    );
    check(
        "let lo=new Date(-8640000000000000),hi=new Date(8640000000000000);lo.getUTCFullYear()===-271821 && lo.getUTCMonth()===3 && lo.getUTCDate()===20 && hi.getUTCFullYear()===275760 && hi.getUTCMonth()===8 && hi.getUTCDate()===13",
    );
    for method in [
        "getUTCFullYear",
        "getUTCMonth",
        "getUTCDate",
        "getUTCDay",
        "getUTCHours",
        "getUTCMinutes",
        "getUTCSeconds",
        "getUTCMilliseconds",
    ] {
        check(&format!(
            "Number.isNaN(new Date(NaN).{method}({{valueOf(){{throw 1;}}}}))"
        ));
    }
}

#[test]
fn branded_methods_reject_inherited_slots_and_primitive_receivers_before_arguments() {
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
        for method in [
            "getTime",
            "valueOf",
            "getUTCFullYear",
            "toISOString",
            "toString",
            "setTime",
            "setUTCDate",
        ] {
            type_error(&format!(
                "Date.prototype.{method}.call({receiver},{{valueOf(){{throw 7;}}}})"
            ));
        }
    }
    check(
        "let d=new Date(0);Object.setPrototypeOf(d,null); Date.prototype.getTime.call(d)===0 && ({}).toString.call(d)==='[object Date]'",
    );
    check(
        "let d=new Date(0); d[Symbol.toStringTag]='Custom'; ({}).toString.call(d)==='[object Custom]' && d.getTime()===0",
    );
    check("({}).toString.call(Date.prototype)==='[object Object]'");
}

#[test]
fn settime_converts_once_clips_and_commits_only_after_successful_conversion() {
    check(
        "let d=new Date(7),calls=0; Object.freeze(d);d.setTime({valueOf(){calls++;d.setTime(9);return -1.9;}})===-1 && d.getTime()===-1 && calls===1 && 1/d.setTime(-0)===Infinity && 1/d.getTime()===Infinity && Number.isNaN(d.setTime()) && Number.isNaN(d.getTime()) && d.setTime(3)===3",
    );
    check(
        "let d=new Date(7),caught=false;try{d.setTime({valueOf(){throw 9;}});}catch(e){caught=e===9;}caught && d.getTime()===7",
    );
    type_error("new Date(0).setTime(1n)");
    check("let d=new Date(0); Number.isNaN(d.setTime(Infinity)) && Number.isNaN(d.getTime())");
}

#[test]
fn iso_and_json_serialization_preserve_expanded_years_and_invalid_dates() {
    for (time, text) in [
        (0.0, "1970-01-01T00:00:00.000Z"),
        (-1.0, "1969-12-31T23:59:59.999Z"),
        (-8640000000000000.0, "-271821-04-20T00:00:00.000Z"),
        (8640000000000000.0, "+275760-09-13T00:00:00.000Z"),
    ] {
        check(&format!(
            "let d=new Date({time}); d.toISOString()==='{text}' && d.toJSON()==='{text}' && JSON.stringify([d])==='[\"{text}\"]'"
        ));
    }
    check(
        "let d=new Date(NaN); d.toJSON()===null && JSON.stringify(d)==='null' && d.toString()==='Invalid Date' && d.toDateString()==='Invalid Date' && d.toTimeString()==='Invalid Date' && d.toUTCString()==='Invalid Date'",
    );
    assert!(matches!(
        Realm::default().eval("new Date(NaN).toISOString()"),
        Err(Error::Exception {
            kind: ExceptionKind::RangeError,
            ..
        })
    ));
}

#[test]
fn json_conversion_is_generic_observes_numeric_hint_and_invokes_original_object() {
    check(
        "let trace='',o={valueOf(){trace+='v';return 1;},get toISOString(){trace+='g';return function(){trace+='c';return this===o && arguments.length===0;};}}; Date.prototype.toJSON.call(o,{toString(){throw 1;}})===true && trace==='vgc'",
    );
    for number in ["NaN", "Infinity", "-Infinity"] {
        check(&format!(
            "Date.prototype.toJSON.call({{valueOf(){{return {number};}},get toISOString(){{throw 7;}}}})===null"
        ));
    }
    for primitive in ["'text'", "true", "undefined", "null", "Symbol()", "1n"] {
        check(&format!(
            "let h='',o={{[Symbol.toPrimitive](hint){{h=hint;return {primitive};}},toISOString(){{return 7;}}}};Date.prototype.toJSON.call(o)===7 && h==='number'"
        ));
    }
    for receiver in ["undefined", "null", "{valueOf(){return 0;},toISOString:7}"] {
        type_error(&format!("Date.prototype.toJSON.call({receiver})"));
    }
}

#[test]
fn primitive_conversion_is_generic_validates_hint_without_coercion_and_orders_methods() {
    for (hint, trace, expected) in [
        ("default", "s", "'text'"),
        ("string", "s", "'text'"),
        ("number", "v", "7"),
    ] {
        check(&format!(
            "let trace='',o={{toString(){{trace+='s';return 'text';}},valueOf(){{trace+='v';return 7;}}}};Date.prototype[Symbol.toPrimitive].call(o,'{hint}')==={expected} && trace==='{trace}'"
        ));
    }
    check(
        "let trace='',o={toString(){trace+='s';return {};},valueOf(){trace+='v';return 7;}};Date.prototype[Symbol.toPrimitive].call(o,'default')===7 && trace==='sv'",
    );
    for hint in [
        "undefined",
        "null",
        "1",
        "'bad'",
        "new String('number')",
        "{toString(){throw 7;}}",
    ] {
        type_error(&format!(
            "Date.prototype[Symbol.toPrimitive].call({{}},{hint})"
        ));
    }
    for receiver in ["undefined", "null", "0", "'text'", "true", "Symbol()", "1n"] {
        type_error(&format!(
            "Date.prototype[Symbol.toPrimitive].call({receiver},'number')"
        ));
    }
    check(
        "let d=new Date(7);d.toString=()=> 'text'; +d===7 && d+1==='text1' && `${d}`==='text' && String(d)==='text'",
    );
    check(
        "String(new Date(NaN))==='Invalid Date' && Number(new Date(NaN))!==Number(new Date(NaN))",
    );
}

#[test]
fn unsupported_calendar_and_local_operations_remain_host_failures_and_keep_coercion_order() {
    for source in [
        "Date()",
        "new Date(2026,0)",
        "Date.UTC(2026)",
        "Date.parse('1970-01-01T00:00')",
        "new Date('1970-01-01T00:00')",
        "new Date(0).getFullYear()",
        "new Date(0).toString()",
        "new Date(0)+1",
        "new Date(0).setUTCMonth(2)",
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "try{{{source};}}catch{{marker=1;}}finally{{marker=2;}}"
                )),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
        assert_eq!(realm.eval("new Date(7).getTime()"), Ok(Value::Number(7.0)));
    }
    for source in [
        "new Date({valueOf(){trace+='a';return NaN;}},{valueOf(){trace+='b';throw 7;}})",
        "Date.UTC({valueOf(){trace+='a';return NaN;}},{valueOf(){trace+='b';throw 7;}})",
    ] {
        let mut realm = Realm::default();
        realm.eval("var trace='' ").unwrap();
        assert_eq!(realm.eval(source), Err(Error::Thrown(Value::Number(7.0))));
        assert_eq!(realm.eval("trace"), Ok(Value::String(JsString::from("ab"))));
    }
    check(
        "let flag=0,caught=false;try{new Date(2026,Symbol(),flag++);}catch(e){caught=e instanceof TypeError;}caught && flag===1",
    );
    check("let flag=0;try{Date.parse({toString(){flag=1;throw 9;}});}catch(e){}flag===1");
}
