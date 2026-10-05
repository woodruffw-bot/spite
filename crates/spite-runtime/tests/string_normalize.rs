//! Unicode forms, generic coercion, intrinsic retention, and opted-in limits.

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn all_forms_handle_recursive_mappings_reordering_blocking_and_hangul() {
    check(
        "'e\\u0301'.normalize()==='é' && 'é'.normalize('NFD')==='e\\u0301' && 'é'.normalize(undefined)==='é'",
    );
    check(
        "'Å'.normalize()==='Å' && 'ﬃ①'.normalize('NFKC')==='ffi1' && 'ﬃ①'.normalize('NFD')==='ﬃ①'",
    );
    check(
        "'Ḋ\\u0323'.normalize()==='Ḍ\\u0307' && 'D\\u0307\\u0323'.normalize('NFD')==='D\\u0323\\u0307'",
    );
    check(
        "'A\\u030A\\u0301'.normalize()==='Ǻ' && 'A\\u030B\\u030A'.normalize()==='A\\u030B\\u030A'",
    );
    check("'\\u0344'.normalize()==='\\u0308\\u0301' && 'क़'.normalize()==='क़'");
    check(
        "'각'.normalize()==='각' && '각'.normalize('NFD')==='각' && '가'.normalize()==='가' && '힣'.normalize('NFD')==='힣'",
    );
    check(
        "'𝕬'.normalize('NFKD')==='A' && '\\u{10FFFF}😀'.normalize()==='\\u{10FFFF}😀' && ''.normalize()===''",
    );
}

#[test]
fn lone_surrogates_preserve_identity_and_separate_ordering_sequences() {
    for form in ["NFC", "NFD", "NFKC", "NFKD"] {
        check(&format!(
            "'\\uD800\\uD800\\uDC00\\uDC00'.normalize('{form}')==='\\uD800\\uD800\\uDC00\\uDC00'"
        ));
        check(&format!(
            "'a\\uD800\\u0301\\uDC00'.normalize('{form}')==='a\\uD800\\u0301\\uDC00'"
        ));
    }
    check(
        "'e\\uD800\\u0301'.normalize()==='e\\uD800\\u0301' && '\\u0301\\uD800\\u0323'.normalize('NFD')==='\\u0301\\uD800\\u0323'",
    );
}

#[test]
fn receiver_precedes_form_conversion_and_both_use_the_string_hint_once() {
    check(
        "let log='',s={[Symbol.toPrimitive](hint){log+='s'+hint;return 'é';}},f={[Symbol.toPrimitive](hint){log+='f'+hint;return 'NFD';}};String.prototype.normalize.call(s,f)==='e\\u0301' && log==='sstringfstring'",
    );
    check(
        "let log='',caught=false,s={toString(){log+='s';throw 7;}},f={toString(){log+='f';throw 8;}};try{String.prototype.normalize.call(s,f);}catch(e){caught=e===7;}caught && log==='s'",
    );
    check(
        "let log='',caught=false;try{String.prototype.normalize.call(null,{toString(){log+='f';throw 8;}});}catch(e){caught=e instanceof TypeError;}caught && log===''",
    );
    check(
        "let marker={},caught=false;try{'abc'.normalize({toString(){throw marker;}});}catch(e){caught=e===marker;}caught",
    );
    check(
        "String.prototype.normalize.call(7)==='7' && String.prototype.normalize.call(true)==='true' && String.prototype.normalize.call(7n)==='7' && new String('e\\u0301').normalize()==='é'",
    );
    for receiver in [
        "null",
        "undefined",
        "Symbol()",
        "Object(Symbol())",
        "{__proto__:null}",
    ] {
        assert!(
            matches!(
                Realm::default().eval(&format!("String.prototype.normalize.call({receiver})")),
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
fn invalid_forms_throw_rangeerror_after_conversion_without_case_folding() {
    for form in [
        "null",
        "true",
        "1",
        "1n",
        "''",
        "'nfc'",
        "'NFC '",
        "'NＦC'",
        "'\\uD800'",
        "'NFK'",
        "'NFKCC'",
    ] {
        assert!(
            matches!(
                Realm::default().eval(&format!("'abc'.normalize({form})")),
                Err(Error::Exception {
                    kind: ExceptionKind::RangeError,
                    ..
                })
            ),
            "{form}"
        );
    }
    for form in [
        "Symbol()",
        "Object(Symbol())",
        "{[Symbol.toPrimitive](){return Symbol();}}",
    ] {
        assert!(
            matches!(
                Realm::default().eval(&format!("'abc'.normalize({form})")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{form}"
        );
    }
    check(
        "let Original=RangeError;RangeError=TypeError;let caught=false;try{'x'.normalize('wrong');}catch(e){caught=e instanceof Original && !(e instanceof TypeError);}caught",
    );
    check(
        "let log='',o={toString(){throw 7;}};'e\\u0301'.normalize((log+='a',undefined),(log+='b',o))==='é' && log==='ab'",
    );
}

#[test]
fn metadata_descriptors_deletion_and_retention_are_standard() {
    let mut realm = Realm::default();
    let Value::Object(handle) = realm.eval("String.prototype.normalize").unwrap() else {
        panic!("normalize")
    };
    let object = realm.inspect_object(&handle).unwrap();
    assert!(object.is_callable() && !object.is_constructor());
    assert!(object.own_property(&JsString::from("prototype")).is_none());
    for (name, value) in [
        ("name", Value::String("normalize".into())),
        ("length", Value::Number(0.0)),
    ] {
        let d = object
            .own_property(&JsString::from(name))
            .unwrap()
            .as_data()
            .unwrap();
        assert_eq!(d.value, value);
        assert!(!d.writable && !d.enumerable && d.configurable);
    }
    check(
        "let d=Object.getOwnPropertyDescriptor(String.prototype,'normalize');d.writable && !d.enumerable && d.configurable",
    );
    assert!(matches!(
        realm.eval("new String.prototype.normalize()"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    realm.eval("let saved=String.prototype.normalize;delete String.prototype.normalize;delete globalThis.String;").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert!(realm.inspect_object(&handle).is_ok());
    assert_eq!(
        realm.eval("saved.call('e\\u0301')==='é' && 'x'.normalize===undefined"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn long_default_inputs_work_and_opted_in_failures_bypass_cleanup() {
    check(
        "'é'.repeat(10000).normalize('NFD').length===20000 && ('x'+'\\u0301\\u0323'.repeat(5000)).normalize('NFD')==='x'+'\\u0323'.repeat(5000)+'\\u0301'.repeat(5000)",
    );
    let mut realm = Realm::new(Limits {
        max_string_units: Some(9),
        ..Limits::default()
    });
    realm.eval("let flag=0;").unwrap();
    assert!(matches!(
        realm.eval("try{'\\uFDFA'.normalize('NFKC');}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0;").unwrap();
    assert!(matches!(realm.eval("try{String.prototype.normalize.call({toString(){while(true){}}});}catch{flag=1;}finally{flag=2;}"),Err(Error::Limit {..})));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::default();
    realm.eval("let flag=0;").unwrap();
    assert!(matches!(
        realm.eval("try{'x'.normalize({toString(){Function('class C{}');}});}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
