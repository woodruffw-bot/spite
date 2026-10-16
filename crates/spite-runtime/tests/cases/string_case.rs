//! Full Unicode default casing, original-text sigma contexts, and host locale.

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
fn full_mappings_expand_without_normalization_and_cover_supplementary_planes() {
    check("'aBcXYZ09'.toLowerCase()==='abcxyz09' && 'aBcXYZ09'.toUpperCase()==='ABCXYZ09'");
    check("'ßﬃŉΐև'.toUpperCase()==='SSFFIʼNΪ́ԵՒ' && 'İẞK'.toLowerCase()==='i̇ßk'");
    check("'𐐀𐐨😀'.toLowerCase()==='𐐨𐐨😀' && '𐐀𐐨😀'.toUpperCase()==='𐐀𐐀😀'");
    // U+1DF95 was added to the project's pinned Unicode 18 data.
    check("'\\u{1DF95}'.toUpperCase()==='SS'");
    check("'ÉE\\u0301'.toLowerCase()==='ée\\u0301' && 'ée\\u0301'.toUpperCase()==='ÉE\\u0301'");
    check(
        "'\\0\\uFFFF\\u{10FFFF}'.toUpperCase()==='\\0\\uFFFF\\u{10FFFF}' && ''.toLowerCase()==='' && ''.toUpperCase()===''",
    );
}

#[test]
fn sigma_context_uses_original_cased_and_case_ignorable_properties() {
    for (input, lower) in [
        ("Σ", "σ"),
        ("ΣΣ", "σς"),
        ("ΟΣ", "ος"),
        ("ΟΣΑ", "οσα"),
        ("AΣ B", "aς b"),
        ("AΣ+B", "aς+b"),
        ("AΣ'B", "aσ'b"),
        ("AΣ'", "aς'"),
        ("AΣ\\u0301", "aς\\u0301"),
        ("AΣ\\u0301B", "aσ\\u0301b"),
        ("A\\u0301Σ", "a\\u0301ς"),
        ("\\u0345Σ", "\\u0345σ"),
        ("AΣ\\u0345", "aς\\u0345"),
        ("AΣ\\u0345B", "aσ\\u0345b"),
        ("AΣ\\u02B0", "aς\\u02B0"),
        ("AΣ\\u02B0B", "aσ\\u02B0b"),
        ("ΣAΣ", "σaς"),
    ] {
        check(&format!("\"{input}\".toLowerCase()===\"{lower}\""));
    }
    check("'AΣ'.toLowerCase().toUpperCase()==='AΣ' && 'ςσ'.toUpperCase()==='ΣΣ'");
}

#[test]
fn lone_surrogates_are_preserved_and_form_context_boundaries() {
    check(
        "'A\\uD800B\\uDC00'.toLowerCase()==='a\\uD800b\\uDC00' && 'a\\uD800b\\uDC00'.toUpperCase()==='A\\uD800B\\uDC00'",
    );
    check(
        "'AΣ\\uD800B'.toLowerCase()==='aς\\uD800b' && 'A\\uD800Σ'.toLowerCase()==='a\\uD800σ' && 'AΣ\\uDC00B'.toLowerCase()==='aς\\uDC00b'",
    );
    check("'\\uD800\\uD800\\uDC00\\uDC00'.toLowerCase()==='\\uD800\\uD800\\uDC00\\uDC00'");
}

