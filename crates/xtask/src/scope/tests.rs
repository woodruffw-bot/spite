//! Tampering controls for provenance and scope accounting.

use super::Object;
use super::Tree;
use super::check;
use crate::integrity;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

const RETAINED: &str = "test/built-ins/retained.js";
const PENDING: &str = "test/built-ins/RegExp/property-escapes/pending.js";
const STAGING: &str = "test/staging/pending.js";

struct Corpus(PathBuf);

impl Corpus {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("spite-scope-{}-{id}", std::process::id()));
        fs::create_dir(&root).expect("create isolated test directory");
        let corpus = Self(root);
        let mut tree = Tree::default();
        let mut rows = String::new();
        let mut manifest = String::new();
        for (path, data, retained) in [
            (RETAINED, "retained;\n", true),
            (PENDING, "pending;\n", false),
            ("test/built-ins/helper_FIXTURE.js", "helper;\n", true),
            ("test/annexB/optional.js", "optional;\n", false),
            (STAGING, "pending;\n", false),
            ("harness/assert.js", "assert;\n", true),
        ] {
            let id = integrity::object_id("blob", data.as_bytes());
            tree.insert(
                &integrity::path_parts(path).unwrap(),
                Object {
                    mode: "100644".to_owned(),
                    id: id.clone(),
                },
            )
            .unwrap();
            rows.push_str(&format!("100644\t{id}\t{path}\n"));
            corpus.write(&format!("upstream/{path}"), data);
            if retained {
                // Scope checking deliberately does not trust the local SHA-256.
                manifest.push_str(&format!("script-pass\tunused-local-checksum\t{path}\n"));
            }
        }
        corpus.write("audit/upstream-tree.tsv", &rows);
        let commit = format!(
            "tree {}\nauthor Test <test@example.com> 0 +0000\ncommitter Test <test@example.com> 0 +0000\n\nTest\n",
            tree.id().unwrap()
        );
        corpus.write("audit/upstream-commit.txt", &commit);
        corpus.write(
            "REVISION",
            &integrity::object_id("commit", commit.as_bytes()),
        );
        corpus.write("manifest.tsv", &manifest);
        corpus.write("runner.tsv", &format!("{RETAINED}\t-\t-\t-\n"));
        corpus.write("audit/scope.tsv", &format!("baseline-review\t{PENDING}\noptional-annex-b\ttest/annexB/optional.js\nstaging-review\t{STAGING}\n"));
        corpus
    }

    fn read(&self, path: &str) -> String {
        fs::read_to_string(self.0.join(path)).unwrap()
    }

    fn write(&self, path: &str, data: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, data).unwrap();
    }

    fn replace(&self, path: &str, from: &str, to: &str) {
        self.write(path, &self.read(path).replace(from, to));
    }

    fn rejects(&self, message: &str) {
        let error = check(&self.0)
            .expect_err("tampering must fail verification")
            .to_string();
        assert!(
            error.contains(message),
            "expected {message:?}, got {error:?}"
        );
    }
}

impl Drop for Corpus {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove isolated test directory");
    }
}

#[test]
fn complete_inventory_distinguishes_originals_from_support() {
    check(&Corpus::new().0).unwrap();
}

#[test]
fn commit_object_must_match_the_pin() {
    let corpus = Corpus::new();
    corpus.write("REVISION", &"0".repeat(40));
    corpus.rejects("commit object");
}

#[test]
fn omitted_tree_entries_cannot_change_the_denominator() {
    let corpus = Corpus::new();
    let rows = corpus.read("audit/upstream-tree.tsv");
    let rows: Vec<_> = rows
        .lines()
        .filter(|line| !line.contains("pending.js"))
        .collect();
    corpus.write("audit/upstream-tree.tsv", &rows.join("\n"));
    corpus.rejects("tree does not match");
}

#[test]
fn duplicate_tree_entries_fail() {
    let corpus = Corpus::new();
    let rows = corpus.read("audit/upstream-tree.tsv");
    corpus.write(
        "audit/upstream-tree.tsv",
        &format!("{rows}{}\n", rows.lines().next().unwrap()),
    );
    corpus.rejects("duplicate upstream");
}

#[test]
fn retained_bytes_are_checked_independently_of_manifest_hashes() {
    let corpus = Corpus::new();
    corpus.write(&format!("upstream/{RETAINED}"), "rewritten;\n");
    corpus.rejects("bytes differ from upstream");
}

