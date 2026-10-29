//! Static host loading retains phase, identity and explicit unsupported outcomes.

use spite_runtime::Limits;
use spite_test262::{ModuleSource, Outcome, Runner, Stage};

fn source(metadata: &str, body: &str) -> String {
    format!("/*---\n{metadata}\n---*/\n{body}")
}
fn outcome(
    body: &str,
    metadata: &str,
    mut load: impl FnMut(&str) -> Result<ModuleSource, String>,
) -> Outcome {
    Runner::default()
        .run_with_module_loader(
            &source(metadata, body),
            None,
            "root",
            |_| Ok(String::new()),
            |_, request| load(&String::from_utf16(request.specifier.code_units()).unwrap()),
        )
        .unwrap()
        .remove(0)
        .outcome
}

#[test]
fn self_aliases_shared_dependencies_and_cycles_use_stable_host_identities() {
    let root = source(
        "flags: [module]",
        "import * as self from'alias';import{seen,increment}from'b';export var x=1;export function inc(){x++};if(self.inc!==inc||seen!==undefined)throw 1;increment();if(x!==2)throw 2",
    );
    let result = Runner::default().run_with_module_loader(
        &root, None, "root", |_| Ok(String::new()),
        |_, request| {
            let name = String::from_utf16(request.specifier.code_units()).unwrap();
            Ok(if matches!(name.as_str(), "alias" | "root") {
                ModuleSource { identifier: "root".into(), source: root.clone() }
            } else {
                ModuleSource {
                    identifier: "b".into(),
                    source: "import{x,inc}from'root';export const seen=x;export function increment(){inc()}".into(),
                }
            })
        },
    ).unwrap();
    assert_eq!(result[0].outcome, Outcome::Passed);
}

#[test]
fn dependency_loading_parsing_and_linking_failures_never_satisfy_runtime_negatives() {
    let metadata = "flags: [module]\nnegative:\n  phase: runtime\n  type: SyntaxError";
    assert!(matches!(
        outcome("import{missing}from'leaf'", metadata, |_| Ok(
            ModuleSource {
                identifier: "leaf".into(),
                source: "export const present=1".into()
            }
        )),
        Outcome::Failed {
            stage: Stage::Resolution,
            ..
        }
    ));
    assert!(matches!(
        outcome("import'leaf'", metadata, |_| Ok(ModuleSource {
            identifier: "leaf".into(),
            source: "@".into()
        })),
        Outcome::Failed {
            stage: Stage::Resolution,
            ..
        }
    ));
    assert!(matches!(
        outcome("import'leaf'", metadata, |_| Err(
            "source unavailable".into()
        )),
        Outcome::SetupFailure(_)
    ));
    assert_eq!(
        outcome("import'leaf';throw new SyntaxError()", metadata, |_| Ok(
            ModuleSource {
                identifier: "leaf".into(),
                source: String::new()
            }
        )),
        Outcome::Passed
    );
}

#[test]
fn repeated_host_identifiers_must_retain_exact_source_text() {
    let mut calls = 0;
    let result = outcome("import'first';import'second'", "flags: [module]", |_| {
        calls += 1;
        Ok(ModuleSource {
            identifier: "same".into(),
            source: format!("export const x={calls}"),
        })
    });
    assert!(
        matches!(result,Outcome::SetupFailure(message) if message.contains("identity changed source"))
    );
}

#[test]
fn scripts_and_parse_negatives_never_request_module_sources() {
    let cases = Runner::default()
        .run_with_module_loader(
            &source("", "1"),
            None,
            "root",
            |_| Ok(String::new()),
            |_, _| panic!("Script loaded modules"),
        )
        .unwrap();
    assert_eq!(cases.len(), 2);
    assert!(cases.iter().all(|case| case.outcome == Outcome::Passed));
    let cases = Runner::default()
        .run_with_module_loader(
            &source(
                "flags: [module]\nnegative:\n  phase: parse\n  type: SyntaxError",
                "import.meta=1",
            ),
            None,
            "root",
            |_| panic!("parse negative loaded harness"),
            |_, _| panic!("parse negative loaded modules"),
        )
        .unwrap();
    assert!(matches!(cases[0].outcome, Outcome::UnverifiedSyntax(_)));
}

#[test]
fn import_attributes_and_asynchronous_graphs_keep_explicit_unsupported_outcomes() {
    assert!(matches!(
        outcome(
            "import'leaf' with {type:'json'}",
            "flags: [module]",
            |_| panic!("unsupported attributes loaded source")
        ),
        Outcome::Unsupported {
            stage: Stage::Resolution,
            ..
        }
    ));
    assert!(matches!(
        outcome("import'leaf'", "flags: [module]", |_| Ok(ModuleSource {
            identifier: "leaf".into(),
            source: "await 0".into()
        })),
        Outcome::Unsupported {
            stage: Stage::Runtime,
            ..
        }
    ));
}

