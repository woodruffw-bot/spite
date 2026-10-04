# Test262 regression fixtures

These 1727 unmodified test fixtures and four harness files come from
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
Forty-eight BigInt files cover construction, integer strings in all radices,
ordered conversions, signed/unsigned width reduction, prototype identity,
radix formatting, and branded primitive/wrapper receivers.
The 173 Object files cover SameValue, own-property checks, descriptor conversion and
reflection, prototype identity/mutation, extensibility, creation, value copying,
key enumeration, and frozen/sealed integrity. Getter-mutation cases use accessor
literals to exercise snapshot keys, live descriptors, and abrupt reads.
The 286 String files cover wrappers, raw construction, character
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
gate. Array.from/fromAsync, additional includes, async completion,
and agent helpers remain separate harness gaps.

The 677 Array and Array iterator files cover call/new construction, of, branding, literal elisions,
indexed growth, truncation, generic at/join/push/pop, toString/toLocaleString, and ordered
forEach/every/some callback traversal, find/findIndex/findLast/findLastIndex,
includes/indexOf/lastIndexOf searches, reduce/reduceRight accumulators, species-aware map/filter/slice/concat/flat/flatMap/splice, sparse reverse, fill/copyWithin range mutations, shift/unshift front mutations, and sort/toSorted and toReversed/with/toSpliced copies, plus keys/values/entries
iteration and live mapped/unmapped arguments. The 286 String files now include
String iterator conversion, ancestry, branding, and surrogate-pair traversal.
The 34 Symbol files cover identity, construction, boxing, descriptions, registry
access, branded methods, and conversion hooks. These files run unchanged with
the upstream harness in their prescribed default/strict Script modes.

## Map/filter/slice review

The 73 files added at the existing pin comprise 23 map, 23 filter, and 27 slice
files. They add 144 variants: two callback receiver tests prescribe `noStrict`;
the remaining 71 files run in both Script modes. Coverage includes constructor
and species lookup failures, null/undefined species fallback, custom constructor
arguments and result identity, non-Array receivers, failed own data definitions,
live sparse callback visits, slice range signs and omitted ends, inherited
read-only output indices, and Array length RangeErrors before copying.

Reviewed candidates requiring `isConstructor.js`/Reflect, `propertyHelper.js`,
foreign realms, proxies, or resizable buffers remain outside this corpus. Their
sources are not rewritten and their outcomes are not counted as passes. The
existing native and Script regressions cover additional ordering, partial effects,
custom-result descriptors, and host limits.

## Concat review

The 47 concat files add 93 variants: one duplicate-parameter arguments test
prescribes `noStrict`; the other 46 files run in both Script modes. They cover
species lookup and construction, ordered spreadability hooks, truthy/falsy and
undefined markers, sparse and inherited elements, boxed primitives,
mapped/unmapped arguments, length coercion failures, and failed result definitions.
The near-safe-integer-length test throws from its first indexed getter, so it
checks full-width length handling without an unbounded scan.

Candidates requiring classes, proxies, foreign realms, typed arrays, RegExp,
`isConstructor.js`/Reflect, or `propertyHelper.js` remain outside this corpus.
The spreadable-function candidate requires the missing `Function` global. The
4,000-hole sparse-object fixture now runs unchanged in both Script modes under
the default runtime configuration, with host resource quotas disabled.
Local runtime regressions also cover safe-integer overflow before indexed reads,
strict final length writes, aliased species results, and bounded host scans.

## Flat/flatMap review

The 28 files added at the existing pin comprise 13 flat and 15 flatMap files.
They add 55 variants: the flatMap callback receiver test prescribes `onlyStrict`;
the other 27 files run in both Script modes. Coverage includes generic receivers,
bound calls and callbacks, depth coercion, infinite depth, nullish elements and
receivers, poisoned source lengths, callback exceptions, species fallback and
construction failures, and failed output definitions. The 10,001-index sparse
flatMap input runs unchanged under the ordinary runtime defaults.

Reviewed candidates requiring typed arrays, proxies/Reflect, `isConstructor.js`,
or `propertyHelper.js` remain outside this corpus. Local runtime regressions
cover metadata and descriptors, sparse live mutations, aliased species results,
safe-integer overflow, and iterative traversal of deeply nested Arrays.

## Splice review

The 66 files added at the existing pin add 132 variants, with every file running
in both Script modes. They cover positive/negative/omitted deletion ranges,
fractional/NaN/infinite conversions, generic objects, inherited properties,
full-width indices near 2^53-1, shrinking/growing sparse movement, constructor
and species failures/fallback, custom species identity and arguments, failed
result definitions, and strict final length writes including zero arguments.

