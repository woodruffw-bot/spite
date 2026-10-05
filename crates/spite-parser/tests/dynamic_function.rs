//! Independent dynamic Function grammar goals and combined early errors.

use spite_core::DiagnosticKind;
use spite_parser::{MAX_DEPTH, parse_dynamic_function};

#[test]
fn dynamic_function_retains_standard_source_and_combined_syntax() {
    let parameters = "x = 1, ...rest";
    let body = "return [x, rest, new.target];";
    let function = parse_dynamic_function(parameters, body).unwrap();
    assert_eq!(
        function.source.as_str(),
        "function anonymous(x = 1, ...rest\n) {\nreturn [x, rest, new.target];\n}"
    );
    assert_eq!(function.name.as_ref().unwrap().name, "anonymous");
    assert!(!function.body.is_strict());
    insta::assert_debug_snapshot!(function);
}

#[test]
fn grammar_goals_accept_comments_trailing_commas_and_function_context() {
    for (parameters, body) in [
        ("", ""),
        ("a,a", "return a;"),
        ("a, b,", "return a + b;"),
        ("a // final parameter", "return a // final body statement"),
        ("/* empty */", "/* empty */"),
        ("eval, arguments, yield, await", "return this;"),
        ("x = new.target", "return new.target;"),
        ("x = 'p' in {}", "return x;"),
        ("x = (function(){ return 1; })()", "return x;"),
        ("...rest", "return arguments;"),
        ("x", "'use strict'; return x;"),
        ("x,x", "'use\\x20strict'; return x;"),
        ("x = 010", "return x;"),
        ("async", "async\nfunction f(){}"),
    ] {
        assert!(
            parse_dynamic_function(parameters, body).is_ok(),
            "{parameters:?} / {body:?}: {:?}",
            parse_dynamic_function(parameters, body)
        );
    }
    assert!(
        parse_dynamic_function("x", "'use strict'; return x;")
            .unwrap()
            .body
            .is_strict()
    );
}

#[test]
fn parameter_and_body_text_cannot_complete_or_escape_each_other() {
    let cases = [
        ("/*", "*/ ) {"),
        ("a) { return 1; } //", "return 2;"),
        ("", "} function injected() {"),
        ("a =", "1"),
        ("...rest,", ""),
        ("a,,b", ""),
        ("a", "return ("),
        ("a", "break;"),
        ("a", "continue;"),
    ];
    let diagnostics: Vec<_> = cases
        .into_iter()
        .map(|(parameters, body)| {
            let error = parse_dynamic_function(parameters, body).unwrap_err();
            assert_eq!(
                error.kind,
                DiagnosticKind::Syntax,
                "{parameters:?}/{body:?}"
            );
            error
        })
        .collect();
    insta::assert_debug_snapshot!(diagnostics);
}

#[test]
fn combined_early_errors_use_body_strictness_and_parameter_bindings() {
    for (parameters, body) in [
        ("a,a", "'use strict';"),
        ("eval", "'use strict';"),
        ("arguments", "'use strict';"),
        ("a = 1", "'use strict';"),
        ("...rest", "'use strict';"),
        ("a = 1,a", ""),
        ("a", "let a;"),
        ("a", "const a = 1;"),
        ("", "'use strict'; 010;"),
        ("", "let a; var a;"),
        ("", "outer: { function inner(){ break outer; } }"),
    ] {
        assert_eq!(
            parse_dynamic_function(parameters, body).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{parameters:?} / {body:?}"
        );
    }
}

#[test]
fn unsupported_syntax_and_native_stack_guards_keep_their_categories() {
    for (parameters, body) in [
        ("[x]", "return x;"),
        ("{x}", "return x;"),
        ("", "class C {}"),
        ("", "function* f(){}"),
        ("", "async function f(){}"),
        ("x = (async function(){})", "return x;"),
    ] {
        assert_eq!(
            parse_dynamic_function(parameters, body).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{parameters:?} / {body:?}"
        );
    }
    let body = format!(
        "return {}0{};",
        "(".repeat(MAX_DEPTH + 1),
        ")".repeat(MAX_DEPTH + 1)
    );
    assert_eq!(
        parse_dynamic_function("", &body).unwrap_err().kind,
        DiagnosticKind::Limit
    );
    let parameters = format!(
        "x = {}0{}",
        "(".repeat(MAX_DEPTH + 1),
        ")".repeat(MAX_DEPTH + 1)
    );
    assert_eq!(
        parse_dynamic_function(&parameters, "").unwrap_err().kind,
        DiagnosticKind::Limit
    );
}
