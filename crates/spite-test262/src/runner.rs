use crate::{Metadata, MetadataError, Mode, Negative, Phase};
use spite_core::{Diagnostic, DiagnosticKind, JsString, Span};
use spite_parser::{
    parse_module, parse_module_with_source_limit, parse_script, parse_script_with_source_limit,
};
use spite_runtime::{
    Error, FinalizationCleanupOutcome, Limits, PromiseJobOutcome, Realm, Value,
    object::PromiseState,
};

enum ParsedSource {
    Script(spite_parser::ast::Script),
    Module(spite_parser::ast::Module),
}

mod async_completion;
mod modules;
use async_completion::AsyncCompletion;
pub use modules::ModuleSource;
use modules::{ModuleCache, ModuleLoader};

/// A runner stage, distinguishing harness setup from language phases.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stage {
    /// Test parsing and early errors.
    Parse,
    /// Host capabilities, harness loading, or harness evaluation.
    Harness,
    /// Static dependency loading and linking, before evaluation.
    Resolution,
    /// Test evaluation.
    Runtime,
}

/// A reviewed parse-negative rejection, in the unchanged test source.
///
/// Require both fields while the parser is incomplete. An unrelated syntax error
/// must not count as a passing conformance test.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseExpectation {
    /// Expected byte range before strict-mode prefix insertion; EOF may be empty.
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

/// Script and Module execution with phase-aware conformance accounting.
#[derive(Clone, Debug, Default)]
pub struct Runner {
    /// Limits used for each Script evaluation in a fresh test realm.
    /// Uses the runtime defaults, with all host resource quotas disabled.
    pub limits: Limits,
}

impl Runner {
    /// Runs all metadata-selected variants in independent realms.
    ///
    /// `load_harness` provides unchanged UTF-8 harness files by safe relative name.
    /// Parse and resolution negatives never evaluate harness code or test bodies. Async
    /// variants use the unchanged doneprintHandle.js completion protocol and drain
    /// Promise jobs. Module parse negatives use the Module goal and the same exact
    /// review checks. CanBlockIsFalse selects a nonsuspending agent; other cases
    /// use a suspending Test262 host. Raw async harness setup
    /// remains explicitly unsupported. Host helper names are reserved so missing helpers
    /// cannot masquerade as ReferenceError
    /// passes. The default JavaScript host is not modified by this runner.
    pub fn run(
        &self,
        source: &str,
        review: Option<&ParseExpectation>,
        mut load_harness: impl FnMut(&str) -> Result<String, String>,
    ) -> Result<Vec<CaseResult>, MetadataError> {
        self.run_variants(source, review, &mut load_harness, None)
    }

    /// Runs variants with host-provided static and dynamic Module sources.
    ///
    /// The root identifier and returned source identifiers determine shared
    /// records and cycles in one cache per execution variant. Source preparation
    /// failures never satisfy runtime negatives. Dynamic imports complete through
    /// explicit host requests and Promise jobs. The host supports no import
    /// attribute keys: static requests fail with SyntaxError during resolution,
    /// while dynamic requests reject with TypeError during evaluation.
    pub fn run_with_module_loader(
        &self,
        source: &str,
        review: Option<&ParseExpectation>,
        root_identifier: &str,
        mut load_harness: impl FnMut(&str) -> Result<String, String>,
        mut load_module: impl FnMut(
            &str,
            &spite_parser::ast::ModuleRequest,
        ) -> Result<ModuleSource, String>,
    ) -> Result<Vec<CaseResult>, MetadataError> {
        let mut loader = ModuleLoader {
            root_identifier,
            load: &mut load_module,
        };
        self.run_variants(source, review, &mut load_harness, Some(&mut loader))
    }

