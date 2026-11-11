# Test262 corpus

This directory contains the upstream tests selected for spite's ECMAScript 2026
target, plus records explaining the selection. Files under `upstream/` are
unchanged from the revision in [REVISION](REVISION), under the upstream
[license](LICENSE).

An **original** is an upstream test, rather than a harness helper or support
fixture. Selected originals run through the conformance target. Every other
original in the pinned revision has a reason recorded in
[audit/scope.tsv](audit/scope.tsv).

## Run the tests

From the repository root:

```sh
cargo xtask test262
```

This runs the complete selected corpus in an optimized profile with debug
assertions and overflow checks enabled. Each variant gets a fresh realm with
runtime quotas disabled. The gate uses three workers; results retain manifest
order.

For a serial run with a report for each variant:

```sh
cargo run -p spite-test262 --profile conformance --locked -- tests/test262
```

The runner reads each original's frontmatter to select script strictness, raw or
module mode, asynchronous completion, harness includes, and negative-test phase.
File names and feature tags do not determine how a test executes.

## Understand results

A test passes when execution succeeds as specified or produces its verified
expected exception. The runner keeps other outcomes distinct:

| Outcome | Meaning |
| --- | --- |
| `failed` | An unexpected result or JavaScript exception |
| `unsupported` | Missing language functionality or a required host facility |
| `limit` | An explicit quota or native capacity was reached |
| `unverified` | A parse failure does not match an exact reviewed syntax diagnostic |
| `setup-failure` | Harness loading or evaluation failed independently of the test |
| `metadata-invalid`, `metadata-unsupported`, `metadata-limit` | Frontmatter could not be processed; reported separately from execution |

A parse-negative test must match the reviewed syntax diagnostic's span and
message. Unsupported syntax cannot substitute for a `SyntaxError`. Runtime and
resolution negatives must fail in the declared phase with the expected exception
constructor. Async tests must call `$DONE` exactly once.

Host failures and limits cannot satisfy a negative JavaScript test. Excluded
originals are not passes; see [compatibility](../../docs/compatibility.md) for the
host choices, specification interpretations, and native constraints.

## Manifests

| File | Contents |
| --- | --- |
| [manifest.tsv](manifest.tsv) | Fixture kind, SHA-256, and upstream-relative path for each retained file |
| [runner.tsv](runner.tsv) | Selected originals and their reviewed parse diagnostics; executed entries use dashes |
| [audit/](audit/README.md) | Upstream Git identities, excluded originals, source reviews, and supporting evidence |

The integrity checker verifies retained bytes against both the local manifest
and upstream Git blobs. It also requires the runner and scope records to account
for every original in the pinned revision. Support modules and harness files are
verified, but do not count as executable originals.

```sh
cargo xtask verify
```

This command uses local records and needs no network access.

## Admit or update an original

1. Compare the source's actual requirements with the published edition and host
   choices. A proposal tag alone is not a reason to exclude a baseline test.
2. Retain the original bytes and all required upstream harness and support files.
3. Add manifest and runner entries. For a parse-negative test, review its exact
   diagnostic; for other tests, preserve the declared execution modes and phase.
4. Remove the old scope entry and update any associated source review or witness.
5. Run `cargo xtask verify` and `cargo xtask test262`.

Do not shorten originals, change their metadata, or increase default quotas to
make them pass. Smaller local regressions can isolate behavior, but do not
replace execution of the original.

The [audit guide](audit/README.md) explains the evidence required for Unicode
differences, staging decisions, and native resource constraints.
