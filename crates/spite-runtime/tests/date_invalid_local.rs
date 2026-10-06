//! Invalid Date branches resolve before local time-zone conversion.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

const GETTERS: [&str; 9] = [
    "getDate",
    "getDay",
    "getFullYear",
    "getHours",
    "getMilliseconds",
    "getMinutes",
    "getMonth",
    "getSeconds",
    "getTimezoneOffset",
];
const SETTERS: [(&str, usize); 6] = [
    ("setDate", 1),
    ("setMonth", 2),
    ("setHours", 4),
    ("setMinutes", 3),
    ("setSeconds", 2),
    ("setMilliseconds", 1),
];

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn numeric_construction_creates_invalid_dates_for_nonfinite_calendar_or_time_results() {
    for index in 0..7 {
        for value in ["undefined", "NaN", "Infinity", "-Infinity"] {
            let mut args = ["1970", "0", "1", "0", "0", "0", "0"];
            args[index] = value;
            check(&format!(
                "let d=new Date({});d instanceof Date && Number.isNaN(d.getTime()) && d.toJSON()===null",
                args.join(",")
            ));
        }
    }
    for args in [
        "Number.MAX_VALUE,0",
        "1970,Number.MAX_VALUE",
        "1970,0,Number.MAX_VALUE",
        "1970,0,1,Number.MAX_VALUE",
        "1970,0,1,0,Number.MAX_VALUE",
        "1970,0,1,0,0,Number.MAX_VALUE",
        "1e100,0,-3.652425e102",
    ] {
        check(&format!("Number.isNaN(new Date({args}).getTime())"));
    }
    check(
        "let d=new Date(NaN,0),caught=false;try{d.toISOString();}catch(e){caught=e instanceof RangeError;}caught && d.toString()==='Invalid Date'",
    );
}

