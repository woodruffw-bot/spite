//! Phase matching, realm isolation, and non-passing infrastructure outcomes.

use spite_core::Span;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};
use spite_test262::{Mode, Outcome, ParseExpectation, Runner, Stage};

fn source(metadata: &str, body: &str) -> String {
    format!("/*---\n{metadata}\n---*/\n{body}")
}
fn raw(body: &str) -> Outcome {
    Runner::default()
        .run(&source("flags: [raw]", body), None, |_| {
            panic!("raw test loaded harness")
        })
        .unwrap()
        .remove(0)
        .outcome
}
fn negative(phase: &str, error_type: &str, body: &str) -> Outcome {
    Runner::default()
        .run(
            &source(
                &format!("flags: [raw]\nnegative:\n  phase: {phase}\n  type: {error_type}"),
                body,
            ),
            None,
            |_| panic!("raw test loaded harness"),
        )
        .unwrap()
        .remove(0)
        .outcome
}

#[test]
fn runtime_negatives_match_both_phase_and_error_type() {
    for (body, kind) in [
        ("missing", "ReferenceError"),
        ("1n + 1", "TypeError"),
        ("1n / 0n", "RangeError"),
        ("let Infinity;", "SyntaxError"),
    ] {
        assert_eq!(negative("runtime", kind, body), Outcome::Passed);
        assert!(matches!(
            negative("runtime", "Test262Error", body),
            Outcome::Failed {
                stage: Stage::Runtime,
                ..
            }
        ));
        assert!(matches!(
            negative("parse", kind, body),
            Outcome::Failed {
                stage: Stage::Parse,
                ..
            }
        ));
        assert!(matches!(
            raw(body),
            Outcome::Failed {
                stage: Stage::Runtime,
                ..
            }
        ));
    }
    assert!(matches!(
        negative("runtime", "SyntaxError", "const x;"),
        Outcome::Failed {
            stage: Stage::Parse,
            ..
        }
    ));
    assert!(matches!(
        negative("runtime", "ReferenceError", "1"),
        Outcome::Failed {
            stage: Stage::Runtime,
            ..
        }
    ));
    assert!(matches!(
        negative("runtime", "ReferenceError", "throw 'ReferenceError'"),
        Outcome::Failed {
            stage: Stage::Runtime,
            ..
        }
    ));
    assert_eq!(raw("let x = 1n; x += 2n;"), Outcome::Passed);
}

#[test]
fn runtime_negatives_inspect_explicit_errors_rethrows_and_current_constructor_names() {
    for (body, name) in [
        ("throw new Error('explicit')", "Error"),
        ("throw new TypeError('explicit')", "TypeError"),
        ("try{+1n;}catch(e){throw e;}", "TypeError"),
        (
            "TypeError.prototype.constructor=RangeError;+1n",
            "RangeError",
        ),
        (
            "TypeError.prototype.constructor={name:'Custom'};+1n",
            "Custom",
        ),
        (
            "let T=TypeError;delete globalThis.TypeError;throw T('saved')",
            "TypeError",
        ),
    ] {
        assert_eq!(negative("runtime", name, body), Outcome::Passed, "{body}");
        assert!(
            matches!(
                negative("runtime", "WrongError", body),
                Outcome::Failed {
                    stage: Stage::Runtime,
                    ..
                }
            ),
            "{body}"
        );
        assert!(
            matches!(
                raw(body),
                Outcome::Failed {
                    stage: Stage::Runtime,
                    ..
                }
            ),
            "{body}"
        );
    }
    for body in [
        "throw 1",
        "throw null",
        "throw {__proto__:null,name:'TypeError'}",
        "throw {constructor:undefined}",
        "throw {constructor:{name:123}}",
        "TypeError.prototype.constructor=null;+1n",
    ] {
        assert!(
            matches!(
                negative("runtime", "TypeError", body),
                Outcome::Failed {
                    stage: Stage::Runtime,
                    ..
                }
            ),
            "{body}"
        );
    }
    // A name property on the thrown object is not its constructor's name.
    assert!(matches!(
        negative("runtime", "TypeError", "throw {name:'TypeError'}"),
        Outcome::Failed {
            stage: Stage::Runtime,
            ..
        }
    ));
}

