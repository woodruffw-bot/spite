# Development

- Follow docs/design.md and keep docs/roadmap.md accurate.
- Keep roadmap entries focused on concrete specification implementation. Record
  test execution and fixture inventories in docs/coverage.md and tests/test262/README.md.
- Use safe Rust. Every crate inherits the workspace lints.
- Production dependencies must be workspace crates, except regex, jiff, and
  jiff-tzdb when needed.
- No other new direct dependencies or external build dependencies are approved.
- Keep member crates under crates/ and share dependency versions in Cargo.toml.
- Implement ECMA-262 behavior. Do not add Node.js, browser, or syntax extensions.
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