    fn run_variants(
        &self,
        source: &str,
        review: Option<&ParseExpectation>,
        load_harness: &mut impl FnMut(&str) -> Result<String, String>,
        mut modules: Option<&mut ModuleLoader<'_>>,
    ) -> Result<Vec<CaseResult>, MetadataError> {
        let metadata = Metadata::parse(source)?;
        Ok(metadata
            .modes()
            .into_iter()
            .map(|mode| {
                let outcome = self.run_mode(
                    source,
                    &metadata,
                    mode,
                    review,
                    load_harness,
                    modules.as_deref_mut(),
                );
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
        mut modules: Option<&mut ModuleLoader<'_>>,
    ) -> Outcome {
        let source = mode.prepare_source(original);
        let parsed = if mode == Mode::Module {
            match self.limits.max_source_bytes {
                Some(limit) => parse_module_with_source_limit(original, limit),
                None => parse_module(original),
            }
            .map(ParsedSource::Module)
        } else {
            match self.limits.max_source_bytes {
                Some(limit) => parse_script_with_source_limit(&source, limit),
                None => parse_script(&source),
            }
            .map(ParsedSource::Script)
        };
        let parsed = match parsed {
            Ok(parsed) => parsed,
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
        if let Some(negative) = metadata.negative.as_ref()
            && negative.phase == Phase::Resolution
        {
            let ParsedSource::Module(module) = &parsed else {
                unreachable!("resolution negatives require Module metadata")
            };
            return self.run_module_resolution(module, original, negative, modules);
        }
        let asynchronous = metadata.has_flag("async");
        if asynchronous && metadata.has_flag("raw") {
            return unsupported(
                Stage::Harness,
                "raw asynchronous Test262 harness setup is not implemented",
            );
        }
        let mut realm = if metadata.has_flag("CanBlockIsFalse") {
            Realm::new(self.limits)
        } else {
            Realm::new_suspending(self.limits)
        };
        if let Err(error) = realm.install_test262_host() {
            return host_failure(&error, Stage::Harness)
                .unwrap_or_else(|| Outcome::SetupFailure(format!("Test262 host: {error}")));
        }
        let outcome = (|| {
            let completion = if asynchronous {
                match AsyncCompletion::install(&mut realm) {
                    Ok(completion) => Some(completion),
                    Err(error) => {
                        return host_failure(&error, Stage::Harness).unwrap_or_else(|| {
                            Outcome::SetupFailure(format!("async print host: {error}"))
                        });
                    }
                }
            } else {
                for name in ["print", "$DONE"] {
                    realm.reserve_unsupported_global(name);
                }
                None
            };
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
            let mut module_cache = ModuleCache::default();
            if modules.is_some() {
                realm.enable_module_imports();
            }
            let evaluated = match &parsed {
                ParsedSource::Script(script) => realm.evaluate(script),
                ParsedSource::Module(module) => {
                    if let Some(loader) = modules.as_deref_mut() {
                        let loaded = loader
                            .load_graph(&mut realm, module, original, &mut module_cache)
                            .map_err(modules::ModuleLoadError::into_error)
                            .and_then(|record| {
                                realm.link_module(&record)?;
                                Ok(record)
                            });
                        let record = match loaded {
                            Ok(record) => record,
                            Err(error) => {
                                return host_failure(&error, Stage::Resolution).unwrap_or_else(
                                    || failed(Stage::Resolution, error.to_string()),
                                );
                            }
                        };
                        realm.evaluate_module_record(&record)
                    } else {
                        realm.evaluate_module(module)
                    }
                }
            };
            let (mut result, module_promise) = match evaluated {
                Ok(value) => (Ok(()), (mode == Mode::Module).then_some(value)),
                Err(error) => (Err(error), None),
            };
            let module_promise = match module_promise {
                Some(value) => match realm.root_value(value, usize::MAX) {
                    Ok(root) => Some(root),
                    Err(error) => {
                        return Outcome::SetupFailure(format!("module Promise root: {error}"));
                    }
                },
                None => None,
            };
            if result.is_ok()
                && (asynchronous
                    || mode == Mode::Module
                    || modules.is_some()
                    || realm.has_requested_gc())
            {
                loop {
                    if let Err(error) =
                        realm.run_requested_gc(self.limits.max_steps.unwrap_or(usize::MAX))
                    {
                        result = Err(error);
                        break;
                    }
                    match realm.run_next_atomics_wait_job() {
                        Ok(true) => continue,
                        Ok(false) => (),
                        Err(error) => {
                            result = Err(error);
                            break;
                        }
                    }
                    match realm.run_finalization_cleanup_job() {
                        Ok(FinalizationCleanupOutcome::NoJob) => (),
                        Ok(FinalizationCleanupOutcome::Completed) => continue,
                        Ok(FinalizationCleanupOutcome::Reported(error)) | Err(error) => {
                            result = Err(error);
                            break;
                        }
                    }
                    if let Some(loader) = modules.as_deref_mut()
                        && let Err(error) = loader.service_requests(&mut realm, &mut module_cache)
                    {
                        return host_failure(&error, Stage::Resolution).unwrap_or_else(|| {
                            failed(
                                Stage::Resolution,
                                format!("dynamic source preparation failed: {error}"),
                            )
                        });
                    }
                    match realm.run_next_promise_job() {
                        Ok(PromiseJobOutcome::NoJob) => {
                            let Some(delay) = realm.next_atomics_wait_delay() else {
                                break;
                            };
                            let completion_pending = if let Some(completion) = completion.as_ref() {
                                match completion.has_completion(&realm) {
                                    Ok(complete) => !complete,
                                    Err(outcome) => return outcome,
                                }
                            } else {
                                false
                            };
                            let module_pending = module_promise.as_ref().is_some_and(|root| {
                                let Value::Object(promise) = root.value() else {
                                    unreachable!("module promise")
                                };
                                matches!(
                                    realm
                                        .inspect_object(promise)
                                        .expect("rooted module Promise")
                                        .promise_state(),
                                    Some(PromiseState::Pending)
                                )
                            });
                            if !completion_pending && !module_pending {
                                break;
                            }
                            std::thread::sleep(delay.min(std::time::Duration::from_millis(10)));
                        }
                        Ok(PromiseJobOutcome::Completed) => {}
                        Ok(PromiseJobOutcome::Reported(error)) | Err(error) => {
                            result = Err(error);
                            break;
                        }
                    }
                }
            }
            if result.is_ok()
                && let Some(Value::Object(promise)) = module_promise
                    .as_ref()
                    .map(spite_runtime::RootedValue::value)
            {
                match realm
                    .inspect_object(promise)
                    .expect("module evaluation Promise")
                    .promise_state()
                    .expect("module Promise brand")
                {
                    PromiseState::Fulfilled(_) => (),
                    PromiseState::Rejected(reason) => result = Err(Error::Thrown(reason.clone())),
                    PromiseState::Pending => {
                        return failed(Stage::Runtime, "module evaluation Promise did not settle");
                    }
                }
            }
            if result.is_ok()
                && let Some(completion) = completion
                && let Err(outcome) = completion.finish(&mut realm)
            {
                return outcome;
            }
            match result {
                Ok(_) if metadata.negative.is_none() => Outcome::Passed,
                Ok(_) => failed(
                    Stage::Runtime,
                    "expected an exception, but evaluation completed normally",
                ),
                Err(error) => {
                    if let Some(outcome) = host_failure(&error, Stage::Runtime) {
                        return outcome;
                    }
                    if let Error::Parse(diagnostic) = error {
                        return failed(Stage::Parse, diagnostic.to_string());
                    }
                    let Some(negative) = metadata.negative.as_ref() else {
                        return failed(
                            Stage::Runtime,
                            match error {
                                Error::Thrown(value) => format!("uncaught throw: {value}"),
                                _ => format!("uncaught exception: {error}"),
                            },
                        );
                    };
                    // Test262 INTERPRETING.md defines `type` as the thrown value's
                    // constructor name. Read it through the realm so accessor
                    // effects and failures are not replaced with host heuristics.
                    match exception_constructor_name(&mut realm, &error) {
                        Ok(Some(name))
                            if negative.phase == Phase::Runtime
                                && name == JsString::from(negative.error_type.as_str()) =>
                        {
                            Outcome::Passed
                        }
                        Ok(name) => failed(
                            Stage::Runtime,
                            format!(
                                "runtime exception with constructor name {name:?} does not match {:?} {}",
                                negative.phase, negative.error_type
                            ),
                        ),
                        Err(error) => host_failure(&error, Stage::Runtime).unwrap_or_else(|| {
                            failed(
                                Stage::Runtime,
                                format!("exception inspection failed: {error}"),
                            )
                        }),
                    }
                }
            }
        })();
        if let Err(error) = realm.finish_test262_agents() {
            return host_failure(&error, Stage::Runtime).unwrap_or_else(|| {
                failed(Stage::Runtime, format!("Test262 worker shutdown: {error}"))
            });
        }
        outcome
    }
}

fn exception_constructor_name(realm: &mut Realm, error: &Error) -> Result<Option<JsString>, Error> {
    let Some(value) = realm.exception_value(error)? else {
        return Ok(None);
    };
    if !matches!(value, Value::Object(_)) {
        return Ok(None);
    }
    let constructor = realm.read_property(&value, &JsString::from("constructor"))?;
    if !matches!(constructor, Value::Object(_)) {
        return Ok(None);
    }
    let name = realm.read_property(&constructor, &JsString::from("name"))?;
    Ok(if let Value::String(name) = name {
        Some(name)
    } else {
        None
    })
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
        Error::InvalidObject(_) | Error::Host { .. } => {
            Some(Outcome::SetupFailure(error.to_string()))
        }
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
