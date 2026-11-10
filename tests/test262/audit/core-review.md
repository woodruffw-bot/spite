# Review of the remaining core originals

This review covers the 202 core originals left after the
[published-feature review](post-baseline.md). It uses the published edition-17
algorithms and the implementation's explicit non-browser host policies. One
earlier exclusion was wrong: `__proto__-permitted-dup.js` uses required object
literal semantics without the optional Object.prototype accessor. Both unchanged
modes pass, and the original is now retained.

The other 201 originals have the following dispositions. Every path remains in
the verified upstream inventory and `scope.tsv`; none is counted as passing.

| Disposition | Originals | Evidence |
| --- | ---: | --- |
| `unicode-version-difference` | 92 | Ninety exhaustive property datasets disagree with Unicode18; two literal loops require obsolete join-control IdentityEscapes. |
| `optional-legacy-helpers` | 74 | Fifty-four lookup/definition helper tests, four private-accessor tests using those helpers, fifteen __proto__ accessor tests and one TypedArray prototype check through that accessor require disabled normative optional Object.prototype facilities. |
| `optional-function-introspection` | 23 | Non-strict `.caller` reads assume a caller-chain extension or an undefined fallback; this host retains the required Function.prototype poison accessors and supplies no own caller extension. |
| `optional-annex-b-dependent` | 1 | One String.split file requires Annex B RegExp escapes. |
| `post-publication-assertion` | 6 | Iterator.take/drop assertions reject finite limits above 2**53-1; the published algorithms reject NaN/negative limits and retain larger mathematical counts. |
| `allocation-assumption` | 5 | Dense-allocation assumptions contradict the host's sparse byte-block representation; the TypedArray object-length case also assumes allocation failure after ToLength clamps 2**53. |

## Unicode data

[Conformance](https://262.ecma-international.org/17.0/#sec-conformance) requires
the latest Unicode standard. Unicode18 is the implementation's explicit current
version. The review independently parses the original UCD files pinned in
[UNICODE-SOURCES.json](../../../vendor/regress/UNICODE-SOURCES.json), rather than
using the generated matcher tables as its reference. It compares each original
property dataset's complete ranges with those independently derived ranges.

[unicode-witnesses.tsv](unicode-witnesses.tsv) records one contradictory code
point for each of the ninety whole property originals, including both expected
memberships. Actual matching agrees with the independent UCD result for every
witness in both u/v modes. CI requires a contradictory witness for each such
scope entry and executes all 180 public RegExp checks in the runner controls.

The two S7.8.5 literal loops deliberately assume U+200C/U+200D are outside
ID_Continue. Both are members in UCD18. The core IdentityEscape production
therefore rejects their literal escaped forms; the files require acceptance.
The supplementary control verifies current ID_Continue matching and the required
SyntaxErrors. No fragment of an incompatible original is substituted for it.

## Optional properties and escape grammar

The legacy Object.prototype lookup/definition helpers and __proto__ accessor
are explicitly normative optional in edition 17 (20.1.3.8-9). The sixteen
accessor-dependent originals were initially labelled Annex B; their scope
category is corrected without changing their policy or passing credit.
Extended RegExp IdentityEscape grammar belongs to disabled Annex B. The String.split
original includes forms such as `/\XA0/` and incomplete `/\x/`; these are not
valid under the core Pattern grammar. Required prototype initializers in object
literals remain enabled, including the newly retained duplicate-method test.

All twenty-one Function originals read a non-strict function's `.caller`
without catching the inherited TypeError. The two arguments-object originals
likewise assume `.callee.caller` returns a value. Providing the required poison
accessors while declining caller-chain extensions is the documented host choice.
The controls check absence of the optional properties and preservation of the
shared Function.prototype throwing getter/setter identities.

## Iterator limits and byte allocation

The published [take](https://262.ecma-international.org/17.0/#sec-iterator.prototype.take)
and [drop](https://262.ecma-international.org/17.0/#sec-iterator.prototype.drop)
algorithms contain no finite maximum-safe-integer rejection. The controls check
conversion/next lookup order, absence of premature closing, and completed
iteration for 2**53, 1e100, Number.MAX_VALUE and Infinity. Other retained originals
and native cases protect NaN/negative rejection and required closing.

CreateByteDataBlock throws RangeError if creating its Data Block is impossible;
it does not require a dense physical allocation. Sparse ordinary/shared backing
can represent 7-PiB and maximum-safe-integer byte blocks. The controls construct
both fixed and resizable/growable forms and verify zero reads and isolated writes
at their last byte, without materializing their zero regions.

`TypedArrayConstructors/ctors/object-arg/length-excessive-throws.js` supplies an
array-like length of 2**53. ToLength clamps this to 2**53-1. A byte-element
destination can represent that buffer; a getter sentinel confirms allocation
finishes and element reading begins. This does not change ToIndex rejection for
primitive excessive lengths, the byte-length bound, or explicit host quotas.

The five allocation originals are ArrayBuffer and SharedArrayBuffer's respective
`allocation-limit.js` and `options-maxbytelength-allocation-limit.js`, plus that
TypedArray object-argument file. They receive no passing credit.

[published-policy-probes.js](published-policy-probes.js) contains the bounded
supplementary probes, executed in both Script modes by the runner controls.
The 105-candidate policy investigation preserves all outcomes: the one wrongly
excluded object-literal original passes two modes; the other 104 originals fail
185 modes under the documented policies or published algorithms. Separate
conformance-core-policy and conformance-unicode18 artifacts under `/workspace/work`
retain raw results, exact source/build identities and the independent UCD range
comparison. The staging review remains open.
