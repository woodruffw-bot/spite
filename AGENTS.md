# Development

- Follow docs/design.md and keep docs/roadmap.md accurate.
- Keep documentation focused on current APIs, design, test commands and constraints.
  Keep fixture identities in machine-readable manifests. Do not add run logs,
  benchmark results, commit narratives or personal workspace paths to docs.
- Follow [Google's Rust style guide](https://google.github.io/styleguide/rustguide.html)
  and conventional Cargo layout. Prefer focused crates, leaf modules, explicit
  imports and direct, readable control flow.
- Use safe Rust. Every crate inherits the workspace lints.
- Production dependencies must be workspace crates, except regex, regress, jiff,
  and jiff-tzdb when needed.
- The development-only xtask may use serde_json, toml, sha1 and sha2 for Cargo
  metadata, configuration and source integrity checks. Engine crates must not
  depend on xtask or its tooling dependencies.
- No other new direct dependencies or external build dependencies are approved.
- Tests may use insta and test-case as dev-dependencies. Prefer inline snapshots;
  reserve sidecar snapshots for output longer than 100 lines.
- Keep member crates under crates/ and share dependency versions in Cargo.toml.
- Implement ECMA-262 behavior. Do not add Node.js, browser, or syntax extensions.
- Resolve the edition 17 RegExpBuiltinExec inconsistency for Unicode matches
  starting inside a surrogate pair by following Node/V8: use the leading boundary
  for the whole-match start, consistent with the matcher and capture boundaries.
  This narrowly approved exception covers empty and consuming matches and is
  documented in docs/design.md.
- Cite specification sections for subtle semantics and add regression tests.
- Separate unsupported features and host limits from JavaScript exceptions.
- Host resource quotas are opt-in. Do not add default quotas or test-only
  allowances to make conformance fixtures pass. Document necessary native-stack
  and platform safety checks as implementation constraints.
- Never count unsupported syntax as a passing negative conformance test.
- Use insta for syntax and diagnostic snapshots. Inspect changes before committing.
- Run formatting, Clippy, tests, and the dependency-policy check before pushing.
- Commit coherent changes and push completed commits regularly.
- Keep public API documentation and human-facing prose clear and concise.
