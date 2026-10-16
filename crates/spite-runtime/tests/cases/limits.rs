//! Host resource quotas are opt-in and independent of language exceptions.

use spite_core::DiagnosticKind;
use spite_runtime::{Error, Limits, Realm, Value};

#[test]
fn default_realms_accept_programs_beyond_the_former_resource_quotas() {
    let cases = [
        (
            "function f(){for(var i=0;i<11000;i++){}return i;}f()===11000".to_owned(),
            Limits {
                max_heap_entries: Some(10_000),
                ..Limits::default()
            },
        ),
        (
            "let a=[];for(var i=0;i<1100;i++)a.push(i);a.length===1100 && a[1099]===1099"
                .to_owned(),
            Limits {
                max_properties: Some(1024),
                ..Limits::default()
            },
        ),
        (
            "let k='x'.repeat(1048577);k.length===1048577 && Symbol.keyFor(Symbol.for(k))===k"
                .to_owned(),
            Limits {
                max_string_units: Some(1024 * 1024),
                ..Limits::default()
            },
        ),
        (
            "((1n<<65536n)>>65536n)===1n".to_owned(),
            Limits {
                max_bigint_bits: Some(65_536),
                ..Limits::default()
            },
        ),
        (
            "Boolean.apply(null,Array(16385))===false".to_owned(),
            Limits {
                max_arguments: Some(16_384),
                ..Limits::default()
            },
        ),
    ];
    for (source, limits) in cases {
        assert_eq!(
            Realm::default().eval(&source),
            Ok(Value::Boolean(true)),
            "{source}"
        );
        assert!(
            matches!(Realm::new(limits).eval(&source), Err(Error::Limit { .. })),
            "{source}"
        );
    }
}

#[test]
fn source_quota_precedes_parsing_and_realm_initialization_when_enabled() {
    let source = " ".repeat(1024 * 1024 + 1) + "true";
    assert_eq!(Realm::default().eval(&source), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_source_bytes: Some(1024 * 1024),
        max_heap_entries: Some(0),
        ..Limits::default()
    });
    assert!(matches!(realm.eval(&source), Err(Error::Parse(d)) if d.kind == DiagnosticKind::Limit));
    assert_eq!(realm.collect(usize::MAX).unwrap().live, 0);
}
