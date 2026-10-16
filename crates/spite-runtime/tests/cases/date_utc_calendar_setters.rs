//! UTC calendar setters preserve captured fields and literal short years.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

const METHODS: [(&str, usize); 2] = [("setUTCMonth", 2), ("setUTCFullYear", 3)];

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn omitted_fields_preserve_captured_components_and_undefined_is_nan() {
    for (method, argument, expected) in [
        ("setUTCMonth", "2", "2000-03-29T12:34:56.789Z"),
        ("setUTCFullYear", "2004", "2004-02-29T12:34:56.789Z"),
    ] {
        check(&format!(
            "let d=new Date('2000-02-29T12:34:56.789Z');d.{method}({argument})===d.getTime() && d.toISOString()==='{expected}'"
        ));
    }
    for (method, arity) in METHODS {
        check(&format!(
            "let d=new Date(0);Number.isNaN(d.{method}()) && Number.isNaN(d.getTime())"
        ));
        for index in 0..arity {
            let mut args = vec!["1"; index + 1];
            args[index] = "undefined";
            check(&format!(
                "let d=new Date(0);Number.isNaN(d.{method}({})) && Number.isNaN(d.getTime())",
                args.join(",")
            ));
        }
    }
}

#[test]
fn fields_truncate_and_roll_over_without_adjusting_literal_short_years() {
    for (original, method, args, expected) in [
        ("2000-01-31", "setUTCMonth", "1", "2000-03-02"),
        ("2000-02-29", "setUTCFullYear", "1900", "1900-03-01"),
        ("2000-02-29", "setUTCFullYear", "0.9", "0000-02-29"),
        ("2000-02-29", "setUTCFullYear", "1.9", "0001-03-01"),
        ("2000-02-29", "setUTCFullYear", "70", "0070-03-01"),
        ("2000-01-01", "setUTCFullYear", "-0.9", "0000-01-01"),
        ("2000-01-01", "setUTCFullYear", "-1.9", "-000001-01-01"),
        ("2000-01-01", "setUTCMonth", "-1.9,0.9", "1999-11-30"),
        ("2000-01-01", "setUTCMonth", "12.9,1.9", "2001-01-01"),
        (
            "2000-01-01",
            "setUTCFullYear",
            "2001.9,13.9,0.9",
            "2002-01-31",
        ),
    ] {
        check(&format!(
            "let d=new Date('{original}T12:34:56.789Z');d.{method}({args})===d.getTime() && d.toISOString()==='{expected}T12:34:56.789Z'"
        ));
    }
    check("let d=new Date(0);1/d.setUTCMonth(-0.9)===Infinity && 1/d.getTime()===Infinity");
}

#[test]
fn captured_fields_survive_mutating_hooks_and_only_full_year_revives_invalid_dates() {
    check(
        "let d=new Date('2000-02-29T12:34:56.789Z');d.setUTCMonth({valueOf(){d.setTime(0);return 2;}})===d.getTime() && d.toISOString()==='2000-03-29T12:34:56.789Z'",
    );
    check(
        "let d=new Date('2000-02-29T12:34:56.789Z');d.setUTCFullYear({valueOf(){d.setTime(NaN);return 2001;}})===d.getTime() && d.toISOString()==='2001-03-01T12:34:56.789Z'",
    );
    check(
        "let d=new Date(NaN),trace='';Number.isNaN(d.setUTCMonth({valueOf(){trace+='m';d.setTime(7);return 0;}},{valueOf(){trace+='d';d.setTime(9);return 1;}})) && trace==='md' && d.getTime()===9",
    );
    check(
        "let d=new Date(NaN);d.setUTCFullYear({valueOf(){d.setTime(Date.parse('2000-07-09T12:34:56.789Z'));return 1;}})===d.getTime() && d.toISOString()==='0001-01-01T00:00:00.000Z'",
    );
    check(
        "let d=new Date(NaN);Number.isNaN(d.setUTCFullYear({valueOf(){d.setTime(9);return NaN;}})) && Number.isNaN(d.getTime())",
    );
    check(
        "let d=new Date(NaN);d.setUTCFullYear(2000,1,29)===951782400000 && d.getTime()===951782400000",
    );
}

