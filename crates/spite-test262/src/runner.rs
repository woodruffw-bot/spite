use crate::{Metadata, MetadataError, Mode, Negative, Phase};
use spite_core::{Diagnostic, DiagnosticKind, Span};
use spite_parser::parse_script;
use spite_runtime::{Error, ExceptionKind, Limits, Realm};

/// A runner stage, distinguishing harness setup from language phases.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stage {
    /// Test parsing and early errors.
    Parse,
    /// Host capabilities, harness loading, or harness evaluation.
    Harness,
    /// Test evaluation.
    Runtime,
}

/// A reviewed parse-negative rejection, in the unchanged test source.
///
/// Require both fields while the parser is incomplete. An unrelated syntax error
/// must not count as a passing conformance test.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseExpectation {
    /// Expected source range before strict-mode prefix insertion.
    pub span: Span,
    /// Exact expected parser diagnostic message.
    pub message: String,
}

/// A conformance result, including non-passing infrastructure categories.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// The supported test matched its required completion or reviewed exception.
    Passed,
    /// The test completed incorrectly or failed at the wrong phase or with the wrong type.
    Failed {
        /// Stage where the mismatch was observed.
        stage: Stage,
        /// Explanation of the mismatch.
        message: String,
    },
    /// The implementation or conformance host lacks a required feature.
    Unsupported {
        /// Stage requiring the missing feature.
        stage: Stage,
        /// Missing capability or original diagnostic.
        message: String,
    },
    /// A host resource limit aborted processing; this is never a language exception.
    Limit {
        /// Stage whose resource budget was exhausted.
        stage: Stage,
        /// Original resource diagnostic.
        message: String,
    },
    /// A parse-negative syntax error lacks reviewed evidence for its rejection point.
    UnverifiedSyntax(Diagnostic),
    /// Harness loading or evaluation failed independently of the test's expectation.
    SetupFailure(String),
}

/// The outcome of one required execution variant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseResult {
    /// Requested source mode.
    pub mode: Mode,
    /// Result for this variant, not combined with other modes.
    pub outcome: Outcome,
}

/// A synchronous Script runner with explicit unsupported-capability reporting.
#[derive(Clone, Debug)]
pub struct Runner {
    /// Limits used for each Script evaluation in a fresh test realm. Defaults
    /// to one million work units and the runtime defaults for other resources.
    pub limits: Limits,
}

impl Default for Runner {
    fn default() -> Self {
        Self {
            limits: Limits {
                // Upstream tests combine many assertions and expensive exact
                // numeric conversions in one Script. Keep a bounded allowance
                // separate from the smaller interactive runtime default.
                max_steps: 1_000_000,
                ..Limits::default()
            },
        }
    }
}

impl Runner {
    /// Runs all metadata-selected variants in independent realms.
    ///
    /// `load_harness` provides unchanged UTF-8 harness files by safe relative name.
    /// Parse-negative tests never evaluate harness code or test bodies. Modules,
    /// async completion, and agent settings are explicitly unsupported. Host helper
    /// names are reserved so missing helpers cannot masquerade as ReferenceError
    /// passes. The default JavaScript host is not modified by this runner.
    pub fn run(
        &self,
        source: &str,
        review: Option<&ParseExpectation>,
        mut load_harness: impl FnMut(&str) -> Result<String, String>,
    ) -> Result<Vec<CaseResult>, MetadataError> {
        let metadata = Metadata::parse(source)?;
        Ok(metadata
            .modes()
            .into_iter()
            .map(|mode| {
                let outcome = self.run_mode(source, &metadata, mode, review, &mut load_harness);
                CaseResult { mode, outcome }
            })
            .collect())
    }

