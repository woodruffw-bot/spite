//! Complete pinned ECMAScript conformance corpus.

use spite_test262::run_corpus_with_workers;
use std::num::NonZeroUsize;
use std::path::Path;

#[test]
fn pinned_corpus_runs_all_reviewed_variants() {
    let report = run_corpus_with_workers(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/test262"),
        NonZeroUsize::new(3).unwrap(),
    )
    .unwrap();
    assert!(report.is_success(), "{report:#?}");
    assert_eq!(report.revision, "7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd");
    assert_eq!(report.tests.len(), 42889);
    assert_eq!(report.counts()["passed"], 82574);
    assert!(
        report
            .counts()
            .iter()
            .all(|(name, count)| *name == "passed" || *count == 0)
    );
}
