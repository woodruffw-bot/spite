//! Lossless source code points in CreateDynamicFunction (11.1, 20.2.1.1.1).

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{
    ast::{ExprKind, Literal, StatementKind},
    parse_dynamic_function, parse_dynamic_function_utf16,
};

fn string(parts: &[&[u16]]) -> JsString {
    JsString::from_code_units(parts.concat())
}

#[test]
fn literal_comments_templates_and_retained_source_preserve_distinct_code_units() {
    let parameters = string(&[
        JsString::from("x = '").code_units(),
        &[0xdfff, 0xfffd],
        JsString::from("'").code_units(),
    ]);
    let body = string(&[
        JsString::from("//").code_units(),
        &[0xd800],
        JsString::from("\nreturn [x, `").code_units(),
        &[0xd800, 13, 10, 0xfffd],
        JsString::from("`, () => '").code_units(),
        &[0xdc00],
        JsString::from("'];").code_units(),
    ]);
    let function = parse_dynamic_function_utf16(&parameters, &body).unwrap();
    let expected = string(&[
        JsString::from("function anonymous(").code_units(),
        parameters.code_units(),
        JsString::from("\n) {\n").code_units(),
        body.code_units(),
        JsString::from("\n}").code_units(),
    ]);
    assert_eq!(function.source.to_js_string(), expected);
    assert!(function.source.as_str().is_none());
    insta::assert_debug_snapshot!(function);
}

#[test]
fn every_lone_surrogate_is_a_literal_code_point_and_an_invalid_identifier() {
    for unit in 0xd800..=0xdfff {
        for escaped in [false, true] {
            let prefix = if escaped { "return '\\" } else { "return '" };
            let body = string(&[
                JsString::from(prefix).code_units(),
                &[unit, 0xfffd],
                JsString::from("';").code_units(),
            ]);
            let function = parse_dynamic_function_utf16(&JsString::default(), &body).unwrap();
            let StatementKind::Return(Some(expression)) = &function.body.statements()[0].kind
            else {
                panic!("return expression");
            };
            assert_eq!(
                expression.kind,
                ExprKind::Literal(Literal::String(JsString::from_code_units(vec![
                    unit, 0xfffd
                ])))
            );
            assert!(function.source.as_str().is_none());
            assert!(
                function
                    .source
                    .to_js_string()
                    .code_units()
                    .windows(2)
                    .any(|pair| pair == [unit, 0xfffd])
            );
        }
        let parameters = JsString::from_code_units(vec![unit]);
        let error = parse_dynamic_function_utf16(&parameters, &JsString::default()).unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Syntax);
        assert_eq!((error.span.start, error.span.end), (0, 3));
    }
}

#[test]
fn invalid_code_points_and_separate_goals_report_exact_syntax_ranges() {
    let cases = [
        (vec![0xd800], JsString::default()),
        (vec![b'a' as u16, 0xdfff], JsString::default()),
        (
            vec![],
            string(&[
                JsString::from("return ").code_units(),
                &[0xdc00],
                JsString::from(";").code_units(),
            ]),
        ),
        (
            vec![],
            string(&[
                JsString::from("return 1").code_units(),
                &[0xd800],
                JsString::from(";").code_units(),
            ]),
        ),
        (
            vec![],
            string(&[
                JsString::from("'use strict';return '").code_units(),
                &[0xd800],
                JsString::from("' + 010;").code_units(),
            ]),
        ),
        (
            string(&[JsString::from("/*").code_units(), &[0xdfff]])
                .code_units()
                .to_vec(),
            JsString::from("*/){"),
        ),
    ];
    let diagnostics: Vec<_> = cases
        .into_iter()
        .map(|(parameters, body)| {
            let error = parse_dynamic_function_utf16(&JsString::from_code_units(parameters), &body)
                .unwrap_err();
            assert_eq!(error.kind, DiagnosticKind::Syntax);
            error
        })
        .collect();
    insta::assert_debug_snapshot!(diagnostics);
}

#[test]
fn valid_pairs_and_nested_scalar_sources_remain_exact_utf8() {
    let body = JsString::from("return '😀�';");
    let utf16 = parse_dynamic_function_utf16(&JsString::default(), &body).unwrap();
    assert_eq!(utf16, parse_dynamic_function("", "return '😀�';").unwrap());
    let body = string(&[
        JsString::from("/*").code_units(),
        &[0xd800],
        JsString::from("*/function inner(){return 7;}/*").code_units(),
        &[0xdfff],
        JsString::from("*/return inner;").code_units(),
    ]);
    let function = parse_dynamic_function_utf16(&JsString::default(), &body).unwrap();
    assert_eq!(
        function.body.function_declarations()[0].source.as_str(),
        Some("function inner(){return 7;}")
    );
}
