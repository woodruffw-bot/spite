//! ModuleRequests equality/order and ParseModule export entry rewriting (16.2).

use spite_core::JsString;
use spite_parser::{ast::ImportName, parse_module};

#[test]
fn requests_follow_first_source_occurrence_and_compare_canonical_attributes() {
    let module = parse_module("export {} from 'b';import value from 'a';export * from 'c';import 'c';export {} from 'b' with {b:'2',a:'1'};import 'b' with {a:'1',b:'2'};export {} from 'b' with {a:'1',b:'3'};import('dynamic')").unwrap();
    let entries = module.entries();
    let names: Vec<_> = entries
        .requested_modules
        .iter()
        .map(|request| request.specifier.clone())
        .collect();
    assert_eq!(names, ["b", "a", "c", "b", "b"].map(JsString::from));
    assert!(
        entries.requested_modules[..3]
            .iter()
            .all(|request| request.attributes.is_empty())
    );
    assert_eq!(
        entries.requested_modules[3].attributes[0].key,
        JsString::from("a")
    );
    assert_eq!(
        entries.requested_modules[3].attributes[1].value,
        JsString::from("2")
    );
    assert_eq!(
        entries.requested_modules[4].attributes[1].value,
        JsString::from("3")
    );
    assert_eq!(entries.import_entries.len(), 1);
}

#[test]
fn entry_records_cover_local_indirect_namespace_star_and_synthetic_default_exports() {
    let module = parse_module("import defaultValue,{named as local,'*' as star} from 'first' with {type:'json'};import * as ns from 'second';const own=1;export {own,local as renamed,defaultValue as fallback,ns as namespace,star as literalStar};export {remote as indirect} from 'third';export * as whole from 'fourth';export * from 'fifth';export default (function internal(){})").unwrap();
    let entries = module.entries();
    assert_eq!(entries.import_entries.len(), 4);
    assert_eq!(entries.local_export_entries.len(), 2);
    assert_eq!(entries.indirect_export_entries.len(), 6);
    assert_eq!(entries.star_export_entries.len(), 1);
    insta::assert_debug_snapshot!(entries);
}

#[test]
fn imported_namespace_reexports_and_literal_star_names_remain_distinct() {
    let module = parse_module("import * as namespace from 'source';import {'*' as literal} from 'source';import value from 'source';export {namespace as ns,literal as star,value};export default value").unwrap();
    let entries = module.entries();
    assert_eq!(entries.import_entries[0].imported, ImportName::Namespace);
    assert_eq!(
        entries.import_entries[1].imported,
        ImportName::Name(JsString::from("*"))
    );
    assert_eq!(
        entries.import_entries[2].imported,
        ImportName::Name(JsString::from("default"))
    );
    assert_eq!(entries.indirect_export_entries[0].imported, None);
    assert_eq!(
        entries.indirect_export_entries[1].imported,
        Some(JsString::from("*"))
    );
    assert_eq!(
        entries.indirect_export_entries[2].imported,
        Some(JsString::from("default"))
    );
    assert!(
        entries
            .indirect_export_entries
            .iter()
            .all(|entry| entry.request.specifier == JsString::from("source"))
    );
    assert_eq!(
        entries.local_export_entries[0].local,
        JsString::from("*default*")
    );
    assert_eq!(
        entries.local_export_entries[0].exported,
        JsString::from("default")
    );
    assert!(entries.star_export_entries.is_empty());
}

#[test]
fn declarations_and_default_bindings_produce_local_entries_in_export_order() {
    let module = parse_module("export var [x,y]=source;export const {a:b}=source;export function f(){};export class C{};export default class Named{};export {Named as Alias}").unwrap();
    let entries = module.entries();
    let locals: Vec<_> = entries
        .local_export_entries
        .iter()
        .map(|entry| entry.local.clone())
        .collect();
    let exports: Vec<_> = entries
        .local_export_entries
        .iter()
        .map(|entry| entry.exported.clone())
        .collect();
    assert_eq!(
        locals,
        ["x", "y", "b", "f", "C", "Named", "Named"].map(JsString::from)
    );
    assert_eq!(
        exports,
        ["x", "y", "b", "f", "C", "default", "Alias"].map(JsString::from)
    );
    assert!(entries.indirect_export_entries.is_empty());
    for source in [
        "export default function(){}",
        "export default class{}",
        "export default (function internal(){})",
    ] {
        let entries = parse_module(source).unwrap().entries();
        assert_eq!(
            entries.local_export_entries[0].local,
            JsString::from("*default*")
        );
    }
}

#[test]
fn requests_preserve_lossless_strings_and_import_aliases_use_decoded_identifiers() {
    let module = parse_module("import '\\ud800' with {'\\udc00':'\\ud800'};export {} from '\\ud800' with {'\\udc00':'\\ud800'};import '\\udc00';import {remote as v\\u0061lue} from 'source';export {value as alias}").unwrap();
    let entries = module.entries();
    assert_eq!(entries.requested_modules.len(), 3);
    let request = &entries.requested_modules[0];
    assert_eq!(request.specifier.code_units(), [0xd800]);
    assert_eq!(request.attributes[0].key.code_units(), [0xdc00]);
    assert_eq!(request.attributes[0].value.code_units(), [0xd800]);
    assert_eq!(
        entries.requested_modules[1].specifier.code_units(),
        [0xdc00]
    );
    assert_eq!(entries.import_entries[0].local, JsString::from("value"));
    assert_eq!(
        entries.indirect_export_entries[0].imported,
        Some(JsString::from("remote"))
    );
    assert_eq!(
        entries.indirect_export_entries[0].exported,
        JsString::from("alias")
    );
}

#[test]
fn empty_modules_and_dynamic_imports_have_no_static_entries() {
    for source in [
        "",
        "const f=()=>import('dynamic');import.meta;await import('other')",
    ] {
        let entries = parse_module(source).unwrap().entries();
        assert!(entries.requested_modules.is_empty());
        assert!(entries.import_entries.is_empty());
        assert!(entries.local_export_entries.is_empty());
        assert!(entries.indirect_export_entries.is_empty());
        assert!(entries.star_export_entries.is_empty());
    }
}
