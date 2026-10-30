use crate::{CaseResult, MetadataError, MetadataErrorKind, Outcome, ParseExpectation, Runner};
use spite_core::Span;
mod modules;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    num::NonZeroUsize,
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
    thread,
};

/// Results for one reviewed upstream source file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorpusTest {
    /// Original upstream-relative path.
    pub path: String,
    /// Metadata error, or the individual execution variants.
    pub result: Result<Vec<CaseResult>, MetadataError>,
}

/// Results for the explicitly selected, revision-pinned regression corpus.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorpusReport {
    /// The declared upstream revision; byte hashes are verified separately in CI.
    pub revision: String,
    /// Every selected test, including all non-passing outcomes.
    pub tests: Vec<CorpusTest>,
}

impl CorpusReport {
    /// Returns whether a nonempty corpus ran with every variant passing.
    pub fn is_success(&self) -> bool {
        !self.tests.is_empty()
            && self.tests.iter().all(|test| {
                test.result.as_ref().is_ok_and(|cases| {
                    !cases.is_empty() && cases.iter().all(|case| case.outcome == Outcome::Passed)
                })
            })
    }

    /// Counts execution variants and metadata failures in separate categories.
    pub fn counts(&self) -> BTreeMap<&'static str, usize> {
        let mut counts: BTreeMap<_, _> = [
            "passed",
            "failed",
            "unsupported",
            "limit",
            "unverified",
            "setup-failure",
            "metadata-invalid",
            "metadata-unsupported",
            "metadata-limit",
        ]
        .into_iter()
        .map(|name| (name, 0))
        .collect();
        for test in &self.tests {
            match &test.result {
                Ok(cases) => {
                    for case in cases {
                        let category = match case.outcome {
                            Outcome::Passed => "passed",
                            Outcome::Failed { .. } => "failed",
                            Outcome::Unsupported { .. } => "unsupported",
                            Outcome::Limit { .. } => "limit",
                            Outcome::UnverifiedSyntax(_) => "unverified",
                            Outcome::SetupFailure(_) => "setup-failure",
                        };
                        *counts.get_mut(category).expect("known category") += 1;
                    }
                }
                Err(error) => {
                    let category = match error.kind {
                        MetadataErrorKind::Invalid => "metadata-invalid",
                        MetadataErrorKind::Unsupported => "metadata-unsupported",
                        MetadataErrorKind::Limit => "metadata-limit",
                    };
                    *counts.get_mut(category).expect("known category") += 1;
                }
            }
        }
        counts
    }
}

/// Runs entries from `runner.tsv` using unchanged files beneath `upstream/`.
///
/// Each row has path, reviewed start/end byte offsets, and exact diagnostic text.
/// Executed rows, including runtime negatives, use `-` for all three review fields.
/// Runtime negatives match their original metadata's exception type and phase.
/// `REVISION` declares the pin;
/// run `tools/check-test262.py` to verify fixture bytes against the digest manifest.
/// Invalid manifests and I/O failures abort the corpus rather than skipping files.
pub fn run_corpus(root: &Path) -> io::Result<CorpusReport> {
    let (revision, manifest) = read_manifest(root)?;
    let mut seen = BTreeSet::new();
    let mut tests = Vec::new();
    for line in rows(&manifest) {
        let entry = parse_entry(line, &mut seen)?;
        tests.push(run_entry(root, &entry)?);
    }
    finish(revision, tests)
}

