//! Repetition and padding: UTF-16, conversion order, and bounded generated output.

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn repetition_uses_integer_counts_and_preserves_code_units() {
    check(
        "'ab'.repeat(3)==='ababab' && 'ab'.repeat(2.9)==='abab' && 'ab'.repeat('2')==='abab' && 'ab'.repeat(true)==='ab'",
    );
    for count in ["undefined", "NaN", "null", "false", "-0", "-0.9", "0"] {
        check(&format!("'ab'.repeat({count})===''"));
    }
    check("''.repeat(1e308)==='' && ''.repeat(9007199254740991)==='' && 'ab'.repeat()===''");
    check(
        "'💩'.repeat(2)==='💩💩' && '\\uD800'.repeat(2)==='\\uD800\\uD800' && '\\uDC00\\uD800'.repeat(2)==='\\uDC00\\uD800\\uDC00\\uD800'",
    );
    check("String.prototype.repeat.call(12,2)==='1212' && new String('a').repeat(2)==='aa'");
    for count in ["-1", "-1.9", "-Infinity", "Infinity"] {
        for receiver in ["''", "'x'"] {
            assert!(matches!(
                Realm::default().eval(&format!("{receiver}.repeat({count})")),
                Err(Error::Exception {
                    kind: ExceptionKind::RangeError,
                    ..
                })
            ));
        }
    }
    assert!(matches!(
        Realm::default().eval("''.repeat(0n)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn repetition_converts_receiver_before_count() {
    check(
        "let log='';let r={toString:()=>{log+='r';return 'ab';}},n={valueOf:()=>{log+='n';return 2;},toString:()=>{throw 1;}};String.prototype.repeat.call(r,n)==='abab' && log==='rn'",
    );
    assert_eq!(
        Realm::default().eval(
            "String.prototype.repeat.call({toString:()=>{throw 7;}},{valueOf:()=>{throw 8;}})"
        ),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    assert_eq!(
        Realm::default().eval("''.repeat({valueOf:()=>{throw 8;}})"),
        Err(Error::Thrown(Value::Number(8.0)))
    );
}

#[test]
fn padding_uses_truncated_repeated_filler_and_default_space() {
    check(
        "'abc'.padStart(7)==='    abc' && 'abc'.padEnd(7,undefined)==='abc    ' && 'abc'.padStart(8,'xy')==='xyxyxabc' && 'abc'.padEnd(8,'xy')==='abcxyxyx'",
    );
    check(
        "'abc'.padStart(6,null)==='nulabc' && 'abc'.padEnd(5,false)==='abcfa' && 'abc'.padStart(5,12n)==='12abc'",
    );
    check("'x'.padStart('3.9','a')==='aax' && 'x'.padEnd(3.9,'a')==='xaa'");
    check(
        "'x'.padStart(2,'💩')==='\\uD83Dx' && 'x'.padEnd(4,'💩')==='x💩\\uD83D' && 'x'.padStart(3,'\\uDC00')==='\\uDC00\\uDC00x'",
    );
    check(
        "''.padStart(3,'ab')==='aba' && ''.padEnd(3,'ab')==='aba' && String.prototype.padStart.call(12,4,0)==='0012' && new String('x').padEnd(2)==='x '",
    );
}

#[test]
fn padding_short_circuits_before_filler_coercion_and_after_empty_filler() {
    for name in ["padStart", "padEnd"] {
        for max in [
            "undefined",
            "NaN",
            "null",
            "false",
            "-Infinity",
            "-1",
            "0",
            "2.9",
            "3",
        ] {
            check(&format!(
                "'abc'.{name}({max},{{toString:()=>{{throw 1;}}}})==='abc'"
            ));
        }
        check(&format!(
            "'abc'.{name}(Infinity,'')==='abc' && ''.{name}(1e308,'')===''"
        ));
        check(&format!(
            "let n=0;'abc'.{name}(Infinity,{{toString:()=>{{n++;return '';}}}})==='abc' && n===1"
        ));
        // Argument expressions run even when filler coercion is skipped.
        check(&format!(
            "let n=0;'abc'.{name}(1,(n++,{{toString:()=>{{throw 1;}}}}))==='abc' && n===1"
        ));
        assert!(matches!(
            Realm::default().eval(&format!("'abc'.{name}(1n,'')")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
}

#[test]
fn padding_observes_receiver_then_length_then_filler() {
    for (name, expected) in [("padStart", "xxa"), ("padEnd", "axx")] {
        check(&format!(
            "let log='';let r={{toString:()=>{{log+='r';return 'a';}}}},n={{valueOf:()=>{{log+='n';return 3;}}}},f={{toString:()=>{{log+='f';return 'x';}},valueOf:()=>{{throw 1;}}}};String.prototype.{name}.call(r,n,f)==='{expected}' && log==='rnf'"
        ));
        assert_eq!(Realm::default().eval(&format!("String.prototype.{name}.call({{toString:()=>{{throw 7;}}}},{{valueOf:()=>{{throw 8;}}}},{{toString:()=>{{throw 9;}}}})")),Err(Error::Thrown(Value::Number(7.0))));
        assert_eq!(
            Realm::default().eval(&format!(
                "'a'.{name}({{valueOf:()=>{{throw 8;}}}},{{toString:()=>{{throw 9;}}}})"
            )),
            Err(Error::Thrown(Value::Number(8.0)))
        );
        assert_eq!(
            Realm::default().eval(&format!(
                "'a'.{name}(Infinity,{{toString:()=>{{throw 9;}}}})"
            )),
            Err(Error::Thrown(Value::Number(9.0)))
        );
    }
}

#[test]
fn methods_reject_nullish_receivers_and_have_standard_metadata() {
    for name in ["repeat", "padStart", "padEnd"] {
        for receiver in ["null", "undefined"] {
            assert!(matches!(
                Realm::default().eval(&format!(
                    "String.prototype.{name}.call({receiver},{{valueOf:()=>{{throw 1;}}}})"
                )),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        }
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(String.prototype,'{name}');d.writable && !d.enumerable && d.configurable && d.value.name==='{name}' && d.value.length===1 && !Object.hasOwn(d.value,'prototype')"
        ));
        assert!(matches!(
            Realm::default().eval(&format!("new String.prototype.{name}()")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    let mut realm = Realm::default();
    realm.eval("delete globalThis.String").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("'x'.repeat(2).padStart(3).padEnd(4)===' xx '"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn generated_output_limits_are_host_aborts_and_do_not_run_language_handlers() {
    let mut realm = Realm::new(Limits {
        max_string_units: Some(64),
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    for expression in [
        "'ab'.repeat(33)",
        "'x'.repeat(1e308)",
        "'x'.padStart(65)",
        "'x'.padEnd(Infinity)",
    ] {
        assert!(
            matches!(
                realm.eval(&format!(
                    "try{{{expression};}}catch{{flag=1;}}finally{{flag=2;}}"
                )),
                Err(Error::Limit { .. })
            ),
            "{expression}"
        );
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
    assert_eq!(
        realm.eval("'ab'.repeat(32).length"),
        Ok(Value::Number(64.0))
    );
    assert_eq!(
        realm.eval("try{''.repeat(Infinity);}catch(e){e instanceof RangeError;}"),
        Ok(Value::Boolean(true))
    );
}
