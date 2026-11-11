# spite

spite is a JavaScript parser and interpreter written in Rust. It provides a
command-line runner and a library for embedding JavaScript in Rust applications.
The language target is [ECMAScript 2026 (ECMA-262, 17th edition)](https://262.ecma-international.org/17.0/).

The runtime supplies ECMAScript built-ins. Browser APIs, Node.js APIs such as
`console` and `process`, and internationalization APIs from ECMA-402 are outside
its scope. See [compatibility](docs/compatibility.md) for host choices, native
resource constraints, and conformance evidence. All workspace crates forbid unsafe
Rust.

## Run JavaScript

Install Rust 1.99 or later, then build and run the CLI:

```sh
git clone https://github.com/woodruffw-bot/spite.git
cd spite
cargo run --release -p spite -- --eval '6 * 7'
```

The runner prints the script's completion value:

```text
42
```

You can also run a file or check its syntax without executing it:

```sh
cargo run --release -p spite -- example.js
cargo run --release -p spite -- --check example.js
```

The [CLI guide](docs/cli.md) covers modules, JSON imports, standard input, resource
limits, and exit codes.

## Embed the interpreter

The `spite` crate exposes the parser, runtime, and supporting types. Start with
the [embedding guide](docs/embedding.md) for a complete example and the host's
responsibilities for Promise jobs and garbage collection. The crates are not
published to crates.io; use a path dependency on a local checkout.

## Develop and test

From the repository root:

```sh
cargo test --workspace --locked
cargo xtask check
cargo xtask test262
```

The first command runs unit, integration, and documentation tests. `xtask check`
checks formatting, Clippy, API documentation, dependency policy, source
provenance, and generated data; it also needs Python 3, rustfmt, Clippy, and network
access. The last command runs the pinned Test262 corpus separately from the native
tests. See the [development guide](docs/development.md) for setup and focused checks.

## Documentation

| Guide | Contents |
| --- | --- |
| [CLI](docs/cli.md) | Run scripts and modules from the command line |
| [Embedding](docs/embedding.md) | Use the Rust API and manage a realm |
| [Compatibility](docs/compatibility.md) | Language scope, host choices, and constraints |
| [Architecture](docs/design.md) | Crate boundaries and interpreter design |
| [Development](docs/development.md) | Build, test, and maintain generated data |
| [Test262](tests/test262/README.md) | Corpus selection and conformance testing |
| [Roadmap](docs/roadmap.md) | Further implementation work |
