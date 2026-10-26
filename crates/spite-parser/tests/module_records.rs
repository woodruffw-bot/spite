//! ModuleRequests equality/order and ParseModule export entry rewriting (16.2).

use spite_core::JsString;
use spite_parser::{ast::ImportName, parse_module};

#[test]
fn module_has_tla_finds_awaits_in_expressions_bindings_and_all_statement_positions() {
    for source in [
        "await 0",
        "if(false){await 0}",
        "while(false){await 0}",
        "do{await 0}while(false)",
        "for(await 0;false;){}",
        "for(;await 0;){}",
        "for(;;await 0){break}",
        "for await(const x of []){}",
        "for(const x of await source){}",
        "for(const x in await source){}",
        "for(value[await key] of []){}",
        "for({x=value[await key]} of []){}",
        "switch(await 0){}",
        "switch(0){case await 0:break}",
        "switch(0){default:await 0}",
        "try{await 0}catch{}",
        "try{}catch{await 0}",
        "try{}finally{await 0}",
        "throw await 0",
        "label:{await 0}",
        "let {x=await 0}=o",
        "let {[await 0]:x}=o",
        "let [,{x:[y=await 0]}]=o",
        "({x=await 0}=o)",
        "([obj[await 0]]=o)",
        "({[await 0]:x}=o)",
        "([...obj[await 0]]=o)",
        "([{x=await 0}]=o)",
        "({...obj[await 0]}=o)",
        "(await value).x",
        "obj[await key]",
        "fn(await 0)",
        "new C(await 0)",
        "[...await source]",
        "({...await source})",
        "({[await key](){}})",
        "`prefix${await 0}`",
        "tag`${await 0}`",
        "obj?.[await key]",
        "obj?.(await 0)",
        "import(await specifier)",
        "import('m',await options)",
        "false&&(await 0)",
        "true?0:await 0",
        "obj.x+=await 0",
        "export default await 0",
    ] {
        assert!(
            parse_module(source)
                .unwrap_or_else(|error| panic!("{source}: {error:?}"))
                .has_top_level_await(),
            "{source}"
        );
    }
}

#[test]
fn module_has_tla_excludes_nested_functions_methods_arrows_and_textual_awaits() {
    for source in [
        "",
        "'await 0';/* await 0 */",
        "let awaitText='await';({await:1})",
        "async function f(){await 0}",
        "async function* f(){for await(let x of []){yield await x}}",
        "const f=async()=>await 0",
        "const f=async()=>{await 0}",
        "function f(){return async()=>await 0}",
        "export default async()=>await 0",
        "({async f(){await 0},async *g(){yield await 0}})",
        "class C{async f(){await 0};static async g(){await 0};x=async()=>await 0;static{async function f(){await 0}}}",
        "export default async function(){await 0}",
        "let {[async()=>await 0]:x}=o",
        "import 'await' with {await:'await'};export * from 'await'",
    ] {
        assert!(
            !parse_module(source)
                .unwrap_or_else(|error| panic!("{source}: {error:?}"))
                .has_top_level_await(),
            "{source}"
        );
    }
}

#[test]
fn module_has_tla_checks_class_heritage_and_computed_names_without_entering_initializers() {
    for source in [
        "class C extends(await base){}",
        "(class extends(await base){})",
        "class C{[await key](){}}",
        "class C{static [await key](){}}",
        "class C{[await key]=async()=>await 0}",
        "class C{static [await key]=0}",
        "export default class{[await key](){}}",
        "class C{[class{[await key](){}}](){}}",
    ] {
        assert!(
            parse_module(source)
                .unwrap_or_else(|error| panic!("{source}: {error:?}"))
                .has_top_level_await(),
            "{source}"
        );
    }
    for source in [
        "class C extends(async()=>await base){}",
        "class C{[async()=>await key](){}}",
        "class C{static x=class{async method(){await 0}}}",
    ] {
        assert!(
            !parse_module(source).unwrap().has_top_level_await(),
            "{source}"
        );
    }
}

#[test]
fn module_has_tla_handles_large_flat_trees_and_lossless_utf16_input_without_recursion() {
    let expression = std::iter::repeat_n("0", 6000).collect::<Vec<_>>().join("+");
    assert!(!parse_module(&expression).unwrap().has_top_level_await());
    assert!(
        parse_module(&format!("{expression}+await 0"))
            .unwrap()
            .has_top_level_await()
    );
    let statements = "void 0;".repeat(6000);
    assert!(!parse_module(&statements).unwrap().has_top_level_await());
    assert!(
        parse_module(&format!("{statements}await 0"))
            .unwrap()
            .has_top_level_await()
    );
    let source = JsString::from("let text='\u{FFFD}';await 0");
    let mut units = source.code_units().to_vec();
    let position = units.iter().position(|unit| *unit == 0xfffd).unwrap();
    units[position] = 0xd800;
    assert!(
        spite_parser::parse_module_utf16(&JsString::from_code_units(units))
            .unwrap()
            .has_top_level_await()
    );
}

#[test]
fn requests_follow_first_source_occurrence_and_compare_canonical_attributes() {
    let module = parse_module("export {} from 'b';import value from 'a';export * from 'c';import 'c';export {} from 'b' with {b:'2',a:'1'};import 'b' with {a:'1',b:'2'};export {} from 'b' with {a:'1',b:'3'};import('dynamic')").unwrap();
    let entries = module.entries();
    assert_eq!(module.requested_modules(), entries.requested_modules);
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
