# spite

A JavaScript parser and interpreter written in safe Rust.

spite targets [ECMAScript 2026](https://262.ecma-international.org/17.0/).
It implements an initial subset and is **not a conforming engine yet**.
Node.js compatibility, browser APIs, and syntax extensions are outside its scope.

Production crates use std and workspace crates, plus jiff/jiff-tzdb for time zones.
The design also permits regex or regress for matching when needed. Workspace lints
forbid unsafe code. insta is used only for tests.

## Run

Rust 1.85 or later is required.

```sh
cargo run -p spite -- --eval 'let x = 6; x * 7'
cargo run -p spite -- example.js
printf '1 + 2' | cargo run -p spite -- -
```

The CLI prints the Script's completion value. Strings use a quoted representation
that preserves lone surrogates. Source input must be UTF-8. The host installs no
console, process, filesystem, or other non-standard JavaScript globals.

Host resource quotas are disabled by default. Supply `--max-steps N` before the input
arguments to opt into a per-Script work limit. Work units count interpreter and
builtin operations, rather than JavaScript statements or elapsed time.

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

The workspace contains `spite-core`, `spite-parser`, `spite-runtime`, and the
`spite` facade and CLI. All members live under `crates/`.

See the [design](docs/design.md), [roadmap](docs/roadmap.md), and
[current coverage](docs/coverage.md). GitHub Actions checks formatting, Clippy,
unit and integration tests, documentation tests, dependency policy, and pinned
Test262 fixture integrity on each push and pull request. Tests run on Linux and
Windows with stable Rust and the minimum supported Rust version.
