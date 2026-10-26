//! Explicit parallel execution preserves complete serial reports and failures.

use spite_test262::{Mode, Outcome, Stage, run_corpus, run_corpus_with_workers};
use std::{
    fs,
    num::NonZeroUsize,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct TemporaryCorpus(PathBuf);

impl TemporaryCorpus {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "spite-parallel-corpus-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::create_dir_all(root.join("upstream/test")).unwrap();
        fs::create_dir_all(root.join("upstream/harness")).unwrap();
        fs::write(root.join("REVISION"), "a".repeat(40)).unwrap();
        for name in ["assert.js", "sta.js"] {
            fs::write(root.join("upstream/harness").join(name), "").unwrap();
        }
        Self(root)
    }

    fn source(&self, name: &str, metadata: &str, body: &str) -> String {
        let source = format!("/*---\n{metadata}\n---*/\n{body}");
        fs::write(self.0.join("upstream/test").join(name), &source).unwrap();
        source
    }

    fn manifest(&self, manifest: &str) {
        fs::write(self.0.join("runner.tsv"), manifest).unwrap();
    }
}

impl Drop for TemporaryCorpus {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn parallel_reports_preserve_order_modes_isolation_and_non_passing_categories() {
    let corpus = TemporaryCorpus::new();
    corpus.source("z-first.js", "", "Object.prototype.shared = 1;");
    corpus.source(
        "a-second.js",
        "",
        "if (Object.prototype.shared !== undefined) throw 'realm leaked';",
    );
    corpus.source(
        "strict.js",
        "negative:\n  phase: runtime\n  type: TypeError",
        "(function () { return this; })().missing;",
    );
    let negative = corpus.source(
        "parse.js",
        "negative:\n  phase: parse\n  type: SyntaxError",
        "@",
    );
    corpus.source("module.js", "flags: [module]", "1;");
    corpus.source("host.js", "flags: [raw]", "print");
    corpus.source("metadata.js", "flags: [unknownFlag]", "1;");
    corpus.source("setup.js", "includes: [missing.js]", "1;");
    corpus.source(
        "unverified.js",
        "negative:\n  phase: parse\n  type: SyntaxError",
        "@",
    );
    corpus.source("raw.js", "flags: [raw]", "1;");
    let start = negative.rfind('@').unwrap();
    corpus.manifest(&format!(
        "test/z-first.js\t-\t-\t-\n\
         test/a-second.js\t-\t-\t-\n\
         test/strict.js\t-\t-\t-\n\
         test/parse.js\t{start}\t{}\tunexpected character\n\
         test/module.js\t-\t-\t-\n\
         test/host.js\t-\t-\t-\n\
         test/metadata.js\t-\t-\t-\n\
         test/setup.js\t-\t-\t-\n\
         test/unverified.js\t-\t-\t-\n\
         test/raw.js\t-\t-\t-\n",
        start + 1
    ));

    let serial = run_corpus(&corpus.0).unwrap();
    assert_eq!(serial.tests.len(), 10);
    let counts = serial.counts();
    for (category, count) in [
        ("passed", 9),
        ("failed", 1),
        ("unsupported", 1),
        ("metadata-unsupported", 1),
        ("setup-failure", 2),
        ("unverified", 2),
    ] {
        assert_eq!(counts[category], count, "{category}: {serial:#?}");
    }
    let strict = serial.tests[2].result.as_ref().unwrap();
    assert_eq!(strict[0].mode, Mode::Script);
    assert!(matches!(
        strict[0].outcome,
        Outcome::Failed {
            stage: Stage::Runtime,
            ..
        }
    ));
    assert_eq!(strict[1].mode, Mode::StrictScript);
    assert_eq!(strict[1].outcome, Outcome::Passed);
    let module = &serial.tests[4].result.as_ref().unwrap()[0];
    assert_eq!(module.mode, Mode::Module);
    assert_eq!(module.outcome, Outcome::Passed);
    assert!(matches!(
        serial.tests[5].result.as_ref().unwrap()[0].outcome,
        Outcome::Unsupported {
            stage: Stage::Runtime,
            ..
        }
    ));
    for workers in [1, 2, 3] {
        let parallel =
            run_corpus_with_workers(&corpus.0, NonZeroUsize::new(workers).unwrap()).unwrap();
        assert_eq!(parallel, serial, "{workers} workers");
    }
}

#[test]
fn parallel_manifest_and_entry_errors_never_produce_partial_success() {
    let corpus = TemporaryCorpus::new();
    corpus.source("valid.js", "flags: [raw]", "1;");
    for manifest in [
        "",
        "test/valid.js\t-\t-\t-\ntest/valid.js\t-\t-\t-\n",
        "test/../valid.js\t-\t-\t-\n",
        "test/valid.js\t-\n",
        "test/missing.js\t-\t-\t-\n",
        "test/valid.js\tbad\t1\tmessage\ntest/valid.js\t-\t-\t-\n",
    ] {
        corpus.manifest(manifest);
        let serial = run_corpus(&corpus.0).unwrap_err();
        for workers in [1, 2] {
            let parallel = run_corpus_with_workers(&corpus.0, NonZeroUsize::new(workers).unwrap())
                .unwrap_err();
            assert_eq!(parallel.kind(), serial.kind(), "{manifest}");
        }
    }

    // Entry execution can finish out of order; the earliest manifest error wins.
    corpus.source("second.js", "flags: [raw]", "1;");
    corpus.manifest(
        "test/valid.js\tbad\t1\tfirst error\n\
         test/second.js\t0\tbad\tsecond error\n",
    );
    let serial = run_corpus(&corpus.0).unwrap_err();
    let parallel = run_corpus_with_workers(&corpus.0, NonZeroUsize::new(2).unwrap()).unwrap_err();
    assert_eq!(parallel.to_string(), serial.to_string());
    assert_eq!(parallel.to_string(), "invalid review start");
}

#[test]
fn multiple_workers_validate_inventory_before_reading_entries() {
    let corpus = TemporaryCorpus::new();
    corpus.manifest(
        "test/missing.js\t-\t-\t-\n\
         test/missing.js\t-\t-\t-\n",
    );
    let serial = run_corpus(&corpus.0).unwrap_err();
    assert_eq!(serial.kind(), std::io::ErrorKind::NotFound);
    let one_worker = run_corpus_with_workers(&corpus.0, NonZeroUsize::new(1).unwrap()).unwrap_err();
    assert_eq!(one_worker.kind(), serial.kind());
    assert_eq!(one_worker.to_string(), serial.to_string());
    let parallel = run_corpus_with_workers(&corpus.0, NonZeroUsize::new(2).unwrap()).unwrap_err();
    assert_eq!(parallel.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(
        parallel.to_string(),
        "unsafe, duplicate, or non-test path: test/missing.js"
    );
}