    fn run_mode(
        &self,
        original: &str,
        metadata: &Metadata,
        mode: Mode,
        review: Option<&ParseExpectation>,
        load_harness: &mut impl FnMut(&str) -> Result<String, String>,
    ) -> Outcome {
        if mode == Mode::Module {
            return unsupported(
                Stage::Parse,
                "module parsing and resolution are not implemented",
            );
        }
        let source = mode.prepare_source(original);
        let script = match parse_script(&source) {
            Ok(script) => script,
            Err(diagnostic) => {
                return parse_result(
                    diagnostic,
                    metadata.negative.as_ref(),
                    review,
                    source.len() - original.len(),
                );
            }
        };
        if metadata
            .negative
            .as_ref()
            .is_some_and(|n| n.phase == Phase::Parse)
        {
            return failed(
                Stage::Parse,
                "expected a parse exception, but parsing succeeded",
            );
        }
        if metadata.has_flag("async") {
            return unsupported(
                Stage::Harness,
                "asynchronous Test262 completion is not implemented",
            );
        }
        if metadata.has_flag("CanBlockIsTrue") || metadata.has_flag("CanBlockIsFalse") {
            return unsupported(
                Stage::Harness,
                "Test262 agent configuration is not implemented",
            );
        }
        let mut realm = Realm::new(self.limits);
        for name in ["print", "$262"] {
            realm.reserve_unsupported_global(name);
        }
        let files = metadata.harness_files();
        for name in &files {
            if name.starts_with('/')
                || name.contains('\\')
                || name.split('/').any(|part| matches!(part, "" | "." | ".."))
            {
                return Outcome::SetupFailure(format!("unsafe harness path: {name}"));
            }
        }
        for name in files {
            let harness = match load_harness(name) {
                Ok(source) => source,
                Err(message) => return Outcome::SetupFailure(format!("{name}: {message}")),
            };
            if let Err(error) = realm.eval(&harness) {
                return host_failure(&error, Stage::Harness)
                    .unwrap_or_else(|| Outcome::SetupFailure(format!("{name}: {error}")));
            }
        }
        match realm.evaluate(&script) {
            Ok(_) if metadata.negative.is_none() => Outcome::Passed,
            Ok(_) => failed(
                Stage::Runtime,
                "expected an exception, but evaluation completed normally",
            ),
            Err(error) => {
                if let Some(outcome) = host_failure(&error, Stage::Runtime) {
                    return outcome;
                }
                match error {
                    Error::Exception { kind, .. } => match_exception(
                        metadata.negative.as_ref(),
                        Phase::Runtime,
                        exception_name(kind),
                    ),
                    Error::Thrown(value) => {
                        failed(Stage::Runtime, format!("uncaught primitive throw: {value}"))
                    }
                    Error::Parse(diagnostic) => failed(Stage::Parse, diagnostic.to_string()),
                    Error::Unsupported { .. } | Error::Limit { .. } => {
                        unreachable!("host failures handled above")
                    }
                }
            }
        }
    }
}

fn parse_result(
    diagnostic: Diagnostic,
    negative: Option<&Negative>,
    review: Option<&ParseExpectation>,
    prefix: usize,
) -> Outcome {
    match diagnostic.kind {
        DiagnosticKind::Unsupported => return unsupported(Stage::Parse, diagnostic.to_string()),
        DiagnosticKind::Limit => {
            return Outcome::Limit {
                stage: Stage::Parse,
                message: diagnostic.to_string(),
            };
        }
        DiagnosticKind::Syntax => {}
    }
    let result = match_exception(negative, Phase::Parse, "SyntaxError");
    if result != Outcome::Passed {
        return result;
    }
    let Some(review) = review else {
        return Outcome::UnverifiedSyntax(diagnostic);
    };
    if review.span.start.checked_add(prefix) == Some(diagnostic.span.start)
        && review.span.end.checked_add(prefix) == Some(diagnostic.span.end)
        && review.message == diagnostic.message
    {
        Outcome::Passed
    } else {
        failed(
            Stage::Parse,
            format!("syntax error differs from reviewed rejection: {diagnostic}"),
        )
    }
}

fn match_exception(negative: Option<&Negative>, phase: Phase, error_type: &str) -> Outcome {
    if negative.is_some_and(|n| n.phase == phase && n.error_type == error_type) {
        Outcome::Passed
    } else {
        failed(
            if phase == Phase::Parse {
                Stage::Parse
            } else {
                Stage::Runtime
            },
            format!("unexpected {error_type} during {phase:?}; expected {negative:?}"),
        )
    }
}

fn host_failure(error: &Error, stage: Stage) -> Option<Outcome> {
    match error {
        Error::Unsupported { .. } => Some(unsupported(stage, error.to_string())),
        Error::Limit { .. } => Some(Outcome::Limit {
            stage,
            message: error.to_string(),
        }),
        Error::Parse(diagnostic) if diagnostic.kind == DiagnosticKind::Unsupported => {
            Some(unsupported(stage, diagnostic.to_string()))
        }
        Error::Parse(diagnostic) if diagnostic.kind == DiagnosticKind::Limit => {
            Some(Outcome::Limit {
                stage,
                message: diagnostic.to_string(),
            })
        }
        _ => None,
    }
}

fn exception_name(kind: ExceptionKind) -> &'static str {
    match kind {
        ExceptionKind::SyntaxError => "SyntaxError",
        ExceptionKind::ReferenceError => "ReferenceError",
        ExceptionKind::TypeError => "TypeError",
        ExceptionKind::RangeError => "RangeError",
    }
}
fn failed(stage: Stage, message: impl Into<String>) -> Outcome {
    Outcome::Failed {
        stage,
        message: message.into(),
    }
}
fn unsupported(stage: Stage, message: impl Into<String>) -> Outcome {
    Outcome::Unsupported {
        stage,
        message: message.into(),
    }
}
