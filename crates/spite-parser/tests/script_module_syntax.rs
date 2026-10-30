//! Script goal restrictions for static module declarations and ImportMeta (16.1).

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{
    EvalContext, parse_dynamic_async_function, parse_dynamic_async_generator_function,
    parse_dynamic_function, parse_dynamic_generator_function, parse_eval_utf16, parse_module,
    parse_module_utf16, parse_script, parse_script_utf16,
};

#[test]
fn static_import_export_and_import_meta_are_syntax_errors_in_scripts() {
    for body in [
        "import 'source'",
        "import value from 'source'",
        "import {value} from 'source'",
        "export {}",
        "export default 1",
        "export let value=1",
        "import.meta",
        "let x=import.meta",
    ] {
        for source in [
            body.to_owned(),
            format!("function f(){{{body}}}"),
            format!("{{{body}}}"),
            format!("async function f(){{{body}}}"),
            format!("function*f(){{{body}}}"),
        ] {
            let error = parse_script(&source).unwrap_err();
            assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}: {error:?}");
        }
    }
}

#[test]
fn dynamic_function_bodies_keep_the_script_goal_for_module_only_forms() {
    for body in ["import 'source'", "export {}", "return import.meta"] {
        for parse in [
            parse_dynamic_function,
            parse_dynamic_async_function,
            parse_dynamic_generator_function,
        ] {
            assert_eq!(
                parse("", body).unwrap_err().kind,
                DiagnosticKind::Syntax,
                "{body}"
            );
        }
    }
}

#[test]
fn valid_dynamic_import_and_keyword_property_names_parse() {
    for source in [
        "import('source')",
        "let x=import('source')",
        "function f(){return import('source')}",
        "async function f(){return await import('source')}",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    for source in [
        "({import:1,export:2}).import",
        "({import(){return 1},export(){return 2}})",
        "class C{import(){}export(){}}",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}

#[test]
fn forbidden_statement_continuations_are_syntax_errors_in_every_source_goal() {
    // Rules of Automatic Semicolon Insertion: a same-line continuation that
    // cannot belong to this statement does not permit an inserted semicolon.
    let sources = [
        "debugger(0)",
        "debugger[0]",
        "debugger.value",
        "debugger?.value",
        "while(false){break(0)}",
        "while(false){continue[0]}",
        "label:while(false){break label.value}",
        "var x(0)",
        "let x[0]",
    ];
    let errors: Vec<_> = sources
        .iter()
        .map(|source| parse_script(source).unwrap_err())
        .collect();
    for (source, error) in sources.iter().zip(&errors) {
        assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
        assert_eq!(
            parse_script_utf16(&JsString::from(*source)).unwrap_err(),
            *error
        );
        assert_eq!(
            parse_eval_utf16(&JsString::from(*source), EvalContext::default()).unwrap_err(),
            *error
        );
        assert_eq!(
            parse_module(source).unwrap_err().kind,
            DiagnosticKind::Syntax
        );
        for parse in [
            parse_dynamic_function,
            parse_dynamic_async_function,
            parse_dynamic_generator_function,
            parse_dynamic_async_generator_function,
        ] {
            assert_eq!(
                parse("", source).unwrap_err().kind,
                DiagnosticKind::Syntax,
                "{source}"
            );
        }
    }
    insta::assert_debug_snapshot!(errors);
    for source in ["import 'source'(0)", "export {}[0]"] {
        assert_eq!(
            parse_module(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
        assert_eq!(
            parse_module_utf16(&JsString::from(source))
                .unwrap_err()
                .kind,
            DiagnosticKind::Syntax
        );
    }
}

#[test]
fn allowed_asi_and_expression_continuations_remain_valid() {
    for source in [
        "debugger\n(0)",
        "debugger\n[0]",
        "while(false){break\n(0)}",
        "while(false){continue\n[0]}",
        "var x\n(0)",
        "let x\n[0]",
        "function f(){}f\n(0)",
        "'value'\n.length",
        "({x:0})\n?.x",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
        assert!(parse_module(source).is_ok(), "{source}");
    }
    for source in ["import 'source'\n(0)", "export {}\n[0]"] {
        assert!(parse_module(source).is_ok(), "{source}");
    }
}