/// Runs the complete reviewed corpus with an explicitly selected worker count.
///
/// Each worker constructs and drops its own runner and execution realms. Reports
/// retain manifest order and the same individual modes and outcome categories as
/// [`run_corpus`]. Workers use ordinary native thread stacks and runtime defaults;
/// this does not impose JavaScript resource quotas. A worker panic propagates.
///
/// One worker uses [`run_corpus`] directly. With multiple workers, inventory
/// paths are validated before execution, so an invalid later row takes precedence
/// over an earlier entry's I/O or review error. After inventory validation, the
/// first entry error in manifest order is returned. Thread-creation failures
/// return an I/O error after every successfully started worker has joined.
pub fn run_corpus_with_workers(root: &Path, workers: NonZeroUsize) -> io::Result<CorpusReport> {
    if workers.get() == 1 {
        return run_corpus(root);
    }
    let (revision, manifest) = read_manifest(root)?;
    let mut seen = BTreeSet::new();
    let entries = rows(&manifest)
        .map(|line| parse_entry(line, &mut seen))
        .collect::<io::Result<Vec<_>>>()?;
    let next = AtomicUsize::new(0);
    let mut results = thread::scope(|scope| {
        let mut handles = Vec::new();
        let mut spawn_error = None;
        for _ in 0..workers.get().min(entries.len()) {
            match thread::Builder::new().spawn_scoped(scope, || {
                let mut results = Vec::new();
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(entry) = entries.get(index) else {
                        return results;
                    };
                    // Parsing and all non-Send realm state stay on this worker.
                    results.push((index, run_entry(root, entry)));
                }
            }) {
                Ok(handle) => handles.push(handle),
                Err(error) => {
                    spawn_error = Some(error);
                    break;
                }
            }
        }
        let mut results = Vec::new();
        for handle in handles {
            match handle.join() {
                Ok(rows) => results.extend(rows),
                Err(payload) => std::panic::resume_unwind(payload),
            }
        }
        match spawn_error {
            Some(error) => Err(error),
            None => Ok(results),
        }
    })?;
    results.sort_unstable_by_key(|(index, _)| *index);
    let tests = results
        .into_iter()
        .map(|(_, result)| result)
        .collect::<io::Result<Vec<_>>>()?;
    finish(revision, tests)
}

fn read_manifest(root: &Path) -> io::Result<(String, String)> {
    let revision = fs::read_to_string(root.join("REVISION"))?.trim().to_owned();
    if revision.len() != 40
        || !revision
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(invalid("REVISION must be a full lowercase commit SHA"));
    }
    let manifest = fs::read_to_string(root.join("runner.tsv"))?;
    Ok((revision, manifest))
}

fn rows(manifest: &str) -> impl Iterator<Item = &str> {
    manifest
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}

struct Entry<'a> {
    path: &'a str,
    review: [&'a str; 3],
}

fn parse_entry<'a>(line: &'a str, seen: &mut BTreeSet<&'a str>) -> io::Result<Entry<'a>> {
    let fields: Vec<_> = line.split('\t').collect();
    if fields.len() != 4 {
        return Err(invalid("runner row must contain four tab-separated fields"));
    }
    let path = fields[0];
    if !path.starts_with("test/")
        || !path.ends_with(".js")
        || path.contains('\\')
        || path.contains("_FIXTURE")
        || path.split('/').any(|part| matches!(part, "" | "." | ".."))
        || !seen.insert(path)
    {
        return Err(invalid(format!(
            "unsafe, duplicate, or non-test path: {path}"
        )));
    }
    Ok(Entry {
        path,
        review: [fields[1], fields[2], fields[3]],
    })
}

fn run_entry(root: &Path, entry: &Entry<'_>) -> io::Result<CorpusTest> {
    let source = fs::read_to_string(root.join("upstream").join(entry.path))?;
    let review = if entry.review == ["-", "-", "-"] {
        None
    } else {
        let start = entry.review[0]
            .parse::<usize>()
            .map_err(|_| invalid("invalid review start"))?;
        let end = entry.review[1]
            .parse::<usize>()
            .map_err(|_| invalid("invalid review end"))?;
        if start > end
            || (start == end && end != source.len())
            || !source.is_char_boundary(start)
            || !source.is_char_boundary(end)
            || entry.review[2].is_empty()
            || entry.review[2] == "-"
        {
            return Err(invalid("invalid reviewed source span or diagnostic"));
        }
        Some(ParseExpectation {
            span: Span::new(start, end),
            message: entry.review[2].into(),
        })
    };
    let result = Runner::default().run_with_module_loader(
        &source,
        review.as_ref(),
        entry.path,
        |name| {
            fs::read_to_string(root.join("upstream/harness").join(name))
                .map_err(|error| error.to_string())
        },
        |referrer, request| modules::load_source(root, referrer, request),
    );
    Ok(CorpusTest {
        path: entry.path.into(),
        result,
    })
}

fn finish(revision: String, tests: Vec<CorpusTest>) -> io::Result<CorpusReport> {
    if tests.is_empty() {
        return Err(invalid("runner inventory is empty"));
    }
    Ok(CorpusReport { revision, tests })
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