#[test]
fn numeric_construction_coerces_all_seven_fields_in_order_before_prototype_lookup() {
    let args = (0..7)
        .map(|i| {
            format!(
                "{{[Symbol.toPrimitive](h){{if(h!=='number')throw 1;trace+='{i}';return NaN;}}}}"
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    check(&format!(
        "let trace='',count=0;let d=new Date({args},count++,{{valueOf(){{throw 7;}}}});trace==='0123456' && count===1 && Number.isNaN(d.getTime())"
    ));
    check(
        "let p={},trace='';function C(){}let x={valueOf(){trace+='y';return NaN;}},y={valueOf(){trace+='m';C.prototype=p;return 0;}};let d=Reflect.construct(Date,[x,y],C);trace==='ym' && Object.getPrototypeOf(d)===p && Number.isNaN(Date.prototype.getTime.call(d))",
    );
    check(
        "function C(){}C.prototype=3;let d=Reflect.construct(Date,[NaN,0],C);Object.getPrototypeOf(d)===Date.prototype && Number.isNaN(d.getTime())",
    );
    check(
        "let trace='',p={},C=(function(){}).bind(null);Object.defineProperty(C,'prototype',{get(){trace+='p';return p;}});let d=Reflect.construct(Date,[{valueOf(){trace+='y';return NaN;}},{valueOf(){trace+='m';return 0;}}],C);trace==='ymp' && Object.getPrototypeOf(d)===p && Number.isNaN(Date.prototype.getTime.call(d))",
    );
    check(
        "let trace='',caught=false,C=(function(){}).bind(null);Object.defineProperty(C,'prototype',{get(){trace+='p';throw 9;}});try{Reflect.construct(Date,[{valueOf(){trace+='y';return NaN;}},0],C);}catch(e){caught=e===9;}caught && trace==='yp'",
    );
    check(
        "class D extends Date{#x=7;read(){return this.#x;}}let d=new D(NaN,0);d instanceof D && d instanceof Date && d.read()===7 && Number.isNaN(d.getTime())",
    );
}

#[test]
fn later_abrupt_conversions_still_throw_after_an_earlier_nonfinite_field() {
    for failure in 0..7 {
        let args = (0..7)
            .map(|i| {
                format!(
                    "{{valueOf(){{trace+='{i}';{} }}}}",
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
            "let trace='',caught=false;try{{new Date({args});}}catch(e){{caught=e===7;}}caught && trace==='{expected}'"
        ));
    }
    for value in ["1n", "Symbol()"] {
        assert!(matches!(
            Realm::default().eval(&format!("new Date(NaN,{value})")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
}

#[test]
fn finite_local_setters_remain_unresolved() {
    for source in ["new Date(0).setMonth(0)", "new Date(NaN).setFullYear(2000)"] {
        let mut realm = Realm::default();
        assert!(
            matches!(realm.eval(source), Err(Error::Unsupported { .. })),
            "{source}"
        );
        assert_eq!(realm.eval("new Date(7).getTime()"), Ok(Value::Number(7.0)));
    }
}

#[test]
fn local_getters_return_nan_for_invalid_dates_and_check_their_own_slot() {
    for method in GETTERS {
        check(&format!(
            "let d=new Date(NaN);Number.isNaN(d.{method}({{valueOf(){{throw 7;}}}})) && Number.isNaN(d.getTime())"
        ));
        for receiver in [
            "undefined",
            "null",
            "0",
            "{}",
            "Date.prototype",
            "Object.create(new Date(NaN))",
        ] {
            assert!(
                matches!(
                    Realm::default().eval(&format!("Date.prototype.{method}.call({receiver})")),
                    Err(Error::Exception {
                        kind: ExceptionKind::TypeError,
                        ..
                    })
                ),
                "{method}: {receiver}"
            );
        }
        check(&format!(
            "let d=new Date(NaN);Object.setPrototypeOf(d,null);Object.freeze(d);Number.isNaN(Date.prototype.{method}.call(d))"
        ));
    }
}

#[test]
fn invalid_local_setters_convert_present_fields_and_preserve_hook_revival() {
    for (method, arity) in SETTERS {
        check(&format!(
            "let d=new Date(NaN);Number.isNaN(d.{method}()) && Number.isNaN(d.getTime())"
        ));
        let args=(0..arity).map(|i|format!("{{[Symbol.toPrimitive](h){{if(h!=='number')throw 1;trace+='{i}';d.setTime({i}+7);return NaN;}}}}"))
            .collect::<Vec<_>>().join(",");
        let expected = (0..arity).map(|i| i.to_string()).collect::<String>();
        check(&format!(
            "let trace='',count=0,d=new Date(NaN);Number.isNaN(d.{method}({args},count++,{{valueOf(){{throw 7;}}}})) && trace==='{expected}' && d.getTime()==={} && count===1",
            arity + 6
        ));
        for failure in 0..arity {
            let args = (0..arity)
                .map(|i| {
                    format!(
                        "{{valueOf(){{trace+='{i}';d.setTime(77);{} }}}}",
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
                "let trace='',caught=false,d=new Date(NaN);try{{d.{method}({args});}}catch(e){{caught=e===7;}}caught && trace==='{expected}' && d.getTime()===77"
            ));
        }
        for value in ["1n", "Symbol()"] {
            assert!(matches!(
                Realm::default().eval(&format!("new Date(NaN).{method}({value})")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        }
        for receiver in [
            "null",
            "{}",
            "Date.prototype",
            "Object.create(new Date(NaN))",
        ] {
            assert!(matches!(
                Realm::default().eval(&format!(
                    "Date.prototype.{method}.call({receiver},{{valueOf(){{throw 7;}}}})"
                )),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        }
        check(&format!(
            "let d=new Date(NaN);Object.setPrototypeOf(d,null);Object.freeze(d);Number.isNaN(Date.prototype.{method}.call(d,1)) && Number.isNaN(Date.prototype.getTime.call(d))"
        ));
    }
}

#[test]
fn recursive_constructor_and_invalid_setter_coercion_restore_state_after_host_abort() {
    let mut realm = Realm::default();
    realm
        .eval("var flag=0;var x={valueOf(){return new Date(x,0);}};")
        .unwrap();
    assert!(matches!(
        realm.eval("try{new Date(x,0);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(
        realm.eval("Number.isNaN(new Date(NaN,0).getTime())"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(realm.eval("new Date(7).getTime()"), Ok(Value::Number(7.0)));
    for (method, _) in SETTERS {
        let mut realm = Realm::default();
        realm
            .eval(&format!(
                "var flag=0;var d=new Date(NaN),x={{valueOf(){{return d.{method}(x);}}}};"
            ))
            .unwrap();
        assert!(matches!(
            realm.eval(&format!(
                "try{{d.{method}(x);}}catch{{flag=1;}}finally{{flag=2;}}"
            )),
            Err(Error::Limit { .. })
        ));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
        assert_eq!(
            realm.eval("Number.isNaN(d.getTime())"),
            Ok(Value::Boolean(true))
        );
        assert_eq!(
            realm.eval(&format!("Number.isNaN(d.{method}(1))")),
            Ok(Value::Boolean(true))
        );
    }
}
