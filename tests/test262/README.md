# Test262 lexical smoke tests

These 11 unmodified fixtures come from
[tc39/test262](https://github.com/tc39/test262) at the commit in `REVISION`.
They cover hashbang placement and line terminators. `manifest.tsv` records each
upstream path, expected outcome, and SHA-256 digest. `LICENSE` is the upstream BSD
license. Each fixture also retains its copyright notice.

All selected tests use the `raw` flag. The integration test runs each positive
case once in a fresh realm without changing its source or adding a harness.
Negative cases must fail during parsing with a syntax diagnostic at the misplaced
hash character. Unsupported functionality and unrelated syntax failures fail the
regression test.

The manifest is an explicit, reviewed inventory. It is not a YAML metadata parser
or a general Test262 runner. These results do not measure whole-suite conformance.
Other tests require grammar, objects, functions, or harness facilities that have
not been implemented yet. The general runner remains on the roadmap.

Run `python3 tools/check-test262.py` to verify the vendored bytes. Git attributes
prevent line-ending normalization of fixtures. Do not edit the source files to
make the engine pass. When updating the pin, copy the selected files unchanged,
review their metadata, update hashes and expectations, and rerun the checks.