#[test]
fn all_present_fields_coerce_once_in_order_even_after_nan_and_extras_are_ignored() {
    for (method, arity) in METHODS {
        let args = (0..arity)
            .map(|i| format!("{{[Symbol.toPrimitive](hint){{if(hint!=='number')throw 1;trace+='{i}';return {};}}}}", if i == 0 { "NaN" } else { "0" }))
            .collect::<Vec<_>>()
            .join(",");
        let expected = (0..arity).map(|i| i.to_string()).collect::<String>();
        for time in ["0", "NaN"] {
            check(&format!(
                "let trace='',d=new Date({time});Number.isNaN(d.{method}({args})) && trace==='{expected}'"
            ));
        }
        let args = if arity == 2 { "0,1" } else { "1970,0,1" };
        check(&format!(
            "let count=0,d=new Date(0);d.{method}({args},count++,{{valueOf(){{throw 7;}}}})===0 && count===1 && d.getTime()===0"
        ));
    }
}

#[test]
fn abrupt_conversion_preserves_hook_mutations_and_stops_later_conversions() {
    for (method, arity) in METHODS {
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
            for time in ["0", "NaN"] {
                check(&format!(
                    "let trace='',caught=false,d=new Date({time});try{{d.{method}({args});}}catch(e){{caught=e===7;}}caught && trace==='{expected}' && d.getTime()===77"
                ));
            }
        }
        for value in ["1n", "Symbol()"] {
            for time in ["0", "NaN"] {
                assert!(matches!(
                    Realm::default().eval(&format!("new Date({time}).{method}(NaN,{value})")),
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
fn brand_checks_precede_conversion_and_frozen_internal_slots_remain_mutable() {
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
                    Realm::default().eval(&format!(
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
        let args = if method == "setUTCMonth" {
            "1,1"
        } else {
            "1970,1,1"
        };
        check(&format!(
            "let d=new Date(0);Object.setPrototypeOf(d,null);Object.freeze(d);Date.prototype.{method}.call(d,{args})===2678400000 && Object.isFrozen(d) && Date.prototype.getTime.call(d)===2678400000"
        ));
    }
}

#[test]
fn final_clipping_allows_outside_intermediates_and_enormous_components_to_cancel() {
    for (time, year, month, day) in [
        ("-8640000000000000", "-271821", "3", "20"),
        ("8640000000000000", "275760", "8", "13"),
    ] {
        check(&format!(
            "let d=new Date({time});d.setUTCMonth({month})==={time} && d.setUTCFullYear({year})==={time} && d.setUTCFullYear({year},{month},{day})==={time}"
        ));
    }
    check(
        "let d=new Date(8640000000000000);Number.isNaN(d.setUTCMonth(8,14)) && Number.isNaN(d.getTime())",
    );
    check(
        "let d=new Date(-8640000000000000);Number.isNaN(d.setUTCFullYear(-271821,3,19)) && Number.isNaN(d.getTime())",
    );
    check(
        "let d=new Date(0);d.setUTCFullYear(-(2**54),216172782113783840,1)===Date.parse('0000-09-01') && d.setUTCFullYear(2**54+4,-216172782113783840,1)===Date.parse('0000-05-01')",
    );
    check(
        "let d=new Date(0);d.setUTCFullYear(-Number.MAX_VALUE/12,Number.MAX_VALUE,1)===Date.parse('0000-09-01') && d.setUTCFullYear(Number.MAX_VALUE/12,-Number.MAX_VALUE,1)===Date.parse('0000-05-01')",
    );
    check(
        "let d=new Date(0);d.setUTCFullYear(5000000000,0,1-1826211780472)===0 && d.setUTCFullYear(1e200,0,-3.652425e202)===-86400000",
    );
    for (method, arity) in METHODS {
        for index in 0..arity {
            for invalid in [
                "NaN",
                "Infinity",
                "-Infinity",
                "Number.MAX_VALUE",
                "-Number.MAX_VALUE",
            ] {
                let mut args = vec!["1"; arity];
                args[index] = invalid;
                check(&format!(
                    "let d=new Date(0);Number.isNaN(d.{method}({})) && Number.isNaN(d.getTime())",
                    args.join(",")
                ));
            }
        }
    }
}

#[test]
fn recursive_coercion_uses_normal_stack_guards_and_restores_call_state() {
    for (method, _) in METHODS {
        let mut realm = Realm::default();
        realm
            .eval(&format!(
                "var flag=0;var d=new Date(7),arg={{valueOf(){{return d.{method}(arg);}}}};"
            ))
            .unwrap();
        assert!(matches!(
            realm.eval(&format!(
                "try{{d.{method}(arg);}}catch{{flag=1;}}finally{{flag=2;}}"
            )),
            Err(Error::Limit { .. })
        ));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
        assert_eq!(realm.eval("d.getTime()"), Ok(Value::Number(7.0)));
        let argument = if method == "setUTCMonth" { "0" } else { "1970" };
        assert_eq!(
            realm.eval(&format!("d.{method}({argument})")),
            Ok(Value::Number(7.0))
        );
    }
}
