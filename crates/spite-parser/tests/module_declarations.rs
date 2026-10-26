//! Exported declarations share ordinary Module declarations and scope rules (16.2.3).

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{ast::ExportKind, parse_module, parse_script};

#[test]
fn declaration_exports_retain_all_destructured_names_and_declaration_ranges() {
    let source = "export var {a:b,c}=source,[d,...e]=source;export const [f,{g:h}]=source;export function* gen(){}export class C{}";
    let module = parse_module(source).unwrap();
    assert_eq!(module.statements().len(), 4);
    let records: Vec<_> = module
        .exports()
        .iter()
        .map(|declaration| {
            let ExportKind::Declaration { names } = &declaration.kind else {
                panic!("declaration")
            };
            let names: Vec<_> = names
                .iter()
                .map(|name| {
                    assert!(!name.quoted);
                    (name.value.clone(), &source[name.span.start..name.span.end])
                })
                .collect();
            (names, &source[declaration.span.start..declaration.span.end])
        })
        .collect();
    assert_eq!(
        records[0]
            .0
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>(),
        ["b", "c", "d", "e"].map(JsString::from)
    );
    insta::assert_debug_snapshot!(records);
}

#[test]
fn variable_lexical_function_async_generator_and_class_exports_parse() {
    for source in [
        "export var x",
        "export let x",
        "export const x=1,y=2",
        "export var [x,y]=source",
        "export let {x:y,...rest}=source",
        "export function f(){}",
        "export function* g(){yield 1}",
        "export async function f(){await 1}",
        "export class C{}",
        "export class C extends B{method(){return super.method()}}",
        "export var x\nexport let y",
        "export function f(){};export class C{};",
    ] {
        assert!(
            parse_module(source).is_ok(),
            "{source}: {:?}",
            parse_module(source)
        );
    }
}

#[test]
fn exported_declarations_participate_in_shared_module_scope_conflicts() {
    for source in [
        "import x from 'source';export var x",
        "import x from 'source';export let x",
        "export let x;var x",
        "let x;export var x",
        "export function f(){};var f",
        "function f(){};export function f(){}",
        "export class C{};let C",
        "export const x=1;{var x}",
        "export var {x}=source;let x",
        "export function f(eval){}",
        "export let await=1",
    ] {
        assert_eq!(
            parse_module(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    for source in [
        "export var x;var x",
        "export let x;{let x}",
        "export function f(){};{function f(){}}",
    ] {
        assert!(parse_module(source).is_ok(), "{source}");
    }
}

#[test]
fn declaration_export_names_conflict_with_aliases_reexports_and_namespaces() {
    for source in [
        "export var x,x",
        "export var x;export var x",
        "export var {a:x,b:x}=source",
        "export var x;export{x}",
        "export function f(){};export {remote as f} from 'source'",
        "export class C{};export * as C from 'source'",
        "export let x;export {other as 'x'} from 'source'",
    ] {
        let error = parse_module(source).unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
        assert_eq!(error.message, "duplicate exported name", "{source}");
    }
}

#[test]
fn named_local_exports_can_reference_exported_declarations_in_either_order() {
    for source in [
        "export{x as alias};export let x",
        "export var x;export{x as alias}",
        "export{f as alias};export function f(){}",
        "export{C as alias};export class C{}",
        "export const {x:y}=source;export{y as alias}",
    ] {
        assert_eq!(parse_module(source).unwrap().exports().len(), 2, "{source}");
    }
}

#[test]
fn malformed_or_nested_declaration_exports_are_syntax_errors() {
    for source in [
        "export const x",
        "export var [x]",
        "export let",
        "export function(){}",
        "export class{}",
        "export using x=null",
        "export await using x=null",
        "export async\nfunction f(){}",
        "export 1",
        "export ;",
        "export funct\\u0069on f(){}",
        "{export let x}",
        "function f(){export var x}",
        "if(true)export function f(){}",
    ] {
        assert_eq!(
            parse_module(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    assert_eq!(
        parse_script("export var x").unwrap_err().kind,
        DiagnosticKind::Syntax
    );
    assert_eq!(parse_module("export default 1").unwrap().exports().len(), 1);
    assert_eq!(
        parse_module("export async function* g(){}")
            .unwrap()
            .exports()
            .len(),
        1
    );
}
