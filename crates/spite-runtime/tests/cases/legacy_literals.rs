//! Legacy literal values and strict early errors (ECMA-262 12.9.3–4).

use spite_core::{DiagnosticKind, JsString};
use spite_runtime::{Error, Realm, Value};

#[test]
fn legacy_literals_execute_only_in_non_strict_code() {
    for (source, expected) in [
        ("010 + 08", 16.0),
        ("077 + 099", 162.0),
        ("08.5e1", 85.0),
        ("-00", -0.0),
    ] {
        let value = Realm::default().eval(source).unwrap();
        assert!(value.same_value(&Value::Number(expected)), "{source}");
    }
    for (source, expected) in [
        (r"'\101\102'", "AB"),
        (r"`${'\377'}:${'\8'}`", "ÿ:8"),
        (r"'\08'", "\08"),
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Ok(Value::String(JsString::from(expected)))
        );
    }
}

#[test]
fn strict_lexical_errors_precede_all_execution() {
    let mut realm = Realm::default();
    realm.eval("let flag = 0").unwrap();
    for source in [
        r"'\1'; 'use strict'; flag = 1;",
        "'use strict'; flag = 1; 010",
        r"'use strict'; flag = 1; '\8'",
    ] {
        assert!(
            matches!(realm.eval(source), Err(Error::Parse(error)) if error.kind == DiagnosticKind::Syntax)
        );
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
}
