# Test262 regression fixtures

These 7434 unmodified test fixtures and eleven harness files come from
[tc39/test262](https://github.com/tc39/test262) at the commit in `REVISION`.
`manifest.tsv` records each upstream path, test mode, and SHA-256 digest.
`LICENSE` is the upstream BSD license. Each fixture retains its copyright notice.



## RegExp constructor pattern and flag coercion fixture review

Thirteen unchanged complete S15.10.4.1_A8 originals add twenty-six normal/strict
harness-positive variants at the existing pin. Complete programs verify pattern
and flag conversion through primitive values, boxed strings, calls, eval,
toString and valueOf, along with initialized flags, source and lastIndex.
Original exception programs verify thrown string values from pattern/flag hooks
and pattern conversion preceding invalid flags. Historical prose and TODO
comments remain unchanged.

Stable, MSRV and Node pass all twenty-six complete variants without failures or
host limits. Upstream Git blob identities and manifest SHA-256 digests verify every
source byte. Original metadata, Sputnik copyright, BSD license, the pin and eleven
helpers are preserved. Caught exceptions are harness-positive execution;
parse-negative and runtime-negative inventories are unchanged.

The active corpus has 7434 fixtures, eleven helpers, 7415 reviewed Script sources
and 14308 variants: 12831 harness positives, 1469 parse negatives, four raw
positives and four runtime negatives. RegExp builtin execution contains 587 whole
positive programs, separately from 337 grammar fixtures.

Stable and MSRV pass the complete 14308-variant corpus, workspace targets
and documentation checks with unlimited defaults. Formatting, denied-warning
Clippy, dependency/fixture policy and all offline data checks also pass.

## RegExp constructor object and malformed Pattern fixture review

Six unchanged complete S15.10.4.1_A6, A7 and A9 originals add twelve normal/strict
harness-positive variants at the existing pin. Complete programs verify the
RegExp object tag, inherited properties and the original RegExp prototype, and
caught SyntaxError results for malformed question marks and character ranges.
Original historical prose, try/catch programs and TODO comments remain unchanged.

Stable, MSRV and Node pass all twelve complete variants without failures or
host limits. Upstream Git blob identities and manifest SHA-256 digests verify every
source byte. Original metadata, Sputnik copyright, BSD license, the pin and eleven
helpers are preserved. Caught SyntaxErrors are harness-positive execution;
parse-negative and runtime-negative inventories are unchanged.

The active corpus has 7421 fixtures, eleven helpers, 7402 reviewed Script sources
and 14282 variants: 12805 harness positives, 1469 parse negatives, four raw
positives and four runtime negatives. RegExp builtin execution contains 574 whole
positive programs, separately from 337 grammar fixtures.

Stable and MSRV pass the complete 14282-variant corpus, workspace targets
and documentation checks with unlimited defaults. Formatting, denied-warning
Clippy, dependency/fixture policy and all offline data checks also pass.

## RegExp constructor invalid flags whole-program fixture review

Seven unchanged complete S15.10.4.1_A5 originals add fourteen normal/strict
harness-positive variants at the existing pin. Complete original programs catch
and verify SyntaxError for duplicate flags, invalid flags obtained through eval,
and flag coercion from null, Number, Boolean and an object whose toString returns
undefined. The original try/catch programs, TODO comments and historical flag
lists remain unchanged; their assertions exercise runtime constructor behavior.

Stable, MSRV and Node pass all fourteen complete variants without failures or
host limits. Upstream Git blob identities and manifest SHA-256 digests verify every
source byte. Original metadata, Sputnik copyright, BSD license, the pin and eleven
helpers are preserved. The caught SyntaxErrors are harness-positive execution;
parse-negative and runtime-negative inventories are unchanged.

The active corpus has 7415 fixtures, eleven helpers, 7396 reviewed Script sources
and 14270 variants: 12793 harness positives, 1469 parse negatives, four raw
positives and four runtime negatives. RegExp builtin execution contains 568 whole
positive programs, separately from 337 grammar fixtures.

Stable and MSRV pass the complete 14270-variant corpus, workspace targets
and documentation checks with unlimited defaults. Formatting, denied-warning
Clippy, dependency/fixture policy and all offline data checks also pass.

## RegExp constructor defaults and explicit flags whole-program fixture review

Twelve unchanged complete S15.10.4.1_A2, A3 and A4 originals add 24 normal/strict
harness-positive variants at the existing pin. Original programs accept explicit flags on a RegExp
pattern and reject invalid object flags with SyntaxError. Other originals exercise
omitted or undefined pattern/flags through void, hoisted variables, missing
properties and immediately invoked functions, checking the default Boolean flags.
Null patterns retain the original source assertions. Historical descriptions and the try/catch TODO stay unchanged.

Stable, MSRV and Node pass all 24 complete variants without failures or host
limits. Upstream Git blob identities and manifest SHA-256 digests verify every
source byte. Original metadata, Sputnik copyright, BSD license, the pin and eleven
helpers are preserved. The caught SyntaxError is harness-positive execution;
parse-negative and runtime-negative inventories are unchanged.

The active corpus has 7408 fixtures, eleven helpers, 7389 reviewed Script sources
and 14256 variants: 12779 harness positives, 1469 parse negatives, four raw
positives and four runtime negatives. RegExp builtin execution contains 561 whole
positive programs, separately from 337 grammar fixtures.

Stable and MSRV pass the complete 14256-variant corpus, workspace targets
and documentation checks with unlimited defaults. Formatting, denied-warning
Clippy, dependency/fixture policy and all offline data checks also pass.

## RegExp constructor copy whole-program fixture review

Five unchanged complete S15.10.4.1_A1 originals add ten normal/strict
harness-positive variants at the existing pin. Complete original programs verify
that new RegExp copies source and multiline/global/ignoreCase flags from a RegExp
pattern when flags are absent or evaluate to undefined through a hoisted variable,
void, the undefined value or an immediately invoked function. Empty, literal and
call-created patterns remain covered by the unchanged original assertions.

Stable, MSRV and Node pass all ten complete variants without failures or
host limits. Upstream Git blob identities and manifest SHA-256 digests verify every
source byte. Original metadata, Sputnik copyright, BSD license, the pin and eleven
helpers are preserved. Parse-negative and runtime-negative inventories are unchanged.

The active corpus has 7396 fixtures, eleven helpers, 7377 reviewed Script sources
and 14232 variants: 12755 harness positives, 1469 parse negatives, four raw
positives and four runtime negatives. RegExp builtin execution contains 549 whole
positive programs, separately from 337 grammar fixtures.

Stable and MSRV pass the complete 14232-variant corpus, workspace targets
and documentation checks with unlimited defaults. Formatting, denied-warning
Clippy, dependency/fixture policy and all offline data checks also pass.

## RegExp call whole-program fixture review

Nine unchanged complete S15.10.3.1 originals add eighteen normal/strict
harness-positive variants at the existing pin. Complete original programs verify
RegExp call identity with absent, undefined, void and hoisted undefined flags,
invalid string and numeric flag SyntaxErrors, and string/object pattern coercion
with constructor identity and source preservation. Every original assertion and
try/catch remains unchanged.

Stable, MSRV and Node pass all eighteen complete variants without failures or
host limits. Upstream Git blob identities and manifest SHA-256 digests verify every
source byte. Original metadata, Sputnik copyright, BSD license, the pin and eleven
helpers are preserved. Parse-negative and runtime-negative inventories are unchanged.

The active corpus has 7391 fixtures, eleven helpers, 7372 reviewed Script sources
and 14222 variants: 12745 harness positives, 1469 parse negatives, four raw
positives and four runtime negatives. RegExp builtin execution contains 544 whole
positive programs, separately from 337 grammar fixtures.

Stable and MSRV pass the complete 14222-variant corpus, workspace targets
and documentation checks with unlimited defaults. Formatting, denied-warning
Clippy, dependency/fixture policy and all offline data checks also pass.

## Pattern runtime-validation whole-program fixture review

Sixteen unchanged complete S15.10.1_A1 originals add thirty-two normal/strict
harness-positive variants at the existing pin. The original complete programs
call the RegExp constructor with repeated quantifiers, leading quantifiers and
consecutive counted quantifiers. Each original try/catch and instanceof SyntaxError
assertion remains unchanged and verifies the language exception during constructor
evaluation. Original TODO comments, metadata, copyright and license remain intact.

Stable, MSRV and Node pass all thirty-two complete variants without failures or
host limits. Upstream Git blob identities and manifest SHA-256 digests verify every
source byte. These complete Script harness positives leave parse-negative and
runtime-negative inventories unchanged; the pin and eleven helpers are preserved.

The active corpus has 7382 fixtures, eleven helpers, 7363 reviewed Script sources
and 14204 variants: 12727 harness positives, 1469 parse negatives, four raw
positives and four runtime negatives. RegExp builtin execution contains 535 whole
positive programs, separately from 337 grammar fixtures.

Stable and MSRV pass the complete 14204-variant corpus, workspace targets
and documentation checks with unlimited defaults. Formatting, denied-warning
Clippy, dependency/fixture policy and all offline data checks also pass.

## CharacterRange runtime-validation whole-program fixture review

Forty-one unchanged complete S15.10.2.15 originals add eighty-two normal/strict
harness-positive variants at the existing pin. Original complete programs call
the RegExp constructor with descending character ranges, including surrounding
ranges, character-class escapes, NUL, legacy octal, backspace and control escapes,
hexadecimal and Unicode escapes and identity escapes. Each original try/catch
and instanceof SyntaxError assertion remains unchanged and verifies the native
language exception during constructor evaluation.

Stable, MSRV and Node pass all eighty-two complete variants without failures or
host limits. Upstream Git blob identities and manifest SHA-256 digests verify every
source byte. Original metadata, Sputnik copyright, BSD license, the pin and eleven
helpers are preserved. These complete Script harness positives leave the
parse-negative and runtime-negative inventories unchanged.

The active corpus has 7366 fixtures, eleven helpers, 7347 reviewed Script sources
and 14172 variants: 12695 harness positives, 1469 parse negatives, four raw
positives and four runtime negatives. RegExp builtin execution contains 519 whole
positive programs, separately from 337 grammar fixtures.

Stable and MSRV pass the complete 14172-variant corpus, workspace targets
and documentation checks with unlimited defaults. Formatting, denied-warning
Clippy, dependency/fixture policy and all offline data checks also pass.

## CharacterClass whole-program fixture review

Twenty-eight unchanged complete S15.10.2.13 originals add fifty-six
normal/strict harness-positive variants at the existing pin. Original checks
cover empty and negated-empty classes, inclusive and mixed ranges, digit and
whitespace class escapes, punctuation, counted and optional classes, whitespace
lookahead, negated ranges and in-class backspace with escaped brackets. Original
boolean, null-match, full array length, index, input and element-loop assertions
remain unchanged.

Stable, MSRV and Node pass all fifty-six complete variants without failures or
host limits. Upstream Git blob identities and manifest SHA-256 digests verify every
source byte. Original metadata, Sputnik copyright, BSD license, the pin and eleven
helpers are preserved.

The active corpus has 7325 fixtures, eleven helpers, 7306 reviewed Script sources
and 14090 variants: 12613 harness positives, 1469 parse negatives, four raw
positives and four runtime negatives. RegExp builtin execution contains 478 whole
positive programs, separately from 337 grammar fixtures.

Stable and MSRV pass the complete 14090-variant corpus, workspace targets
and documentation checks with unlimited defaults. Formatting, denied-warning
Clippy, dependency/fixture policy and all offline data checks also pass.

## Word CharacterClassEscape whole-program fixture review

Two unchanged complete S15.10.2.12 originals add four normal/strict
harness-positive variants at the existing pin. Original checks cover the ASCII
word class and its complement, including whitespace, punctuation, every ASCII
letter and digit, underscore and global execution loops. Original null-match
checks and full loop-count assertions remain unchanged.

Stable, MSRV and Node pass all four complete variants without failures or host
limits. Upstream Git blob identities and manifest SHA-256 digests verify every
source byte. Original metadata, Sputnik copyright, BSD license, the pin and eleven
helpers are preserved.

The active corpus has 7297 fixtures, eleven helpers, 7278 reviewed Script sources
and 14034 variants: 12557 harness positives, 1469 parse negatives, four raw
positives and four runtime negatives. RegExp builtin execution contains 450 whole
positive programs, separately from 337 grammar fixtures.

Stable and MSRV pass the complete 14034-variant corpus, workspace targets
and documentation checks with unlimited defaults. Formatting, denied-warning
Clippy, dependency/fixture policy and all offline data checks also pass.

## AtomEscape whole-program fixture review

Four unchanged complete S15.10.2.9 originals add eight normal/strict
harness-positive variants at the existing pin. Original whole checks cover a
word-boundary repeated word, numbered references following optional captures with
both unmatched and participating siblings, and required empty references after
an empty variable capture. Full array length, index, input, every capture and
loop assertion remain unchanged.

Stable, MSRV and Node pass all eight complete variants without failures or host
limits. Upstream Git blob identities and manifest SHA-256 digests verify every
source byte. Original metadata, Sputnik copyright, BSD license, the pin and eleven
helpers are preserved.

The active corpus has 7295 fixtures, eleven helpers, 7276 reviewed Script sources
and 14030 variants: 12553 harness positives, 1469 parse negatives, four raw
positives and four runtime negatives. RegExp builtin execution contains 448 whole
positive programs, separately from 337 grammar fixtures.

Stable and MSRV pass the complete 14030-variant corpus, workspace targets
and documentation checks with unlimited defaults. Formatting, denied-warning
Clippy, dependency/fixture policy and all offline data checks also pass.

## DecimalEscape whole-program fixture review

Seven unchanged complete S15.10.2.11 originals add fourteen normal/strict
harness-positive variants at the existing pin. Original full checks exercise
NUL in literal and constructor patterns, participating numbered references,
undefined forward reads, independent repeated captures and two orders of
references to ten nested captures. Every original capture and loop assertion
remains unchanged.

Stable, MSRV and Node pass all fourteen complete variants without failures or host
limits. Upstream Git blob identities and manifest SHA-256 digests verify every
source byte. Original metadata, Sputnik copyright, BSD license, the pin and eleven
helpers are preserved.

The active corpus has 7291 fixtures, eleven helpers, 7272 reviewed Script sources
and 14022 variants: 12545 harness positives, 1469 parse negatives, four raw
positives and four runtime negatives. RegExp builtin execution contains 444 whole
positive programs, separately from 337 grammar fixtures.

Stable and MSRV pass the complete 14022-variant corpus, workspace targets
and documentation checks with unlimited defaults. Formatting, denied-warning
Clippy, dependency/fixture policy and all offline data checks also pass.

## CharacterEscape whole-program fixture review

Thirteen unchanged complete S15.10.2.10 originals add 26 normal/strict
harness-positive variants at the existing pin. Original complete checks exercise
tab, line feed, vertical tab, form feed and carriage-return escapes, all upper
and lower ASCII control letters, hexadecimal bytes, four-digit Unicode escapes,
Latin and Cyrillic alphabets, and identity escapes for punctuation. Dynamic
constructors, repeated literal escapes, original loops and every assertion remain.

Stable, MSRV and Node pass all 26 complete variants without failures or host
limits. Upstream Git blob identities and manifest SHA-256 digests verify every
source byte. Original metadata, Sputnik copyright, BSD license, the pin and eleven
helpers are preserved.

The active corpus has 7284 fixtures, eleven helpers, 7265 reviewed Script sources
and 14008 variants: 12531 harness positives, 1469 parse negatives, four raw
positives and four runtime negatives. RegExp builtin execution contains 437 whole
positive programs, separately from 337 grammar fixtures.

Stable and MSRV pass the complete 14008-variant corpus, workspace targets
and documentation checks with unlimited defaults. Formatting, denied-warning
Clippy, dependency/fixture policy and all offline data checks also pass.

## Disjunction whole-program fixture review

Seventeen unchanged complete S15.10.2.3 originals add 34 normal/strict
harness-positive variants at the existing pin. Preserved complete checks cover
source-ordered alternatives and their continuations, nested alternative captures,
unmatched sibling slots, counted and repeated alternatives, ignoreCase with
original input text, dot branches and both orders of captured/uncaptured empty
alternatives. All match-array, index, input and boolean failure checks remain.

Stable, MSRV and Node pass all 34 complete variants without failures or host
limits. Upstream Git blob identities and manifest SHA-256 digests verify every
source byte. Original metadata, Sputnik copyright, BSD license, the pin and eleven
helpers are preserved.

The active corpus has 7271 fixtures, eleven helpers, 7252 reviewed Script sources
and 13982 variants: 12505 harness positives, 1469 parse negatives, four raw
positives and four runtime negatives. RegExp builtin execution contains 424 whole
positive programs, separately from 337 grammar fixtures.

Stable and MSRV pass the complete 13982-variant corpus, workspace targets
and documentation checks with unlimited defaults. Formatting, denied-warning
Clippy, dependency/fixture policy and all offline data checks also pass.

## Quantifier whole-program fixture review

Sixty-nine unchanged complete S15.10.2.7 originals add 138 normal/strict
harness-positive variants at the existing pin. All six QuantifierPrefix groups
retain complete original checks for bounded and unbounded counts, exact counts,
optional matches, greedy adjacent captures and continuations, literal and escaped
question marks, empty matches, unmatched optional captures and backreferences.
Preserved digit, word, whitespace and negated-class programs include non-ASCII
input boundaries, escapes and original input/index assertions. Originals calling
test retain their complete boolean failure checks.

Stable, MSRV and Node pass all 138 complete variants without failures or host
limits. Upstream Git blob identities and manifest SHA-256 digests verify every
source byte. Exact shared headers and assertion suffixes reconstruct every whole
source for review; all distinct metadata and program bodies remain unchanged.
The original Sputnik copyright, BSD license, pin and eleven helpers are preserved.

The active corpus has 7254 fixtures, eleven helpers, 7235 reviewed Script sources
and 13948 variants: 12471 harness positives, 1469 parse negatives, four raw
positives and four runtime negatives. RegExp builtin execution contains 407 whole
positive programs, separately from 337 grammar fixtures.

Stable and MSRV pass the complete 13948-variant corpus, workspace targets
and documentation checks with unlimited defaults. Formatting, denied-warning
Clippy, dependency/fixture policy and all offline data checks also pass.

## Term quantification whole-program fixture review

Five unchanged complete S15.10.2.5 originals add ten normal/strict harness-positive
variants at the existing pin. Their preserved checks cover greedy and lazy bounded
classes, source-ordered repeated alternatives, last-iteration captures, clearing
unmatched optional children and a required repetition of an empty backreference.
Every original retains complete match-array, index, input and capture assertions.

Stable, MSRV and Node pass all ten complete variants without failures or host
limits. Upstream Git blob identities and manifest SHA-256 digests verify every
source byte. The original metadata, Sputnik copyright, BSD license, existing pin
and eleven helpers are preserved.

The active corpus has 7185 fixtures, eleven helpers, 7166 reviewed Script sources
and 13810 variants: 12333 harness positives, 1469 parse negatives, four raw
positives and four runtime negatives. RegExp builtin execution contains 338 whole
positive programs, separately from 337 grammar fixtures.

Stable and MSRV pass the complete 13810-variant corpus, workspace targets
and documentation checks with unlimited defaults. Formatting, denied-warning
Clippy, dependency/fixture policy and all offline data checks also pass.

## Nullable repeated capture whole-program fixture review

The unchanged complete S15.10.2.8_A3_T17 original adds both normal and strict
harness-positive variants at the existing pin. Its nested lazy repeated capture
body retains all assertions for the complete HTML match, match index, original
input and final iteration captures. With optional empty iterations rejected and
capture effects rolled back, this completes all sixty whole Sputnik Atom and
lookahead originals reviewed in the preceding cohort.

Stable and MSRV pass both complete variants without failures or host limits;
Node passes all 120 original cohort variants. The upstream Git blob identity and
manifest SHA-256 digest verify every source byte. Metadata, Sputnik copyright,
the BSD license, the existing pin and eleven helpers are preserved.

The active corpus has 7180 fixtures, eleven helpers, 7161 reviewed Script sources
and 13800 variants: 12323 harness positives, 1469 parse negatives, four raw
positives and four runtime negatives. RegExp builtin execution contains 333 whole
positive programs, separately from 337 grammar fixtures.

Stable and MSRV pass the complete 13800-variant corpus, workspace targets
and documentation checks with unlimited defaults. Formatting, denied-warning
Clippy, dependency/fixture policy and all offline data checks also pass.

## Atom and lookahead whole-program fixture review

Fifty-nine unchanged complete originals from the S15.10.2.8 cohort add 118
normal/strict harness-positive variants at the existing pin. The complete review
covers all sixty originals: five positive-lookahead programs, eleven negative-
lookahead programs, 33 capture/group programs, nine dot programs and two ordinary
case-matching programs. Preserved checks exercise lookahead capture commitment
and negative rollback, nested and repeated captures, numeric backreferences,
replacement substitutions, optional unmatched groups, dot and ignoreCase. Two
constructor programs retain all 200 nested capture or noncapture scopes.

The remaining A3_T17 original requires an unproved nested repeated capture body
and stays Unsupported in both modes, with no fixture or passing credit. All 120
complete variants pass Node; stable and MSRV pass the selected 118 without failures
or host limits. Git blob identities and manifest SHA-256 digests verify every
selected source byte. All metadata, Sputnik copyright notices and the BSD license
are preserved. The existing pin, eleven helpers, dependencies, negative reviews
and unlimited defaults stay unchanged.

The active corpus has 7179 fixtures, eleven helpers, 7160 reviewed Script sources
and 13798 variants: 12321 harness positives, 1469 parse negatives, four raw
positives and four runtime negatives. RegExp builtin execution contains 332 whole
positive programs, separately from 337 grammar fixtures.

Stable and MSRV pass the complete 13798-variant corpus, workspace targets
and documentation checks with unlimited defaults. Formatting, denied-warning
Clippy, dependency/fixture policy and all offline data checks also pass.

## Outside-reference lookbehind whole-program fixture review

Two further unchanged complete originals, lookBehind/back-references.js and
lookBehind/sliced-strings.js, add four normal/strict harness-positive variants
at the existing pin. Their 21 assertions check stable outside references,
pre-match captures, ignoreCase, references imported from lookahead, repeated
references, insufficient prefixes and exact null results on sliced inputs.
Every assertion, source byte, metadata field, specification quotation and V8
copyright notice is retained.

Git blob identities and manifest SHA-256 digests verify both complete originals.
The existing compareArray.js include and all eleven helpers stay byte-identical.
All four complete Script variants also pass Node. The 17-file lookBehind review
now contains five vendored originals and twelve exclusions; those exclusions
remain Unsupported in both modes and receive no passing credit. The separate
115-file named-groups/exec review remains at 192 Passed and 38 Unsupported.
The pin, dependencies, negative expectations and unlimited defaults are unchanged.

The active corpus has 7120 fixtures, eleven helpers, 7101 reviewed Script sources
and 13680 variants: 12203 harness positives, 1469 parse negatives, four raw
positives and four runtime negatives. RegExp builtin execution contains 273 whole
positive programs, separately from 337 grammar fixtures.

Stable and MSRV pass the complete 13680-variant corpus, workspace targets
and documentation checks with unlimited defaults. Formatting, denied-warning
Clippy, dependency/fixture policy and all offline data checks also pass.

## Nested fixed lookbehind word-boundary fixture review

The unchanged whole lookBehind/word-boundary.js original adds two normal/strict
harness-positive variants at the existing pin. All four assertions are preserved:
word/non-word boundary matches, composed nested lookbehind and an exact null
result. Every source byte, metadata field, specification quotation and V8
copyright notice is retained. Git blob identity and manifest SHA-256 digest
verify the complete original; the existing compareArray.js include and all eleven
harness helpers remain unchanged.

All seventeen whole lookBehind originals were reviewed without isolating cases.
The selection now contains three of seventeen; fourteen originals still need
captures, variable counts, alternatives, references or other nested assertions
and receive no passing credit. Stable, MSRV and Node pass both complete Script
variants under unlimited defaults. The pin, dependencies and existing negative
expectations remain unchanged.

The active corpus has 7118 fixtures, eleven helpers, 7099 reviewed Script sources
and 13676 variants: 12199 harness positives, 1469 parse negatives, four raw
positives and four runtime negatives. RegExp builtin execution contains 271 whole
positive programs, separately from 337 grammar fixtures.

Stable and MSRV pass the complete 13676-variant corpus, workspace targets
and documentation checks with unlimited defaults. Formatting, denied-warning
Clippy, dependency/fixture policy and all offline data checks also pass.

## Fixed lookbehind whole-program fixture review

Two unchanged complete originals, lookBehind/simple-fixed-length.js and
lookBehind/negative.js, add four normal/strict harness-positive variants at the
existing pin. Their sixteen positive/failure assertions and twelve negative
lookbehind assertions check complete String.match results, classes, dot, fixed
character/count combinations, whole input positions and required null outcomes.
Every original assertion, metadata field, specification quotation and V8 copyright
notice is preserved. The negative-lookbehind program is an executed positive
fixture; existing parse-negative expectations are unchanged.

All seventeen whole lookBehind originals were reviewed without isolating cases.
The selection now contains two of seventeen; the other fifteen need captures,
variable counts, alternatives, references or nested assertions and receive no
passing credit. Stable, MSRV and Node pass both complete originals in both Script
modes under unlimited defaults. Git blob identities and manifest SHA-256 digests
verify exact source bytes; compareArray.js and all eleven helpers are unchanged.
The pin and dependencies remain unchanged.

The active corpus has 7117 fixtures, eleven helpers, 7098 reviewed Script sources
and 13674 variants: 12197 harness positives, 1469 parse negatives, four raw
positives and four runtime negatives. RegExp builtin execution contains 270 whole
positive programs, separately from 337 grammar fixtures.

Stable and MSRV pass the complete 13674-variant corpus, workspace targets
and documentation checks with unlimited defaults. Formatting, denied-warning
Clippy, dependency/fixture policy and all offline data checks also pass.

## Input, line and word assertion fixture review

All 44 unchanged whole originals in the S15.10.2.6 assertion cohort add 88
normal/strict harness-positive variants at the existing pin. Five end-assertion
programs, ten start-assertion programs, fifteen word-boundary programs, eight
non-boundary programs, two repeated-assertion programs and four combined-assertion
programs check complete exec/test results. Cases include ordinary/multiline and
global flags, canonicalized inverted classes, literal/Unicode string escapes,
quantified classes, greedy/lazy endpoint selection and captured colon/end choices.

Every original assertion, metadata field and Sputnik copyright notice is
preserved, including historical descriptions that differ from their source.
Pinned Git blob identities and manifest SHA-256 digests verify all complete source
bytes. The same eleven unchanged helpers suffice. Stable, MSRV and Node pass all
88 complete variants under unlimited defaults. The pin, earlier negative
expectations and dependencies are unchanged. This completes the 44-file assertion
cohort and makes no whole-RegExp-directory coverage claim.

The active corpus has 7115 fixtures, eleven helpers, 7096 reviewed Script sources
and 13670 variants: 12193 harness positives, 1469 parse negatives, four raw
positives and four runtime negatives. RegExp builtin execution contains 268 whole
positive programs, separately from the 337 grammar fixtures.

Stable and MSRV pass the complete 13670-variant corpus, workspace targets
and documentation checks with unlimited defaults. Formatting, denied-warning
Clippy, dependency/fixture policy and all offline data checks also pass.

## Zero-width lookahead repetition fixture review

One unchanged whole original, lookahead-quantifier-match-groups.js, adds two
normal/strict harness-positive variants at the existing pin. All four original
assertions check required captures versus undefined optional captures through
String.match: unquantified, ?, {1,1} and {0,1} lookahead wrappers. Its source,
metadata, specification quotation and copyright notice are preserved.

The pinned Git blob identity and manifest SHA-256 digest verify exact bytes.
The existing compareArray.js dependency and all eleven helpers remain unchanged.
Stable, MSRV and Node pass both complete Script variants with unlimited defaults.
The pin, earlier negative expectations and dependencies are unchanged.

The active inventory is 7071 fixtures, eleven helpers and 7052 reviewed Script
sources. Its 13582 variants comprise 12105 harness positives, 1469 parse negatives,
four raw positives and four runtime negatives. RegExp builtin execution now
contains 224 whole positive programs, separately from 337 grammar fixtures.
This focused original adds no whole-directory coverage claim.

Stable and MSRV pass the complete 13582-variant corpus, workspace targets
and documentation checks with unlimited defaults. Formatting, denied-warning
Clippy, dependency/fixture policy and all offline data checks also pass.

## Repeated RegExp branch capture fixture review

Seven further unchanged whole originals add fourteen normal/strict harness-positive
variants at the existing pin. Three named-group programs check duplicate-name
exec/match/test behavior, references within repeated branches and final undefined
aliases after another iteration. Four exec programs check ordered coercion,
variable-width source-order capture selection, nested optional quantifiers and
cleared final slots, plus duplicate-name property order and indices groups.
Every original assertion, copyright notice and metadata field is preserved.

Git blob identities and manifest SHA-256 digests verify all seven files. Their
existing compareArray.js dependency and all eleven helpers remain byte-identical.
The pin, negative expectations, dependencies and unlimited defaults are unchanged.
Named-group execution now includes 21 of 36 whole originals, with 15 exclusions;
exec includes 75 of 79, with four exclusions. Remaining originals need lookaround
or Unicode-mode matching and receive no passing credit.

The current inventory is 7070 fixtures, eleven unchanged helpers and 7051 reviewed
Script sources. Its 13580 variants comprise 12103 harness positives, 1469 parse
negatives, four raw positives and four runtime negatives. RegExp builtin execution
includes 223 whole positive programs, separately from the 337 grammar fixtures.

Stable and MSRV pass all fourteen selected variants and the complete 13580-variant
corpus, workspace targets and documentation checks under unlimited defaults.
Formatting, denied-warning Clippy, dependency policy and all offline data checks
also pass. Node executes every original in both Script modes.

## Ordinary RegExp fallback fixture review

The unchanged whole-program review revisits all 36 named-group originals and all
79 exec originals at the existing pin. Eighteen further complete originals add
36 normal/strict harness-positive variants. The named-group original checks valid
non-Unicode names, including astral identifier characters and escapes, complete
capture arrays, greedy/lazy dot prefixes and named references. It adds no Unicode
matching credit. Seventeen exec originals check nested source-order captures,
choice continuations with optional digits, all eighteen matches through the
original poem, ordered global lastIndex conversion (including abrupt conversion),
and negative/out-of-input positions. Every original assertion and metadata field
is preserved; none is isolated or rewritten to obtain passing credit.

Git blob identities and manifest SHA-256 digests verify all eighteen new files.
All eleven unchanged helpers were already present; the pin, negative expectations,
dependencies and unlimited runtime defaults are unchanged. The named selection
is now 18 of 36 whole originals, with 18 exclusions; exec is 71 of 79, with eight
exclusions. Exclusions require general repeated choices/nested quantifiers,
lookbehind or Unicode-mode matching and receive no passing credit.

The corpus now contains 7063 fixtures, eleven helpers and 7044 reviewed Script
sources. Its 13566 variants comprise 12089 harness positives, 1469 parse negatives,
four raw positives and four runtime negatives. RegExp builtin execution coverage
contains 216 whole positive programs, separately from the 337 grammar fixtures.

## Native named RegExp capture review

All 36 whole programs and their metadata in built-ins/RegExp/named-groups were
reviewed at the existing pin. Fifteen unchanged originals add 30 normal/strict
harness-positive variants. Thirteen programs exercise named result groups and
indices, duplicate-name property order, matchAll, replace/replaceAll, search and
split, and generic own/inherited groups from custom exec methods. Two programs
check invalid group-name SyntaxErrors; they add no Unicode matching credit.

The unchanged compareIterator.js helper is newly vendored from the same pin,
with its original copyright and BSD notice. The matchAll program uses it to
check every iteration result and completion. All other required helpers were
already present. Git blob identities and manifest SHA-256 digests verify every
new source byte, assertion and metadata field. No test body or expectation is
rewritten, and no runtime allowance or default limit is added.

Native ordinary reference compositions additionally enable the unchanged
non-unicode-match.js original and two normal/strict harness-positive variants.
Its complete assertions cover dot/class/literal captures, named and numbered
references, captures containing references, source-order groups, and String.match
arrays. The native named-directory selection is now 17 of 36 complete originals;
Git blob and SHA-256 identities preserve its full source and metadata.

The unchanged non-unicode-references.js original adds two further normal/strict
harness-positive variants. Its complete program checks named references, failed
matches, self and forward references, references to multiple captures, groups
properties and an inactive named group in another top-level branch. The native
named-directory selection is now 17 of 36 complete originals.

Nineteen whole originals remain excluded without passing credit because they
require Unicode-mode matching, lookbehind or additional quantified patterns. The separate exec inventory remains 54 of 79 originals,
with 25 exclusions. The corpus now has 7045 fixtures, eleven helpers and 7026
reviewed Script sources: 13530 variants comprising 12053 harness positives,
1469 parse negatives, four raw positives and four runtime negatives.

## Native literal RegExp exec review

The directory review covers all 79 whole programs at the existing pin. With
native top-level literal alternatives, seven further unchanged originals add
fourteen normal/strict positives: S15.10.6.2_A1_T1, T9, T10, T11, T14, T17 and T18.
They verify source-order ties, earliest-position selection, match Array contents,
and function/Number/Boolean/null/undefined input conversion. Fifty-four directory
programs are now vendored, including the prior not-a-constructor original; 25
still need broader execution and remain excluded. Both compilers pass all added
variants with the ten original helpers and unlimited defaults. No source,
assertion, helper, metadata or pin is changed to earn credit.

One-unit dot/literal choices enable the unchanged complete
S15.10.6.2_A12.js original and two normal/strict positives. After an earlier
`/foo/` test, its original `(.|\r|\n)*` constructor verifies that `exec()` converts
an absent argument to `"undefined"` rather than reusing prior matching input.
The complete body, assertions and metadata were read and compared with the
pinned upstream source; its Git blob and manifest SHA-256 verify the exact bytes.
Both Script modes use the existing unchanged helpers and unlimited defaults.
The directory selection is 54 of 79 whole originals, with 25 exclusions receiving
no credit. No pin, helper or negative expectation changes.

The ordinary dot matcher enables three further unchanged whole programs:
success-g-lastindex-no-access.js, success-lastindex-access.js and
y-init-lastindex.js. Their six normal/strict positives verify ordered lastIndex
conversion, global writes, nonglobal reads without writes and sticky start
positions. Both compilers pass all six variants with original assertions,
metadata, helpers, pin and unlimited defaults. The remaining directory selection
receives no passing credit. Iterative binary-chain parsing lets all three
previously depth-limited concatenation programs parse normally. Fixed sequences
enable S15.10.6.2_A3_T2; quantified literal continuations enable T3. T4 still needs
nested choices and remains excluded without passing credit.

Fixed ordinary sequences add four unchanged whole programs and eight normal/strict
positives: S15.10.6.2_A1_T13, T20, T21 and A3_T2. They check Boolean/undefined
input conversion, intrinsic match Array contents/index/input, class ranges and
eleven successive global matches through the complete original poem. All source
bytes, assertions, metadata, ten helpers and the pin are unchanged. Both compilers
pass all eight variants with unlimited defaults. The reviewed exec directory now
has 54 vendored programs and 25 excluded programs, with no credit for exclusions.

Single-atom quantifiers add three unchanged whole programs and six normal/strict
positives: S15.10.6.2_A1_T19, A3_T5 and A3_T7. They verify undefined input conversion
with exact repetition, three successive global digit runs, and ten nonglobal
calls returning the same first run. Both compilers pass every selected variant
with unlimited defaults. Git blob hashes and manifest SHA-256 digests verify the
original bytes; assertions, metadata, the ten helpers and pin are unchanged.
The exec-directory inventory is now 54 vendored programs and 25 exclusions out
of 79 complete reviewed originals, with no passing credit for exclusions.

Quantified literal continuations add the unchanged complete S15.10.6.2_A3_T3
program and two normal/strict positives. Its original poem requires twelve
successive global [Nn]?evermore matches, including evermore without an initial N.
Both compilers pass both variants with unlimited defaults, original assertions,
metadata, ten helpers and pin. Git blob and manifest digests verify the original
bytes. The reviewed exec directory now has 54 vendored originals and 25 excluded
originals out of 79 complete programs, with no credit for exclusions.

Quantified captures add the unchanged complete S15.10.6.2_A3_T6 program and two
normal/strict positives. Its original /(\d+)/g pattern finds three successive
digit runs through the original loop and assertions. Both compilers pass both
variants with unlimited defaults, original metadata, ten helpers and pin. Git
blob and manifest digests verify the exact source bytes. The exec directory now
has 54 vendored originals and 25 excluded originals out of 79 reviewed programs,
with no credit for exclusions.

Literal prefixes add the unchanged complete S15.10.6.2_A1_T3 and T4 programs
and four normal/strict positives. Their original greedy and lazy bounded patterns
check boxed String and custom toString input conversion, match Array contents,
index and input. Both compilers pass all four variants with unlimited defaults,
original assertions and metadata, the ten helpers and the same pin. Git blob and
manifest digests verify the exact bytes. The exec inventory is now 54 vendored
originals and 25 excluded originals out of 79 reviewed programs, with no passing
credit for exclusions.

A further focused review reads 25 whole exec programs after literal evaluation
became available. Twelve unchanged originals add 24 normal/strict positives for
throwing input coercions, numeric/boolean/absent input conversion, ignore-case
matching, global failure and past-end resets, nonglobal lastIndex reads without
writes, sticky failures and successful end-index writes, and non-writable
lastIndex rejection. At that stage, thirteen reviewed programs required alternatives, character
classes, quantifiers or wildcard matching and remained excluded. The later
directory-wide inventory above records the current selection. Both compilers execute every selected variant
with the original assertions, ten unchanged helpers and unlimited defaults.

A focused review reads twenty whole programs in the pinned RegExp.prototype.exec
area. Eighteen newly vendored originals add 36 normal/strict positives for native
literal search results and Array stringification, exec name/length/descriptors,
construction rejection and own-brand checks across object, function, primitive,
wrapper and undefined receivers. The prior not-a-constructor program is unchanged.
One whole program still needs broader matching and
remains outside the corpus. This is a focused review, not a complete directory
review. Both compiler versions pass every selected program with ordinary unlimited
defaults. The existing pin and ten harness helpers are unchanged.

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

## Date timestamp and UTC operation review

162 unchanged whole programs add 324 positive Script variants, running in both
normal and strict modes. They cover constructor metadata, timestamp clipping and
Date copying, observable primitive conversion and abrupt completions, Date.now,
UTC/explicit-offset parsing with both time-range endpoints and rejected expanded
negative zero, UTC getters at millisecond/calendar boundaries, branded receivers,
setTime mutation and conversion errors, ISO metadata/branding/RangeErrors, generic
JSON conversion and callback context, primitive hint ordering, and invalid-date
strings. Original sources, assertions and metadata are unchanged.

The focused review read 177 complete programs. At that step, fourteen required pending numeric
calendar construction, local time zones or finite local/legacy strings; one uses
the unchanged property helper to enumerate the incomplete global object. Those
whole programs were held out at that step; subsequent Date reviews below expand
the corpus. This is not a complete Date directory review. The existing pin,
eight harness helpers and unlimited runtime defaults
are unchanged; pending Date bodies remain explicit Unsupported failures.

A follow-up review read all 36 whole files in the UTC hour/minute/second/
millisecond setter directories. 28 unchanged programs add 56 normal/strict
positives for intrinsic metadata, ordered conversion before invalid-date checks,
and capturing the original time before hooks mutate the Date. Invalid-date hooks
may install a valid time that survives the setter's NaN return. Eight whole
programs required numeric Date.UTC construction and were held out for follow-up.
This addition uses the same pin and harness files.

A UTC string review read all nine whole programs in toUTCString. Seven newly
vendored originals add 14 normal/strict positives for weekday/month names,
negative-year widths and intrinsic metadata. The invalid-date program was
already included; the format program still requires pending RegExp.exec and
remains outside the corpus. The pin and eight harness files are unchanged.

The setUTCDate review read all seven whole programs in its directory. All seven
unchanged originals add 14 normal/strict positives for argument conversion order,
capturing the stored time before mutating hooks and intrinsic metadata. The pin
and eight harness files are unchanged.

A numeric UTC review read all 17 whole Date.UTC programs, then revisited the eight
UTC time setter rollover programs previously held out for Date.UTC. All 25
unchanged originals add 50 normal/strict positives for coercion order/errors,
absent defaults, short years, fractional/non-finite components, clipping, ordered
floating arithmetic, metadata and UTC time rollover. All 36 programs in the four
UTC time setter directories are now included. The pin and eight harness files
are unchanged.

The UTC calendar setter review read all nine whole setUTCMonth programs and all
six setUTCFullYear programs. All 15 unchanged originals add 30 normal/strict
positives for ordered coercion, captured valid/invalid time values, literal short
years, month/day rollover and intrinsic metadata. The pin and eight harness
files are unchanged.

An invalid-Date review adds 29 unchanged whole programs and 58 normal/strict
positives: two constructor-infinity ISO RangeError programs from the earlier
held-out group, nine invalid-time local getter programs, and eighteen local
setter programs for invalid return values, conversion order and captured-time
mutation. Finite local operations and local setFullYear remain pending. This is
a focused invalid-branch review; the pin and eight harness files are unchanged.

A local calendar review read 127 previously unvendored whole programs at the same
pin. 119 unchanged originals add 238 normal/strict positives for numeric calendar
construction, coercion order and errors, short-year adjustment, month/day rollover,
subclass prototypes, date-only UTC versus local date-time parsing, and all nine
local getters. Each getter directory is now included in full. The cohort passes
under UTC, America/New_York, Australia/Lord_Howe and Europe/Paris without source
rewrites or test-only quotas. The unchanged upstream assertRelativeDateMs.js and
dateConstants.js helpers bring the harness inventory to ten.

Eight whole programs remain outside this selection: three constructor tests need
cross-realm host hooks; S15.9.2.1_A1.js, S15.9.2.1_A2.js and
value-to-primitive-get-meth-err.js call Date as a function; parse/zero.js needs own
local string output and parsing; and prop-desc.js asks the property helper to
enumerate the incomplete global object. These exclusions receive no pass credit.

The local setter review read all 105 remaining whole programs across setDate,
setFullYear, setMonth, setHours, setMinutes, setSeconds and setMilliseconds. All
105 originals are now vendored unchanged, adding 210 normal/strict positives for
component conversion, defaults and rollover, captured valid/invalid values,
setFullYear revival, final clipping, branding and intrinsic metadata. Together
with the eighteen earlier invalid-branch programs, all 123 files in these seven
directories are included. The cohort passes on MSRV and stable under UTC,
America/New_York, Australia/Lord_Howe and Europe/Paris. The existing pin, ten
harness helpers, source bytes and unlimited runtime defaults are unchanged.

The local string review read all 21 whole programs across toString,
toDateString and toTimeString, then revisited four previously held-out Date()
and own-string parsing programs. Nineteen newly vendored originals add 38
normal/strict positives for function-call output and ignored values, own-output
parsing, negative-year padding, receiver branding and intrinsic metadata. Three
invalid-date originals were already included. The three format.js programs still
require RegExp.exec and remain excluded with no pass credit. All 38 new variants
pass on both compilers under UTC, America/New_York, Australia/Lord_Howe and
Europe/Paris. The pin, ten harness helpers, original bytes and unlimited defaults
are unchanged.

The locale string review includes all twelve unchanged whole programs across
toLocaleString, toLocaleDateString and toLocaleTimeString. Their 24 normal/strict
positives verify length, name, property descriptors and non-constructibility.
These are metadata regressions for the non-Intl methods; they claim no ECMA-402
coverage. The cohort passes on both compilers under UTC, America/New_York,
Australia/Lord_Howe and Europe/Paris. The pin, ten helpers, original sources and
unlimited defaults are unchanged.

## RegExp escape review

The focused review read all twenty whole programs in RegExp/escape at the existing
pin. Nineteen unchanged originals add 38 normal/strict positives for leading
ASCII protection, syntax/other punctuators, controls, line terminators and white
space, unpaired surrogates, preserved ordinary Unicode, String-only inputs, and
intrinsic metadata. Both compilers pass every selected variant. cross-realm.js
requires the pending $262.createRealm hook and remains excluded without pass
credit. The pin, ten helpers, source bytes and unlimited defaults are unchanged;
this cohort claims no construction or matching coverage.

## Generic RegExp operation review

The review read all sixteen flags programs and all nine toString programs at the
same pin. Twenty-two unchanged originals add 44 normal/strict positives: fourteen
flags files verify ordered gets, truthiness, abrupt completions, receiver checks,
the ordinary prototype and metadata; eight toString files verify intrinsic
metadata and strict receiver semantics. Both compilers pass every selected
variant. flags/return-order.js and flags/this-val-regexp.js still need RegExp
literal execution. The construction review below adds toString/not-a-constructor.js
unchanged. The pin, ten helpers, original bytes and unlimited defaults
are unchanged; these programs claim no native matching coverage.

## RegExp match/search review

The review read all fifty-three Symbol.match programs and all twenty-three
Symbol.search programs at the existing pin. Twenty-one unchanged originals add
42 normal/strict positives: seven match programs cover intrinsic metadata,
receiver checks, flags/exec getters and missing matcher slots; fourteen search
programs cover custom-exec result validation, uncoerced indices, exact lastIndex
reads/writes, restoration, abrupt completions and metadata. Both compilers pass
all selected variants. The construction review below adds both not-a-constructor
programs unchanged. The other fifty-three whole programs require RegExp literal
execution or native matching and remain excluded without rewriting or
pass credit. The pin, ten helpers and unlimited defaults are unchanged; this
cohort claims no native matcher coverage.

## RegExp replacement review

The review read all seventy whole Symbol.replace programs at the existing pin.
Six unchanged originals add twelve normal/strict positives for receiver checks,
flags/exec getter failures, global lastIndex initialization and intrinsic metadata.
Both compilers pass every selected variant. The construction review below adds
not-a-constructor.js unchanged. The other sixty-three programs require RegExp
literal execution or native matching and remain excluded without source rewriting or pass credit.
The pin, ten helpers, original bytes and unlimited defaults are unchanged; this
cohort claims no native matcher coverage.

## RegExp splitting review

The review read all forty-four whole Symbol.split programs at the existing pin.
Twenty-one unchanged originals add 42 normal/strict positives for custom species
construction, flags conversion, zero limits, empty input, lastIndex reads/writes,
length coercion, abrupt capture access, receiver checks and intrinsic metadata.
Both compilers pass every selected variant. The construction review below adds
not-a-constructor.js unchanged. The other twenty-two programs need RegExp literal
execution or native matching; splitter-proto-from-ctor-realm.js also needs
cross-realm support. They remain excluded as whole files without source rewriting
or pass credit, including the native exec used by species-ctor-ctor-non-obj.js's
initial baseline call. The pin, ten helpers, original
bytes and unlimited defaults are unchanged; no native matcher coverage is claimed.

## RegExp matchAll and string iterator review

The review read all twenty-six whole Symbol.matchAll programs and all seventeen
RegExpStringIteratorPrototype programs at the existing pin. Four unchanged
matchAll originals add eight normal/strict positives for name/length/property
metadata and primitive receivers. Both compilers pass every selected variant.
The construction review below adds not-a-constructor.js unchanged. The other
thirty-eight programs require RegExp literal execution or native matching,
including every iterator original. They remain excluded as whole files without source rewriting or
pass credit. Historical descriptions sometimes refer to earlier draft algorithms;
selected assertions match edition 17. The pin, ten helpers, original bytes and
unlimited defaults are unchanged; no native construction or matching is claimed.

## RegExp construction and source review

The focused review reads nineteen constructor programs, all twelve source getter
programs, and seven prototype not-a-constructor programs at the existing pin.
Twenty-eight unchanged originals add 56 normal/strict positives for construction,
copying, regexp-like identity and getter errors, flags, lastIndex, source metadata
and receiver brands, and prototype non-construction. Both compilers pass every
selected variant. The ten excluded whole files need literal execution, native
matching, cross-realm support, or global own-key reflection. No source fragments
are rewritten. This is a scoped constructor review, not a complete review of the
RegExp root directory. Historical descriptions refer to older algorithms or flag
sets; selected assertions remain valid for edition 17. The pin, ten helpers,
original bytes and unlimited defaults are unchanged; no native matching coverage
is claimed.

## RegExp lexical boundary review

Thirty-four unchanged originals add 68 variants: eighteen harness positives and
sixteen reviewed parse negatives, each in both Script modes. Positive programs
verify that `//` starts a comment and that eval throws SyntaxError for forbidden
line terminators in literal bodies and backslash sequences. Negative programs
reject the intended malformed tokens, unterminated comment, or leading dot after
an empty-literal comment; exact diagnostic messages and byte spans are recorded.
Historical CR-labelled files containing LF retain their original bytes and are
checked at the actual LF. Native regressions independently cover all four line
terminators and UTF-16 preservation.

The focused review read 53 complete legacy/boundary programs. Eighteen whole files
require RegExp execution, including one negative whose initial complete literal
would otherwise mask the intended later error. One paragraph-separator program
contains an earlier line separator, so it cannot verify the intended separator
and is excluded. This is not a complete RegExp directory review. No Pattern,
flag-validity, matching, or RegExp object execution receives credit. Original
sources, assertions, metadata, the existing pin, eight helpers, and unlimited
runtime defaults are unchanged.

## RegExp literal flag review

Two unchanged originals add four reviewed parse-negative variants, each in both
Script modes. They reject the uppercase `G` flag and the repeated `g` in `gig`
with exact literal spans and diagnostics. These flag checks precede Pattern
parsing in IsValidRegularExpressionLiteral and do not require RegExp execution.
Both whole programs and their metadata were reviewed; all source bytes,
assertions, the pin, eight helpers, and unlimited defaults are unchanged. Native
regressions separately cover every allowed flag subset and the u/v exclusion.

## RegExp core Pattern review

106 unchanged originals add 212 reviewed parse-negative variants in the two
Script modes. They cover scoped modifier grammar and its duplicate/overlap/empty
list early errors, quantifier placement and reversed decimal bounds, forbidden
assertion quantifiers, control/identity/Unicode escapes, and numbered references
beyond the final capture count. Every reviewed error checks the complete literal's
byte span and the exact diagnostic. These are supported Pattern rejections, with
no matching or RegExp object execution credit.

All 112 candidate programs and metadata were reviewed. Four character-class range
cases were excluded from this core selection. Two escaped overlap cases are
excluded because the forbidden modifier escape masks the intended enabled/disabled overlap; separate
originals directly test forbidden modifier escapes. Historic descriptions and
unexpanded global-modifier placeholder comments retain their original bytes.
Both toolchains pass the selected variants with no other outcome. The pin, all
assertions and metadata, eight helpers, runtime defaults and native guards are
unchanged.

## RegExp ordinary character-class range review

Four unchanged originals add eight reviewed parse-negative variants, covering
set-valued first, second and paired range endpoints, including a literal dash
endpoint. Each complete program and its metadata were reviewed; exact literal
byte spans and diagnostics verify the intended core grammar early errors. These
four files were excluded from the earlier 112-program core Pattern selection.
Historic runtime-semantics descriptions and Annex B notes retain their bytes.
Both toolchains pass every selected variant without matching or RegExp object
execution credit. Source bytes, assertions, metadata, the pin, eight helpers,
unlimited defaults and native guards are unchanged.

## RegExp Unicode property escape review

136 unchanged parse-negative originals add 272 Script/StrictScript variants for
property-expression grammar, exact property/value spelling, forbidden binary
values, excluded properties, missing nonbinary values, and set-valued class-range
endpoints. All 144 whole programs and metadata in the property-escapes directory
were reviewed at the existing pin. Two positives require matching. Six negatives
escape the backslash itself, so their quantifier error masks the intended unknown
property-value rejection; they are excluded without altering upstream bytes.
Historical clause names and descriptions are retained. Exact literal spans and
diagnostic messages verify each selected rejection on both toolchains. This adds
no matching or RegExp object execution credit and changes no default quotas,
dependencies, or harness files.

## RegExp named capture review

55 unchanged originals add 110 variants: 54 reviewed parse negatives in both
Script modes and one harness positive whose four eval calls reject lone surrogate
group names as SyntaxError. Parse reviews check exact complete literal spans and
diagnostics for missing references, duplicates within an alternative, malformed
names and Unicode escapes, invalid identifier code points, and one invalid
identity escape inside a named capture. Historical descriptions and clause names
retain their original bytes.

All 56 complete programs and metadata in the literal named-groups directory were
reviewed. The forward-reference execution original still requires matching and
is excluded; native regressions separately validate that grammar. This selection
adds no matching or RegExp object execution credit. Both toolchains pass every
selected variant with the original sources, assertions, metadata, existing pin,
eight helpers, unlimited defaults and native guards unchanged.

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
The 430 String files cover wrappers, raw construction, character
access, searches, concatenation, substrings, trimming, repetition, padding, Unicode well-formedness, UTF-16 encoding/decoding,
and ordered conversions. Each runs in both required Script
modes. The `harness` manifest mode verifies support-file bytes without counting
them as test cases;
`script-pass` selects non-raw positive tests. Harness frontmatter describes helper
definitions and is not parsed as test execution metadata.

Local controls execute successful assertions and deliberately failing assertions
against these exact harness files. Test262Error construction, native Error
construction, and built-in exception catch/constructor checks execute normally.
String comparison failure formatting now executes JSON.stringify and produces
ordinary assertion failures. Array.fromAsync, additional includes, async completion,
and agent helpers remain separate harness gaps.

The 713 Array and Array iterator files cover call/new construction, of, branding, literal elisions,
indexed growth, truncation, generic at/join/push/pop, toString/toLocaleString, and ordered
forEach/every/some callback traversal, find/findIndex/findLast/findLastIndex,
includes/indexOf/lastIndexOf searches, reduce/reduceRight accumulators, species-aware map/filter/slice/concat/flat/flatMap/splice, sparse reverse, fill/copyWithin range mutations, shift/unshift front mutations, and sort/toSorted and toReversed/with/toSpliced copies, plus keys/values/entries
iteration and live mapped/unmapped arguments. The 430 String files now include
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

The later Array species/intrinsic review adds the species/non-construction
candidates using `isConstructor.js` and Reflect. Other reviewed candidates
requiring foreign realms, proxies, or resizable buffers remain outside this corpus. Their
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

Remaining concat candidates require proxies, foreign realms, typed arrays, or
RegExp. The later Array species/intrinsic review adds the class, spreadable-function,
constructor-helper, and property-helper candidates that can now run unchanged. The
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

Remaining flat/flatMap candidates require typed arrays or proxies. The later
Array species/intrinsic review adds the constructor-helper and property-helper
candidates that can now run unchanged. Local runtime regressions
cover metadata and descriptors, sparse live mutations, aliased species results,
safe-integer overflow, and iterative traversal of deeply nested Arrays.

## Splice review

The 66 files added at the existing pin add 132 variants, with every file running
in both Script modes. They cover positive/negative/omitted deletion ranges,
fractional/NaN/infinite conversions, generic objects, inherited properties,
full-width indices near 2^53-1, shrinking/growing sparse movement, constructor
and species failures/fallback, custom species identity and arguments, failed
result definitions, and strict final length writes including zero arguments.

Remaining splice candidates require foreign realms or proxies, including the
unchanged `proxyTrapsHelper.js` include. The later Array species/intrinsic review
adds the for-in, Math.pow, constructor-helper, and property-helper candidates that
can now run unchanged. These remaining candidates stay outside
this corpus. Sources are unchanged; no default quotas or test allowances are
introduced. Local runtime regressions also cover species aliasing, partial
mutations on failure, and complete Array prototype reflection/integrity operations.

## Array species and intrinsic review

Thirty-six additional unchanged originals add 72 Script/StrictScript variants:
two each for map, filter, and slice, eleven concat, five flat, six flatMap, and
eight splice. They cover non-constructor species and method construction rejection,
custom species/new.target, replacement of configurable non-writable result indices,
inherited read-only indices, native metadata/descriptors, a non-Array class receiver,
spreadable functions, splice result descriptors, and Array length rejection before
source mutation. They use the existing unchanged constructor, property, and Array
comparison helpers. The 65 whole-program reviews retain 36 originals and exclude
29 requiring foreign realms (ten), proxies (fourteen), typed arrays (four), or
RegExp (one). This completes the supported selection from the concat, flat,
flatMap, and splice directories at the existing pin; the map/filter/slice revisit
is limited to their constructor/species/non-construction candidates. No source,
assertion, flag, helper, pin, runtime default, or resource allowance is changed.

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
candidates requiring dynamic Function construction or Date remain outside this
corpus. The Reflect metadata review below adds the property-helper files. The
existing pin and execution quotas remain unchanged.

## Reflect prototype and extensibility review

Twenty-five unchanged files add 50 variants in both Script modes: six
getPrototypeOf, ten setPrototypeOf, four isExtensible, and five preventExtensions
files. They cover object-only targets, prototype identity, cycles, non-extensible
changes, idempotence, and non-constructibility using the pinned isConstructor
helper. The preventExtensions Symbol-target fixture checks isExtensible in its
original source; local regressions check all four methods directly. Reviewed
Proxy candidates remain outside this corpus; the Reflect metadata review below
adds the property-helper files. The pin, harness bytes, and execution quotas
remain unchanged.

## Reflect property read, presence, and deletion review

Twenty-one unchanged files add 42 variants in both Script modes: eight get, six
has, and seven deleteProperty files. They cover ordered key conversion, explicit
read receivers, getters and inherited reads, symbol keys, boolean deletion,
primitive target rejection, and non-constructibility. Reviewed Proxy candidates
remain outside this corpus; the Reflect metadata review below adds the
property-helper files. Local regressions also
cover key-conversion mutations, exact primitive accessor receivers, dormant
getters, Array/String/arguments exotics, metadata, collection, and host failures.
The pin, harness bytes, and execution quotas remain unchanged.

## Reflect descriptors and own keys review

Twenty-five unchanged files add 50 variants in both Script modes: seven
defineProperty, nine getOwnPropertyDescriptor, and nine ownKeys files. They cover
ordered conversions, boolean definitions, data/accessor descriptor fields, symbol
keys, non-enumerables, chronological order and large indices, primitive target
rejection, and non-constructibility. Reviewed Proxy candidates remain outside
this corpus; the Reflect metadata review below adds the property-helper files.
Local regressions cover Array partial
truncation, fresh descriptors, setter/species bypass, exotics, collection, and host
gaps. The pin, harness bytes, and execution quotas remain unchanged.

## Reflect set review

Fourteen unchanged files add 28 variants in both Script modes. They cover
ordered key conversion, data and accessor writes, explicit receivers, boolean
rejections, prototype setters, symbol keys, primitive target rejection, and
non-constructibility. The Reflect metadata review below adds the property-helper
files. Local regressions cover Array partial truncation, String/arguments
receivers, ordinary assignment, missing descriptors before mutation, and complete
Reflect own reflection/integrity operations. The pin, harness bytes, and execution
quotas remain unchanged.

## Math basic numeric review

The forty unchanged Math sources comprise five abs, two sign, eight ceil, eight
floor, eight round, and nine trunc fixtures. Their eighty Script/StrictScript
variants cover special values, signed zero, fractional rounding, constructor
rejection, values adjacent to halfway boundaries, and large odd integers. The
complete Math reflection review below adds these Math-property descriptor files;
the metadata review adds the name/length files. Local regressions cover fixed
constants and attributes, ordered single numeric conversion, ignored extra
arguments, exact half ties, and intrinsic retention during collection. The
upstream pin, harness bytes, and opt-in execution limits are unchanged.

## Math extrema and integer review

The twenty-three unchanged sources comprise seven max, seven min, seven clz32,
and two imul fixtures. Their forty-six Script/StrictScript variants cover empty
calls, NaN results after argument conversion, signed-zero extrema, unsigned
32-bit wrapping and bit counts, modular signed products, and constructor
rejection. The complete Math reflection review below adds these Math-property
descriptor files; the metadata review adds the name/length files. Local
regressions add ordered abrupt conversions, exact receiver/extra-argument
behavior, large lists with default limits, integer boundaries, descriptors, host
failures, and collection. The pin, harness bytes, and opt-in execution limits
are unchanged.

## Math binary32/binary16 rounding review

The eight unchanged sources comprise six fround and two f16round fixtures, with
sixteen Script/StrictScript variants. They cover NaN, infinities, signed zero,
binary32 halfway ties, rounding conversions, and constructor rejection. The
unchanged `byteConversionValues.js` is the fifth harness file; its binary16
table covers exact normal/subnormal boundaries, midpoint neighbors, signed
underflow, and overflow. Local controls execute both floating-point tables and
deliberately failing assertions. The complete Math reflection review below adds
these Math-property descriptor files; the metadata review adds the name/length
files. Float16Array feature annotations describe this shared proposal; these
selected tests exercise Math.f16round without constructing a Float16Array. Local
regressions additionally enumerate every finite binary16 value and adjacent
rounding boundary for both signs. The pin and opt-in execution limits are
unchanged.

## Math power and square-root review

The thirty-two unchanged sources comprise twenty-five pow and seven sqrt
fixtures, with sixty-four Script/StrictScript variants. They cover
exponentiation special values, signed zeros and infinities, odd/even integral
exponents, negative fractional bases, very large exponents, and constructor
rejection. The sqrt results fixture verifies one thousand exact square-root
pairs over a broad binary64 range. The complete Math reflection review below
adds these Math-property descriptor files; the metadata review adds the
name/length files. Local regressions add ordered abrupt coercion before numeric
shortcuts, receiver and extra-argument behavior, primitive type errors, range
endpoints, descriptors, host aborts, and collection. The pin, harness bytes, and
opt-in execution limits are unchanged.

## Math exponential and logarithmic review

The twenty unchanged sources comprise six exp, six log, and two each of expm1,
log1p, log2, and log10. Their forty Script/StrictScript variants cover NaN,
infinities, signed zero, domain boundaries, exact endpoint results,
representative logarithms, and constructor rejection. The complete Math
reflection review below adds these Math-property descriptor files; the metadata
review adds the name/length files. Local regressions add the full binary64
power-of-two exponent range, small-input accuracy without intermediate
addition/subtraction, ordered conversions, metadata, host aborts, and
collection. The pin, harness bytes, and opt-in execution limits are unchanged.

## Math inverse-function and cube-root review

The thirty-three unchanged sources comprise five acos, four acosh, six asin, two
asinh, four atan, two atanh, eight atan2, and two cbrt fixtures. Their sixty-six
Script/StrictScript variants cover NaN, domain endpoints, signed
zeros/infinities, quadrant behavior, and constructor rejection. The complete
Math reflection review below adds these Math-property descriptor files; the
metadata review adds the name/length files. Local regressions add every
zero/infinity sign combination, ordered conversions before NaN results, finite
approximations, large and subnormal inputs, metadata, host aborts, and
collection. The pin, harness bytes, and opt-in execution limits are unchanged.

## Math function metadata and property-helper review

Fifty-six unchanged name/length sources for the twenty-eight installed Math
methods add 112 Script/StrictScript variants. They verify exact values and
read-only, non-enumerable, configurable descriptors through the unchanged
`propertyHelper.js`, the sixth harness file. The helper retains its captured
primordial functions and mutation/restore behavior. Local controls cover data,
Symbol and accessor properties, callable metadata, restoration, deliberately
wrong values/attributes, and unsupported operations that cannot become passes.
The complete Math reflection review below subsequently adds the twenty-eight
Math-property descriptor files. Other features' property-helper selections await
separate review. The pin and opt-in execution limits are unchanged.

## Math circular and hyperbolic review

Thirty-five unchanged sources comprise seven sin, eight cos, eight tan, and four
each of sinh, cosh, and tanh. Their seventy Script/StrictScript variants cover
NaN, signed zeros, infinity domains, constructor rejection, and standard
function name/length descriptors through the pinned property helper. Local
regressions add finite approximations, large and subnormal inputs, symmetry,
coercion order, type/host failures, ignored extra arguments, and collection. The
complete Math reflection review below adds the six Math-property descriptor
files. The upstream pin, harness bytes, and opt-in execution limits are
unchanged.

## Math hypot review

Eleven unchanged hypot sources add twenty-two Script/StrictScript variants. They
cover empty/all-zero calls, NaN/infinities, Infinity precedence over NaN,
ordered abrupt argument conversion, a 3/4/5 result, constructor rejection, and
standard name/length descriptors. Local regressions add overflow/underflow
avoidance, subnormal norms, compensated small contributions in both argument
orders, primitive type errors, host aborts, and collection. The complete Math
reflection review below adds the Math-property descriptor file. The pin, harness
bytes, and opt-in execution limits are unchanged.

## Math exact summation review

Eight unchanged sumPrecise sources add sixteen Script/StrictScript variants.
They cover signed zero, infinities/NaN, exact finite summation including
previously reported rounding bugs, element rejection without coercion, iterator
closing, constructor rejection, and standard name/length descriptors. The
generator-based iterable candidate remains outside this selection because
generators are not implemented; its source is not rewritten. The complete Math
reflection review below adds the Math-property descriptor file. Local
regressions add synchronous iterator ordering, closing precedence, the
specification count boundary, independent exact rounding references, metadata,
host failures, and collection. The pin, harness bytes, and opt-in execution
limits are unchanged.

## Complete Math reflection and random review

Forty-one unchanged sources add eighty-two Script/StrictScript variants. Five
random sources cover repeated values in [0, 1), constructor rejection, and
standard name/length/property descriptors. The other thirty-six sources verify
each previously installed method's Math-property descriptor using the original
property helper. Complete Math own reflection now supports their enumeration and
mutation checks. All thirty-seven standard methods have name, length, and Math
property descriptor fixtures at the existing pin. Local regressions cover exact
random interval mapping, distinct realm sequences, identity capacity, metadata,
collection, and Math reflection/freezing without getter calls. Other features'
property-helper selections await separate review. The pin, harness bytes, and
opt-in execution limits are unchanged.

## Math constants, tag, and prototype review

Eighteen unchanged sources add thirty-six Script/StrictScript variants: numeric
values and immutable descriptors for all eight constants, the standard
Symbol.toStringTag descriptor, and Object.prototype ancestry. The SQRT1_2
attributes fixture uses the original helper's deprecated descriptor assertions.
The Math global-property descriptor source remains outside this selection because
its helper enumeration reaches the global object's incomplete own reflection;
its source is not rewritten. Local runtime regressions cover the global binding's
attributes directly. The pin, harness bytes, and opt-in execution limits are
unchanged.

## Reflect function metadata and method-property review

Thirty-nine unchanged sources add seventy-eight Script/StrictScript variants.
Each of the thirteen Reflect methods now has name, length, and Reflect-property
descriptor fixtures using the original property helper. Function attributes are
read-only/non-enumerable/configurable; Reflect method properties are
writable/non-enumerable/configurable. Complete Reflect own reflection enables
the helper's original enumeration and mutation checks. Other reviewed Proxy,
foreign-realm, and dynamic-function candidates remain separate gaps. The pin,
harness bytes, and opt-in execution limits are unchanged.

## Iterator constructor review

Four unchanged sources add eight Script/StrictScript variants. They cover the
constructor's function type, abstract call/construction rejection, Function
prototype inheritance, and the prototype constructor accessor descriptor.
Reviewed candidates needing classes, destructuring, cross-realm host helpers,
or complete global enumeration remain outside this selection. The static
reflection review below adds the constructor name/length/prototype files. Local
regressions cover distinct and bound newTargets, ordered prototype access,
intrinsic fallback, protected setter semantics, and collection. The pin, harness
bytes, and opt-in execution limits are unchanged.

## Iterator.from review

Seven unchanged sources add fourteen Script/StrictScript variants. They cover
the function type/prototype, name/length descriptors, primitive rejection,
Array and String-wrapper iterable inputs, and absent-return iterator results.
Reviewed candidates using classes, generators, or Proxy-based Temporal helpers
remain outside this selection. The static reflection review below adds the
Iterator.from property descriptor file.
Their original source and helpers are not rewritten. Local regressions cover
direct iterators, exact acquisition and receiver order, cached next/live return,
primitive results, internal slots, reentrancy, errors, and collection. The pin,
harness bytes, and opt-in execution limits are unchanged.

## Iterator.concat and complete static reflection review

Thirty-four unchanged sources add sixty-eight Script/StrictScript variants.
Thirty concat files cover ordered validation, lazy iterator opening, fresh
results, done-before-value, calls without arguments, iteration/closing errors,
reentry rejection, primitive wrappers, constructor rejection, and metadata.
Two reviewed concat candidates still require classes and remain outside this
selection. Four other files verify Iterator's name/length/prototype and the
Iterator.from property descriptor; complete static reflection supports their
original property-helper checks. Shared-prototype enumeration and global
enumeration remain guarded. Local regressions add capture mutation, branded
resumes, permanent completion, opt-in host failures, large inputs, and collection.
The pin, upstream/helper bytes, and opt-in execution limits are unchanged.

## Iterator.toArray review

Seven unchanged sources add fourteen Script/StrictScript variants. They cover
function type/prototype, name/length descriptors, primitive receiver rejection
before next lookup, non-callable next rejection, and plain direct iterators.
Reviewed candidates using classes, generators, or destructuring remain outside
this selection. Local regressions cover captured next and exact receivers,
done-before-value, acquisition/step failures without closing, fresh intrinsic
results, species/setter bypass, element identities, collection, large default
inputs, and opt-in host failures. The pin, original source/helper bytes, and
opt-in execution limits are unchanged.

## Iterator.forEach review

Nine unchanged sources add eighteen Script/StrictScript variants. They cover
callback validation before next lookup, closing on validation failure, plain
direct iterators, primitive and non-callable-next rejection, function metadata,
and name/length descriptors through the original property helper. Reviewed
candidates requiring classes, generators, or destructuring remain outside this
selection. Local regressions add callback/step error precedence, exact
mathematical indices, cached next, mutation, collection, large default inputs,
and opt-in host failures. The pin and original source/helper bytes are
unchanged.

## Iterator.every/some/find review

Twenty-seven unchanged sources add fifty-four Script/StrictScript variants, with
nine sources for each predicate consumer. They cover callback validation before
next lookup, closing on validation failure, plain direct iterators, primitive
and non-callable-next rejection, function metadata, and name/length descriptors
through the original property helper. Reviewed candidates requiring classes,
generators, or destructuring remain outside this selection. Local regressions
add short-circuit/throw closing precedence, exact indices, truthiness without
coercion hooks, live next/return mutation, found-value identity, collection,
large default inputs, and opt-in host failures. The pin and original
source/helper bytes are unchanged.

## Iterator.reduce review

Ten unchanged sources add twenty Script/StrictScript variants. They cover
reducer validation before next lookup, closing on validation failure, arbitrary
accumulator types, plain direct iterators, primitive and non-callable-next
rejection, function metadata, and name/length descriptors through the original
property helper. Reviewed candidates requiring classes, generators, or
destructuring remain outside this selection. Local regressions add initial-value
presence, empty/singleton inputs, callback indices/receivers/argument counts,
accumulator identity, mutation, incoming throw precedence, step errors without
closing, collection, large default inputs, and opt-in host failures. The pin and
original source/helper bytes are unchanged.

## Lazy Iterator.map/filter review

Eighteen unchanged sources add thirty-six Script/StrictScript variants, with
nine sources for each helper. They cover callback validation before next lookup,
closing on validation failure, plain direct iterators, delayed non-callable-next
rejection, primitive receivers, function metadata, and name/length descriptors
through the original property helper. Reviewed candidates requiring classes,
generators, or destructuring remain outside this selection. Local regressions
add suspension/return ordering, callback and closing errors, reentry, exact
indices, original filtered-value identity, capture tracing/release, large
default pipelines, and opt-in host failures. The pin and original source/helper
bytes are unchanged.

## Iterator.take/drop published-edition review

Twelve unchanged sources add twenty-four Script/StrictScript variants, with six
sources for each helper. They cover function type/prototype, name/length
descriptors through the original property helper, primitive receiver rejection
before count conversion, and delayed non-callable-next rejection. Reviewed
candidates requiring classes, generators, or destructuring remain outside this
selection.

The pin also includes three files for each method (`argument-effect-order.js`,
`argument-validation-failure-closes-underlying.js`, and `limit-rangeerror.js`)
that assert RangeError for finite counts above Number.MAX_SAFE_INTEGER.
Published ECMAScript 2026, edition 17, has no such check in 27.1.3.3.2/11. These
post-baseline assertions remain outside the corpus, without source rewriting or
pass credit. Local regressions cover count conversion/closing order,
fractional/signed-zero counts, infinity and large exact finite countdowns,
discarded-value bypass, return/reentry, errors and completion, capture
tracing/release, large default inputs, and opt-in host failures. The pin and
original source/helper bytes are unchanged.

## Iterator.flatMap review

Nine unchanged sources add eighteen Script/StrictScript variants. They cover
callback validation before next lookup, closing on validation failure, plain
direct iterators, delayed non-callable-next rejection, primitive receivers,
function metadata, and name/length descriptors through the original property
helper. Reviewed candidates requiring classes, generators, or destructuring
remain outside this selection. Local regressions cover one-level flattening,
iterable/direct fallback, primitive rejection before hooks, exact outer indices,
inner-before-outer closing, incoming error precedence, reentry, capture
tracing/release, large default inputs, and host failures without JavaScript
cleanup. The pin and original source/helper bytes are unchanged.

## Shared Iterator prototype reflection review

Seventeen unchanged sources add thirty-four Script/StrictScript variants. Eleven
sources verify all shared helper method property descriptors through the
original property helper; these were deferred by earlier helper selections while
shared prototype enumeration was incomplete. Five sources verify
Symbol.iterator's function type, name/length descriptors, property attributes,
and generic receiver identity. One source verifies the Symbol.toStringTag
accessor descriptor.

The edition-17 prototype inventory now supports complete ordered reflection and
integrity operations. Local regressions verify all fourteen properties,
getter-free descriptor copying, sealing/freezing, protected setters, ordinary
mutation/order, inherited iterator enumeration, and intrinsic retention.
Reviewed constructor/tag setter originals requiring destructuring remain
excluded. Post-baseline iterator APIs remain outside this published edition. The
pin and source/helper bytes are unchanged; no default quota or testing allowance
is introduced.

## Tagged template review

Nineteen unchanged sources add thirty-four prescribed Script/StrictScript
variants. They cover site identity across top-level loops and function instances,
distinct sites with identical source, differing raw strings and component counts,
ordered unconverted arguments, member/call receivers, chained calls, constructor
precedence, invalid cooked escapes, frozen template/raw Arrays, per-realm template
map reuse, and exact descriptors through the original property helper.

Reviewed originals requiring eval, dynamic Function construction, JavaScript
cross-realm hooks, or proper tail calls remain excluded. Local regressions add
AST/diagnostic snapshots, syntax-clone identity, separate parses/host realms,
collection of cached templates without public links, getter/substitution errors,
late callable checks, public-hook bypass, large default inputs, and atomic cache
publication after successful construction. The pin and original source/helper
bytes remain unchanged; all host quotas remain disabled by default.

## Optional chaining review

Twenty-seven unchanged sources add fifty-four Script/StrictScript variants:
sixteen positives cover nullish checks, optional property/call grammar, method
receivers, ungrouped suffixes, skipped side effects, decimal lookahead,
new.target calls, and iteration statements. Eleven parse negatives reject direct
template tags (including newline forms) and assignment/prefix/postfix targets.
Each review records the intended template or chain target's exact byte range and
diagnostic; unsupported syntax cannot substitute for these early errors.

Reviewed originals requiring classes/super, async functions/promises, RegExp, or
eval remain excluded. Local regressions also cover deletion without final
getters, grouping boundaries, ordered computed keys and arguments, abrupt
completions, strict receivers, spread, grouped construction, syntax retention
through collection, large flat chains, and opt-in host aborts. The pin and original
source/helper bytes are unchanged; no default work or heap limit is introduced.

## URI encoding review

Fifty-eight unchanged encodeURI/encodeURIComponent sources add 116
Script/StrictScript variants. They cover lone high/low surrogates and invalid
pairs, distinct reserved/unescaped sets, real URLs and Unicode text, control
characters, string-hint object conversion and abrupt results, function name/length
attributes and mutation, and non-construction. The unchanged pinned
decimalToHexString.js helper supplies hexadecimal reporting for the exhaustive
surrogate loops; the harness inventory now contains seven originals.

Twelve additional unchanged sources now execute their String case conversion
paths using the native implementation. Four global enumeration/descriptor sources
remain deferred pending complete global own keys through the original property helper. They receive
no pass credit. Local regressions add exact uppercase UTF-8 boundary escapes,
Symbol coercion errors, intrinsic URIError retention/materialization, large default
outputs, host aborts, and direct descriptor inspection. The pin and all original
bytes are unchanged; host quotas remain opt-in.

## URI decoding review

One hundred seven unchanged decodeURI/decodeURIComponent sources add 214
Script/StrictScript variants. They cover valid one-to-four-byte UTF-8, exhaustive
literal UTF-16 and supplementary scalar loops, malformed percent triplets,
continuation bytes, overlong sequences, escaped surrogates and out-of-range values,
reserved escape preservation, ordinary strings and URLs, ordered object conversion
and abrupt results, and function name/length attributes and non-construction.
The existing unchanged decimalToHexString.js helper supplies hexadecimal reporting.

Four global enumeration/descriptor sources remain excluded because they require
complete global own-key reflection through the original property helper. They
receive no pass credit. Native regressions add boundary cases, literal lone
surrogate preservation, lower-case reserved escape spelling, non-recursive percent
handling, intrinsic URIError retention, and opt-in host aborts. All original bytes
and the pin are unchanged. Exhaustive loops use the ordinary unlimited defaults;
no source rewriting or test-only allowances are introduced.

## String case conversion review

One hundred two unchanged sources add 204 Script/StrictScript variants across
toLowerCase, toUpperCase, toLocaleLowerCase, and toLocaleUpperCase. They cover
generic receivers, full mappings and expansions, final sigma, canonical text
remaining unnormalized, supplementary characters, abrupt conversions, function
name/length attributes, and non-construction. Eight sources requiring eval or
RegExp literals remain excluded and receive no credit.

Twelve previously deferred URI encoding sources add 24 variants using the now
implemented case methods on executed paths. Their original Unicode comments and
all test bytes are preserved. Existing unchanged harness files suffice; the pin
and ordinary runtime defaults remain unchanged. Native regressions also cover
Unicode 18 additions, lone surrogates, overlapping casing properties, intrinsic
retention, and opted-in host aborts.

## AggregateError review

Twenty-one unchanged constructor/prototype sources add 42 Script/StrictScript
variants. They cover call/construction, custom prototypes, ordered message/cause
and iterable processing, cached next and abrupt iterator reads, dense own errors
Arrays, property descriptors, and intrinsic prototype/error inheritance.

The complete unchanged promiseHelper.js is the eighth harness original. The order
test uses its synchronous checkSequence function; this does not add Promise support.
Four sources requiring Proxy, dynamic Function or cross-realm construction, or
complete global own-key reflection remain excluded and receive no credit. Native
regressions add cause access order, step-error behavior without IteratorClose,
intrinsic Array construction, collection, and opt-in host failures. The pin,
original bytes, and ordinary runtime defaults remain unchanged.

## String normalization review

Fourteen unchanged sources add 28 Script/StrictScript variants for all four forms,
generic receivers, receiver/form conversion order, invalid form errors, and
function metadata. The original prototype descriptor source now executes through
the complete String property inventory and unchanged propertyHelper.js.
Native regressions cover canonical blocking, composition exclusions, Hangul,
supplementary mappings, lone surrogates, intrinsic RangeError retention, collection,
and opt-in host aborts. The complete original Unicode 18 normalization oracle is
independent of these fixtures. The pin, source/helper bytes, and runtime defaults
remain unchanged.

## String locale comparison review

Thirteen unchanged sources add 26 Script/StrictScript variants for canonical
equivalence, symmetric comparison, omitted comparison values, generic wrappers,
non-coercible receivers, metadata, and non-construction. Native regressions add
positive zero, supplementary equivalences, consistent total ordering, compatibility
distinctions, lone surrogates, conversion/argument order, collection, and host aborts.
The documented fixed locale-neutral NFD ordering is an ECMA-262 host choice without
ECMA-402. The pin, original bytes, helpers, and ordinary defaults remain unchanged.

## String matching-hook review

Fifteen unchanged match/matchAll/search sources add 30 Script/StrictScript variants
for custom object hooks, getter errors, exact invocation arguments and receivers,
non-coercible matchAll receivers, name/length/property descriptors, and
non-construction. The direct creation review below revisits the native creation
boundary. Native regressions also cover global-flag
ordering, non-callable hooks, ignored primitive hooks, preserved results, collection,
and host aborts. Fallback matching remains Unsupported. The pin and all original
bytes are unchanged; no default resource quota or test allowance is introduced.

## String direct RegExp creation review

The scoped review reads all twenty-five String.matchAll programs and thirteen
match/search fallback programs at the existing pin. Six originals are already
present. Four newly vendored whole programs add eight normal/strict positives for
direct creation, native source/flags/lastIndex, live Symbol invocation, original
results and converted String receivers. Both compilers pass every new variant.
The other twenty-eight programs require RegExp literal execution or native
matching and remain excluded without source rewriting or pass credit. Several
primitive-hook programs exercise matching after their hook checks and therefore
remain excluded. This is not a complete match or search directory review.
Historical descriptions retain edition-17-compatible assertions. The pin, ten
helpers, original bytes and unlimited defaults are unchanged; no native matching
coverage is claimed.

## JSON value creation review

Fifty-two unchanged JSON.parse sources add 104 Script/StrictScript variants for
exact JSON whitespace, string delimiters and escapes, control-character rejection,
ordered text coercion and abrupt failures, negative zero, ordinary __proto__ keys,
duplicate names, and function metadata/non-construction. This initial cohort
omits the descriptor requiring whole-object enumeration, callable-reviver,
stringify, and raw JSON cases; later cohorts review those implemented paths. Local regressions additionally cover intrinsic prototype
and exception retention, own data descriptors, inherited setter bypass, binary64
boundaries, deep default values, and opted-in host aborts. The pin, original bytes,
harness files, and unlimited defaults remain unchanged.

## JSON reviver review

Thirteen unchanged sources add 26 Script/StrictScript variants for postorder and
numeric-key ordering, inherited child values, ignored descriptor/deletion
rejections, getter and callback errors, the root wrapper, and primitive/array/object
source contexts. Native regressions cover changed values and replaced containers,
duplicate-key lexemes, snapshot keys and array length, exact callback arguments,
setter bypass, deep traversal, and opted-in cyclic traversal aborts. Originals
requiring Proxy or destructuring parameters remain excluded without credit. The pin, original bytes, harness files, and unlimited defaults are unchanged.

## Raw JSON review

Twelve unchanged rawJSON/isRawJSON sources add 24 Script/StrictScript variants for
primitive validation, surrounding whitespace and empty-input errors, null-prototype
object shape, internal-slot branding, function metadata, and non-construction.
This initial cohort omits stringify and whole-JSON-object descriptor cases, which
are reviewed separately. Originals requiring destructuring remain excluded. Local regressions additionally
cover exact large-integer and surrogate text, frozen descriptors, ordered conversion,
getter-free/inherited/copied brand checks, collection, large default inputs, and
opt-in work aborts. The pin, original bytes, harness files, and unlimited defaults
remain unchanged.

## JSON serialization review

Sixty-three unchanged sources add 126 Script/StrictScript variants: 54 stringify
sources, four formerly deferred parse/raw JSON descriptor or embedding sources,
and five JSON object metadata sources. They cover ordered hooks and conversion,
replacer functions and deduplicated property lists, live reads, omission and array
null substitution, indentation, cycles, BigInt errors and hooks, exact raw JSON
embedding, well-formed surrogate quoting, and complete JSON reflection. Native
regressions additionally cover deep iterative serialization, collection, mutation,
shared references, callback reentry, all UTF-16 code units, and opt-in host aborts.
Originals requiring Proxy, cross-realm support, RegExp literals, destructuring, or
complete global-object reflection remain excluded without credit. The existing
pin, all original bytes, eight harness files, and unlimited defaults are unchanged.

## Map and Map Iterator review

One hundred eighty-seven unchanged sources add 371 variants: 176 Map sources and
11 Map Iterator sources. Two forEach sources specify a single strict/sloppy mode,
and one computed-insertion source specifies strict mode. The cohort covers
constructor and adder ordering, closing and abrupt results, canonical key types,
insertion/update/deletion/clear order, live iterators and forEach mutation,
getOrInsert and computed callback mutation, intrinsic groupBy, branding, complete
reflection, species, and non-construction. All selected source bytes are verified
against their upstream Git blob identities. Twelve Set receiver originals are added
with the Set cohort below. Whole originals requiring WeakMap
receivers, typed arrays, WeakRef, classes, cross-realm support,
or global reflection remain excluded without credit. One original tagged WeakMap
exercises only Map and needs no WeakMap implementation. The published edition-17
baseline includes both insertion methods. The pin, eight original harness files,
and unlimited defaults are unchanged.

## WeakSet review

Eighty-three unchanged originals add 166 harness-positive variants, with every
file running in both Script modes. They cover constructor and iterator acquisition,
cached adders and closing on failures, object/non-registered/well-known Symbol
keys, registered Symbol rejection, identity/duplicate additions, branded method
receivers, add/has/delete return values, metadata/descriptors, and method
construction rejection. The complete 85-file directory review excludes the
cross-realm newTarget program and the global descriptor program whose unchanged
property helper requires complete global enumeration. Neither receives credit.
All original bytes, assertions, metadata, eight helpers, the pin, and unlimited
runtime defaults are unchanged. Native regressions separately exercise weak
reachability, generation reuse, Symbol release, key cycles, pruning, opted-in
aborts, and normal-debug stack safety.

## Set and Set Iterator review

Three hundred fifty-three unchanged sources comprise 342 Set and 11 Set Iterator
files. Another twelve Map receiver-brand files use Set instances. Together they
add 728 variants: two Set forEach files prescribe a single Script mode and every
other source runs in both modes. Coverage includes construction and cached adder
ordering, canonical primitive/Object/Symbol identity, live iteration and callback
mutation, iterator closing, all seven set-like combination/predicate algorithms,
observable size/has/keys order, duplicate keys, snapshot versus live membership,
intrinsic results, branding, complete reflection, species, and non-construction.
All bytes are verified against the original Git blob identities. Forty-one reviewed
Set originals require generators, weak/typed collections, WeakRef,
cross-realm support, or complete global reflection and remain excluded without
credit. This includes unused generator methods: whole files are retained or
excluded, never rewritten. The pin, eight original helpers, and unlimited defaults
are unchanged.

The class/WeakSet revisit adds 29 originals and 58 variants: seven WeakSet
receiver-brand programs and 22 class programs covering set-like acquisition
order, overridden subclass methods, ignored species, and intrinsic results.
The focused review read 48 whole Set/Map programs. Seventeen generator programs
and two mixed valid-value/key programs requiring typed arrays, WeakMap, or WeakRef
remain excluded. No Map original is added by this revisit; no program, assertion,
metadata, helper, or runtime default is changed.

## Function construction and reflection review

One hundred ninety unchanged Function sources and one Map computed-callback
source add 297 variants. Eighty-five originals prescribe a single Script mode;
the other 106 run in both modes. They cover ordered source conversion, independent
parameter/body grammar checks, combined early errors, global scope, calls and
construction, custom newTarget prototypes, parameter and arguments behavior,
constructor/prototype metadata, and exact ordinary function source retention.
The Map original now exercises dynamically constructed callbacks unchanged.

The 298 Function candidates were reviewed as whole files. The 108 excluded
originals require eval, optional non-strict caller extensions, with statements,
classes, generators, async functions, private elements, Proxy, foreign realms,
complete global reflection, or nativeFunctionMatcher.js. That unchanged helper
requires RegExp literals even in originals whose assertions otherwise use exact
source strings. These exclusions receive no execution credit. Included source
bytes match their upstream Git blob identities. The existing pin, eight unchanged
helpers, and unlimited runtime defaults are retained.

## With statement review

One hundred one unchanged with-statement sources and one formerly deferred strict
Function construction source add 104 variants: 87 harness-positive variants and
17 reviewed parse negatives. Eighty-five with originals are positive; sixteen
reject strict with statements, declarations in Statement positions, labelled
functions, or the let-array expression lookahead. Every negative is reviewed at
its actual rejection token or complete strict with statement, with an exact
diagnostic and original byte range. Declarations forbidden in Statement positions
are rejected by that grammar rule before their unavailable body syntax is parsed.

The positive cohort covers variable scope, captured environments, live object
lookup, primitive boxing and nullish TypeErrors, unscopables truthiness and abrupt
getters, mutation between resolution and Get/Set, update identity, and restoration
after normal/abrupt completion. Of 181 reviewed with originals, 71 reference eval,
seven require Proxy, and two require typed arrays; those whole files remain
excluded without credit. Original bytes match their upstream Git blob identities.
The existing pin, eight unchanged helpers, and unlimited defaults are retained.

## Catch binding pattern review

Seventy-seven unchanged sources from `language/statements/try/dstr` and five
reviewed catch-parameter early errors add 162 variants: 142 harness positives and
20 parse negatives. The positives cover ordered property access and computed
keys, nested/default bindings, anonymous function names, nullish rejection,
iterator acquisition and exhaustion, step/value errors and closing, elisions,
array rest targets, and object rest descriptors. Every negative is checked at its
intended token: strict arguments/eval bindings, duplicate/conflicting catch
names, rest initializers, or non-final rest elements.

All 93 originals in that destructuring directory were reviewed; fourteen require
generators and two require classes and remain excluded as whole files without
credit. The remaining top-level try directory has not been fully reviewed.
Original Git blob identities and SHA-256 digests verify the unchanged bytes. The
pin, eight helpers, and ordinary unlimited defaults are retained.

## Lexical declaration pattern review

Seventy-seven unchanged sources each from `language/statements/let/dstr` and
`language/statements/const/dstr` add 308 variants: 284 harness positives and 24
parse negatives. The positives cover nested properties and iterator elements,
defaults and anonymous function names, empty and nullish sources, iterator
acquisition, exhaustion and abrupt steps/values, closing, elisions, nested array
rest targets, and object rest getters and descriptors. Negative diagnostics reject
rest initializers or non-final rest elements at the original `=` or comma token.

All 186 originals in the two directories were reviewed. Twenty-eight require
generators and four require classes; those whole files remain excluded without
credit. Exact comparison confirms each let/const pair differs only in its
template comment, description, and declaration keyword. Both originals are
vendored unchanged and run in both Script modes. The existing pin, eight helpers,
and ordinary unlimited defaults are retained. Lexical loop-pattern originals are
recorded in the synchronous loop review below.

## Var declaration pattern review

Seventy-seven unchanged sources from `language/statements/variable/dstr` add 154
variants: 142 harness positives and twelve parse negatives. They cover nested
properties and iterator elements, defaults and anonymous function names, nullish
sources, iterator acquisition/exhaustion/errors/closing, elisions, nested array
rest targets, and object rest descriptors. Every negative rejects a rest
initializer or following element at the original `=` or comma token.

All 97 originals in that directory were reviewed. Fourteen require generators
and six require classes, including four class-static-block await cases; those
whole files remain excluded without credit. Exact comparison with the reviewed
let sources verifies the shared executable bodies and metadata, with var's
different BindingInitialization environment reviewed against the specification.
Git blob identities and SHA-256 digests verify every vendored original. The pin,
eight helpers, and ordinary unlimited defaults are retained. Var loop-pattern
originals are recorded in the synchronous loop review below.

## Formal binding pattern review

The existing pin adds 154 function-declaration, 154 function-expression, and 199
arrow parameter sources. Their 1,006 variants include 852 positives using the
unchanged harness and 154 reviewed parse negatives. They cover object/array
parameters, nested defaults and patterns, top-level defaults, iterator acquisition
and closing, elisions, anonymous function naming, rest bindings, property reads
and abrupt completion, and invocation of the original body exactly once.

All 603 originals in these three directories were reviewed. Eighty-four require
generators and twelve require classes; they remain excluded as complete files
without credit. Exact source-body comparisons account for the declaration,
expression, and arrow context templates. Original metadata and case algorithms
are compared separately, including the default/non-default pairs. Comparisons
never rewrite the stored fixtures, which retain their original Git blob hashes.
The pin and eight harness files are unchanged.

Six malformed rest cases occur in each default/non-default function context and
reject the intended equals or comma. The 45 additional arrow negatives reject
literal/escaped reserved identifiers, with eight original onlyStrict flags.
These negatives exercise binding parameters through the arrow cover grammar;
their historical destructuring-assignment feature tag does not add general
assignment-pattern execution. Unsupported syntax cannot satisfy a negative.
Method/setter patterns and other formal rest originals remain separate reviews.

## Destructuring assignment fixtures

The selection adds 335 unchanged originals from
`language/expressions/assignment/dstr`: 258 positives and 77 parse negatives,
producing 445 harness-positive and 130 parse-negative variants. They cover RHS
identity, identifier and member targets, repeated writes, computed keys, nested
patterns, defaults and function names, primitive/nullish sources, rest data
properties and exclusions, iterator acquisition/closing, partial writes, const
and TDZ errors, unresolvable references, strictness, and escaped reserved names.

All 368 originals were reviewed. Thirty generator-dependent and three
class-dependent originals remain excluded as whole files without credit. The
ordinary function-name fixture has a historical class annotation but no class
syntax; its bytes and metadata remain unchanged. Exact Git blob hashes verify
the original sources. Negative diagnostics identify the intended assignment
target, forbidden rest equals/comma, strict reference, or reserved shorthand.
Unsupported syntax cannot satisfy a negative. Both stable and MSRV pass all 575
variants with ordinary unlimited defaults. The pin and eight helpers are unchanged.

## Synchronous loop pattern fixtures

The selection adds 32 unchanged for-in parse-negative sources and 482 unchanged
for-of sources. Their 970 variants comprise 826 harness positives and 144 reviewed
parse negatives. The for-of sources include 245 assignment patterns and 79 each
of var, let, and const binding patterns. They cover ordered reference resolution,
defaults, nested/rest patterns, iterator acquisition and closing, primitive
sources, property access, anonymous names, and abrupt completion before the loop
body runs. The negatives reject invalid targets, strict references, reserved
identifiers, forbidden rest equals/comma, and loop binding initializers at their
intended tokens.

All 602 originals were reviewed. Seventy-nine require generators and nine require
classes; they remain excluded as whole files without credit. Six class-dependent
binding originals omit the class feature tag, while one ordinary function-name
original has a historical class tag without class syntax. Selection follows the
complete original programs, and their metadata remains unchanged. Exact template
comparisons account for assignment versus binding contexts and var/let/const
headers; original Git blob hashes verify every source. Both stable and MSRV pass
all 970 variants with ordinary unlimited defaults. The pin and eight helpers are
unchanged. Unsupported syntax cannot satisfy a negative.

## Eval intrinsic and indirect execution fixtures

The initial selection added eight eval intrinsic sources and 49 indirect eval
sources,
producing 112 variants: 110 harness positives and two runtime negatives. They
cover metadata and construction rejection, non-String identity returns, global
lookup and this, own strictness, empty/value completions, fresh lexical environments
and TDZs, global lexical conflicts, var/function instantiation, and last-function
precedence. Three historically Annex-B-labelled strict block/switch controls
exercise core lexical function scope and do not require legacy extensions.

All 71 candidates were reviewed. Fourteen were initially excluded as whole files
without
credit: six require complete global reflection through the unchanged property
helper, two require modules, two classes, one generators, one another realm, and
two direct String eval (one also uses private identifiers). Historical descriptions
that say direct eval in indirect tests remain unchanged; selection follows the
complete program and metadata. Original Git blob hashes verify every source.
Both stable and MSRV pass all 112 variants with the existing pin, eight helpers,
and ordinary unlimited defaults.

The runtime-negative original has a valid outer Script and invalid eval input;
compilation throws SyntaxError during outer execution in both Script modes.
Its original runtime phase/type is retained and recorded as `script-runtime-error`
in the digest inventory. Executed rows use three dashes in `runner.tsv`; the
runner matches the thrown constructor name against the original metadata.
Unsupported syntax, setup failures, host limits, or unrelated outer parse failures
cannot satisfy that expectation. Parse negatives still require exact reviewed
source spans and diagnostics.

## Direct eval execution fixtures

The first direct-eval selection added 131 unchanged originals and the indirect
`global-env-rec-eval.js` original, whose caller requires direct String eval.
Their 180 variants comprise 178 harness positives and two runtime negatives.
They cover caller lookup through functions, catch, with, and nested eval; inherited
strictness; this/new.target; completion values; fresh lexical bindings and TDZs;
variable/function declaration instantiation, preflight conflicts, last-function
precedence, mutable and deletable local bindings, and default-parameter arguments
conflicts in ordinary functions, methods, and arrows. Nine historically Annex-B
labelled strict block/switch controls exercise core lexical function scope.

The direct directory contains 286 candidates. The complete synchronous ordinary,
arrow, and method programs and their metadata were reviewed; actual async/generator
function forms identify 144 generated exclusions even where feature tags are
absent. Another generator declaration, two class programs, two outer Modules,
and five global property-helper programs remain excluded as whole files without
credit. The now-supported super-property original is reviewed below. The unchanged
helper requires complete global intrinsic reflection for those five. No assertion or source was rewritten.
Original Git blob hashes verify every source. Both stable and MSRV pass all 180
new variants with ordinary unlimited defaults; the pin and eight helpers are
unchanged.

The two new runtime-negative originals test inherited strict parse errors and
non-strict variable collisions with global lexical bindings. Both are valid outer
Scripts; only the intended runtime SyntaxError satisfies the original metadata.
The intrinsic/indirect cohort above now also includes the newly eligible nested
caller original: 58 sources and 114 variants. Thirteen candidates from that review
remain excluded; the remaining direct-eval dependency uses private identifiers.
The combined eval inventory now includes the super-property original reviewed
below: 190 sources and 294 variants, with 187 positive and three runtime-negative
originals.

## Super property fixtures

The selection adds 27 unchanged originals: 22 from the super-expression directory,
four from object method definitions, and the direct-eval `super-prop-method.js`
original. Their 50 variants comprise 46 harness positives and four reviewed parse
negatives. They cover dot/computed references, home-prototype lookup, actual call
and accessor receivers, strict/non-strict writes, null bases, abrupt key evaluation
and conversion, arrows/eval/default parameters, and internal prototype lookup
without public `__proto__` access. Four 2024 originals check that GetSuperBase
precedes ToPropertyKey for reads, writes, compound assignment, and increments.
Historical algorithm excerpts are preserved; current edition-17 behavior governs
execution. The poisoned public `__proto__` test exercises core internal operations
without depending on Annex B accessor semantics.

The reviewed synchronous originals retain every statement and metadata field.
Actual whole-program forms exclude 72 class-dependent super originals, one also
requiring another realm, plus four async and four generator method originals.
Four of the excluded method programs have parse-negative metadata; unsupported
syntax cannot satisfy those negatives. The two retained parse negatives reject
SuperCall in ordinary object method bodies/defaults at the reviewed `super` token
with the exact diagnostic. No assertions, sources, helpers, or metadata were
rewritten. Git blob hashes verify every source. Both stable and MSRV pass all 50
variants under ordinary unlimited defaults; the existing pin and eight helpers
are unchanged.

## Base class fixtures

The selection adds 23 unchanged originals from the class definition and name-binding
inventories: 22 harness-positive sources and one reviewed ordinary-method duplicate
parameter negative. Their 46 variants comprise 44 positives and two parse negatives.
They cover base/default construction, prototype and constructor attributes,
strict constructors, instance/static methods and accessors, descriptor merging,
Symbol names, restricted function properties, computed static prototype rejection,
and captured immutable class names in declarations and expressions.

All 73 originals in the definition, name-binding, and strict-mode directories were
reviewed with their complete programs and metadata. Thirty-four remain excluded as
whole files without credit: 22 contain generator methods and twelve contain async
methods. The sixteen heritage/derived originals are retained below. The two
retained computed static accessor rejection originals have historical generator feature
annotations but contain only ordinary accessors; the original metadata is retained.
Nine excluded async negatives and six generator negatives receive no credit.
The retained ordinary method negative checks the duplicate parameter at bytes 667–668 and the exact
parser diagnostic; the strict variant adds only the prescribed directive prefix.

Git blob hashes verify every source at the existing pin. Both stable and MSRV pass
all 46 variants with ordinary unlimited defaults. No source, assertion, metadata,
harness helper, or pin was changed.

## Class heritage fixtures

Sixteen more unchanged originals from the same reviewed directories add 32 variants:
30 harness positives and two parse negatives. They cover superclass/prototype
validation, constructor and instance prototype links, superclass evaluation effects,
explicit/default construction, uninitialized and repeated super calls, immutable
class names and heritage TDZs, numeric super methods, strict inherited constructors,
and restricted arguments properties. The strict-mode negative checks the complete
with statement in a function nested within ClassHeritage, at its reviewed original
byte range and exact diagnostic; neither variant receives unsupported-feature credit.

These bring the class cohort to 39 sources and 78 variants: 74 positives and four
parse negatives. The remaining 34 whole-file exclusions require generator or async
methods. Git blob hashes verify every original at the existing pin. Both stable and
MSRV pass all 32 added variants with ordinary unlimited defaults. Sources, assertions,
metadata, helpers, and the pin remain unchanged.

## Public field execution and ASI fixtures

Sixty-six unchanged public-field originals add 131 variants: 115 harness positives
and sixteen parse negatives. Their complete declarations/expressions cover own
property descriptors, inherited setter bypass, frozen receivers, field ordering,
computed key conversion and abrupt completion, repeated fields, per-instance values,
base constructor timing, static this/eval/arrows, inferred class names, repeated
super calls, and automatic semicolon insertion.

The focused review read 72 complete originals from the two elements directories;
six also contain private fields and are now vendored in the private-field cohort.
This is a focused review, not a whole-directory execution count. Two retained
accessor-named originals carry decorator annotations but contain only ordinary
newline-separated public fields. Two ASI negatives carry generator annotations:
the star continues the initializer as multiplication and cannot begin a method.
Their intended rejection is the following body brace. The other six ASI negatives
check that brace or the same-line method name, with exact original ranges and
messages in both Script modes. Unsupported syntax never satisfies these reviews.

Both stable and MSRV pass all 131 variants under ordinary unlimited defaults.
Git blob hashes verify every unchanged source at the existing pin. Every assertion,
metadata field, helper, and the pin is retained.

## Static initialization block fixtures

Twenty-four unchanged originals add 48 variants: 28 harness positives and twenty
parse negatives. They cover empty blocks, interleaved field/block order, abrupt
completion, isolated lexical/variable scopes, class-name capture, constructor this,
undefined new.target, super properties, and await grammar boundaries in arrow
bodies and ordinary constructor parameters. Negative reviews reject the original
await binding, escaped arguments reference, duplicate label/lexical binding,
lexical/var conflict, return, super call, or undefined control target at its exact
byte range and message in both Script modes.

All 29 static-init originals in the declaration/expression class root directories
were read as complete programs. Four whole files remain excluded without credit:
two combine ordinary arguments checks with generator/async function or method
forms, one requires an async function, and one a generator. The private-scope
original is now vendored in the private-field cohort.
Both toolchains pass all 48 selected variants under ordinary unlimited defaults.
Git blob hashes, source bytes, assertions, metadata, helpers, and the pin are
unchanged.

## Private field execution fixtures

Fifty-six unchanged originals add 112 harness-positive variants. They cover
instance/static fields, nested class and ordinary/arrow function capture, optional
access, private-name identity, inheritance, direct eval in methods/initializers/
blocks, inferred name/length metadata, ordinary-property separation, duplicate
stamping, missing brands and primitive receivers, and assignment/destructuring/
loop evaluation order. The cohort includes six previously excluded public-field
dependencies and the static block's private-scope dependency.

The focused review read 65 complete programs. Two private-method dependencies are
now in the method/accessor cohort; seven remain excluded without credit:
four need async execution, and one requests the historical
host extension that forbids private elements on non-extensible objects. Core
ECMA-262 allows private fields on those objects. Two nested-static originals call
the absent `methodAccess` member in their TypeError assertion, so that assertion
cannot establish the intended private brand check. No upstream source is edited
to make it eligible. This selection does not claim whole-directory coverage.
Both toolchains pass all 112 selected variants under ordinary unlimited defaults;
Git blob hashes, source bytes, assertions, metadata, helpers, and the pin are
unchanged.

## Private method and accessor execution fixtures

128 unchanged originals add 256 harness-positive variants. They cover private
instance/static methods and accessors, shared method identities, fresh class
brands, superclass/subclass receivers, nested/ordinary/arrow capture, cross-kind
shadowing, direct eval in methods and initializers, name/length metadata,
ordinary-property separation, repeated method/accessor stamping, method write
rejection, missing getters/setters, computed names, and abrupt getter/setter calls.
Two earlier private-field dependencies are included in this cohort.

The focused review read 133 complete programs. Four whole files require Annex B
`__lookupGetter__`/`__lookupSetter__` helpers and remain excluded. One static-setter
program calls the absent `getWithEval` member in its TypeError assertion; that
assertion does not verify the intended setter brand check, so the unchanged file
also remains excluded. Earlier static-field programs with a similar unrelated
TypeError remain outside this selection. No upstream file is rewritten to pass.
Both toolchains pass every selected variant with ordinary unlimited defaults.
Git blob hashes, source bytes, assertions, metadata, helpers, and pin are unchanged;
this is a focused inventory, not a whole-directory coverage claim.

## Private element early-error fixtures

Forty-eight unchanged originals add 96 parse-negative variants. They cover
duplicates across private fields/methods/accessors and static/instance elements,
getter/setter staticness conflicts, forbidden constructor names, undeclared names
in computed keys and nested heritage expressions, whitespace after the hash, and
super-private access. Each review checks the intended original token and exact
message in both Script modes. No private element executes in these parse negatives.

The focused review read 52 complete originals. Four heritage programs contain
unparenthesized arrows, so an earlier heritage grammar rejection cannot establish
their intended private-scope error; they remain excluded without credit. This
does not claim whole-directory coverage. Both toolchains pass all 96 selected
variants with ordinary unlimited defaults. Git blob hashes, source bytes,
assertions, metadata, helpers, and the pin remain unchanged.

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

The `spite-test262` command runs 13680 variants from 7101 reviewed sources: the eleven
raw hashbang fixtures, ten BigInt parse-negative files in both Script modes,
eleven arrow parse-negative files in their prescribed Script modes, and fourteen
new.target parse-negative files in both Script modes, plus nine positive function
and capture tests, 40 call/construction iterable-spread tests, 30 call/construction
object-spread tests, 133 Reflect call, construction, prototype, extensibility, descriptor, and property tests, 325 Math numeric/metadata tests, 154 Iterator constructor/acquisition/sequencing/consumption/reflection tests, twelve Boolean tests, 63 Number tests, ten numeric parsing
tests, five global numeric predicate tests, 58 URI encoding and 107 URI decoding tests, 43 Error and AggregateError tests, 140 JSON builtin tests, 200 Map and Map Iterator tests, 191 Function builtin tests, 353 Set and Set Iterator tests, 48 BigInt API tests, 173 Object tests,
430 String and String iterator tests, 713 Array and Array iterator tests
(including fifteen nested object-spread files),
19 tagged-template tests, 27 optional-chaining files (16 positive and 11 parse-negative),
34 Symbol tests, 27 object method/accessor tests, and 75 for-of files (53 positive
and 22 parse-negative), plus seven rest-parameter positives and twelve parameter
parse negatives, and 56 for-in files (36 positive and 20 parse-negative), plus
101 with-statement files (85 positive and 16 parse-negative), and 82 catch-binding
files (71 positive and 11 parse-negative), and 154 lexical declaration pattern
files (142 positive and 12 parse-negative), and 77 var declaration pattern
files (71 positive and six parse-negative), and 507 formal binding pattern
files (426 positive and 81 parse-negative), and 335 destructuring assignment
files (258 positive and 77 parse-negative), and 514 synchronous loop pattern
files (426 positive and 88 parse-negative), and 190 eval intrinsic/direct/indirect
files (187 positive and three runtime-negative), and 22 super-expression positives,
and 528 Date timestamp/UTC/local-calendar/metadata files, and 39 class
definition/name-binding/strict-mode files (37 positive and two
parse-negative), and 66 public-field execution/ASI files (58 positive and eight
parse-negative), and 24 static initialization block files (14 positive and ten
parse-negative), and 48 private element parse-negative files, and 56 private-field
execution files, and 128 private method/accessor execution files, and 83 WeakSet
files, and 337 RegExp literal boundary, flag, core Pattern, class-range, named
capture, and Unicode property files (nineteen positives and 318 parse negatives),
plus 273 RegExp builtin positives for escape, construction, slots, native exec,
named result groups and generic matching operations.
The method/accessor files
cover computed key conversion and exceptions, numeric/string/escaped names,
reserved method names, and closure scope. Eight Object entries/values files use
accessor literals to test live enumeration changes and abrupt reads.
That means four raw positives,
12203 positives using the upstream harness, 1469 reviewed parse-negative variants,
and four runtime-negative variants.
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
the complete pinned corpus integration test once in every platform/toolchain
configuration, with the same inventory and outcome checks. Its `conformance`
profile optimizes exhaustive loops while retaining debug assertions and overflow
checks. All other unit, integration, CLI, and documentation tests use their ordinary
profiles. The complete corpus test is excluded from the initial workspace command
and executed explicitly afterward; no fixture or execution variant is omitted.

Run that CI gate locally with
`cargo test -p spite-test262 --profile conformance --test corpus --locked pinned_corpus_runs_all_reviewed_variants -- --exact`.
Main-branch CI runs are allowed to finish when later commits arrive, so incremental
pushes cannot repeatedly cancel every complete result. Superseded branch runs
still cancel.

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
String SameValue mismatch formatting executes JSON.stringify and produces ordinary
assertion failures. Local controls cover these paths.
