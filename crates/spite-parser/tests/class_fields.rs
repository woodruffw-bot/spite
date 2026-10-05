//! Public field grammar, initializer boundaries, and early errors (15.7).

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{EvalContext, parse_eval_utf16, parse_script};

#[test]
fn public_field_syntax_and_initializer_context_snapshot() {
    insta::assert_debug_snapshot!(
        parse_script("class C { x; static [1]=()=>this; y=super.z; }").unwrap()
    );
    for source in [
        "class C { x\ny=1\nstatic; get; set=2; async=3; }",
        "class C { get\nx; set\ny; async\nm(){} }",
        "class C { get\nx(){} set\nx(v){} }",
        "class C { static\nx; ['constructor']; static ['prototype']=1; }",
        "class C { x=new.target; y=()=>new.target; z=()=>super.x; }",
        "class C { x=function(){return arguments;}; y=({m(){return arguments;}}); }",
        "function f(){return class C {[arguments[0]]=1;};}",
        "class C { x=class D { [function(){return arguments;}()](){} }; }",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
    let context = EvalContext {
        strict: true,
        in_function: true,
        in_method: true,
        in_class_field_initializer: true,
        ..Default::default()
    };
    assert!(parse_eval_utf16(&JsString::from("new.target;super.x;"), context).is_ok());
    assert!(parse_eval_utf16(&JsString::from("function f(){return arguments;}"), context).is_ok());
    for source in [
        "arguments",
        "()=>arguments",
        "({arguments})",
        "({[arguments](){}})",
    ] {
        assert_eq!(
            parse_eval_utf16(&JsString::from(source), context)
                .unwrap_err()
                .kind,
            DiagnosticKind::Syntax
        );
    }
}

#[test]
fn public_field_early_errors_snapshot() {
    let errors: Vec<_> = [
        "class C { constructor; }",
        "class C { 'constructor'=1; }",
        "class C { static constructor=1; }",
        "class C { static prototype; }",
        "class C { x=arguments; }",
        "class C { x=arg\\u0075ments; }",
        "class C { x=()=>arguments; }",
        "class C { x=({arguments}); }",
        "class C { x=({[arguments](){}}); }",
        "class C { x=class extends arguments{}; }",
        "class C extends B { x=super(); }",
        "class C { static x=()=>super(); }",
        "class C { x=1 y=2; }",
        "class C { x=1, y=2; }",
        "class C { x=delete value; }",
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
