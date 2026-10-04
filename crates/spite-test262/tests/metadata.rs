//! Execution metadata must not silently change the modes or setup of a test.

use spite_test262::{Metadata, MetadataErrorKind, Mode, Negative, Phase};
use std::{borrow::Cow, fs, path::Path};

fn source(frontmatter: &str) -> String {
    format!("/*---\n{frontmatter}\n---*/\n1;")
}
fn metadata(frontmatter: &str) -> Metadata {
    Metadata::parse(&source(frontmatter)).unwrap()
}

#[test]
fn flags_plan_all_required_execution_variants() {
    for (flags, modes) in [
        ("[]", vec![Mode::Script, Mode::StrictScript]),
        (
            "[generated, non-deterministic]",
            vec![Mode::Script, Mode::StrictScript],
        ),
        ("[noStrict]", vec![Mode::Script]),
        ("[onlyStrict]", vec![Mode::StrictScript]),
        ("[raw]", vec![Mode::Script]),
        ("[module]", vec![Mode::Module]),
        ("[module, raw]", vec![Mode::Module]),
    ] {
        assert_eq!(metadata(&format!("flags: {flags}")).modes(), modes);
    }
    assert_eq!(
        metadata("description: default modes").modes(),
        [Mode::Script, Mode::StrictScript]
    );
}

#[test]
fn raw_and_module_sources_preserve_exact_bytes() {
    let original = "#! hashbang\r\n/*---\nflags: [raw]\n---*/\n'usestrict';\u{2028}1;";
    let meta = Metadata::parse(original).unwrap();
    let mode = meta.modes()[0];
    assert!(matches!(mode.prepare_source(original), Cow::Borrowed(_)));
    assert_eq!(
        mode.prepare_source(original).as_bytes(),
        original.as_bytes()
    );
    assert_eq!(
        Mode::Module.prepare_source(original).as_bytes(),
        original.as_bytes()
    );
    assert_eq!(
        Mode::StrictScript.prepare_source(original),
        format!("\"use strict\";\n{original}")
    );
    // A raw Script can contain its own strict directive; the runner never removes it.
    let raw = "'use strict'; /*---\nflags: [raw]\n---*/";
    assert_eq!(meta.modes()[0].prepare_source(raw), raw);
}

#[test]
fn harness_files_keep_default_async_and_declared_order() {
    let meta = metadata("flags: [async]\nincludes: [first.js, second.js, first.js]");
    assert_eq!(
        meta.harness_files(),
        [
            "assert.js",
            "sta.js",
            "doneprintHandle.js",
            "first.js",
            "second.js",
            "first.js"
        ]
    );
    assert_eq!(
        metadata("includes:\n  - first.js\n  - 'second.js'").harness_files(),
        ["assert.js", "sta.js", "first.js", "second.js"]
    );
    assert!(
        metadata("flags: [raw]\nincludes: [ignored.js]")
            .harness_files()
            .is_empty()
    );
}

#[test]
fn negative_metadata_retains_phase_and_type() {
    for (phase, expected, flags) in [
        ("parse", Phase::Parse, "[]"),
        ("runtime", Phase::Runtime, "[]"),
        ("resolution", Phase::Resolution, "[module]"),
    ] {
        let meta = metadata(&format!(
            "flags: {flags}\nnegative:\n  type: SyntaxError\n  phase: {phase}"
        ));
        assert_eq!(
            meta.negative,
            Some(Negative {
                phase: expected,
                error_type: "SyntaxError".into()
            })
        );
    }
}

#[test]
fn descriptive_blocks_cannot_inject_execution_metadata() {
    let meta = metadata(
        "description: >\n  A test with fake metadata in its description.\n  flags: [onlyStrict]\ninfo: |\n  negative:\n    phase: runtime\n    type: TypeError\nfeatures: ['BigInt', \"numeric-separator-literal\"]\nlocale:\n  - en-US\n  - ar",
    );
    assert_eq!(meta.modes(), [Mode::Script, Mode::StrictScript]);
    assert!(meta.negative.is_none());
    assert_eq!(meta.features, ["BigInt", "numeric-separator-literal"]);
    assert_eq!(meta.locales, ["en-US", "ar"]);
    insta::assert_debug_snapshot!(meta);
}

