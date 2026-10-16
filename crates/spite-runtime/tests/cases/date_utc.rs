//! UTC numeric construction, coercion order, and calendar/Number boundaries.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn defaults_distinguish_absence_and_undefined_and_adjust_only_input_short_years() {
    check(
        "Number.isNaN(Date.UTC()) && Number.isNaN(Date.UTC(undefined)) && Date.UTC(1970)===0 && Date.UTC(1970,0)===0 && Date.UTC(70)===0 && Date.UTC(99.9)===915148800000 && Date.UTC(100)===-59011459200000 && Date.UTC(-1)===-62198755200000",
    );
    for year in ["0", "-0", "0.9", "-0.9", "null", "false"] {
        check(&format!("Date.UTC({year})===-2208988800000"));
    }
    for index in 1..7 {
        let mut args = ["1970", "0", "1", "0", "0", "0", "0"];
        args[index] = "undefined";
        check(&format!("Number.isNaN(Date.UTC({}))", args.join(",")));
    }
    check("Date.UTC.call({},1970)===0 && Date.UTC.bind(null,1970)()===0");
}

#[test]
fn fields_truncate_roll_over_and_clip_only_the_final_combined_timestamp() {
    check(
        "Date.UTC(1970.9,0.9,1.9,0.9,0.9,0.9,0.9)===0 && Date.UTC(-1970.9,-0.9,-0.9,-0.9,-0.9,-0.9,-0.9)===-124334438400000",
    );
    check(
        "Date.UTC(2000,1,29,12,34,56,789)===951827696789 && Date.UTC(1900,1,29)===Date.parse('1900-03-01') && Date.UTC(2000,-1,1)===Date.parse('1999-12-01') && Date.UTC(2000,12,1)===Date.parse('2001-01-01')",
    );
    check(
        "Date.UTC(-271821,3,20)===-8640000000000000 && Number.isNaN(Date.UTC(-271821,3,19,23,59,59,999)) && Date.UTC(275760,8,13)===8640000000000000 && Number.isNaN(Date.UTC(275760,8,13,0,0,0,1))",
    );
    // These original Test262 vectors observe floating evaluation order.
    check(
        "Date.UTC(1970,0,1,80063993375,29,1,-288230376151711740)===29312 && Date.UTC(1970,0,213503982336,0,0,0,-18446744073709552000)===34447360",
    );
    check("1/Date.UTC(1970,0,1,0,0,0,-0)===Infinity");
}

#[test]
fn all_present_fields_coerce_once_in_order_even_after_nan_and_extras_are_ignored() {
    let args=(0..7).map(|i| format!("{{[Symbol.toPrimitive](hint){{if(hint!=='number')throw 1;trace+='{i}';return {};}}}}",if i==0{"NaN"}else{"0"})).collect::<Vec<_>>().join(",");
    check(&format!(
        "let trace='';Number.isNaN(Date.UTC({args})) && trace==='0123456'"
    ));
    check(
        "let trace='',year={valueOf(){trace+='v';return {};},toString(){trace+='s';return '1970';}}; Date.UTC(year)===0 && trace==='vs'",
    );
    check("let count=0; Date.UTC(1970,0,1,0,0,0,0,count++,{valueOf(){throw 7;}})===0 && count===1");
}

#[test]
fn abrupt_conversion_stops_at_the_failing_field_and_nonfinite_values_still_convert_later_fields() {
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
        let trace = (0..=failure).map(|i| i.to_string()).collect::<String>();
        check(&format!(
            "let trace='',caught=false;try{{Date.UTC({args});}}catch(e){{caught=e===7;}}caught && trace==='{trace}'"
        ));
    }
    for value in ["1n", "Symbol()"] {
        assert!(matches!(
            Realm::default().eval(&format!("Date.UTC(NaN,{value})")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    for invalid in ["NaN", "Infinity", "-Infinity"] {
        for index in 0..7 {
            let mut args = ["1970", "0", "1", "0", "0", "0", "0"];
            args[index] = invalid;
            check(&format!("Number.isNaN(Date.UTC({}))", args.join(",")));
        }
    }
}

#[test]
fn enormous_components_can_cancel_to_valid_times_without_arbitrary_input_caps() {
    check(
        "Date.UTC(-(2**54),216172782113783840,1)===Date.parse('0000-09-01') && Date.UTC(2**54+4,-216172782113783840,1)===Date.parse('0000-05-01')",
    );
    check(
        "Date.UTC(-Number.MAX_VALUE/12,Number.MAX_VALUE,1)===Date.parse('0000-09-01') && Date.UTC(Number.MAX_VALUE/12,-Number.MAX_VALUE,1)===Date.parse('0000-05-01')",
    );
    check(
        "Date.UTC(5000000000,0,1-1826211780472)===0 && Date.UTC(1e200,0,-3.652425e202)===-86400000 && Number.isNaN(Date.UTC(1e100,0,-3.652425e102)) && Number.isNaN(Date.UTC(Number.MAX_VALUE,0,1))",
    );
}

#[test]
fn recursive_year_coercion_uses_normal_stack_guards_and_restores_state() {
    let mut realm = Realm::default();
    realm
        .eval("var flag=0;var arg={valueOf(){return Date.UTC(arg);}}")
        .unwrap();
    assert!(matches!(
        realm.eval("try{Date.UTC(arg);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(realm.eval("Date.UTC(1970)"), Ok(Value::Number(0.0)));
}
