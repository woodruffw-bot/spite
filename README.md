# spite

A JavaScript parser and interpreter written in safe Rust.

spite targets [ECMAScript 2026](https://262.ecma-international.org/17.0/).
The [published-specification review](docs/spec-review.md) is complete; exact
[coverage and implementation constraints](docs/coverage.md) are recorded separately.
Full conformance is not claimed.
Node.js compatibility, browser APIs, and syntax extensions are outside its scope.

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
directory for argument/stdin sources. Canonical paths share dependency records,
including cycles. Paths are read as UTF-8 Module source without package or extension
search. Imported syntax errors reject dynamic imports with SyntaxError; filesystem,
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

## Test

Run the workspace tests, documentation tests, and complete reviewed Test262 corpus:

```sh
python3 tools/run-cargo-workspace-tests.py --toolchain stable
cargo +stable test --workspace --doc --locked
cargo +stable test -p spite-test262 --profile conformance --test corpus --locked pinned_corpus_runs_all_reviewed_variants -- --exact
```

The workspace runner executes Cargo's test invocations with their original
arguments and environment, leaving the full corpus to the separate command.
Core test builds optimize the exhaustive Unicode checks. Debug assertions and
overflow checks remain enabled; interpreter and integration test builds retain
their normal unoptimized profile. Test builds retain line and function debug
information while omitting full variable and type details to reduce linking costs.
Link-time optimization is disabled for test builds.

The complete Test262 gate explicitly uses three corpus workers. Every reviewed
variant runs, and reports retain manifest order. Each evaluated variant uses a
fresh realm, ordinary native stacks, and runtime defaults with JavaScript quotas
disabled. The library's `run_corpus` API and the CLI use serial execution.

Runtime integration cases live under `crates/spite-runtime/tests/cases` and share
one harness. For example, run a subset with
`cargo test -p spite-runtime --test integration regexp_unicode_devanagari_script`.
Add each new case file to the harness module list; CI checks for omitted or
duplicate files. Keep fixture helpers under `cases/fixtures`.

Manual parser, initialization, evaluation and collection measurements are
available through the dependency-free [benchmark example](docs/benchmarking.md).

## Embed

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
