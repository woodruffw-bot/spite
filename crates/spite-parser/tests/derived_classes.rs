//! Derived constructor context and ClassHeritage grammar (15.7, 13.3.7).

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{EvalContext, parse_eval_utf16, parse_script};

#[test]
fn derived_class_syntax_snapshot_and_inherited_context() {
    insta::assert_debug_snapshot!(
        parse_script(
            "class D extends B { constructor(x=()=>super(1)){x();} m(){return super.x;} }"
        )
        .unwrap()
    );
    for source in [
        "class C extends 7{}",
        "class C extends null{}",
        "class C extends (x=>x)(B){}",
        "class C extends (a,b){}",
        "class C extends o?.B{}",
        "class C extends B{constructor(){return ()=>super();}}",
        "class C extends B{constructor(){class X{[super()](){}}}}",
        "class C extends B{constructor(){new (super())();}}",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    let context = EvalContext {
        strict: true,
        in_function: true,
        in_method: true,
        in_derived_constructor: true,
        in_class_field_initializer: false,
    };
    assert!(parse_eval_utf16(&JsString::from("()=>super(...[1]);"), context).is_ok());
    assert_eq!(
        parse_eval_utf16(&JsString::from("function f(){super();}"), context)
            .unwrap_err()
            .kind,
        DiagnosticKind::Syntax
    );
}

#[test]
fn derived_constructor_and_heritage_errors_snapshot() {
    let errors: Vec<_> = [
        "class C extends B{m(){super();}}",
        "class C extends B{static constructor(){super();}}",
        "class C extends B{['constructor'](){super();}}",
        "class C extends B{constructor(){function f(){super();}}}",
        "class C extends B{constructor(){new super();}}",
        "class C extends B{constructor(){new super().m();}}",
        "class C extends B{constructor(){super?.();}}",
        "class C extends B{constructor(){super()=7;}}",
        "class C extends +1{}",
        "class C extends B+1{}",
        "class C extends B()++{}",
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