Reviewed candidates requiring for-in, Math.pow, foreign realms, proxies/Reflect,
`isConstructor.js`, `proxyTrapsHelper.js`, or `propertyHelper.js` remain outside
this corpus. Sources are unchanged; no default quotas or test allowances are
introduced. Local runtime regressions also cover species aliasing, partial
mutations on failure, and complete Array prototype reflection/integrity operations.

## BigInt API review

The 48 files added at the existing pin add 96 variants, all in both Script modes:
16 constructor files, ten each for asIntN/asUintN, eight for toString, three for
valueOf, and one for the prototype's prototype. They exercise exact Number and
integer-string conversion, invalid inputs, abrupt hooks, ToIndex/ToBigInt order,
multiword reductions, lowercase radix digits, and receiver brands.

Reviewed candidates needing the Function constructor, Date, for-of, foreign
realms, `isConstructor.js`/Reflect, or `propertyHelper.js` remain outside the
corpus. The proposed parseInt directory is outside the edition-17 target. No
fixture source or execution quota changes. Local regressions cover descriptors,
non-constructibility, observable tags, huge fitting widths, and opted-in failures.

## Array.from review

The 39 files added at the existing pin add 74 variants: two receiver tests
prescribe noStrict and two prescribe onlyStrict; the other 35 run in both modes.
They cover iterable and array-like inputs, dense undefined elements, live source
changes, mapper arguments/receivers, constructor selection/arity, abrupt iterator
access and stepping, failed result definitions, and final strict length writes.

Candidates needing ArrayBuffer, foreign realms, `isConstructor.js`/Reflect,
generators, or `propertyHelper.js` remain outside this corpus. Sources and harness
files are unchanged, and runtime defaults continue to disable resource quotas.
Local regressions additionally check cached next methods, done/value access order,
iterator cleanup precedence, aliased outputs, partial effects, huge array-like
length arguments, and host failures during traversal or cleanup.

## Array literal spread review

Twenty unchanged generated files add 40 variants, all in both Script modes.
They cover empty/single/multiple spread inputs, AssignmentExpression evaluation,
custom iterators, unresolvable operands, abrupt iterator getters/calls/steps/values,
and exact accumulation order. Generator candidates remain outside this corpus;
the separate object-spread review below adds supported nested object cases.
Existing pinned sources and harness files are unchanged.
Local tests also cover literal holes versus yielded undefined, strings and sparse
Arrays, cached next calls, inherited setters, native Array length boundaries,
and host work failures without iterator cleanup.

## Call/construction spread review

Forty unchanged generated files add 80 variants, all in both Script modes:
twenty call files and twenty new-expression files. They cover mixed/empty
argument lists, AssignmentExpression operands, custom iterators, unresolvable
references, and abrupt iterator getters/calls/steps/values. Candidates needing
generators or eval remain outside this corpus; supported nested object cases are
reviewed separately below. No source
or harness rewriting and no execution quota changes are required. Local tests
also check retained receivers/callees, prototype lookup after argument evaluation,
bound construction, partial argument evaluation, and a 4,000-argument default call.

## Object spread review

Forty-five unchanged generated files add 90 variants in both Script modes:
fifteen each in Array initializers, calls, and new expressions. They cover nullish
sources, unresolvable references, getter evaluation, repeated spread, own-key order,
non-enumerable properties, Symbols, and ordered overwrites. The twelve reviewed
candidates requiring `propertyHelper.js` remain outside this corpus, as do the
object-expression Proxy cases. No source, harness, or quota changes are needed.
Local regressions additionally cover live descriptors after getter mutations,
primitive boxing, copy attributes, setter bypass, and `__proto__` data keys.

## Synchronous for-of review

Seventy-five unchanged files add 139 variants: 53 positive sources (97 variants)
and 22 parse-negative sources (42 variants). They cover Array and arguments
iteration, live lengths, String code points, custom iterators, reference targets,
lexical TDZ and fresh bindings, cached next, and iterator-close precedence. Each
negative has a reviewed source span and message for invalid headers, contextual
keywords, forbidden body declarations, or lexical conflicts. Unsupported patterns,
generators, collections, typed arrays, Proxies, RegExp, and eval-dependent cases
remain outside this selection. Sources, harness files, and default quotas are unchanged.

## Identifier rest parameter review