#[test]
fn every_unretained_original_needs_an_explicit_entry() {
    let corpus = Corpus::new();
    corpus.write(
        "audit/scope.tsv",
        "optional-annex-b\ttest/annexB/optional.js\n",
    );
    corpus.rejects("account for every unretained");
}

#[test]
fn retained_and_scoped_must_be_disjoint() {
    let corpus = Corpus::new();
    corpus.write(
        "manifest.tsv",
        &format!(
            "{}script-pass\tunused\t{PENDING}\n",
            corpus.read("manifest.tsv")
        ),
    );
    corpus.write(
        "runner.tsv",
        &format!("{}{PENDING}\t-\t-\t-\n", corpus.read("runner.tsv")),
    );
    corpus.rejects("account for every unretained");
}

#[test]
fn scope_duplicates_and_inappropriate_categories_fail() {
    let corpus = Corpus::new();
    let original = corpus.read("audit/scope.tsv");
    for (text, error) in [
        (
            format!("{original}{}\n", original.lines().next().unwrap()),
            "duplicate scope",
        ),
        (
            original.replace("baseline-review", "optional-annex-b"),
            "invalid scope",
        ),
        (
            original.replace("baseline-review", "unknown"),
            "invalid scope",
        ),
    ] {
        corpus.write("audit/scope.tsv", &text);
        corpus.rejects(error);
    }
}

#[test]
fn only_reviewed_post_baseline_features_are_accepted() {
    let corpus = Corpus::new();
    corpus.replace(
        "audit/scope.tsv",
        "baseline-review",
        "outside-edition17:Temporal",
    );
    check(&corpus.0).unwrap();
    corpus.replace("audit/scope.tsv", "Temporal", "unknown");
    corpus.rejects("invalid scope");
}

#[test]
fn unicode_dispositions_need_complete_contradictory_witnesses() {
    let corpus = Corpus::new();
    corpus.replace(
        "audit/scope.tsv",
        "baseline-review",
        "unicode-version-difference",
    );
    corpus.rejects("Unicode witnesses must cover");
    corpus.write(
        "audit/unicode-witnesses.tsv",
        &format!("{PENDING}\tAlphabetic\t0558\tfalse\ttrue\n"),
    );
    check(&corpus.0).unwrap();
    corpus.replace("audit/unicode-witnesses.tsv", "false", "true");
    corpus.rejects("non-contradictory Unicode witness");
}

#[test]
fn closed_staging_entries_require_exact_pinned_source_reviews() {
    let corpus = Corpus::new();
    corpus.replace(
        "audit/scope.tsv",
        "staging-review",
        "outside-edition17:decorators",
    );
    corpus.rejects("require matching pinned source reviews");
    let id = integrity::object_id("blob", b"pending;\n");
    let review =
        format!("outside-edition17:decorators\t{STAGING}\t{id}\tRequires auto-accessor syntax.\n");
    corpus.write("audit/staging-reviewed.tsv", &review);
    check(&corpus.0).unwrap();
    for invalid in [
        review.replace(&id, &"0".repeat(40)),
        review.replace("decorators", "Temporal"),
        review.repeat(2),
    ] {
        corpus.write("audit/staging-reviewed.tsv", &invalid);
        corpus.rejects("invalid or unpinned staging review");
    }
}

#[test]
fn resource_receipts_cannot_omit_limits_or_claim_passing_credit() {
    let corpus = Corpus::new();
    corpus.replace(
        "audit/scope.tsv",
        "staging-review",
        "native-resource-constraint",
    );
    let id = integrity::object_id("blob", b"pending;\n");
    corpus.write(
        "audit/staging-reviewed.tsv",
        &format!("native-resource-constraint\t{STAGING}\t{id}\tNative allocation failure.\n"),
    );
    corpus.rejects("complete nonpassing receipts");
    let mut receipt = json!({
        "revision": corpus.read("REVISION"),
        "results": [{"path": STAGING, "git_blob": id, "runner": "defaults",
            "outcomes": [{"mode": "Script", "category": "limit", "stage": "Runtime", "message": "native capacity"}]}]
    });
    corpus.write("audit/resource-stress.json", &receipt.to_string());
    check(&corpus.0).unwrap();
    receipt["passing_credit"] = json!(1);
    corpus.write("audit/resource-stress.json", &receipt.to_string());
    corpus.rejects("cannot claim passing credit");
    receipt["passing_credit"] = json!(0);
    receipt["results"][0]["outcomes"][0]["category"] = json!("passed");
    corpus.write("audit/resource-stress.json", &receipt.to_string());
    corpus.rejects("nonpassing Limit outcomes");
}
