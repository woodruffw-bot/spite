# Development guide

Use Rust 1.99 or later. The workspace's minimum version and shared dependency
versions are declared in [Cargo.toml](../Cargo.toml).

```sh
rustup component add rustfmt clippy
cargo build --workspace --locked
```

Python 3 is needed for data generation and its checks. Ordinary Rust tests do not
need Python or Node.js. Node.js is used only when deliberately capturing reference
results for RegExp fixtures.

## Test and check commands

Run commands from the repository root:

| Command | Purpose |
| --- | --- |
| `cargo test --workspace --locked` | Unit, integration, and documentation tests |
| `cargo xtask check` | Formatting, Clippy, API docs, dependency policy, fixture provenance, and generated data |
| `cargo xtask test262` | The complete pinned Test262 corpus |
| `cargo xtask verify` | Local dependency policy and fixture inventories, without network access |

`xtask check` needs network access to verify pinned specification and Unicode
sources. The full Test262 target uses a separate optimized profile while keeping
debug assertions and overflow checks enabled. It is intentionally separate from
the default `cargo test` run.

[CI](../.github/workflows/ci.yml) runs native tests and Test262 on Linux and Windows
with stable Rust and the minimum supported version. A separate job runs
`xtask check`.

### Focused tests

Use Cargo's package, target, and name filters:

```sh
cargo test -p spite-core --locked
cargo test -p spite-parser --test parser_integration --locked regexp_metadata
cargo test -p spite-runtime --test integration --locked regexp_exec::
```

Parser cases live directly under `crates/spite-parser/tests/`. Runtime cases live
under `crates/spite-runtime/tests/cases/` and share helpers from `tests/common/`.
Each crate uses one integration-test entry point. After adding or removing a case
file, regenerate the module lists:

```sh
cargo xtask generate test-modules
```

Keep runtime fixture helpers under `cases/fixtures/`, so they are not treated as
test modules.

### Readable test cases

Use named `#[test_case(...; "case_name")]` cases when inputs share the same
setup and assertions. Keep distinct behaviors in separate tests so a failure
identifies the operation and condition being checked.

Write embedded JavaScript as formatted, multiline source. Use descriptive names
and keep setup, the operation, and its observable results easy to distinguish.
Keep compact source only when its exact spelling, whitespace, or offsets are
part of the behavior under test.

Use insta for structured output, syntax trees, and diagnostics that benefit from
a reviewed diff. Prefer inline snapshots of at most 100 lines. Use sidecar files
only when the output exceeds 100 lines. Assert simple scalar results directly.

### Updating snapshots

Install `cargo-insta` when you need to review snapshot changes. Run a focused set
of tests, then review both inline and sidecar changes:

```sh
cargo install cargo-insta --locked
INSTA_UPDATE=new cargo test -p spite-parser --test parser_integration --locked regexp_metadata
cargo insta review
```

Inspect each change against the intended syntax or diagnostic behavior, then
check the source diff and rerun the tests with `INSTA_UPDATE=no`. CI never updates
snapshots. Keep outputs of at most 100 lines inline when accepting changes.

## Generated data

Generators live in [crates/xtask/codegen](../crates/xtask/codegen/README.md).
Run them through Cargo:

```sh
cargo xtask generate all --check
```

Replace `all` with a generator name, such as `unicode`, to run a focused check.
Unicode generators verify their pinned inputs before comparing output; sources
and licenses are described in the
[Unicode data notes](../third-party/unicode/README.md). Numeric fixtures use
independent reference calculations. Capturing RegExp results from another engine
is an explicit maintenance step; see the generator guide for the command and
required Node.js version.

Test262 has a separate [corpus maintenance guide](../tests/test262/README.md).

## Code and documentation

Follow [Google's Rust style guide](https://google.github.io/styleguide/rustguide.html)
and the repository rules in [AGENTS.md](../AGENTS.md). Prefer focused modules,
explicit imports, conventional formatting, and direct control flow.

For subtle language behavior, cite the relevant specification algorithm and test
observable behavior, including error order. Keep host failures distinct from
JavaScript exceptions.

Documentation should explain current behavior and useful commands. Keep run
logs, benchmark results, development history, and personal workspace details out
of it. The [benchmarking guide](benchmarking.md) describes local measurements
without publishing machine-specific results.
