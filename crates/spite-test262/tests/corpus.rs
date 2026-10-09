//! Pinned corpus inventory and command-line failure semantics.

use spite_test262::run_corpus;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

fn pinned() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/test262")
}
fn command(root: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_spite-test262"))
        .arg(root)
        .output()
        .unwrap()
}

struct TemporaryCorpus(PathBuf);
impl TemporaryCorpus {
    fn new(source: &str, row: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "spite-corpus-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::create_dir_all(root.join("upstream/test")).unwrap();
        fs::write(root.join("REVISION"), "a".repeat(40)).unwrap();
        fs::write(root.join("runner.tsv"), row).unwrap();
        fs::write(root.join("upstream/test/example.js"), source).unwrap();
        Self(root)
    }
}
impl Drop for TemporaryCorpus {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn pinned_corpus_runs_all_reviewed_variants() {
    let report = run_corpus(&pinned()).unwrap();
    assert!(report.is_success(), "{report:#?}");
    assert_eq!(report.revision, "7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd");
    assert_eq!(report.tests.len(), 7306);
    assert_eq!(report.counts()["passed"], 14090);
    assert!(
        report
            .counts()
            .iter()
            .all(|(name, count)| *name == "passed" || *count == 0)
    );
}

#[test]
fn command_reports_successful_variants_and_counts() {
    // The pinned test above runs the entire corpus, including in CI's optimized
    // conformance profile. A minimal corpus exercises CLI formatting/exit behavior
    // without a duplicate execution of every exhaustive upstream loop.
    let corpus = TemporaryCorpus::new(
        "/*---\nflags: [raw]\n---*/\n1",
        "test/example.js\t-\t-\t-\n",
    );
    let output = command(&corpus.0);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("1 source files"));
    assert!(stdout.contains("passed=1"));
    assert!(stdout.contains("unverified=0"));
    assert!(output.stderr.is_empty());
}

#[test]
fn metadata_failures_are_reported_and_fail_the_gate() {
    let corpus = TemporaryCorpus::new(
        "/*---\nflags: [unknownFlag]\n---*/\n1",
        "test/example.js\t-\t-\t-\n",
    );
    let report = run_corpus(&corpus.0).unwrap();
    assert!(!report.is_success());
    assert_eq!(report.counts()["metadata-unsupported"], 1);
    assert_eq!(report.counts()["passed"], 0);
    let output = command(&corpus.0);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("metadata-unsupported=1")
    );
}

#[test]
fn missing_or_mismatched_reviews_never_pass() {
    let source = "/*---\nflags: [raw]\nnegative:\n  phase: parse\n  type: SyntaxError\n---*/\n@";
    let corpus = TemporaryCorpus::new(source, "test/example.js\t-\t-\t-\n");
    assert_eq!(run_corpus(&corpus.0).unwrap().counts()["unverified"], 1);
    assert_eq!(command(&corpus.0).status.code(), Some(1));
    fs::write(
        corpus.0.join("runner.tsv"),
        "test/example.js\t0\t1\tunexpected character\n",
    )
    .unwrap();
    assert_eq!(run_corpus(&corpus.0).unwrap().counts()["failed"], 1);
    assert_eq!(command(&corpus.0).status.code(), Some(1));
}

#[test]
fn corrupt_manifests_and_missing_sources_abort_instead_of_skipping() {
    let corpus = TemporaryCorpus::new(
        "/*---\nflags: [raw]\n---*/\n1",
        "test/example.js\t-\t-\t-\n",
    );
    for row in [
        "",
        "test/example.js\t-\n",
        "test/../secret.js\t-\t-\t-\n",
        "test/example_FIXTURE.js\t-\t-\t-\n",
        "test/missing.js\t-\t-\t-\n",
        "test/example.js\t99999\t100000\tbad\n",
        "test/example.js\t-\t-\t-\ntest/example.js\t-\t-\t-\n",
    ] {
        fs::write(corpus.0.join("runner.tsv"), row).unwrap();
        assert!(run_corpus(&corpus.0).is_err(), "{row}");
        assert_eq!(command(&corpus.0).status.code(), Some(2));
    }
    fs::write(corpus.0.join("REVISION"), "main").unwrap();
    assert!(run_corpus(&corpus.0).is_err());
}
