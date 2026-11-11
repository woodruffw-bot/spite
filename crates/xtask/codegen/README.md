# Data generation

Run generators through `cargo xtask generate NAME [ARGS]`. Generated files are
checked in; ordinary Cargo tests do not download data or require Python or Node.
Generators require Python 3 and accept `--check` to compare without writing.

| Name | Output and source |
| --- | --- |
| `test-modules` | Sorted Parser and Runtime integration module lists; implemented in Rust |
| `unicode` | Identifier tables from pinned Unicode DerivedCoreProperties.txt |
| `case-mappings` | Casing tables and regress legacy expansions from pinned Unicode data |
| `normalization` | Decomposition/composition tables and the pinned normalization test fixture |
| `regexp-properties` | RegExp property tables from pinned Unicode property files |
| `number-radix` | Binary64 radix-conversion expectations computed with exact rational arithmetic |
| `number-sum` | Correctly rounded summation expectations computed with exact rational arithmetic |

`cargo xtask generate all --check` checks the Unicode and numeric outputs. Unicode
generators download checksum-pinned inputs unless given a local source path; see
their `--help` and the [Unicode source notes](../../../third-party/unicode/README.md).
Keep numeric generators independent of the engine's conversion algorithms.

`cargo xtask generate regexp-results CAPTURE.tsv ...` rebuilds the static native
RegExp comparison table. Capture mode requires Node 24.19 and a
`SPITE_REGEXP_ORACLE_CAPTURE` output path when running Runtime tests. It compares
engine results during capture. Review changes against the specification: native
results do not override edition-specific behavior. This generator is deliberately
excluded from `all` and does not accept `--check`.
