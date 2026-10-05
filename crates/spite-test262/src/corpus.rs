use crate::{CaseResult, MetadataError, MetadataErrorKind, Outcome, ParseExpectation, Runner};
use spite_core::Span;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
};

/// Results for one reviewed upstream source file.
#[derive(Clone, Debug)]
pub struct CorpusTest {
    /// Original upstream-relative path.
    pub path: String,
    /// Metadata error, or the individual execution variants.
    pub result: Result<Vec<CaseResult>, MetadataError>,
}

/// Results for the explicitly selected, revision-pinned regression corpus.
#[derive(Clone, Debug)]
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
    let revision = fs::read_to_string(root.join("REVISION"))?.trim().to_owned();
    if revision.len() != 40
        || !revision
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(invalid("REVISION must be a full lowercase commit SHA"));
    }
    let manifest = fs::read_to_string(root.join("runner.tsv"))?;
    let mut seen = BTreeSet::new();
    let mut tests = Vec::new();
    for line in manifest
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
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
        let source = fs::read_to_string(root.join("upstream").join(path))?;
        let review = if fields[1..] == ["-", "-", "-"] {
            None
        } else {
            let start = fields[1]
                .parse::<usize>()
                .map_err(|_| invalid("invalid review start"))?;
            let end = fields[2]
                .parse::<usize>()
                .map_err(|_| invalid("invalid review end"))?;
            if start >= end
                || !source.is_char_boundary(start)
                || !source.is_char_boundary(end)
                || fields[3].is_empty()
                || fields[3] == "-"
            {
                return Err(invalid("invalid reviewed source span or diagnostic"));
            }
            Some(ParseExpectation {
                span: Span::new(start, end),
                message: fields[3].into(),
            })
        };
        let result = Runner::default().run(&source, review.as_ref(), |name| {
            fs::read_to_string(root.join("upstream/harness").join(name))
                .map_err(|error| error.to_string())
        });
        tests.push(CorpusTest {
            path: path.into(),
            result,
        });
    }
    if tests.is_empty() {
        return Err(invalid("runner inventory is empty"));
    }
    Ok(CorpusReport { revision, tests })
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
