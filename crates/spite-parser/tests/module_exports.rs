//! Named/star export records, aliases and Module binding early errors (16.2.3).

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{ast::ExportKind, parse_module, parse_script};

#[test]
fn named_and_namespace_exports_retain_requests_aliases_attributes_and_ranges() {
    let source = "import x from 'first';export {x as local};export {'remote-name' as 'public-name'} from 'second' with {z:'last',a:'first'};export * as ns from 'third';export * from 'fourth'";
    let module = parse_module(source).unwrap();
    assert_eq!(module.exports().len(), 4);
    let ExportKind::Named {
        specifiers,
        request: None,
    } = &module.exports()[0].kind
    else {
        panic!("local export")
    };
    assert_eq!(specifiers[0].name.value, JsString::from("x"));
    assert_eq!(specifiers[0].exported.value, JsString::from("local"));
    assert_eq!(
        &source[module.exports()[0].span.start..module.exports()[0].span.end],
        "export {x as local};"
    );
    insta::assert_debug_snapshot!(module);
}

#[test]
fn all_named_and_star_forms_parse_with_optional_attributes_and_arbitrary_names() {
    for source in [
        "export {}",
        "let x;export {x,}",
        "export {x};let x",
        "import x from 'source';export {x as default}",
        "export {} from 'source'",
        "export * from 'source'",
        "export * as ns from 'source'",
        "export * as 'name-space' from 'source'",
        "export {'remote-name' as 'public-name','' as 'empty',default as named} from 'source'",
        "export {if as null,true as false,await as yield} from 'source'",
        "let x;export{x as await,x as '😀'}",
        "export * from 'source' with {type:'json'}",
        "export {x} from 'source' with {z:'last',a:'first'}",
        "export * from 'first';export * from 'second'",
        "export {}\nexport {}",
    ] {
        assert!(
            parse_module(source).is_ok(),
            "{source}: {:?}",
            parse_module(source)
        );
    }
}

#[test]
fn duplicate_exported_names_conflict_across_local_reexport_and_namespace_forms() {
    for source in [
        "let x;export {x,x}",
        "let x;export{x as a,x as 'a'}",
        "let x;export{x};export{x}",
        "export {x as name} from 'first';export * as name from 'second'",
        "export * as name from 'first';export * as 'name' from 'second'",
        "export {x as default} from 'first';export {y as 'default'} from 'second'",
    ] {
        assert_eq!(
            parse_module(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn local_exports_resolve_only_module_bindings_and_reexports_need_no_local_declaration() {
    for source in [
        "export {missing}",
        "export{x};{let x}",
        "export{x};function f(){var x}",
        "export * as ns from 'source';export{ns as local}",
        "export{eval}",
        "export{await}",
        "export{null as value}",
        "let x;export{'x'}",
    ] {
        assert_eq!(
            parse_module(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    for source in [
        "export{x};{var x}",
        "export{f};function f(){}",
        "export{C};class C{}",
        "export{r};using r=null",
        "export{x};import{x}from'source'",
        "export{missing}from'source'",
    ] {
        assert!(parse_module(source).is_ok(), "{source}");
    }
}

#[test]
fn malformed_export_clauses_and_escaped_terminals_are_syntax_errors() {
    for source in [
        "export *",
        "export * as ns",
        "export {x,,y}",
        "export {x y}",
        "export {x as}",
        "export {1 as x} from 'source'",
        "export * fr\\u006fm 'source'",
        "export * a\\u0073 ns from 'source'",
        "export {x} from `source`",
        "export {} export {}",
    ] {
        assert_eq!(
            parse_module(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn quoted_names_require_well_formed_unicode_and_reexports_share_attribute_checks() {
    for source in [
        "export {'\\ud800'} from 'source'",
        "export {x as '\\udc00'} from 'source'",
        "export * as '\\ud800' from 'source'",
        "export * from 'source' with {type:'a','type':'b'}",
        "export {} from 'source' with {type:1}",
    ] {
        assert_eq!(
            parse_module(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    let module = parse_module("export * from '\\ud800' with {type:'\\udc00'}").unwrap();
    let ExportKind::Star { request, .. } = &module.exports()[0].kind else {
        panic!("star")
    };
    assert_eq!(request.specifier.code_units(), [0xd800]);
    assert_eq!(request.attributes[0].value.code_units(), [0xdc00]);
}

#[test]
fn exports_remain_module_items_and_default_forms_keep_their_boundary() {
    for source in ["export {}", "let x;export{x}"] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax
        );
    }
    for source in ["{export {}}", "function f(){export {}}"] {
        assert_eq!(
            parse_module(source).unwrap_err().kind,
            DiagnosticKind::Syntax
        );
    }
    for source in [
        "export let x=1",
        "export function f(){}",
        "export class C{}",
    ] {
        assert_eq!(parse_module(source).unwrap().exports().len(), 1);
    }
    assert_eq!(
        parse_module("export default 1").unwrap_err().kind,
        DiagnosticKind::Unsupported
    );
}