Nineteen unchanged files add 38 variants in both Script modes: seven positive
sources cover rest Arrays, argument indexing, function length, no aliasing,
call/apply, and arrows; twelve parse negatives cover initializers, final-position
rules, trailing commas, and Use Strict Directives with non-simple parameters.
Each negative has a reviewed rejection span and message. Binding-pattern,
eval-dependent scope, generator, and class cases remain outside this selection.
Local tests additionally cover methods, ordinary/bound construction, TDZ,
closures, setter bypass, and opt-in property quotas. No source or harness changes
and no execution quota adjustments are needed.

## For-in review

Fifty-six unchanged files add 99 variants: 36 positive sources (67 variants) and
20 parse-negative sources (32 variants). They cover own/prototype key ordering,
non-enumerable shadowing, live deletion, snapshot additions, reference targets,
comma RHS expressions, lexical TDZ and fresh captures, var hoisting, ASI, and
returns through catch/finally. Each negative has a reviewed span and message for
invalid targets, lexical conflicts, body declarations, or strict binding names.

Binding patterns, eval-dependent completions/scope, async/generator/class syntax,
and resizable-buffer cases remain outside this selection. The upstream const
fresh-binding file actually uses for-of and is omitted from this for-in group.
Local regressions additionally cover Symbols, getters, nullish skipping, primitive
boxing, completion values, unsupported intrinsic chains, and opt-in host quotas.
No source, harness, or execution quota changes are needed.

## Object.fromEntries review

Twenty-one unchanged files add 42 variants in both Script modes. They cover empty
iterables, ordinary result prototypes, key order, entry-read/conversion order,
Symbols, boxed String entries, inherited setter bypass, invalid inputs/entries,
entry failures that close, and step failures that do not close. The four reviewed
candidates needing `propertyHelper.js` or `isConstructor.js`/Reflect remain outside
this selection. Local regressions cover descriptors, non-constructibility,
cleanup precedence, mutation, collection, and opt-in quotas. Sources, harness
files, and execution quotas are unchanged.

## Object.groupBy review

Twelve unchanged files add 24 variants in both Script modes. They cover callback
arguments, empty groups, grouping order, numeric key conversion, null prototypes,
String code-point iteration, invalid callbacks/iterables, callback and key throws,
and abrupt iterator stepping. The two reviewed `propertyHelper.js` metadata files
remain outside this selection. Local regressions additionally cover Symbols,
receiver rules, live mutations, iterator-close precedence, intrinsic result Arrays,
collection, the safe-integer counter boundary, and opt-in host failures. No source,
harness, or execution quota changes are needed.

## String split review

Eighty unchanged files add 160 variants in both Script modes. They cover generic
receivers, empty/ordinary separators, ordered conversions and abrupt failures,
uint32 limits, undefined separators, metadata, object Symbol.split delegation,
and edition-17 primitive hook bypass. Reviewed candidates needing RegExp, Math,
eval, `propertyHelper.js`, or `isConstructor.js`/Reflect remain outside this corpus.
The function-enumeration candidate reaches incomplete Function.prototype and is
also omitted. Local regressions additionally cover surrogate halves, wrapped
separators, inherited setter bypass, collection, large default results, and opt-in
host failures. Sources, harness files, and execution quotas are unchanged.

## String replace review

Twenty-two unchanged files add 44 variants in both Script modes. They cover
generic receivers, first-match replacement, conversion order and abrupt failures,
object Symbol.replace delegation, and edition-17 primitive hook bypass. Reviewed
candidates needing RegExp, the Function constructor, `propertyHelper.js`, or
`isConstructor.js`/Reflect remain outside this corpus. Local regressions cover
callback arguments, literal callback results, zero-capture dollar substitutions,
UTF-16 boundaries, descriptors, collection, and opt-in host failures. The pin,
upstream bytes, harness files, and execution quotas remain unchanged.

## String replaceAll review

Thirty-one unchanged files add 62 variants in both Script modes. They cover
non-overlapping and empty searches, dollar substitutions, functional replacements
and result conversion, ordered fallback conversions, primitive hook bypass,
and object IsRegExp, flag, and replacement-hook failures. Reviewed candidates
requiring RegExp objects, `propertyHelper.js`, or `isConstructor.js`/Reflect remain
outside this corpus. Local regressions cover global-flag validation before hooks,
original callback arguments, UTF-16 boundaries, metadata, collection, and opt-in
host failures. Sources, harness files, and execution quotas are unchanged.

## Reflect call and construction review

Nine unchanged files add 18 variants in both Script modes. Five apply files cover
target validation, object-only array-like arguments, receiver/argument passing,
return values, and non-constructibility. Four construction files cover argument
reads, custom and default newTarget, and resulting prototypes. The pinned,
unchanged `isConstructor.js` is the fourth harness file; local controls verify
true/false results, its non-function error, and propagation of host gaps. Reviewed
candidates requiring the Function or Date globals or `propertyHelper.js` remain
outside this corpus. The existing pin and execution quotas remain unchanged.

