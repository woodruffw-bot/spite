# spite

A JavaScript parser and interpreter written in safe Rust.

spite targets [ECMAScript 2026](https://262.ecma-international.org/17.0/).
The [specification review](docs/spec-review.md) explains the published baseline,
host choices and known interpretations. Native resource constraints are described
in [coverage](docs/coverage.md). Node.js compatibility, browser APIs and ECMA-402
are outside the target. Later-proposal facilities supply no baseline conformance credit.

Production crates use std and workspace crates, regress for RegExp matching, and
jiff/jiff-tzdb for time zones. Workspace lints forbid unsafe code. insta is used
only for tests.

## Run

Rust 1.99 or later is required.

```sh
cargo run -p spite -- --eval 'let x = 6; x * 7'
cargo run -p spite -- example.js
printf '1 + 2' | cargo run -p spite -- -
cargo run -p spite -- --module --eval 'export const answer = await Promise.resolve(42)'
cargo run -p spite -- --check --eval 'function square(value) { return value * value; }'
```

By default, the CLI drains queued Promise jobs, then prints the Script's completion value.
Dynamic import() calls load Module files through the same cache as Module mode.
An idle pending Promise does not delay Script completion. Strings use a quoted
representation that preserves lone surrogates. Source input must be UTF-8. The
host installs no console, process, filesystem, or other non-standard JavaScript globals.

Use `--module` for Module sources from an argument, file or stdin.
Module mode waits for top-level Await and prints the evaluation completion value,
normally `undefined`. Rejected or unsettled evaluations fail. Static and dynamic imports are
literal filesystem paths resolved relative to their importing file, or the current
directory for argument/stdin sources. Canonical paths and module types share
dependency records, including cycles. Use `with { type: "json" }` for a JSON
default export, or `import(path, { with: { type: "json" } })` dynamically. Without
attributes, files are JavaScript Modules. The explicit type selects the parser
independently of the file suffix; no package or extension search is performed.
Imported syntax errors reject dynamic imports with SyntaxError; filesystem,
unsupported-feature and opted-in limit failures remain host failures.
Uncaught Error values show their stored name and message in diagnostics.

Use `--check` to validate syntax and early errors, with `--module` for the Module
goal. Argument, file and stdin inputs share the same UTF-8 handling. Successful
checks produce no output; diagnostics go to stderr with a failing exit status.
Module dependencies are loaded only during evaluation. Execution work limits
apply to evaluations and jobs.

Host resource quotas are disabled by default. Supply `--max-steps N` before the input
arguments to opt into a work limit for each evaluation or job. Work units count interpreter and
builtin operations, rather than JavaScript statements or elapsed time.
Supply `--max-source-bytes N` to limit each parsed source in UTF-8 bytes. This
applies to `--check`, evaluated inputs, imported Modules and eval/Function-generated
source. Source-limit failures are host failures rather than catchable exceptions.
File and stdin readers stop after at most one byte beyond that limit, including
JavaScript and JSON dependencies, so oversized streams need not reach EOF.

## Test

Run the workspace tests, documentation tests, and complete reviewed Test262 corpus:

```sh
cargo test --workspace --locked
cargo xtask check
cargo xtask test262
```

`cargo test --workspace` runs native tests and documentation examples. The full
Test262 corpus is an explicit target, run separately through `cargo xtask test262`.
Both test profiles keep debug assertions and overflow checks enabled; exhaustive
Unicode checks and the interpreter use optimization to keep execution practical.

`cargo xtask check` checks formatting, Clippy, dependency policy, fixture provenance,
and generated data. It requires Python 3 and network access for pinned source
verification. `cargo xtask verify` checks local inventories and provenance offline.
See [data generation](crates/xtask/codegen/README.md) for maintenance commands.

The complete Test262 gate explicitly uses three corpus workers. Every reviewed
variant runs, and reports retain manifest order. Each evaluated variant uses a
fresh realm, ordinary native stacks, and runtime defaults with JavaScript quotas
disabled. The library's `run_corpus` API and the CLI use serial execution.

Runtime integration cases live under `crates/spite-runtime/tests/cases` and share
one harness. For example, run a subset with
`cargo test -p spite-runtime --test integration regexp_unicode_devanagari_script`.
After adding a case file, run `cargo xtask generate test-modules`; CI checks that
the generated module lists include every case. Keep fixture helpers under
`cases/fixtures`.

Manual parser, initialization, evaluation and collection measurements are
available through the dependency-free [benchmark example](docs/benchmarking.md).

## Embed

Provide at least a 2 MiB native thread stack. The
[cleanup example](crates/spite/src/lib.rs) shows an explicit host thread with
that stack size.

```rust
use spite::{Realm, Value};

let mut realm = Realm::default();
assert_eq!(realm.eval("let x = 6; x * 7").unwrap(), Value::Number(42.0));
```

Embedders can opt into an execution work limit with
`Realm::new(Limits { max_steps: Some(100_000), ..Limits::default() })`.
Every `Limits` field is optional and defaults to `None`. For example,
`max_heap_entries: Some(10_000)` opts into a shared object/environment slot quota.
Platform capacity checks and the interpreter's native-stack guards remain active.

The workspace contains `spite-core`, `spite-bigint`, `spite-heap`, `spite-parser`,
`spite-runtime`, `spite-test262`, and the `spite` facade and CLI. All members live
under `crates/`.

See the [design](docs/design.md), [roadmap](docs/roadmap.md), and
[current coverage](docs/coverage.md). GitHub Actions checks formatting, Clippy,
unit and integration tests, documentation tests, dependency policy, and pinned
Test262 fixture integrity on each push and pull request. Tests run on Linux and
Windows with stable Rust and the minimum supported Rust version.