#[test]
fn parse_negatives_require_the_reviewed_rejection_in_every_mode() {
    let original = source("negative:\n  phase: parse\n  type: SyntaxError", "@");
    let offset = original.find('@').unwrap();
    let mut review = ParseExpectation {
        span: Span::new(offset, offset + 1),
        message: "unexpected character".into(),
    };
    let run = |review: Option<&ParseExpectation>| {
        Runner::default()
            .run(&original, review, |_| {
                panic!("parse-negative loaded harness")
            })
            .unwrap()
    };
    let cases = run(Some(&review));
    assert_eq!(
        cases.iter().map(|case| case.mode).collect::<Vec<_>>(),
        [Mode::Script, Mode::StrictScript]
    );
    assert!(cases.iter().all(|case| case.outcome == Outcome::Passed));
    assert!(
        run(None)
            .iter()
            .all(|case| matches!(case.outcome, Outcome::UnverifiedSyntax(_)))
    );
    review.message = "unrelated syntax error".into();
    assert!(run(Some(&review)).iter().all(|case| matches!(
        case.outcome,
        Outcome::Failed {
            stage: Stage::Parse,
            ..
        }
    )));
    review.message = "unexpected character".into();
    review.span = Span::new(0, 1);
    assert!(run(Some(&review)).iter().all(|case| matches!(
        case.outcome,
        Outcome::Failed {
            stage: Stage::Parse,
            ..
        }
    )));
    assert!(matches!(
        negative("parse", "TypeError", "@"),
        Outcome::Failed { .. }
    ));
}

#[test]
fn unsupported_and_limits_are_never_negative_passes() {
    // A newly supported production must now fail a parse-negative expectation.
    assert!(matches!(
        negative("parse", "SyntaxError", "function f() {}"),
        Outcome::Failed {
            stage: Stage::Parse,
            ..
        }
    ));
    assert!(matches!(
        negative("parse", "SyntaxError", "function* f() {}"),
        Outcome::Unsupported {
            stage: Stage::Parse,
            ..
        }
    ));
    assert!(matches!(
        negative("runtime", "ReferenceError", "Math"),
        Outcome::Unsupported {
            stage: Stage::Runtime,
            ..
        }
    ));
    let runner = Runner {
        limits: Limits {
            max_steps: 20,
            ..Limits::default()
        },
    };
    let original = source(
        "flags: [raw]\nnegative:\n  phase: runtime\n  type: RangeError",
        "while (true) {}",
    );
    assert!(matches!(
        runner.run(&original, None, |_| unreachable!()).unwrap()[0].outcome,
        Outcome::Limit {
            stage: Stage::Runtime,
            ..
        }
    ));
    let oversized = source(
        "flags: [raw]\nnegative:\n  phase: parse\n  type: SyntaxError",
        &" ".repeat(spite_parser::MAX_SOURCE_BYTES),
    );
    assert!(matches!(
        runner.run(&oversized, None, |_| unreachable!()).unwrap()[0].outcome,
        Outcome::Limit {
            stage: Stage::Parse,
            ..
        }
    ));
}

#[test]
fn variants_are_isolated_and_strict_prefix_is_effective() {
    let original = source("description: independent realms", "let once = 1; once = 2;");
    let mut loads = Vec::new();
    let cases = Runner::default()
        .run(&original, None, |name| {
            loads.push(name.to_owned());
            Ok(String::new())
        })
        .unwrap();
    assert!(cases.iter().all(|case| case.outcome == Outcome::Passed));
    assert_eq!(loads, ["assert.js", "sta.js", "assert.js", "sta.js"]);
    let cases = Runner::default()
        .run(
            &source("description: strict test", "unbound = 1;"),
            None,
            |_| Ok(String::new()),
        )
        .unwrap();
    assert_eq!(cases[0].outcome, Outcome::Passed);
    assert!(matches!(
        cases[1].outcome,
        Outcome::Failed {
            stage: Stage::Runtime,
            ..
        }
    ));
}