#[test]
fn dependency_source_limits_remain_resource_failures_before_evaluation() {
    let root = source("flags: [module]", "import'leaf'");
    let result = Runner {
        limits: Limits {
            max_source_bytes: Some(root.len()),
            ..Limits::default()
        },
    }
    .run_with_module_loader(
        &root,
        None,
        "root",
        |_| Ok(String::new()),
        |_, _| {
            Ok(ModuleSource {
                identifier: "leaf".into(),
                source: " ".repeat(root.len() + 1),
            })
        },
    )
    .unwrap();
    assert!(matches!(
        result[0].outcome,
        Outcome::Limit {
            stage: Stage::Resolution,
            ..
        }
    ));
}

fn resolution(
    body: &str,
    error_type: &str,
    mut load: impl FnMut(&str) -> Result<ModuleSource, String>,
) -> Outcome {
    let metadata = format!("flags: [module]\nnegative:\n  phase: resolution\n  type: {error_type}");
    Runner::default()
        .run_with_module_loader(
            &source(&metadata, body),
            None,
            "root",
            |_| panic!("resolution negative evaluated harness"),
            |_, request| load(&String::from_utf16(request.specifier.code_units()).unwrap()),
        )
        .unwrap()
        .remove(0)
        .outcome
}

#[test]
fn resolution_negatives_match_missing_and_ambiguous_exports_without_evaluating_harness_or_bodies() {
    for body in [
        "import{missing}from'leaf';$DONOTEVALUATE()",
        "export{missing}from'leaf';$DONOTEVALUATE()",
    ] {
        assert_eq!(
            resolution(body, "SyntaxError", |_| Ok(ModuleSource {
                identifier: "leaf".into(),
                source: "export const present=1".into()
            })),
            Outcome::Passed
        );
    }
    assert_eq!(
        resolution(
            "import{x}from'stars';$DONOTEVALUATE()",
            "SyntaxError",
            |name| Ok(ModuleSource {
                identifier: name.into(),
                source: match name {
                    "stars" => "export * from'first';export * from'second'",
                    _ => "export const x=1",
                }
                .into()
            })
        ),
        Outcome::Passed
    );
}

#[test]
fn successful_linking_never_evaluates_a_resolution_negative_body_to_obtain_its_exception() {
    assert!(matches!(
        resolution("throw new SyntaxError()", "SyntaxError", |_| panic!(
            "source requested"
        )),
        Outcome::Failed {
            stage: Stage::Resolution,
            ..
        }
    ));
    assert!(matches!(
        resolution(
            "import{present}from'leaf';throw new SyntaxError()",
            "SyntaxError",
            |_| Ok(ModuleSource {
                identifier: "leaf".into(),
                source: "export const present=1".into()
            })
        ),
        Outcome::Failed {
            stage: Stage::Resolution,
            ..
        }
    ));
    assert!(matches!(
        resolution("import{missing}from'leaf'", "TypeError", |_| Ok(
            ModuleSource {
                identifier: "leaf".into(),
                source: String::new()
            }
        )),
        Outcome::Failed {
            stage: Stage::Resolution,
            ..
        }
    ));
}

#[test]
fn resolution_negatives_reject_parse_failures_and_preserve_missing_or_unsupported_source_outcomes()
{
    assert!(matches!(
        resolution("import.meta=1", "SyntaxError", |_| panic!(
            "invalid root loaded sources"
        )),
        Outcome::Failed {
            stage: Stage::Parse,
            ..
        }
    ));
    assert!(matches!(
        resolution("import'leaf'", "SyntaxError", |_| Ok(ModuleSource {
            identifier: "leaf".into(),
            source: "@".into()
        })),
        Outcome::Failed {
            stage: Stage::Resolution,
            ..
        }
    ));
    assert!(matches!(
        resolution("import'leaf'", "SyntaxError", |_| Ok(ModuleSource {
            identifier: "leaf".into(),
            source: "await using x=null".into()
        })),
        Outcome::Unsupported {
            stage: Stage::Resolution,
            ..
        }
    ));
    assert!(matches!(
        resolution("import'leaf'", "SyntaxError", |_| Err(
            "missing source".into()
        )),
        Outcome::SetupFailure(_)
    ));
    let case = Runner::default()
        .run(
            &source(
                "flags: [module]\nnegative:\n  phase: resolution\n  type: SyntaxError",
                "import'leaf'",
            ),
            None,
            |_| panic!("resolution negative loaded harness"),
        )
        .unwrap();
    assert!(matches!(
        case[0].outcome,
        Outcome::Unsupported {
            stage: Stage::Resolution,
            ..
        }
    ));
}

#[test]
fn resolution_negative_resource_refusals_never_count_as_language_exceptions() {
    let root = source(
        "flags: [module]\nnegative:\n  phase: resolution\n  type: SyntaxError",
        "import'leaf'",
    );
    let cases = Runner {
        limits: Limits {
            max_source_bytes: Some(root.len()),
            ..Limits::default()
        },
    }
    .run_with_module_loader(
        &root,
        None,
        "root",
        |_| panic!("resolution negative loaded harness"),
        |_, _| {
            Ok(ModuleSource {
                identifier: "leaf".into(),
                source: " ".repeat(root.len() + 1),
            })
        },
    )
    .unwrap();
    assert!(matches!(
        cases[0].outcome,
        Outcome::Limit {
            stage: Stage::Resolution,
            ..
        }
    ));
}
