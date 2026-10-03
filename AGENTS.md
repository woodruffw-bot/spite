# Development

- Follow docs/design.md and keep docs/roadmap.md accurate.
- Use safe Rust. Every crate inherits the workspace lints.
- Production dependencies must be workspace crates, except regex and jiff when needed.
- No other new direct dependencies or external build dependencies are approved.
- Keep member crates under crates/ and share dependency versions in Cargo.toml.
- Implement ECMA-262 behavior. Do not add Node.js, browser, or syntax extensions.
- Cite specification sections for subtle semantics and add regression tests.
- Separate unsupported features and host limits from JavaScript exceptions.
- Never count unsupported syntax as a passing negative conformance test.
- Use insta for syntax and diagnostic snapshots. Inspect changes before committing.
- Run formatting, Clippy, tests, and the dependency-policy check before pushing.
- Commit coherent changes and push completed commits regularly.
- Keep public API documentation and human-facing prose clear and concise.