#[test]
fn harness_runs_in_order_in_the_same_realm_but_its_errors_are_setup_failures() {
    let original = source(
        "flags: [noStrict]\nincludes: [first.js, second.js]",
        "if (order !== 'as12') throw order;",
    );
    let cases = Runner::default()
        .run(&original, None, |name| {
            Ok(match name {
                "assert.js" => "let order = 'a';",
                "sta.js" => "order += 's';",
                "first.js" => "order += '1';",
                "second.js" => "order += '2';",
                _ => unreachable!(),
            }
            .into())
        })
        .unwrap();
    assert_eq!(cases[0].outcome, Outcome::Passed);
    let original = source(
        "flags: [noStrict]\nnegative:\n  phase: runtime\n  type: ReferenceError",
        "missing",
    );
    for harness in ["missing", "throw 'failure'", "const x;"] {
        let cases = Runner::default()
            .run(&original, None, |_| Ok(harness.into()))
            .unwrap();
        assert!(matches!(cases[0].outcome, Outcome::SetupFailure(_)));
    }
    let cases = Runner::default()
        .run(&original, None, |_| Err("file unavailable".into()))
        .unwrap();
    assert!(matches!(cases[0].outcome, Outcome::SetupFailure(_)));
    let cases = Runner::default()
        .run(&original, None, |_| Ok("function* f() {}".into()))
        .unwrap();
    assert!(matches!(
        cases[0].outcome,
        Outcome::Unsupported {
            stage: Stage::Harness,
            ..
        }
    ));
    let runner = Runner {
        limits: Limits {
            max_steps: 20,
            ..Limits::default()
        },
    };
    let cases = runner
        .run(&original, None, |_| Ok("while (true) {}".into()))
        .unwrap();
    assert!(matches!(
        cases[0].outcome,
        Outcome::Limit {
            stage: Stage::Harness,
            ..
        }
    ));
}

#[test]
fn unsupported_host_helpers_never_appear_missing_or_undefined() {
    for body in [
        "print",
        "$262",
        "typeof print",
        "typeof $262",
        "delete print",
        "print = 1",
        "var print",
    ] {
        assert!(
            matches!(
                negative("runtime", "ReferenceError", body),
                Outcome::Unsupported {
                    stage: Stage::Runtime,
                    ..
                }
            ),
            "{body}"
        );
    }
    assert_eq!(
        raw("let print = 1; let $262 = 2; if (print + $262 !== 3) throw 0;"),
        Outcome::Passed
    );
    assert!(matches!(
        Realm::default().eval("print"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
    assert!(matches!(
        Realm::default().eval("$262"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
    let mut realm = Realm::default();
    realm.reserve_unsupported_global("Infinity");
    assert_eq!(realm.eval("Infinity"), Ok(Value::Number(f64::INFINITY)));
}

#[test]
fn unsupported_modes_and_unsafe_harness_paths_do_not_execute() {
    for flags in [
        "[module]",
        "[module, raw]",
        "[async]",
        "[CanBlockIsTrue]",
        "[CanBlockIsFalse]",
    ] {
        let cases = Runner::default()
            .run(&source(&format!("flags: {flags}"), "throw 7"), None, |_| {
                panic!("unsupported host loaded harness")
            })
            .unwrap();
        assert!(
            cases
                .iter()
                .all(|case| matches!(case.outcome, Outcome::Unsupported { .. }))
        );
    }
    for path in [
        "../secret.js",
        "/tmp/secret.js",
        "sub/../../secret.js",
        "sub//file.js",
    ] {
        let cases = Runner::default()
            .run(&source(&format!("includes: [{path}]"), "1"), None, |_| {
                panic!("unsafe include loaded harness")
            })
            .unwrap();
        assert!(
            cases
                .iter()
                .all(|case| matches!(case.outcome, Outcome::SetupFailure(_)))
        );
    }
}

#[test]
fn outcome_snapshot_keeps_non_passing_categories_visible() {
    let outcomes = [
        raw("1"),
        raw("throw 1"),
        negative("runtime", "ReferenceError", "$262"),
        negative("parse", "SyntaxError", "@"),
    ];
    insta::assert_debug_snapshot!(outcomes);
}
