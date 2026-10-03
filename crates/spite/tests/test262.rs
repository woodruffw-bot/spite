//! A pinned lexical smoke suite, not a general Test262 runner.

use spite::{DiagnosticKind, Realm, parse_script};
use std::{fs, path::Path};

#[test]
fn pinned_test262_raw_scripts() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/test262");
    let manifest = fs::read_to_string(root.join("manifest.tsv")).unwrap();
    let mut passed = 0;
    for line in manifest
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
    {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 3, "invalid manifest entry");
        let expectation = fields[0];
        let path = fields[2];
        // These fixtures are component regressions in spite-parser.
        // They are not counted as passing Script evaluations here.
        if matches!(
            expectation,
            "identifier-tokens" | "identifier-error" | "parser-pass"
        ) {
            continue;
        }
        let source = fs::read_to_string(root.join("upstream").join(path)).unwrap();
        assert!(
            source.contains("\nflags: [raw]\n"),
            "{path}: only reviewed raw fixtures are supported"
        );
        match expectation {
            "raw-pass" => {
                assert!(
                    !source.contains("\nnegative:\n"),
                    "{path}: unexpected negative metadata"
                );
                // Raw sources run once, unchanged, without harness injection.
                Realm::default()
                    .eval(&source)
                    .unwrap_or_else(|e| panic!("{path}: {e}"));
            }
            "raw-syntax-error" => {
                assert!(source.contains("\nnegative:\n  phase: parse\n  type: SyntaxError\n"));
                let error = parse_script(&source).expect_err(path);
                assert_eq!(
                    error.kind,
                    DiagnosticKind::Syntax,
                    "{path}: unsupported features are not passes"
                );
                // These reviewed fixtures all reject a hashbang away from byte 0.
                // Match the actual cause, not an unrelated incomplete grammar rule.
                assert_eq!(
                    &source[error.span.start..error.span.end],
                    "#",
                    "{path}: wrong rejection point"
                );
            }
            _ => panic!("{path}: unsupported manifest expectation {expectation}"),
        }
        passed += 1;
    }
    assert_eq!(passed, 11, "the reviewed fixture inventory changed");
}
