//! Generic String Unicode well-formedness and exact replacement semantics.

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
fn valid_pairs_and_non_surrogate_code_units_remain_unchanged() {
    for source in [
        "''",
        "'plain ASCII'",
        "'\u{0000}\u{d7ff}\u{e000}\u{ffff}'",
        "'\u{10000}\u{10ffff}'",
        "'A💩B'",
    ] {
        check(&format!(
            "let s={source};s.isWellFormed() && s.toWellFormed()===s"
        ));
    }
}

#[test]
fn replacement_is_per_unpaired_unit_without_losing_adjacent_pairs() {
    for (source, expected) in [
        ("'\\uD800'", vec![0xfffd]),
        ("'\\uDBFF'", vec![0xfffd]),
        ("'\\uDC00'", vec![0xfffd]),
        ("'\\uDFFF'", vec![0xfffd]),
        ("'\\uD800x'", vec![0xfffd, 0x78]),
        ("'x\\uDC00'", vec![0x78, 0xfffd]),
        ("'\\uDC00\\uD800'", vec![0xfffd, 0xfffd]),
        ("'\\uD800\\uDBFF\\uDC00'", vec![0xfffd, 0xdbff, 0xdc00]),
        ("'\\uDC00\\uD800\\uDFFF'", vec![0xfffd, 0xd800, 0xdfff]),
        ("'\\uD800\\uDC00\\uDFFF'", vec![0xd800, 0xdc00, 0xfffd]),
        ("'\\uD800\\uD800'", vec![0xfffd, 0xfffd]),
        ("'\\uDFFF\\uDFFF'", vec![0xfffd, 0xfffd]),
    ] {
        let mut realm = Realm::default();
        realm.eval(&format!("let s={source}")).unwrap();
        assert_eq!(realm.eval("s.isWellFormed()"), Ok(Value::Boolean(false)));
        assert_eq!(
            realm.eval("s.toWellFormed()"),
            Ok(Value::String(JsString::from_code_units(expected))),
            "{source}"
        );
        assert_eq!(
            realm.eval("let t=s.toWellFormed();t.isWellFormed() && t.length===s.length && t.toWellFormed()===t && !s.isWellFormed()"),
            Ok(Value::Boolean(true))
        );
    }
}

#[test]
fn generic_receivers_use_string_hint_and_ignore_arguments_after_evaluation() {
    for name in ["isWellFormed", "toWellFormed"] {
        let expected = if name == "isWellFormed" {
            "false"
        } else {
            "'�'"
        };
        check(&format!(
            "let log='';let r={{toString:()=>{{log+='s';return '\\uD800';}},valueOf:()=>{{throw 1;}}}},ignored={{toString:()=>{{throw 2;}},valueOf:()=>{{throw 3;}}}};String.prototype.{name}.call(r,(log+='a',ignored))==={expected} && log==='as'"
        ));
        assert_eq!(
            Realm::default().eval(&format!(
                "String.prototype.{name}.call({{toString:()=>{{throw 7;}}}})"
            )),
            Err(Error::Thrown(Value::Number(7.0)))
        );
        for nullish in ["null", "undefined"] {
            assert!(matches!(
                Realm::default().eval(&format!("String.prototype.{name}.call({nullish})")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        }
        assert!(matches!(
            Realm::default().eval(&format!(
                "String.prototype.{name}.call({{toString:()=>({{}}),valueOf:()=>({{}})}})"
            )),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    check(
        "String.prototype.isWellFormed.call(123n) && String.prototype.toWellFormed.call(true)==='true' && String.prototype.toWellFormed.call({toString:()=>({}),valueOf:()=>123})==='123'",
    );
    check(
        "!new String('\\uD800').isWellFormed() && new String('\\uD800').toWellFormed()==='�' && String.prototype.isWellFormed() && String.prototype.toWellFormed()===''",
    );
}

#[test]
fn standard_method_metadata_and_intrinsic_roots_are_preserved() {
    for name in ["isWellFormed", "toWellFormed"] {
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(String.prototype,'{name}');d.writable && !d.enumerable && d.configurable && d.value.name==='{name}' && d.value.length===0 && !Object.hasOwn(d.value,'prototype')"
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
        realm.eval("'valid'.isWellFormed() && '\\uD800'.toWellFormed()==='�'"),
        Ok(Value::Boolean(true))
    );
}
