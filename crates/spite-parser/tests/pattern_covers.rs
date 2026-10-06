//! Destructuring probes select lexical goals through the supplemental grammar.

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{
    EvalContext, MAX_DEPTH, parse_dynamic_function, parse_eval_utf16, parse_script,
    parse_script_utf16,
};

#[test]
fn regexp_contents_cannot_close_destructuring_assignment_or_iteration_covers() {
    let sources = [
        "({x = /[}]/} = source)",
        "[x = /[\\]]/] = source",
        "({[/[}]/]: x} = source)",
        "[{x = /[}]/}] = source",
        "({x: [y = /[(]/]} = source)",
        "[x = `${/[}`]/}tail`] = source",
        "[x = /[}]/ / 2] = source",
        "for ({x = /[}]/} of rows) ;",
        "for ([x = /[\\]]/] in rows) ;",
        "for ({[/[}]/]: x} of rows) ;",
        "[{[/[}]/]: x}.p] = source",
        "[...{[/[}]/]: x}.p] = source",
        "({get x() { return /[}]/; }})",
        "([{get x() { return /[}]/; }}.p] = source)",
    ];
    for source in sources {
        let expected = parse_script(source).unwrap_or_else(|error| panic!("{source}: {error}"));
        assert_eq!(
            parse_script_utf16(&JsString::from(source)).unwrap(),
            expected
        );
        assert_eq!(
            parse_eval_utf16(&JsString::from(source), EvalContext::default()).unwrap(),
            expected
        );
    }
    for body in ["({x = /[}]/} = source);", "for ([x = /[\\]]/] of rows) ;"] {
        parse_dynamic_function("source, rows", body).unwrap();
    }
}

#[test]
fn supplemental_grammar_errors_retain_their_rejection_tokens() {
    let sources = [
        "[...x,] = source",
        "[...x = 1] = source",
        "({...x,y} = source)",
        "({...{x}} = source)",
        "[x+y] = source",
        "[f()] = source",
        "({bre\\u0061k} = source)",
        "[{get x() {}}] = source",
        "({m() {}} = source)",
        "({set x(v) {}} = source)",
        "({x=1})",
        "({__proto__:1,__proto__:2})",
        "({x = /a/gg} = source)",
        "for ([x = /(/] of rows) ;",
        "({x = /a/gg} = {}) => x",
        "([x = /(/] = []) => x",
    ];
    let diagnostics: Vec<_> = sources
        .into_iter()
        .map(|source| {
            let error = parse_script(source).unwrap_err();
            assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
            (source, error)
        })
        .collect();
    insta::assert_debug_snapshot!(diagnostics);
}

#[test]
fn ordinary_literals_and_member_targets_discard_speculative_pattern_errors() {
    for source in [
        "[1,x+y,f(),...x,y]",
        "({...x,y:1})",
        "({m() {}, get x() {}, set x(v) {}})",
        "[{get x() {}}.p] = source",
        "[{m() {}}[key]] = source",
        "({x: {get y() {}}.p} = source)",
        "[...[x].length] = source",
        "({__proto__:x,__proto__:y} = source)",
        "({x = (y = 1) => y} = source)",
        "class C { #x; m() { ({[this.#x]: y} = source); return ({[this.#x]: y}).p; } }",
        "class C { #x; m() { for ({[this.#x]: y} of rows) ; } }",
        "class C { #x; m() { return [{get y() { return this.#x; }}.p] = source; } }",
        "function f() { [x = new.target] = source; }",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
    for source in [
        "class C { m() { ({[this.#missing]: y} = source); } }",
        "class C { m() { return [{get y() { return this.#missing; }}.p] = source; } }",
        "[x = new.target] = source",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn nested_covers_and_cached_pattern_clones_obey_the_existing_stack_guard() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            for n in [8, 16] {
                let source = format!("{}x{}=source", "[".repeat(n), "]".repeat(n));
                assert!(parse_script(&source).is_ok(), "{source}");
                let source = format!("{}1{}", "[x=".repeat(n), "]".repeat(n));
                assert!(parse_script(&source).is_ok(), "{source}");
            }
            for source in [
                format!(
                    "{}x{}=source",
                    "[".repeat(MAX_DEPTH * 2),
                    "]".repeat(MAX_DEPTH * 2)
                ),
                format!(
                    "{}1{}",
                    "[x=".repeat(MAX_DEPTH * 2),
                    "]".repeat(MAX_DEPTH * 2)
                ),
            ] {
                assert_eq!(
                    parse_script(&source).unwrap_err().kind,
                    DiagnosticKind::Limit
                );
            }
        })
        .unwrap()
        .join()
        .unwrap();
}