#[test]
fn generic_receivers_convert_once_with_string_hint_and_propagate_abrupt_results() {
    for method in [
        "toLowerCase",
        "toUpperCase",
        "toLocaleLowerCase",
        "toLocaleUpperCase",
    ] {
        check(&format!(
            "String.prototype.{method}.call(12)==='12' && String.prototype.{method}.call(true)==='{}' && String.prototype.{method}.call(7n)==='7'",
            if method.contains("Upper") {
                "TRUE"
            } else {
                "true"
            }
        ));
        let expected = if method.contains("Upper") { "SS" } else { "ß" };
        check(&format!(
            "let log='',o={{[Symbol.toPrimitive](hint){{log+=hint;return 'ß';}},toString(){{throw 7;}}}};String.prototype.{method}.call(o)==='{expected}' && log==='string'"
        ));
        check(&format!(
            "let marker={{}},caught=false;try{{String.prototype.{method}.call({{toString(){{throw marker;}},valueOf(){{throw 7;}}}});}}catch(e){{caught=e===marker;}}caught"
        ));
        for receiver in [
            "undefined",
            "null",
            "Symbol()",
            "Object(Symbol())",
            "{__proto__:null}",
        ] {
            assert!(
                matches!(
                    Realm::default().eval(&format!("String.prototype.{method}.call({receiver})")),
                    Err(Error::Exception {
                        kind: ExceptionKind::TypeError,
                        ..
                    })
                ),
                "{method}: {receiver}"
            );
        }
    }
    check(
        "new String('ABC').toLowerCase()==='abc' && String.prototype.toUpperCase.call({toString(){return {};},valueOf(){return 'ß';}})==='SS'",
    );
}

#[test]
fn locale_methods_use_fixed_neutral_casing_and_ignore_reserved_arguments() {
    check("'Iİiı'.toLocaleLowerCase('tr')==='ii̇iı' && 'Iİiı'.toLocaleUpperCase('tr')==='IİII'");
    check("'I\\u0301'.toLocaleLowerCase('lt')==='i\\u0301' && 'ΟΣ'.toLocaleLowerCase()==='ος'");
    check(
        "let log='',o={toString(){throw 7;},[Symbol.iterator](){throw 8;}};'ß'.toLocaleUpperCase((log+='a',o),(log+='b',o))==='SS' && 'ABC'.toLocaleLowerCase(o,o)==='abc' && log==='ab'",
    );
    check(
        "String.prototype.toLowerCase!==String.prototype.toLocaleLowerCase && String.prototype.toUpperCase!==String.prototype.toLocaleUpperCase",
    );
}

#[test]
fn method_descriptors_and_retained_intrinsics_are_standard() {
    let mut realm = Realm::default();
    let mut retained = Vec::new();
    for method in [
        "toLowerCase",
        "toUpperCase",
        "toLocaleLowerCase",
        "toLocaleUpperCase",
    ] {
        let Value::Object(handle) = realm.eval(&format!("String.prototype.{method}")).unwrap()
        else {
            panic!("method")
        };
        let object = realm.inspect_object(&handle).unwrap();
        assert!(object.is_callable() && !object.is_constructor());
        assert!(object.own_property(&JsString::from("prototype")).is_none());
        for (name, value) in [
            ("name", Value::String(method.into())),
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
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(String.prototype,'{method}');d.writable && !d.enumerable && d.configurable"
        ));
        assert!(matches!(
            realm.eval(&format!("new String.prototype.{method}()")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        assert_eq!(
            realm.eval(&format!(
                "delete String.prototype.{method};'x'.{method}===undefined"
            )),
            Ok(Value::Boolean(true))
        );
        retained.push(handle);
    }
    realm.collect(usize::MAX).unwrap();
    assert!(
        retained
            .iter()
            .all(|handle| realm.inspect_object(handle).is_ok())
    );
}

#[test]
fn large_default_expansions_work_and_opted_in_failures_skip_language_cleanup() {
    check(
        "'ß'.repeat(10000).toUpperCase().length===20000 && ('AΣ'+'\\u0301'.repeat(10000)).toLowerCase().slice(0,2)==='aς'",
    );
    let mut realm = Realm::new(Limits {
        max_string_units: Some(1),
        ..Limits::default()
    });
    realm.eval("let flag=0;").unwrap();
    assert!(matches!(
        realm.eval("try{'ß'.toUpperCase();}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0;").unwrap();
    assert!(matches!(realm.eval("try{String.prototype.toLowerCase.call({toString(){while(true){}}});}catch{flag=1;}finally{flag=2;}"), Err(Error::Limit {..})));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::default();
    realm.eval("let flag=0;").unwrap();
    assert!(matches!(realm.eval("try{String.prototype.toUpperCase.call({toString(){Function('function* gap(){}');}});}catch{flag=1;}finally{flag=2;}"), Err(Error::Unsupported {..})));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
