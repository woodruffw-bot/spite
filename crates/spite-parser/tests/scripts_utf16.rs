//! Script grammar from lossless UTF-16 source (11.1, 16.1, 19.2.1.1).

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{parse_script, parse_script_utf16};

#[test]
fn scalar_inputs_share_script_grammar_strictness_and_exact_source() {
    for source in [
        "",
        "/*comment*/",
        "1;var x;",
        "'use strict';let x=7;",
        "'use\\x20strict';with({}){}",
        "function f(){return new.target;}",
        "let f=()=>1;",
        "var [x]=['😀�'];",
    ] {
        assert_eq!(
            parse_script_utf16(&JsString::from(source)),
            parse_script(source)
        );
    }
}

#[test]
fn lone_source_code_points_survive_literals_and_retained_function_ranges() {
    let mut units = JsString::from("let x='").code_units().to_vec();
    units.extend([0xd800, 0xfffd]);
    units.extend_from_slice(JsString::from("';function f(){return `").code_units());
    units.push(0xdfff);
    units.extend_from_slice(JsString::from("`;}").code_units());
    let script = parse_script_utf16(&JsString::from_code_units(units)).unwrap();
    assert!(!script.is_strict());
    assert!(script.function_declarations()[0].source.as_str().is_none());
    insta::assert_debug_snapshot!(script);
}

#[test]
fn top_level_eval_goals_keep_control_and_constructor_early_errors() {
    let mut sources: Vec<_> = [
        "return 7;",
        "break;",
        "continue;",
        "new.target",
        "()=>new.target",
        "super.x",
        "super()",
        "let x;var x;",
        "'use strict';with({}){}",
        "'use strict';var eval;",
    ]
    .into_iter()
    .map(JsString::from)
    .collect();
    sources.push(JsString::from_code_units(vec![0xd800]));
    let diagnostics: Vec<_> = sources
        .into_iter()
        .map(|source| {
            let diagnostic = parse_script_utf16(&source).unwrap_err();
            assert_eq!(diagnostic.kind, DiagnosticKind::Syntax);
            diagnostic
        })
        .collect();
    insta::assert_debug_snapshot!(diagnostics);
    assert_eq!(
        parse_script_utf16(&JsString::from("class C{#field;}"))
            .unwrap_err()
            .kind,
        DiagnosticKind::Unsupported
    );
}

#[test]
fn super_properties_follow_method_and_arrow_contexts_without_crossing_functions() {
    for source in [
        "({m(){return super.x;}})",
        "({m(){return ()=>super.x;}})",
        "({m(x=super.x){}})",
        "({get x(){return super.x;}})",
        "function f(){return {m(){return super.x;}};}",
    ] {
        assert!(
            parse_script_utf16(&JsString::from(source)).is_ok(),
            "{source}"
        );
    }
    for source in [
        "()=>super.x",
        "function f(){return super.x;}",
        "({m(){return function(){return super.x;};}})",
        "({m(){super();}})",
        "({m(){return super;}})",
        "({m(){return super?.x;}})",
    ] {
        assert_eq!(
            parse_script_utf16(&JsString::from(source))
                .unwrap_err()
                .kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}
