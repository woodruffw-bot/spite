# Data generation

Run generators from the repository root with `cargo xtask generate NAME [ARGS]`.
Outputs are checked in. Ordinary builds and tests do not download source data
or require Python or Node.js.

## Reproducible generators

These generators use Python 3 and accept `--check` to compare output without
writing:

| Name | Output |
| --- | --- |
| `unicode` | Identifier ranges from DerivedCoreProperties.txt |
| `case-mappings` | Casing/folding tables and regress legacy case expansions |
| `normalization` | Normalization tables and the unchanged normalization fixture |
| `regexp-properties` | Exact RegExp property aliases |
| `number-radix` | Binary64 radix-conversion expectations |
| `number-sum` | Correctly rounded summation expectations |

```sh
cargo xtask generate all --check
```

`all` runs the Unicode and numeric generators in the table. Omit `--check` to
regenerate their outputs.

Unicode generators verify source checksums before reading data. They download
pinned inputs unless given a local source path; use each generator's `--help`
for arguments. Versions, sources, and licenses are described in the
[Unicode data guide](../../../third-party/unicode/README.md).

Numeric generators calculate expectations with exact rational arithmetic.
Keep those reference calculations independent of the engine's algorithms.

## Integration-test module lists

The `test-modules` generator is implemented in Rust and needs no Python:

```sh
cargo xtask generate test-modules
cargo xtask generate test-modules --check
```

Run it after adding or removing parser or runtime case files. It is separate
from `all`; `cargo xtask check` verifies both.

## RegExp comparison fixtures

Normal tests compare against a checked-in table and need no external engine.
To deliberately capture fresh reference results, install Node.js 24.19 and run
the full runtime integration target. For example, in a POSIX shell:

```sh
capture_dir=$(mktemp -d)
SPITE_REGEXP_ORACLE_CAPTURE="$capture_dir/results.tsv" \
  cargo test -p spite-runtime --test integration --locked
cargo xtask generate regexp-results "$capture_dir/results.tsv"
```

Capture mode still compares spite's actual result with the reference result.
It does not substitute success. The generator replaces the table using the
supplied captures; use the full target or supply all capture files to avoid
dropping cases.

Review the diff against the specification before committing. Node.js is a
comparison source, not the authority for edition-specific behavior.
`regexp-results` requires Python 3, is excluded from `all`, and has no
`--check` mode.
