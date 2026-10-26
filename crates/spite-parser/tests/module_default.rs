//! Default export declarations, synthetic bindings and expression grammar (16.2.3).

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{
    ast::{ExportKind, ExprKind, Literal, StatementKind},
    parse_module, parse_module_utf16, parse_module_with_source_limit,
};

#[test]
fn default_records_distinguish_named_declarations_from_synthetic_bindings() {
    let mut records = Vec::new();
    for source in [
        "export default function named(){}",
        "export default function(){}",
        "export default function* named(){}",
        "export default function*(){}",
        "export default async function named(){}",
        "export default async function(){}",
        "export default class Named{}",
        "export default class{}",
        "export default (function internal(){})",
        "export default (class Internal{})",
        "export default 42;",
    ] {
        let module = parse_module(source).unwrap();
        let declaration = &module.exports()[0];
        let ExportKind::Default { local, keyword } = &declaration.kind else {
            panic!("default")
        };
        assert_eq!(&source[keyword.start..keyword.end], "default");
        assert_eq!(
            &source[declaration.span.start..declaration.span.end],
            source
        );
        let local = local.as_ref().map(|name| {
            assert!(!name.quoted);
            assert_eq!(
                name.value,
                JsString::from(&source[name.span.start..name.span.end])
            );
            name.value.clone()
        });
        let node = match &module.statements()[0].kind {
            StatementKind::Function(_) => "function",
            StatementKind::Class(_) => "class",
            StatementKind::Expression(_) => "expression",
            _ => panic!("default body"),
        };
        records.push((source, local, node));
    }
    insta::assert_debug_snapshot!(records);
}

#[test]
fn default_assignment_expressions_support_await_arrows_objects_and_asi() {
    for source in [
        "export default 1",
        "export default (1,2)",
        "export default {a:1}",
        "export default x=1",
        "export default x=>x",
        "export default async x=>await x",
        "export default await 1",
        "export default import.meta",
        "export default import('source')",
        "export default class extends Base{}",
        "export default (class extends Base{})",
        "export default\n42",
        "export default 1\nconst x=2",
        "export default function(){};",
        "export default class{};",
    ] {
        assert!(
            parse_module(source).is_ok(),
            "{source}: {:?}",
            parse_module(source)
        );
    }
}

#[test]
fn declaration_lookahead_prevents_default_functions_and_classes_becoming_expressions() {
    for source in [
        "export default function(){}(1)",
        "export default class{}(1)",
    ] {
        let module = parse_module(source).unwrap();
        assert_eq!(module.statements().len(), 2, "{source}");
        assert!(!matches!(
            module.statements()[0].kind,
            StatementKind::Expression(_)
        ));
    }
    let module = parse_module("export default async\nfunction f(){}").unwrap();
    assert_eq!(module.statements().len(), 2);
    assert!(matches!(
        module.statements()[0].kind,
        StatementKind::Expression(_)
    ));
    assert!(matches!(
        module.statements()[1].kind,
        StatementKind::Function(_)
    ));
    let module = parse_module("export default (function(){})(1)").unwrap();
    assert_eq!(module.statements().len(), 1);
    assert!(matches!(
        module.statements()[0].kind,
        StatementKind::Expression(_)
    ));
}

#[test]
fn default_export_names_conflict_across_declarations_aliases_and_reexports() {
    for source in [
        "export default 1;export default 2",
        "export default function(){};export default class{}",
        "export default function f(){};export{f as default}",
        "export {x as 'default'} from 'source';export default 1",
        "export default 1;export * as default from 'source'",
        "export * as 'default' from 'source';export default class{}",
    ] {
        let error = parse_module(source).unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
        assert_eq!(error.message, "duplicate exported name", "{source}");
    }
}

#[test]
fn only_named_default_declarations_introduce_source_visible_module_bindings() {
    for source in [
        "export default function f(){};export{f}",
        "export{C};export default class C{}",
        "export default function* g(){};export{g}",
        "export default async function f(){};export{f}",
        "export default (function internal(){});let internal",
        "export default (class Internal{});let Internal",
        "export default function(){};let value",
        "export default class{};let value",
    ] {
        assert!(
            parse_module(source).is_ok(),
            "{source}: {:?}",
            parse_module(source)
        );
    }
    for source in [
        "export default (function internal(){});export{internal}",
        "export default (class Internal{});export{Internal}",
        "import f from 'source';export default function f(){}",
        "export default class C{};var C",
        "let f;export default function f(){}",
        "export default function f(){};function f(){}",
    ] {
        assert_eq!(
            parse_module(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn default_exports_validate_strict_bodies_and_reject_invalid_grammar() {
    for source in [
        "export default",
        "export default 1,2",
        "export default let x=1",
        "export default const x=1",
        "export default var x=1",
        "export default function(eval){}",
        "export default function f(a,a){}",
        "export default function await(){}",
        "export default class await{}",
        "export default function(){with({}){}}",
        "export default function(){await 1}",
        "export default function*(){let yield}",
        "export default class{#x;#x}",
        "{export default 1}",
        "function f(){export default 1}",
    ] {
        assert_eq!(
            parse_module(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    assert_eq!(
        parse_module("export default async function*(){}")
            .unwrap_err()
            .kind,
        DiagnosticKind::Unsupported
    );
}

#[test]
fn default_expression_source_preserves_utf16_and_opted_in_source_limits() {
    let mut units: Vec<_> = "export default '".encode_utf16().collect();
    units.push(0xd800);
    units.extend("';".encode_utf16());
    let module = parse_module_utf16(&JsString::from_code_units(units)).unwrap();
    let StatementKind::Expression(expression) = &module.statements()[0].kind else {
        panic!("expression")
    };
    let ExprKind::Literal(Literal::String(value)) = &expression.kind else {
        panic!("string")
    };
    assert_eq!(value.code_units(), [0xd800]);
    assert_eq!(module.exports()[0].span.end, "export default '".len() + 5);
    let source = "export default function(){}";
    assert!(parse_module_with_source_limit(source, source.len()).is_ok());
    assert_eq!(
        parse_module_with_source_limit(source, source.len() - 1)
            .unwrap_err()
            .kind,
        DiagnosticKind::Limit
    );
}
