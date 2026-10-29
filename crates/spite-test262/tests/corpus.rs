//! Pinned corpus inventory and command-line failure semantics.

use spite_test262::{Outcome, run_corpus, run_corpus_with_workers};
use std::{
    fs,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

fn pinned() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/test262")
}

fn empty_harness(root: &Path) {
    fs::create_dir_all(root.join("upstream/harness")).unwrap();
    for name in ["assert.js", "sta.js"] {
        fs::write(root.join("upstream/harness").join(name), "").unwrap();
    }
}

#[test]
fn relative_module_sources_share_normalized_root_identity_and_run_in_serial_and_parallel_corpora() {
    let corpus = TemporaryCorpus::new(
        "/*---\nflags: [module]\n---*/\nimport * as self from './sub/../example.js';import{meta,getMeta}from'./dep.js';export function own(){return import.meta};if(self.own!==own||meta===import.meta||meta!==getMeta())throw 1",
        "test/example.js\t-\t-\t-\n",
    );
    empty_harness(&corpus.0);
    fs::write(
        corpus.0.join("upstream/test/dep.js"),
        "export const meta=import.meta;export function getMeta(){return import.meta}",
    )
    .unwrap();
    let serial = run_corpus(&corpus.0).unwrap();
    assert!(serial.is_success(), "{serial:#?}");
    assert_eq!(
        serial,
        run_corpus_with_workers(&corpus.0, NonZeroUsize::new(3).unwrap()).unwrap()
    );
}

#[test]
fn module_filesystem_failures_and_escape_attempts_never_satisfy_runtime_negatives() {
    for specifier in [
        "../outside.js",
        "/absolute.js",
        "C:/outside.js",
        "./missing.js",
    ] {
        let corpus = TemporaryCorpus::new(
            &format!(
                "/*---\nflags: [module]\nnegative:\n  phase: runtime\n  type: SyntaxError\n---*/\nimport {specifier:?}"
            ),
            "test/example.js\t-\t-\t-\n",
        );
        empty_harness(&corpus.0);
        let report = run_corpus(&corpus.0).unwrap();
        assert!(
            matches!(
                report.tests[0].result.as_ref().unwrap()[0].outcome,
                Outcome::SetupFailure(_)
            ),
            "{specifier}: {report:#?}"
        );
    }
    #[cfg(unix)]
    {
        let corpus = TemporaryCorpus::new(
            "/*---\nflags: [module]\n---*/\nimport'./link.js'",
            "test/example.js\t-\t-\t-\n",
        );
        empty_harness(&corpus.0);
        let outside = corpus.0.join("outside.js");
        fs::write(&outside, "throw 1").unwrap();
        std::os::unix::fs::symlink(outside, corpus.0.join("upstream/test/link.js")).unwrap();
        let report = run_corpus(&corpus.0).unwrap();
        assert!(
            matches!(&report.tests[0].result.as_ref().unwrap()[0].outcome,Outcome::SetupFailure(message) if message.contains("outside corpus"))
        );
    }
}

#[test]
fn dependency_body_exceptions_retain_the_runtime_phase_and_original_constructor() {
    let corpus = TemporaryCorpus::new(
        "/*---\nflags: [module]\nnegative:\n  phase: runtime\n  type: TypeError\n---*/\nimport'./dep.js'",
        "test/example.js\t-\t-\t-\n",
    );
    empty_harness(&corpus.0);
    fs::write(
        corpus.0.join("upstream/test/dep.js"),
        "throw new TypeError()",
    )
    .unwrap();
    let report = run_corpus(&corpus.0).unwrap();
    assert!(report.is_success(), "{report:#?}");
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
    let report = run_corpus_with_workers(&pinned(), NonZeroUsize::new(3).unwrap()).unwrap();
    assert!(report.is_success(), "{report:#?}");
    assert_eq!(report.revision, "7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd");
    assert_eq!(report.tests.len(), 10723);
    assert_eq!(report.counts()["passed"], 20527);
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
