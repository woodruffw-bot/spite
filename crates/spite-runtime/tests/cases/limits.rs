//! Host resource quotas are opt-in and independent of language exceptions.

use spite_core::DiagnosticKind;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

#[test]
fn default_realms_accept_large_programs_without_resource_quotas() {
    let cases = [
        (
            // A nonempty Block requires a new environment on every iteration.
            "function f(){for(var i=0;i<11000;i++){;}return i;}f()===11000".to_owned(),
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

#[test]
fn string_to_number_scans_consume_evaluation_work() {
    for (tail, expected) in [("1", 1.0), ("x", f64::NAN)] {
        let source = format!("+'{}{tail}'", "0".repeat(4096));
        let script = spite_parser::parse_script(&source).unwrap();
        assert!(
            Realm::default()
                .evaluate(&script)
                .unwrap()
                .same_value(&Value::Number(expected))
        );

        let mut realm = Realm::new(Limits {
            max_steps: Some(512),
            max_bigint_bits: Some(0),
            ..Limits::default()
        });
        assert!(matches!(realm.evaluate(&script), Err(Error::Limit { .. })));
        assert_eq!(realm.eval("+'12'"), Ok(Value::Number(12.0)));
        assert_eq!(realm.eval("let flags=0"), Ok(Value::Undefined));
        let guarded = spite_parser::parse_script(&format!(
            "try{{{source}}}catch{{flags=1}}finally{{flags=2}}"
        ))
        .unwrap();
        assert!(matches!(realm.evaluate(&guarded), Err(Error::Limit { .. })));
        assert_eq!(realm.eval("flags"), Ok(Value::Number(0.0)));
        assert!(
            realm
                .eval("+'-0'")
                .unwrap()
                .same_value(&Value::Number(-0.0))
        );
    }
}

#[test]
fn numeric_comparison_string_scans_consume_evaluation_work() {
    let digits = "0".repeat(4096) + "1";
    for source in [
        format!("'{digits}'==1"),
        format!("1=='{digits}'"),
        format!("'{digits}'==true"),
        format!("true=='{digits}'"),
        format!("'{digits}'<2"),
        format!("0<'{digits}'"),
        format!("'{digits}'>=1"),
        format!("1<='{digits}'"),
    ] {
        let script = spite_parser::parse_script(&source).unwrap();
        assert_eq!(Realm::default().evaluate(&script), Ok(Value::Boolean(true)));
        let mut realm = Realm::new(Limits {
            max_steps: Some(512),
            max_bigint_bits: Some(0),
            ..Limits::default()
        });
        assert!(matches!(realm.evaluate(&script), Err(Error::Limit { .. })));
        assert_eq!(realm.eval("1<'2'"), Ok(Value::Boolean(true)));
    }

    for source in [
        format!("'{digits}'==null"),
        format!("(void 0)=='{digits}'"),
        format!("'{digits}'===1"),
    ] {
        let script = spite_parser::parse_script(&source).unwrap();
        assert_eq!(
            Realm::new(Limits {
                max_steps: Some(512),
                ..Limits::default()
            })
            .evaluate(&script),
            Ok(Value::Boolean(false))
        );
    }

    let digits = "0".repeat(16384) + "1";
    let mut realm = Realm::new(Limits {
        max_steps: Some(8192),
        ..Limits::default()
    });
    assert!(matches!(
        realm.eval(&format!("Symbol()<'{digits}'")),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert!(matches!(
        realm.eval(&format!("'{digits}'<Symbol()")),
        Err(Error::Limit { .. })
    ));
}

#[test]
fn string_pair_comparison_scans_consume_evaluation_work() {
    let left = "0".repeat(4096) + "1";
    let right = "0".repeat(4096) + "2";
    for source in [
        format!("'{left}'==='{left}'"),
        format!("'{left}'=='{left}'"),
        format!("'{left}'<'{right}'"),
        format!("switch('{left}'){{case '{left}':true;break;default:false}}"),
    ] {
        let script = spite_parser::parse_script(&source).unwrap();
        assert_eq!(Realm::default().evaluate(&script), Ok(Value::Boolean(true)));
        let mut realm = Realm::new(Limits {
            max_steps: Some(512),
            ..Limits::default()
        });
        assert!(matches!(realm.evaluate(&script), Err(Error::Limit { .. })));
        assert_eq!(realm.eval("'a'==='a'"), Ok(Value::Boolean(true)));
        assert_eq!(realm.eval("'\\uD800'<'\\uD801'"), Ok(Value::Boolean(true)));
    }

    for (source, expected) in [
        (format!("'{left}'==='x'"), false),
        (format!("'{left}'=='x'"), false),
        (format!("'{left}'>'0'"), true),
    ] {
        let script = spite_parser::parse_script(&source).unwrap();
        assert_eq!(
            Realm::new(Limits {
                max_steps: Some(512),
                ..Limits::default()
            })
            .evaluate(&script),
            Ok(Value::Boolean(expected))
        );
    }
}

#[test]
fn string_construction_copies_consume_evaluation_work() {
    let part = "0".repeat(4096);
    for (source, length) in [
        (format!("'{part}'+'x'"), part.len() + 1),
        (format!("`{part}${{'x'}}`"), part.len() + 1),
        (format!("`{part}`"), part.len()),
    ] {
        let script = spite_parser::parse_script(&source).unwrap();
        let Value::String(value) = Realm::default().evaluate(&script).unwrap() else {
            panic!("String result")
        };
        assert_eq!(value.len(), length);
        let mut realm = Realm::new(Limits {
            max_steps: Some(512),
            ..Limits::default()
        });
        assert!(matches!(realm.evaluate(&script), Err(Error::Limit { .. })));
        assert_eq!(realm.eval("'a'+'b'"), Ok(Value::String("ab".into())));
    }

    let mut realm = Realm::new(Limits {
        max_steps: Some(512),
        ..Limits::default()
    });
    for source in [format!("'{part}'+''"), format!("''+'{part}'")] {
        let Value::String(value) = realm.eval(&source).unwrap() else {
            panic!("String result")
        };
        assert_eq!(value.len(), part.len());
    }
    realm.eval("let marker=0").unwrap();
    assert!(matches!(
        realm.eval(&format!("`{part}${{marker=1}}`")),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));

    let part = "0".repeat(16384);
    let mut realm = Realm::new(Limits {
        max_steps: Some(8192),
        ..Limits::default()
    });
    realm.eval("let marker=0").unwrap();
    assert!(matches!(
        realm.eval(&format!("'{part}'+({{toString(){{marker=1;return 'x'}}}})")),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("marker"), Ok(Value::Number(1.0)));
}
