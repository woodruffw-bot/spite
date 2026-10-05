//! Private names, lexical class boundaries, and early errors (15.7/16.1).

use spite_core::DiagnosticKind;
use spite_parser::parse_script;

#[test]
fn private_elements_and_brand_precedence_snapshot() {
    insta::assert_debug_snapshot!(parse_script(
        "class C{#x=1;static #y;get #z(){return this.#x;}set #z(v){this.#x=v;}m(o){return #x in o << 1 === true;}n(o){return o?.#x;}}"
    ).unwrap());
}

#[test]
fn private_scope_allows_forward_references_and_crosses_functions() {
    for source in [
        "class C{constructor(){this.#x;}#x;}",
        "class C{m(){return ()=>this.#x;}#x;}",
        "class C{m(){return function(o){return o.#x;};}#x;}",
        "class C{m(){return class{m(o){return o.#x;}};}#x;}",
        "class C{m(){return class{#x;m(o){return o.#x;}};}#x;}",
        "class C{m(){return class extends (this.#x) {#x;};}#x;}",
        "class C{static{let f=()=>this.#x;}static #x;}",
        "class C{static get #x(){}static set #x(v){}}",
        "class C{set #x(v){}get #x(){}}",
        "class C{#prototype;static #static;#true;#null;#if;#await;#yield;}",
        "class C{get\n#x;set\n#y;}",
        "class C{get\n#x(){}set\n#x(v){}}",
        "class C{#arguments;static{this.#arguments;}m(){return this.#arguments;}}",
        "class C{#x;m(o){o.#\\u0078++;++o.#x;o.#x ||= 1;[o.#x]=[1];({x:o.#x}={x:1});for(o.#x of []){}return o.#x();}}",
        "class C{#x;m(o){return o?.#x.y?.() + o.#x`x`;}}",
        "class C{#x;m(o){delete o?.#x.y;delete (o.#x, 1);return new o.#x;}}",
        "class C{#x;m(o){return (#x in o) && #x in o in {};}}",
        "class C{#x;m(o){for(let v=(#x in o);;){break;}}}",
        "class C{#x;m(o){return 1 < (#x in o);}}",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}

#[test]
fn private_element_early_errors_snapshot() {
    let errors: Vec<_> = [
        "o.#x",
        "#x in o",
        "class C{m(o){return o.#x;}}",
        "class C{m(o){return o?.#x;}}",
        "class C{m(o){return #x in o;}}",
        "class C{m(){return class{m(o){return o.#x;}};}}",
        "class C extends (o.#x){#x;}",
        "class C{#x;#x;}",
        "class C{#x;static #x;}",
        "class C{#x;#\\u0078;}",
        "class C{get #x(){}get #x(){}}",
        "class C{get #x(){}static set #x(v){}}",
        "class C{get #x(){}set #x(v){}get #x(){}}",
        "class C{#x;get #x(){}}",
        "class C{#constructor;}",
        "class C{static #constructor(){}}",
        "class C{get #\\u0063onstructor(){}}",
        "class C{#x;m(o){delete o.#x;}}",
        "class C{#x;m(o){delete ((o.#x));}}",
        "class C{#x;m(o){delete o?.#x;}}",
        "class C{#x;m(o){return super.#x;}}",
        "class C{#x;m(o){return !#x in o;}}",
        "class C{#x;m(o){return 1 < #x in o;}}",
        "class C{#x;m(o){return #x + 1;}}",
        "class C{#x;m(o){for(let x=#x in o;;){}}}",
        "class C{# x;}",
        "({#x:1})",
    ]
    .into_iter()
    .map(|source| {
        let error = parse_script(source).unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
        (source, error)
    })
    .collect();
    insta::assert_debug_snapshot!(errors);
}

#[test]
fn nested_private_scopes_hit_the_parser_guard_on_a_two_mebibyte_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let source = format!(
                "let C={}0{};",
                "class {#x;m(){return ".repeat(spite_parser::MAX_DEPTH * 2),
                ";}}".repeat(spite_parser::MAX_DEPTH * 2),
            );
            assert_eq!(
                parse_script(&source).unwrap_err().kind,
                DiagnosticKind::Limit
            );
        })
        .unwrap()
        .join()
        .unwrap();
}
