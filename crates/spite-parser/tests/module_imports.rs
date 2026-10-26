//! ImportDeclaration records, imported bindings and attribute early errors (16.2.2).

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{ast::ImportName, parse_module, parse_script};

#[test]
fn imports_retain_default_namespace_named_bindings_requests_and_source_ranges() {
    let source = "import main,* as ns from 'first' with {z:'last',a:'first'};import {default as other,\"a-b\" as dash,plain} from 'second';let value=1";
    let module = parse_module(source).unwrap();
    assert_eq!(module.imports().len(), 2);
    assert_eq!(module.statements().len(), 1);
    assert_eq!(
        module.imports()[0].request.specifier,
        JsString::from("first")
    );
    assert_eq!(
        module.imports()[0].bindings[0].imported,
        ImportName::Name(JsString::from("default"))
    );
    assert_eq!(
        module.imports()[0].bindings[1].imported,
        ImportName::Namespace
    );
    assert_eq!(
        &source[module.imports()[0].span.start..module.imports()[0].span.end],
        "import main,* as ns from 'first' with {z:'last',a:'first'};"
    );
    insta::assert_debug_snapshot!(module);
}

#[test]
fn all_import_clause_combinations_and_identifier_names_parse() {
    for source in [
        "import 'source'",
        "import {} from 'source'",
        "import value from 'source'",
        "import * as ns from 'source'",
        "import value,{} from 'source'",
        "import value,* as ns from 'source'",
        "import value,{x as y,} from 'source'",
        "import {as,from,async} from 'source'",
        "import {default as d,await as a,yield as y,if as i,null as n,true as t,false as f} from 'source'",
        "import {'*' as star,'' as empty,'😀' as face} from ''",
        "import {d\\u0065fault as d} from 'source'",
        "import value from 'source' with {}",
        "import 'source' with {if:'x',null:'y',true:'z',}",
        "import 'first'\nimport 'second'",
        "let x=1;import value from 'source';x++",
    ] {
        assert!(
            parse_module(source).is_ok(),
            "{source}: {:?}",
            parse_module(source)
        );
    }
}

#[test]
fn import_bindings_share_module_lexical_conflict_checks_without_affecting_nested_scopes() {
    for source in [
        "import x,{a as x} from 'source'",
        "import x,* as x from 'source'",
        "import {a as x,b as x} from 'source'",
        "import x from 'first';import x from 'second'",
        "import x from 'source';let x",
        "var x;import x from 'source'",
        "import x from 'source';{var x}",
        "import x from 'source';function x(){}",
        "import x from 'source';class x{}",
        "import eval from 'source'",
        "import arguments from 'source'",
        "import await from 'source'",
        "import yield from 'source'",
        "import {await} from 'source'",
        "import {default} from 'source'",
    ] {
        assert_eq!(
            parse_module(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    for source in [
        "import x from 'source';{let x}",
        "import x from 'source';function f(x){return x}",
        "import x from 'source';function f(){var x}",
        "import x from 'source';for(let x of []){}",
    ] {
        assert!(parse_module(source).is_ok(), "{source}");
    }
}

#[test]
fn malformed_clauses_and_nonliteral_specifiers_are_syntax_errors() {
    for source in [
        "import",
        "import value",
        "import * from 'source'",
        "import * as from 'source'",
        "import value,other from 'source'",
        "import value, from 'source'",
        "import {a,,b} from 'source'",
        "import {'x'} from 'source'",
        "import {1 as n} from 'source'",
        "import value from 1",
        "import value from `source`",
        "import value fr\\u006fm 'source'",
        "import * a\\u0073 ns from 'source'",
        "import 'first' import 'second'",
    ] {
        assert_eq!(
            parse_module(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn attributes_have_unique_keys_string_values_and_canonical_utf16_order() {
    for source in [
        "import 'x' with {type:'a',type:'b'}",
        "import 'x' with {type:'a','type':'b'}",
        "import 'x' with {t\\u0079pe:'a',type:'b'}",
        "import 'x' with {1:'a'}",
        "import 'x' with {type:1}",
        "import 'x' with {...other}",
    ] {
        assert_eq!(
            parse_module(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    let module =
        parse_module("import 'x' with {'\\ue000':'later','\\u{10000}':'first',type:'text'}")
            .unwrap();
    let attributes = &module.imports()[0].request.attributes;
    assert_eq!(attributes[0].key, JsString::from("type"));
    assert_eq!(attributes[1].key, JsString::from("\u{10000}"));
    assert_eq!(attributes[2].key, JsString::from("\u{e000}"));
}

#[test]
fn quoted_import_names_require_well_formed_unicode_without_restricting_specifier_strings() {
    assert_eq!(
        parse_module("import {'\\ud800' as x} from 'source'")
            .unwrap_err()
            .kind,
        DiagnosticKind::Syntax
    );
    assert_eq!(
        parse_module("import {'\\udc00' as x} from 'source'")
            .unwrap_err()
            .kind,
        DiagnosticKind::Syntax
    );
    assert!(parse_module("import {'\\ud800\\udc00' as x} from 'source'").is_ok());
    let module = parse_module("import '\\ud800' with {type:'\\udc00'}").unwrap();
    assert_eq!(module.imports()[0].request.specifier.code_units(), [0xd800]);
    assert_eq!(
        module.imports()[0].request.attributes[0].value.code_units(),
        [0xdc00]
    );
}

#[test]
fn static_imports_remain_module_items_and_dynamic_import_expressions_keep_their_grammar() {
    for source in ["import 'source'", "import value from 'source'"] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax
        );
    }
    for source in [
        "{import 'source'}",
        "function f(){import value from 'source'}",
    ] {
        assert_eq!(
            parse_module(source).unwrap_err().kind,
            DiagnosticKind::Syntax
        );
    }
    assert!(
        parse_module("import('source');import.meta")
            .unwrap()
            .imports()
            .is_empty()
    );
    assert_eq!(
        parse_module("export {}").unwrap_err().kind,
        DiagnosticKind::Unsupported
    );
}
