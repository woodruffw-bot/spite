# Test262 regression fixtures

These 925 unmodified test fixtures and three harness files come from
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

## Upstream harness execution

The pinned, unchanged `assert.js` and `sta.js` files now execute in each non-raw
positive test realm. Nine positive files cover calls/construction, new.target
whitespace/comments, lexical new.target, lexical arguments, and lexical this
through call/apply/bind. Twelve Boolean files cover primitive conversion,
construction, constructor links, and the fallback object tag after deleting
Boolean.prototype.toString. Sixty-three Number files cover explicit BigInt
conversion, numeric predicate boundaries, and fixed-point formatting including
exact decimal rounding, significant precision, exponential output, and primitive
and wrapper values in every non-decimal radix. Ten global
numeric parsing tests cover decimal prefixes/exponents, invalid radices, and
hexadecimal/default-decimal rules. Five global predicate tests cover function
shape, abrupt coercion, and nonfinite values. Twenty-two Error files cover
construction, message conversion, prototype identity, branding, and toString.
The 140 Object files cover SameValue, own-property checks, descriptor conversion and
reflection, prototype identity/mutation, extensibility, creation, value copying,
key enumeration, and frozen/sealed integrity. Getter-mutation cases use accessor
literals to exercise snapshot keys, live descriptors, and abrupt reads.
The 153 String files cover wrappers, raw construction, character
access, searches, concatenation, substrings, trimming, repetition, padding, Unicode well-formedness, UTF-16 encoding/decoding,
and ordered conversions. Each runs in both required Script
modes. The `harness` manifest mode verifies support-file bytes without counting
them as test cases;
`script-pass` selects non-raw positive tests. Harness frontmatter describes helper
definitions and is not parsed as test execution metadata.

Local controls execute successful assertions and deliberately failing assertions
against these exact harness files. Test262Error construction, native Error
construction, and built-in exception catch/constructor checks execute normally.
Formatting some failed comparisons still requires JSON or other missing
standard APIs; those paths report Unsupported and fail the
gate. Remaining Array methods, additional includes, async completion,
and agent helpers remain separate harness gaps.

The 389 Array and Array iterator files cover call/new construction, of, branding, literal elisions,
indexed growth, truncation, generic at/join/push/pop, toString/toLocaleString, and ordered
forEach/every/some callback traversal, find/findIndex/findLast/findLastIndex,
includes/indexOf/lastIndexOf searches, reduce/reduceRight accumulators, sparse reverse, fill/copyWithin range mutations, shift/unshift front mutations, and sort/toSorted and toReversed/with/toSpliced copies, plus keys/values/entries
iteration and live mapped/unmapped arguments. The 153 String files now include
String iterator conversion, ancestry, branding, and surrogate-pair traversal.
The 34 Symbol files cover identity, construction, boxing, descriptions, registry
access, branded methods, and conversion hooks. These files run unchanged with
the upstream harness in their prescribed default/strict Script modes.

## Scope and maintenance

The conformance runner allows one million work units per Script evaluation so
unchanged upstream files can combine many assertions and exact conversions.
Other limits retain their runtime defaults. Exhaustion remains a non-passing
`Limit` result; no test is retried with an unbounded budget.
Runtime-negative matching reads the thrown object's constructor name as required
by upstream `INTERPRETING.md`. These bounded host reads execute getters normally;
inspection failures cannot satisfy the original expected exception. Primitive
throws and missing/non-string constructor names do not pass an error expectation.

The `spite-test262` command runs 1770 variants from 906 reviewed sources: the eleven
raw hashbang fixtures, ten BigInt parse-negative files in both Script modes,
eleven arrow parse-negative files in their prescribed Script modes, and fourteen
new.target parse-negative files in both Script modes, plus nine positive function
and capture tests, twelve Boolean tests, 63 Number tests, ten numeric parsing
tests, five global numeric predicate tests, 22 Error tests, 140 Object tests,
153 String and String iterator tests, 389 Array and Array iterator tests,
34 Symbol tests, and 23 object method/accessor tests. The method/accessor files
cover computed key conversion and exceptions, numeric/string/escaped names,
reserved method names, and closure scope. Eight Object entries/values files use
accessor literals to test live enumeration changes and abrupt reads.
That means four raw positives,
1694 positives using the upstream harness, and 72 reviewed parse-negative variants.
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
passes. Harness support is limited to the executed paths above; modules, async
completion, and agent configuration remain unsupported.

Run `python3 tools/check-test262.py` to verify the vendored bytes. Git attributes
prevent line-ending normalization of fixtures. Do not edit the source files to
make the engine pass. When updating the pin, copy the selected files unchanged,
review their metadata, update hashes and expectations, and rerun the checks.

The pinned `compareArray.js` include is a compatibility file; its assertions
are defined in `assert.js`. Successful comparisons execute unchanged. Mismatch
formatting currently needs the missing Array.prototype.map method and is
reported as Unsupported, never Passed. Local controls cover both paths.
