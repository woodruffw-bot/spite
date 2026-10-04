# Test262 regression fixtures

These 65 unmodified fixtures come from
[tc39/test262](https://github.com/tc39/test262) at the commit in `REVISION`.
`manifest.tsv` records each upstream path, test mode, and SHA-256 digest.
`LICENSE` is the upstream BSD license. Each fixture retains its copyright notice.

## Script smoke tests

Eleven hashbang fixtures use the `raw` flag. The integration test in `spite` runs
each positive case once in a fresh realm without changing its source or adding a
harness. Negative cases must fail during parsing with a syntax diagnostic at the
misplaced hash character. Unsupported functionality and unrelated syntax failures
fail the regression test.

## Identifier component tests

Thirteen fixtures test the lexer directly in `spite-parser`. Eight files form
literal/escaped pairs for Unicode 16 and 17 identifier additions. Each pair must
produce identical decoded tokens. These inputs cover 8,949 identifier starts and
two identifiers containing the additional ID_Continue characters. Five negative
fixtures must reject the exact invalid identifier escape with the expected
message and source span.

These are lexer regressions, not passing Script tests. Their assertions compare
tokens, without evaluating the sources. The original sources are never rewritten
to make them executable, and they do not contribute to an execution pass count.
Unicode 18 additions are covered separately by the pinned UCD tables and local
parser tests. Each test group checks its reviewed manifest inventory.

## Statement parser tests

Six `noStrict` fixtures cover the `let` lookahead rules in while, for, and if bodies.
The parser receives each complete, unchanged source. Tests check that ASI leaves
`let` as an identifier expression and that the following block or assignment
becomes a separate statement. These are parser regressions only; they do not run
the Test262 harness or count as full Script execution passes.

## Scope and maintenance

The `spite-test262` command runs 76 variants from 46 reviewed sources: the eleven
raw hashbang fixtures, ten BigInt parse-negative files in both Script modes,
eleven arrow parse-negative files in their prescribed Script modes, and fourteen
new.target parse-negative files in both Script modes.
That means four raw positive evaluations and 72 reviewed parse-negative variants.
Arrow reviews cover the no-line-terminator restriction, duplicate simple
parameters, default-parameter duplicates, strict/reserved bindings and initializer
references, and Use Strict Directives with non-simple parameters. Each error is checked at the
intended token; unsupported bodies or unrelated syntax cannot substitute for it.
NewTarget reviews cover Script/arrow-only scope, escaped grammar terminals, and
assignment/prefix/postfix targets, including parenthesized forms. Invalid assignment
diagnostics highlight the rejected target rather than the following RHS token.
The other nineteen files remain component regressions, outside this result count.
`runner.tsv` records exact rejection byte ranges and messages for negative tests.
Strict variants adjust these ranges only for the prescribed directive prefix.
No test bodies are rewritten, and parse-negative tests never evaluate harness code.

Run `cargo run -p spite-test262 --locked -- tests/test262` from the repository root.
Every selected file and non-passing result is reported. Missing files, invalid
metadata, unsupported features, limits, setup failures, unverified diagnostics,
and unexpected outcomes cannot produce a successful gate. GitHub Actions runs
this command in every platform/toolchain test configuration.

These results do not measure whole-suite conformance. Other tests require grammar,
objects, functions, or harness facilities that have not been implemented yet.
The `spite-test262` crate now reads a documented subset of frontmatter and plans
execution modes and harness include order. Its Script runner now separates parse,
harness, and runtime outcomes, requiring a reviewed diagnostic for parse-negative
passes. The complete upstream harness, modules, async completion, and agent
configuration remain unsupported.

Run `python3 tools/check-test262.py` to verify the vendored bytes. Git attributes
prevent line-ending normalization of fixtures. Do not edit the source files to
make the engine pass. When updating the pin, copy the selected files unchanged,
review their metadata, update hashes and expectations, and rerun the checks.