#[test]
fn contradictory_duplicate_and_incomplete_metadata_is_invalid() {
    for frontmatter in [
        "flags: [onlyStrict, noStrict]",
        "flags: [raw, onlyStrict]",
        "flags: [module, noStrict]",
        "flags: [module, onlyStrict]",
        "flags: [CanBlockIsTrue, CanBlockIsFalse]",
        "flags: [raw, raw]",
        "flags: []\nflags: []",
        "negative:\n  phase: parse",
        "negative:\n  type: SyntaxError",
        "negative:\n  phase: parse\n  phase: parse\n  type: SyntaxError",
        "negative:\n  phase: resolution\n  type: SyntaxError",
        "negative:\n  phase: nonsense\n  type: SyntaxError",
    ] {
        assert_eq!(
            Metadata::parse(&source(frontmatter)).unwrap_err().kind,
            MetadataErrorKind::Invalid,
            "{frontmatter}"
        );
    }
    for original in ["1;", "/*---\nflags: []"] {
        assert_eq!(
            Metadata::parse(original).unwrap_err().kind,
            MetadataErrorKind::Invalid
        );
    }
}

#[test]
fn unknown_semantics_and_unsupported_yaml_never_default_to_a_normal_test() {
    for frontmatter in [
        "flags: [newFlag]",
        "newField: value",
        "flags: &flags [raw]",
        "flags: *flags",
        "flags: !!seq [raw]",
        "flags: [\"r\\u0061w\"]",
        "flags: [raw] # trailing comment",
        "negative: {phase: parse, type: SyntaxError}",
        "features: [true]",
        "features: [123]",
        "features: [1e3]",
        "features: [2026-01-01]",
        "features: [-1.5]",
        "features: [.NaN]",
        "features: [-.Inf]",
        "features: [YES]",
        "features: [Off]",
        "includes:\n  - one.js\n    - two.js",
        "includes:\n\t- one.js",
        "negative:\n  phase: parse\n    type: SyntaxError",
        "negative:\n  phase: parse\n  type: SyntaxError\n  extra: value",
        "flags: [raw]\n  nested: ignored",
        "description: text\n  flags: [raw]",
    ] {
        assert_eq!(
            Metadata::parse(&source(frontmatter)).unwrap_err().kind,
            MetadataErrorKind::Unsupported,
            "{frontmatter}"
        );
    }
    assert_eq!(metadata("features: ['true']").features, ["true"]);
    assert_eq!(
        metadata("features: ['1e3', '2026-01-01', '.NaN', 'YES']").features,
        ["1e3", "2026-01-01", ".NaN", "YES"]
    );
}

#[test]
fn frontmatter_size_limits_are_opt_in() {
    let source = source(&format!("description: {}", "a".repeat(65_536)));
    assert!(Metadata::parse(&source).is_ok());
    assert_eq!(
        Metadata::parse_with_frontmatter_limit(&source, 65_536)
            .unwrap_err()
            .kind,
        MetadataErrorKind::Limit
    );
}

#[test]
fn all_pinned_fixture_metadata_is_read_without_rewriting_sources() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/test262");
    let manifest = fs::read_to_string(root.join("manifest.tsv")).unwrap();
    let mut count = 0;
    let mut harness_count = 0;
    for line in manifest
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let fields: Vec<_> = line.split('\t').collect();
        if fields[0] == "harness" {
            assert!(fields[2].starts_with("harness/"));
            harness_count += 1;
            continue;
        }
        let original = fs::read_to_string(root.join("upstream").join(fields[2])).unwrap();
        let meta =
            Metadata::parse(&original).unwrap_or_else(|error| panic!("{}: {error}", fields[2]));
        if fields[0].starts_with("raw-") {
            assert_eq!(meta.modes(), [Mode::Script]);
            assert!(meta.has_flag("raw"));
        }
        if fields[0] == "script-pass" {
            assert!(!meta.has_flag("raw"));
            assert!(meta.negative.is_none());
        }
        if fields[0] == "raw-syntax-error" {
            assert_eq!(
                meta.negative,
                Some(Negative {
                    phase: Phase::Parse,
                    error_type: "SyntaxError".into()
                })
            );
        }
        count += 1;
    }
    assert_eq!(count, 2105);
    assert_eq!(harness_count, 6);
}
