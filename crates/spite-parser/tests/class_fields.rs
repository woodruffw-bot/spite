//! Public field grammar, initializer boundaries, and early errors (15.7).

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{
    EvalContext, parse_eval_utf16, parse_module, parse_module_utf16, parse_script,
    parse_script_utf16,
};

#[test]
fn field_initializers_reset_suspension_context_but_computed_names_keep_it() {
    for source in [
        "var await=1;async function f(){return class{x=await;static y=await;};}",
        "async function f(){class C{x=aw\\u0061it;[await 1]=2;}await 3;}",
        "function* f(){class C{x=1;[yield 2]=3;}yield 4;}",
        "async function* f(){class C{x=await;[yield await 1]=2;}yield await 3;}",
        "async function f(){return class{x=()=>await;y=async()=>await 1;};}",
        "function* f(){return class{x=function*(){yield 1;};};}",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
        assert!(
            parse_script_utf16(&JsString::from(source)).is_ok(),
            "{source}"
        );
    }
    for source in [
        "async function f(){class C{x=1;[await 2]=3;}await 4;}",
        "function* f(){class C{x=1;[yield 2]=3;}yield 4;}",
        "class C{x=async()=>await 1;}await 2;",
    ] {
        assert!(parse_module(source).is_ok(), "{source}");
        assert!(
            parse_module_utf16(&JsString::from(source)).is_ok(),
            "{source}"
        );
    }
    let errors: Vec<_> = [
        "async function f(){return class{x=await 1;};}",
        "async function f(){return class{static x=await 1;};}",
        "async function f(){return class{#x=await 1;};}",
        "async function f(){return class{x=()=>await 1;};}",
        "function* f(){return class{x=yield 1;};}",
        "function* f(){return class{static #x=yield 1;};}",
        "async function* f(){return class{x=yield await 1;};}",
        "async function f(){return class{[await]=1;};}",
    ]
    .into_iter()
    .map(|source| {
        let error = parse_script(source).unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
        assert_eq!(
            parse_script_utf16(&JsString::from(source)).unwrap_err(),
            error
        );
        (source, error)
    })
    .collect();
    insta::assert_debug_snapshot!(errors, @r#"
        [
            (
                "async function f(){return class{x=await 1;};}",
                Diagnostic {
                    kind: Syntax,
                    span: Span {
                        start: 40,
                        end: 41,
                    },
                    message: "expected class field terminator",
                },
            ),
            (
                "async function f(){return class{static x=await 1;};}",
                Diagnostic {
                    kind: Syntax,
                    span: Span {
                        start: 47,
                        end: 48,
                    },
                    message: "expected class field terminator",
                },
            ),
            (
                "async function f(){return class{#x=await 1;};}",
                Diagnostic {
                    kind: Syntax,
                    span: Span {
                        start: 41,
                        end: 42,
                    },
                    message: "expected class field terminator",
                },
            ),
            (
                "async function f(){return class{x=()=>await 1;};}",
                Diagnostic {
                    kind: Syntax,
                    span: Span {
                        start: 44,
                        end: 45,
                    },
                    message: "expected class field terminator",
                },
            ),
            (
                "function* f(){return class{x=yield 1;};}",
                Diagnostic {
                    kind: Syntax,
                    span: Span {
                        start: 35,
                        end: 36,
                    },
                    message: "expected class field terminator",
                },
            ),
            (
                "function* f(){return class{static #x=yield 1;};}",
                Diagnostic {
                    kind: Syntax,
                    span: Span {
                        start: 43,
                        end: 44,
                    },
                    message: "expected class field terminator",
                },
            ),
            (
                "async function* f(){return class{x=yield await 1;};}",
                Diagnostic {
                    kind: Syntax,
                    span: Span {
                        start: 41,
                        end: 46,
                    },
                    message: "expected class field terminator",
                },
            ),
            (
                "async function f(){return class{[await]=1;};}",
                Diagnostic {
                    kind: Syntax,
                    span: Span {
                        start: 38,
                        end: 39,
                    },
                    message: "expected an expression",
                },
            ),
        ]
        "#);
    for source in [
        "class C{x=await;}",
        "async function f(){return class{static x=await;};}",
        "class C{x=()=>await;}",
        "class C{x=await 1;}",
    ] {
        assert_eq!(
            parse_module(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
        assert_eq!(
            parse_module_utf16(&JsString::from(source))
                .unwrap_err()
                .kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

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