## Reflect prototype and extensibility review

Twenty-five unchanged files add 50 variants in both Script modes: six
getPrototypeOf, ten setPrototypeOf, four isExtensible, and five preventExtensions
files. They cover object-only targets, prototype identity, cycles, non-extensible
changes, idempotence, and non-constructibility using the pinned isConstructor
helper. The preventExtensions Symbol-target fixture checks isExtensible in its
original source; local regressions check all four methods directly. Reviewed
Proxy and `propertyHelper.js` candidates remain outside this corpus. The pin,
harness bytes, and execution quotas remain unchanged.

## Reflect property read, presence, and deletion review

Twenty-one unchanged files add 42 variants in both Script modes: eight get, six
has, and seven deleteProperty files. They cover ordered key conversion, explicit
read receivers, getters and inherited reads, symbol keys, boolean deletion,
primitive target rejection, and non-constructibility. Reviewed Proxy and
`propertyHelper.js` candidates remain outside this corpus. Local regressions also
cover key-conversion mutations, exact primitive accessor receivers, dormant
getters, Array/String/arguments exotics, metadata, collection, and host failures.
The pin, harness bytes, and execution quotas remain unchanged.

## Reflect descriptors and own keys review

Twenty-five unchanged files add 50 variants in both Script modes: seven
defineProperty, nine getOwnPropertyDescriptor, and nine ownKeys files. They cover
ordered conversions, boolean definitions, data/accessor descriptor fields, symbol
keys, non-enumerables, chronological order and large indices, primitive target
rejection, and non-constructibility. Reviewed Proxy and `propertyHelper.js`
candidates remain outside this corpus. Local regressions cover Array partial
truncation, fresh descriptors, setter/species bypass, exotics, collection, and host
gaps. The pin, harness bytes, and execution quotas remain unchanged.

## Scope and maintenance

`Runner::default()` uses the ordinary runtime defaults: every `Limits` field is
`None`, with no source-size, work, string/BigInt size, argument, heap, or property
quota. Test262 frontmatter also has no default size quota. Embedders can opt into a
limit with `Runner { limits: Limits { max_steps: Some(units), ..Limits::default() } }`.
Each harness file and each test body then starts with a fresh allowance. Assertion
functions called by a test consume that test body's allowance. Work units account
for interpreter operations, property/prototype scans, value copies, and numeric
work; they are not elapsed time or a count of JavaScript statements.

Checked arithmetic, addressable capacity, handle validation, and native-stack
guards remain implementation safety checks. Exhaustion of an opted-in quota
remains a non-passing `Limit` result and
establishes a host limitation, not a semantic failure or an unsupported specification
feature. The runner does not retry failures with different limits.
Runtime-negative matching reads the thrown object's constructor name as required
by upstream `INTERPRETING.md`. These checked host reads execute getters normally;
inspection failures cannot satisfy the original expected exception. Primitive
throws and missing/non-string constructor names do not pass an error expectation.

The `spite-test262` command runs 3342 variants from 1708 reviewed sources: the eleven
raw hashbang fixtures, ten BigInt parse-negative files in both Script modes,
eleven arrow parse-negative files in their prescribed Script modes, and fourteen
new.target parse-negative files in both Script modes, plus nine positive function
and capture tests, 40 call/construction iterable-spread tests, 30 call/construction
object-spread tests, 80 Reflect call, construction, prototype, extensibility, descriptor, and property tests, twelve Boolean tests, 63 Number tests, ten numeric parsing
tests, five global numeric predicate tests, 22 Error tests, 48 BigInt API tests, 173 Object tests,
286 String and String iterator tests, 677 Array and Array iterator tests
(including fifteen nested object-spread files),
34 Symbol tests, 23 object method/accessor tests, and 75 for-of files (53 positive
and 22 parse-negative), plus seven rest-parameter positives and twelve parameter
parse negatives, and 56 for-in files (36 positive and 20 parse-negative). The method/accessor files
cover computed key conversion and exceptions, numeric/string/escaped names,
reserved method names, and closure scope. Eight Object entries/values files use
accessor literals to test live enumeration changes and abrupt reads.
That means four raw positives,
3168 positives using the upstream harness, and 170 reviewed parse-negative variants.
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
are defined in `assert.js`. Successful comparisons execute unchanged. Numeric mismatch
formatting executes map and reports the thrown Test262Error as a runtime failure.
String SameValue mismatch formatting still requires JSON and remains Unsupported. Local
controls cover these paths.
