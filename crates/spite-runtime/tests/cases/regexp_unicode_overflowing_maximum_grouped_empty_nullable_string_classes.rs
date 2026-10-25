//! Grouped-empty string-class ranges with finite overflowing maxima (22.2.2.3).

use spite_core::JsString;
use spite_runtime::{Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn complete_grouped_overflowing_maximum_proofs_finish_finalizers_and_prove_every_leaf() {
    for inner in [
        r"(?:(?:))|[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]",
        r"([\q{a|b}])|(?:X|(?:)?)",
        r"[\q{a|b}]|(?:\b|())",
        r"[\q{a|b}]|(?:\b|())",
        r"[\q{a|b}]|(?:\b|())",
        r"[\q{a|b}]|(\b)?",
        r"[\q{a|ab}&&a]||(?:(?:))",
        r"\b[\q{a|ab}&&a]|(?:(?:))",
    ] {
        let source = format!(r"({inner}){{2,1000}}");
        assert!(
            crate::common::regexp_oracle::check(
                &mut Realm::default(),
                &format!("/{source}/v.test('a')")
            ),
            "{source}"
        );
    }
    for pattern in [
        r"([^\q{ab}]|(?:(?:)))",
        r"([\q{ab}&&a--a]|(?:(?:)))",
        r"([\q{\d}&&a]|(?:(?:)))",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}{{2,1000}}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}
