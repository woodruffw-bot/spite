//! Ordinary method/accessor grammar, source capture, and scoped early errors.

use spite_core::DiagnosticKind;
use spite_parser::{
    MAX_DEPTH,
    ast::{ExprKind, ObjectElement, StatementKind},
    parse_script,
};

#[test]
fn method_and_accessor_syntax_snapshot() {
    insta::assert_debug_snapshot!(
        parse_script("({m(a=1){return a;}, get [key](){return this.x;}, set x(v){this.y=v;}})")
            .unwrap()
    );
}

#[test]
fn names_modifiers_defaults_and_nested_functions_parse() {
    for source in [
        "({m(){},get x(){},set x(v){},['x'](){},get 1(){}})",
        "({get(){},set(){},async(){},get get(){},set set(value){}})",
        "({get\nx(){return 1;},set\nx(value){}})",
        "({null(){},true(){},false(){},if(){},1n(){},'\\uD800'(){}})",
        r"({\u0069f(){},g\u0065t(){},get \u0078(){}})",
        "({m(a=1,b=()=>a,){return b();},set x(v=1){}})",
        "({__proto__:null,__proto__(){},get __proto__(){},set __proto__(v){}})",
        "({m(){return new.target;},get x(){return ()=>new.target;},set x(v=new.target){}})",
        "({m(){return {get x(){return function(){return 1;};}};}})",
        "for(;;){({m(){while(true){break;}return 1;}});break;}",
        "function f(){return {get [new.target](){return new.target;}};}",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
}

#[test]
fn method_early_errors_are_syntax_errors_with_precise_ranges() {
    let sources = [
        "({m(a,a){}})",
        "({get x(a){}})",
        "({get x(,){}})",
        "({set x(){}})",
        "({set x(a,b){}})",
        "({set x(a,){}})",
        "({set x(...a){}})",
        "({m(a=1){'use strict';}})",
        "({set x(v=1){'use strict';}})",
        "({m(a){let a;}})",
        "({set x(a){let a;}})",
        "({m(eval){'use strict';}})",
        "'use strict';({m(arguments){}})",
        "'use strict';({get x(){let yield;}})",
        "outer:while(1){({m(){break outer;}})}",
        "while(1){({m(){continue;}})}",
        "({[new.target](){}})",
        "({m(){}});new.target",
        r"({g\u0065t x(){}})",
        r"({s\u0065t x(v){}})",
    ];
    let errors: Vec<_> = sources
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
fn function_source_includes_the_method_name_and_accessor_prefix() {
    let script =
        parse_script("({ m /*name*/ (x) { return x; }, get ['x'] /*body*/ () {return 1;} })")
            .unwrap();
    let StatementKind::Expression(expr) = &script.statements()[0].kind else {
        panic!("expression");
    };
    let ExprKind::Parenthesized(expr) = &expr.kind else {
        panic!("parentheses");
    };
    let ExprKind::Object(properties) = &expr.kind else {
        panic!("object");
    };
    for (property, expected) in properties.iter().zip([
        "m /*name*/ (x) { return x; }",
        "get ['x'] /*body*/ () {return 1;}",
    ]) {
        let ObjectElement::Property(property) = property else {
            panic!("property");
        };
        let ExprKind::Function(function) = &property.value.kind else {
            panic!("method");
        };
        assert_eq!(function.source.as_str().unwrap(), expected);
        assert!(function.name.is_none());
    }
}

#[test]
fn unimplemented_method_dependencies_remain_explicit() {
    for source in [
        "({*m(){}})",
        "({async m(){}})",
        "({async *m(){}})",
        "({m(){return super.x;}})",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
}

#[test]
fn nested_method_bodies_obey_parser_depth_on_the_supported_native_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let source = format!(
                "({}0{})",
                "{m(){return ".repeat(MAX_DEPTH * 2),
                ";}}".repeat(MAX_DEPTH * 2)
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
