//! Script goal restrictions for static module declarations and ImportMeta (16.1).

use spite_core::DiagnosticKind;
use spite_parser::{
    parse_dynamic_async_function, parse_dynamic_function, parse_dynamic_generator_function,
    parse_script,
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
