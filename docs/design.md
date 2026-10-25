# Design

## Scope

Implement the latest published ECMA-262 edition in Rust. The initial baseline is
ECMAScript 2026, edition 17, published in June 2026. Track the living specification
separately and adopt new published editions with an explicit baseline update.
Proposals outside that baseline are not part of the language.

Target a non-browser host. Normative optional features require an explicit policy.
Do not enable Annex B's browser compatibility extensions. Annex B is standardized,
but optional for this host. ECMA-402, Node.js, CommonJS, DOM APIs, timers, and network
APIs are outside the target. Standard facilities such as Date, Promise, modules,
RegExp, and shared memory remain in scope even when they need host integration.
The edition-17 `Legacy` label does not itself mean optional. In particular,
leading-zero numeric literals and octal/decimal string escapes are required by
12.9.3–4 in non-strict code and rejected as early errors in strict code. Preserve
their lexical metadata until the entire directive prologue establishes strictness.
The optional legacy Object.prototype accessor/helpers in 20.1.3.8–9 are disabled;
these clauses are explicitly Normative Optional, separately from Annex B.

Production crates use Rust's standard library and workspace crates. Approved direct
external production dependencies are `regex` or `regress` for regular expressions,
`jiff` for host time-zone discovery, and `jiff-tzdb` for portable raw IANA time-zone
data. Prefer `regress` when its ECMAScript matching semantics fit the implementation.
No other new direct dependencies are approved. External build dependencies remain
forbidden. The existing `insta` development dependency remains allowed. Review
features and transitive dependencies when introducing an exception. Do not add a
dependency before its implementation needs it.

Jiff's public calendar range does not cover all ECMAScript Date values. Read raw
TZif data from the host or the pinned `jiff-tzdb` fallback and use the checked
workspace implementation for historical and recurring offsets, including local
time gaps and folds. Keep the data version explicit; do not map historical
instants into a different year to fit a backend calendar range.

These libraries are implementation tools, not alternative language specifications.
RegExp syntax, matching, and UTF-16 behavior must still follow ECMA-262, including
features that the selected library does not implement. Time operations must likewise expose
only the specified ECMAScript behavior. Workspace lints forbid unsafe code in every crate,
including tests and tools. This forbids unsafe Rust in our code, not the standard
library's internal implementation.

Correctness comes first, followed by simplicity and safety. Optimize only after
measurements identify a problem and conformance tests protect the behavior.

## Crates

| Crate | Responsibility | Dependencies |
| --- | --- | --- |
| `spite-bigint` | Bounded arbitrary-precision integer arithmetic | std |
| `spite-heap` | Safe generational storage, explicit roots, and bounded tracing | std |
| `spite-core` | Source locations, UTF-16 strings, shared language primitives | std, bigint |
| `spite-parser` | Lexical grammar, AST, parsing, static semantics and early errors | core |
| `spite-runtime` | Values, abstract operations, environments, objects, execution | core, bigint, heap, parser, jiff, jiff-tzdb |
| `spite` | Small embedding facade and command-line host | core, parser, runtime |
| `spite-test262` | Test262 metadata, execution modes, harness and result accounting | engine, development tooling |

Create crates when they first have real functionality. Keep parser internals in
one crate. Keep built-ins inside the runtime until an actual dependency boundary
justifies another crate. Integer arithmetic lives in `spite-bigint`. A regular
expression implementation may later need its own crate and may use `regex` or
`regress`, subject to the language's matching and UTF-16 requirements.

## Syntax

Use a handwritten lexer and recursive descent parser with binding powers for
expressions. Script source positions are UTF-8 byte offsets into the supplied Rust
source string. Dynamic Function inputs are UTF-16 code units: decode valid pairs
to supplementary code points and preserve lone surrogates as source code points
(11.1). Their spans retain UTF-8 lengths for scalars and use three bytes per lone
surrogate. JavaScript strings retain every UTF-16 code unit.
JsString uses shared immutable Arc slice storage. Cloning a String value does
not copy its code units, so repeated custom exec calls and iterator snapshots do
not duplicate the complete input. Concatenation and substrings build independent
storage; equality, hashing and ordering compare UTF-16 contents, and the public
type remains Send/Sync. Existing optional work charges stay conservative and
unchanged; actual scans and output allocations retain their checks.

The parser controls lexical goals for division, regular expressions, and template
tails. Preserve whether trivia contains a line terminator. Do not approximate ASI
by splitting lines. Carry grammar parameters for strict mode, await, yield, return,
and context-sensitive syntax. Keep parentheses where early errors depend on them.

Left-associated binary expression spines (13.6–13.13, 13.16) use an initial
operand and an ordered vector of operator/right-operand steps. Each step retains
its original complete prefix span; right subtrees and parentheses retain their
grouping. Appending a step updates cached structural depth without rescanning
earlier operands. Parsing, early-error validation, cloning, comparison, dropping
and evaluation therefore avoid native recursion along a flat binary chain.
Evaluation preserves each right operand's evaluation before primitive conversion,
per-step short circuit decisions, intermediate string checks and existing optional
work charges. Comma results still have unbound call receivers and indirect eval.
Right-associated exponentiation and genuinely nested syntax remain subject to
the existing native-stack safety guards; host resource quotas remain opt-in.

The parser owns its source scanner and caches tokens as syntactic lookahead
demands them. Consumed tokens retain spans and strict legacy-token evidence.
Scanner and computed-name lookahead failures are reported at every public parsing
boundary; a synthetic EOF cannot make a lexical failure successful. Ordinary
cursor operations remain infallible to keep recursive grammar frames small.
The scanner accepts each of the five lexical goals explicitly and has no template
brace state. Parser-owned substitution context selects Div/TemplateTail while
caching lookahead. Every cached token retains the scanner position and template
context from before its trivia. Initial input uses HashbangOrRegExp. A
primary-expression solidus restores that checkpoint and replaces its cached
division suffix with a RegExp token. Subsequent scanning resumes after the full
literal with the restored substitution context; braces, backticks, and quotes
inside its body cannot substitute unrelated JavaScript diagnostics.
This scanner implements token boundaries only. Literal validation rejects unknown
or repeated flags and simultaneous u/v modes before Pattern parsing. The separate
Pattern validator traverses groups iteratively, validates assertions, quantifiers,
scoped modifiers and character escapes, and resolves numbered forward references
against the final capture count. Decimal bounds are compared exactly without a
machine-integer size limit. Non-Unicode patterns use individual UTF-16 units;
u/v patterns decode pairs and retain lone surrogates. Annex B grammar extensions
are excluded. Ordinary character classes validate range endpoints and ordering;
Unicode hex escapes pair adjacent lead/trail surrogates before determining range
values. Capture names decode identifier code points, and named references resolve
against all captures. Duplicate names require a separating disjunction under
MightBothParticipate; names in successive terms remain conflicting. Flat
UnicodeSetsMode unions validate scalar ranges, reserved punctuation and string
disjunctions, rejecting inversion when MayContainStrings is true. Nested classes
use iterative frames; intersections combine string containment with AND, while
subtraction retains the left operand's result. Operator expressions exclude
implicit unions and ranges. Unicode property expressions accept only exact,
case-sensitive edition-17 property aliases and pinned Unicode 18 general-category
and script values. String properties require v mode, cannot use P escapes, and
participate in class MayContainStrings analysis. Pattern validation is separate
from the native matching operations described below.
The public validate_regexp_pattern entry point shares this validator with
constructor input. It accepts the original UTF-16 body without literal delimiters,
including raw line terminators and lone surrogates, and returns the exact capture
count. Flag errors precede Pattern errors. Diagnostics retain the caller's source
span rather than treating UTF-16 offsets as source byte positions.
The public parse_regexp_pattern entry point also retains CapturingGroupName
metadata (22.2.1.5, 22.2.2.3). Each decoded name has its one-based source-order
capture index and original UTF-16 GroupSpecifier range; permitted duplicate names
retain their separate slots. Metadata collection is opt-in within the existing
validator, so count-only validation adds no name records or offset conversion.
Unicode-mode code-point boundaries translate to UTF-16 through one forward scan
of disjoint specifiers, preserving surrogate pairs, lone surrogates, raw source
and escape spelling. Names retain their decoded identifier units without
normalization or case folding. Flag, duplicate-name and reference early errors
still precede a successful result, and diagnostics keep the caller's source span.
Collection, translation and deep groups use flat vectors and iterative scans.
ParsePattern additionally retains each decoded named backreference and its complete
original UTF-16 AtomEscape range (22.2.1, GroupSpecifiersThatMatch). Repeated,
forward and self references stay in source order; escaped spellings and astral or
surrogate source offsets remain exact. Capture specifiers and reference escapes
are translated by separate forward scans, keeping preparation linear. The
count-only validation API collects neither optional metadata inventory and
preserves its prior validation and flag-error precedence. Reference metadata does
not by itself add native matching support.

This supplies capture metadata for native matching. Named result construction
is described below, together with ordinary literal named-reference matching.
Capturing-group totals must remain strictly below 2^32 - 1 (22.2.1.1). Reject
the first forbidden total before incrementing the counter; this is the specified
grammar early error, not a host resource quota.
Validated literals produce AST values retaining the original body and flag Strings
and the complete token byte span. Each evaluation calls intrinsic RegExpCreate
(13.2.7), creating a fresh branded object with +0 lastIndex. Public RegExp bindings
and Symbol.match hooks cannot intercept creation. Constructor and literal input
share validation and native storage; unimplemented matching remains a runtime
Unsupported boundary after any preceding effects.
Arrow-head lookahead now probes the formal-parameter and expression grammars
rather than balancing division tokens. Successful heads retain their parameter
ASTs; rejected heads retain contextual recognition results. Memo keys include
native grammar depth, grammar flags and private scope depth. Probes retain
scanner tokens but restore private-name uses and computed-name diagnostics;
selected heads replay their own uses exactly once. Scanner failures remain
authoritative, and RegExp rescans invalidate cached heads. Probes retain forbidden binding names and rest initializer/list continuations
with a deferred diagnostic while consuming the expression/list grammar. The
first deferred error keeps its exact rejection point; a lexically complete RegExp
literal contributes its AST or its Pattern/flag diagnostic. Temporary
probe values cannot enter a public AST: selecting the head reports the deferred
error first, while ordinary expressions parse normally.
Destructuring assignment, nested-target and for-in/of covers similarly probe the
supplemental AssignmentPattern grammar. Computed keys and initializers select
expression goals. Selected patterns retain their exact rejection points and
private-name uses; member/call/template continuations instead parse as ordinary
references. Rest continuations and forbidden methods/accessors are consumed only
inside probes, with a deferred error that prevents returning their temporary
pattern ASTs. Ordinary object/array literals retain their own prototype-setter,
initialized-shorthand and spread rules. RegExp rescans invalidate both cover memo
tables. Valid RegExp literal ASTs survive the same cover grammar as other expressions.
Computed class accessor lookahead parses its name with the expression grammar and
retains that AST and its private-name uses for the ensuing accessor or ASI field.
Nested computed names are parsed once.

RegExp Canonicalize (22.2.2.7.3) is an allocation-free core character operation.
Without IgnoreCase, it preserves the input. With either Unicode flag, it uses only
the pinned Unicode 18 simple/common case-folding records; full expansions and
Turkic mappings do not participate. Ordinary ignore-case applies the existing
Unicode default uppercase conversion, retains mappings whose UTF-16 length is
one, and preserves a non-ASCII character that would map to ASCII. Both modes
preserve surrogate values. The existing case-table generator checks the pinned
CaseFolding.txt input; Rust's Unicode version cannot change this behavior.

Core literal-only Pattern compilation (22.2.2) accepts the ordinary UTF-16 mode,
concatenated literal characters, control escapes, fixed hex/Unicode escapes,
escaped syntax characters and ordinary unquantified capturing/noncapturing groups
of that subset.
Ordinary IdentityEscape accepts source characters outside pinned Unicode
ID_Continue, with '$' allowed separately from IdentifierPartChar. Whitespace,
constructor line terminators and lone surrogates retain their original code units.
Identifier continuations, class escapes and assertions cannot become substitute
literal matches; Unicode-mode identity escapes keep their stricter grammar.
Compilation flattens nested groups iteratively. Noncapturing groups contribute
the body's matcher without capture slots. Ordinary capturing groups additionally
retain relative UTF-16 ranges in opening-parenthesis order, including empty
ranges. Every group in this fixed sequence participates. Validation and compiler
capture counts must agree. Named groups, quantifiers, assertions and scoped
modifiers remain unsupported.
Other productions return an explicit unsupported
compilation result; that result is distinct from a failed search. Keep Pattern
validation separate and require it before compilation. Fold each compiled unit
with the pinned ordinary Canonicalize operation. A flat prefix-failure table
supports linear first-match search; sticky matching compares only the requested
suffix. Empty Patterns match through the input's end inclusively. Ranges use
UTF-16 offsets and preserve both halves of surrogate pairs independently.
Programs share immutable storage across clones, and search allocates nothing.
Unicode-mode matching remains pending.

Top-level ordinary Disjunction compilation (22.2.2.3) accepts alternatives that
each compile completely as a literal, fixed character-set sequence, outer-
anchored sequence, quantified atom or quantified prefix with a fixed
ordinary continuation. The iterative delimiter scan skips escaped units and treats
ordinary bracket classes as opaque, including their bars and parentheses.
Nested capturing/noncapturing groups are tracked without native recursion;
nested choices and unsupported productions reject the whole plan. Each branch
reuses its existing matcher. The earliest start wins, with source order breaking
ties, including empty alternatives; match length does not break a tie.
Compilation never expands combinations; plans share immutable storage and
matching allocates nothing. Literal branches and quantified atoms with literal
continuations retain linear search; fixed sequences retain their per-candidate
atom bounds, and anchors add boundary scans.
Opted-in accounting covers every branch's search passes before execution and
reaches character-set construction. Sticky branches inspect one start with their
original boundary semantics. A compiled flag records whether any quantified
branch can inspect the full remaining input; opted-in sticky work then charges
that suffix instead of bounding input work by the Pattern text length. Ordinary
unlimited defaults impose no branch or work cap.
The plan stores each branch's first source-order capture slot and its own
relative ranges. Storage is linear in branches and captures, without a matrix of
all groups for every branch. A successful match borrows its immutable plan
and resolves the selected branch's ranges; result construction visits the complete
global capture count
and creates own undefined String/indices elements for unselected groups. Empty
participating groups retain empty Strings and zero-length indices pairs.

Fresh intrinsic RegExp result and indices Arrays initialize their indexed own
data properties together (RegExpBuiltinExec, 22.2.7.2). The internal initializer
requires an extensible Array with writable final length and no existing indexed
properties. It validates object edges, logical length, optional property/work
limits and storage capacity before appending unique numeric keys. Standard
writable/enumerable/configurable attributes, named metadata order, undefined
slots and Array length/truncation semantics retain ordinary storage behavior.
Each indices pair remains a distinct intrinsic Array. This removes repeated
ordinary property scans during initialization; work is linear in the number of
captures and copied key/value units. It invokes no JavaScript hooks. lastIndex
writes still precede result allocation, temporary value storage is checked, and
default quotas remain disabled.

Ordinary fixed sequences also compile with leading ^, trailing $, or both
(CompileAssertion, 22.2.2.4). The assertions preserve capture offsets and consume
no units. Without multiline, only the exact input beginning/end satisfies each
boundary; sticky offsets do not change the meaning of ^. Multiline additionally
checks the immediately preceding/following UTF-16 unit against all four
LineTerminators, including each position around CRLF. Rejected candidates
continue the same prefix-failure scan for literal bodies, with overlaps preserved.
Class bodies continue their original candidate scan after a rejected boundary.
Empty anchored bodies scan positions linearly. Literal bodies retain their
existing matcher; class bodies reuse fixed-sequence compilation, shared set plans
and relative captures. DotAll, ignore-case and multiline settings remain
independent. Optional construction work reaches the class compiler, and optional
search passes cover consuming terms plus boundaries while sticky mode checks
one candidate.
Quantified atoms and quantified prefixes with a fixed ordinary continuation
also accept these outer anchors. Standalone repeated atoms reuse the continuation
scan with an empty literal, exposing every admissible repetition endpoint. The
scan advances an additional monotone cursor to the next permitted input/line
start and filters continuation endpoints by the end assertion before selecting the
shortest lazy or longest greedy repetition at the earliest start. It never
commits an unconstrained match and then rejects its final length. Membership,
literal search and boundary advancement each visit input linearly; set/dot
continuations retain their fixed-sequence bound. Search needs no allocation,
expanded counts or native recursion. Optional work covers all consuming and
boundary scans even at a single sticky start, including when the anchored
plan appears inside top-level alternatives. Fixed and quantified captured
branches share their existing global slot layout.
Outer assertion sequences now also accept ordinary word boundaries (`\b`) and
non-boundaries (`\B`), alongside repeated or mixed `^` and `$` assertions
(CompileAssertion, 22.2.2.4; WordCharacters, 22.2.2.9.3). A forward source scan
keeps escape pairs, bracket classes and group contents opaque. Each side stores
one constant-size conjunction; duplicate assertions do not add matcher nodes,
and contradictory boundary/non-boundary assertions always fail. Assertions
consume no units and add no capture slots. In ordinary mode, WordCharacters is
exactly ASCII letters, digits and underscore, including with IgnoreCase; long s,
Kelvin sign, other non-ASCII units and surrogate halves remain non-word units.
Missing neighbors are non-word, so boundaries work at both input ends and at
individual UTF-16 positions within surrogate pairs.

Fixed bodies test both conjunctions at each complete candidate. Quantified
bodies advance the same monotone start cursor to an allowed position and test
continuation endpoints before greedy/lazy selection. Fixed prefixes filter their
whole-match starts with the leading conjunction. Existing capture layouts,
branch order, sticky behavior and generic consumers stay shared. Search needs no
new allocation or native recursion. Optional search passes include word-neighbor
checks, including the complete remaining suffix for sticky repetitions; unlimited
defaults remain unchanged. Complete enclosing groups around supported root
assertion Patterns retain the existing normalization and whole-match captures.

Fixed consuming sequences also store word assertions at their actual UTF-16
consuming offsets, including assertions within ordinary groups. Captures use
consuming widths, so assertion-only and empty groups have exact empty ranges.
A sparse, ordered vector stores one conjunction per offset; duplicate assertions
collapse even across group delimiters, and contradictory conjunctions fail. Checks
read the complete input's neighboring units, preserving boundaries outside the
matched slice. Search remains iterative and allocates nothing. Programs and
character sets retain their existing immutable sharing, with optional work covering
each distinct assertion offset separately from consuming terms.

Fixed prefixes and continuations reuse these plans. An assertion-only prefix must
filter the candidate stream instead of taking the unconstrained empty-prefix
shortcut. A continuation's consuming width determines repetition and capture
endpoints even when assertions add zero-width groups. Greedy/lazy endpoint order,
whole-run/final-iteration captures, outer boundaries, group normalization and
alternative offsets stay shared. This also composes word assertions inside
complete groups around a supported quantified concatenation.

Fixed consuming bodies can also compile embedded input/line assertions (`^`,
`$`) with an explicit multiline flag. The flag-dependent entry point extends the
same sparse offset conjunctions, preserving zero-width captures and duplicate
collapse. Ordinary word-only entry points retain their existing flag-independent
behavior for quantified compositions. Runtime fixed-body dispatch, outer
assertion bodies and alternatives pass multiline explicitly. Existing complete
body plans take precedence, preserving earlier capture layouts and accounting.
Assertions compare the complete input's neighboring UTF-16 units, including both
positions around CRLF; sticky offsets do not redefine input beginning/end.
IgnoreCase and DotAll remain independent. Fixed searches keep their existing
candidate stream, shared immutable plans and optional boundary accounting,
without allocation or native recursion.

Fixed prefixes and continuations around one quantified atom can also carry these
input/line assertions. Their explicit flag-dependent entry points pass multiline
to each fixed component and preserve the word-only entry points. Prefix delimiter
scanning accepts assertions without assigning them consuming width. The existing
prefix stream filters assertion-only starts, and the continuation stream filters
complete endpoints before repetition selection. Empty assertion captures,
whole-run/final-iteration slots, complete enclosing groups, outer boundaries and
alternative offsets retain the shared layout. Runtime dispatch and complete-body
fallbacks pass the same multiline flag throughout. Search remains monotone,
allocation-free and iterative; optional work covers fixed boundary checks over
the complete suffix even for sticky repetitions. Repeated groups containing
choices, compositions around repeated consuming groups, multiple quantifiers,
named groups, backreferences and Unicode
modes remain pending.

Ordinary single character-set atoms compile separately (CharacterSetMatcher,
22.2.2.7.1). One bracket class supports raw/escaped characters, ordered ranges,
outer inversion, empty sets and all six class escapes; one class escape or dot
also forms a complete Pattern. Class backspace differs from the outside word
assertion. Shared CharacterEscape decoding preserves literal/group behavior.
A fixed UTF-16 bitmap holds the positive set after pinned ordinary Canonicalize;
matching canonicalizes the candidate before applying outer inversion. DotAll
controls the four LineTerminators, and ordinary mode preserves surrogate halves.
Compilation charges opted-in parse, bitmap initialization and each construction
loop before filling storage; separate charges avoid work-total overflow and
retain host failures separately from unsupported syntax. Search is linear,
sticky matching examines one unit, and immutable plans share storage across
copies. Quantifiers and Unicode-mode sets remain pending.

Fixed ordinary sequences compile literal characters and character-set atoms with
unquantified capturing/noncapturing groups (22.2.2.3, 22.2.2.7). An iterative
group stack records fixed relative capture endpoints, including empty groups.
The complete Pattern is accepted before immutable bitmaps are constructed; the
single-atom preparation and membership rules are shared. Plans use storage
linear in Pattern size and share it across copies. Search checks candidate starts
in order without allocation or native recursion, with a conservative worst-case
bound of input length times consuming atom count. Sticky matching checks one
candidate. Optional search accounting follows those bounds with separate charges
instead of overflowing a product. Assertions, choices, quantifiers, named/scoped
groups, backreferences and Unicode modes remain outside this sequence compiler.

Within one sequence compilation, identical atom source shares one immutable
character-set plan. A temporary cache borrows UTF-16 source slices, using the
compilation's fixed ignore-case/DotAll settings. It is discarded before the plan
returns; no source borrow or global cache escapes. Captures remain independent
per-occurrence endpoints. Reserving cache capacity up front avoids rehashing
previously charged keys. Optional work covers cache setup/lookups and each
distinct bitmap's first construction, preserving host aborts separately from
unsupported syntax and match failure.

Single ordinary consuming atoms additionally compile with greedy/lazy *, +, ?,
exact, lower-bounded and bounded-range quantifiers (CompileQuantifier, 22.2.2.5–6;
RepeatMatcher, 22.2.2.3.1). Literal characters reuse CharacterEscape and pinned
Canonicalize; sets reuse their existing preparation and membership rules. Bounds
remain compact mathematical counts, without expansion or floating-point
conversion. A minimum beyond representable input lengths can never match; an
oversized maximum cannot constrain a representable input. The complete supported
Pattern is accepted before optional set-construction charges and allocation.
Search visits candidate starts in order and selects the longest greedy or
shortest lazy run. A failed short run is skipped together because later starts
inside it also fail the minimum. Every repetition consumes one UTF-16 unit, so
search is linear without allocation, recursive calls or a backtracking stack.
Sticky matching still checks one start but may scan its complete input suffix;
optional search work covers that suffix rather than the shorter Pattern text.
Transparent noncapturing groups can surround the atom or its quantified run at
any depth. An iterative prefix/closing scan tracks group depth and the single
quantifier without a recursive syntax tree. Moving that one quantifier across
transparent one-atom wrappers retains its greedy/lazy behavior and exact bounds.
Multi-atom groups, multiple quantifiers, concatenated continuations, embedded assertions,
choices, backreferences and Unicode modes remain outside its subset.

Ordinary capturing wrappers use the same nested single-atom grammar. Their
opening source text is an implicit stack: closing a group before the quantifier
places its capture inside each iteration; closing it afterwards captures the
complete run (RepeatMatcher, 22.2.2.3.1). Source-order groups are therefore a prefix
of whole-run captures followed by final-iteration captures, recorded with two
counts rather than a vector or recursive tree. A successful match resolves each
range in constant time. A group such as (a)+ captures only the last UTF-16 unit;
(a+) captures the complete run. Zero repetitions leave inner groups undefined,
while outer groups capture an empty String. Noncapturing wrappers can mix with
capturing wrappers at any depth without changing these distinctions.
Continuation plans resolve prefix captures using the final prefix endpoint,
excluding the fixed literal suffix. Anchored plans preserve the chosen repetition
ranges; alternatives keep their global capture offsets and own undefined slots
for every unselected group. Runtime results borrow the immutable matcher and its
selected branch without constructing a capture matrix or allocating during
search. Multi-atom groups, named groups and multiple quantifiers remain outside
these consuming plans.

A quantified consuming atom can additionally precede a complete fixed
literal continuation (CompileSubpattern, 22.2.2.3). Prefix atom membership and
bounds reuse the single-quantifier plan, while the continuation retains its
prefix-failure matcher, including overlapping occurrences and empty noncapturing
groups. Compilation requires the complete supported continuation before building
the prefix set. It never partially accepts an unsupported suffix or expands
repetitions; plans share immutable storage across clones.
The literal continuation also accepts ordinary nested and empty captures. Its
static ranges follow every prefix capture in source order. Total capture count
is their checked sum; resolving a suffix slot adds its fixed relative range to
the selected prefix endpoint, rather than the complete match's start. Prefix
whole-run/final-iteration semantics remain unchanged. Empty suffix groups still
participate with empty Strings and zero-length indices, even when the prefix
repeats zero times and its inner groups are undefined. Captures preserve original
input units and reuse the same layout through anchors, alternatives and copies.
No capture ranges are allocated or collected during search.
Continuation occurrences arrive in increasing UTF-16 position order. Their
earliest admissible prefix start is the maximum of the requested start, the
preceding atom-run start and the maximum-repetition boundary. These quantities
are monotone, so the first eligible occurrence establishes the earliest whole
match. Lazy matching returns that occurrence; greedy matching keeps the last
occurrence with that same earliest start. Each prefix input unit is checked at
most once, and the literal prefix-failure scan remains linear. Search allocates
nothing and uses no recursion or backtracking stack. Sticky matching fixes the
whole start while still inspecting continuation occurrences within the suffix.
Optional accounting covers both complete-input passes, including sticky runs.
Character-set continuations, multiple quantifiers, enclosing groups
around the complete concatenation, assertions, choices and Unicode modes remain
outside this compiler.

Top-level alternative bodies also accept complete ordinary enclosing groups.
Each branch first tries the existing complete literal, sequence, anchored or
quantified compiler, retaining every prior fixed capture layout. If those
compilers reject it, the shared outer-group analysis removes complete ordinary
wrappers and retries the same consuming compilers. A branch stores one checked
total capture count and one whole-match prefix count beside its nonrecursive
body plan. Prefix captures resolve to that selected branch's complete range;
shifted inner slots keep their existing layout. Global offsets still follow
source order, and every group in an unselected branch remains undefined.
Optional accounting covers the group scans, private copy and retry before their
work runs. Search retains the body's existing pass count and full-suffix metadata;
there is no wrapper tree, capture matrix, recursion or repetition expansion.
Nested alternative bodies, groups followed by unsupported continuations and
quantified multi-atom groups remain pending.

Outer input/multiline anchors can also contain complete ordinary consuming
groups. The anchored compiler retains existing fixed layouts when a body
already compiles. Otherwise it removes only complete ordinary wrappers and
retries the same literal, sequence, quantified and literal-continuation plans.
A checked total count and one whole-body prefix count resolve enclosing capture
slots before shifted inner captures. Assertions constrain repetition endpoints
before capture ranges are resolved, including lazy runs and zero iterations.
The same prefix layout composes with alternative offsets and outer runtime
captures. Groups with their own quantifier or a following continuation stay in
the body; unsupported whole bodies still reject the complete plan. Optional
construction work covers group scans, the private copy and retry. Search keeps
the existing assertion and consuming bounds without a wrapper tree, recursion,
capture matrices or allocation. Choices inside an anchored body, embedded
assertions, named groups and backreferences remain pending.

A capture-free literal prefix can precede a single quantified atom with a fixed
literal continuation (CompileSubpattern, 22.2.2.3). Compilation splits only
complete literal characters/escapes before the first consuming group, class, dot
or quantifier; unsupported complete bodies remain rejected. Prefix unit length
comes from the compiled literal, keeping escaped characters and surrogate units
aligned with input offsets. The existing literal prefix-failure search now
exposes an allocation-free occurrence cursor that retains its failure state
between overlapping matches. The quantified continuation asks that cursor for
the next prefix endpoint at or after its monotone run/bounds frontier. Increasing
prefix endpoints also mean increasing whole-match starts, preserving earliest
starts before greedy/lazy endpoint selection. Each cursor visits input units
only linearly; no candidate list, repeated suffix scans, capture matrix or native
recursion is needed. Captures resolve through the body range after excluding
the fixed prefix; enclosing whole-match captures still include the prefix.
Sticky matching first checks the prefix at the exact requested start. Outer
anchors filter whole-prefix starts and repetition endpoints before selection.
Alternatives retain their existing source-order choice and global capture slots.
Optional search work conservatively covers three consuming passes, or four
with outer anchors, including the complete suffix for sticky runs. Default
quotas remain disabled. Multiple
consuming quantifiers and nested choices remain
pending.

Ordinary capturing and noncapturing groups may also form the fixed literal
prefix, including nested and empty groups (CompileSubpattern, 22.2.2.3). A scalar
nesting scan splits only complete top-level Atoms; a quantifier inside a group
keeps that complete group in the consuming body. Literal compilation supplies
fixed prefix capture endpoints in source order. The plan stores a checked total
capture count; prefix ranges translate from the whole match start, while body
and suffix slots use the remaining range after the decoded prefix. Empty
prefixes delegate directly to the quantified body with the same anchor and
sticky constraints, retaining participating empty captures. No per-candidate
capture storage, wrapper tree or native recursion is added. Existing outer and
branch capture prefixes compose with these slots. Grouped
multi-atom repetition, multiple quantifiers, embedded choices, named groups,
backreferences and Unicode modes remain pending.

Fixed ordinary character-set sequences can precede the quantified body too
(CompileSubpattern, 22.2.2.3). Preparation shares the ordinary class/dot parser
and tracks complete top-level Atom boundaries across groups and escapes. The
prefix retains either the existing literal KMP plan or the existing fixed
sequence plan, including shared immutable sets and static capture ranges. The
sequence matcher exposes an allocation-free occurrence cursor, checking each
overlapping candidate once; the body consumes that monotone stream through its
existing endpoint and repetition constraints. Literal prefixes retain linear
search. Set/dot prefixes require at most input length times prefix atom count,
plus the body and continuation bounds; optional work metadata includes every prefix atom.
Sticky search checks one prefix and accounts conservatively for three input
passes with literal continuations, plus one with anchors. Capture order, UTF-16 widths, pinned Canonicalize,
inversion and dotAll reuse existing plans. No candidate lists, recursive plans
or default quotas are introduced. Repeated multi-atom
groups, multiple consuming quantifiers, embedded choices, backreferences and
Unicode modes remain pending.

Fixed ordinary classes, class escapes and dots can also follow the repeated
atom (CompileSubpattern, 22.2.2.3). The continuation retains either its literal
KMP plan or the shared fixed-sequence plan, including immutable sets and static
capture ranges. Occurrences still arrive in monotone start order, so the existing
run, bounds, start and end predicates select earliest starts before greedy/lazy
endpoints. Decoded continuation width identifies the repetition endpoint for
whole-run and final-iteration captures; fixed suffix slots translate from there.
Prefix, repeated-body and suffix captures compose without per-candidate storage.
Literal continuation search remains linear. Set/dot continuation work is bounded
by input length times continuation atom count, plus one membership pass. This
bound applies to sticky repetition too, because it can inspect every endpoint.
Fixed prefixes and outer anchors add their existing bounds; optional metadata
reaches the full runtime and alternative plans. Complete unsupported continuations
reject before constructing the repeated set. No recursion, expanded counts,
candidate lists or default quotas are added. Repeated multi-atom groups, multiple
quantifiers, embedded choices, embedded assertions, backreferences and Unicode modes
remain pending.

Standalone literal groups of at least two decoded UTF-16 units can carry one
greedy/lazy quantifier (CompileSubpattern and RepeatMatcher, 22.2.2.3–3.1).
The immutable plan shares literal decoding, pinned ordinary Canonicalize, static
inner capture endpoints and exact decimal quantifier parsing. It retains a KMP
failure table for two word widths and the word's primitive period. A minimum
search treats repetition as a virtual word: short prefix failures use the exact
table, while prefixes longer than two widths fall back by the primitive period.
Neither large bounds nor searches expand repetitions. The first minimum match
has the earliest start. Greedy extension uses the existing overlapping literal
cursor, skipping occurrences before the next complete-iteration boundary; a gap
ends the run. A zero minimum considers the requested start before later matches.
Lazy matching returns the minimum complete iterations. Search is linear and
allocation-free, with two conservative input passes for optional work, including
the full suffix during sticky matching. Captures inside the atom come from its
last iteration; zero iterations leave every such slot undefined. Empty inner
groups preserve their last empty range. Complete outer captures and top-level
alternative offsets use their existing layouts. Compilation, matching, clones
and destruction remain iterative, with no default quotas. Assertions or fixed
terms surrounding these groups, repeated choices, additional
quantifiers, named groups, backreferences and Unicode modes remain pending.

Standalone fixed consuming groups can also repeat classes, class escapes and
dots (CompileSubpattern and RepeatMatcher, 22.2.2.3–3.1). Their existing sequence
plan supplies pinned case behavior, DotAll, decoded widths, shared immutable
sets and static nested/empty capture ranges. Preparation rejects assertions or
other unsupported terms before constructing sets. Nonsticky searches with a
minimum above one retain one run counter per offset modulo the group width,
visiting each overlapping fixed-group candidate once. Equal minimum widths make
the first qualifying endpoint the earliest start. This temporary storage depends
on the group's width, with checked platform capacity; repetition bounds remain
scalar. One-iteration minimums use the existing sequence search directly. Sticky
minimums and greedy extension check consecutive groups without phase storage.
Lazy matches return the minimum complete iterations, while a zero minimum keeps
the requested empty candidate. Captures translate from the final iteration;
zero iterations leave inner slots undefined. Enclosing groups and alternatives
retain their existing layouts. Nonsticky work is bounded by input length times
group width plus one pass, including phase setup; sticky work covers two complete
suffix passes. Default quotas remain disabled, and no recursive plans or expanded
counts are introduced. Surrounding fixed terms, repeated choices, additional quantifiers,
named groups, backreferences and Unicode modes remain pending.

RegExpBuiltinExec shares an immutable String with the previous participating
capture when their UTF-16 ranges are equal (22.2.7.2). This includes deeply nested
enclosing captures that all span one long match. Range comparison uses constant
space and does not compare or copy substring contents. Every logical substring
still checks the optional String limit and consumes the same optional output work;
lastIndex ordering, own result properties and distinct mutable indices Arrays
retain their specified behavior. Different ranges use the existing checked copy.

An explicit flag-dependent repeated-group entry point accepts internal
word/input/line assertions and fixed consuming groups of at least one UTF-16
unit (CompileAssertion, 22.2.2.4; RepeatMatcher, 22.2.2.3.1). Preparation and each
iteration share the fixed sequence plan's sparse boundary conjunctions, explicit
multiline flag, pinned character behavior and actual consuming offsets. The
minimum occurrence stream therefore admits only complete asserted iterations;
greedy extension checks the next complete group against the same input neighbors.
Captures still translate from the final successful iteration. Zero iterations
leave every inner slot undefined; empty assertion/ordinary captures participate
with exact empty ranges when an iteration succeeds. One-unit groups with empty
inner captures use the same path. Existing body plans retain precedence, and the
legacy assertion-free compiler preserves its two-unit subset. Complete outer
captures and alternatives retain their shared layouts. Optional search metadata
covers each assertion position in minimum search and extension over the complete
suffix, including sticky runs. Duplicate assertions collapse without recursive
plans, expanded counts or default quotas. Fixed terms around repeated groups, repeated choices,
additional quantifiers, named groups, backreferences and Unicode modes remain
pending.

Outer word/input/line assertions now constrain a repeated fixed consuming group
inside the shared anchored plan (22.2.2.4; RepeatMatcher, 22.2.2.3.1). Sticky
search checks the requested start and each eligible complete iteration endpoint.
Nonsticky search tracks overlapping occurrence runs by their group-width phase.
Each run retains the earliest allowed start in the moving minimum/maximum window;
failed or expired starts advance monotonically. Endpoint checks choose the
earliest source position first, then greedy or lazy length for that position.
A separate earliest empty candidate handles zero iterations outside consuming
runs. This avoids restarting a repetition scan at every rejected end assertion.
The shared fixed-sequence candidate bound remains in effect, with constant work
per outer assertion check and temporary storage proportional to group width.
Enclosing captures span the whole match, inner captures retain the final
iteration, and zero iterations leave inner slots undefined. Explicit multiline,
DotAll and ordinary pinned case behavior flow through the group plan. Original
source text and earlier successful plan layouts remain intact. Optional work
metadata covers endpoint/window checks and complete sticky suffixes; there are
no default quotas, recursive plans, expanded counts or new dependencies.

The fixed repeated-group plan also accepts empty and assertion-only bodies
(RepeatMatcher, 22.2.2.3.1). Its explicit flag-dependent compiler admits zero
consuming width while legacy entry points retain their existing subset. A zero
minimum takes zero iterations and leaves every inner capture undefined, even
when the body assertions fail. Once the minimum is satisfied, another empty
iteration fails the unchanged-EndIndex check; greedy and lazy order therefore
produce the same result for this deterministic fixed subset. A positive minimum
tests the shared fixed assertions at one unchanged input position and supplies
participating empty captures. Repeating that pure body cannot change assertion
truth or capture ranges, so even oversized exact positive decimal minima collapse
without expanding a counter or recursing. Outer assertions still filter candidate
positions, enclosing captures span the empty match, and alternatives retain
source order and their global undefined slots. Zero-width dispatch occurs before
division, phase storage or consuming iteration loops. Native exec preserves an
empty match's lastIndex; existing String consumers handle their specified forward
progress. Optional host errors remain separate and default quotas stay disabled.
Surrounding fixed terms, repeated choices and variable bodies, additional
quantifiers, named groups, backreferences and Unicode modes remain pending.

A repeated fixed group can now be followed by a fixed sequel in one ordinary
plan (RepeatMatcher, 22.2.2.3.1). Compilation reuses the exact quantifier scanner
and accepts both complete fixed components before constructing immutable sets.
The existing repeated-group endpoint selector checks the sequel at each eligible
endpoint before choosing the earliest start and greedy or lazy length. Fixed
sequel assertions use complete input neighbors and explicit multiline; outer
assertions check the final sequel end. Captures before the sequel resolve from
the last required iteration or remain undefined for zero iterations. Sequel
captures translate from its actual start, and complete enclosing/alternative
layouts retain their source order. Empty/assertion-only repeated groups and
empty fixed sequels use the same composition. Earlier successful plans retain
precedence and original source/flags remain unchanged. Candidate work is bounded
by input length times the sum of both fixed widths and assertion positions;
optional work metadata also covers endpoint/window checks and full sticky
suffixes. There is no recursion, expanded repetition count, default quota or new
dependency. Captures spanning partial
variable bodies, repeated choices, additional quantifiers, named groups,
backreferences and Unicode modes remain pending.

Fixed prefixes now compose with one repeated fixed group and an optional fixed
sequel (CompileSubpattern and RepeatMatcher, 22.2.2.3–3.1). The prefix has a fixed
consuming width and may contain captures, classes, dots and word/input/line
assertions, including assertion-only and empty groups. Preparation identifies a
complete quantified group outside the prefix's groups and classes, then accepts
both components before constructing prefix sets. Matching translates each body
start to its prefix start and checks the fixed prefix against complete input
neighbors. The repeated-group moving window caches this immutable start predicate,
so rejecting a prefix never restarts the complete repetition search. Earliest
body starts correspond to earliest prefix starts; sequel and outer endpoint
predicates retain greedy/lazy order. Prefix captures translate from the actual
match start, repeated captures from the last iteration, and sequel captures from
the selected endpoint. Zero iterations retain undefined inner slots. Complete
enclosing groups and alternatives keep their existing layouts and original
source/flags. Conservative optional work covers prefix checks, assertion windows
and complete sticky suffixes; bounds stay scalar, with no recursive plans or
default quotas. Repeated choices, additional quantifiers, named groups,
backreferences and Unicode modes remain
pending.

Ordinary groups can now enclose only part of a fixed repeated expression, as in
`((ab)+)c` or `(x)((ab)+(c))` (CompileSubpattern, 22.2.2.3). An iterative group
scan identifies the single quantified group. A dry fixed-sequence layout replaces
that complete repetition with a one-unit capture marker, distinguishing endpoints
before and after even a zero-width repetition. Captures outside the repeated atom
store checked offsets from the successful match's start or end; captures inside
it delegate to the existing final-iteration/undefined layout. Every enclosing
capture participates even when its repeated contents take zero iterations.
The matching body removes outside group delimiters with empty noncapturing
barriers, preserving lexical boundaries so `\0()1` cannot become an octal escape.
It then reuses the fixed-prefix/group/sequel matcher without changing candidate
order, assertion neighbors or greedy/lazy endpoints. Complete outer wrappers
retain their earlier scalar layouts and optional work. Shared immutable capture
metadata keeps clones and destruction iterative; result Strings retain equal-range
sharing while indices pairs remain distinct. Original source, flags, global/sticky
state and generic consumers use their existing paths. Preparation rejects other
variable syntax before constructing sets. No recursion, expanded bounds, default
quotas or dependencies are introduced. Repeated choices, additional quantifiers,
named groups, backreferences and Unicode modes remain pending.

Partial enclosing captures also accept a single quantified literal character,
class, class escape or dot (CompileAtom and RepeatMatcher, 22.2.2.3.1, 22.2.2.7).
The iterative scan consumes complete decoded character escapes and class bodies
before reading quantifiers; ordinary supplementary characters retain their two
separate UTF-16 atoms. An ungrouped quantified atom receives a synthetic
noncapturing wrapper in the private matching body, adding no capture slot. The
existing one-unit repeated-group matcher and dry marker layout then supply exact
greedy/lazy/bounded ranges and participating empty enclosing captures at zero
iterations. Grouped bodies keep their earlier source and layouts. Classes, DotAll,
ordinary pinned case behavior, assertions, complete wrappers and source-ordered
alternatives use the shared paths. Original Pattern/flag text remains untouched,
and preparation rejects multiple quantifiers before constructing sets. Counts
remain scalar with no recursion, expanded alternatives or default quotas.

Repeated groups can contain two or more capture-free literal alternatives that
each consume one UTF-16 unit (CompileSubpattern and RepeatMatcher, 22.2.2.3–3.1).
Every successful branch has the same endpoint and no branch-specific capture,
so source order cannot change the repeated group's capture or sequel selection.
Preparation converts their decoded units to one escaped ordinary character class
in the private matching body, retaining whether the group captures. The existing
fixed repetition and partial-capture plans then preserve final-iteration ranges,
undefined slots at zero iterations, exact bounds, greediness, prefix/sequel terms
and complete input assertions. The top-level alternative scanner tracks ordinary
groups iteratively and validates these inner choices before constructing or
charging branches; wider or capture-dependent choices still reject early. Original
source, flags and pinned ordinary case behavior remain intact. Normalization is
linear in source size, with no expanded repetition counts, recursive plans or
default quotas. Empty/wider branches, captures within individual alternatives,
additional quantifiers, named groups, backreferences and Unicode modes remain
pending.

Complete ordinary capturing/noncapturing wrappers can also enclose a literal-unit
choice inside its repeated atom (CompileSubpattern, 22.2.2.3). Preparation removes
only complete wrappers, records their source-order capture count, and rebuilds the
private character-set body with the same captured layers. Each such capture spans
the final successful unit; zero iterations leave all of them undefined. Captures
outside the repetition still retain their whole-run or partial ranges. The
top-level scanner validates each direct choice once at its ordinary group edge,
keeping deeply nested preparation linear. Earlier accepted layouts remain unchanged.
Copies share immutable metadata and result Strings; indices pairs remain distinct.
Branch-specific captures, empty/wider alternatives, additional quantifiers,
named groups, backreferences and Unicode modes remain pending.

Literal-unit choices also compose inside fixed sequences and fixed repeated
bodies (CompileSubpattern, 22.2.2.3). After the ordinary fixed preparation fails,
an iterative scanner rewrites disjoint direct choices to escaped character
classes without changing group delimiters or capture order. Every branch
consumes one unit and has no internal capture, so all fixed capture endpoints
remain static. Enclosing wrappers and adjacent empty groups retain their lexical
boundaries, including `\0()1`. Set-cache keys borrow the private normalized
source; original source and flags remain exposed unchanged. Existing fixed
prefix, continuation, assertion and repetition plans use the same preparation.
Partial enclosing capture preparation normalizes before flattening surrounding
groups, retaining start/end-relative and final-iteration ranges. Preparation
remains linear without expanded alternatives, recursive plans or default quotas;
optional construction work accounts for the private source and shares identical
immutable predicates. Wider/empty alternatives, branch-specific captures,
additional quantifiers and Unicode modes remain pending.

Capture-free alternatives may also contain one ordinary non-inverted character
class, class escape or dot (CompileSubpattern and CharacterSetMatcher,
22.2.2.3, 22.2.2.7.1, 22.2.2.8–9). Each successful branch still consumes one unit
with identical capture endpoints. Preparation serializes parsed class members
as complete escaped characters/ranges and retains class escapes, so concatenating
bodies cannot turn `[a-]|[-b]` into a range. Dot emits the complete UTF-16 range
with `s`, or ranges excluding exactly the four line terminators otherwise.
Empty classes contribute no members; zero iterations leave repeated captures
undefined. The normalized private body uses the same fixed, assertion and
repetition plans and their immutable predicate cache. Ordinary Canonicalize and
class-escape complements remain unchanged. Source-size preparation is linear and
optional work accounts for each source/bitmap construction; no default quota or
expanded choice combinations are introduced. Branch-specific captures,
empty/wider alternatives, additional quantifiers and Unicode modes remain
unsupported.

One-unit inverted-class alternatives use a flat union of compiled predicates
(CompileSubpattern, CharacterSetMatcher and Canonicalize, 22.2.2.3,
22.2.2.7.1–7.3). Each branch first canonicalizes its member set with the same
ordinary `i` flag, then applies its own inversion; bitmap union combines those
membership predicates. Complementing raw characters before canonicalization
would wrongly accept `A` for `([^a]|b)` with `i`, so these choices are never
rewritten as a positive raw class. Positive members share one predicate build;
inverted branches reuse one temporary bitmap and merge into the immutable union.
Matching retains one membership lookup per consuming term.

An iterative group inventory records direct choices before fixed preparation.
Capture-free branches consume exactly one unit, so ordinary group captures
retain their fixed positions and complete wrappers remain iterative. Partial
repetition preparation preserves outside choice bodies as noncapturing unions;
the original captures already use the separate fixed endpoint layout. Repeated
choices still preserve final-iteration/undefined slots, bounds, assertions and
alternative order. Identical complete choice sources share compiled predicates.
The plan contains only flat atom vectors, with no recursive union tree, expanded
branch combinations or default quotas. Optional construction work charges the
additional parsing passes, atom bounds, temporary clearing and bitmap merges
before construction. Branch-specific captures, empty/wider alternatives,
additional quantifiers, named groups, backreferences and Unicode modes remain
pending.

Capture-free one-unit alternatives can nest within ordinary noncapturing groups
(CompileSubpattern, 22.2.2.3). An explicit stack verifies that every branch has
exactly one consuming literal, class, class escape or dot, while collecting its
leaves into one flat predicate. Groups containing direct choices own their
nested choice ranges, so preparation visits disjoint source spans in order;
it does not repeatedly normalize nested groups or expand branch combinations.
All leaves have the same endpoint and no conditional capture slots. Root group
captures remain source ordered, and zero repetitions leave their inner slots
undefined. Positive leaves use the existing escaped class representation;
inverted leaves merge their individual canonicalized-and-inverted memberships.
Fixed, repeated and partial capture plans retain original flags and source text.
Deep nesting uses linear scans and explicit frames, with no native recursion or
default quotas. Branch captures, empty/wider bodies, assertions inside branches,
additional quantifiers and Unicode modes remain pending.

Transparent empty noncapturing groups may precede or follow the consuming unit
inside these alternative branches (CompileSubpattern, 22.2.2.3). Each explicit
group frame records whether its current branch consumes a unit and whether the
group has alternatives. Closing an empty group leaves the containing branch's
width unchanged; a consuming group contributes its one unit and rejects a
second consuming sibling. Every alternative still requires a consuming unit,
so empty alternatives and differing widths remain unsupported. These groups
add no capture slots, assertions or other matching effects. The same flat
predicate paths preserve captures and flags in fixed, repeated, partial and
top-level alternative plans, with linear preparation and no default quotas.

Complete ordinary outer noncapturing groups can enclose any currently supported
Pattern body (CompileSubpattern, 22.2.2.3). After validating the original Pattern
and capture count, two constant-space UTF-16 scans identify all leading groups
whose closures occupy the end of the complete Pattern. Escapes and bracket
classes remain opaque. A closure belonging to an earlier sibling group never
substitutes for the original leading group. Unquantified whole wrappers are
removed only from the private compilation input; original source/flag slots and
source serialization remain unchanged. The body reuses existing literal, set,
sequence, quantified, continuation, anchored and alternative plans, including
all inner capture slots. No matcher search or repetition ordering changes.
Optional construction accounting covers both scans and the body copy; defaults
remain unlimited. A group followed by a quantifier or continuation stays inside
the body, where the existing compiler accepts or rejects its complete grammar.
Ordinary capturing outer groups use a corresponding three-scan body-range
analysis that can mix capturing and noncapturing openers at arbitrary depth.
Every removed capture encloses the complete chosen body match, and its opening
precedes every retained capture in source order. The runtime plan stores one
whole-match prefix count and a checked total capture count beside the existing
nonrecursive body enum. Capture lookup returns the complete range for a prefix
slot, then delegates shifted slots to the existing body layout. Zero-iteration
whole captures participate with empty Strings; repeated inner captures retain
their final-iteration or undefined state. Unselected alternative captures remain
undefined. No wrapper tree, search allocation or capture matrix is needed.
Original Pattern validation and the normative capture-count bound still precede
compilation. Optional work covers all three scans and the private body copy;
original source/flags, constructor identity/copies and observable coercions keep
their existing order. Groups with their own quantifier or following continuation
stay in the body and require an existing complete supported plan. Named groups,
backreferences, embedded choices and multiple consuming quantifiers remain
pending.

EscapeRegExpPattern (22.2.6.11.1) separately streams an existing validated
Pattern for literal source representation. Empty input becomes (?:). Unescaped
solidus and raw line terminators gain escapes; existing reverse solidus parity
is preserved, including non-Unicode identity escapes of raw line terminators.
All other UTF-16 units retain their exact values. Its cloneable iterator allows
the runtime to check output size before allocation without rewriting Pattern
syntax as RegExp.escape would.

RegExp.escape's String encoding (22.2.5.1) is a cloneable, allocation-free UTF-16
iterator in core. It hex-escapes a leading ASCII letter/digit, distinguishes
syntax characters from other punctuators, uses ControlEscape values, and escapes
WhiteSpace, LineTerminator and unpaired surrogates without changing paired
surrogates. The iterator permits checking
the exact output size before allocating or enforcing an opted-in output quota.
Runtime RegExp.escape rejects every non-String, including wrappers, without
coercion, ignores this and extra argument values, and charges its two linear
passes before checked allocation. The rooted RegExp constructor/prototype graph
exposes edition-17 data/accessor metadata and the generic species getter. The
prototype is ordinary and owns no RegExp slots. Generic flags reads the eight
properties in dgimsuvy order using ToBoolean. Generic toString gets/converts source
before flags and concatenates their exact UTF-16 values; opted-in output quotas
apply after both conversions. Flag getters return undefined on the intrinsic
prototype, whose source is "(?:)"; other unbranded receivers throw TypeError.
Generic test converts its argument before exec lookup, delegates a single String
argument to a callable exec, and accepts only an Object or null result. Shared
RegExpExec performs a fresh lookup for each invocation. Generic Symbol.match
(22.2.6.8) converts flags after the input and uses intrinsic own data elements for
global results. Empty matches apply ToLength and advance by UTF-16 code unit or,
with u/v flags, code point. Its exact counter has no default count cap or
BigInt-value magnitude quota. Generic
Symbol.search (22.2.6.12) preserves SameValue lastIndex identity, including signed
zero, and restores it after normal exec completion before reading the result's
index without coercion. Abrupt exec completion does not restore lastIndex.
Generic Symbol.replace (22.2.6.11) converts its input and non-callable replacement
before flags, and collects live exec result objects before capture reads or
replacement callbacks. Global empty matches share ToLength and UTF-16 Unicode
advancement. Result processing reads length, match, index, captures and groups in
order, with clamped integer positions and undefined captures kept distinct.
Callbacks receive undefined this, the matched String, converted captures,
position, input and an optional uncoerced groups value. Text replacements box
groups even when no named reference appears, then resolve template references in
order without rescanning expansions. Overlapping/backward matches still perform
all reads, calls and conversions before their output is ignored. Checked storage
and output follow the existing optional quotas and native-stack guards.
Generic Symbol.split (22.2.6.14) performs SpeciesConstructor before flags and
Construct before ToUint32(limit). Missing/null species select the intrinsic
RegExp constructor; primitive constructors and non-constructible species throw
before flags lookup. Custom constructors receive the original object and exact
flags with y appended only when absent. The intrinsic result Array uses own data
elements, with empty input executing once without lastIndex writes. Nonempty
input strictly sets the splitter's lastIndex, uses live RegExpExec, clamps its
ToLength end position and advances failed/empty matches by code point for u/v.
Capture values remain uncoerced; result[0], index and groups are never read.
Limits stop before later length/capture reads, and backward custom results follow
the specified positions without a default work cap. Copies use checked storage
and optional output quotas.
Generic Symbol.matchAll (22.2.6.9) converts its input, obtains the species
constructor and exact flags, constructs the matcher, then copies the original
lastIndex through ToLength and a strict Set. It performs no exec lookup until
iteration. Global/Unicode flags come from that captured flags String, not public
matcher properties. The branded RegExp String Iterator (22.2.9) inherits the
shared Iterator prototype and owns only internal slots; its prototype has next
and the standard RegExp String Iterator tag. Each next snapshots matcher/input
and flags, performs live RegExpExec and returns fresh IteratorResults containing
the exact exec object. Non-global success and null mark Done; global empty
matches read/coerce/advance/set lastIndex. Abrupt exec/result/index operations
leave Done unchanged, and reentrant calls cannot reset nested completion.
Matcher slots remain traced, including after Done, until the iterator is
collected. Shared String snapshots do not copy the input. Two additional rooted
intrinsic records bring the initialized-realm count to 387.
Native RegExp construction (22.2.4.1) performs IsRegExp before identity checks.
Only calls with undefined flags and the active intrinsic constructor can return
the input object. Otherwise, native input copies its original private source and
flags even after a false Symbol.match override; regexp-like objects read public
source and flags before allocation and either String conversion. RegExpAlloc
reads newTarget.prototype and creates the own non-configurable, writable
lastIndex before RegExpInitialize converts and validates source/flags. Successful
initialization stores immutable UTF-16 original values and sets lastIndex to +0.
Subclass prototypes and bound construction use the existing construction path.
Original-slot storage is boxed to keep recursive native frames small; shared
Strings remain owned through collection and do not introduce heap edges.
Source uses EscapeRegExpPattern; flag getters read validated original flags.
Brands are own slots, never inherited or inferred from public properties. The
ordinary intrinsic prototype retains its special source/flag getter results.
IsRegExp's undefined-marker fallback and Object.prototype.toString inspect the
native brand. Exec requires that brand before converting its argument.
Supported ordinary-mode Patterns compile to an immutable shared matcher;
broader valid syntax retains an explicit Unsupported execution result.
No matcher dependency or default quota is activated.

RegExpBuiltinExec (22.2.7.2) converts lastIndex after the input, including when g/y
are absent. Original flags choose search/sticky behavior, strict lastIndex writes
and d indices; public flag/source overrides cannot change execution. Non-global,
non-sticky matching starts at zero without writing lastIndex. A failed g/y search
resets +0; success writes its end before result construction. A past-end offset
fails without invoking the matcher. Fresh intrinsic Arrays own the whole match
and ordered capture strings, index, input and groups (undefined without names).
With d, indices
owns an intrinsic start/end pair for the whole match and each capture, plus
a groups property (22.2.7.8). All offsets count UTF-16 units. Array and
prototype overrides cannot intercept these data properties. RegExpExec's
non-callable-exec fallback uses the same native operation; custom exec and all
generic consumers retain live lookup. Opted-in work accounting charges the
compiler and linear search; host aborts remain outside JavaScript handlers.

Ordinary named captures reuse these plans (CapturingGroupName, 22.2.1.5;
CompileSubpattern, 22.2.2.3; RegExpBuiltinExec, 22.2.7.2). Validated GroupSpecifier
ranges remove only name prefixes from the private matching source, retaining
ordinary capturing parentheses, source-order slots, flags and original source
text. Decoded names bind to source-order slot lists, with duplicate names merged
in first-appearance order. MightBothParticipate ensures at most one such slot
participates; a selected empty capture remains distinct from undefined.
Immutable metadata is shared through Arc and contains no heap-object edges.
Existing fixed, repeated, partial, anchored and alternative capture plans supply
the ranges. Result groups are fresh null-prototype ordinary Objects with own
writable, enumerable, configurable data properties for every name, including
unmatched groups. With d, indices.groups has the same keys and refers to the
already allocated numeric capture pairs; updates through either reference affect
the shared pair. Reassigning a property replaces that reference independently.
Prototype hooks cannot intercept initialization, and generic consumers receive
the resulting groups normally. Optional construction/result work remains
explicitly charged; defaults remain unlimited. Broader reference compositions,
unsupported conditional local choices and Unicode-mode matching remain pending.

Ordinary Patterns containing numbered or named references execute
through a flat immutable instruction program (CompileDisjunction, 22.2.2.3;
BackreferenceMatcher, 22.2.2.9). Capturing/noncapturing groups, arbitrarily nested
or sequential alternatives, literal units, character predicates and input/line/
word assertions compose without source expansion. Complete source validation and
subset preparation precede predicate construction and optional host charging.
Enclosing groups stay in the program to preserve global source-order capture slots.

Group preparation reserves an entry instruction and records source-order branch
entries. A completed disjunction replaces that entry with an ordered choice and
patches its forward jumps to the common continuation before the enclosing capture
closes. Non-disjunction entries remain no-ops. All instructions and targets stay
flat; construction and destruction use no native recursion. Storage remains
linear in the original Pattern, including deeply nested choices and wide shared
prefixes; source and continuation instructions are never copied per alternative.

Search checks increasing candidate starts, or only the requested sticky start.
Execution retains completed input capture ranges, separate open-capture start
positions, and a trail of captures completed by the current path. Each ordered
choice saves its next alternative, input position and completed-capture checkpoint
in a flat frame buffer. A failed body or continuation clears only captures
completed after the saved checkpoint and resumes the next source-order alternative.
Exhausted frames are discarded iteratively. Common prefix captures stay available;
groups opened before a choice keep their original starts when closing after a
retried body. Inactive captures remain undefined, while participating empty groups
retain defined empty ranges. Forward and currently open references match empty;
completed references compare actual original input units using pinned ordinary
case canonicalization. No reference value is expanded into source or a literal.

Named references use shared checked binding inventories and direct instructions
(CompileAtomEscape, 22.2.2.9). A forward merge removes only capture-name specifiers,
retains original named escapes, and records their new absolute UTF-16 intervals
and binding indices. Following decimal characters remain independent literals;
public source/flags, copies, group names and indices aliases retain their original
behavior. Compilation checks ordered/disjoint escape intervals, slot bounds,
source-order inventories and unique slot ownership before any host charging.

MightBothParticipate is checked using an iterative stack of alternative/current
name sets, matching ParsePattern's participation rule. Concatenated terms must
have disjoint possible names; distinct alternatives union their inventories.
A group's own name belongs to its parent term and can coexist with its body's
names. Merging moves the larger set allocation, so names are not copied through
every enclosing group. Recorded set operations are charged only after the complete
program is accepted. Malformed inventories return unsupported before charging.
The retained program needs only one capture-to-name map and a scalar binding index
per named reference. Closing a named capture updates one completed input range;
restoring failed captures clears their name values. Name lookup remains constant
work per reference, independent of the number of alternative slots.

Character classes, class escapes and dot reuse pinned ordinary Canonicalize,
inversion and DotAll behavior (CompileAtom, 22.2.2.7). Identical UTF-16 atom source
shares one immutable predicate per compilation, with charged setup, lookup and
first construction. Each predicate consumes one original UTF-16 unit. Assertions
reuse the existing conjunction at the current absolute input position and inspect
the complete input with the validated Multiline flag (22.2.2.4). Later search or
sticky positions preserve preceding units and input boundaries. Ordinary word
membership stays ASCII even with IgnoreCase; line assertions cover all four
ECMAScript line terminators.

Original absolute capture ranges feed the existing result builder, including
null-prototype named groups, undefined slots and shared indices pairs. The program
contains no heap-object edges and survives copying and collection. Optional work
charges frame buffers, executed instructions, branch attempts, capture trail
writes/resets, assertions and actual comparisons; aborts occur before lastIndex
mutation and remain outside language handlers. Default work and heap quotas stay
disabled. Composite reference-containing repetitions, other consuming
quantifiers, lookaround/scoped modifiers and Unicode matching remain pending.

Reference atoms accept greedy/lazy `*`, `+`, `?` and bounded quantifiers
(CompileQuantifier, 22.2.2.5; RepeatMatcher, 22.2.2.3.1). Decimal bounds reuse exact
integer parsing: a consuming minimum exceeding every representable input cannot
match, while an oversized maximum cannot constrain that input. Repetition counts
are bounded by remaining input units and the original nonempty capture width.
Greedy execution validates the longest available run and saves shorter endpoints;
lazy execution validates its minimum and compares one additional original capture
when a failed continuation requests another repetition. Frames restore captures
completed by the failed continuation before retrying; enclosing starts and prefix
targets retain their original ranges. Any number of reference atoms can quantify
without copying source or input strings.

A reference atom contains no capturing parentheses and has no capture effects.
Undefined or empty targets therefore preserve the same MatchState for every
required finite repetition, including minimums too large for native integers.
Execution can satisfy that minimum directly; optional empty iterations stop under
RepeatMatcher's zero-progress rule. This optimization keeps huge finite empty
reference bounds correct and avoids native loops or recursion. Typed named binding
inventories must register every named escape outside classes.

A quantified noncapturing group containing one or more occurrences of the same
unquantified reference and any transparent noncapturing wrappers uses the same
reference instruction and repetition frames. Compilation inspects its prepared
body at the quantifier, accepts only that target and no-op group boundaries,
and retains the original
capture ordinals and source. Empty noncapturing groups around the reference have
no MatchState effects. Nested unquantified wrappers need no repeated body scans;
100,000 wrappers remain iterative with linear storage. Different references,
consuming terms, alternatives, assertions and an already
quantified reference prevent this simplification. Default quotas stay disabled;
composite reference repetitions, quantified consuming atoms,
lookaround/scoped modifiers and Unicode matching remain pending in this program.

Capturing groups enclosing the whole reference body use this repetition
instruction. Preparation checks balanced capturing boundaries before/after the
body, records their original slots, and replaces only those boundaries with
no-ops. Each selected nonempty repetition endpoint determines every enclosing
slot's final-iteration range; zero iterations leave these slots undefined
(RepeatMatcher, 22.2.2.3.1). The target is a completed external prefix capture or
an undefined capture within the repeated body: the latter remains undefined when
RepeatMatcher clears its captures before each iteration. Required empty
iterations therefore leave empty final captures even for huge finite minimums;
optional empty iterations leave them undefined under the zero-progress rule.
Repetition frames save their checkpoint before writing wrapper captures. Failed
continuations clear those writes before a retry installs the new final ranges,
including their shared named bindings. Capture inventories and writes receive
opt-in work charges. Deep enclosing groups remain iterative with linear storage.
Empty sibling captures before and after the whole reference body are also accepted.
Preparation assigns each capture a before/reference/after span rather than
replaying its boundaries during every iteration. Its final range is the final
iteration's beginning, whole body span or ending, respectively. An internal
target before the reference is defined and empty; other internal targets are
undefined until the reference finishes. Both match the empty string, so the same
deterministic empty-target execution remains valid. Zero iterations leave all
body captures undefined. Required empty iterations preserve all their empty final
ranges even for huge finite minimums. Compact spans keep 100,000 empty siblings
iterative with linear storage.

Identical references within a repeated body share one target descriptor and an
exact multiplicity. Quantifier bounds continue to count whole bodies rather than
individual reference occurrences (RepeatMatcher, 22.2.2.3.1). Consuming count
limits divide remaining units by the original capture width and then multiplicity
before calculating any stride. Zero iterations avoid stride multiplication;
saved retry counts guarantee the stride and its selected endpoint fit the input.
Comparisons validate every occurrence against the original range, and retries
move by complete body lengths. Whole enclosing captures cover the final body,
with empty before/after positions retained. Empty targets satisfy finite minimums
without iterating over occurrences or required repetitions. Preparation needs no
source expansion or multiplied quantifier bounds, including 100,000 occurrences.
Other consuming terms, alternatives and nested quantifiers remain pending in
this program.

Repeated groups may also contain different numbered/named targets.
Preparation retains an ordered descriptor inventory in one sequence instruction,
with no source expansion. Entry freezes the completed original input ranges;
undefined/empty targets consume nothing. Checked addition computes the whole
iteration width. Overflow makes consuming iterations impossible while preserving
the valid zero-count path; a wholly empty sequence satisfies finite required
minimums directly (RepeatMatcher, 22.2.2.3.1). Count limits use remaining units
before any endpoint arithmetic. Greedy matching validates whole sequences; lazy
matching validates the minimum and retries one whole sequence at a time. Frozen
ranges remain valid through failed-continuation capture resets. Buffers, target
lookups, instruction/frame work and actual comparisons receive opt-in charges.
Any number of targets and sequences fit flat storage, including 100,000 reference
occurrences, and numbered/named descriptors may alias one original capture.
Whole enclosing captures and empty before/after siblings share the same compact
spans as identical-target bodies. Internal targets are undefined until their
whole enclosure closes, or defined and empty before the references; both consume
nothing at each iteration after RepeatMatcher clears the body's captures.
Completed external targets stay frozen. Frames record the capture checkpoint
before selected body ranges are written. Failed continuations clear those writes,
then a whole-sequence retry installs new final-iteration ranges and named aliases.
Zero iterations leave every body slot undefined; required empty iterations leave
empty final slots even for huge finite minimums. Deep enclosures remain iterative,
including 100,000 capturing wrappers.
Empty captures between references retain their original reference ordinal.
Checked prefix widths include every target, including undefined and empty ranges,
so a final empty capture lands after exactly its preceding references. Identical
references derive the prefix from their shared width and multiplicity; mixed
sequences retain a flat prefix-offset buffer with their retry frame. Required
empty iterations place all empty captures at the starting position, while zero
iterations leave them undefined. Internal empty targets consume nothing whether
already closed or still undefined after RepeatMatcher clears body slots. Final
positions and named aliases are restored after whole-body retries without replaying
capture boundaries per iteration. Buffers and writes receive opt-in work charges;
100,000 empty captures between references remain iterative.
Partial capture ranges use the same checked prefix widths and original reference
ordinals. Each capture records a start/end pair; final writes add those offsets
to the selected last iteration, including ranges that are empty because an
external target did not participate. References following the complete repetition
then read those final original input ranges. Preparation maps numbered/named body slots to their original spans once.
An open or forward target is explicitly empty after RepeatMatcher clears body
slots. A completed body target uses its current iteration's relative start/end
range; external targets retain frozen absolute input ranges. Prefix widths are
computed in source order, so completed local spans only read earlier offsets.
Checked addition handles chained local widths without expansion. Overflow or a
width larger than remaining input prevents a consuming body while preserving a
valid zero-count path. Count limits bound the whole body before any relative
range is added to its current iteration's input base. Greedy matching validates
whole bodies; lazy retries use a fresh iteration base for every local target.
Capture checkpoints then install final partial ranges and named aliases. Required
empty iterations validate their constant empty effects directly even for huge
finite minimums; optional zero counts retain undefined slots. Flat descriptor and
range buffers keep 100,000 local references iterative without copied literals.
Global slot/name lookup is prepared once, while each body is scanned linearly.
Repeated reference bodies also accept ordinary character units, character sets,
class escapes and dot. Their descriptor retains a flat root instruction index;
ordinary traversal skips those owned instructions while each body comparison
executes them in source order. Predicates share the same pinned immutable cache
as outside terms, retaining IgnoreCase and DotAll without source expansion.
One-unit terms contribute exactly one prefix unit, so body capture spans and
completed local targets use the same checked geometry. Actual input values may
change between iterations when a set or dot accepts different units; relative
ranges preserve those values for later references and final capture writes.
Comparisons validate every body, and whole-body retry checkpoints restore the
chosen final ranges. A consuming unit prevents a huge positive finite minimum
from fitting short input, while a valid zero-count path leaves body slots
undefined. Descriptor buffers, root lookup, predicate construction and actual
comparisons receive opt-in charges. Deep capturing bodies remain iterative,
including 100,000 partial enclosures, and 50,000 identical predicates share
construction.
Ordinary input/line and word assertions also retain flat owned instructions.
They contribute zero prefix width and test the complete input at the current
body position, using the original Multiline flag and ordinary word predicate.
Every consuming iteration validates its assertions before final capture writes;
lazy retries use the new iteration position and greedy retries select an already
validated body. Assertions inside zero-width required bodies are validated once
at their unchanged position before installing empty final captures, including
huge finite minimums (RepeatMatcher, 22.2.2.3.1). Optional empty iterations stop
without capture effects under the zero-progress rule, even when the assertion
would fail. This preserves full-input sticky boundaries and defined-empty versus
undefined result slots without replaying a mandatory count. Root lookups,
assertion work and buffers receive opt-in charges. Captured empty assertion
siblings remain iterative, including 100,000 groups.
Deterministic repetition groups need not contain a reference themselves when the
complete Pattern contains references elsewhere. Their original group entry owns
the flat sequence instruction, including an empty body; actual unit/assertion
instructions remain separate owned terms. The same checked widths and final
capture spans supply outside references after the selected body count. Greedy
and lazy retries clear later writes before reinstating final ranges, including
mutually exclusive named slots. Empty required groups define empty final captures,
while optional empty groups retain undefined slots under the zero-progress rule
(RepeatMatcher, 22.2.2.3.1). Pure empty bodies satisfy huge finite minimums directly.
Patterns without any reference retain their existing matchers. Deep capturing
repetition groups remain iterative, including 100,000 enclosures.

Quantified character, set, class-escape and dot atoms in reference programs use
the same flat sequence instruction (RepeatMatcher, 22.2.2.3.1). A separate entry
precedes the owned unit, preserving earlier branch/group indices and shared
pinned predicates. Parent capture opens/closes remain ordinary instructions, so
`(a+)` records its entire selected prefix while `(a)+` records the final body
iteration. Retry checkpoints undo later closes before selecting another atom
count; multiple sequential quantified units preserve their parent ranges. A
zero-count atom still reaches its enclosing close and defines an empty capture.
Checked consuming bounds reject impossible huge minimums without replaying them;
huge maximums retain the available input bound. All storage and construction are
iterative, including 100,000 enclosing captures, and host quotas stay opt-in.
Alternatives in repeated bodies, nested quantifiers, lookaround/scoped modifiers
and Unicode matching remain pending.

The flat ordinary matcher also compiles supported Patterns without references
(22.2.2.4, 22.2.2.3.1). Its reference-specific constructors still reject a body
without an actual reference, including decimal-looking class escapes. A shared
prepared-program compiler validates named slot inventories and exclusive names
before charging accepted work. Runtime construction tries the existing specialized
matchers first, then compiles this fallback from the complete name-stripped source.
Captures peeled for specialized plans are restored by resetting the enclosure
prefix before installing the fallback; every match agrees with the original
CapturingGroupsCount. Ordinary nested/sequential alternatives and multiple
quantified character units use the existing flat frames and dynamic ranges,
including inactive duplicate-name slots and greedy/lazy parent captures. Default
quotas remain disabled. General repeated choices, nested quantifiers, lookaround,
scoped modifiers and Unicode matching still require implementation.

Capture-free repeated alternatives are accepted when every source-order branch
contains a consuming character, set, class escape or dot (RepeatMatcher,
22.2.2.3.1). Assertions and outside references may accompany those units, but
cannot establish progress themselves; open and forward targets remain empty.
Admission checks the complete body before charging accepted work and rejects
body captures, inner choices, nested quantifiers and possibly empty branches.

The original entry choice moves to a tail entry; original branch instructions and
jumps retain their indices. A repeat-end instruction connects each successful
body to its next count decision. Counts are bounded by actual consumed input,
so huge required consuming bounds reject without replaying them. Each existing
flat retry frame saves its optional iteration context. Greedy/lazy continue and
exit alternatives are consumed once, and restoring an earlier body choice also
restores that iteration's count and input position. Capture checkpoints undo
later enclosing closes before selecting another branch/count. Sequential loops
restore their contexts independently, with no nested body loops admitted. The
same program supplies full outside captures, named aliases and intrinsic
consumers. All frames and preparation remain iterative; default quotas stay
disabled. Body captures, empty alternatives, inner choices and nested repetition
still need broader repeat semantics.

Whole-body progress is checked across nested and sequential capture-free choices
(RepeatMatcher, 22.2.2.3.1). The unquantified body forms an acyclic forward graph.
A backwards Boolean pass marks consuming units, follows checked forward jumps,
requires every choice target to progress, and carries common suffix progress
through assertions, references and no-op group entries. Thus an empty inner
alternative is accepted when a shared mandatory term still consumes input.
Captures, nested repetition instructions and possibly empty complete body paths
are rejected before accepted host work is charged. The proof is linear in body
instructions and choice edges, with flat temporary storage.

A neutral original group entry uses a tail jump to its next instruction; an
entry choice retains the prior tail-choice layout. Original branch positions
stay stable. Existing iteration contexts and retry frames select nested and
sequential source-order choices without another execution engine, restoring
counts and later parent closes before retries. Deep noncapturing wrappers and
wide shared paths remain iterative, including 100,000 wrappers and 10,000
branches. Default quotas remain disabled.

Deterministic capture-free quantifiers compose inside progressing repeated
bodies (RepeatMatcher, 22.2.2.3.1). Aggregate reference/term instructions with
empty capture inventories retain their existing count retries. A required
aggregate containing a character or prepared set establishes progress; optional
aggregates, assertions and outside references rely on a consuming continuation.
Owned character/set/assertion terms are skipped by ordinary traversal and do
not independently establish progress. The backwards proof still checks all
instructions and rejects capture effects, nested branch repetition and possibly
empty complete body paths before charging accepted work.

Body captures remain read-only, so inner count frames safely retain the outer
iteration context. Retrying an inner aggregate restores that context before
continuing the body; finishing the whole body increments only the outer count.
Existing checkpoints undo later enclosing closes. This extends preparation
without new execution instructions, contexts, source expansion or default
quotas. Deep enclosing groups, long iteration runs and huge bounds remain
iterative and fallible.

Capture-free branch repetitions nest through proven child summaries and
parent-linked iteration contexts (RepeatMatcher, 22.2.2.3.1). Each prepared loop
retains its original body entry. A parent treats a previously proven child as
one capture-free operation: required child iterations consume input, while
optional children need a consuming continuation. Preparation reuses its Boolean
progress buffer and skips child bodies. Remembering the latest prepared loop
also avoids rescanning it as a deterministic reference wrapper. Deep nesting
therefore retains flat storage and linear preparation over the prepared graph.

Execution stores immutable iteration records in a flat vector. Each record
retains its parent, loop instruction, count and input start. Entering a child
creates a new record; completing an iteration creates an updated record.
Exiting restores the parent. Retry frames save the current record index and
vector checkpoint alongside the capture checkpoint. Restoring a frame truncates
abandoned records without invalidating live frames or parent links. Buffers are
cleared between candidate starts. Allocation work is charged through the existing
fallible callback; defaults remain unlimited. Captures inside branch loops and
possibly empty complete body paths remain pending.

Captures compose inside progressing repeated branches and nested loops
(RepeatMatcher, 22.2.2.3.1). Group frames record the first source-order capture
slot. Each loop owns the contiguous range through its closing capture count,
including nested loops and aggregate quantifiers. This avoids copied inventories
or rescanning proven child bodies. Open/close instructions preserve the minimum
progress proof; references remain potentially empty. Nullable complete body
paths use the zero/one-minimum execution described below; larger required nullable
counts still need an additional proof.

Entering each body clears its capture range and named aliases. A captured root
entry retains its open effect in loop metadata, while the original body positions
remain stable. The capture trail records previous starts, completed ranges and
named values, including clearing effects; unchanged values need no undo record.
Retries restore these changes in reverse order to their checkpoint, alongside
parent-linked iteration contexts. Thus unselected captures become undefined on
the next iteration, while reducing a count restores the previous iteration's
final ranges. Ordinary and aggregate closes use the same range writer, including
zero-count aggregate clears. Open and forward references observe cleared current
iteration targets. All execution and rollback remain iterative, with fallible
work accounting and unlimited defaults.


Positive and negative ordinary lookahead compose with choices, captures,
references and proven progressing repetitions (Assertion, 22.2.2.8). The original
assertion entry jumps to a flat control instruction. Its body ends at an explicit
assertion terminal; original branch positions and exit targets remain stable.
Preparation remembers complex group controls so deterministic wrapper checks do
not rescan nested assertion bodies. A parent repetition skips a proven assertion
body and uses only its continuation to establish consumption: examining input
never proves progress. Quantified lookahead remains outside the core grammar.

Assertion contexts retain the input position, capture checkpoint, enclosing choice
count, iteration record and iteration checkpoint. Body success discards its retry
frames and restores the input/iteration position. Positive success keeps captures,
including ranges extending beyond the whole match; negative success restores
captures and fails. Exhausted body choices unwind before outer choices: negative
failure succeeds with the original captures, while positive failure propagates.
Nested contexts and retry depths remain flat. This preserves atomic positive
assertions and prevents an outer continuation from reconsidering their captures.
The same capture undo trail restores starts, ranges and named aliases. Deep
positive and negative nesting, cloning and dropping remain iterative; host work
is fallible and limits remain opt-in. Other lookbehind bodies, scoped modifiers,
Unicode-mode matching and possibly empty consuming repetition paths remain pending.


Transparent repetition wrappers containing lookahead use a proven zero-width body
(RepeatMatcher, 22.2.2.3.1). Preparation follows ordinary neutral/capture/assertion
instructions and skips validated lookahead or previously proven zero-width child
bodies. Choices, references and consuming terms outside these assertions remain
outside this increment. Each wrapper retains its original body entry and capture
range. The backwards progress proof treats this child as zero-width, so only a
consuming parent continuation can establish iteration progress.

If the minimum is zero, optional zero-progress iterations are rejected and their
capture slots stay undefined. For a positive minimum, entering the body clears
its capture range and restores its root open effect. The first successful body
execution has the same input position, prefix state and final captures as every
required repetition: internal captures are reset between iterations, assertions
are atomic and no consuming operation or outside choice is accepted. Required
iterations therefore collapse to one body execution, including counts larger
than platform capacity. The terminal returns directly to the original
continuation; no count frame or recursive execution is needed. Inner captures
may extend beyond the empty outer capture. The shared undo trail restores these
ranges and named aliases on outer retries. Nested wrappers, copies and dropping
remain flat and host limits remain opt-in.


Zero-width lookahead repetition bodies also admit nested and sequential choices
when every complete path is zero-width (RepeatMatcher, 22.2.2.3.1). A backwards
Boolean proof checks all original forward edges. Ordinary assertions and capture
instructions inherit their continuation; every choice branch must prove zero
width. Lookahead and previously proven zero-width child repetitions summarize
their bodies. Consuming terms and outside references remain rejected. Preparation
reuses the proof buffer and skips child bodies, keeping nested wrappers linear.

A moved original entry choice lives at the wrapper tail, preserving branch
positions and exit jumps. Required iterations still collapse to one execution:
clearing internal captures makes each iteration independent at the same input
position, so only the final iteration's path can affect the outer continuation.
The ordinary choice frames retain source order and restore capture starts, ranges
and named aliases when that continuation fails. Positive lookahead remains
atomic inside each branch; retrying an outer choice never reopens it. Optional
zero-progress iterations still leave all owned slots undefined. Parent consuming
loops treat the whole child as zero-width and retain their per-iteration clearing.
All preparation, execution, rollback, cloning and dropping remain flat, with
fallible opt-in work accounting.

Fixed capture-free ordinary positive and negative lookbehind use a separate flat
control instruction (Assertion, 22.2.2.8). Preparation accepts character terms,
ordinary word/input/line assertions and transparent noncapturing groups. It proves
an exact UTF-16 width and rejects captures, alternatives, quantifiers, references
and nested assertions inside the body. The original body entry jumps to its
control; original surrounding captures, choices and continuation positions stay
unchanged. Preparation records this control as a complex group, preventing a
wrapper from treating its examined characters as forward consumption.

For a candidate input position, the body has exactly one possible start. If the
available prefix is shorter than the proved width, positive lookbehind fails and
negative lookbehind succeeds. Otherwise the immutable flat body is evaluated from
that start with the full input and existing IgnoreCase, Multiline and DotAll
predicates. In this subset, the same fixed character positions and boundary
positions are checked by backward Assertion evaluation; there are no capture or
branch effects whose order could differ. Assertions retain their full input
context rather than seeing a sliced prefix. The outer cursor and capture state
are unchanged, and negative lookbehind inverts the result.

Parent progress and zero-width proofs summarize the complete lookbehind body as
one zero-width operation. Required transparent assertion repetitions therefore
retain the existing finite collapse, optional zero-progress iterations keep
undefined slots, and consuming parent loops use only their forward terms to
establish progress. Execution charges the actual fixed body operations and unit
comparisons; native stack usage stays flat through deep transparent groups,
cloning and dropping. Limits remain opt-in, with unlimited defaults. Other
lookbehind bodies and Unicode-mode matching remain pending.

Capture-free fixed lookbehind additionally admits exact repetition counts of one
ordinary character or class term (22.2.2.3.1, 22.2.2.8). Preparation retains the
existing quantified term instruction and verifies equal representable minimum
and maximum, one consuming term, and no body captures or references. Checked
width addition accounts for its complete count; the stored character or class
unit is owned by that repetition and contributes no second width. Zero counts
contribute zero width. Greedy/lazy spelling cannot change an exact-count result.
Transparent noncapturing wrappers around a single unit share the same plan.

Execution compares that retained term against each position in the proved input
prefix. It charges each actual unit comparison and retains no expanded source,
repeated term vector or count-dependent allocation. Too-short prefixes fail
before count iteration; even a platform-maximum count remains compact and finite
for representable inputs. Ordinary characters, classes and their counted terms
share one comparison helper, preserving existing IgnoreCase and DotAll behavior.
Input/line/word assertions still use the whole input, and outer captures, cursor,
negative inversion and repetition summaries retain their existing semantics.
Counts whose complete fixed width cannot be represented, variable counts and
repeated boundary assertion bodies are described below. The subsequent
extensions admit nested fixed lookbehind, unrepeated captures and counted
character sequences. Host quotas
remain opt-in; defaults are unlimited.

Nested fixed capture-free lookbehind retains the same ordinary character/count
and boundary predicates (Assertion, 22.2.2.8). The width proof now walks the flat
body backwards and summarizes each completed lookbehind child as zero-width,
skipping its complete instruction region. Original child entries jump to their
owners. Deep nesting therefore needs no repeated scan of descendant bodies;
each child keeps its independently proved fixed width and polarity. Other nested
assertions, references, alternatives and variable counts remain pending;
unrepeated captures are described below.

Execution uses explicit fixed-assertion frames containing the body instruction,
terminal, local cursor, expected end position and polarity. A child starts from
the parent's exact local position and retains a saved parent continuation. Its
success resumes that parent without moving the parent cursor; failure propagates
through the parent's polarity. Unavailable prefixes and failed body predicates
share the same Boolean propagation. The outer matcher cursor and all outside
captures remain unchanged. Debug checks verify each successful fixed body's
expected endpoint. No Rust recursion is used for nested assertion execution.

Capture-free fixed bodies check identical absolute character and boundary
positions in forward or backward evaluation. Nested assertions only contribute
Boolean predicates at those positions, so their source evaluation order adds no
capture or choice effects. Every child sees the complete input and original
flags, preserving line and word boundaries. Parent progress/zero-width summaries,
required/optional wrapper rules and outer source-order retries remain intact.
Actual body instructions, child frame pushes/pops and unit comparisons are
charged; limits remain opt-in and defaults unlimited. Preparation, execution,
cloning, collection and dropping remain flat through deep positive/negative
nesting. Unicode-mode matching and the other lookbehind body features remain
pending.


Unrepeated captures inside fixed ordinary lookbehind share the existing capture
trail (22.2.2.8). Preparation accepts capture open/close operations as zero-width
while retaining the exact character/count proof. Internal references, variable widths and nested lookahead remain excluded.
Equal-width choices are described below. Exact-count repeated
character captures are described below.
Each unrepeated capture therefore has the same absolute endpoints under forward
and backward body evaluation. Exact-count character terms may occur inside an
unrepeated capture; exact-count capturing sequences are described below.

Fixed assertion frames retain a capture checkpoint in addition to their local
input position. Open/close operations record full-input starts and ranges, which
can precede the whole match's start. Positive success retains these captures.
Every negative outcome and every failed positive body restores starts, ranges
and named aliases before returning to its parent or outer continuation. Nested
checkpoints preserve captures established before the child, including a negative
child that completes captures before a later predicate fails. Outside references
read successful positive ranges through the existing binding inventory; result
construction and indices retain the original absolute input coordinates.

Parent progress and zero-width proofs still summarize the complete assertion
owner. Consuming iterations clear their capture inventory, required zero-width
wrappers retain captures and skipped optional iterations leave them undefined.
Flat preparation, assertion frames and capture undo support deep groups without
Rust recursion. Actual capture writes and restoration charge fallible opt-in
work; default quotas remain unlimited. Other lookbehind bodies and Unicode-mode
matching remain pending.


Exact-count capture-free ordinary character sequences also use compact fixed
lookbehind plans (22.2.2.3.1, 22.2.2.8). Preparation requires a nonempty sequence
of one-unit character/class predicates, no body captures or references, and equal
representable minimum/maximum counts. It checks complete width multiplication
and addition without expanding source or terms. Transparent group structure is
retained; capture enclosures outside the repeated sequence remain unrepeated and
keep their proved full-input endpoints.

Execution compares each retained sequence in source order for the exact count.
Every predicate sees the same absolute input unit under backward Assertion
semantics because the body has no capture, reference, assertion or choice effects.
A mismatch fails immediately. Zero counts inspect no terms, and too-short input
prefixes fail before iteration even for a platform-maximum representable width.
Iteration allocates no count-dependent state and charges each actual comparison.
Nested fixed assertion frames, negative capture rollback, boundary context and
parent progress/zero-width summaries retain their existing behavior. Variable
counts, repeated lookaround, nested counted sequences and internal
references/choices remain pending. Defaults remain unlimited.


Exact-count capturing character sequences retain the same fixed-width plan
(RepeatMatcher, 22.2.2.3.1; Assertion, 22.2.2.8). Their bodies contain only one-unit
character/class predicates and transparent group structure, with no internal
capture reads, alternatives, assertions or nested counts. Preparation retains
original capture slots and term-relative spans; checked complete widths still
bound every comparison. Capture enclosures outside the repeated sequence remain
unrepeated and retain their complete ranges.

Backward repetition finishes with the leftmost input iteration. After a successful
exact-count comparison, execution records each repeated slot's span relative to
that first forward input iteration. It does not use the forward matcher's final
iteration. Empty partial captures retain their leftmost boundary, while a zero
count writes undefined slots rather than manufacturing an iteration. The body
has no capture reads, so only the final observable ranges need to be written;
no capture-start mutation or per-count capture state is required.

Capture writes use the existing trail and named binding inventory. Negative
assertions and failed surrounding bodies restore them to their frame checkpoint;
outer retries and consuming parent loops retain their normal capture clearing.
Positive ranges before the whole match's start feed outside references, result
indices and consumers unchanged. Comparison and capture work remains fallible
and opt-in. Deep capture inventories, copies, collection and dropping stay flat;
default quotas remain unlimited. Variable counts, branch/lookaround/reference
bodies, empty-only repeated bodies and nested counted sequences remain pending.


Equal-width alternatives inside fixed ordinary lookbehind retain source order
(Disjunction, 22.2.2.4; Assertion, 22.2.2.8). Preparation computes sparse backward
suffix widths across the flat branch graph. Forward branch exits read their
already-proved destination width; every alternative at a choice must have the
same complete suffix width. Nested fixed assertion owners contribute zero and
skip their complete regions, keeping preparation linear through deep nesting.
Character/count widths remain checked, and internal references, variable counts,
differing-width choices and nested lookahead remain excluded.

The assertion owner retains its root branch inventory when its original entry
becomes a jump. Execution uses flat pending branch records containing the branch
instruction, next alternative, local input cursor and capture checkpoint. A
failed branch restores completed partial captures, starts and named aliases
before trying the next alternative. Each nested assertion frame records its
branch-stack base so its retries and discard affect only that frame; parent
branch continuations remain available. Source-order alternatives inspect fixed
absolute positions without internal capture reads, and counted character
captures retain their existing leftmost iteration ranges.

A successful body discards its remaining branches before returning its assertion
result. Outside continuation failure therefore cannot reopen an accepted positive
lookbehind. Negative assertions invert the whole body result and restore all
owned captures; nested failures propagate through the existing polarity and
checkpoint rules. Empty same-width branches and boundary-only choices preserve
zero-width positions. Parent progress and required/optional wrapper summaries
remain unchanged. Actual branch selection, retries, discard and capture undo
charge fallible opt-in work. Preparation, execution, copies and dropping remain
flat; default quotas remain unlimited. Repeating a choice inside lookbehind and
other unproved body forms remain pending.


Exact-count ordinary boundary predicates now compose with character terms in
fixed lookbehind (RepeatMatcher, 22.2.2.3.1; Assertion, 22.2.2.8). Preparation
assigns input/line/word predicates width zero and character/class predicates
width one, then checks complete counted widths. For bodies containing zero-width
terms it stores immutable prefix offsets from term ordinals to UTF-16 input
positions. Capture spans use those offsets, preserving partial and empty ranges
rather than mistaking an assertion ordinal for a consumed input unit. Accepted
owners receive metadata once; summarized nested children retain their own plans.
The ordinary forward matcher retains its existing runtime offset calculation.

Execution evaluates each boundary against the full input, original Multiline
flag and exact local position without consuming a unit. Character comparisons
retain their existing flags. Pure boundary bodies have no capture reads, choices
or moving positions: a required exact count therefore executes their predicates
once and writes the identical final capture effects. A zero count executes no
predicates and leaves repeated slots undefined. Huge representable required
counts remain compact. Mixed consuming bodies execute their actual term/count
comparisons; repeated captures retain their leftmost backward input iteration.

Root alternatives, nested assertions, atomic results, negative rollback and
required/optional parent wrappers share their existing checkpoints and width
summaries. Captures and named aliases use original absolute input coordinates,
including empty ranges before the whole match's start. Work charges actual
predicates and capture operations; default quotas remain unlimited. Metadata and
execution allocate no count-dependent state. Nested lookaround inside a counted
body, variable counts, repeated choices, internal references and nested counted
sequences remain pending; empty-only repeated bodies are described below.


Exact-count empty bodies inside ordinary lookbehind share the existing zero-width
count execution (22.2.2.3.1, 22.2.2.8). The fixed-width proof accepts an empty term
inventory when the minimum and maximum are equal and representable. Transparent
and nested empty captures retain original slots with zero term-relative spans.
A required count records their empty ranges once at the body's local absolute
input boundary; a zero count leaves repeated slots undefined. Surrounding
unrepeated captures still record their complete body range.

An empty sequence may precede or follow consuming terms, so its capture boundary
can differ from the whole match's start. It moves no input and allocates no
count-dependent state. Negative/failed-body restoration, root alternative
atomicity, named aliases, outside empty references and parent wrapper summaries
retain their existing behavior. Deep capture inventories, copies, collection and
dropping remain flat. Actual capture work is fallible and opt-in, with unlimited
defaults. Variable counts, repeated choices, internal references, nested counted
sequences and lookaround inside a counted body remain pending.


Variable counts of pure zero-width bodies in ordinary lookbehind use the same
fixed-position plan (22.2.2.3.1, 22.2.2.8). After validating every term, the width
proof accepts any bounds only when the complete iteration width is zero. Empty
groups and ordinary boundary predicates cannot read captures or change input.
Consuming sequences still require equal, representable minimum/maximum counts
and checked total widths.

RepeatMatcher rejects optional zero-progress iterations. A zero minimum therefore
skips predicates and leaves repeated captures undefined, even for a greedy count.
A positive minimum evaluates the body once and retains its empty ranges at the
local boundary; repeated predicates and final capture effects are identical at
that position. A minimum exceeding usize remains required rather than becoming
unbounded or zero. Greedy/lazy, bounded/unbounded and oversized bounds need no
expanded source, per-count allocation or count-dependent traversal. The existing
source-order choices, atomic assertion results and negative rollback remain intact.
Explicit host work quotas are fallible and opt-in; defaults stay unlimited.
Variable consuming bodies, internal references, repeated choices, nested counted
sequences and lookaround inside a counted body remain pending.


Repeated pure empty and boundary alternatives use the existing zero-width wrapper
executor (22.2.2.3.1, 22.2.2.8). Preparation considers the zero-width proof for every
quantified group, admitting choices only when every complete source-order branch
is zero-width. Linear reference-free bodies retain their compact sequence plan.
Child assertion/repetition summaries and checked forward jumps preserve flat
preparation without rewriting original capture or branch indices.

Required identical zero-width iterations execute one body with its final capture
state; optional zero-progress iterations are skipped and leave slots undefined.
The existing branch checkpoints restore starts, ranges and named aliases before
retrying a failed arm. Pure boundary alternatives use full input context, including
multiline boundaries. Enclosing wrappers, parent consuming loops and ordinary
assertions retain their prior behavior. Huge bounds and deeply nested captures
require no source/count expansion or native recursion. Work remains fallible and
opt-in with unlimited defaults. Mixed empty/consuming repetitions, internal
references, counted choices inside lookbehind and Unicode-mode matching remain
outside this proof.


Zero-width sequence repetitions compose through the existing flat wrapper proof
(22.2.2.3.1, 22.2.2.8). A counted child sequence is zero-width when its exact count
is zero, or when every term is an ordinary boundary predicate and there are no
capture reads. An empty term inventory also proves zero width. Repeated character,
set and assertion terms are inert during ordinary traversal; their sequence owner
determines the complete matching effect. Any other nonzero consuming/reference
sequence rejects this proof. Child wrapper summaries retain the existing linear
preparation without rescanning nested bodies or expanding counts.

Required parent wrappers retain their own empty capture ranges even when an
optional child leaves its slots undefined. Exact-zero children skip all terms and
their capture effects; required predicates preserve full input context. Huge
required bounds execute the same final zero-width effects once. Existing branch
checkpoints, named aliases, surrounding assertions and parent consuming loops
retain their semantics. Deep wrapper traversal, copies and dropping use no native
recursion. Default work and heap quotas remain unlimited. Nonzero consuming or
unproved reference children, counted wrappers inside lookbehind and Unicode-mode
matching remain outside this proof.


Nested zero-width sequence repetitions also accept proven empty references
(22.2.2.3.1, 22.2.2.9.2). Open/forward same-body captures are already lowered to an
empty target. Completed local spans must have ordered endpoints and end no later
than the current reference ordinal. Every preceding term has already been proved
zero-width, so a local span can contain earlier references or assertions and still
be empty. Outside capture reads and consuming terms reject this proof, except for
the existing exact-zero count case. The proof visits terms once and needs no new
state, allocation or runtime instruction.

The existing sequence executor preserves required empty captures, skipped optional
slots, partial ranges and named aliases. Wrapper checkpoints retain negative and
failed-arm restoration. References do not expand source or depend on the count;
oversized required bounds keep their mathematical meaning. Deep captures and
wrapper nesting remain flat under copying, execution and dropping. Default work
and heap quotas stay unlimited. Unproved outside/local dependencies, counted
wrappers inside lookbehind and Unicode-mode matching remain pending.


Fixed ordinary lookbehind also accepts nested zero-width repetition wrappers
(22.2.2.3.1, 22.2.2.8). Its backward width proof records each repetition root,
validates the actual body once and requires every complete branch to preserve the
suffix width. Nested fixed lookbehind remains summarized. Lookahead, internal
references, mixed empty/consuming alternatives and unproved variable consuming
counts reject this proof. The scan uses a flat map of pending roots and does not
expand bounds or revisit completed child regions.

Required wrappers execute one zero-width body through explicit fixed-body frames;
their predicates and final capture effects are identical at every iteration.
Optional wrappers clear their owned capture slots and skip the body, following
RepeatMatcher's empty-progress check. Each required frame records its checkpoint
before resetting/opening captures, scopes its branch choices and restores failed
effects before resuming its parent. Selecting its first successful body is valid
in this subset because preparation excludes capture-dependent continuations
inside lookbehind. The enclosing assertion remains atomic for outside references.
Nested lookbehind can export pre-match ranges while wrapper captures stay at the
local cursor. Huge required counts, deep captures, nested wrappers, clones and
dropping use flat storage; default host quotas remain unlimited.


Repeated ordinary lookbehind accepts proven empty same-body references
(22.2.2.3.1, 22.2.2.8, 22.2.2.9.2). The fixed-width proof permits lowered empty
targets and completed local spans with ordered endpoints ending no later than
their reference ordinal, but only when the entire repeated unit has zero width.
Every same-body capture is then empty in either matching direction. Outside reads
and any combination of a reference with consuming terms reject this proof; the
forward lowering of open/forward references is insufficient for nonempty
backward units.

Character/assertion terms retain their existing comparison. Proved empty/local
targets need no capture read and add no input movement. Their term offsets are
zero and preserve empty ranges at the actual lookbehind cursor, including
captures around earlier references and boundaries. Required zero-width counts
execute one pass; optional counts skip effects with undefined slots. Existing
flat wrapper/assertion frames preserve nested ranges, source-order choices and
completed negative restoration. No new instruction or runtime allocation is
needed, source and huge bounds are not expanded, and default host quotas remain
unlimited. Unicode modes, consuming reference units, outside reads and lookahead
inside lookbehind remain pending.


Exact-zero native group repetitions use one shared skip instruction
(22.2.2.3.1). After validating the full Pattern and lowering each native child,
preparation handles an exact minimum and maximum of zero before requiring body
progress, zero width or reference-target proofs. This admits mixed empty/consuming
choices and lookahead/reference combinations whose body will never execute.
Unlowered child bodies still report Unsupported. Capture and name inventories
retain every original slot, including duplicate names on exclusive branches.

Forward matching and fixed ordinary lookbehind clear only the skipped owner's
capture range and jump to its end. They never open a skipped capture, enter a
branch, read a reference or compare input. Outside capture values survive, while
required enclosing zero-width wrappers retain their own empty ranges. Parent
progress and zero-width proofs summarize each completed skipped region once;
fixed lookbehind similarly assigns it zero width. Existing checkpoints preserve
branch retries, completed enclosing captures and negative assertion rollback.

A final reverse pass discards unused body instructions before allocating choice
frames or compiling character sets. It visits outer skipped owners before their
descendants and bypasses the cleared region, so disjoint and nested bodies cost
one pass without changing instruction indices. Preparation, matching, copying and
dropping remain flat under deep nesting, with no count expansion or default host
quota. Exact-zero standalone reference atoms retain their existing target-free
width proof. Nonzero mixed choices, unproved native children and Unicode modes
remain pending.


Fixed lookahead can run inside ordinary lookbehind (22.2.2.8). Preparation records
a forward width when the existing fixed-body proof accepts every complete branch,
including captures. It summarizes an already proved child as zero parent width,
checking the original root and owner/head/end positions and skipping the complete
region once. The proof excludes capture-dependent continuations and unproved
variable-width bodies. Generic forward lookahead retains its existing machinery.

The fixed executor starts a lookahead child at the current cursor and checks its
complete forward endpoint against the actual input length. Its head retains the
original source-order choices; its body uses full input/line and word-boundary
context. An unavailable endpoint fails a positive assertion and succeeds a
negative assertion. Explicit frames retain the parent cursor, checkpoint and
choice base, commit the first successful positive body and restore negative or
failed effects before resuming the parent. Captures may precede or extend beyond
the enclosing match; named groups and indices retain their original slots.

Each fixed frame selects the repetition capture iteration required by its
assertion direction (22.2.2.3.1). Lookbehind keeps the first, leftmost iteration;
lookahead keeps the last, rightmost iteration. A completed forward consuming count
uses the final cursor minus the proved unit width as its capture base. Required
zero-width wrappers inherit that choice, while a nested assertion starts its own
direction. Empty counts leave owned slots undefined. Checked endpoints and proved
unit offsets preserve exact partial ranges without expanding source or counts.

Nested lookahead/lookbehind, completed negative rollback, deep captures, copying,
collection and dropping share flat traversal without native recursion. Default
work and heap quotas stay unlimited. Capture-dependent child continuations,
unproved variable-width bodies, other unsupported native features and Unicode
modes remain pending.


Linear ordinary lookbehind can read stable outside numeric and named capture
references (22.2.2.8, 22.2.2.9.2). Preparation first attempts the existing fixed
proof, then a separate linear plan containing only ordinary character/set and
boundary operations, unrepeated captures and outside references. A numeric target
must lie outside the complete owner's capture declaration range; every slot of
a named alias group must also lie outside it. Original branch/name inventories
and final reference bounds validation remain intact.

The plan retains a constant width and a flat list of outside targets. At assertion
entry it adds their current UTF-16 range lengths with checked arithmetic; undefined
or empty targets contribute zero. Checked subtraction finds the complete available
prefix. No target can change while the owner executes, so source-order comparison
of that prefix is equivalent to backward reference comparison. The existing
canonicalizer, input context and capture checkpoints preserve ignoreCase,
boundaries, owned pre-match ranges, negative restoration and imported values.
Open or forward targets outside the assertion read their current undefined value.

Already proved fixed lookahead/lookbehind and exact-zero skipped scopes can be
summarized without rescanning their body. Fixed backward parents and fixed
lookahead reject children with dynamic reference widths, preserving their
capture-read-free ordering proof. The linear fallback also rejects such children,
internal reads, choices, unproved variable counts and unproved internal-reference
units. Generic forward execution
retains its existing child assertion semantics.

The width list and execution frames are flat, do not expand input or source and
need no per-match target allocation. Checked unavailable or overflowing widths
fail positive assertions and succeed for negative assertions. Deep owned slots,
clones and completed negative rollback use no native recursion. Default host
quotas remain unlimited; other reference-dependent backward plans and Unicode
modes remain pending.


Exact-count repetitions of a single stable outside reference also use this linear
lookbehind plan (22.2.2.3.1, 22.2.2.8, 22.2.2.9.2). The existing flat reference
wrapper retains its source-order copies and owned capture spans. Preparation
requires equal representable minimum/maximum counts and checks the target against
the complete owner's numeric or named capture inventory. A zero count skips all
target reads. Every nonzero count contributes target length times source copies
times count to the checked complete prefix width; greedy and lazy exact counts
have the same deterministic behavior.

Comparison uses the unchanged outside ranges and existing Canonicalize operation.
Consuming units run only within their proved available prefix. Required empty
iterations have identical effects and execute one pass, including counts up to
usize::MAX; skipped units leave their owned slots undefined. Successful backward
repetitions export the first, leftmost unit's capture spans. Completed or partial
negative/failed bodies use the existing checkpoint rollback and leave imported
values intact. Overflowing nonempty widths cannot match finite input, so positive
assertions fail and negative assertions succeed without arithmetic overflow.

This extends the width inventory without source-count expansion, per-match target
allocation, native recursion, dependencies or default quotas. Fixed backward
parents and fixed lookahead still reject dynamic reference-width children. Other
dependent backward plans, unproved variable counts, unproved internal-reference units
and Unicode modes remain pending.


Counted fixed character, class and boundary units compose with stable outside
references in linear ordinary lookbehind (22.2.2.3.1, 22.2.2.8). Both fixed and
outside-reference plans share the same unit-width proof. Consuming units require
equal representable counts and add checked constant widths to the complete prefix.
Wholly empty units retain the existing required/optional effects; same-body local
references whose completed spans lie to the left of the read are also
admissible in exact-count consuming backward units. Forward consuming local
reads retain their existing forward plan.

Counted units own their repeated character/set/assertion instruction fields;
ordinary body traversal skips those fields after accounting for their control.
Zero-width terms retain source-relative prefix offsets for exact capture ranges.
Only an accepted complete owner receives these offsets, and child assertion
regions keep their own metadata. The existing matcher executes every boundary in
full input context, retains the first backward iteration and restores negative or
failed capture effects. No matcher instruction or execution path changes.

The proof stays flat and does not expand counts, copy input, add dependencies or
impose default quotas. Stable imported targets remain outside the complete owner.
Consuming units with unproved internal reads,
unproved variable counts, choices and unproved dynamic child assertions remain unsupported.


Exact-count linear lookbehind units can mix ordinary characters, classes and
boundary predicates with stable outside numeric or named references of different
lengths (22.2.2.3.1, 22.2.2.8, 22.2.2.9.2). The existing fixed-unit proof runs
first. A mixed unit then requires equal representable bounds or an unrepresentable
required minimum, and contains only
ordinary repeated terms or input targets outside the complete owner's capture
declaration range; every slot of a named alias must be outside. Forward-lowered
empty/local targets remain excluded from consuming mixed units. Internal reads,
unproved variable counts, choices and unproved dynamic child assertions remain pending.

Entry width adds checked constant term widths times count and a flat outside
width term for each reference occurrence. Different target lengths, source-order
copies, future ranges imported from prior lookahead and undefined open/forward
targets retain their existing values. A proved available complete prefix bounds
every consuming iteration; overflowing or insufficient widths fail positives
and succeed for negatives without running an unavailable body.

The existing forward sequence range/offset resolution is shared as a pure helper,
with identical charge, allocation and forward matching behavior. Each mixed unit
resolves its unchanged outside ranges and source-relative capture offsets once,
then compares exact iterations through the existing Canonicalize and full-context
boundary operations. Required empty units execute one pass without count
expansion. Success exports the first backward unit's offsets; partial/completed
negative failures restore checkpoints and leave imported ranges intact.

The plan stays flat and uses no native recursion, new dependency or default
quota. Unit-sized range/offset vectors are independent of the repetition count.
Parent assertion widths skip proved stable imported child assertions as zero-width
terms; each child resolves its checked complete width at entry.
Unicode modes and other capture-dependent backward plans remain unsupported.


Outside-reference lookbehind width plans preserve unrepresentable required
minimums rather than rejecting their validated decimal syntax (22.2.2.3.1,
22.2.2.8, 22.2.2.9.2). A missing native count denotes a mathematical minimum
larger than usize::MAX. Such bounds can be exact, bounded or unbounded, greedy
or lazy; validation still rejects a maximum below the minimum.

The constant part of the complete width can likewise be unavailable after
checked addition/multiplication. Resolve every outside range at entry. An empty
range contributes zero even when its required count is unrepresentable; a
nonempty range or consuming constant term makes that prefix unavailable. An
unavailable positive assertion fails and a negative assertion succeeds without
executing the body or exporting owned captures. All body operations and outside
target ownership remain checked during preparation, even after width overflow.

A wholly empty imported unit executes one required pass because its stable
reads, boundaries and capture writes have identical effects on every required
iteration. Optional empty iterations do not advance or change those final
required effects. Scalar and mixed sequence execution share the existing exact
capture offsets and scoped rollback. Nonempty consuming execution remains bounded
by an available checked prefix and uses only representable counts. Ordinary
variable counts with representable minimums, internal consuming reads, dynamic
child assertions and Unicode modes remain pending. No count expansion, default
quota, new dependency or native recursion is introduced.


Exact-count fixed lookbehind units can contain references to completed same-unit
capture spans lying to the left of the read (22.2.2.3.1, 22.2.2.8, 22.2.2.9.2).
Forward preparation retains those targets as Local spans with end <= the read's
term ordinal. Backward execution visits the read before that target has matched,
and RepeatMatcher clears the unit's owned captures for each iteration. Thus the
read is undefined and consumes zero, including after an earlier iteration.

The shared fixed-unit proof now records these zero-width terms among ordinary
characters, sets and boundaries, requires equal representable counts for consuming
units, and stores checked source-relative prefix offsets. Existing fixed sequence
execution compares only consuming terms and predicates, and exports the first
backward unit's exact ranges. Bare outside references can accompany proved units
through the unchanged complete-owner width inventory. Negative/failed assertions
retain existing checkpoint rollback and imported values.

Width preparation receives the assertion's matching direction. Forward fixed
units retain the checked consumed width of completed Local spans, while backward
units read those still-unmatched targets as undefined. Nested assertions keep
their own matching direction. Forward-lowered
Empty targets represent empty right-hand spans; nonempty right-hand targets
retain their spans for the fixed backward width proof. Enclosing Open targets are tracked separately and retain their
undefined value in either matching direction. Unproved variable counts, choices,
unproved capture-dependent child assertions and Unicode modes remain pending. This extends
preparation; matching operations, quotas, dependencies and native-stack behavior
are unchanged.


Reference-unit preparation now distinguishes a target enclosing its own read from
a target lying wholly to the right (22.2.2.3.1, 22.2.2.8, 22.2.2.9.2). A retained
capture span with start <= read ordinal < end is Open. Its group has not closed
in either matching direction, so it reads undefined even among consuming terms.
The existing forward range resolver treats Open exactly as its former Empty
classification; forward matching, charges and capture ranges are unchanged.

The shared fixed-unit proof admits Open as zero-width in exact consuming units
alongside ordinary characters, classes, boundaries and proved backward Local
reads. Checked prefix offsets preserve nested empty/enclosing ranges. Existing
fixed matching executes the same required terms and exports the first backward
or rightmost forward iteration. Fixed lookahead units can therefore use the Open
proof independently of their enclosing backward assertion's direction. Negative
failures retain scoped rollback and imported ranges.

Right-hand targets retain their spans for fixed backward matching;
unproved variable counts,
choices and Unicode modes remain pending. Exact-zero bodies keep their existing
skip path. No new matching algorithm, count expansion, default quota, dependency
or native recursion is introduced.


Right-hand same-unit references retain a Future capture span during preparation
(22.2.2.3.1, 22.2.2.8, 22.2.2.9.2). Backward matching has already completed that
target, whereas forward matching has not reached it. The ordinary forward range
resolver preserves its former undefined behavior, charges and capture effects.
Known empty future spans remain Empty and consume zero in either direction.

The shared fixed-unit proof scans terms from right to left. A Future reference's
width is the checked suffix difference for its target span. Dependencies always
point to later terms, so a flat suffix inventory also handles nested references
without recursion or source expansion. Conversion to source-order prefix offsets
retains exact capture positions. Consuming units still require equal representable
counts; wholly empty units retain their required and optional effects.

Fixed backward execution compares each Future read with its proved right-hand
input range in the current iteration. Existing character/class/assertion checks
validate that range and its dependencies before the assertion succeeds. Captures
retain the leftmost backward iteration; fixed forward assertions skip still
undefined Future reads and retain their rightmost capture effects. Negative and
failed assertions use the existing scoped checkpoint rollback.

Outside bare references can accompany these proved fixed units. Unproved variable consuming counts, capture-dependent
child assertions, unproved choices and Unicode modes remain pending. Checked
width overflow does not establish a fixed proof. There is no default quota,
dependency addition, repeated-source expansion or native recursion.


Mixed exact-count ordinary lookbehind units can combine stable outside Input
captures with proved empty same-unit Local, Open and Empty reads (22.2.2.3.1,
22.2.2.8, 22.2.2.9.2). Local spans ending before their read are unmatched after
RepeatMatcher clears owned captures in backward matching. Open spans remain
unclosed, and Empty represents a known-empty right-hand span. These reads add
zero to the complete owner's checked width inventory.

The existing sequence resolver receives the matching direction. Ordinary forward
resolution keeps its previous local-range calculation, charges and future/open
undefined behavior. Backward mixed resolution treats proved Local reads as empty
and retains imported outside ranges and source-order capture offsets. The complete
owner proof still excludes every outside numeric or named alias that belongs to
its owned scope, so imported values stay immutable while all units are compared.
Existing matching exports the first backward iteration and restores owned slots
after completed negative failures without overwriting imported captures.

Consuming mixed units still require exact representable counts. Unrepresentable
required minima retain one pass only for wholly empty resolved units; nonempty
widths cannot fit a finite input. Unproved variable counts, choices, dependent child assertions and Unicode modes
remain pending. This shares
the existing resolver and matcher with no source expansion, recursion, dependency
addition or default resource quota.


Exact ordinary lookbehind units may contain both stable outside Input references
and same-unit Future dependencies (22.2.2.3.1, 22.2.2.8, 22.2.2.9.2). Preparation
retains one flat instruction/count descriptor per dynamic unit after validating
every outside target and named alias against the complete assertion owner. Terms
retain their character/class/boundary proof, Local reads remain empty backward,
and Future spans must lie strictly to the right inside the same unit.

At assertion entry, backward resolution scans the unit from right to left. It
reads immutable outside lengths, uses checked suffix differences for dependent
right-hand capture spans, then translates local ranges and capture offsets into
source order. Dependencies are acyclic in that direction. The same resolver feeds
the existing unit comparator; captured right-hand text is checked against actual
current-unit input ranges, with every ordinary predicate still evaluated. Forward
resolution preserves its previous ranges, charges and undefined future reads.

The width inventory scales each resolved unit once by its required count. Checked
overflow denotes an unavailable consuming prefix; no unit expansion or dependency
coefficient expansion is necessary. This also represents mathematical overflow in
otherwise proved fixed character sequences. All-zero dependencies remain zero
even through long chains and unrepresentable required counts, retaining one
required effect pass. Exact-zero bodies retain their skip path. Repeated captures
keep their first backward unit; completed failures restore owned captures while
preserving imports.

Variable consuming counts, unproved choices, unproved capture-dependent child assertions
and Unicode modes remain pending. Resolution uses flat vectors and charges their
allocation and traversal. There is no default quota, new dependency, native
recursion or repeated-source expansion.


Fixed forward lookahead units can reference completed same-unit Local spans
(22.2.2.3.1, 22.2.2.8, 22.2.2.9.2). The shared fixed-unit proof scans in the
assertion's matching direction. Forward matching builds prefix widths and reads
Local spans already completed to the left; backward matching builds suffix widths
and reads Future spans already completed to the right. Dependencies always point
to previously proved terms in that direction. The opposite direction remains
undefined, as do enclosing Open spans and known-empty targets.

Checked differences retain nested reference widths; source-order offsets identify
actual current-unit capture ranges. Forward fixed execution compares each Local
read with its completed left-hand range, while backward Local reads still consume
zero. Existing predicates validate the terms, repetitions export their rightmost
forward or leftmost backward unit, and enclosing capture spans retain their exact
positions. A failed negative assertion restores completed owned effects through
the existing checkpoint.

Consuming units require equal representable counts and checked complete widths.
All-zero required units retain one effect pass, even with unrepresentable counts;
exact-zero units remain skipped. Changing enclosing-owner capture dependencies, variable consuming counts,
unproved choices and Unicode modes remain pending. This extends the shared proof and comparator
without new dependencies, default quotas, source expansion or native recursion.


Ordinary nullable repeated bodies use RepeatMatcher's optional-progress check
(22.2.2.3.1). A prepared native loop retains whether every complete body path
consumes input. Proven consuming loops keep arbitrary required counts and the
existing remaining-input check. Nullable native loops accept zero/one minimums;
they may execute one required empty iteration, while every later accepted
iteration must consume input. Existing fixed and pure-zero plans retain their
previous count behavior. Larger required nullable native counts still need a
separate effect/progress proof; variable backward bodies and Unicode matching
remain pending.

The end instruction rejects an empty optional attempt before incrementing its
count. Existing flat frames can retry the body's remaining branches or inner
counts and restore the attempt's capture starts, completed ranges, named aliases
and parent iteration record. Greedy/lazy count decisions retain source order;
failed empty attempts preserve the previous accepted iteration's effects. A
required initial empty iteration exports its own ranges, and existing positive
and negative assertion contexts retain their atomic commitment and rollback.

Parent width proofs summarize prepared child loops once. A required child proves
progress only when its body is itself proved consuming; nullable or optional
children use the continuation's proof. This avoids assuming a nullable required
child consumes input and keeps preparation and nesting flat. Count increments
remain bounded by the entry's checked maximum and actual accepted consumption.
The change uses the existing checkpoint mechanism, with no new dependencies,
default resource quotas, source expansion or native recursion.


Linear forward lookahead inside ordinary lookbehind can import stable capture
ranges preceding every enclosing backward owner (22.2.2.8, 22.2.2.9.2). Group
frames cache the earliest backward owner's first capture slot, inherited at entry,
so nested forward assertion preparation never rescans its ancestors. Each numeric
target and every alias of a named target must precede this bound. Open outside
captures remain undefined; mutable enclosing-owner and future captures stay
outside this proof.

The shared checked assertion-width resolver receives matching direction. Scalar
imports, exact imported counts and counted mixed units resolve their lengths from
original input ranges. Forward units resolve completed left-hand Local spans;
backward units retain their right-hand Future proof. An empty required unit needs
one capture-effect pass even for unrepresentable counts. A consuming unavailable
or overflowing width makes a positive assertion fail and a negative assertion
succeed without exporting owned captures. Exact-zero units do not read targets.

Forward frames retain full input context and require the computed end position.
Imported captures preserve their original positions; owned captures retain the
rightmost forward unit's ranges. Parent backward widths skip these zero-width
children, and failed negative assertions restore completed effects through their
existing checkpoints. Flat instructions, frames and shared units keep deep scopes
and large counts safe without new dependencies, source expansion, native recursion
or default quotas. Variable consuming counts, unproved choices, changing owner
captures and Unicode modes remain pending.


Nested linear ordinary lookbehind can resolve stable imported capture widths
inside fixed or imported-width backward parents and forward assertions
(22.2.2.8, 22.2.2.9.2). A nested backward child's imports must precede the earliest
enclosing backward owner's first capture slot, including every named alias.
Cached group-frame bounds retain that proof without rescanning ancestor scopes.
Root backward assertions preserve their existing complete-own-scope check and
undefined outside open/forward reads.

A proved child assertion consumes no parent input, even when its required prefix
width depends on imported ranges. Parent width preparation skips the complete
child region. The existing assertion frame resolves the child's complete width
at its actual input position, then executes its own backward terms and unit
reference dependencies. Parent and child have independent checked positions;
forward lookahead inside either retains full original input context.

Stable imports retain their original ranges. Repeated backward children retain
their leftmost unit captures, including right-hand Future dependencies; forward
children keep their rightmost unit effects. Exact-zero units skip reads, and
required all-empty units retain one effect pass even for unrepresentable counts.
An unavailable consuming prefix fails a positive child and succeeds for a negative
child without exporting owned captures. Partial and completed failures retain the
existing capture/alias checkpoints. This reuses flat width proofs and frames with
no new dependency, native recursion, source expansion or default quota. Changing
owner targets, variable consuming counts, unproved choices and Unicode modes stay
outside the proof.


Linear ordinary assertion bodies containing bare capture references reuse an
exact one-iteration reference-unit plan (22.2.2.8, 22.2.2.9.2). An assertion body
executes once per attempt; the existing unit preparation can therefore retain its
ordinary terms and original Local, Open and Future capture spans without adding
a matcher or expanding source. Assertion preparation requires an actual bare
reference so it preserves the control entry. Failed preparation changes no
instruction. Already lowered native child regions skip this scan through the
existing last-complex-child bound, keeping deep nested assertion preparation
linear in its prepared instruction count.

The shared matching-direction proof computes forward Local or backward Future
widths. Open and opposite-direction reads remain undefined. Stable imported Input
ranges keep the complete-owner checks, including every named alias. Checked
prefix/suffix differences retain dependent widths and original capture offsets;
an overflowing proved consuming width is unavailable for every finite input.
Every ordinary term and referenced input range is still compared before a result
is exported. Forward and backward frames retain their own positions and existing
positive commitment and negative rollback.

One required unit clears its owned captures, compares through the existing shared
reference comparator and writes exact source-relative ranges. Empty open targets
retain empty ranges, and imports retain their original positions. Original source,
names and shared indices pairs remain unchanged. This reuses the existing unit
plan, width resolver and scoped checkpoints, with no new dependency, native
recursion, repeated source or default quota. Variable consuming counts, unproved
choices, changing enclosing-owner reads and Unicode modes remain pending.


Counted same-body references in linear ordinary assertions can reuse the required
one-unit plan when every repetition of the reference atom is proved empty
(22.2.2.3.1, 22.2.2.8, 22.2.2.9.2). The scalar counted atom must own no capture
effects. Its declared target is classified from the original same-body span:
Open and already-empty reads are empty in either direction; a closed Local target
is still undefined when matching backward, and a Future target is still undefined
when matching forward. A zero maximum also proves a declared owned target empty.
Outside Input reads remain visible and unproved; preparation never hides invalid
references or assumes an imported capture is empty.

Every finite required repetition of this effect-free empty atom preserves the
same MatchState, including a minimum with too many digits for usize. Optional
empty iterations stop immediately. Preparation replaces only these proved atoms
with empty unit terms, keeping surrounding captures and source spans. The proof
finishes before changing instructions; failed preparation leaves them unchanged.
Owned capture clearing, original range writes, names and shared index pairs,
forward/backward frame positions and negative rollback keep the existing behavior.
Capturing counted children, consuming unproved counts, choices, changing owner
reads and Unicode modes remain pending. There is no repetition expansion, new
matcher, dependency, recursion or default quota.


An effect-free scalar reference counted exactly once in an ordinary assertion is
also the bare read (22.2.2.3.1, 22.2.2.8). Preparation requires min = max = 1,
one reference copy and no owned capture effects. It retains the original declared
Input, Local or Future target instead of requiring an empty proof. Existing
matching-direction and complete-owner checks determine its consuming width;
original named aliases and validation remain visible. Failed proof remains
unsupported without changing the assertion's behavior.

The existing sequence comparator and checkpoints perform the read once. When a
pure imported sequence collapses to counted copies, every original scalar
instruction is cleared, avoiding leftover duplicate reads. Original capture spans,
required-one-unit clearing, indices and shared named pairs, frame positions and
negative rollback remain unchanged. This adds no matcher, dependency, native
recursion, source expansion or default quota. Larger unproved consuming counts,
capturing counted children, choices, changing-owner imports and Unicode modes
remain pending.


Effect-free reference atoms with a representable exact positive count can retain
one flat Input, Local or Future target plus a scalar copy count (22.2.2.3.1,
22.2.2.8, 22.2.2.9.2). Multiplying the exact atom count by its original copies is
checked before lowering. Direction-specific undefined/empty proofs retain their
existing behavior; unproved variable counts, unrepresentable copy products,
unproved outside-zero reads and capturing counted children remain unsupported.
Declared reference validation and complete-owner checks include the counted target.

Forward prefix and backward suffix resolution multiply each nonempty referenced
span width by its copies with checked arithmetic, retaining original source-order
capture offsets. The existing comparator reads that original range for each copy;
it never allocates or duplicates ranges by the count. Full unit width limits the
work to available input before comparisons. Required empty ranges use one constant
pass. A proved width overflow is unavailable for finite input: positive assertions
fail and negative assertions retain their original rollback and undefined captures.

Only all-one imported targets use the old scalar collapse, preserving its ordinal
capture spans. Other multiplicities retain flat sequence offsets and original range
writes, named aliases and shared indices pairs. No matcher, dependency, recursion,
source expansion or default quota is added. Choices, changing-owner reads and
Unicode modes retain their existing conservative boundaries.


Nested ordinary units can retain effect-free exact reference multiplicities through
noncapturing group lowering (22.2.2.3.1, 22.2.2.8, 22.2.2.9.2). The existing flat
Input/Local/Future sequence targets and checked prefix/suffix width resolution are
shared by consuming repetitions and assertions. Exact counts preserve declared
reads and original capture offsets without count-sized expansion or allocation.

When a group's direction is not established by an assertion, only proofs valid in
both directions may erase a counted read: Empty/Open targets, a lexically empty
owned capture span or a zero maximum for that owner. Other targets retain their
exact multiplicity and complete-owner validation. Variable consuming counts and
unproved owner dependencies remain unsupported. Required empty units preserve
capture effects once, while optional empty units retain their existing stop and
rollback rules. No matcher, dependency, recursion or default quota is added.


Group compilation caches the actual matching direction separately from enclosing
complete-owner limits (22.2.2.8). Ordinary children inherit direction; lookbehind
sets backward and lookahead resets forward, including inside another lookbehind.
The flat group stack retains this bit without walking ancestors or adding runtime
state. Assertion-unit preparation separately requires a real reference, so pure
ordinary groups retain their original lowering and required empty effects.

Effect-free counted references to owned captures that have not participated in
this direction are proved undefined and therefore empty for every count. Backward
units may erase Local reads, forward units Future reads; Open/empty and owned-zero
proofs remain valid in either direction. Exact consuming multiplicities and checked
widths retain their previous behavior. Outside reads stay visible to complete-owner
and declared-reference validation. Variable consuming widths remain unsupported.
There is no count-sized expansion, new matcher, dependency, recursion or default
quota. Existing capture rollback, negative assertions and result offsets are reused.


A single-reference captured atom counted exactly once can compose into an ordinary
reference unit without dropping its capture effects (22.2.2.3.1, 22.2.2.9.2). Its
closed full or empty spans map to one source-relative term with checked ordinal
addition. The existing Input/Local/Open/Future classification, actual matching
direction, complete-owner proof and checked widths determine the original ranges.
Capture aliases, negative rollback and the surrounding unit's repetition are reused.
Other child counts and partial spans across multiple reference copies remain unproved.

After importing nested capture effects, lowering retains a flat sequence instead
of collapsing back to the scalar instruction. A further wrapper cannot flatten
that sequence again, preventing repeated copying of growing child-effect lists.
The list is proportional to the source's declared capture slots, never to a repeat
count. No runtime matcher, dependency, source expansion, recursion or default quota
is added. The child's declared read and capture slots stay visible to validation.


A required single-reference child may retain its full/empty capture effects for
counts beyond one when the owned read is proved empty (22.2.2.3.1, 22.2.2.9.2).
Open captures, lexically empty owners and not-yet-participating captures in the
actual direction give that proof. Each required iteration writes identical ranges
at the same input position, so one pass preserves the final capture state, including
huge bounds. Greedy/lazy bounds do not change those ranges. The existing flat
source-relative capture spans, named writes and rollback remain unchanged.

An explicit requires-empty condition rejects every non-count-one captured child
whose read could consume input before lowering or instruction mutation. Optional
captured children, outside reads and partial spans across multiple copies remain
unproved. The existing sequence boundary prevents repeated flattening of child
capture lists. There is no count-sized allocation, expansion, new matcher,
dependency, recursion or default quota. Required outer repetitions reuse their
existing empty-state effects and optional stop behavior.


Required scalar atoms with multiple original copies may retain capture effects
when their common read is proved empty (22.2.2.3.1, 22.2.2.9.2). Every full,
partial and empty source-copy span then denotes the same input point, including
huge positive counts. The compiler checks each original span against the original
copy count and maps it to one source-relative point. Reads of child-owned numeric
slots or named aliases are excluded for multiple copies, preventing the provisional
point mapping from proving its own input empty.

A requires-empty condition includes every multi-copy captured child and is checked
before lowering. Single-copy count-one children retain their earlier consuming
behavior. Other consuming multi-copy spans, optional captured children and outside
reads remain unproved. Capture writes, names, indices, rollback and the sequence
boundary are reused. The effect list is proportional to declared source captures,
never to a count; no matcher, dependency, recursion or default quota is added.


A captured atom consisting of multiple copies of the same reference can compose
with an exact count of one when every capture endpoint is a boundary of the whole
atom (22.2.2.3.1, 22.2.2.9.2). Checked ordinals map those endpoints onto one scaled
term. The existing checked copy multiplicity and width determine the original full
span or empty boundary; the actual matching direction and owner proofs are reused.
Reads of child-owned slots and aliases remain excluded from this multiple-copy
scalar proof. Interior capture boundaries still require the established empty-read
proof. Other consuming child counts and optional child effects remain unproved.

The existing flat sequence boundary retains nested capture effects without
reflattening growing lists. This adds neither an instruction nor a runtime matcher,
source expansion, recursion, dependency or default quota.


A same-reference child counted exactly once may retain capture endpoints inside
its original atom (22.2.2.3.1, 22.2.2.9.2). The compiler collects and deduplicates
declared endpoints, splitting only at those boundaries. Each segment carries a
checked scalar copy multiplicity; existing complete-owner, matching-direction and
width proofs determine its input range. Child capture ordinals then index the
existing segment offsets, preserving partial spans and interior-empty points.

Reads of child-owned numeric slots or named aliases remain excluded from this
multiple-copy proof. Other consuming child counts and optional effects stay
unproved. Empty reads retain their source-relative point semantics. The existing
sequence boundary prevents reflattening nested effect lists. Storage follows
declared boundaries rather than the repetition count; this adds no instruction,
runtime matcher, recursive traversal, dependency or default quota.


Exactly repeated same-reference children preserve captures from their final
mandatory iteration (22.2.2.3.1, 22.2.2.9.2). For a representable positive exact
count, checked multiplication determines the total copy width. Declared child
endpoints shift to the last iteration: its suffix in forward matching, or its
physical leftmost span in backward matching. Deduplicated endpoints split the
total atom into flat segments with checked multiplicities. Existing owner, width,
direction, capture and rollback machinery interprets those segments.

The consuming proof excludes reads of child-owned numeric slots and named aliases.
Unrepresentable products and bounds retain the existing required-empty proof,
including undefined reads in both directions; they do not acquire a consuming
plan. Variable and optional consuming child counts remain unproved. Storage
follows declared capture boundaries rather than repetition counts. This adds no
instruction, runtime matcher, recursive traversal, dependency or default quota.


Unicode literal matching (22.2.2) admits a nonempty case-sensitive `u` literal
whose compiled units all lie outside the surrogate range. Each consumed character
therefore occupies exactly one UTF-16 unit. An occurrence cannot begin or end
inside a surrogate pair, even when the input contains pairs or lone surrogates.
Existing immutable literal programs, KMP searches and flat capture ranges preserve
Unicode matching positions without materializing code-point arrays. Validated
named-capture normalization retains original source and complete capture metadata.
Empty Unicode literals, ignore-case Unicode matching, `v`, surrogate literals,
nonliteral syntax remains outside this proof. Construction and
execution preserve opted-in work accounting; default resource quotas stay absent.


Mandatory empty literal bodies and capture groups execute in both `u` and `v`
modes, including ignore-case flags, since no character is consumed or canonicalized.
They reuse immutable literal programs and flat zero-length capture ranges. Before
search, an offset inside a surrogate pair moves to its leading code-unit boundary;
lone surrogates and end-of-input retain their boundaries. Existing consumer
AdvanceStringIndex logic advances by code point after an empty Unicode match.
This adds no code-point arrays, recursion, dependency or default resource quota.

The boundary rule resolves a specific inconsistency in published ECMAScript 2026
RegExpBuiltinExec (22.2.7.2). With `/(?:)/uy`, input `"😀"` and `lastIndex = 1`,
the matcher starts at the complete Unicode character. GetStringIndex maps its
empty-match end to 0, but the published algorithm retains 1 as the whole-match
start and result `index`. This violates GetMatchString's requirement that start
not exceed end. The approved resolution follows Node/V8: both whole-match bounds,
result `index`, participating captures and updated `lastIndex` are 0.

The same approved leading-start rule applies to consuming Unicode matches.
With `/(\u{1F600})/duy`, input `"😀"` and `lastIndex = 1`, the published algorithm
retains whole-match start 1 and end 2, returning the trailing surrogate as `m[0]`,
while the participating capture maps to the complete character at 0..2. Following
Node/V8, the whole match and capture both contain the emoji, `index` is 0, both
indices are 0..2 and `lastIndex` is 2. This also applies in `v` mode and searching
matches. The exception is limited to an initial UTF-16 offset inside a surrogate
pair and grants no broader specification departure. Nonempty BMP literals cannot
match inside a pair and retain their separate proof.

The discrepancy was verified against the official
[`es2026` source tag](https://github.com/tc39/ecma262/blob/f7db29f16c5175a93f0d6e8fb27a8e3cb9b97a9e/spec.html).
Construction still validates the complete Pattern and preserves original source
and named-capture metadata. Nonliteral empty bodies, Unicode backreferences,
nonempty ignore-case bodies and other unproved Unicode syntax remain Unsupported.


Validated UnicodeSets (`v`) Patterns now use the same nonempty case-sensitive
nonsurrogate BMP literal proof as `u` Patterns (22.2.2, 22.2.7.2). UnicodeSets
changes class syntax and class-set semantics; admitted literal characters and
mandatory capture groups consume the same Unicode code points. Full Pattern
validation precedes the unchanged flat literal compiler. Every consumed code
point has one UTF-16 unit, so successful starts, ends and captures cannot split a
surrogate pair. No Unicode class operations, string properties, case folding,
surrogate literals acquire a matching plan. Empty Unicode bodies
retain their separate approved boundary resolution. This adds no core matching
operation, allocation strategy, dependency or default quota.


Unicode literal compilation now decodes `\u{CodePoint}` escapes whose values
are nonsurrogate BMP characters (22.2.1, 22.2.2.2). Full `u`/`v` validation still
precedes compilation. Checked hexadecimal accumulation rejects values outside
`u16`, malformed escapes and empty digit lists; the existing BMP proof excludes
surrogates. Leading zeros use constant decoder storage. Decoded syntax characters
remain literal units rather than being reparsed as Pattern syntax. Ordinary
character and class escape decoding remains separate, retaining the project's
edition-17 scope without Annex B. Flat capture storage, linear literal matching,
UTF-16 ranges and the narrow approved empty-body boundary behavior are unchanged.


Validated Unicode u-mode Patterns now admit exactly one case-sensitive
CharacterSetMatcher atom whose members are all nonsurrogate BMP characters
(22.2.2.7.1, 22.2.7.2). Positive classes can contain literal characters, ordinary
character escapes and ranges excluding surrogate values, plus digit, word and
whitespace class escapes. Empty positive classes produce no matches. Validation
precedes parsing, and the compiler rejects inverted sets, complemented escapes,
dot, surrogate-containing ranges and non-atom syntax before construction.

Each member consumes one code point and one UTF-16 unit. Scanning either half of
a surrogate pair cannot succeed, so existing membership bits and unit search
preserve successful Unicode boundaries. The caller uses the u-only entry point;
UnicodeSets syntax and ignore-case folding require separate proofs. Construction
retains existing opt-in work charges; matching, storage and default quotas are
unchanged. Literal capture plans and the approved empty-body boundary behavior
retain their separate paths.


Validated UnicodeSets v-mode Patterns now admit one case-sensitive atom with
exclusively nonsurrogate BMP membership (22.2.2.7.1, 22.2.7.2). Flat positive
unions and ranges have the same membership as the proved u-mode class subset.
Complete UnicodeSets validation precedes compilation. The ordinary class parser
cannot consume a complete nested class or property/string escape; a conservative
scan additionally rejects possible intersection and subtraction operators before
construction. This prevents operators from acquiring ordinary literal semantics.

The compiler retains digit, word and whitespace escapes and one-unit Unicode
boundaries through the existing membership proof. Escaped punctuation remains
literal. Inverted sets, complemented escapes, UnicodeSets operators, nested
classes, class strings, braced escapes, supplementary values and ignore-case
matching remain unproved. Construction accounting, matching, storage, dependency
allowances and default quotas are unchanged. The approved empty-body boundary
resolution retains its separate scope.


Braced Unicode escapes now retain their scalar values inside proved u/v BMP
character classes and range endpoints (22.2.1, 22.2.2.7.1). The class parser uses
the literal compiler's checked, constant-storage hexadecimal decoder only when
Unicode mode is explicit. The ordinary parser retains its original grammar.
Surrogates, supplementary values and ranges crossing the surrogate interval are
rejected before membership construction.

An escape that decodes to a bracket, hyphen, caret or ampersand contributes a
literal character; it does not become class syntax or a UnicodeSets operator.
Full Pattern validation still precedes compilation. The existing conservative
v-mode operator scan and positive BMP membership proof remain in force. Long
leading-zero spellings require no additional decoder storage. Matching, capture
storage, construction accounting, dependency allowances and opt-in quota policy
are unchanged. Supplementary membership, inversion, complemented escapes,
ignore-case matching, nested classes and UnicodeSets operations remain unproved.


Case-sensitive u/v scalar literal concatenations now include supplementary
characters (22.2.1, 22.2.2.7, 22.2.7.2). Each atom is decoded before group
flattening: a raw source pair, one braced scalar escape, or directly adjacent
fixed lead/trail Unicode escapes denotes one code point. Escaped surrogates
separated by groups and braced surrogate atoms cannot become an emoji through
flattening. Lone surrogate atoms remain unproved.

All admitted atoms encode either as a nonsurrogate BMP unit or a complete pair.
Consequently each occurrence and capture endpoint is a code-point boundary.
Existing immutable KMP programs can search UTF-16 directly in linear time without
code-point arrays or native recursion. The Unicode dispatch uses the approved
initial leading boundary and retains ordinary result construction, named metadata,
coercion order and strict lastIndex writes. Construction charges each compilation
pass; opted-in search accounting includes a possible one-unit rewind. Default
work and heap quotas remain absent. Supplementary character classes, quantifiers,
assertions, backreferences and nonempty ignore-case matching remain unproved.


Case-sensitive Unicode literal concatenations now include lone surrogate atoms
(22.2.1, 22.2.2.7, 22.2.7.2). A shared atom decoder preserves raw source pairs
and directly adjacent fixed lead/trail escape pairs while allowing isolated
surrogates as Unicode code points. The existing scalar API retains its narrower
proof. Braced surrogate escapes and surrogates separated by group syntax remain
separate atoms.

Before flattening, a distinct lone trail atom immediately after a lone lead atom
is rejected: no Unicode input can contain that adjacent pair as two characters.
This also catches empty and noncapturing groups between the atoms. Every other
internal atom and capture boundary agrees with UTF-16 decoding. The two external
match bounds are checked against the input; an occurrence consuming half of an
input pair is rejected. The same KMP failure state resumes after rejection,
retaining linear search without code-point arrays, recursion or mutable programs.

The previously approved Node/V8 initial-pair boundary rule, metadata, result
construction, strict lastIndex writes, conversion order and opt-in work accounting
are retained. Long leading-zero escapes and deep groups use flat storage. Default
work and heap quotas remain absent. Distinct surrogate atoms which would flatten
into a pair, character classes containing surrogates, supplementary membership,
nonempty case folding and other unproved Unicode bodies remain Unsupported.


Native case-sensitive Unicode literal construction now selects the complete
code-point literal proof directly, covering BMP units, supplementary characters
and isolated surrogates in one runtime plan. The separate BMP and scalar core
APIs retain their documented contracts. Repeated narrower compilation attempts
and the duplicate scalar runtime variant are removed; mandatory empty bodies
retain their independent ignore-case proof and dispatch.

All literal occurrences use the existing atom proof, linear KMP scan and complete
input-boundary checks. Captures, the approved initial-pair rule, validation,
original metadata, conversion order and strict lastIndex behavior are preserved.
Construction precharges one literal compilation pass instead of retrying narrower
plans. Search still accounts for a possible one-unit rewind. Quotas remain opt-in,
without dependencies or changes to the admitted syntax.


Single case-sensitive Unicode dot and CharacterClassEscape bodies now have a
constant-size plan (22.2.2.7, 22.2.2.7.1, 22.2.2.9). Dot excludes exactly the
four LineTerminator values unless s is set. The d/w/s escapes use decimal ASCII,
ASCII word membership and ECMAScript whitespace plus line terminators; their
uppercase counterparts complement those sets across all Unicode code points,
including supplementary characters and isolated surrogates.

Matching decodes input characters while scanning, with no input-sized allocation,
consuming paired surrogates together and lone surrogates individually. UTF-16
result ranges use complete-character boundaries, with the approved initial-pair
leading-boundary rule. Existing BMP APIs and ordinary unit matching are preserved.
Only a complete dot or escape body is admitted; groups, concatenations, classes,
anchors, quantifiers and ignore-case Unicode matching remain separate proofs.

Opted-in construction charges constant plan work; search precharges up to two
sticky units or the suffix plus a possible rewind unit. This covers a dot's
one-unit Pattern spelling consuming a two-unit input character. Quotas remain
opt-in, and host failures preserve completion and lastIndex state.


Unicode single-dot and character-escape plans now admit an optional leading ^
and/or trailing $ (22.2.2.4). Multiline assertions recognize exactly the four
LineTerminator values in the preceding/following complete-character context.
Without m, ^ requires the actual input start and $ requires the actual input end;
there is no implicit acceptance before a final newline. CR and LF are individual
characters, each eligible as a multiline boundary.

The plan adds constant-size assertion flags to the existing complete-character
scan. Sticky execution checks only the normalized initial character; nonsticky
execution preserves candidate order. Successful ranges, the narrowly approved
initial-pair rule, original metadata, consumers and strict lastIndex behavior
remain shared. Source decoding and ordinary matching are unchanged.

The original bare-atom core entry points retain their contracts; separate
assertion entry points validate the complete admitted body and precharge its
source length. Optional search quotas cover the input scan and each assertion
check before execution, with at most two sticky units and a possible rewind unit.
Groups, literal/class compositions, quantifiers, alternatives, empty anchored
bodies and Unicode ignore-case matching remain separate proofs. Quotas stay
opt-in, with no new dependency or input-sized matching allocation.


Case-sensitive flat u/v classes now compile to immutable, disjoint Unicode
intervals (22.2.2.7.1, 22.2.2.9). Unions, ranges, outer inversion and d/D/s/S/w/W
escapes cover all code points, including supplementary values and lone surrogates.
The existing literal atom decoder preserves raw/fixed surrogate pairing, while
braced surrogates remain distinct class elements. Backspace and reserved escaped
punctuation remain character data; decoded hyphens/brackets never create grammar.
Raw v intersection/subtraction and nested class operands remain separate proofs,
as do property/string escapes, groups, anchors, quantifiers and Unicode folding.

Repeated class escapes set one of six bits and emit each fixed predicate once,
so repeated escapes never expand into repeated universe-sized loops. Three stable
byte passes sort 21-bit starts with exact linear work bounds, followed by a linear
overlap/adjacency merge. Temporary storage scales with source size; ranges never
expand into individual characters. Immutable intervals are shared across clones.

Input matching shares the complete-character decoder and approved initial-pair
rule with Unicode dot/escape plans, with no input-sized allocation. Membership
uses binary interval lookup. Opt-in construction precharges source, fixed-set
expansion, sorting, merging and final storage; search covers decoding and lookup
passes, including up to two sticky units and one possible rewind. Existing BMP
core APIs and ordinary matching retain their contracts. Quotas remain opt-in,
with no dependency or native recursion added by the compiler/matcher.


Case-sensitive flat Unicode classes also admit one optional leading ^ and one
optional trailing $ (22.2.2.4). Class compilation borrows the body and charges the
complete original source without copying it. Existing bare-class core entry
points retain their contracts; assertion entry points take the multiline flag.
Both Unicode character and class plans share one assertion-context helper, using
complete input-character boundaries and all four line terminators. CR and LF are
individual contexts; non-multiline $ requires the actual input end. Search work
includes each enabled assertion check in addition to decoding and membership.
Sticky matching considers only the normalized initial complete character.
Groups, concatenations, quantifiers, Unicode folding and v operators remain
separate proofs, while escaped punctuation remains class data.


Single Unicode dot and d/D/s/S/w/W escape atoms also admit i, including their
optional boundary assertions (22.2.2.7.3, 22.2.2.9). Dot, digit and space sets are
invariant under folding. Word escapes canonicalize each complete input character
through the pinned simple/common table before ASCII word membership or inversion,
so long s and Kelvin sign match while full/Turkic mappings do not apply. No fold
changes the consumed character width or the original returned text. Existing
case-sensitive bare/assertion APIs retain their contracts; flag-aware APIs admit i.
Word search work additionally bounds pinned-table binary lookup. Unicode classes,
literal folding, groups, concatenations and quantifiers retain their independent
proof boundaries. The runtime guards class/BMP plans before any i fallback.


Flat u/v classes also admit i (22.2.2.7.3, 22.2.2.9). Original intervals are
normalized first. A bounded scan of the pinned simple/common table adds targets
only when the corresponding source belongs to the original set, followed by a
second normalization. Mapped source values can remain in the interval union:
idempotent canonical input never reaches them. Ranges never expand into members.
Input canonicalization precedes membership and outer inversion, while consuming
and returning the original complete input character and its UTF-16 width.

Word complements exclude the pinned non-ASCII ASCII-word aliases before folding,
preventing long s or Kelvin sign from adding ASCII word targets to the complement.
Exhaustive tests verify these additions against the pinned table. Construction
work covers each fixed-table lookup, possible target append, interval extension
including an allocation move, second normalization and final immutable storage.
Search work additionally bounds input folding. Bare/assertion core APIs retain
case-sensitive contracts; flag-aware APIs admit i. Property/string escapes,
operators, nested v sets, groups, concatenations and quantifiers remain separate.

Unicode 18 includes U+1DF95 -> U+00DF in simple folding. The former consumes two
UTF-16 units and the latter one; folding never substitutes either width for the
original input width. The installed Node v24.19.0 uses Unicode 17 and differs on
eight of twenty-four dedicated cross-plane cases. Those cases are independently
checked against the digest-pinned Unicode 18 CaseFolding.txt; shared-version
cases agree with Node. This data-version difference does not change the approved
Node/V8 initial-pair boundary rule or the existing Unicode 18 baseline.


Single Unicode character/class plans admit complete enclosing capturing and
noncapturing groups (22.2.2.7). The existing constant-space three-scan wrapper
helper borrows the retained body. The complete body must compile as a proved
atom before any wrappers are admitted, excluding nested v sets and string
escapes whose delimiters the helper does not interpret. Original construction
validation, named-slot normalization and source/flag metadata remain shared.
A source-bounded body copy is charged before allocation. All enclosing captures
span the original complete input character, with existing named bindings and
indices aliasing. Search retains the same matcher and input widths. Wrappers do
not add native recursion. Quantified groups, continuation/alternative bodies and
assertions outside the complete wrappers remain separate matching proofs.


A separate Unicode character entry point admits one decoded literal atom with
optional leading ^ and trailing $ (22.2.2.7.3). It shares Unicode literal decoding
with the existing concatenation compiler. An assertion is recognized only after
decoding the atom, so escaped dollars/carets and fixed/braced escapes retain their
literal values. The original dot/escape APIs retain their previous subsets.

The decoded pattern character and complete input characters use the pinned
simple/common canonicalization when i is set. A constant-size literal plan stores
the canonical point, while search consumes the original input character and
returns its original UTF-16 width. Existing complete wrappers provide captures
and names. Source work is charged before decoding even rejected bodies: braced
escapes may contain source-sized runs of leading zeroes. Construction and search
also cover bounded pinned-table lookups. The installed Node's Unicode 17 data
differs on six of sixteen dedicated cross-plane cases; independent digest-pinned
Unicode 18 data verifies all sixteen. No older data replaces the existing pin.
Concatenations, quantified literals and interior groups remain separate proofs.


Unicode i literal concatenations share the existing complete-character literal
proof (22.2.2.7.3). Original source capture offsets become code-point ordinals,
then immutable canonical points receive a freshly computed KMP prefix table.
This preserves borders introduced by folding without using source UTF-16 widths
to predict input boundaries. Mandatory and empty groups remain in source order.

Search folds complete input points and charges actual decoder, lookup and
prefix operations through the existing fallible dispatch. Only a successful
match walks its matched points backward to find the original start, then
resolves captures in one forward pass with source-sized temporary storage.
Captured text, empty positions, indices and names use original input widths;
the input itself is never copied or retained by the match. No native recursion
is added. Construction charges source passes before decoding/storage and bounds
each pinned lookup and capture-boundary binary search. Search charges storage
before allocating it. Existing single-atom plans remain the first choice.

The installed Node's Unicode 17 data differs on ten of twenty-four dedicated
Unicode 18 capture-width cases. Independent digest-pinned CaseFolding.txt verifies
all twenty-four, including supplementary/BMP changes and interior empty captures.
References, alternatives, classes, quantified bodies and assertions remain
separate proofs. The prior exclusion of distinct surrogate atoms that would
flatten into one pair remains: Unicode input cannot match them as two characters.


Unicode folded literal concatenations also admit an optional leading ^ and
trailing $ around the body, including assertions inside complete enclosing
ordinary groups (22.2.2.4 and 22.2.2.7.3). Named wrappers retain source-order
capture slots. A separate core assertion entry point preserves the bare literal
API's subset. After charged constant-space wrapper scans, the complete retained
body must pass the existing literal proof. Escaped anchors stay literal; interior
assertions and quantified/alternative/class bodies remain separate proofs.

The existing Unicode context helper checks actual original input boundaries.
Without m, ^ accepts only the initial boundary and $ only the actual end. With m,
LF, CR, LS and PS independently provide boundaries, including both CR and LF.
Search retries overlapping folded candidates using the existing KMP failure
table. End assertions are checked before walking back to a candidate start.
Only a multiline start assertion allocates a ring of pattern-point-count plus
one original UTF-16 boundaries; every rejected candidate's start is then an O(1)
lookup. This keeps repeated assertion failures linear in the input length,
without input-sized storage or native recursion. Successful captures are still
resolved once from original complete input characters. Source scans/copies,
ring allocation/writes and context checks are charged through opt-in host work.


The same flat Unicode literal program supports case-sensitive boundary matching
(22.2.2.4 and 22.2.2.7.3). Its explicit flag entry point canonicalizes pattern
and input points only with i. Without i, it compares exact complete code points
and does no pinned-fold lookup or lookup work charge. Both paths compute KMP
prefixes from the actual comparison points and use identical original-width
capture resolution and multiline context. Existing bare core APIs retain their
Unicode i subset; prior single-atom and bare case-sensitive runtime plans remain
the first choice. The final fallback admits the complete validated normalized
Unicode source only when the shared literal proof succeeds. Interior assertions,
references, alternatives, classes and quantified bodies remain separate proofs.


Standalone ASCII/Any property escapes reuse the immutable Unicode interval plan.
Exact existing grammar names select 0..127 or 0..0x10ffff; no generated data or
dependency is added. Optional boundary assertions and complete enclosing
capturing/noncapturing groups share the previous context and capture machinery.
Any consumes all complete input characters, including isolated surrogates and
all line terminators, independently of s. Original text and UTF-16 widths remain.

The case-insensitive complements follow edition 17 CompileToCharSet,
MaybeSimpleCaseFolding, AllCharacters and CharacterSetMatcher. With u, P
complements the raw property before folding. With v, it complements the folded
property within canonical characters; the shared plan's outer inversion is
equivalent because input points are canonicalized. Construction shares the
existing source/set/fold/normalization charges and immutable Arc storage.
Other property sets and continuations remain separate proofs.

The installed Node v24.19.0 differs on ASCII property escapes with iv: it rejects
long s and Kelvin sign for p{ASCII} and accepts them for P{ASCII}. The tagged
ES2026 source (commit 0248456c758431e4bb8e5d26333ff1865123c9cd) instead applies
Canonicalize to the input and property members, so p matches both and P rejects
both. Independent raw-code-point predicates verify that rule for every code
point, and independent candidate/context/capture oracles verify all gold rows.
This implementation follows the specification; the approved Node exception
remains limited to leading whole-match starts inside surrogate pairs.
See [CompileToCharSet](https://262.ecma-international.org/17.0/#sec-compiletocharset),
[MaybeSimpleCaseFolding](https://262.ecma-international.org/17.0/#sec-maybesimplecasefolding)
and [CharacterSetMatcher](https://262.ecma-international.org/17.0/#sec-runtime-semantics-charactersetmatcher-abstract-operation).


Flat Unicode classes also admit ASCII/Any property escapes as set-valued atoms
(CompileToCharSet and CharacterRange). Four bits deduplicate the fixed positive
and negative sets, which append at most three intervals each after source parsing.
Property sets remain excluded as range endpoints. Exact names, ordinary unions,
outer inversion, existing escapes and boundaries share validation and immutable
interval normalization. Nested v sets, set operators and other properties remain
separate proofs.

For v with i, P{ASCII}'s interval preimage excludes long s and Kelvin sign before
union and the common fold closure; otherwise folding those source members would
incorrectly add s/k to the complemented canonical set. A literal K or a word
escape can add those characters back as part of a union. With u, the raw property
complement intentionally includes those aliases before folding. The same tagged
edition-17 rule documented for standalone properties applies, including the
installed Node discrepancy. Original widths and captures remain unchanged.
Property interval appends charge the possible original-vector move before
allocation. Repeated properties use source-linear parsing and constant additional
fixed sets, rather than repeated property expansion. No native recursion is added.


Optional leading ^ and trailing $ may also surround complete ordinary wrappers
around a single proved Unicode atom (22.2.2.4). The constructor removes those
outside assertions, peels the same complete ordinary wrappers, and restores the
assertions around the retained body before complete atom compilation. Moving a
zero-width assertion inside the wrapper preserves its whole-character capture
range. An escaped trailing dollar is kept literal; consecutive backslash parity
is inspected before treating a raw final dollar as an assertion. Original named
slots, source/flags, text and indices remain unchanged.

The source scan is charged before delimiter/parity passes and both source-sized
copies are charged before allocation. No input storage or native recursion is
added. Full dot/escape/literal/class/property body proof still excludes nested
v sets, string sets, continuations, alternatives and quantifiers. Repeated
assertions and assertions around interior groups remain separate proofs. Existing
full literal-concatenation plans remain the fallback when one-atom proof fails.


The fixed Unicode binary property matcher also admits ASCII_Hex_Digit and its
exact AHex alias (22.2.2.9). Unicode 18 PropList.txt defines only 0030..0039,
0041..0046 and 0061..0066. The constructor and flat class parser share the same
three positive or four complementary intervals. The entire pinned simple/common
fold table preserves membership in this set, so u/v complement ordering yields
the same membership with i. Class inversion still follows union and folding.

Fixed intervals are appended once per property kind and charged before any
allocation, even for repeated property names. Shared normalized immutable ranges
retain complete-character matching, original capture text and UTF-16 indices,
optional assertions and clones. No per-input storage or recursion is introduced.
Hex_Digit, other properties, nested sets, v operators and repeated property atoms
remain separate proofs.


Hex_Digit and its exact Hex alias use the same fixed binary property path
(22.2.2.9). Unicode 18 PropList.txt adds FF10..FF19, FF21..FF26 and FF41..FF46
to ASCII hex membership. The six positive or seven complementary intervals
share ordinary property parsing, deduplicated class union and immutable range
normalization. Every pinned simple/common fold preserves membership, so u/v
complement orders agree with i while retaining fullwidth original input text.

The larger fixed append is charged before allocation; flat class appends remain
bounded by seven intervals per kind. Complete-character assertions, wrapper and
named captures, source/flags, clone compilation and UTF-16 indices reuse the
existing plans. Other Unicode properties and v set operators remain separate
proofs, with no new dependency, recursion or input-sized storage.


White_Space and its exact space alias use the fixed Unicode binary property
matcher (22.2.2.9). Unicode 18 PropList.txt defines 25 members, represented as
ten positive or eleven complementary intervals after adjacent normalization.
This property includes NEL (0085) and excludes BOM (FEFF); the ordinary character
escape s has the opposite membership for those two characters. Complete pinned
C/S folding preserves property membership, so u/v complement orders agree with i.

Standalone and flat-class property parsing now share one exact alias lookup.
The class property mask has sixteen bits, enough for the ten admitted positive
and complementary kinds. Each fixed kind contributes its intervals only once;
the source-sized move and larger fixed append are charged before allocation.
Complete-character boundaries, immutable ranges, original captures and indices,
assertions, consumers and clones reuse the existing plans. Other properties and
set operators remain separate proofs.


The Unicode class/property matcher has a separate entry point for one optional
leading and/or trailing word assertion b/B, combined with optional ^/$
(22.2.2.4, 22.2.2.7.1). It strips only those complete outside assertions, restores
the existing line assertions, and requires complete flat-class or fixed-property
body proof. Consecutive backslash parity distinguishes a final word assertion
from escaped literal backslashes. Existing class/property entry points retain
their own admitted subsets.

WordCharacters is ASCII letters, digits and underscore, plus the pinned long-s
and Kelvin aliases only with Unicode i. Complete-character endpoints make the
adjacent raw input units sufficient: surrogate units cannot be word characters.
The matcher compares left and right membership at the original start/end; B
requires equality. Line assertions and original capture ranges remain unchanged.

The source scan and source-sized copy are charged before allocation. Eight
additional fixed work passes per word assertion precharge neighbor checks before
search. No input storage or recursion is added. Whole ordinary atom wrappers
retain original named slots, text and UTF-16 indices. Word assertions outside wrappers and repeated/continued bodies remain separate
proofs.


The Unicode character matcher separately admits the same optional word assertions
around one complete literal, dot or d/D/s/S/w/W escape atom (22.2.2.4,
22.2.2.7.1). It shares the class matcher's source/parser and neighbor helper, then
requires complete existing dot/escape or single-literal proof. Prior entry points
retain their own subsets. Source-sized parsing/copying is charged before work and
allocation, with eight extra fixed search passes per word assertion.

Neighbor folding has its own i flag. Dot/digit/space atoms avoid unnecessary
canonicalization while their neighbors still include long-s and Kelvin under
Unicode i. Literal and word-escape folding use the existing pinned C/S data;
original input widths, line context and capture/indices ranges remain intact.
Escaped backslashes and literal dollars remain consuming atoms, distinguished
from outside assertions by the shared complete-body proof. No input allocation
or recursion is added.


The Unicode character matcher has a separate constant-storage entry point for
one standalone \b/\B assertion, optionally surrounded by ^/$ (22.2.2.4).
Complete validated u/v bodies must consist of exactly those assertions. The
existing consuming entry points retain their subsets. Construction is charged
before examination; the existing neighbor and line-context helpers perform the
match without source/input allocation or recursion.

Zero-width search includes the original input end. Failed candidates advance
by complete code points (22.2.7.2); successful ranges are empty at the original
UTF-16 position. Exec writes that position without advancing an empty success,
while the shared match/replace/matchAll consumers advance with AdvanceStringIndex.
Complete ordinary wrappers capture that same empty range, with original named
slots and indices aliases. The approved initial-offset leading-boundary rule
continues to apply when a caller starts inside a surrogate pair.

Node 24.19.0 has a separate standalone \B discrepancy:
`/\B/ug.exec("A😀B")` reports a zero-width match at UTF-16 index 2, between
the surrogate halves; `/\B/uy` with lastIndex 2 also reports that interior index. Edition 17
22.2.7.2 converts the input with StringToCodePoints, supplies character-list
indices to the matcher, and advances failed search candidates with
AdvanceStringIndex. Assertion IsWordChar (22.2.2.4) inspects neighboring elements
of that list. Those rules exclude the interior surrogate position; the native
matcher follows them. This example has no valid B match, so its result is null.
The already approved initial-offset ambiguity and leading-boundary resolution
remain documented above; this additional Node behavior has no specification
ambiguity.


A separate Unicode line-assertion entry point admits exactly ^, $ or ^$
(22.2.2.4). It reuses the constant-storage empty atom and the existing complete
character candidate search. Construction is charged before examination; no
source/input allocation or recursion is added. The prior word/consuming APIs
retain their subsets. IgnoreCase does not affect line assertions, while the
original i flag remains observable on the native object.

Multiline ^/$ inspect the preceding/following line terminator. The four line
terminator code points are tested separately, including the empty position
between CR and LF. Non-multiline ^/$ require the actual input start/end. Empty
input and input-end matches retain their original UTF-16 position, and the
approved initial-offset leading-boundary resolution remains in force. Complete
ordinary wrappers capture that same empty range; exec and the shared generic
consumers retain their established empty-match advancement behavior.


A separate Unicode assertion-sequence entry point admits complete bodies with
at least two ^, $, \b or \B assertions (22.2.2.4). Every condition observes the
same unchanged position. A charged linear parse deduplicates line assertions and
each word condition into constant storage; opposite word conditions retain a
failing conjunction. No source/input allocation, recursion or dependency is
added. The prior entry points retain their subsets, and the constructor still
requires complete u/v validation before choosing this proof.

The existing complete-character search preserves multiline context, i-folded
word neighbors, original empty capture indices and the approved initial-offset
resolution. Complete ordinary wrappers retain their existing capture proof.
Consumers keep their established empty-match advancement. Quantified groups,
alternatives, lookaround and mixed consuming bodies require separate proofs.
The documented Node interior-pair \B discrepancy also affects repeated \B
assertions; it does not change the edition-17 candidate positions.


Unicode literal concatenations have a separate word-boundary entry point
(22.2.2.4, 22.2.2.7.1). Optional leading/trailing \b or \B and ^/$ surround a
complete nonempty literal body. Complete ordinary wrappers retain their capture
slots, including partial and empty captures. The constructor validates the
whole u/v Pattern and normalizes named groups before trying this proof; prior
literal APIs retain their subsets.

Construction charges source examination and copies before allocation. Unicode i
folds literal comparisons and original word neighbors independently, while
captures retain original complete-character UTF-16 boundaries. A source-sized
ring retains candidate starts whenever a leading word assertion needs them.
Rejected overlapping KMP candidates check both boundaries in constant time;
they never rescan the matched prefix. Successful captures use the established
original-boundary resolver. No input-sized storage, native recursion or
dependency is added. Internal assertions and nonliteral bodies require separate
proofs, and the approved initial-offset resolution remains unchanged.


Fixed binary Unicode property plans also admit Join_Control and its exact
Join_C alias (22.2.2.9). The pinned PropList.txt contains only U+200C and U+200D;
one positive interval and two complementary intervals represent the complete
set. Pinned C/S folding preserves membership in both directions, so the u/v
complement orders coincide for this property. Existing flat unions, inversion,
assertions and complete enclosing capture proofs remain shared.

Repeated aliases and complements append each fixed set once through the
existing u16 property mask. Bounded interval append work is charged before
allocation. No Unicode expansion, source/input-sized membership table,
recursion, dependency or baseline change is added. Join controls remain
distinct from JavaScript whitespace and WordCharacters.


Noncharacter_Code_Point and its exact NChar alias use eighteen positive and
eighteen complementary intervals (22.2.2.9). The pinned PropList.txt specifies
U+FDD0 through U+FDEF and the final two code points of each of the seventeen
Unicode planes: sixty-six code points, including U+10FFFF. Noncharacters are
valid ECMAScript string and RegExp characters, rather than encoding errors.
Pinned C/S folding preserves membership, so u/v complement orders coincide.

The existing fixed-property u16 mask deduplicates repeated aliases and
complements before interval allocation. Bounded append and normalization work
is charged before growth. Flat unions, inversion, assertions and complete
enclosing captures share the established plans. There is no Unicode expansion,
new dependency, native recursion, default quota or baseline change.


A dedicated Unicode repetition plan proves one complete consuming literal, dot,
escape, flat class or supported binary property followed by *, +, ?, or exact,
open or bounded decimal counts, with optional lazy ordering (22.2.2.3,
22.2.2.7). Full u/v syntax validation precedes the proof; assertions, alternatives,
concatenations and quantified capturing groups require separate plans. Complete
enclosing captures share the selected original whole-match range.

The retained atom is proved completely before choosing a quantifier parse,
including the ambiguity between a braced Unicode escape and a numeric bound
before ?. Decimal comparisons retain exact bound order without machine-sized
truncation. A minimum beyond representable string storage cannot match; a
maximum beyond it cannot constrain any representable input. Counts never expand
into repetitions, and leading zeros add no artificial source limit.

Each attempt consumes complete characters up to its bound, greedily or lazily.
If a run is shorter than the minimum, no later start within that run can succeed;
search skips that run and its failing character once. Search is linear with
constant storage, and charges opt-in work before each character/membership
operation. Initial offsets inside a pair use the approved leading boundary,
including empty quantified matches. Consumers retain AdvanceStringIndex for
empty results. There is no new dependency, recursion or default resource quota.


The single Unicode atom repetition plan also proves an optional leading ^
(22.2.2.4). Candidate positions must be the actual beginning or, with m, follow a
LineTerminator. This includes positions between CR and LF and the input end
when an empty quantified match is possible. The assertion examines the original
input rather than a sliced suffix. A dedicated flag-aware entry point retains
the existing unanchored API's complete-body contract.

Candidates failing the assertion advance by one complete character. A failed
short run still cannot meet the minimum from any later start within that run,
so the established skip remains valid. Search remains linear with constant
storage and charges context/character work before use. A nonmultiline or sticky
start cannot retry an ineligible candidate. Complete enclosing groups, including
an assertion outside them, retain the selected original match range. Trailing
assertions and quantified capture bodies remain separate proofs.


A dedicated line-assertion entry point also proves a trailing $ on the single
Unicode repetition plan without m (22.2.2.4). Only the actual input end satisfies
this assertion, including when the final character is a LineTerminator. Optional
leading ^ requires the actual beginning. The established multiline-start entry
point and its unanchored contracts remain available. Multiline $ has its own
monotone endpoint scan.

The complete-character scan records the final matching run. A monotone second
pointer trims that run to a finite maximum, leaving its earliest eligible start.
The minimum must still be satisfied. Sticky search and leading ^ reject any
necessary change of their fixed start. The end assertion gives one possible
endpoint, so greedy and lazy repetition order produce this same earliest range.
A zero maximum selects the actual input end directly, subject to the fixed-start
assertions. All returned ranges, including empty ones, use original indices.

This is equivalent to independent candidate/count enumeration with $, while
remaining linear in time and constant in search storage. Character membership
and pointer movement charge opted-in work before use. No input-sized offset
array, repetition expansion, recursion, dependency or default quota is added.
Complete enclosing captures keep the selected whole-match range.


Multiline trailing $ on the single quantified Unicode atom accepts the actual
input end and complete-character positions before each LineTerminator
(22.2.2.4). CR and LF remain separate assertion positions. Optional leading ^
accepts only the beginning or a position after a LineTerminator. The constructor
still proves the whole body, retaining complete outer captures and exact counts.

Two monotone pointers maintain the earliest matching-run start within a finite
maximum and the complete-character endpoint under examination. A leading
assertion can further trim that run. Eligible ends are visited in order. The
first successful start cannot move backwards; lazy search returns its first
eligible end, while greedy search extends that same start until its maximum or
membership failure. A later start never displaces an earlier successful one.
Independent start/count enumeration verifies this ordering, including atoms
which consume terminators and zero maxima. Each pointer crosses a character
at most once, preserving linear work and constant search storage. Folded
membership, original pair/lone-surrogate indices and opted-in charges retain
their established rules. No recursion, dependency or default quota is added.


A dedicated capture entry point proves complete ordinary/noncapturing wrappers
inside a repeated Unicode atom (22.2.2.7). The constructor validates the entire
Pattern and normalizes named groups first. A charged source scan counts only
ordinary captures, verifies all closing wrappers, and constructs a source-sized
retained atom/repetition body. Its established unanchored compiler must prove
one consuming atom before any outer line assertion is reapplied. Assertions
inside repeated groups cannot move outside their original scopes; empty bodies,
alternatives and concatenations require separate proofs. Prior public compiler
entry points retain their established subsets.

Every complete inner group captures the final selected atom iteration. After
the repetition's range is selected, one checked complete-character boundary
determines this shared original UTF-16 span. Zero selected iterations leave all
inner captures undefined; complete outer groups still capture the whole range.
A constant-size optional span in the runtime match serves every inner slot,
without per-capture search storage. Existing result construction retains named
properties, original strings, undefined own slots and indices-group aliases.
Greedy backtracking to an earlier line endpoint derives captures from that
selected endpoint rather than from the discarded scan. Source construction,
search and span extraction charge opted-in work before use; no recursion,
input-sized offsets, expanded counts, dependency or default quota is added.


A separate repetition entry point admits one leading \b/\B outside a complete
quantified Unicode atom (22.2.2.4), optionally after ^ and before a trailing $.
The existing charged word-assertion scan preserves line scopes. Established
line and complete-inner-wrapper proofs must accept the entire retained body;
trailing word assertions and assertions inside the atom require separate proofs.
Prior compiler entry points retain their established subsets.

The established monotone start/endpoint search checks both leading line and
word predicates before selecting a candidate. Its endpoint gate retains EOF
and multiline $ contexts, or accepts any endpoint without $. Earliest starts,
finite maxima and greedy/lazy count order remain unchanged. WordCharacters
uses the shared pinned Unicode-i aliases and original neighboring units at
complete-character boundaries. Final selected iterations supply inner captures;
complete outer groups retain the whole range. Search remains linear in input
length with constant storage and no native recursion. Source scans, copies,
membership and predicates charge opted-in work before use; defaults stay
unlimited and no dependency or fixture allowance is added.


The complete outside-word repetition compiler adds a trailing \b/\B, optionally
paired with a leading word assertion and ^/$ (22.2.2.4). Both public word entry
points share the established full retained-body proof; the start-only API keeps
its prior restriction. The constructor selects the broader entry once, without
reparsing an already accepted leading predicate. Assertions inside repeated
groups retain their scopes and require separate proofs.

The monotone endpoint search checks the original trailing word context along
with any line predicate. It retains the best endpoint for the earliest
successful start in greedy mode and returns the first eligible endpoint in
lazy mode. Finite maxima trim starts only before a success. Backtracking never
retains discarded atom captures: existing span extraction uses the final
selected complete-character endpoint, or leaves inner captures undefined for
zero iterations. Named groups and original enclosing capture bounds continue
through existing result construction. Every predicate charges opted-in work
before use. Search stays linear with constant storage and no input offsets,
expanded counts, native recursion, dependency or default quota.


A dedicated repeated-group compiler retains optional ^/$ inside a complete
consuming Unicode atom (22.2.2.4, 22.2.2.7). It shares the complete-wrapper and
quantifier proof with the prior capture entry, which still excludes these inner
assertions. Only this scoped route enables line predicates on the retained
character, literal, flat-class or fixed-property matcher. Each consuming
iteration checks its own original start/end and multiline context. Outside
line assertions remain separate predicates on the whole selected repetition.

No assertion is moved across a group or quantifier. For example, (^a)+ selects
one initial a from aaa, and (a$)+ selects the final a. Required repeated counts
cannot reuse either assertion as an outside predicate. With m/s, adjacent line
terminators can each satisfy both assertions in (^.$)+. Zero selected iterations
never evaluate the inner predicates and leave all inner captures undefined.
The selected final complete character continues to supply every complete inner
capture. Empty units, inner word predicates, alternatives and concatenations
remain separate proofs. Existing monotone searches retain linear work and
constant input storage. Source scans and copies use established precharges;
predicates charge opted-in work before use. No expanded counts, recursion,
dependency, corpus alteration or default quota is added.


The scoped outside-word repetition entry composes the established word scan
with the complete per-iteration line-group proof (22.2.2.4, 22.2.2.7). The entire
retained body must pass that group proof before any outside word predicate is
attached. The prior word entry points keep their established restrictions.
Line assertions inside the atom remain checked at every consuming iteration;
outside ^/$ and word assertions apply to the whole selected repetition.

This adds no search algorithm or input storage. Established monotone candidate
and endpoint gates select the earliest eligible start and greedy/lazy endpoint.
The final selected iteration supplies complete inner captures, while zero counts
leave them undefined and do not evaluate inner assertions. For example,
\b(^a)+\B selects one initial a from aaa, while \b(^a)+\b cannot pass its
trailing word assertion. Existing folded WordCharacters, UTF-16 boundaries,
named groups, clone flags and result construction retain their behavior.
All scans, copies and predicates use established opted-in charges. No assertion
moves across a quantifier, and no recursion, expanded counts, dependency,
fixture change or default quota is introduced.


A dedicated complete-group entry admits optional word and line assertions
around the single consuming atom inside a repetition (22.2.2.4, 22.2.2.7).
The established character and flat-class/property compilers prove the entire
retained atom before the repetition is accepted. Their word predicates use
original neighboring input characters, including the Unicode i aliases. The
private assertion mode keeps the older entry points' narrower contracts.

Each selected iteration executes its own assertions. Thus (\ba)+ selects one
initial a from aaa, while (a\b)+ selects only the final a. Moving either word
assertion outside the quantifier would change the result. A zero count never
executes the child and leaves its captures undefined; complete inner wrappers
otherwise capture the last selected consuming character. Optional outside ^/$
keep their separate scopes. Outside word assertions, empty units, alternatives
and concatenations still require other complete proofs.

Search and capture construction remain shared with the established monotone
repetition matcher, with no new input storage, recursion or expanded counts.
Existing leaf work charges include the retained word and line predicates.
There is no new dependency, fixture change or default quota.


The composed-word repetition entry combines outside word predicates with the
complete consuming-group proof for retained per-iteration word and line
assertions (22.2.2.4, 22.2.2.7). It proves the entire retained group before
attaching any outside predicate; older entry points keep their narrower
contracts. No assertion crosses a group or quantifier boundary.

For example, \b(\ba)+\B selects one initial a from aaa, while \ba+\B selects
two. Inner assertions execute on each selected character against the original
input context; outside word and line assertions apply to the selected whole
repetition. Zero counts leave inner captures undefined without executing the
child. The shared matcher retains the earliest start, greedy/lazy endpoint,
final selected captures and complete UTF-16 character boundaries.

This composition adds no search algorithm, input storage, expanded counts,
native recursion, dependencies, fixtures or default quotas. Existing leaf and
outside-predicate charges remain opt-in.


The nested assertion-group entry proves groups interleaved with word and line
predicates inside one repeated consuming atom (22.2.2.4, 22.2.2.7). After full
Pattern validation and named-group normalization, a flat edge scan counts every
ordinary/noncapturing opener and closer around the retained leaf. Every group
must contain that same one character; empty assertion groups, multiple leaves,
alternatives, concatenations and duplicate predicates remain excluded.

Word and line assertions have no capture effects and consume no characters.
Their conjunction therefore uses the established leaf order at the same input
positions, without moving any assertion across the quantifier. Prefix \b^ and
suffix $\B retain their required results. Optional outside ^/$ keep separate
whole-repetition scopes. All complete inner captures share the final selected
iteration, while zero iterations leave them undefined. For example,
(\b(a))+ selects one initial a from aaa, and ((a)\b)+ selects only the final a;
both numbered groups capture the selected character.

The scanner and retained-source copy use opted-in work charges and source-sized
storage. Matching keeps the existing monotone search and final-capture logic,
with no new recursion, expanded counts, input storage, dependency or default
quota. Outside words around these interleaved groups require another proof.


The outside-word entry for interleaved nested groups composes the existing
outside assertion scan with the complete consuming-group proof (22.2.2.4,
22.2.2.7). The whole retained body must pass that proof before any outside word
predicate is attached. Earlier entry points keep their established contracts.
Each complete inner group still contains the same one selected character, so
its capture is the final iteration; zero counts leave all inner captures
undefined without executing their predicates.

For example, \b(\b(a))+\B selects one initial a from aaa, while \ba+\B selects
two. Inner word/line assertions are evaluated per iteration in the original
input context. Outside word and line gates constrain the selected whole match.
The established earliest-start, greedy/lazy endpoint and complete UTF-16 range
logic is shared. There is no new matching algorithm, input storage, recursion,
expanded count, dependency, fixture change or default quota.


The repeated Unicode alternative proof follows Disjunction and RepeatMatcher
(22.2.2.3, 22.2.2.4) for complete ordinary/noncapturing wrappers around branches
that each consume exactly one Unicode character. Every branch must pass a
complete consuming leaf proof. Empty branches, concatenations, branch captures,
nested branch groups and nullable branch units remain excluded. Original class
and escape boundaries govern separators; branch word and line assertions retain
their input positions and original i/m/s context. Optional outside line gates
still constrain the complete repetition, rather than its branches.

For example, (\ba|b\b)+ selects the initial a in aaa, while (a|b)+ selects all
three characters. Successful branches at a position consume the same decoded
Unicode character. They introduce no branch captures or side effects, so their
left-to-right choice cannot alter a later continuation or the wrapper captures.
Every complete wrapper captures the last selected iteration; zero counts leave
these captures undefined. Existing earliest-start and greedy/lazy range logic
therefore remains applicable without a new backtracking algorithm.

The branch representation and scan are flat, with source-sized prepaid storage
and no native recursion or expanded counts. The sum of branch search work is
computed once during construction and charged safely before each candidate
character; matching work is bounded by input length times the proved branch
predicates. Earlier entry points keep their contracts. Dependencies, fixture
sources and opt-in quota defaults are unchanged.


The outside-word entry for repeated alternatives composes the existing word
assertion scan with the complete one-character alternative proof (22.2.2.3,
22.2.2.4, 22.2.2.7). It proves the entire retained repetition before attaching
any outside predicate. Every branch still consumes the same decoded input
character when it succeeds, with no branch captures or side effects. Wrappers
retain their final-iteration captures; zero counts leave them undefined.

For example, \b(a|b)+\B selects abb from abba. In \b(\ba|b)+\B on aaa, the
branch's word predicate permits only the first a, so the selected match is a.
Branch word and line assertions stay inside each iteration. Outside word and
line gates constrain the selected whole range. Earlier entry points retain
their contracts, and no predicate crosses a quantifier or capture boundary.
The established range, capture and safely charged branch logic is shared;
there is no new search algorithm, source/input storage, recursion, count
expansion, dependency, fixture change or default quota.


Shared predicates around repeated Unicode alternatives follow Disjunction and
RepeatMatcher (22.2.2.3, 22.2.2.4) without moving a branch predicate onto other
branches. The complete flat group scan records the depth of each extracted
prefix/suffix word or line assertion. A shared prefix must precede another
complete group opening; a shared suffix must follow another complete group
closing. Thus (\b(a|b))+ can gate the whole alternative per iteration, while
(\ba|b)+ keeps its predicate only on the first branch and uses the earlier
branch proof. The dedicated shared entry rejects such branch-edge extractions.

Every retained branch must pass its complete one-character proof. Captures
inside branches, nullable/multiple-character branches, empty capture groups and
other unproved compositions remain excluded. Extracted shared predicates become
a separate complete consuming character guard, with DotAll enabled solely to
permit every decoded character. That guard and the left-to-right branches
inspect the same original input position. Branch i/m/s context remains intact;
the guard preserves its original word/line context and adds no capture effects.
All complete group wrappers capture the final selected character, while zero
counts leave them undefined without executing the shared predicates.

Earlier consuming-group entry points preserve their proof contracts and charges.
The representation remains flat and source-sized. Guard and branch work is
precomputed and charged safely before each candidate character; matching adds
no new search algorithm, input storage, native recursion or expanded counts.
Dependencies, fixture sources and opt-in quota defaults are unchanged.


Outside word assertions compose with the complete shared alternative-group proof
(22.2.2.3, 22.2.2.4, 22.2.2.7). The existing word scan must leave a whole body
that passes that proof before outside predicates are attached. Branch assertions
still constrain their own branch; shared word/line guards constrain every
selected iteration; outside word/line gates constrain the selected whole range.
Earlier entries retain their established contracts.

For example, \b(\b(a|b))+\B selects the first a from aaa, while
\B((a|b)\b)+\b selects the final a. Complete inner wrappers capture that last
selected character, and zero counts leave their captures undefined without
executing their shared guards. The existing flat branch/guard representation,
earliest-start and greedy/lazy endpoints, UTF-16 ranges and safe work charges are
shared. This composition adds no search algorithm, source/input storage, native
recursion, expanded counts, dependency, fixture change or default quota.


Noncapturing branches in repeated Unicode alternatives retain their complete
one-character proof (22.2.2.3, 22.2.2.4, 22.2.2.7). A dedicated entry point
records balanced group ends using source-bounded flat storage before stripping
whole-unit wrappers. Thus the first branch's noncapturing group cannot be
mistaken for a group enclosing the whole alternative. The branch split respects
escaped characters, u/v classes and complete group boundaries.

Every complete branch must prove one unquantified consuming character. A branch
wrapper reuses the existing scoped leaf proof with an exact-one internal marker;
it must return no captures, whole-range predicates or nested alternatives before
its character/class predicate is retained. For example, ((?:\ba)|(?:b\b))+
keeps the word predicates local to their respective branches. Capturing branch
groups, empty branches, concatenations, nested choices, modifier groups and
branch quantifiers remain excluded. Earlier entry points retain their contracts.

The final plan shares the existing flat alternative representation, original
source-order predicates, cached work, greedy/lazy counts, last-character captures
and UTF-16 ranges. The leaf-proof callback type is fixed at its boundary; that
non-alternative proof cannot recursively admit choices. Construction adds no
native recursion and search adds no input-sized storage. Counts are not expanded.
No dependency, fixture or default quota changes are involved.


Outside word assertions compose with complete noncapturing one-character branches
(22.2.2.3, 22.2.2.4, 22.2.2.7). The existing word scan must retain a whole
repetition that passes the complete branch proof before outside predicates are
attached. For example, \b((?:\ba)|(?:b\b))+\B keeps each branch's word
predicates local to its character, with separate word/line gates on the whole
selected range. Noncapturing branch wrappers add no capture effects. Complete
enclosing groups retain the final selected character; zero counts leave their
inner captures undefined.

The existing branch representation, balanced whole-wrapper proof, cached work,
earliest-start and greedy/lazy endpoints, UTF-16 ranges and fallible charges are
shared. Earlier entries retain their contracts. This composition adds no search
algorithm, source/input storage, native recursion, expanded counts, dependency,
fixture change or default quota.


Line assertions before or after a complete noncapturing character branch remain
local to that branch (22.2.2.3, 22.2.2.4, 22.2.2.7). A dedicated proof mode
requires the complete branch to consume one unquantified character, contain no
captures and retain no outside word gates. Its exact-one leaf proof places the
internal marker before any proved trailing dollar assertion. Earlier entries
retain their original branch contracts.

For example, (^(?:a)|(?:b)$)+ applies the first branch's start gate and the
second branch's end gate at each attempted iteration. These gates constrain
the proved character's original boundaries, while any whole-repetition line
assertions keep their separate scopes. Each branch has a flat character/class
leaf and optional consuming DotAll line guard. Both see the same decoded input
character; only the branch leaf uses the actual dotAll flag. Cached work includes
the guard. Every enclosing capture selects the final character; zero counts skip
the branch predicates and leave those captures undefined.

The existing source-order choice scan, balanced whole-wrapper proof, linear
input search, UTF-16 ranges and fallible charges are shared. No recursive branch
representation, input-sized capture history, expanded counts, dependency,
fixture changes or default quotas are added. Word assertions outside a branch
wrapper, branch captures, nullable branches, nested choices, concatenations,
modifiers and branch quantifiers remain outside this proof.


Whole-match word assertions compose with the complete noncapturing branch-line
proof (22.2.2.3, 22.2.2.4, 22.2.2.7). The existing word scan must leave a whole
repetition that passes that proof before outside gates are attached. For example,
\b(^(?:a)|(?:b)$)+\B keeps the branches' line predicates separate from the
word predicates on the selected whole range. Branch-local word predicates remain
inside their existing consuming leaf proof; enclosing groups retain the final
selected character and zero counts leave their inner captures undefined.

The existing flat branch/guard representation, balanced wrapper proof, cached
work, earliest-start and greedy/lazy endpoints, UTF-16 ranges and fallible
charges are shared. Earlier entries retain their contracts. This composition
adds no search algorithm, source/input storage, native recursion, expanded
counts, dependency, fixture change or default quota. A whole-range capture
between the outside word gates and the repetition remains a separate unproved
composition; retained controls distinguish it from a capture enclosing the
complete asserted repetition.


Unicode word gates around complete whole-capture wrappers preserve the original
capture ranges (22.2.2.4, 22.2.2.7). After full validation and source-order name
normalization, the existing constructor wrapper scan recognizes an optional
leading/trailing word assertion alongside its existing line assertions. Escaped
suffixes remain literal. The complete transparent group proof must succeed before
the original predicates are retained around its complete body. Their order and
multiline/folding context are unchanged. The retained body still must pass an
existing supported Unicode proof; modifiers, continuations and quantified whole
wrappers are not stripped.

For example, \b(?<whole>(a|b)+)\b preserves whole as the entire selected range
and the inner capture as its final character. With a zero-iteration body, the
whole capture is empty and its inner capture remains undefined. The empty
boundary capture in (\b)\b also retains its empty range. Complete peeled
captures precede the retained body's captures in their original source order;
the existing result builder and name bindings provide whole ranges for those
wrappers. The public source remains the original Pattern.

This extends the existing constructor step without another core matcher, native
recursion, input-sized storage or expanded counts. Source storage and work remain
linear and fallible. No dependency, fixture or default quota changes are involved.


Word assertions outside a complete noncapturing Unicode alternative branch keep
their per-iteration scope (22.2.2.4, 22.2.2.7). For example,
(\b(?:a)|(?:b)\b)+ checks the start boundary only for its a branch and the end
boundary only for its b branch. These predicates remain separate from assertions
around the whole repetition and from predicates retained within each branch.

After full validation and name normalization, a dedicated compiler entry point
extracts optional branch word gates, then requires the entire retained branch to
prove exactly one unquantified character without captures. Its existing line
gates and extracted word gates become a dotAll guard on that character's original
boundaries, with the actual multiline and folding flags. The retained character
or class proof keeps its own matching predicates. Earlier entry points retain
their narrower contracts and original charges. Empty branches, continuations,
branch captures, quantified branch bodies, modifiers and nested alternatives
remain excluded from this proof.

The branch representation stays flat. Matching uses the existing work-accounted
branch scan and repetition selection without native recursion, expanded counts
or input-sized storage. Source construction and search propagate opt-in work
failures. No dependency, fixture or default quota changes are involved.


Whole-match word assertions compose with the complete Unicode noncapturing branch
assertion proof (22.2.2.4, 22.2.2.7). The word scanner retains the original
whole-match gates, then the entire body must pass the existing exact-character
alternative and repetition proof before those predicates are attached. For
example, \b(\b(?:a)|(?:b)\b)+\B separates the repetition's outside word gates
from the predicates checked for each a or b iteration. Line assertions and
assertions retained within a branch keep their original scopes and flags.

The constructor uses the existing capture summary, name bindings, result builder
and public source. Complete transparent whole-capture wrappers preserve their
whole ranges; iteration captures retain their final character or stay undefined
for zero iterations. Empty branches, branch captures, modifiers, continuations
and quantified branch bodies remain excluded from this proof. The earlier
branch-line/word entry point retains its narrower rejection contract.

This composition adds no matching algorithm, recursive representation, expanded
counts or input-sized search storage. Source and search work remain fallible,
with unchanged clone, collection and opt-in quota behavior. No dependency,
fixture or default quota changes are involved.


Exterior assertion sequences on Unicode character branches retain their
per-iteration scope (22.2.2.4, 22.2.2.7). The dedicated sequence entry point
recognizes leading word/start-line assertions and trailing word/end-line
assertions. Repeated identical predicates inspect the same original boundary
and can be combined; their order does not change this pure conjunction. Escaped
literal suffixes remain part of the retained consuming body.

The complete retained branch must still prove exactly one unquantified character
without captures. Its canonical line gates and outside word gates use the
existing per-character guard. Opposing word predicates at the same boundary
mark the fully proved branch as impossible. For example, (\b\B(?:a)|b)+ can
select only b. An impossible branch still allows the repetition to choose zero
iterations; whole wrappers then capture the empty range while iteration
captures remain undefined. Assertions inside the branch keep their own
predicates. Earlier entry points retain their narrower contracts and charges.

The sequence scan and canonical source storage remain bounded by source length.
The flat branch representation adds one impossible flag and keeps existing
matching, clone and fallible work accounting. No recursion, count expansion,
input-sized storage, dependency, fixture or default quota changes are involved.


Whole-match word gates compose with the complete Unicode branch assertion-sequence
proof (22.2.2.4, 22.2.2.7). The scanner retains the outside word predicates, then
the entire body must prove its alternatives and repetition before those gates
are attached. For example, \b(\b\b(?:a)|(?:b)\b\b)+\B separates whole-range
predicates from the combined predicates checked for each iteration. A contradictory
branch keeps its impossible flag, while zero iterations still check the outside
predicates at the selected empty range.

Complete whole captures retain their whole or empty ranges; iteration captures
retain their last character or remain undefined. The existing result builder,
name bindings and public source remain in use. Escaped dollar literals and
predicates inside branches keep their original meaning. Earlier compiler entry
points retain their narrower contracts. The composition adds no matching
algorithm, native recursion, expanded counts or input-sized search storage.
Source and search work remain fallible. No dependency, fixture or default quota
changes are involved.


Assertions interleaved with complete noncapturing Unicode branch wrappers retain
their original boundaries (22.2.2.4, 22.2.2.7). A dedicated entry point permits
the existing assertion-sequence scan to peel noncapturing openers and matching
trailing closers while combining predicates. For example,
((?:\b(?:\ba))|(?:(?:b\b)\b))+ keeps both start predicates on the a iteration
and both end predicates on the b iteration. With one consuming character and
no captures, these nested boundaries coincide.

The retained source is enclosed in one transparent noncapturing wrapper, with
canonical line gates and word predicates. Its entire body still must prove one
unquantified character without captures. Concatenation, captured branches,
nested choices, quantifiers and modifiers remain excluded. Word conflicts across
wrapper levels mark only that fully proved branch as impossible; zero iterations
keep empty whole ranges and undefined iteration captures. Escaped literal
suffixes are retained, and the earlier exterior-only mode keeps wrappers opaque.

Both scans and canonical source storage are bounded by source length. Matching
uses the existing flat branch representation, guards and fallible work accounting
without native recursion, count expansion or input-sized storage. No dependency,
fixture or default quota changes are involved.


Whole-match word assertions compose with complete nested noncapturing Unicode
branch assertion proofs (22.2.2.4, 22.2.2.7). The outside scanner retains the
whole-range predicates, then the entire retained body must prove its branch
wrappers, predicates, character alternatives and repetition before those gates
are attached. Assertions at nested branch boundaries remain predicates on each
iteration; their scope is distinct from the outside whole-match word gates.

Conflicting nested predicates preserve impossible branches. Zero iterations
still evaluate outside word gates at the selected empty range. Complete whole
captures retain empty or whole ranges, while iteration captures remain undefined
or retain the last matched character. The existing result builder, name bindings
and public source remain in use. Earlier entry points keep their narrower
contracts, and unsupported continuations, captures within branches, modifiers and
nested choices stay excluded from this proof.

This composition adds no matching algorithm, native recursion, expanded counts
or input-sized search storage. Source and search remain work-accounted and
fallible. No dependency, fixture or default quota changes are involved.


Unicode nullable repetition has a complete proof for exactly two alternatives:
one raw empty branch and one unquantified character branch under `*`, `+`, or `?`,
including lazy forms (22.2.2.3). The consuming branch must pass the existing full
character and noncapturing assertion proof without captures, concatenations,
nested alternatives or branch quantifiers. Balanced complete unit wrappers may
capture each iteration; whole captures and outside line/word gates retain their
original scope. Guarded empty branches, extra alternatives and braced counts
remain outside this proof.

RepeatMatcher rejects empty optional iterations. A nullable `+` can select one
mandatory empty iteration, so its unit captures are defined empty ranges, while
zero iterations under `*` or `?` leave unit captures undefined. A selected
nonempty run retains its final character capture. Greedy search consumes the
eligible character run; lazy search preserves the original branch order. For
lazy `+` with the character first, separate nonempty and empty searches select
the earliest start, preferring the character at a tie. Empty matches at earlier
positions must precede consuming matches found farther into the input.

The proof duplicates the pure consuming branch in a source-sized temporary
pattern to reuse complete validation, then retains the original capture count,
quantifier, empty-branch order and whole predicates. Search remains linear with
constant storage and at most two passes. Existing non-nullable plans retain their
original bounds and work charges. All work callbacks remain fallible; no native
recursion, input-sized storage, dependency or default quota is added.


Unicode nullable alternatives also admit an empty branch containing only balanced
complete noncapturing wrappers (22.2.2.3, 22.2.2.7). The source scanner proves
every opener is `(?:` and the retained suffix consists of precisely the matching
closers. Captures, assertions, nested choices, quantifiers and modifiers in the
empty branch remain excluded. The consuming branch still requires the complete
one-character proof, and exactly two branches and raw `*`, `+`, `?` remain required.

Transparent empty wrappers add no capture slots or matching effects. The existing
nullable search therefore preserves source order, mandatory empty unit captures,
zero-iteration undefined captures and outside predicates without a new matching
kernel. A new compiler entry point enables the wrapper proof; the older raw-empty
entry point retains its narrower contract and original work charges. The bounded
wrapper scans receive an explicit source-sized charge, preserve callback errors,
and add no recursion, dependency, input-sized search storage or default quota.


The complete Unicode nullable proof admits braced repetition when its lower
bound is zero (22.2.2.3). This includes exact zero, zero-to-finite ranges, open
upper bounds, padded decimal spellings and lazy forms. Every iteration is
optional, so RepeatMatcher rejects the empty branch's zero-progress path. The
existing character-run search therefore retains the original upper bound and
greediness. Zero selected iterations leave unit captures undefined; selected
characters retain the final iteration, while whole captures keep whole ranges.

The consuming branch must still prove exactly one unquantified character without
branch captures or effects. The empty branch may be raw or contain only complete
noncapturing wrappers. Exactly two alternatives remain required. Braced counts
with positive lower bounds retain distinct mandatory-empty semantics and remain
outside this proof.

A dedicated proof mode enables zero-lower-bound braces. Earlier raw-only and
simple-quantifier entry points retain their narrower contracts and work charges.
No new search or capture kernel is introduced. Decimal counts remain source
bounded and are never expanded. An upper bound too large for native indexing
cannot constrain a finite input's character run; exact finite bounds are retained.
All source/search callbacks remain fallible, with no dependency, recursion,
input-sized search storage or default quota added.


Unicode nullable repetition admits the braced open count `{1,}`, including padded
decimal lower bounds and lazy forms (22.2.2.3). Its RepeatMatcher limits are
exactly those of `+`: one mandatory iteration and no upper bound. The complete
two-branch proof therefore reuses the existing nullable search and capture
selection. Empty mandatory iterations have defined empty unit captures; selected
nonempty runs retain the final character. Lazy matching preserves the original
position of the empty branch, and earlier empty candidates precede consuming
candidates farther into the input.

A dedicated proof mode requires a lower bound of exactly one and an explicitly
empty upper-bound spelling. Finite positive-minimum ranges and lower bounds above
one remain outside this proof. Raw or balanced noncapturing empty branches and
complete pure character branches retain the existing restrictions. Earlier
zero-lower-bound and simple-quantifier entry points keep their narrower contracts
and work charges. This adds no matching/capture kernel, expanded counts,
recursion, input-sized storage, dependency or default quota.


Unicode nullable repetition admits exactly one mandatory iteration, spelled
`{1}` or `{1,1}`, including padded decimal bounds and lazy forms (22.2.2.3).
There is no optional repetition to reorder. At the earliest matching position,
the original alternative order decides whether the empty or consuming branch
wins. An empty mandatory iteration has defined empty unit captures; a consuming
iteration retains its character and original capture boundaries.

The complete proof requires exactly two branches: a raw or balanced
noncapturing empty branch and a pure, unquantified character branch. A dedicated
mode requires both quantifier bounds to equal one. Two existing monotone searches
find the earliest consuming and empty candidates, with source order breaking a
tie. Complete whole-match and character assertions remain at their original
boundaries. Earlier proof contracts and work charges are preserved. This adds
no expanded counts, recursion, input-sized storage, dependency or default quota.


Unicode nullable repetition admits positive open counts `{m,}`, including padded
decimal spellings, lazy forms and lower bounds too large for `usize` (22.2.2.3).
The complete proof still requires exactly one pure character branch and one raw
or balanced noncapturing empty branch, with captures only around the entire unit.
Mandatory empty iterations satisfy any source lower bound without expanding it.

Greedy open repetition selects the longest accepted character run. Lazy matching
with the empty branch first prefers the shortest accepted run. With the character
branch first, lazy matching tries accepted counts at or above the lower bound in
ascending order, then smaller counts in descending order. Two existing monotone
searches retain that priority and select the earliest start before comparing
paths at the same start. Complete assertions retain their original boundaries.

When the character branch is first and fewer characters than the mandatory count
are selected, the final iteration is empty and unit captures contain a defined
empty range at the match end. Otherwise, nonempty runs retain the final character.
Capture selection counts only selected UTF-16 characters, stopping at the lower
bound. An unrepresentable bound exceeds every possible finite input count. This
adds linear input work with constant search storage, no expanded source counts,
native recursion, dependency or default quota. Earlier proof contracts and work
charges are unchanged. Finite positive upper bounds remain outside this proof.


Unicode nullable repetition admits exact counts greater than one when both bounds
fit `usize`, including equal-range, padded and lazy spellings (22.2.2.3). Every
iteration is mandatory. The quantifier's greedy/lazy spelling cannot reorder the
unit's original alternatives. Character-first alternatives prefer longer accepted
runs up to the exact count; empty-first alternatives prefer shorter accepted runs.
For example, `(|a){2}` selects an empty match on `aaa`, while `(a|){2}?` selects
`aa`. Complete whole-match and character assertions retain their original scopes.

The complete two-branch proof preserves pure unquantified character branches,
raw or balanced noncapturing empty branches and captures around the entire unit.
A single existing monotone endpoint search implements mandatory branch priority.
The existing bounded capture scan preserves a defined empty final capture when
character-first matching consumes fewer characters than the mandatory count.
Source counts are not expanded; matching and capture selection add linear input
work and constant search storage. Earlier proof contracts and work charges are
unchanged. Unequal finite positive bounds and unrepresentable exact counts remain
outside this proof; no dependency, recursion or default quota is added.


Unicode nullable repetition admits unequal finite bounds `{m,n}` with a positive
lower bound when both bounds fit `usize`, including padded and lazy spellings
(22.2.2.3). The complete two-branch proof preserves a pure unquantified character
branch, a raw or balanced noncapturing empty branch and unit captures. Whole-match
and per-character assertions retain their original boundaries.

Character-first greedy repetition prefers longer accepted runs up to `n`.
Character-first lazy repetition first tries accepted counts from `m` through `n`
in ascending order, then counts below `m` in descending order. Empty-first lazy
repetition prefers shorter runs. Empty-first greedy repetition first uses the
optional iteration budget `n-m`, preferring counts from that budget down to zero.
Only then does it backtrack mandatory empty choices to characters, trying higher
counts through `n` in ascending order. Thus `(|a){2,3}` selects `a` on `aaa`,
while `(|a){2,3}$` selects `aaa`.

At most two existing monotone searches preserve that priority. Earliest starts
precede later candidates; phase order resolves ties at the same start. Existing
capture selection retains a defined empty final iteration when character-first
matching consumes fewer characters than `m`; otherwise, a nonempty run retains
its final character. Matching and capture selection add linear input work with
constant search storage, no expanded counts or native recursion. Earlier proof
contracts and work charges are unchanged. Unrepresentable finite bounds remain
outside this proof: treating a large finite upper bound as infinity would lose
the exact optional iteration budget. No dependency or default quota is added.


Unicode nullable repetition admits exact source counts larger than `usize`,
including single-count, padded, equal-range and lazy spellings (22.2.2.3).
Canonical decimal source slices prove equal bounds without a big integer or count
expansion. Empty/open upper-bound spellings and unequal overflowed bounds are
excluded. Earlier proof entry points keep their narrower contracts.

Every possible finite input has fewer characters than such an exact count.
Character-first mandatory alternatives therefore prefer descending accepted
endpoints and leave a defined empty final capture at the match end. Empty-first
alternatives prefer ascending accepted endpoints; an empty match has a defined
empty capture, and a nonempty match retains its final character. The quantifier's
greedy/lazy spelling cannot reorder mandatory branch choices.

The new proof sets only the matcher's effective branch priority and reuses its
existing monotone search and capture kernels. Original source, flags, names,
capture ordinals and clone metadata remain in the constructor. Complete pure
character and raw/balanced noncapturing empty branch restrictions and assertion
scopes are preserved. Canonical decimal comparisons charge bounded source work;
matching adds no expanded count, recursion, input-sized storage, dependency or
default quota.


RegExp.prototype.test (22.2.6.16) shares RegExpExec's live exec lookup and
Object/null result validation (22.2.7.1). After input string conversion, the
Boolean consumer recognizes only the internal builtin RegExp exec callable or
the noncallable exec fallback. Those paths use the existing RegExpBuiltinExec
matcher, lastIndex conversion and strict writes, returning a Boolean after the
same match/no-match decision instead of materializing discarded result arrays,
capture strings, named groups or d-flag index pairs (22.2.7.2).

Exact builtin dispatch retains the internal call nesting and argument guards.
Custom functions, bound native functions, borrowed receivers and functions merely
named exec retain ordinary call behavior and result validation. Getters and
coercions still run in specification order; no second exec lookup occurs.
Unsupported matching and explicit search-quota failures retain their boundaries.
Ordinary exec and other result-consuming operations keep result materialization.
This adds no dependency, matcher kernel, unsafe code or default quota.


Unicode nullable repetition admits unequal finite bounds when both exceed usize
(22.2.2.3). Validated decimal subtraction preserves a representable optional
count, including long borrow/zero runs, using constant integer storage. An
overflowing difference exceeds every finite input; it is not substituted for a
small optional count. Equal counts, open bounds and represented lower bounds
remain outside this entry point; earlier APIs retain their narrower contracts.

Character-first mandatory choices prefer descending finite endpoints and leave
a defined empty final capture, independently of the quantifier's lazy spelling.
Empty-first lazy repetition prefers ascending endpoints. Empty-first greedy
repetition first tries descending endpoints within the exact optional budget,
then ascending endpoints above it; earliest start wins, with the first phase
winning a tie. A checked increment excludes an unreachable higher phase when
the optional count is usize::MAX.

Only new empty-first plans retain the finite optional budget in nullable branch
metadata. Existing monotone search and capture kernels, complete pure-character
and raw/balanced empty-branch proofs, assertion scopes and original constructor
metadata are reused. Decimal source work is charged before subtraction. No
mandatory count expansion, native recursion, input-sized search storage,
dependency or default quota is added.


Nullable Unicode repetition also admits explicit finite braced bounds with a
represented positive minimum and an overflowing maximum (22.2.2.3). Complete
pure-character and raw/balanced empty-branch validation precedes plan creation.
The existing charged decimal subtraction retains the optional difference when
it fits usize; an overflowing difference cannot constrain a finite input.

Character-first branches reuse positive-minimum repetition and capture priority.
Empty-first branches retain the exact optional budget and reuse the two existing
monotone search phases. A native maximum minimum followed by a maximum one larger
therefore permits exactly one optional character. Open braced bounds remain outside this entry point. Simple quantifiers
reuse the existing complete body proof without decimal subtraction, preserving
earlier API contracts.

No count expansion, matcher field, kernel, native recursion, input-sized search
storage, dependency or default quota is added. Original constructor source and
flags remain intact.


Nullable Unicode alternatives admit complete sequences of unquantified empty
noncapturing groups (22.2.2.3). A flat, balanced token proof accepts only (?: and
closing parentheses, excluding captures, assertions, branches, characters and
quantifiers. Empty groups have no observable matching effects.

Private compile options separate the existing count proof from the empty-branch
proof. Earlier entry points retain their raw or nested-only empty contracts and
the same work charges. The new entry point reuses every existing count mode,
including exact and finite overflowing bounds, and the complete pure-character
branch proof. It adds no matcher field, matching/capture kernel, recursion,
input-sized search storage, dependency or default quota.

The constructor invokes this complete proof after the earlier count entry
points. Original source, names, flags, branch priority, capture ordinals and
lastIndex behavior remain intact. Empty-branch source work is charged before
the flat scan; all fallible callbacks can abort.


Nullable Unicode alternatives also prove quantified sequences of effect-free
noncapturing empty groups (22.2.2.3). A flat balanced scan permits a validated
quantifier only after a complete empty group. It excludes captures, assertions,
characters, branch choices and scoped modifiers. Every accepted path succeeds
at the same input boundary and produces no observable matching effects, whether
the inner count is zero, required, optional, greedy, lazy, open or finite.

The shared quantifier parser validates decimal syntax and bound ordering without
expanding counts, including unrepresentable finite bounds. The existing outer
count and branch-priority proofs share one dispatcher. Earlier empty-branch APIs
retain their contracts and charges; the new scan is charged before execution.
No matching/capture kernel, matcher field, input-sized search storage, recursion,
dependency or default quota is added.

The constructor tries this proof before the existing legacy BMP ignore-case
guard. Original source, flags, names, wrapper captures, Unicode boundaries and
lastIndex behavior are preserved through the established runtime paths.


Nullable Unicode alternatives also admit complete noncapturing choices whose
branches contain only effect-free empty groups and validated quantifiers
(22.2.2.3). The flat scan accepts a choice separator only within a balanced
noncapturing group. Every branch succeeds at the same boundary and produces
no captures, assertions or consuming effects, so redundant paths preserve the
established outer repetition and capture priority.

Earlier raw, nested, sequential and quantified empty-branch entry points keep
their complete contracts and work charges. The shared count dispatcher and
validated decimal parser are reused. The constructor tries the complete proof
before the existing legacy BMP ignore-case guard. Original source, flags,
names, Unicode boundaries, captures and lastIndex behavior remain intact.
No path/count expansion, matching/capture kernel, matcher field, recursion,
input-sized search storage, dependency or default quota is added.


Standalone Unicode Patterns made only of balanced effect-free noncapturing
groups, nested empty choices and validated quantifiers use the existing empty
literal matching plan (22.2.2.3, 22.2.2.5). Every path succeeds at the same
boundary with no capture, assertion or consuming effects. The shared flat group
proof retains the nullable compiler's earlier contracts and work charges; the
standalone entry point charges its scan before proving the source. Numeric
counts are validated without expansion, including unrepresentable bounds.

The constructor retains the earlier empty-capture path before trying the new
proof. Original source/flags and zero capture counts are preserved. Unicode
offsets inside a surrogate pair retain the documented leading-boundary rule.
Existing result construction, strict lastIndex writes, consumers, cloning and
tracing are reused. No recursion, matcher field, search kernel, input-sized
search storage, dependency or default quota is added.


Mandatory complete ordinary capturing wrappers around pure-empty Unicode
groups and choices preserve their original empty spans (22.2.2.3, 22.2.2.7).
The existing outer-group scan removes only complete unquantified wrappers;
the retained body must have no capture, assertion or consuming effects.
An extra transparent noncapturing proof wrapper retains the scope of any
empty alternatives exposed when enclosing noncapturing groups are removed.
Every enclosing capture is therefore defined at the whole match's empty span.

The entry point requires validated Unicode syntax and normalized named groups.
Source work is charged before the wrapper scan, temporary proof copy and capture
allocation. The existing empty literal plan stores zero-relative capture ranges;
result construction retains original names and shared indices-group aliases.
The earlier empty-capture and pure noncapturing paths are tried first and keep
their contracts. Source, flags and the documented Unicode leading-boundary rule
are unchanged. No count/path expansion, matcher field, matching kernel,
recursion, input-sized search storage, dependency or default quota is added.


Mandatory captures throughout pure-empty Unicode concatenations select the
whole match's empty span (22.2.2.3, 22.2.2.5, 22.2.2.7). A source-sized iterative
group scan excludes captures within a branch or quantified group. A group's
own capture surrounds its alternatives, while captures nested in a branch
would be conditional and remain excluded. Quantifying any group containing
captures also remains outside this proof.

Only after those effects are excluded are capturing openers projected to
noncapturing openers. The unchanged pure-empty group proof validates the
complete projected source, including exact quantifier placement, decimal
ordering and lazy modifiers. The existing empty literal plan retains one
zero-relative range per mandatory capture. Earlier entry points are tried
first and preserve their contracts.

Unicode syntax validation and named-group normalization remain prerequisites.
Source work is charged before the metadata scan, projection and capture
allocation. Assertions, characters, scoped modifiers, conditional captures
and quantified captures remain outside this subset. No count/path expansion,
matcher field, matching kernel, native recursion, input-sized search storage,
dependency or default quota is introduced.


Pure-empty capturing groups with positive repetition minima define the same
empty capture spans on every admitted iteration (22.2.2.3, 22.2.2.5, 22.2.2.7).
The iterative capture metadata proof permits quantifying a group containing
captures only when its minimum is strictly positive. Captures nested inside
alternatives and groups containing captures with zero-minimum quantifiers
remain excluded. A group's own capture still surrounds its alternatives.

A plus quantifier witnesses a positive minimum directly. A counted quantifier
must have a nonzero lower decimal; the unchanged complete projected-source
proof then validates exact syntax, decimal ordering and laziness. No count
is converted into a loop or expanded source. The existing empty literal plan
retains one zero-relative range per capture and the existing Unicode boundary
search handles execution. Names and original source/flags remain unchanged.

The earlier mandatory-only public proof shares the scan with this capability
disabled, preserving its original source charge and quantified-capture
exclusions. The new public entry charges source work before metadata, decimal
witnesses, projection and capture allocation. Unicode validation and named
group normalization remain prerequisites. No matcher field, search kernel,
recursive representation, input-sized history, dependency or default quota
is introduced.


Complete capture-free pure-empty Unicode disjunctions match the same empty span
on every path (22.2.2.3, 22.2.2.5). A flat scan checks original balanced group
scopes and witnesses a top-level alternative before allocating one transparent
noncapturing proof wrapper. The unchanged complete empty-group scanner then
validates the entire projected source, including exact decimal ordering and
lazy quantifiers. Counts and paths are never expanded.

Only empty noncapturing groups, choices and quantifiers are admitted. Captures,
assertions, characters, scoped modifiers and malformed bounds remain excluded.
The existing zero-capture Unicode empty matcher retains decoded UTF-16 start
boundaries and ordinary global/sticky state. Original source and flags remain
separate from the proof projection. Earlier public proofs keep their contracts.

Unicode validation remains a prerequisite. Source work is charged before the
scope scan, wrapper allocation and shared syntax proof. No matcher field,
search kernel, recursive representation, input-sized history, dependency or
default quota is introduced.


Unicode Bidi_Control and its exact Bidi_C alias reuse the fixed-property class
matcher (22.2.2.9, 22.2.2.8). The pinned Unicode 18 PropList supplies twelve
members in four intervals: 061C, 200E..200F, 202A..202E and 2066..2069. Their
membership has no simple/common-fold preimages outside the set, verified against
the complete pinned CaseFolding data, so u/v complement ordering agrees with i.
Five complementary intervals include lone surrogates and all supplementary
characters. Exact property aliases follow the edition-17 binary-property table.

Standalone atoms, flat unions and inversion share existing normalization,
source-sized interval storage, complete-character search and opt-in accounting.
Each fixed property is appended once even if repeated in the source. Existing
enclosing captures, word/line assertions, quantified atoms and lastIndex state
use the same paths. The final two bits of the existing property mask represent
the positive and complementary sets. Earlier property kinds retain their work
charges. No dependency, matcher field, search kernel or default quota is added.


Unicode Quotation_Mark and its exact QMark alias reuse the fixed-property class
matcher (22.2.2.9, 22.2.2.8). The pinned Unicode 18 PropList supplies thirty
members in twenty-eight raw intervals. Adjacent intervals normalize to thirteen
positive intervals and fourteen complementary intervals. Exact property aliases
follow the edition-17 binary-property table. The complete pinned CaseFolding data
shows that simple/common folding preserves membership in both directions;
therefore u/v complement ordering agrees with i for this property.

Standalone atoms, flat unions and inversion reuse complete-character matching,
normalization, source-sized storage and opt-in accounting. Repeated fixed sets
are appended once. The local class property mask becomes u32 to admit the two
new positive/complementary bits; its earlier bits and construction charges keep
their meanings. Enclosing captures, quantified atoms, word/line assertions and
lastIndex reuse existing paths. No matcher field, search kernel, dependency or
default quota is added.


Unicode Pattern_White_Space and its exact Pat_WS alias reuse the fixed-property
class matcher (22.2.2.9, 22.2.2.8). The pinned Unicode 18 PropList supplies eleven
members in five positive and six complementary intervals. This property includes
NEL, LRM and RLM and excludes BOM; its membership is distinct from both White_Space
and JavaScript's whitespace set. The complete pinned CaseFolding data verifies
that common/simple folding preserves membership in both directions.

Standalone atoms, flat unions, inversion, enclosing captures, quantified atoms
and word/line assertions reuse existing normalization and complete-character
search. The existing u32 local property mask holds the two additional bits, and
earlier property kinds retain their work charges. Repeated fixed sets are added
once with source-sized storage. No matcher field, search kernel, dependency or
default quota is added.


Unicode Logical_Order_Exception and its exact LOE alias reuse the fixed-property
class matcher (22.2.2.9, 22.2.2.8). The pinned Unicode 18 PropList supplies nineteen
members in seven positive and eight complementary intervals. The complete pinned
CaseFolding data verifies common/simple folding preserves membership in both
directions. Exact aliases follow the edition-17 binary-property table.

Standalone atoms, flat unions, inversion, enclosing captures, quantified atoms
and word/line assertions reuse existing normalization and complete-character
search. The existing u32 local property mask holds the two additional bits, and
earlier property kinds retain their work charges. Repeated fixed sets are added
once with source-sized storage. No matcher field, search kernel, dependency or
default quota is added.


Unicode Variation_Selector and its exact VS alias reuse the fixed-property class
matcher (22.2.2.9, 22.2.2.8). The pinned Unicode 18 PropList supplies 260 members
in four positive and five complementary intervals, including supplementary
selectors E0100..E01EF. The complete pinned CaseFolding data verifies that
common/simple folding preserves membership in both directions. Exact aliases
follow the edition-17 binary-property table.

Standalone atoms, flat unions, inversion, enclosing captures, quantified atoms
and word/line assertions reuse existing normalization and complete-character
search. Supplementary selectors retain their original two UTF-16 units. The
existing u32 local property mask holds the two additional bits; earlier property
kinds retain their work charges. Repeated fixed sets are added once with
source-sized storage. No matcher field, search kernel, dependency or default
quota is added.


Unicode Dash reuses the fixed-property class
matcher (22.2.2.9, 22.2.2.8). The pinned Unicode 18 PropList supplies 31 members
in 24 positive and 25 complementary intervals, including supplementary Garay
and Yezidi hyphens U+10D6E and U+10EAD. The complete pinned CaseFolding data verifies that
common/simple folding preserves membership in both directions. Exact aliases
follow the edition-17 binary-property table.

Standalone atoms, flat unions, inversion, enclosing captures, quantified atoms
and word/line assertions reuse existing normalization and complete-character
search. Supplementary hyphens retain their original two UTF-16 units. The
existing u32 local property mask holds the two additional bits; earlier property
kinds retain their work charges. Repeated fixed sets are added once with
source-sized storage. No matcher field, search kernel, dependency or default
quota is added.


Han script matching (22.2.2.9) admits the exact Script/sc property names and
Han/Hani value aliases. Unicode 18 Scripts.txt, SHA-256
0071fd81b6aeae25f6e8bce8efec3066a6476a91b49bdb2f52dc76e817862a6a,
defines 103352 members in 25 raw records, normalized into 21 positive and 22
complementary intervals. Complete pinned common/simple folding preserves
membership in both directions, so u/v complement ordering agrees under i.
The existing property class matcher stores intervals and appends each set once;
earlier property kinds and charges keep their meanings. Matching, enclosing and
final repeated captures preserve original UTF-16 text, including supplementary
members and leading-boundary lastIndex behavior. Storage scales with Pattern
source, without member or repetition expansion, new dependencies or default quotas.

Unicode 18 adds U+2B81E to Han compared with Node's Unicode 17 data. The pinned
Unicode 18 baseline remains authoritative. Shared-data results agree with Node;
the new member is verified independently against Scripts.txt. Set operators and
concatenated consuming terms remain separate implementation work. Two original unsupported Han examples now have positive
whole-program regressions; their prior gap cases use Script=Greek.


Captured Unicode property/dot concatenation (22.2.2.3, 22.2.7.2) proves exactly
one ordinary capturing group around an admitted property escape followed by
one ordinary capturing group around dot. Named groups use the existing validated
normalization and metadata; complete enclosing groups retain the whole range.
Each delegated atom proof consumes one complete original character. Dot retains
s behavior, and properties retain exact aliases and the pinned folding/complement
rules, including the documented ASCII iv difference from Node.

Search considers complete-character candidates in order. If dot fails, the next
candidate begins at the preceding property character's end, so the input prefix
is not rescanned. The two inner captures retain distinct widths, including
supplementary/lone-surrogate input and the approved leading boundary for initial
surrogate-pair offsets. Core returns two constant capture ranges. Runtime reuses
its existing final-atom range field for the first capture and derives the adjacent
second range from the whole end; no match field or capture allocation is added.
Conservative optional work covers both scans, membership/folding and a possible
one-unit rewind. Sticky search examines at most four input units. Construction
and an additional search charge may abort before state writes. Dependencies,
default quotas and other concatenation proofs retain their existing contracts.


Pattern_Syntax and its exact Pat_Syn alias use the pinned UCD 18 PropList
definition (22.2.2.9 and UnicodeMatchProperty). The 259 source records contain
2760 members and normalize to 28 positive intervals; their complement has 29
intervals. All common/simple case-fold preimages preserve membership, so both
Unicode complement orders retain these intervals. Standalone atoms, flat unions,
class inversion, captures, assertions and the existing repetition and captured
property/dot proofs share the same complete-character matcher.

Two additional tags fit the existing 32-bit property mask. Earlier tag tables,
work charges and matcher fields remain unchanged. Construction precharges the
fixed intervals and any moved source intervals; repeated property spellings add
each fixed set once. No dependency, search kernel, default quota or input-sized
search storage is introduced.


Bidi_Mirrored and its exact Bidi_M alias use the pinned UCD 18 UnicodeData
mirrored field (22.2.2.9 and UnicodeMatchProperty). The 570 members normalize to
120 positive intervals and 121 complementary intervals. All common/simple
case-fold preimages preserve membership, so both Unicode complement orders
retain these intervals. Standalone atoms, flat unions, class inversion,
captures, assertions and the existing repetition and captured property/dot
proofs share the same complete-character matcher.

Tags 30 and 31 fit the existing unsigned 32-bit property mask. The bounded tag
scan includes bit 31 and shifts only by values below 32. Earlier tag tables,
work charges and matcher fields remain unchanged. No dependency, search kernel,
default quota or input-sized search storage is introduced.

UCD 18 adds sixteen mirrored members to the UCD 17 definition used by the
installed Node and the pinned Test262 fixture. Native matching follows the
project's existing UCD 18 pin. Shared-data differential checks retain Node's
results; the additions have independent membership and native runtime tests.
The original UCD 17 fixture's complement disagrees at those new members, so
that fixture remains unchanged upstream and is not vendored.


Extender and its exact Ext alias use the pinned UCD 18 PropList records
(22.2.2.9 and UnicodeMatchProperty). The 63 members normalize to 43 positive
and 44 complementary intervals. Every common/simple folding preimage preserves
membership. Standalone atoms, flat unions and inverted classes retain their
complete-character behavior in both Unicode modes, including existing captures,
assertions, repetitions and captured property/dot patterns.

The temporary property mask is widened from u32 to u64 to hold tags 32 and 33.
Only tags below 34 are shifted or scanned. Earlier property tables, work charges
and matcher fields are preserved; mixed old/new mask checks prevent truncation
or overflow. Standalone construction charges the bounded 44-interval append;
class construction charges each distinct set before it allocates. No dependency,
search kernel, default quota or input-sized search storage is added.

UCD 18 adds U+0B54 to the Unicode 17 definition used by installed Node and the
pinned Test262 fixture. Native membership follows UCD 18. Shared-data checks
retain Node's results, and an independent native test covers the addition with
both aliases and complements. The original Unicode 17 fixture is unchanged and
remains unvendored because its complementary set still includes U+0B54.


Sentence_Terminal and its exact STerm alias use UCD 18 PropList membership
(22.2.2.9 and UnicodeMatchProperty). The 172 members use 89 positive and
90 complementary intervals. Every common/simple case-fold preimage preserves
membership, so both Unicode complement orders retain the fixed sets. Atoms,
flat unions, inversion, captures, assertions, repetitions and the captured
property/dot proof share the existing complete-character matcher.

Tags 34 and 35 fit the existing temporary u64 property mask. The bounded tag
scan stops below 36. All earlier property tables, work charges and matcher
fields remain unchanged. Standalone construction charges the bounded 90-interval
append; classes charge each distinct set before allocating. No dependency,
search kernel, recursion, default quota or input-sized storage is added.

UCD 18 adds U+2E60 and U+2E61 to the Unicode 17 definition in installed Node
and the pinned Test262 fixture. Native membership follows UCD 18. Shared-data
checks agree with Node; independent membership and native tests cover both
additions. The original Unicode 17 fixture remains unchanged and unvendored
because its complementary set still contains those two characters.


Terminal_Punctuation and its exact Term alias use UCD 18 PropList membership
(22.2.2.9 and UnicodeMatchProperty). The 293 members use 117 positive and
118 complementary intervals. All common/simple folding preimages preserve
membership; both Unicode complement orders retain these fixed sets. Atoms,
unions, inversion, captures, assertions, repetitions and captured property/dot
patterns use the existing complete-character matcher.

Tags 36 and 37 fit the existing temporary u64 property mask, whose bounded
scan stops below 38. Earlier property tables, work charges and matcher fields
remain unchanged. Standalone construction charges the bounded 118-interval
append; classes charge each distinct set before allocating. No dependency,
search kernel, recursion, default quota or input-sized storage is added.

UCD 18 adds U+2E60 and U+2E61 to the Unicode 17 definition used by installed
Node and the pinned Test262 fixture. Native membership follows UCD 18; shared
data checks agree with Node and native tests cover the additions. The original
Unicode 17 fixture remains unchanged and unvendored because its complementary
set still contains those two characters.


IDS_Binary_Operator and its exact IDSB alias use UCD 18 PropList membership
(22.2.2.9 and UnicodeMatchProperty): thirteen members in three positive and
four complementary intervals. Every common/simple folding preimage preserves
membership. Both Unicode complement orders, property atoms and class unions
use the existing complete-character matcher.

Tags 38/39 fit the existing temporary u64 property mask; the bounded scan
stops below 40. Earlier tables, charges and matcher fields remain unchanged.
Construction charges the four-interval append before allocation; classes
charge each distinct set. No dependency, recursion or default quota is added.
The pinned Unicode 17 Test262 fixture has the same full positive and negative
sets. Its unchanged provenance is reviewed separately from implementation.


IDS_Trinary_Operator and its exact IDST alias use UCD 18 PropList membership
(22.2.2.9 and UnicodeMatchProperty): two members in one positive and
two complementary intervals. Every common/simple folding preimage preserves
membership. Both Unicode complement orders, property atoms and class unions
use the existing complete-character matcher.

Tags 40/41 fit the existing temporary u64 property mask; the bounded scan
stops below 42. Earlier tables, charges and matcher fields remain unchanged.
Construction charges the two-interval append before allocation; classes
charge each distinct set. No dependency, recursion or default quota is added.
The pinned Unicode 17 Test262 fixture has the same full positive and negative
sets. Its unchanged provenance is reviewed separately from implementation.


The exact Radical property name use UCD 18 PropList membership
(22.2.2.9 and UnicodeMatchProperty): 329 members in three positive and
four complementary intervals. Every common/simple folding preimage preserves
membership. Both Unicode complement orders, property atoms and class unions
use the existing complete-character matcher.

Tags 42/43 fit the existing temporary u64 property mask; the bounded scan
stops below 44. Earlier tables, charges and matcher fields remain unchanged.
Construction charges the four-interval append before allocation; classes
charge each distinct set. No dependency, recursion or default quota is added.
The pinned Unicode 17 Test262 fixture has the same full positive and negative
sets. Its unchanged provenance is reviewed separately from implementation.


Regional_Indicator and its exact RI alias use UCD 18 PropList membership
(22.2.2.9 and UnicodeMatchProperty). Its 26 members, U+1F1E6 through U+1F1FF,
form one positive and two complementary intervals. Every common/simple fold
preserves membership. Atoms, class unions and complements use the existing
complete-character matcher and retain UTF-16 boundaries. The existing narrowly
approved leading-boundary rule also covers starts inside these surrogate pairs.

Tags 44/45 fit the existing temporary u64 mask, with a bounded scan below 46.
Earlier tables, charges and matcher fields remain unchanged. Construction
charges the two-interval append before allocation; classes charge each distinct
set. No dependency, recursion or default quota is added. The complete positive
and negative sets match the pinned Unicode 17 fixture, reviewed separately.


Unified_Ideograph and its exact UIdeo alias use UCD 18 PropList membership
(22.2.2.9 and UnicodeMatchProperty): 101997 members in sixteen positive and
seventeen complementary intervals. Every common/simple fold preserves
membership. Atoms, unions, inversion, captures, assertions, repetitions and
captured property/dot patterns use the existing complete-character matcher.

Tags 46/47 fit the existing temporary u64 mask; the bounded scan stops below 48.
Earlier tables, charges and matcher fields remain unchanged. Construction
charges the seventeen-interval append before allocation; classes charge each
distinct set. No dependency, recursion or default quota is added.

UCD 18 adds U+2B81E to the Unicode 17 set used by Node and the pinned Test262
fixture. Native membership follows UCD 18. Shared data agrees with Node, and
native tests cover both aliases and complements at the addition. The original
fixture remains unchanged and unvendored because its complement contains U+2B81E.


Ideographic and its exact Ideo alias use UCD 18 PropList membership
(22.2.2.9 and UnicodeMatchProperty): 123244 members in twenty-four positive
and twenty-five complementary intervals. Every common/simple fold preserves
membership. Atoms, unions, inversion, captures, assertions, repetitions and
captured property/dot patterns reuse the complete-character matcher.

Tags 48/49 fit the existing temporary u64 mask; the bounded scan stops below 50.
All earlier tables, charges and matcher fields remain unchanged. Construction
charges the twenty-five-interval append before allocation; class compilation
charges each distinct set. No dependency, recursion or default quota is added.

UCD 18 adds 12301 members across six ranges to the Unicode 17 set used by Node
and the pinned Test262 fixture. Native membership follows UCD 18. Shared data
agrees with Node; dedicated native tests cover endpoints and interior members
of every addition range across both aliases, complements and Unicode/case modes.
The original fixture remains unchanged and unvendored because its complement
contains those new members.


Deprecated and its exact Dep alias use UCD 18 PropList membership
(22.2.2.9 and UnicodeMatchProperty): fifteen members in eight positive and
nine complementary intervals. Every common/simple fold preserves membership.
Both Unicode complement orders, atoms, unions, inversion, captures, assertions,
repetitions and captured property/dot patterns use the complete-character matcher.

Tags 50/51 fit the existing temporary u64 property mask; the bounded scan
stops below 52. Earlier tables, charges and matcher fields remain unchanged.
Construction charges the nine-interval append before allocation; classes
charge each distinct set. No dependency, recursion or default quota is added.
The pinned Unicode 17 Test262 fixture has identical complete positive and
negative sets. Its unchanged provenance is reviewed separately from implementation.


Diacritic and its exact Dia alias use UCD 18 PropList membership
(22.2.2.9 and UnicodeMatchProperty): 1,334 members in 228 positive and
229 raw complementary intervals. U+0345 folds to nonmember U+03B9. Under
u+i, complement before folding admits all four iota equivalents in both
p and P; under v+i, complement after folding excludes them from P. The
230-interval v+i complement removes the full preimage before union, folding
and any class inversion. Captures, assertions, repetitions and property/dot
patterns retain complete-character matching and UTF-16 boundaries.

Tags 52/53 fit the temporary u64 property mask; the bounded scan stops
below 54. Earlier tables and matcher fields remain unchanged. Standalone
construction charges 230 intervals before allocation; each distinct class
set charges 920. No dependency, recursion or default quota is added.
UCD 18 adds 87 members absent from the pinned Unicode 17 fixture, which
remains unchanged and unvendored. This version difference is documented
separately from the narrowly approved inside-pair Node/V8 boundary behavior.


Soft_Dotted and its exact SD alias use UCD 18 PropList membership
(22.2.2.9 and UnicodeMatchProperty): 54 members in 36 positive and 37 raw
complementary intervals. Twelve nonmembers fold to members. Under u+i,
complement before folding admits both ends of each crossing pair in both
p and P; under v+i, complement after folding excludes them from P. The
43-interval v+i complement removes every additional preimage before union,
folding and class inversion. Captures, assertions, repetitions and property/dot
patterns retain complete-character matching and UTF-16 boundaries.

Tags 54/55 fit the temporary u64 property mask; the bounded scan stops
below 56. Earlier tables and matcher fields remain unchanged. Standalone
construction charges 43 intervals before allocation; each distinct class set
charges 172. No dependency, recursion or default quota is added. UCD 18
adds four members absent from the pinned Unicode 17 fixture, which remains
unchanged and unvendored. The new U+1DF6F member also changes folded
membership for its uppercase U+1DF6E preimage.


Grapheme_Extend and its exact Gr_Ext alias use pinned UCD 18
DerivedCoreProperties membership (22.2.2.9 and UnicodeMatchProperty):
2,269 members in 389 positive and 390 raw complementary intervals.
U+0345 folds to nonmember U+03B9. Complement before folding under u+i
admits all four iota equivalents in both p and P; complement after folding
under v+i excludes them from P. Its 393-interval v+i complement removes
the full preimage before union, folding and class inversion. Captures,
assertions, repetitions and property/dot patterns keep complete-character
matching and UTF-16 boundaries.

Tags 56/57 fit the temporary u64 property mask; the bounded scan stops
below 58. Earlier tables and matcher fields remain unchanged. Standalone
construction charges 393 intervals before allocation; each distinct class
set charges 1,572. No dependency, recursion or default quota is added.
UCD 18 adds 37 members absent from the pinned Unicode 17 fixture, which
remains unchanged and unvendored. This data-version difference is separate
from the documented, narrowly approved inside-pair Node/V8 boundary behavior.


Grapheme_Base and its exact Gr_Base alias use pinned UCD 18
DerivedCoreProperties membership (22.2.2.9 and UnicodeMatchProperty):
170,464 members in 910 positive and 911 raw complementary intervals.
Nonmember U+0345 folds to member U+03B9. Complement before folding
under u+i admits every iota equivalent in both p and P; complement after
folding under v+i excludes them from P. Its 912-interval v+i complement
removes the additional U+0345 preimage before union, folding and inversion.
Captures, assertions, repetitions and property/dot patterns keep complete
characters, original UTF-16 ranges and ASCII word-boundary semantics.

Tags 58/59 fit the temporary u64 property mask; the bounded scan stops
below 60. Earlier tables and matcher fields remain unchanged. Standalone
construction charges 912 intervals before allocation; each distinct class
set charges 3,648. No dependency, recursion or default quota is added.
UCD 18 adds 12,970 members absent from the pinned Unicode 17 fixture,
which remains unchanged and unvendored. Independent comparisons use
Unicode 17 folding for Node and Unicode 18 folding for this implementation.
This data-version difference is separate from the narrowly approved
inside-pair Node/V8 whole-match boundary behavior.


Case_Ignorable and its exact CI alias use pinned UCD 18
DerivedCoreProperties membership (22.2.2.9 and UnicodeMatchProperty):
2,890 members in 468 positive and 469 raw complementary intervals.
U+0345 folds to nonmember U+03B9. Complement before folding under u+i
admits all four iota equivalents in both p and P; complement after folding
under v+i excludes them from P. Its 470-interval v+i complement removes
the full preimage before union, folding and inversion. Captures, assertions,
repetitions and property/dot patterns retain complete characters and
original UTF-16 boundaries. Joiners and BOM are members, independent
of ASCII word membership and the RegExp whitespace escape.

Tags 60/61 fit the temporary u64 property mask; the bounded scan stops
below 62. Earlier tables and matcher fields remain unchanged. Standalone
construction charges 470 intervals before allocation; each distinct class
set charges 1,880. No dependency, recursion or default quota is added.
UCD 18 adds 96 members absent from the pinned Unicode 17 fixture, which
remains unchanged and unvendored. Independent Node comparisons use
matching Unicode 17 folding data. The production Unicode 18 pin and
the narrowly approved inside-pair Node/V8 exception remain unchanged.


Default_Ignorable_Code_Point and its exact DI alias use pinned UCD 18
DerivedCoreProperties membership (22.2.2.9 and UnicodeMatchProperty):
4,174 members in seventeen positive and eighteen complementary intervals.
Every common/simple fold preserves membership, so both Unicode complement
orders retain the same set. Atoms, classes, unions, inversion, captures,
assertions, repetitions and captured property/dot patterns preserve complete
characters and original UTF-16 ranges. Joiners and BOM are members; this
property is distinct from word and RegExp whitespace membership.

Tags 62/63 occupy the highest existing temporary u64 property-mask bits.
The bounded scan stops below 64; no shift reaches the mask width. Earlier
tables and matcher fields remain unchanged. Standalone construction charges
eighteen intervals before allocation; each distinct class set charges 72.
No dependency, recursion or default quota is added. The pinned Unicode 17
fixture has identical complete positive and negative sets; its unchanged
provenance is reviewed separately from this implementation.


Cased uses the exact ECMA property name and pinned UCD 18
DerivedCoreProperties membership (22.2.2.9 and UnicodeMatchProperty):
4,797 members in 163 positive and 164 complementary intervals.
Common/simple case folds preserve membership; u+i and v+i complements
therefore describe the same set. Atoms, classes, unions, inversion,
captures, assertions, repetitions and property/dot captures retain
complete characters and original UTF-16 ranges.

The temporary class-property mask widens from u64 to u128 before adding
tags 64/65. Its bounded scan stops below 66, preserving all earlier
property bits and interval tables without a shift at the mask width.
Standalone construction charges 164 intervals before allocation; each
distinct class set charges 656. No recursion, dependency or default
quota is added. UCD 18 adds 165 members absent from the pinned Unicode 17
fixture, which remains unchanged and unvendored. The version difference
is separate from the approved Node/V8 inside-pair boundary behavior.


Alphabetic and its exact Alpha alias use pinned UCD 18 DerivedCoreProperties
membership (22.2.2.9 and UnicodeMatchProperty): 160,254 members in 775
positive and 776 complementary intervals. Common/simple folds preserve
membership, so Unicode complement orders retain identical sets. Atoms,
classes, unions, inversion, captures, assertions, repetitions and captured
property/dot patterns retain complete characters and original UTF-16 ranges.

Tags 66/67 fit the temporary u128 property mask; the bounded scan stops below
68. All earlier tables and matcher fields remain unchanged. Standalone
construction charges 776 intervals before allocation and each distinct
class set charges 3,104. No dependency, recursion or default quota is added.
Unicode 18 adds 12,833 members in 32 ranges absent from the pinned Unicode 17
fixture, which remains unchanged and unvendored. Data-version differences
are separate from the approved Node/V8 inside-pair boundary behavior.


ID_Continue and its exact IDC alias use pinned UCD 18 DerivedCoreProperties
membership (22.2.2.9 and UnicodeMatchProperty): 162,100 members in 814 positive
and 815 complementary intervals. Common/simple folds preserve membership,
so both Unicode complement orders retain identical sets. Letters, digits,
underscore and joining controls are members; BOM is a nonmember. RegExp
word-character membership retains its separate ASCII and folding rules.
Atoms, classes, unions, inversion, captures, assertions, repetitions and
property/dot captures preserve complete characters and UTF-16 boundaries.

Tags 68/69 fit the temporary u128 property mask and the scan stops below 70.
All earlier tables and matcher fields remain unchanged. Standalone
construction charges 815 intervals before allocation; each distinct class
set charges 3,260. No dependency, recursion or default quota is added.
Unicode 18 adds 12,860 members in 38 ranges absent from the pinned Unicode 17
fixture, which remains unchanged and unvendored. Data-version differences
are separate from the approved Node/V8 inside-pair boundary behavior.


XID_Continue and its exact XIDC alias use pinned UCD 18 DerivedCoreProperties
membership (22.2.2.9 and UnicodeMatchProperty): 162,081 members in 821 positive
and 822 complementary intervals. Common/simple folds preserve membership,
so both Unicode complement orders retain identical sets. The normalization-
stable property retains its own membership tables; ID_Continue stays unchanged.
Word and identifier grammar rules retain their separate definitions. Atoms,
classes, unions, inversion, captures, assertions, repetitions and property/dot
captures preserve complete characters and original UTF-16 boundaries.

Tags 70/71 fit the temporary u128 property mask; the scan stops below 72.
All earlier tables and matcher fields remain unchanged. Standalone
construction charges 822 intervals before allocation, and each distinct
class set charges 3,288. No dependency, recursion or default quota is added.
Unicode 18 adds 12,860 members in 38 ranges absent from the pinned Unicode 17
fixture, which remains unchanged and unvendored. Data-version differences
remain separate from the approved Node/V8 inside-pair boundary behavior.


ID_Start and its exact IDS alias use pinned UCD 18 DerivedCoreProperties
membership (22.2.2.9 and UnicodeMatchProperty): 158,739 members in 695 positive
and 696 raw complementary intervals. Nonmember U+0345 folds to member U+03B9.
Complement before folding under u+i admits all Greek iota equivalents in
both p and P; complement after folding under v+i excludes them from P.
The 697-interval v+i complement removes the additional U+0345 preimage before
union, folding and inversion. Captures, assertions, repetitions and captured
property/dot patterns retain complete characters and original UTF-16 ranges.

Tags 72/73 fit the temporary u128 property mask; the bounded scan stops below
74. Earlier property tables and matcher fields remain unchanged. Standalone
construction charges 697 intervals before allocation, and each distinct
class set charges 2,788. No dependency, recursion or default quota is added.
UCD 18 adds 12,823 members in 27 ranges absent from the pinned Unicode 17
fixture, which stays unchanged and unvendored. Reference comparisons use
Unicode 17 folding for Node and Unicode 18 folding for the implementation.
Data-version differences remain separate from the approved Node/V8
inside-pair boundary behavior.


XID_Start and its exact XIDS alias use pinned UCD 18 DerivedCoreProperties
membership (22.2.2.9 and UnicodeMatchProperty): 158,716 members in 702 positive
and 703 raw complementary intervals. Nonmember U+0345 folds to member U+03B9.
Complement before folding under u+i admits all Greek iota equivalents in
both p and P; complement after folding under v+i excludes them from P.
The 704-interval v+i complement removes the additional U+0345 preimage before
union, folding and inversion. Captures, assertions, repetitions and captured
property/dot patterns retain complete characters and original UTF-16 ranges.

Tags 74/75 fit the temporary u128 property mask; the bounded scan stops below
76. Earlier property tables and matcher fields remain unchanged. Standalone
construction charges 704 intervals before allocation, and each distinct
class set charges 2,816. No dependency, recursion or default quota is added.
UCD 18 adds 12,823 members in 27 ranges absent from the pinned Unicode 17
fixture, which stays unchanged and unvendored. Reference comparisons use
Unicode 17 folding for Node and Unicode 18 folding for the implementation.
Data-version differences remain separate from the approved Node/V8
inside-pair boundary behavior.


Math uses pinned UCD 18 DerivedCoreProperties membership (22.2.2.9 and
UnicodeMatchProperty): 2,384 members in 145 positive and 146 raw complementary
intervals. Seven mathematical Greek variants fold to nonmembers. Folding
admits twelve further preimages in their case equivalence classes. Under u+i,
complementing before folding admits these families in both p and P; under v+i,
complementing after folding excludes them from P. The 158-interval v+i
complement excludes all such preimages before union, folding and inversion.
Captures, assertions, repetitions and captured property/dot patterns retain
complete characters and original UTF-16 ranges.

Tags 76/77 fit the temporary u128 property mask; the bounded scan stops below
78. Earlier property tables and matcher fields remain unchanged. Standalone
construction charges 158 intervals before allocation, and each distinct class
set charges 632. No dependency, recursion or default quota is added.
UCD 18 adds 62 members in six ranges absent from the pinned Unicode 17 fixture,
which stays unchanged and unvendored. Reference comparisons use Unicode 17
folding for Node and Unicode 18 folding for the implementation. These data
version differences remain separate from the approved Node/V8 inside-pair
boundary behavior.


Uppercase and its exact Upper alias use pinned UCD 18 DerivedCoreProperties
membership (22.2.2.9 and UnicodeMatchProperty): 2,026 members in 678 positive
and 679 raw complementary intervals. Of the 1,476 membership-changing simple folds, 1,390 map members to nonmembers
and 86 map nonmembers to members; 1,499 further preimages share these families.
Under u+i, complementing before folding admits mixed families in both p and P;
under v+i, complementing after folding excludes them from P. The 193-interval
v+i complement excludes all preimages before union, folding and inversion.
Captures, assertions, repetitions and captured property/dot patterns retain
complete characters and original UTF-16 ranges.

Tags 78/79 fit the temporary u128 property mask; the bounded scan stops below
80. Earlier property tables and matcher fields remain unchanged. Standalone
construction charges at most 679 intervals before allocation, and each distinct
class set charges 2,716. No dependency, recursion or default quota is added.
UCD 18 adds twenty uppercase members in nineteen ranges. New folding rules also
affect twenty-one older lowercase characters. The pinned Unicode 17 fixture
stays unchanged and unvendored. Reference comparisons use Unicode 17 folding
for Node and Unicode 18 folding for the implementation. These data version
differences remain separate from the approved Node/V8 inside-pair behavior.


Lowercase and its exact Lower alias use pinned UCD 18 DerivedCoreProperties
membership (22.2.2.9 and UnicodeMatchProperty): 2,740 members in 696 positive
and 697 raw complementary intervals. Of the 1,507 membership-changing simple
folds, 86 map members to nonmembers and 1,421 map nonmembers to members.
The mixed families contain 3,029 points, including membership-preserving folds
such as micro sign; 1,507 additional preimages lie outside raw membership.
Under u+i, complementing before folding admits mixed families in both p and P;
under v+i, complementing after folding excludes them from P. The 159-interval
v+i complement excludes all preimages before union, folding and inversion.
Captures, assertions, repetitions and captured property/dot patterns retain
complete characters and original UTF-16 ranges.

Tags 80/81 fit the temporary u128 property mask; the bounded scan stops below
82. Earlier property tables and matcher fields remain unchanged. Standalone
construction charges at most 697 intervals before allocation, and each distinct
class set charges 2,788. No dependency, recursion or default quota is added.
UCD 18 adds 145 lowercase members in 25 ranges. New folding rules also affect
24 older characters. The pinned Unicode 17 fixture stays unchanged and unvendored.
Reference comparisons use Unicode 17 folding for Node and Unicode 18 folding
for the implementation. These data version differences remain separate from
the approved Node/V8 inside-pair behavior.


Changes_When_Casefolded and its exact CWCF alias use pinned UCD 18
DerivedCoreProperties membership (22.2.2.9 and UnicodeMatchProperty):
1,582 members in 649 positive and 650 raw complementary intervals.
The derived property compares case folding of NFD(X) with NFD(X); matching
uses its published membership and the RegExp simple common canonicalization.
Of the 1,500 membership-changing simple folds, 1,500 map members to nonmembers
and 0 map nonmembers to members. Mixed families contain 2,972 points, with
1,472 additional preimages outside raw membership. Under u+i, complementing
before folding admits mixed families in both p and P; under v+i, complementing
after folding excludes them from P. The 153-interval v+i complement excludes
all preimages before union, folding and inversion. Captures, assertions,
repetitions and captured property/dot patterns retain complete characters
and original UTF-16 ranges.

Tags 82/83 fit the temporary u128 property mask; the bounded scan stops below
84. Earlier property tables and matcher fields remain unchanged. Standalone
construction charges at most 650 intervals before allocation, and each distinct
class set charges 2,600. No dependency, recursion or default quota is added.
UCD 18 adds 21 members in twenty ranges. New folding rules also affect twenty
older characters. The pinned Unicode 17 fixture stays unchanged and unvendored.
Reference comparisons use Unicode 17 folding for Node and Unicode 18 folding
for the implementation. These data version differences remain separate from
the approved Node/V8 inside-pair behavior.


Changes_When_Lowercased and its exact CWL alias use pinned UCD 18
DerivedCoreProperties membership (22.2.2.9 and UnicodeMatchProperty):
1,508 members in 636 positive and 637 raw complementary intervals.
Of the 1,507 membership-changing simple folds, 1,421 map members to nonmembers
and 86 map nonmembers to members. Mixed families contain 3,029 points,
including membership-preserving folds; 1,522 further preimages lie outside
raw membership. Under u+i, complementing before folding admits mixed families
in both p and P; under v+i, complementing after folding excludes them from P.
The 153-interval v+i complement excludes all preimages before union, folding
and inversion. Captures, assertions, repetitions and captured property/dot
patterns retain complete characters and original UTF-16 ranges.

Tags 84/85 fit the temporary u128 property mask; the bounded scan stops below
86. Earlier property tables and matcher fields remain unchanged. Standalone
construction charges at most 637 intervals before allocation, and each distinct
class set charges 2,548. No dependency, recursion or default quota is added.
UCD 18 adds twenty members in nineteen ranges. New folding rules also affect
21 older characters. The pinned Unicode 17 fixture stays unchanged and unvendored.
Reference comparisons use Unicode 17 folding for Node and Unicode 18 folding
for the implementation. These data version differences remain separate from
the approved Node/V8 inside-pair behavior.


Changes_When_Uppercased and its exact CWU alias use pinned UCD 18
DerivedCoreProperties membership (22.2.2.9 and UnicodeMatchProperty):
1,601 members in 653 positive and 654 raw complementary intervals.
Of the 1,476 membership-changing simple folds, 86 map members to nonmembers
and 1,390 map nonmembers to members. Mixed families contain 2,975 points,
including membership-preserving folds; 1,476 further preimages lie outside
raw membership. Under u+i, complementing before folding admits mixed families
in both p and P; under v+i, complementing after folding excludes them from P.
The 144-interval v+i complement excludes all preimages before union, folding
and inversion. Captures, assertions, repetitions and captured property/dot
patterns retain complete characters and original UTF-16 ranges.

Tags 86/87 fit the temporary u128 property mask; the bounded scan stops below
88. Earlier property tables and matcher fields remain unchanged. Standalone
construction charges at most 654 intervals before allocation, and each distinct
class set charges 2,616. No dependency, recursion or default quota is added.
UCD 18 adds 21 members in twenty ranges. New folding rules also affect twenty
older characters. The pinned Unicode 17 fixture stays unchanged and unvendored.
Reference comparisons use Unicode 17 folding for Node and Unicode 18 folding
for the implementation. These data version differences remain separate from
the approved Node/V8 inside-pair behavior.


Changes_When_Titlecased and its exact CWT alias use pinned UCD 18
DerivedCoreProperties membership (22.2.2.9 and UnicodeMatchProperty):
1,528 members in 652 positive and 653 raw complementary intervals.
Of the 1,457 membership-changing simple folds, 86 map members to nonmembers
and 1,371 map nonmembers to members. Mixed families contain 2,937 points,
including membership-preserving folds; 1,457 further preimages lie outside
raw membership. Under u+i, complementing before folding admits mixed families
in both p and P; under v+i, complementing after folding excludes them from P.
The 140-interval v+i complement excludes all preimages before union, folding
and inversion. Captures, assertions, repetitions and captured property/dot
patterns retain complete characters and original UTF-16 ranges.

Tags 88/89 fit the temporary u128 property mask; the bounded scan stops below
90. Earlier property tables and matcher fields remain unchanged. Standalone
construction charges at most 653 intervals before allocation, and each distinct
class set charges 2,612. No dependency, recursion or default quota is added.
UCD 18 adds 21 members in twenty ranges. New folding rules also affect twenty
older characters. The pinned Unicode 17 fixture stays unchanged and unvendored.
Reference comparisons use Unicode 17 folding for Node and Unicode 18 folding
for the implementation. These data version differences remain separate from
the approved Node/V8 inside-pair behavior.


Changes_When_Casemapped and its exact CWCM alias use pinned UCD 18
DerivedCoreProperties membership (22.2.2.9 and UnicodeMatchProperty):
3,078 members in 142 positive and 143 complementary intervals. All 1,533
simple common folds preserve membership, so both complement orders retain
these exact sets. Captures, assertions, repetitions and captured property/dot
patterns retain complete characters and original UTF-16 ranges.

Tags 90/91 fit the temporary u128 property mask; the bounded scan stops below
92. Earlier property tables and matcher fields remain unchanged. Standalone
construction charges at most 143 intervals before allocation, and each distinct
class set charges 572. No dependency, recursion or default quota is added.
UCD 18 adds 41 members in thirteen ranges. The pinned Unicode 17 fixture stays
unchanged and unvendored. Reference comparisons use Unicode 17 folding for
Node and Unicode 18 folding for the implementation. These data differences
remain separate from the approved Node/V8 inside-pair behavior.


Changes_When_NFKC_Casefolded and its exact CWKCF alias use pinned UCD 18
DerivedNormalizationProps membership (22.2.2.9 and UnicodeMatchProperty):
10,671 members in 871 positive and 872 raw complementary intervals. Matching
uses the published binary property, rather than normalizing input. All 1,417
membership-changing simple folds map members to nonmembers. Mixed families
contain 2,808 points, including 1,391 preimages outside raw membership.
Under u+i, complementing before folding admits mixed families in both p and
P; under v+i, complementing after folding excludes them from P. The
361-interval v+i complement excludes every preimage before union and inversion.
Captures, assertions, repetitions and captured property/dot patterns retain
complete characters and original UTF-16 ranges.

Tags 92/93 fit the temporary u128 property mask; the bounded scan stops below
94. Earlier property tables and matcher fields remain unchanged. Standalone
construction charges at most 872 intervals before allocation, and each distinct
class set charges 3,488. No dependency, recursion or default quota is added.
UCD 18 adds 88 members in twenty-seven ranges. New folding rules also affect
twenty older characters. The pinned Unicode 17 fixture stays unchanged and
unvendored. Reference comparisons use Unicode 17 folding for Node and Unicode
18 folding for the implementation. These data differences remain separate
from the approved Node/V8 inside-pair behavior.


Emoji_Component and its exact EComp alias use pinned UCD 18 emoji-data
membership (22.2.2.9 and UnicodeMatchProperty): 146 members in ten positive
and eleven complementary intervals. All 1,533 simple common folds preserve
membership, so both complement orders retain these exact sets. This binary
property consumes one complete character, including supplementary modifiers
and tag characters; it does not match whole emoji sequences. Captures,
assertions, repetitions and captured property/dot patterns retain original
UTF-16 ranges. Digits, ZWJ and variation selectors are members; underscore
and ASCII space are nonmembers.

Tags 94/95 fit the temporary u128 property mask; the bounded scan stops below
96. Earlier tables and matcher fields remain unchanged. Standalone construction
charges eleven intervals before allocation; each distinct class set charges
44. No dependency, recursion or default quota is added. The unchanged pinned
Unicode 17 fixture has identical complete positive and negative sets and is
reviewed separately from this implementation.
The static data source is https://www.unicode.org/Public/18.0.0/ucd/emoji/emoji-data.txt
with SHA-256 80d00f8e616a0ef27fd6b8de3b758c06383b5d917e2977709578e68baf733bf1.


Emoji_Presentation and its exact EPres alias use pinned UCD 18 emoji-data
membership (22.2.2.9 and UnicodeMatchProperty): 1,228 members in eighty positive
and eighty-one complementary intervals. All 1,533 simple common folds preserve
membership, so both complement orders retain these exact sets. Each binary
property atom consumes one complete character, preserving captures, assertions,
repetitions, captured property/dot patterns and original UTF-16 ranges.
This property is distinct from emoji components and does not match sequences.

Tags 96/97 fit the temporary u128 property mask; the bounded scan stops below
98. Earlier tables and matcher fields remain unchanged. Standalone construction
charges eighty-one intervals before allocation; each distinct class set charges
324. No dependency, recursion or default quota is added. UCD 18 adds nine members
in six ranges. The pinned Unicode 17 fixture stays unchanged and unvendored;
these data differences remain separate from the approved Node/V8 inside-pair
behavior. The pinned emoji-data source and SHA-256 are recorded with the static
tables and Emoji_Component implementation.


Extended_Pictographic and its exact ExtPict alias use pinned UCD 18 emoji-data
membership (22.2.2.9 and UnicodeMatchProperty): 2,830 members in 156 positive
and 157 raw complementary intervals. The circled capital M (U+24C2) is a member;
the circled small m (U+24DC) is its simple common fold and a nonmember. Under
u+i, complementing before folding admits both characters in p and P; under
v+i, complementing after folding excludes both from P. The 158-interval v+i
complement excludes the extra preimage before union and inversion. Captures,
assertions, repetitions and captured property/dot patterns retain complete
characters and original UTF-16 ranges. This binary property matches a character,
not a complete emoji sequence.

Tags 98/99 fit the temporary u128 property mask; the bounded scan stops below
100. Earlier tables and matcher fields remain unchanged. Standalone construction
charges 158 intervals before allocation; each distinct class set charges 632.
No dependency, recursion or default quota is added. Unicode 18 removes eighteen
members and adds none. The pinned Unicode 17 fixture stays unchanged and unvendored.
These data differences remain separate from the approved Node/V8 inside-pair
behavior. The pinned emoji-data source and SHA-256 are recorded with the static
tables and Emoji_Component implementation.


Emoji uses its one exact edition-17 property name and pinned UCD 18 emoji-data
membership (22.2.2.9 and UnicodeMatchProperty): 1,447 members in 150 positive
and 151 raw complementary intervals. The circled capital M (U+24C2) is a member;
the circled small m (U+24DC) is its simple common fold and a nonmember. Under
u+i, complementing before folding admits both characters in p and P; under
v+i, complementing after folding excludes both from P. The 152-interval v+i
complement excludes the extra preimage before union and inversion. Captures,
assertions, repetitions and captured property/dot patterns retain complete
characters and original UTF-16 ranges. This binary property consumes one
character, including emoji digits; it does not match entire emoji sequences.

Tags 100/101 fit the temporary u128 property mask; the bounded scan stops below
102. Earlier tables and matcher fields remain unchanged. Standalone construction
charges 152 intervals before allocation; each distinct class set charges 608.
No dependency, recursion or default quota is added. Unicode 18 adds nine members
in six ranges and removes none. The pinned Unicode 17 fixture stays unchanged
and unvendored. These data differences remain separate from the approved Node/V8
inside-pair behavior. The pinned emoji-data source and SHA-256 are recorded with
the static tables and Emoji_Component implementation.


Emoji_Modifier_Base/EBase uses exact edition-17 names and pinned UCD 18
emoji-data membership (22.2.2.9 and UnicodeMatchProperty): 136 members in
40 merged positive and 41 complementary intervals. Its membership is invariant
under simple C/S case folding, so u+i and v+i complements agree. Every match
consumes one complete character and preserves original UTF-16 ranges, captures,
assertions, repetition and captured property/dot behavior.

Tags 102/103 fit the temporary u128 property mask; the bounded scan stops below
104. Earlier tables and matcher fields remain unchanged. Standalone construction
charges 41 intervals before allocation; each distinct class set charges 164.
No dependency, recursion or default quota is added. Unicode 18 adds two members
in one range and removes none. The pinned Unicode 17 fixture stays unchanged
and unvendored; data differences remain separate from the approved Node/V8
inside-pair behavior. The pinned emoji-data source and SHA-256 are recorded with
the static tables and Emoji_Component implementation.


Emoji_Modifier/EMod uses exact edition-17 property names and pinned UCD 18
emoji-data membership (22.2.2.9 and UnicodeMatchProperty): five supplementary
members in one positive and two complementary intervals. Membership is
invariant under simple C/S case folding, so u+i and v+i complements agree.
Every match consumes one complete character and preserves original UTF-16
ranges, captures, assertions, repetition and captured property/dot behavior.

Tags 104/105 fit the temporary u128 property mask; the bounded scan stops below
106. Earlier tables and matcher fields remain unchanged. Standalone construction
charges two intervals before allocation; each distinct class set charges eight.
No dependency, recursion or default quota is added. The property has identical
membership in Unicode 17 and 18. The pinned emoji-data source and SHA-256 are
recorded with the static tables and Emoji_Component implementation.


Assigned uses its one exact edition-17 name and pinned UCD 18 General_Category
membership other than Cn (22.2.2.9 and UnicodeMatchProperty): 312,389 members
in 735 positive and 735 complementary intervals. UnicodeData First/Last ranges
include surrogate and private-use categories; noncharacters remain unassigned.
Membership is invariant under simple C/S case folding, so u+i and v+i complements
agree. Captures and repeated property/dot matches preserve complete characters
and original UTF-16 ranges.

Tags 106/107 fit the temporary u128 property mask; the bounded scan stops below
108. Earlier tables and matcher fields remain unchanged. Standalone construction
charges 735 intervals before allocation; each distinct class set charges 2,940.
No dependency, recursion or default quota is added. Unicode 18 adds 13,007 members
in 51 ranges and removes none. The pinned Unicode 17 fixture remains unchanged
and unvendored. These data differences remain separate from the approved Node/V8
inside-pair behavior. UnicodeData uses the existing Unicode 18 input and SHA-256.
Older unsupported-property examples use valid, still-unsupported Script=Latin
patterns; the corresponding unsupported-feature checks remain active.


General_Category escapes admit all 38 pinned Unicode 18 values and 240 exact
names: bare values and their gc=/General_Category= prefixes (22.2.2.9 and
UnicodeMatchPropertyValue). UnicodeData First/Last ranges retain all 30 disjoint
leaf categories; eight aggregate values include LC. Omitted code points are Cn,
surrogates are Cs, and private-use characters are Co.

The u+i complement is formed before folding. The v+i complement excludes every
preimage of the canonical positive set, including identity canonical targets
outside the raw category. Character classes and standalone escapes use the same
sets. A three-word temporary mask tracks the finite 184-kind registry, preserving
earlier properties while accommodating the new values. Construction charges
interval reservation before appending, using existing fallible opt-in work.
No dependency, recursion or default JavaScript quota is added.

Independent tests use unchanged UnicodeData and CaseFolding files at the existing
Unicode 18 pins. Node's Unicode 17 data differences are recorded separately from
the approved inside-surrogate-pair whole-match boundary behavior. Twenty-two
incompatible original Test262 category fixtures remain unchanged and unvendored.
The sixteen compatible original fixtures are reviewed for a separate commit.


Latin Script escapes admit exactly Script=Latin, Script=Latn, sc=Latin and
sc=Latn (22.2.2.9 and UnicodeMatchPropertyValue). Bare Latin names are invalid.
The pinned
Unicode 18 Scripts file defines 1,653 Latin characters in 40 merged intervals.
Its 41-interval complement includes lone surrogates. Both sets are invariant
under the reviewed simple/common case folds, preserving u+i and v+i semantics.

The finite property registry adds two kinds to the existing three-word mask.
Construction reserves and charges intervals before appending; repeated source
properties are deduplicated. Earlier general categories and binary properties
retain their kind numbers and tables. No recursion, dependency or default
JavaScript quota is added. The existing narrowly approved whole-match boundary
behavior also covers matches starting inside a Latin supplementary character.

Node uses Unicode 17. Its 161 missing Latin additions are documented separately
from matching semantics. The original Script=Latin Test262 fixture at the
existing pin is reviewed unchanged and remains unvendored because its negative
set excludes these Unicode 18 additions.


Greek Script escapes admit exactly Script=Greek, Script=Grek, sc=Greek and
sc=Grek (22.2.2.9 and UnicodeMatchPropertyValue). Bare Greek names are invalid.
The pinned Unicode
18 Scripts file defines 520 Greek characters in 37 merged intervals. Its
38-interval complement includes lone surrogates.

The non-Greek micro sign (U+00B5) and combining iota (U+0345) fold to Greek
characters. In u+i, complementing before folding can admit both positive and
negative matches. In v+i, the 40-interval complement excludes these additional
preimages before class union and the shared fold closure. Standalone escapes
and classes retain the same complement order and exact UTF-16 boundaries.

Two additional finite kinds retain the three-word temporary mask. Construction
reserves and charges intervals before appending; repeated properties are
deduplicated. Earlier general categories, binary properties and Latin keep
their kind numbers and tables. No recursion, dependency or default JavaScript
quota is added. Node's Unicode 17 data lacks two Greek additions (U+1DFF3 and
U+1DFF4); matching tests document those differences separately. The unchanged
original Greek Test262 fixture remains unvendored because its negative set
includes those newly assigned Greek characters.


Cyrillic Script escapes admit exactly Script=Cyrillic, Script=Cyrl, sc=Cyrillic
and sc=Cyrl (22.2.2.9 and UnicodeMatchPropertyValue). Bare Cyrillic names are
invalid.
The pinned
Unicode 18 Scripts file defines 508 Cyrillic characters in ten merged intervals.
Its eleven-interval complement includes lone surrogates. Common/simple case
folding preserves both sets, so u+i and v+i complement orders agree.

The finite registry adds two kinds to the existing three-word temporary mask.
Construction reserves and charges intervals before appending; repeated source
properties are deduplicated. Earlier binary properties, general categories,
Latin and Greek retain their kind numbers and tables. No recursion, dependency
or default JavaScript quota is added. Node's Unicode 17 data agrees exactly with
all Cyrillic membership and folding cases. The original Test262 fixture at the
existing pin is reviewed unchanged for a separate vendoring commit.


Arabic Script escapes admit exactly Script=Arabic, Script=Arab, sc=Arabic and
sc=Arab (22.2.2.9 and UnicodeMatchPropertyValue). Bare Arabic names are invalid.
The pinned Unicode
18 Scripts file defines 1,452 Arabic characters in 56 merged intervals. Its
57-interval complement includes lone surrogates. Common/simple case folding
preserves both sets, so u+i and v+i complement orders agree.

The finite registry fills the last two kinds of its existing three-word mask,
including the highest bit in its final word. Construction reserves and charges
intervals before appending; repeated properties are deduplicated. Earlier binary
properties, general categories, Latin, Greek and Cyrillic retain their kind
numbers and tables. No recursion, dependency or default JavaScript quota is
added. Node uses Unicode 17 data and lacks 39 Arabic characters added in the
existing Unicode 18 pin. All shared membership and folding cases agree; the
older original Test262 fixture remains unchanged and unvendored.


Hebrew Script escapes admit exactly Script=Hebrew, Script=Hebr, sc=Hebrew and
sc=Hebr (22.2.2.9 and UnicodeMatchPropertyValue). Bare Hebrew names are invalid.
The pinned Unicode
18 Scripts file defines 136 Hebrew characters in nine merged BMP intervals. Its
ten-interval complement includes lone surrogates and all supplementary points.
Common/simple case folding preserves both sets, so u+i and v+i complement orders
agree.

The finite registry adds a fourth temporary mask word for the two new kinds.
Construction reserves and charges intervals before appending; repeated
properties are deduplicated. Earlier binary properties, general categories and
Script values retain their kind numbers and tables. No recursion, dependency or
default JavaScript quota is added. Node uses Unicode 17 data and lacks the two
Hebrew characters U+05C8 and U+05C9 added in the existing Unicode 18 pin. All
shared membership and folding cases agree. The older original Test262 fixture
remains unchanged and unvendored.


Common Script escapes admit exactly Script=Common, Script=Zyyy, sc=Common and
sc=Zyyy (22.2.2.9 and UnicodeMatchPropertyValue). Bare names are invalid.
The unchanged
Unicode 18 Scripts file defines 9,276 Common characters. Merging the raw ranges yields
179 positive intervals. The raw complement also has 179
intervals; the v+i complement has 181 intervals.

The Common member U+00B5 folds to Greek U+03BC outside the raw set. Positive
ignore-case matching uses the canonical image. For u+i, negation precedes
canonicalization; for v+i it follows canonicalization. The v+i complement removes
U+039C and U+03BC preimages in addition to the raw Common set.

The existing fourth temporary mask word admits two new kinds. Construction
reserves and charges intervals before appending; repeated properties are
deduplicated. Earlier property kinds and tables are preserved. No recursion,
dependency or default JavaScript quota is added. Node uses Unicode 17 data and
lacks 153 Common characters in the existing Unicode 18 pin. Shared membership
and folding cases agree. The older original Test262 fixture remains unchanged
and unvendored.


Inherited Script escapes admit exactly Script=Inherited, Script=Zinh,
Script=Qaai, sc=Inherited, sc=Zinh and sc=Qaai (22.2.2.9 and
UnicodeMatchPropertyValue). Bare names are invalid.
The unchanged Unicode 18 Scripts file
defines 695 Inherited characters in 31 merged intervals. The raw complement
has 32 intervals; the v+i complement has 35.

Inherited U+0345 folds to Greek U+03B9 outside the raw set. Positive ignore-case
matching uses the canonical image. Negation precedes canonicalization for u+i
and follows it for v+i. The v+i complement additionally removes Greek iota
preimages U+0399, U+03B9 and U+1FBE.

Two new kinds use the existing fourth temporary mask word. Construction
reserves and charges intervals before appending, deduplicating repeated
properties. Earlier kinds and tables are preserved. No recursion, dependency or
default JavaScript quota is added. Node uses Unicode 17 data and lacks 11
Inherited characters in the existing Unicode 18 pin. Shared membership and
folding cases agree. The older original Test262 fixture remains unchanged and
unvendored.


Devanagari Script escapes admit exactly Script=Devanagari, Script=Deva,
sc=Devanagari and sc=Deva (22.2.2.9 and UnicodeMatchPropertyValue). Bare names
are invalid.
The
unchanged Unicode 18 Scripts file defines 165 Devanagari characters in five
merged intervals. Its six-interval complement includes lone surrogates. Both
sets are invariant under common/simple case folding, so u+i and v+i complement
orders agree.

Two new kinds use the existing fourth temporary mask word. Construction
reserves and charges intervals before appending and deduplicates repeated
properties. Earlier kinds and tables are preserved. No recursion, dependency or
default JavaScript quota is added. Node uses Unicode 17 data and lacks U+11B0A
in the existing Unicode 18 pin. Shared membership and folding cases agree. The
older original Test262 fixture remains unchanged and unvendored.

Bengali Script escapes admit exactly Script=Bengali, Script=Beng, sc=Bengali
and sc=Beng (22.2.2.9 and UnicodeMatchPropertyValue). Bare names are invalid.
The unchanged
Unicode 18 Scripts file defines 98 Bengali characters in 15 merged intervals.
Its 16-interval complement includes lone surrogates. Both sets are invariant
under common/simple case folding, so u+i and v+i complement orders agree.

Two new kinds use the existing fourth temporary mask word. Construction
reserves and charges intervals before appending and deduplicates repeated
properties. Earlier kinds and tables are preserved. No recursion, dependency or
default JavaScript quota is added. Node uses Unicode 17 data and lacks U+11DF0
and U+11DF1 in the existing Unicode 18 pin. Shared membership and folding cases
agree. The older original Test262 fixture remains unchanged and unvendored.

Gurmukhi Script escapes admit exactly Script=Gurmukhi, Script=Guru,
sc=Gurmukhi and sc=Guru (22.2.2.9 and UnicodeMatchPropertyValue). Bare names
are invalid.
The
unchanged Unicode 18 Scripts file defines 80 Gurmukhi characters in 16 merged
intervals. Its 17-interval complement includes lone surrogates. Both sets are
invariant under common/simple case folding, so u+i and v+i complements agree.
Construction charges intervals before appending and deduplicates repeated
properties in the existing fourth temporary mask word. Earlier kinds and tables
are preserved. No recursion, dependency or default JavaScript quota is added.

Gujarati Script escapes admit exactly Script=Gujarati, Script=Gujr,
sc=Gujarati and sc=Gujr (22.2.2.9 and UnicodeMatchPropertyValue). Bare names
are invalid.
The
unchanged Unicode 18 Scripts file defines 91 Gujarati characters in 14 merged
intervals. Its 15-interval complement includes lone surrogates. Both sets are
invariant under common/simple case folding, so u+i and v+i complements agree.
Construction charges intervals before appending and deduplicates repeated
properties in the existing fourth temporary mask word. Earlier kinds and tables
are preserved. No recursion, dependency or default JavaScript quota is added.

The Script registry completes exact Script/sc value support in the admitted
property and class plans (22.2.2.9 and UnicodeMatchPropertyValue). All 179
canonical values accept 351 distinct value spellings under Script= and sc=,
for 702 exact keys at the existing Unicode 18 pin. Existing kinds 0..205 and the
specialized Latin/Greek/Common/Inherited complement plans are retained. The remaining 167
values occupy internal u16 kinds 206..539. The shared class-property mask also
includes the separate Script_Extensions registry described below. A bounded
adapter preserves the existing u8 General_Category tables and excludes every
later Script or Script_Extensions kind from that dispatch.

The independent pinned ownership partition supplies Unknown/Zzzz for every
unlisted code point, including lone surrogates and private use. The valid
Katakana_Or_Hiragana/Hrkt value is empty. All 1,533 common/simple fold records
preserve membership in each new Script and its complement, allowing the shared
fold closure to retain exact u+i/v+i behavior. Full and Turkic folds remain
excluded. Offline generation verifies exact aliases, the partition, and fold
invariance before emitting static intervals. Construction charges the maximum
positive/complement interval bound before appending and preserves optional work
failure propagation and alias deduplication. No dependency or default quota is
added. Fixture admission is separate.

Edition 17 [UnicodeMatchPropertyValue (22.2.2.9.8)](https://tc39.es/ecma262/2026/multipage/text-processing.html#sec-runtime-semantics-unicodematchpropertyvalue-p-v) requires the values and aliases
in PropertyValueAliases.txt for Script and Script_Extensions; the early errors
in 22.2.1.1 use that same source. Unicode [UAX #44, 2.3.1](https://www.unicode.org/reports/tr44/tr44-38.html)
identifies Katakana_Or_Hiragana as a Script value with no associated characters. Node
24.19.0 with Unicode 17 rejects those values under Script= and sc= with
SyntaxError.
Spite follows the specification and admits the empty set and its full complement.
Differential checks record those Node syntax discrepancies separately, while the
pinned-data oracle and native regressions cover their complete semantics.

Script_Extensions= and scx= admit the same 179 canonical Script values and 351
exact value spellings, giving 702 keys (22.2.2.9 and UnicodeMatchPropertyValue).
For an explicitly listed code point, ScriptExtensions.txt supplies the complete
replacement set; otherwise the value is the singleton containing Script(c).
An explicit set does not retain an omitted Script fallback. Thus U+30FC belongs
to both scx=Hiragana and scx=Katakana, and not scx=Common; U+0300 has several
extensions including Latin, Greek and Coptic, and not Inherited. The 210 records
cover 741 code points. The resulting sets overlap: 1,115,175 memberships cover
all 1,114,112 code points. Unknown retains Script's missing default, including
surrogates, private use and noncharacters. Hrkt remains a valid empty set,
independent of the Hiragana/Katakana union.

Only scx=Common and scx=Greek change membership across the 1,533 pinned
common/simple folds. In u+i, a negative escape complements the raw set before
folding; in v+i it complements the canonical positive set. A negative member
inside a v+i class excludes every fold preimage before union and the shared
fold pass; outer class inversion remains separate. Common's v+i exclusion also
removes U+039C and U+03BC; Greek's also removes U+00B5. U+0345 and its target
U+03B9 both belong to scx=Greek, so scx=Inherited is fold-invariant. In
particular, \P{Script=Greek} under u+i admits U+03B9 through U+0345, while
\P{scx=Greek} does not. Full and Turkic folds remain excluded.

The extension namespace appends u16 kinds 540..897, with exclusive bound 898;
fifteen temporary u64 words deduplicate class properties. Existing property IDs,
Script/General_Category tables and specialized fold plans retain their meanings.
Generated static ranges include raw positives, raw complements and the two
additional v+i negative lists. Before an append can allocate, standalone
construction charges the maximum list length; classes charge prior ranges plus
four times that bound using saturating arithmetic. Repeated aliases share the
same kind. Existing optional work failures, UTF-16 matching boundaries and
native-stack guards remain; no dependency or default JavaScript quota is added.

The offline generator verifies the original PropertyValueAliases, Scripts,
ScriptExtensions and CaseFolding pins and reproduces both matcher modules.
ScriptExtensions-18.0.0.txt has SHA-256
5c9d34a922f687726f2a8bcf57d49f905987e51f1b21b58c95a00fbe255cec23.
Node 24.19.0 with Unicode 17 also rejects all 96 measured constructors for the
four Script_Extensions=/scx= Hrkt keys across six forms and u/ui/v/vi flags.
Those SyntaxErrors are recorded separately from matching agreement. Spite
admits the valid empty set and its complement as specified above.

Optional chains (13.3.10) use a flat list of property/call steps. Evaluate the base
once, preserve references for method receivers, and check only explicitly optional
steps for null or undefined. A nullish check skips the entire ungrouped suffix,
including computed expressions and arguments; parentheses begin a separate node.
Keep the final property reference instead of reading it, so delete bypasses its
getter and grouped calls retain this. Edition-17 GetValue/PutValue/delete continue
to defer coercibility/key conversion. Non-nullish optional calls evaluate arguments
before checking callability. Identifier/TDZ/host errors are never suppressed.

The parser preserves IdentifierName and computed In grammar, rejects chain write
and update targets, rejects ungrouped optional constructors and direct tagged
chains even across line breaks, and leaves grouped calls/construction/tags legal.
Flat steps do not consume recursive syntax depth; runtime traversal is iterative.
Host quotas remain opt-in, with no default chain, work, or heap allowance.
Private access retains its private reference and receiver through optional calls.
Async contexts await their own implementation.

The eval intrinsic is a non-constructible built-in function with standard name,
length, prototype, and global property attributes (19.2.1). PerformEval returns
non-String input unchanged before coercion, compilation, or scope creation
(19.2.1.1). This applies to direct and indirect calls, including boxed Strings.
Global declarations preserve the intrinsic value;
ordinary assignment can replace it, and deletion follows configurable-property
semantics. The retained intrinsic remains a collection root after replacement.

Indirect String eval parses a lossless UTF-16 Script and derives strictness solely
from its own directives (19.2.1.1). An environment reference named eval whose
value is the retained intrinsic selects direct eval, including parenthesized,
escaped, local, and with bindings (13.3.6.1). Property, comma, optional, bound,
call/apply, and Reflect forms use ordinary indirect dispatch. All argument effects
precede this dispatch; non-String input is never coerced.

Indirect eval creates a fresh lexical environment over GlobalEnv. Non-strict vars
and functions use GlobalEnv; strict eval uses the fresh environment for both
lexical and var declarations. EvalDeclarationInstantiation checks all global
lexical conflicts and var/function declaration permissions before mutation,
selects the last function declaration of each name, and initializes only the
selected functions. New global bindings are configurable; existing var properties
and restricted function descriptors retain their original attributes (19.2.1.3,
9.1.1.4.16–17). Eval functions capture the fresh lexical environment, including
closures retained after an abrupt completion. Lexical declarations never enter
the persistent global declaration record.

Direct String eval inherits the caller's strictness, lexical chain, and variable
environment (19.2.1.1). A fresh lexical environment holds let/const declarations;
strict eval also declares vars/functions there. Non-strict eval checks every
intervening declarative environment for var/function conflicts before creating
bindings, including TDZ, parameter, and catch bindings. With environments are
skipped without object hooks; initializer references still resolve live through
them. The core specification's catch conflict rule applies without Annex B.
New local eval var/function bindings are deletable; existing bindings keep their
mutability and deletion attributes, including mapped parameter bindings.

Function calls retain an explicit VariableEnvironment independently of nested
lexical scopes. Sloppy formal parameter expressions use a separate parameter
environment over the initial callee environment; direct eval during defaults
declares vars outside parameter bindings. FunctionDeclarationInstantiation then
creates a separate body variable environment (10.2.11). This applies to ordinary
functions and arrows. Direct eval inherits this/new.target through the nearest
non-arrow function and uses the method's retained home object for HasSuperBinding.
Return remains invalid at eval's Script level. Super properties inherit the method
home object; super outside methods and new.target outside functions are SyntaxError.

Eval returns Script completion values, replacing empty results with undefined.
The caller's scopes, variable environment, and strictness are restored on success,
JavaScript exceptions, Unsupported, and Limit. Parsing and execution share the caller's opted-in work
allowance; eval does not reset it. Source-size quotas use the same encoded-byte
accounting as Function compilation. Native call/evaluation guards still bound
recursive re-entry, and all host quotas remain opt-in.

Tagged templates parse as call/member expressions with unconverted substitution
arguments (13.3.11). Validate each tag and substitution in its surrounding strict
scope; invalid cooked escapes are permitted only for tagged components, represented
as undefined. Preserve raw UTF-16 text and normalized line endings. The shared
component allocation identifies a template Parse Node across Script/function
syntax clones; each separate parse creates new sites, including identical source.

GetTemplateObject (13.2.8.4) uses a per-realm map of these sites to frozen intrinsic
Arrays, with a frozen raw Array and exact data descriptors. Trace cached Arrays
through host collection, independently of user bindings or prototype links.
Template construction bypasses public constructors, setters, and species. Cache
only completely constructed objects; host allocation/quota failures expose no
partial template and skip JavaScript handlers. Call evaluation reads the tag and
its receiver before creating the template or evaluating substitutions, and checks
callability after all arguments. Substitutions retain their original values.
Host quotas remain opt-in; no default cache, heap, or execution limit is imposed.

Use generated, versioned Unicode tables for identifiers. The current tables use
Unicode 18.0.0, with a pinned source digest and reproducible generation. Rust's alphabetic predicate
is not ECMAScript's ID_Start or ID_Continue. Numeric conversion, separators, escapes,
and line terminators need focused boundary tests.

Parsing returns an owned, inspectable syntax tree or a diagnostic with a source
span. A separate validation step can handle non-local early errors. Finish all
parsing and early-error checks before executing any part of a Script or Module.
Parser depth and opted-in evaluator work limits report host resource errors, not JavaScript
exceptions. Syntax or semantics that are not implemented must be recorded as gaps.
Never use unsupported syntax rejection as evidence of conformance to negative tests.

Synchronous for-of supports reference targets and single var/let/const identifiers
(14.7.5). Parse the RHS as AssignmentExpression with In enabled and validate header
initializers, targets, lexical conflicts, labels, and literal contextual keywords.
Complete header parsing before recursively parsing the body to keep native frames
small. Lexical names are uninitialized during RHS evaluation; restore the outer
environment before iterator acquisition. Capture next once, then read each value
before evaluating its assignment target. Both let and const receive a fresh
environment per iteration. Normal exhaustion and matching continue do not close;
break, return, nonmatching control transfers, and body/assignment throws use
IteratorClose after restoring the outer environment. Cleanup errors replace
non-throw completions; incoming throws override cleanup language errors. Iterator
step failures propagate without closing. Host failures stop without executing
cleanup, and host errors during cleanup remain host errors. Destructuring and
async iteration remain separate steps.

For-in uses the same reference/var/lexical iteration bindings (14.7.5). Nullish
inputs produce no iterations; box other primitives without coercion hooks. Walk
the prototype chain iteratively, snapshotting each object's ordered own keys when
reached. Ignore Symbols, check own descriptors live, and visit string names only
once. Deleted own keys do not suppress inherited keys; present non-enumerable keys
do. No property values or user iterator hooks are read. Assignment targets follow
key retrieval, and lexical bindings receive fresh iteration environments. RHS
comma expressions are allowed, unlike for-of. Host quotas remain opt-in. Chains
that reach an incomplete intrinsic report Unsupported, preserving prior effects;
The complete String prototype property inventory uses ordinary ordered enumeration.
The Annex B initialized-var extension remains Unsupported in non-strict code and
is a SyntaxError in strict code; it never changes the core binding algorithm.

Arrow parsing supports non-async arrows with binding parameters, optional
defaults, and assignment-expression or block bodies (15.3). A bounded token lookahead refines the
parenthesized parameter cover without changing ordinary parenthesized expressions.
Parameters are unique in both modes, strict binding rules are inherited, and no
line terminator may precede the arrow. Expression bodies inherit the In grammar
parameter; block bodies reset In and enable Return. Each function resets control
targets and labels. Its directive prologue enables strict parameter/body checks,
including legacy tokens before the directive and in nested functions. An own Use
Strict Directive is forbidden with non-simple parameters, while inherited strictness is allowed.
Defaults parse as AssignmentExpression with In enabled. Top-level
lexical declarations cannot conflict with any parameter BoundName (15.3.1).
Async arrows remain an explicit gap. Function source ranges share an owned source allocation and preserve exact
text for Function.prototype.toString. Arrow instantiation captures the current environment identity and strictness.

Ordinary function expressions and declarations share the binding parameter and
function-body parser with arrows (15.2). Names, parameters, bodies, and source text are retained.
Only simple lists in non-strict ordinary functions permit duplicate parameters;
strict or non-simple lists require unique names. A function's own strict directive
also constrains its optional name, parameters, and nested code. Function
expressions may appear in call/member positions, and nested bodies reset control
targets. Function heads and bodies each charge parser depth; declarations cannot
bypass expression recursion limits. Ordinary functions instantiate; strict calls
and non-strict calls with object, global, Boolean, or Number receivers execute. Ordinary
construction also executes; other primitive wrappers remain a runtime gap.
Generators and async functions remain separate steps.

Rest parameters accept identifiers or nested patterns and are final, with no
initializer or trailing comma (15.1). They collect the remaining internal argument values into a fresh dense
intrinsic Array, without invoking JavaScript iterators, setters, species, or the
global Array binding. A rest pattern then applies ordinary BindingInitialization
to that Array, including observable iterator hooks for an array pattern.
Function length counts whole patterns and stops before the first top-level default or rest
parameter. Rest makes a parameter list non-simple, requiring unique names,
rejecting an own Use Strict Directive, and selecting unmapped arguments. Parameter
expressions, including nested defaults and computed keys, separately determine whether body vars need their own environment
and whether body declarations can suppress the arguments binding (10.2.11).
All parameters, including rest, begin uninitialized; earlier defaults can capture
rest but cannot read it before initialization. Rest arrays share object identities
with supplied arguments while their element assignments remain independent.

New expressions retain the constructor and optional argument list (13.3.5).
Constructor parsing consumes member access but leaves call parentheses to the new
expression, distinguishing `new F.x(a)` from `new F(a).x`. Nested new forms bind
inner argument lists first. Calls and construction share argument parsing with In
enabled; spans, strict validation, and depth budgets cover all children. Derived
construction remains a separate gap. NewTarget is permitted in ordinary
function parameters/bodies and arrows nested within them; the permission crosses
arrow boundaries but is restored after each ordinary function (16.1.1). Script
and arrow-only occurrences are Syntax errors before execution. Escaped grammar
terminals and assignment/update targets remain invalid.

Direct function declarations are var-scoped in Scripts and function bodies;
block and switch declarations are lexical (8.2.6, 8.2.8). Scope validation checks
function names against lexical and nested var declarations, including catch
parameters. Duplicate block functions and functions in bare statement/label
positions do not receive Annex B exceptions. AST inventories expose direct
functions separately from var bindings without descending into nested functions.
Entering a block initializes its function bindings before executing statements.
Script/function bodies instantiate only the last declaration of each function
name. Global declaration conflicts are checked before creating bindings; global
functions cannot replace undefined, NaN, or Infinity (16.1.7). Evaluating a function
declaration then produces an empty completion (15.2.6).

## Runtime

Start with a tree-walking evaluator. Keep evaluation order explicit. Implement
abstract operations near their specification names and link subtle algorithms to
stable section anchors. Use binary64 Numbers, preserve negative zero and NaN, and
keep Number and BigInt distinct. Do not use Rust equality as a substitute for the
language's separate equality algorithms.

BigInt uses a sign and a normalized little-endian vector of 32-bit magnitude
words. Zero has no words and no negative sign. Start with straightforward carry,
borrow, multiplication, and division algorithms. Each arithmetic operation takes
a budget that bounds result bits and optionally charges word operations before
doing the work. The runtime translates budget exhaustion into a host limit, while division
by zero and negative exponents become JavaScript RangeError exceptions.
Comparisons between BigInt and Number must compare mathematical values without
rounding the integer first. Literal grammar, string coercion, and mixed-type
operator rules remain in the parser and runtime rather than the arithmetic crate.
The AST stores validated BigInt digits and their radix. Evaluation converts them
under the realm's budget, so parsing never performs unbounded integer arithmetic.
The evaluator shares any opted-in step budget with integer arithmetic and conversion and
can also opt into a BigInt magnitude-bit quota. BigInt/Number comparisons
inspect the binary64 significand and exponent without rounding the integer.
The integer library also supplies explicit conversion to binary64 for the Number
constructor (21.1.1.1). It retains the leading 53 bits and rounds once using guard,
sticky, and parity bits; halfway results choose the even significand. Rounding
can carry into the exponent or overflow to signed infinity. Conversion allocates
no intermediate integer and charges work before examining its bits.
ToNumber is fallible; it rejects BigInt with TypeError. Language ToString produces
decimal digits. Host value display uses exact hexadecimal BigInt notation to keep
diagnostic formatting linear and independent of the evaluator's remaining budget.

BigInt (21.2.1.1) rejects construction before argument coercion, then performs
ToPrimitive with the number hint. Integral finite Numbers use exact significand/
exponent decoding (NumberToBigInt, 21.2.1.1.1); other values use ToBigInt (7.1.13).
ToBigInt accepts BigInt, Boolean, and integer strings, rejects Numbers and other
incompatible primitives with TypeError, and rejects malformed strings with
SyntaxError. Share StringToBigInt with equality/ordering conversion.
BigInt.prototype is ordinary with no BigIntData and a configurable, non-writable
BigInt tag. Wrappers retain an immutable integer slot, independently of their
current prototype. ToObject and non-strict calls box BigInts through that intrinsic.
Branded valueOf/toString/toLocaleString reject impostors without coercion.
ToString validates the brand before radix conversion, uses exact integer formatting,
and checks opted-in output quotas. ToLocaleString follows the non-ECMA-402 decimal
fallback, ignoring reserved arguments. BigInt.asIntN/asUintN (21.2.2.1–2)
finish ToIndex on the width before ToBigInt on the value, even for zero width.
Reduce modulo 2^bits by copying low words and masking the partial high word;
negative inputs use two's complement, and signed outputs interpret the sign bit.
Do not construct the modulus. Values that already fit return a checked copy
without allocating for the requested width. Check opted-in output/work quotas
and platform allocation capacity; impose no default width or magnitude quota.

Model references separately from values. Model normal completion with an optional
value so an empty completion cannot be confused with JavaScript undefined. Add
return, throw, break, and continue completion records as those forms arrive. Keep
engine failures such as unsupported functionality and resource exhaustion separate
from catchable JavaScript exceptions.

Try-finally evaluates the finalizer after either a normal or abrupt language
completion. A normal finalizer preserves the protected completion; an abrupt
finalizer replaces it (ECMA-262 14.15.3). Host failures abort evaluation without
running further JavaScript, including pending finalizers. They cannot be caught
or suppressed by language control flow. Ordinary lexical scopes are restored
on every exit, including a host abort.

A catch parameter creates a mutable declarative binding in an environment outside
the catch block's lexical environment. Restore both before a finalizer or outer
handler runs. Identifiers bind thrown values, including object identities,
directly. Built-in exceptions materialize as Error instances when a binding needs
their JavaScript value; catch clauses without a parameter need no such value. The optional
Annex B rule permitting var to redeclare a catch parameter is not enabled.

Catch parameters share binding-pattern syntax for object properties, computed
keys, shorthand, array elisions, nested defaults, and rest targets (14.3.3).
BoundNames are collected in source order; duplicates and conflicts with the
catch block's lexical/var declarations are early errors. Every name begins
uninitialized in the catch environment before evaluating keys or defaults.
Defaults run only for undefined values, and identifier defaults use named
evaluation for anonymous functions. Object properties use GetV with their
original primitive/object receiver. Object rest uses shared CopyDataProperties
with a hash set of exact excluded String/Symbol keys, an intrinsic ordinary
result, snapshot own keys, and live enumerable descriptors/values. Array patterns
acquire a synchronous iterator and cache next once; elisions skip value getters.
Step/value errors mark the iterator done. Partial normal consumption and binding
failures close active iterators with the standard completion precedence; nested
iterators close from inside out. Rest Arrays use the retained intrinsic prototype
and own data definitions. Binding recursion shares the existing evaluator
native-stack guard across callback reentry. Host aborts restore the catch scope
and skip JavaScript cleanup.

Let/const declarations use the same BindingElement/BindingPattern syntax and
BindingInitialization (14.3.1). Every bound name participates in lexical,
parameter, catch, and nested-var early conflicts; property keys are not names.
Patterns require an initializer outside for-in/of declarations, and lexical
BoundNames cannot contain let. Declaration instantiation creates all names before
evaluating any initializer. Identifier initializers receive named evaluation;
pattern initializers do not. Failed initialization preserves already initialized
bindings and leaves later names in the temporal dead zone. Const bindings remain
immutable. Three-clause let loops copy every bound name before the body and
update; const loops retain one environment. For-in/of RHS evaluation shadows all
names with uninitialized bindings, then each iteration creates and initializes a
fresh environment. Binding failures restore scopes and close active for-of
iterators with standard precedence. Empty patterns have no per-iteration names.
Var declarations also use BindingElement/BindingPattern syntax (14.3.2).
VarDeclaredNames collect every nested bound identifier, including repeats, while
excluding nested functions. Script/function instantiation preinitializes the
names to undefined; repeated targets assign the same binding in source order.
Identifier declarations resolve before their initializer; pattern declarations
evaluate their RHS before BindingInitialization with an undefined environment.
This assignment mode uses ResolveBinding and PutValue. Single-name object
elements resolve after key conversion and before GetV; array elements resolve
before stepping even an exhausted iterator. Rest identifiers resolve before
copying or consuming their source. A nested pattern resolves its own names when
it is reached. Retaining those references preserves with/unscopables lookup
order and binding-object identity across getters, defaults, and iterator hooks.
Writes preserve strict errors, property setters, and mapped parameter aliases;
abrupt binding writes close active iterators with incoming-throw precedence.
Var loops retain one variable environment. Only identifier initialized for-in
declarations are an optional Annex B form; initialized patterns are syntax errors.
Formal parameters reuse these patterns with all names instantiated before any
initialization. Duplicate names are permitted only in simple, sloppy ordinary
lists; those bindings begin at undefined and use assignment initialization.
Every pattern makes the list non-simple and selects unmapped arguments, even
when it contains no expressions. ContainsExpression traverses nested defaults
and computed keys independently of top-level HasInitializer and ExpectedArgumentCount
(15.1).

AssignmentPattern has its own AST for identifier/member references and nested
patterns (13.15.5). Refine an object/array cover only when the complete outer
literal is followed by assignment or a for-in/of delimiter. Ordinary literals
retain their own grammar. Rest must be final without a comma or initializer;
array rest may target another pattern, while object rest requires a reference.
Repeated names are allowed. Strict targets and computed/default expressions
receive ordinary early-error validation. Assignment evaluates the RHS first and
returns its original value. Synchronous loop assignment heads use the same walker.
Keyed assignment converts the source key, evaluates a simple target reference,
reads GetV, evaluates an undefined-only default, then writes PutValue. Nested
patterns evaluate their targets after that source read. Array references evaluate
before IteratorStepValue, including after exhaustion; elisions skip value reads.
Rest references evaluate before copying or consuming the source. Edition-17
EvaluatePropertyAccessWithExpressionKey retains a computed target's raw key;
RequireObjectCoercible and ToPropertyKey occur at PutValue after source/default
effects (13.3.3.1, 6.2.5.6). Retaining references preserves base-object and resolved
environment identities across mutations. Anonymous defaults infer names only
for IdentifierReference targets. Object rest uses ordered CopyDataProperties with
exact String/Symbol exclusions and own data definitions; array rest builds a
dense intrinsic Array before nested assignment. Partial writes survive later
failures. Active iterators close on normal completion and language throws with
incoming-throw precedence; failed steps mark completion and skip closing. Host
aborts skip JavaScript cleanup. Nested patterns share the native-stack guard;
flat patterns retain ordinary unlimited defaults.

An engine owns realms, environments, execution contexts, and the object heap.
Lexical bindings distinguish uninitialized from undefined and preserve mutability.
Declarations are instantiated before evaluation. Closures retain environment
identities. Global object bindings and lexical bindings have distinct semantics.

Declarative environments now occupy entries in the same generational heap as
objects. Each has an explicit outer handle and a binding map. Active scope stacks
hold handles; lexical references retain the resolved environment identity instead
of a stack index. Name lookup follows outer links with bounded work, preparing for
closures that execute under a different caller. Per-iteration let environments
copy bindings into a fresh identity with the same outer link (14.7.4.4).

When execution-work and heap-slot quotas are both disabled, a three-clause let
loop may retain its first iteration record for subsequent iterations. This
requires initialized Number-valued identifier bindings and a complete positive
syntax check that excludes captures, direct eval and unknown forms. Every write
to a header binding must be increment/decrement or assignment of a Number
literal. The check inspects initializers, test, update, body and computed keys;
scratch allocation failure selects the original path. Initialization, the first
copy, test/body/update order and completion handling remain unchanged. Opted-in
execution-work or heap-slot quotas retain every original environment copy.

The Number restriction prevents omitted records from owning historical object
or Symbol values. Existing outside functions and property callbacks use their
saved lexical contexts. Adding active-Realm host callbacks or ambient caller
environment access requires reviewing this optimization. Allocation counts,
opaque heap identities, Debug output and collection live/reclaimed/work costs
can differ; finite collection-work results are not an equivalence guarantee.

Non-strict with statements evaluate and box their expression before entering an
Object Environment Record (14.11). Its binding object is traced alongside its
outer environment. Name lookup checks own and inherited properties regardless of
enumerability, then reads Symbol.unscopables and the named exclusion using ordinary
Get semantics (9.1.1.2.1). Missing properties skip these observable reads. Resolved
object bindings retain their original object identity through assignment RHS
effects; GetBindingValue and SetMutableBinding recheck property existence without
repeating the unscopables lookup. Strict closures may capture these environments:
disappeared bindings throw ReferenceError, while rejected strict writes throw
TypeError (9.1.1.2.5–6). Identifier calls, optional calls, and tagged calls use the
binding object as their implicit receiver. The record supplies neither this nor
new.target, and dynamically constructed Function bodies retain global scope.
Typeof resolves an identifier only once. Every completion restores the outer
scope; empty body completions become undefined. Strict with statements and bare
function/lexical declarations are syntax errors; Annex B extensions are excluded.

Realm initialization runs on first evaluation, after parsing succeeds. It creates
the global lexical environment, implemented intrinsics, and an ordinary global
object whose prototype is Object.prototype (9.3.1). This fixed graph runs no user
code and is outside the per-Script work allowance. Script evaluation has no work limit by default;
hosts can set `Limits.max_steps` to `Some(units)` to opt into a per-Script budget.
`None` disables work accounting across evaluation, property operations, string
scans/copies, and integer arithmetic. Work units measure implementation operations,
not elapsed time or JavaScript statements. Every `Limits` field defaults to `None`:
source bytes, string code units, BigInt bits, argument counts, heap slots, and own
properties have no host-selected quotas until an embedder supplies `Some(limit)`.
Checked size arithmetic, fallible reservation, and handle validation remain active.
The parser/evaluator native-stack guards are current implementation constraints;
removing them safely requires replacing recursive execution with explicit frames.
References below to execution
work budgets apply when a host enables this option. The host
slot limit is named `max_heap_entries` and counts both objects and environments.
Temporary scopes remain allocated until explicit collection; scope restoration
changes active roots on every normal or abrupt exit. The collector traces outer
links and every binding, charging scans even for uninitialized/primitive values.
Checked object access rejects an environment handle and vice versa. Function records
capture these environment handles. Global object bindings use the actual property
store, while global lexical bindings remain in the declarative environment.

The host uses its ordinary global object as the global this value. `globalThis`
is a writable, non-enumerable, configurable property; deleting or replacing it
never changes the realm's this value. Script this, including strict Scripts, and
arrows capturing global this use that identity. Non-strict ordinary calls replace
undefined/null receivers with it; strict calls preserve their receiver (10.2.1.2).
Global identifier reads, writes, and deletion follow property descriptors and
prototype lookup. Accessors receive the global object. Already-resolved references
recheck presence after RHS side effects, throwing ReferenceError in strict code if
the property disappeared (9.1.1.2.5–6).

Global declaration preflight uses HasRestrictedGlobalProperty, CanDeclareGlobalVar,
and CanDeclareGlobalFunction before creating bindings (9.1.1.4.13–17, 16.1.7).
New Script vars/functions are non-configurable; var preserves existing properties,
including accessors and configurability. Function declarations reconfigure existing
configurable properties and can replace writable enumerable fixed data properties.
Edition 17 has no separate global declared-name list. Missing standard globals stay
explicit host gaps on access, with known property presence; no host extensions are
installed. The global object is a persistent collector root even after globalThis
is deleted. A fully initialized realm currently retains eleven heap entries.

Object support starts with a separately tested `spite-heap` foundation before
object values become visible to JavaScript. Objects use opaque arena handles,
not unsafe pointers or reference-counted object cycles. A handle carries a slot,
a generation, and a shared identity token. The token uses safe `Rc::ptr_eq`; it
contains no object data or graph edges and does not make the object a GC root.
This makes heaps single-threaded, permits moving a heap without invalidating its
handles, and rejects cross-heap handles without a global identity counter.

Slots have an optional host-configured upper bound. Reuse increments the
generation; a generation that cannot increment retires its slot permanently.
Every access checks heap identity, generation, and occupancy. Heap-owned values
cannot be reached through stale handles, including after slot reuse.

Add a non-moving mark-and-sweep collector with caller-supplied roots and iterative
edge traversal. Charge collection work before each scan or edge traversal. If
tracing sees a foreign/stale handle or exceeds its budget, return without sweeping.
Trace iterators yield an optional handle for each inspected field, including
primitive-valued fields, so a scan cannot evade its budget by filtering out all
non-reference properties. Deep object graphs use an explicit work stack.
Reachable containers can additionally enumerate weak-key ephemerons. Waiting
values are indexed by validated key slots and activated once when those keys
become reachable. Activated values can expose further keys and containers;
unreachable conditional cycles do not root themselves. Stale weak keys are
ignored across generation reuse. Foreign keys and invalid active values abort
before sweeping. Ephemeron inspections and activations consume caller-selected
collection work; scratch and sweep free-list allocations are checked before
removal. Object-key WeakMap JavaScript methods use this ephemeron path; weak
Symbol-key reachability remains an explicit Unsupported boundary.
After marking reaches its fixpoint, reachable containers can remove entries
whose keys were not retained. Cleanup work is prepaid during enumeration, and
validated key generations prevent reused slots from preserving stale entries.
Cleanup performs no allocation or graph additions and runs only after every
fallible check. This also releases primitive values physically owned by inactive
entries; an unrooted heap handle alone would not release those Rust values.
Allocation initially never invokes collection implicitly. Before integrating
collection into evaluation, explicitly root environment bindings, intrinsics,
pending completions, suspended frames, expression temporaries, and host-held
values. No allocation-triggered collection or JavaScript-visible GC hook may run
until those root lifetimes are implemented and tested.

Property descriptors, key ordering, prototypes, internal methods, and exotic
objects are semantic requirements. Add ordinary objects first. Implement arrays,
functions, proxies, typed arrays, and other exotic objects on explicit internal
method boundaries rather than ad hoc evaluator branches.

Ordinary records store string/symbol properties in a creation-ordered
vector. Partial descriptors preserve omitted fields, enforce non-configurable
and non-writable invariants, and use SameValue. A successful equivalent-value
definition on a frozen property preserves its stored value, including NaN bits
(10.1.6.3). Enumeration sorts array indices numerically before other strings in
creation order, then symbols in creation order. Deletion followed by re-creation
gives a non-index key a new position within its group.
Property capacity failures remain distinct from descriptor rejection. Storage
exposes checked internal operations; realms supply JavaScript execution and
exception semantics. Realm Symbol hooks remain a separate implementation boundary.

Symbol integration proceeds in three layers (6.1.5, 6.1.7, 7.1.19). First,
spite-core owns immutable JsSymbol identities and a PropertyKey enum that
distinguishes UTF-16 strings from symbols. A symbol owns an Arc containing only
its optional description. Equality and hashing use allocation identity, never
the description. Clones retain identity without copying text; independently
created symbols remain unequal even when both descriptions are absent or equal.
This requires no unsafe code, global counter, or new dependency. Symbols cannot
point to objects, so reference counting cannot create a symbol/object cycle.

Ordinary/exotic storage and work accounting now use PropertyKey. Own
keys must enumerate numeric string indices first, then other strings in creation
order, then symbols in creation order. Symbol keys never trigger Array length,
String index, mapped-argument, or global binding behavior merely because their
description resembles a string key. Descriptor rules, inherited accessors, and
tracing of property values apply to both key kinds. Charge symbol comparisons
and clones as constant work, while string keys retain UTF-16 work accounting.
PropertyKeyRef borrows either key kind without allocating a lookup copy. Storage
accepts borrowed keys for reads and owned keys for mutations; existing string
callers use the same operations. Realm Get/Set/Delete/HasProperty, descriptor
reflection, Object.assign, integrity operations, and own-key enumeration preserve
both key kinds. ToPropertyKey uses the string hint and preserves resulting Symbol
identity. Deferred reference conversion caches the converted identity for updates.
Anonymous function names use bracketed descriptions, with an empty name only for
an absent description (10.2.9); formatting is bounded before allocation.

Runtime values now preserve Symbol identity, truthiness, typeof, equality, and
abrupt numeric/implicit string conversions. String called with a primitive Symbol
uses bounded SymbolDescriptiveString and preserves its UTF-16 description;
String construction still throws TypeError (20.4.3.3.1, 22.1.1.1).
The edition-17 well-known identities live in a fixed, process-shared OnceLock
table, separate from fresh symbols and the registry. Initialization runs
no user code, and each lookup clones only an Arc. Native-injected values and
Script-visible Symbol calls exercise these algorithms.

ToPrimitive now looks up the shared Symbol.toPrimitive identity using GetMethod
semantics: inherited accessors retain the original receiver; only null/undefined
fall back to OrdinaryToPrimitive. Calls receive the exact default/string/number
hint, and object results throw TypeError without falling back (7.1.1).

Symbol wrappers own immutable SymbolData independently of their current prototype.
Symbol.prototype is ordinary and has no SymbolData; branded methods and the
description getter reject it and lookalike objects without coercion (20.4.3).
ToObject retains symbol identity without copying descriptions. Prototype valueOf,
toString, description, Symbol.toPrimitive, and Symbol.toStringTag have their standard
attributes. Description and descriptive-string output are bounded before copying.
The intrinsic Symbol callable creates fresh identities after description ToString;
undefined preserves an absent description. It has length zero and no Construct
method, and its 13 well-known properties are fixed identities (20.4.1–2).

The append-only GlobalSymbolRegistry is shared across realms (20.4.2.2/6).
A process-wide Mutex protects a vector of registered identities. Each
identity's immutable description is its registry key, avoiding a second text copy.
Perform ToString before locking; lookup/insertion under the lock cannot run user
code. This permits reentrant coercion and atomic interning across host threads.
Charge each identity or UTF-16 comparison before inspecting it, and reserve vector
capacity before insertion. Symbol.keyFor accepts only primitive Symbols, scans
identity, and copies a matched key only after releasing the lock and checking output
limits. Fresh and well-known symbols never enter this registry implicitly.
The process registry has no default entry or text quota. Checked total-size
arithmetic and reservation failures remain host failures and must not evict entries,
change existing identities, or become JavaScript exceptions. Private isolated
registry instances test optional capacity edges without filling shared state.

Symbol.for/keyFor use this registry, and the intrinsic Symbol constructor's own
properties can be enumerated. The JavaScript global now exposes that same
intrinsic as a writable, non-enumerable, configurable property (19.3). Deleting
or replacing the global binding never changes retained intrinsic identities.
Implemented objects provide their required symbol properties: coercion and
instance checks, object tags, Array species/unscopables, Array/arguments/String
iteration, and shared iterator tags. New object kinds must add their own required
hooks when implemented. Script integration tests and unmodified reviewed Test262
Symbol/iterator files complement native tests of embedding and resource limits.

Array storage uses sparse indexed properties in the same traced heap,
with an explicit Array exotic identity and a non-configurable data `length`
property (10.4.2). Holes consume no indexed property slots; logical length is a
u32, and the key `4294967295` is an ordinary string property. Array identity does
not depend on the prototype chain. Storage enforces index growth, read-only
length checks, and descending-index truncation independently of parser syntax.
The low-level Objects API accepts only preconverted integral Number length
descriptors; UnnormalizedArrayLength reports a violated storage precondition,
not a JavaScript exception. Realm descriptor definitions and deferred length
assignments now supply these conversions. Array calls/new and Array.isArray now expose these objects to Scripts;
literal syntax and at/join/toString/push/pop are implemented; remaining prototype
methods are pending.


Array own-property lookup first verifies the exact key at the expected dense
position, without trusting insertion order or caching a derived array shape.
Canonical indices at or beyond a valid stored length are absent by the Array
exotic invariant (10.4.2.1). Otherwise lookup scans every stored property. The
same lookup serves ordinary descriptor updates and deletion. Sparse arrays,
named/Symbol metadata, holes, reordered keys and accessors retain their exact
descriptors; a missing own key still permits normal prototype lookup and Set.
This makes common dense reads and append definitions constant in property count
without changing storage, descriptor rules or opted-in work accounting. Other
shapes retain the complete scan, including writes into pre-sized sparse arrays.

ArraySetLength coercion stays in the Realm layer: ToUint32 and ToNumber observe
the original descriptor value separately, before reading the current length
descriptor (10.4.2.4). The same object can therefore run conversion code twice and
mutate the array between conversions. Storage receives a validated integer length
and performs no JavaScript calls under a borrow. Assignment must defer this work
until OrdinarySet's receiver/writability checks have succeeded; Object.defineProperty
must still perform length conversion before descriptor compatibility rejection.

Shrinking length first applies the new length descriptor, then deletes indexed
properties in descending numeric order. A non-configurable element stops deletion,
restores length to that index plus one, and still applies a requested read-only
length. Already deleted higher properties remain deleted. Host work/capacity
checks must precede mutation so an abort cannot expose an inconsistent array.
Sparse truncation scans once for the largest non-configurable index at or above
the requested length, then stably retains properties below the resulting length.
Ordinary deletion invokes no user code, so this exactly preserves the descending
first-failure rule and all partial effects. Named/Symbol creation order is
unchanged. Both scans receive a checked linear work charge before mutation;
neither absent indices nor repeated removal scans are visited. No additional
allocation or cached state is needed.

Land this work as storage invariants, Realm Array construction/Array.isArray and
length coercions, array literal grammar/evaluation, then prototype methods and
additional Test262 coverage. Array.prototype is itself an empty Array exotic
object (23.1.3). Missing constructor/prototype methods remain explicit gaps until
their dependencies are implemented. Symbol.iterator, species, and unscopables
are materialized as described below.
Join uses ToObject and reads LengthOfArrayLike once before separator conversion,
then interleaves indexed Get and element ToString in order. Nullish elements
contribute empty text; inherited properties at holes remain observable. Appends
charge work and check UTF-16 output limits before allocation. Array toString
invokes the current callable join with no arguments, falling back to the intrinsic
Object toString when join is not callable. Native recursive conversion, including
cyclic arrays, is bounded by the existing host call limit. LengthOfArrayLike is
shared with String.raw and Function.prototype.apply. Generic Array at snapshots
length before converting its index, applies ToIntegerOrInfinity, and performs Get
only for an in-range index. It uses u64 indices through the full ToLength range.
Push and pop also use generic LengthOfArrayLike. Each performs observable indexed
Set/Get/Delete operations before the final length Set, preserving partial effects
on failure. Push rejects lengths above 2^53 - 1 before writing elements, while
Array exotic length overflow can throw after ordinary non-index properties were
created. Pop reads an inherited final element but deletes only an own property.
Both perform the length Set even when no elements are inserted or removed.
ForEach/every/some share an ordered callback loop (23.1.3.6/15/29): ToObject,
LengthOfArrayLike, then IsCallable before iteration, including empty receivers.
Each index performs live HasProperty and Get; holes are skipped and inherited
values visited. Callback arguments are value, index, and the boxed receiver.
The supplied thisArg is preserved with copying charged before each call.
Every/some short-circuit on ToBoolean without coercing object results. Work is
charged per visited index, so large lengths still allow early observable exits.
Find/findIndex/findLast/findLastIndex use FindViaPredicate (23.1.3.9–12), which
visits every index with Get, including holes, in ascending or descending order.
An index cursor represents the specification list without allocating it. Value
returning methods preserve the pre-callback value even if the predicate mutates
the source; copies of thisArg and retained values are charged before cloning.
Includes/indexOf/lastIndexOf (23.1.3.16–17/20) share ordered range traversal,
with per-comparison work charged for strings and BigInts. Includes reads holes
as undefined and compares with SameValueZero; the index methods first check
HasProperty and use IsStrictlyEqual. Empty ranges skip fromIndex conversion.
Only an absent fromIndex defaults to length - 1 for lastIndexOf; an explicit
undefined becomes zero. All cursors preserve the full ToLength index range.
Reduce/reduceRight (23.1.3.24–25) share a directional traversal with live
HasProperty/Get operations. An optional accumulator distinguishes an omitted
initial argument from explicit undefined. The first present value supplies an
omitted accumulator; subsequent callbacks receive four arguments with undefined
as thisArgument. Moving the accumulator into Call avoids extra value copies.
A fully sparse range with no supplied initial value throws TypeError, and the
seed search consumes the same bounded work as subsequent traversal.
Reverse (23.1.3.26) snapshots length and walks pairs from the ends inward.
Each lower HasProperty/Get precedes the upper HasProperty/Get; the first getter
can therefore change the second presence check. The four presence combinations
use the specified Set/DeletePropertyOrThrow order, retaining partial effects on
failure. Values move between reads and writes without extra copies. No length
assignment occurs, and the middle element of an odd range is never accessed.
Fill/copyWithin (23.1.3.4/7) share bounded relative-index conversion after a
single LengthOfArrayLike snapshot. Conversions run even when the eventual range
is empty. Fill charges each value copy before Set; copyWithin reads source
presence/value live and deletes a target when its source is absent. Overlapping
forward destinations traverse backward; identical indices still perform reads
and writes. u64 cursors retain the full ToLength range without allocating index
lists. Both methods retain partial effects on failure and never assign length.
Shift/unshift (23.1.3.27/37) share copyWithin's live HasProperty/Get plus
Set/DeletePropertyOrThrow operation. Shift retains the initial first value,
moves left, deletes the last property, then assigns length. Unshift validates
the safe-integer length bound before moving right, inserts arguments in order,
and finally assigns length. Zero arguments skip movement but still assign
length. Earlier writes/deletions remain visible after any later failure.
ToReversed/with (23.1.3.33/39) use ArrayCreate with the intrinsic prototype,
without consulting constructor or species. They define dense own elements via
CreateDataPropertyOrThrow, bypassing inherited setters. ToReversed reads in
descending order; with converts and validates its index before ArrayCreate and
reads other indices in ascending order, never reading the replaced property.
The replacement value moves into the output once. ArrayCreate rejects lengths
above 2^32 - 1 before element access; host allocation/work limits remain distinct.
ToSpliced (23.1.3.35) distinguishes omitted start/skip arguments from explicit
undefined. It snapshots length, converts start then skip, and checks the resulting
safe-integer length before ArrayCreate. It copies the retained prefix, inserts
arguments without coercion, then copies the retained suffix; discarded indices
are never read. Source cursors retain the full ToLength range, so a huge array-like
input can shrink to a small valid array. Output is dense, uses the intrinsic
prototype, and never assigns to the input. All three loops consume work budget.
Array calls and construction observe newTarget.prototype before validating
the argument count/length (23.1.1.1). A lone Number supplies a sparse length; any
other single value becomes an element without coercion. Element creation uses
own data definitions, bypassing inherited setters. IsArray uses the internal
identity independently of prototypes; Proxy forwarding will join it when Proxy
objects exist. All Array intrinsics remain rooted after global properties change.
Array sort/toSorted share SortIndexedProperties and CompareArrayElements
(23.1.3.30.1–2, 23.1.3.34). Validate the comparator before ToObject/length. Collect
indexed values in ascending order before any comparisons; sort uses HasProperty
to skip holes, while toSorted reads through them. ToSorted creates its intrinsic
Array before collection. Use a bottom-up stable merge sort with two bounded
buffers, moving values between them and selecting the left value on equality.
Each move/comparison consumes work; checked allocation grows the input list only
as values are read. Comparator calls use undefined this and two values; ToNumber
normalizes NaN to equality. Undefined values sort last without invoking the
comparator. Default comparison performs ordered ToString and UTF-16 lexicographic
comparison. Propagate the first abrupt completion immediately. Only after sorting
does sort strictly write values and delete the remaining range without assigning
length; toSorted instead defines dense own elements without species lookup.
Array.prototype.toLocaleString follows the ECMA-262 algorithm for hosts without
ECMA-402 (23.1.3.32). This host uses a fixed locale with comma (U+002C) as its
implementation-defined list separator, independent of operating-system settings.
The reserved arguments are ignored. After one length snapshot, it gets each
element live, invokes the current toLocaleString with the original element as
receiver and no arguments, then converts the result to a String. Nullish elements
contribute no text; holes still perform Get. Separator concatenation precedes the
next indexed read, including output-limit checks. Cyclic arrays use the ordinary
reentry limit; no special cycle-to-empty-string behavior is added. BigInt elements
use their branded non-ECMA-402 decimal locale fallback.
Array's Symbol.species accessor returns the original receiver, including primitives,
without coercion (23.1.2.6). It has no setter and is configurable/non-enumerable.
Array.prototype's Symbol.unscopables data property points to the standard mutable
null-prototype table of 16 true-valued names (23.1.3.41). The outer property is
non-writable, non-enumerable, and configurable; "with" is absent from the table.
These properties are materialized independently of individual Array methods
and with environments. Change-by-copy methods and Array.of do not consult species.
ArraySpeciesCreate (7.3.22) reads constructor only for branded Arrays, then reads
Symbol.species only from object-valued constructors. Null species becomes
undefined; undefined selects the intrinsic Array. Other values must be
constructors and receive one Number length. Custom results need not be Arrays;
ArrayCreate's uint32 bound applies only to the intrinsic fallback. Foreign-realm
constructors and Proxy IsArray forwarding remain pending with those object kinds.
Map/filter (23.1.3.8/21) share the present-element callback traversal. Validate the
callback after length conversion and before species lookup. Map creates a result
with the initial source length and defines only present indices. Filter creates
with zero length, retains the pre-callback value with a charged clone, and packs
truthy selections. Both use CreateDataPropertyOrThrow, bypassing setters and
preserving earlier definitions after failure. Neither sets a final custom length.
Flat/flatMap (23.1.3.13–14) share FlattenIntoArray with an explicit frame vector,
avoiding native recursion for nested Arrays. Each frame snapshots its source length
and visits present indices with live HasProperty/Get. Flat converts depth after source
length and before zero-length species construction. FlatMap validates its mapper
before species construction and calls it only for original-source elements, with
the value, index, and original object. Flatten only branded Arrays, independently of
Symbol.isConcatSpreadable. Compact holes at each entered level. Define own data
properties, checking the safe-integer index boundary after reading/mapping each
non-flattened element. Do not set a final length on custom results. Checked frame
allocation reports platform capacity failures; no default flattening-depth quota
applies. Infinite depth stays infinite, and cyclic sources consume opted-in work.
Slice (23.1.3.28) converts start/end after length, then creates the species result
with the nonnegative range count. Visit HasProperty/Get live in ascending order
and define present elements at their relative positions. Missing source indices
do not delete pre-existing custom-result properties. Always perform a final
strict length Set, including for empty ranges. Range cursors retain the full
ToLength width and consume bounded work without an intermediate index list.

Concat (23.1.3.2–2.1) boxes its receiver and constructs a species result with
length zero before testing spreadability. For each receiver/argument in order,
objects consult Symbol.isConcatSpreadable; undefined falls back to the internal
Array brand. Primitive arguments never consult wrapper hooks. Spread inputs
snapshot LengthOfArrayLike, check the combined safe-integer bound before indexed
reads, then visit live HasProperty/Get in ascending order. Holes advance the
output cursor and preserve existing custom-result properties. Own data definitions
bypass setters and retain prior effects on failure. Always perform the final
strict length Set, even for empty results. Do not prevalidate an intrinsic result's
uint32 length: the final Set must follow earlier effects. Sparse scans consume
bounded work and remain host limits rather than JavaScript exceptions.

Splice (23.1.3.31) distinguishes missing start/deleteCount from explicit undefined,
converts ranges after one length snapshot, and checks the resulting safe-integer
length before species construction. Create a species result with the deletion
count; copy present deleted indices with live HasProperty/Get and own data
definitions. Always strictly set its length before touching the source, even when
species returns the source. Shrinking moves the retained suffix forward, then
deletes obsolete tail indices from highest to lowest. Growing moves the suffix
backward to preserve overlap. Missing source properties delete their destination;
present ones use strict Set, as do inserted values. Always strictly set source
length last, including zero arguments. Retain earlier effects on every abrupt
completion; do not preflight all properties or roll back. All Array prototype
methods are materialized, so its enumeration and integrity operations use ordinary
property traversal; Array constructor enumeration awaits fromAsync.

Array iteration follows edition-17 CreateArrayIterator and next (23.1.5.1–3).
Store an optional iterated-object handle, a u64 next index, and key/value/key+value
kind on a distinct ordinary object. Trace the iterated object until exhaustion
clears it; wrappers and prototype changes do not create or remove the iterator brand.
Each next call snapshots the index before reading live LengthOfArrayLike. Exhaustion
clears the source permanently. Otherwise update the index before an indexed Get;
keys never read elements, values return Get's result, and entries create intrinsic
two-element arrays. Length-conversion failures leave the index untouched; indexed
Get failures retain its increment. Reentrant getters observe these exact mutations,
without introducing a generator-style executing flag or automatic completion on
abrupt Get. Native recursion remains subject to the ordinary host reentry limit.
Iterator results are fresh ordinary objects with value/done data properties.
Expose Array keys/values/entries together with the values alias at Symbol.iterator
and the same intrinsic values callable on mapped/unmapped arguments. The shared
Iterator prototype supplies its iterator identity method and protected constructor
and Symbol.toStringTag accessors. Iterator has [[Construct]] but rejects ordinary
calls and construction with itself as newTarget (27.1.3.1.1). A distinct newTarget
creates an ordinary object with its object-valued prototype, falling back to the
intrinsic Iterator prototype otherwise. Bound construction first rewrites the
newTarget as required by 10.4.1.2. Constructor arguments are ignored after normal
evaluation; no next method or iterator brand is installed on the resulting object.
The global binding and native constructor metadata retain standard attributes.
The complete edition-17 shared prototype has constructor, eleven helper methods,
Symbol.toStringTag, and Symbol.iterator. Initialize string properties in
specification order and Symbols after them. Ordinary reflection, copying,
enumeration, and integrity operations use this complete inventory, preserving
accessor identity without invoking getters. Post-edition-17 iterator additions
remain outside this baseline.
The constructor getter returns the intrinsic Iterator independently of receiver
or replaced globals (27.1.3.3.1.1). The tag getter returns "Iterator" for every
receiver (27.1.3.3.13.1). Both setters share
SetterThatIgnoresPrototypeProperties (7.3.37): reject primitives and
the intrinsic home prototype, then inspect the receiver's own descriptor. An
absent property is created as an own writable/enumerable/configurable data
property without inherited lookups. An existing property uses strict Set,
preserving its attributes and invoking its setter when present. These failures
are catchable TypeErrors; recursive user setters retain the host reentry limit.

Iterator.from uses GetIteratorFlattenable in iterate-string-primitives mode
(27.1.3.2.2): reject all other primitives before hooks, get/call Symbol.iterator
with the original receiver when present, otherwise use the input object directly.
GetIteratorDirect captures next once before OrdinaryHasInstance tests against the
intrinsic Iterator constructor. Existing instances retain identity, independently
of public bindings or custom Symbol.hasInstance hooks. Other inputs become ordinary
objects with a private Iterated record and the intrinsic wrapper prototype, which
inherits Iterator.prototype. Trace both the underlying iterator and cached next.
The wrapper's next calls that cached value without arguments. Return reads its
underlying return method on each call and returns a fresh undefined/done result
when absent. Both methods validate their own internal slot, ignore arguments,
and forward present-method results unchanged without inspecting done/value or
tracking completion. Reentrancy and retries after language errors remain valid;
host failures preserve the existing handler bypass and native-stack guards.

Iterator.concat validates object inputs and captures each Symbol.iterator method
in argument order, without opening an iterator (27.1.3.2.1). Its native closure
opens sources lazily, captures their next once, and steps them in order. Each
yield creates a fresh intrinsic result; done skips value and moves to the next
source. Iterator-step/opening errors complete the helper without closing it.
Return before first next opens/closes nothing; at a suspended yield it closes
only the active source with no arguments, validates the close result, and returns
undefined/done. Completion discards captures and is permanent.

The helper prototype inherits Iterator.prototype and exposes native next/return
and the fixed "Iterator Helper" tag (27.1.2.1). Native state models suspended
start/yield, executing, and completed resumes, rejecting reentry as required by
GeneratorValidate/Resume/ResumeAbrupt. This specialized closure does not expose
language Generator syntax. Captures and the active iterator/next remain in the
traced heap throughout execution. Capture release is charged at creation; the
finish transition and active-reference release are charged before a resume
enters user code. Cleanup always runs without allocation after it, including host
failures; failed resumes complete without JavaScript cleanup. Completed helpers
release their captures before the next host collection. Iterator's static
inventory and the shared prototype are complete, so their own reflection and
integrity operations are enabled.

Iterator.prototype.toArray acquires an object receiver directly, captures next,
and consumes IteratorStepValue until done (27.1.3.3.12). It never consults
Symbol.iterator and never closes on acquisition, step, or host failures. Collect
owned values in a fallibly grown internal list, then materialize the intrinsic
Array only after exhaustion through CreateArrayFromList. This preserves element
identities and explicit undefined entries without species, constructor, or
inherited-setter calls. Defaults impose no work, list-size, or heap quota;
native addressable/allocation capacity remains a distinct host failure.

Iterator.prototype.forEach validates its object receiver and procedure before
looking up next (27.1.3.3.7). An invalid procedure closes an uninitialized iterator
record with the new TypeError as its incoming throw. Successful direct acquisition
captures next once; each yielded value calls the procedure with undefined this
and exactly value/index arguments. Ignore normal results, close on callback
language throws with incoming-error precedence, and never close on acquisition,
step, or host failures. Return undefined after exhaustion.

The index counter preserves the mathematical integer before each Number conversion.
Use u64 for ordinary counts, then exact integer storage beyond its range; correctly
round all callback indices, including beyond 2^53. This internal storage produces
Number values and is independent of the host's BigInt-value magnitude quota.
Work accounting still applies, with no default counter or execution quota.

Iterator.prototype.every/some/find share direct acquisition and exact callback
indices with forEach (27.1.3.3.3/5/10). They apply ToBoolean to each predicate
result without invoking conversion hooks. Every short-circuits on false, some
and find on true, then close the iterator before returning. Normal closing errors
replace the result; predicate language throws keep incoming-error precedence.
Find retains the original yielded value across the predicate call. Exhaustion
returns true/false/undefined respectively and never closes. Step and host failures
also bypass closing; internal counters have no default or BigInt-value quota.

Iterator.prototype.reduce validates the callback before direct acquisition
(27.1.3.3.9). An omitted initial value consumes the first yield as the accumulator
and starts callback indices at one; a present value, including undefined, starts
at zero. Empty input without an initial value throws TypeError without closing.
Each reducer call receives undefined this and exactly accumulator/value/index;
its result becomes the next accumulator without conversion. Only callback
validation/language throws close with incoming-error precedence. Acquisition,
step, exhaustion, and host failures never close. Share the exact unbounded
mathematical counter used by other eager consumers.

Iterator.prototype.map/filter eagerly validate the callback and capture direct
next, then suspend native helper closures (27.1.3.3.4/8). No source step or callback
runs until next. Each callback receives undefined this and value/mathematical
index; map yields its result, filter tests ToBoolean and yields the original value.
Rejected filter values loop without native recursion. The exact counter advances
after resuming a yield or skipping a value, independently of BigInt-value quotas.

Helper payloads distinguish concat and callback closures. Captured source, next,
and callback edges remain traced even while executing and are released on permanent
completion. Return before the first next completes before closing the already
captured direct source, so cleanup reentry observes done. Yielded return and
callback throws close the live source; step/host failures never do. Active reentry
throws TypeError. Finish/release work is prepaid and cannot strand an executing
helper after a host abort. All defaults remain unlimited.

Iterator.prototype.flatMap captures direct next and suspends a one-level flattening
closure (27.1.3.3.6). GetIteratorFlattenable rejects all mapped primitives, including
Strings, before any iterator lookup. Object results use their iterator method or
fall back to direct next. Capture each inner next once; exhaust empty inners with
an iterative loop. Advance the exact outer index only after an inner is exhausted.
Outer step failures never close; mapper, acquisition, and inner step language
throws close the outer source with incoming-error precedence, without closing a
failing inner. A return resumed at Yield closes the inner before the outer. Inner
closing errors become the incoming throw for outer cleanup. Host failures skip
JavaScript cleanup and permanently complete the helper. Trace the outer, mapper,
and active inner captures throughout execution and suspension; release them on
completion. Defaults impose no execution, heap, or internal counter quota.

Iterator.prototype.take/drop convert the limit before direct next capture
(27.1.3.3.2/11). Conversion throws, NaN, and negative integer counts close with
incoming-error precedence; negative fractions truncate to zero. Edition 17 accepts
positive infinity and finite counts beyond 2^53. Store finite counts as exact
mathematical integers independently of BigInt-value quotas, so subtraction cannot
stall or round. No source step runs until the helper is resumed.

Take decrements before each step and closes when its count reaches zero on a later
resume; a zero count closes without any source step. Observed source exhaustion
never closes. Drop uses IteratorStep while discarding, bypassing value getters,
then yields through IteratorStepValue. Both inherit helper return/reentry/tracing
and permanent-completion rules; step and host failures never run cleanup. The
pinned Test262 safe-integer-cap assertions describe post-baseline behavior and
remain outside this edition's unchanged selected corpus.

String iteration converts its receiver once, synchronously at creation, after
RequireObjectCoercible (22.1.3.36). The captured value is an owned UTF-16 string;
it does not retain the original receiver. A branded String iterator stores the
captured string and suspended code-unit position. Each next yields the one- or
two-unit substring selected by CodePointAt (11.1.4), preserving lone surrogates
and advancing before returning a fresh IteratorResult. Exhaustion releases the
string and future calls return fresh completed results. Each step reads/copies
at most two units instead of cloning the entire captured string. This specializes
the closure-based GeneratorResume algorithm (22.1.5.1.1, 27.5.3.3): the String
closure never calls user code between resumes, so no executing state can be
observed. General generators will require their own execution-state machine.
The String iterator prototype inherits the shared Iterator prototype, with its
own next function and non-writable/configurable String Iterator tag.

Array.of (23.1.2.4) tests its receiver for [[Construct]] without coercion. It
constructs with one numeric item-count argument or falls back to ArrayCreate.
It defines own writable/enumerable/configurable data elements, then strictly
assigns length even for zero items. Custom constructors may return arbitrary
objects; definitions bypass inherited setters and replace configurable accessors
or read-only properties. A later rejected definition or length write preserves
earlier effects. Nested construction and length setters use shared reentry limits.

Array.from (23.1.2.1) validates a supplied mapper before GetMethod(items, @@iterator).
Iterable results construct with no arguments before calling the iterator method;
array-like results snapshot ToLength before constructing with one length argument.
Neither path observes species. Non-constructors use intrinsic Arrays. Get every
array-like index, including holes, but use live iterator state for iterable input.
Mapping receives exactly value/index with the supplied thisArg. Create own data
elements and strictly set the final length, preserving all earlier effects.
Iterator acquisition calls the method with the original receiver and captures
next once. IteratorStepValue reads done before value, marks completion/abrupt steps,
and never closes after a next/done/value failure. Mapping, element definition,
and the safe-integer overflow guard close an active iterator. IteratorClose with
an incoming throw calls return but preserves that throw over cleanup exceptions
or non-object results. Host failures stop without running cleanup; failures in
cleanup remain host failures. A final length failure after exhaustion does not close.

Array literals distinguish elisions, ordinary expressions, and spread in the AST.
Elisions remain distinct from an explicit undefined expression. AssignmentExpression[+In] parsing separates
elements from comma expressions and preserves trailing-comma lengths. Evaluation
creates an intrinsic Array before evaluating elements, grows length for holes,
and defines own elements in source order without inferring function names. Array
spread acquires a synchronous iterator, captures next once, and consumes each value
before evaluating the next source element (ArrayAccumulation, 13.2.4.1). Iterator
failures propagate directly without closing. Yielded undefined values are dense.
Elisions assign length at their position; the final initializer assigns length
after all elements, even if spreading crosses the Array index boundary. Convert
large element indices through Number before stringifying property names. Native
index arithmetic remains checked, with no default work or output quota. Array
assignment uses the distinct AssignmentPattern semantics described above.

The `Objects` heap context validates prototype handles and prevents ordinary
prototype cycles. Get, HasProperty, Set, and SetPrototypeOf traverse iteratively
under an explicit work budget. Data writes follow the receiver even when lookup
starts on another object, preserve existing attributes, and reject inherited
non-writable properties. Descriptor rejection returns false; invalid handles and
resource exhaustion remain host errors. Allocation and these internal methods
never collect implicitly. Explicit collection combines caller-supplied handles
with live host `Root` tokens. Tokens contain handles, not object data, and the
registry holds only weak references to them. Repeated roots of an object share
a token; dropping its last clone releases that root. Expired entries are reused,
bounding the registry by heap slot capacity. Root registration and registry scans
consume explicit work budgets. Object fields must hold handles, never root tokens.

`Value::Object` carries an unrooted handle. Equality uses identity, every ordinary
object is truthy, and property tracing visits object-valued edges. Heap-context
definitions validate these edges before mutation. Context-free conversion APIs
return a distinct `ConversionError::ObjectNeedsContext`, rather than inventing
a primitive or JavaScript exception. Realm-level ToPrimitive checks its symbol
hook before OrdinaryToPrimitive performs
ordered method lookups and calls using the requested hint and original receiver.
Objects without a method yielding a primitive throw TypeError. Object.prototype
toString and valueOf provide ordinary default conversion. Object.prototype.toString
boxes implemented primitive kinds, selects the internal-slot fallback, then reads
Symbol.toStringTag with the boxed/original object as receiver (20.1.3.6). Only a
String overrides the fallback; other values are never coerced. Tag concatenation
preserves UTF-16 and checks output work/capacity before allocation. Nullish values
skip lookup. Symbol and BigInt receivers use their actual wrappers and observable
prototype tags. Deleting those tags exposes the ordinary Object fallback.
Unimplemented intrinsic algorithms remain Unsupported. Each object kind must
implement its required Symbol hooks alongside its string-keyed API.
Arithmetic and comparisons convert original operands from left to right after
both expressions evaluate; untagged templates and property names use the string hint.

Realm allocation supports opt-in shared heap-slot and per-object property quotas. Object
literals create data properties in source order, convert computed keys before
evaluating values, and implement the required non-computed `__proto__` initializer.
Object spread evaluates sources in order and applies CopyDataProperties with an
empty exclusion list (13.2.5.5, 7.3.25). Skip nullish sources; box other primitives
without coercion hooks. Snapshot ordered own keys, including Symbols, then read
each descriptor and enumerable value live so earlier getters can delete keys or
change their enumerability. Define writable, enumerable, configurable own data
properties, bypassing setters and treating spread `__proto__` keys as data.
The intrinsic Object prototype has a stable, retained identity and its mandatory
string-keyed methods. A lookup that reaches an unimplemented intrinsic
method reports Unsupported; own or nearer inherited data properties can shadow
that method normally. Remaining well-known hooks are explicit implementation gaps.

Ordinary properties distinguish data and accessor records. Partial descriptors
carry mutually exclusive kind-specific fields; omitted fields preserve existing
attributes. Configurable kind changes preserve common attributes and reset the
new kind's omitted fields. Frozen accessors require identical getter/setter
handles. The owning heap validates accessor callability and traces both edges.
Storage returns getter/setter call actions so execution releases heap borrows
before calling with the original receiver (10.1.8.1, 10.1.9.2). Accessor syntax and
JavaScript descriptor/reflection APIs remain separate increments.

Property references retain the evaluated base and the unconverted computed name.
Edition 17's 13.3.3 defers ToPropertyKey until GetValue, PutValue, or deletion;
simple assignment therefore evaluates its RHS before converting the key. GetValue
caches the converted name for a later compound-assignment write. These operations
apply ToObject before key conversion, so a nullish base throws TypeError first.
Deletion does not GetValue. Strict writes/deletions translate false internal-method
results into TypeError. Ordinary data lookup and prototype traversal remain bounded.

Primitive property operations avoid allocating unobservable temporary wrappers,
as permitted by the GetValue/PutValue/delete notes. String own properties expose
UTF-16 length and single-code-unit indices with non-writable, non-configurable
attributes. Canonical numeric index names exclude string `-0`, leading zeros,
fractions, and out-of-range indices. Writes retain the primitive receiver and
fail without persisting a data property; strict mode throws. Boolean, Number, and String primitives
look up their actual prototype graphs, retaining the primitive receiver for
inherited getter/setter calls; setters can succeed in either strictness mode. Missing standard prototype
methods report Unsupported, while absent properties produce undefined. The
optional Annex B String methods and optional legacy Object.prototype accessor
are not installed.

The `in` operator checks that its RHS is an Object before converting the key,
then searches own and inherited properties. Presence checks can report standard
intrinsic names as present even before their callable values are implemented.
The parser carries the grammar's In parameter: for initializers exclude bare
`in`, while parentheses, computed property names, object values, templates, and
the conditional middle expression restore it where the grammar requires.

`Realm::collect` runs only between evaluations and scans persistent lexical/global
bindings, intrinsic handles, and host roots. Environment scans are budgeted even
for primitive bindings. Returned and thrown values are unrooted until the embedder
retains a `RootedValue`; cloning the underlying Value alone does not keep it alive.
No language operation or allocation calls the collector. Before enabling collection
inside evaluation, temporary and pending-completion lifetimes still need roots.

Function support begins with call syntax and reference-aware callee evaluation.
GetValue of the callee precedes arguments; argument evaluation precedes the
IsCallable check. Member calls retain their base as the receiver, including through
parentheses. Optional calls remain unsupported. ArgumentListEvaluation (13.3.8.1)
distinguishes ordinary and spread arguments. Each spread expression acquires a
synchronous iterator and consumes it before later arguments. Cache next and read
done before value; step failures propagate without closing. Calls/construction
check the callee only after all arguments finish. Keep the original member receiver
and constructor value across iterator side effects. Argument storage uses checked
count arithmetic/reservation, and any argument/work quotas remain opt-in. Builtin
function objects carry explicit callable metadata, inherit Function.prototype,
and have standard name/length descriptors. Function.prototype itself is callable
and returns undefined. Object.prototype has an immutable null prototype
(20.1.3, 10.4.7.1). The intrinsic objects are published atomically during realm
initialization and retained as roots; failed initialization leaves only unreachable
allocations for explicit collection. Function.prototype owns configurable,
non-enumerable caller/arguments accessors that share the realm’s non-extensible
%ThrowTypeError% function (9.3.2, 10.2.4). Its name/length descriptors are frozen.
Reads and writes throw TypeError in both modes, while presence and own-property
deletion do not invoke accessors. Unavailable standard methods remain Unsupported.

The Function global exposes the intrinsic constructor with standard name, length,
and immutable prototype descriptors, inheriting the callable Function.prototype.
The prototype's writable/configurable constructor link completes its own-key
reflection and integrity operations (20.2.2.2, 20.2.3.1). The constructor has
call/construct metadata for branding and newTarget validation. Ordinary dynamic
calls and construction compile arbitrary UTF-16 source in the realm's global
environment, with the body alone controlling strictness (20.2.1.1.1).

The parser's dynamic Function entry point (20.2.1.1.1) first parses the joined
FormalParameters text and line-feed-delimited FunctionBody as separate goals.
Neither input can terminate a wrapper or complete an unterminated comment in the
other. It then parses the prescribed `function anonymous` source and validates
combined parameter/body early errors with no inherited caller strictness. Source
ranges and retained source belong to that combined allocation. Identifier/pattern/default/
rest parameters share the existing function grammar; new.target and return use
function context. Unsupported generator/async functions and native-stack exhaustion remain separate
diagnostic categories. UTF-8 and
lossless UTF-16 parser entry points share this grammar and validation.

Source storage uses unchanged scalar UTF-8 text plus an ordered table of lone
surrogate code points at their encoded byte positions. The scanner uses U+FFFD
at these positions, which has the same lexical classification outside literals
and comments, and restores the original code points for every literal value,
template raw/cooked component, and retained source range. Real U+FFFD characters
remain distinct. Template values normalize CR/CRLF while retained source preserves
the original line endings. Functions share the source allocation; slicing a
scalar-only nested function still permits borrowed UTF-8 even if surrounding
source contains surrogates. `FunctionSource::as_str` returns `None` for a range
with lone surrogates; `to_js_string` returns exact source code units (20.2.3.5).

CreateDynamicFunction converts parameter arguments in order, then the body, before
parsing or reading newTarget.prototype. Calling uses the retained intrinsic Function
constructor as newTarget; bound construction preserves the existing forwarding rules.
Successful parsing precedes observable prototype selection, with a retained intrinsic
Function.prototype fallback. Allocation shares OrdinaryFunctionCreate/MakeConstructor
with an explicit global environment and body strictness, so caller locals, strictness,
and a named-expression self binding cannot leak into the result. Functions retain
standard anonymous source text and ordinary constructor/prototype metadata. The
default host permits string compilation; optional source/work/string/heap quotas
remain host aborts. Lone surrogates are accepted in string/template contents and
comments. Invalid identifier characters throw SyntaxError after all conversions
and before prototype lookup. Encoded-byte accounting uses UTF-8 lengths for
scalars and three bytes per lone surrogate, with checked capacities throughout
dynamic source assembly. All host quotas remain opt-in.
The constructor and prototype remain intrinsic roots after public deletion.

Native Object.prototype.valueOf returns fresh Boolean/Number/String wrappers for their primitive
receivers; other primitive wrappers remain Unsupported. Arrow closures use the same callable dispatch with
their captured environment identity.

String construction follows 22.1.1.1, converting the argument before observing
newTarget.prototype. Missing input differs from explicit undefined. StringData
identifies wrapper values for the branded toString/valueOf methods, independently
of prototype identity. StringCreate (10.4.3.4) currently materializes every UTF-16
index as an immutable enumerable data descriptor plus the fixed length property.
This produces String exotic read/define/key-order behavior through the existing
descriptor machinery, while charging length plus one against property capacity.
Bulk construction charges work before publishing the wrapper; a later virtual
representation can improve storage without changing semantics. Primitive index
reads stay virtual and precede prototype access, and primitive writes to those
indices cannot invoke inherited setters. String prototype methods and statics not
yet implemented remain explicit gaps.

String.fromCharCode and fromCodePoint perform ordered numeric conversions and
construct UTF-16 without rejecting lone surrogates (22.1.2.1–2). Character access
methods at/charAt/charCodeAt/codePointAt convert their receiver before the index
(22.1.3.1–4). They retain code-unit indexing, the distinct out-of-range return
values, relative indexing for at, and surrogate-pair decoding for codePointAt.
isWellFormed and toWellFormed perform generic receiver conversion, then decode
UTF-16 with the standard library (22.1.3.10/31, 7.2.7). Decoding errors identify
individual unpaired surrogates; toWellFormed replaces those units with U+FFFD and
preserves every valid pair. Scans charge work in advance, and replacement checks
the unchanged output length before allocation.
concat uses String-hint conversion for its receiver and each argument in order,
checking cumulative output lengths before extending the result (22.1.3.5).
slice and substring convert the receiver, start, and end in order even for empty
inputs (22.1.3.22/25). Their shared implementation distinguishes relative negative
indices from clamping/swapping endpoints and slices code units without repairing
surrogates. Output allocation and copying remain subject to host limits.
trim/trimStart/trimEnd use the shared ECMAScript WhiteSpace and LineTerminator
predicates (22.1.3.32–34), preserving internal whitespace and all other code units.
All baseline whitespace code points lie in the BMP, so checking code units is
equivalent to decoding code points for this membership test; surrogates cannot
match. Scanning and copying are charged separately. Optional trimLeft/trimRight
aliases remain disabled with other Annex B methods.
repeat validates the converted count before returning an empty string; finite
counts on empty input take no repetition loop (22.1.3.18). padStart/padEnd apply
ToLength and return early before filler conversion when no padding is required
(22.1.3.16–17). Empty fillers preserve the input even for huge target lengths.
Both operations copy UTF-16 units directly, including truncated surrogate pairs.
Checked arithmetic, precharged output work, and fallible capacity reservation
keep size/capacity failures distinct from ECMAScript RangeErrors.
indexOf/lastIndexOf convert receiver, search text, and position in that order
(22.1.3.9/11). NaN means position zero for indexOf and positive infinity for
lastIndexOf. Searches compare UTF-16 slices without decoding, return the clamped
position for empty needles, and charge the needle length before each candidate
comparison. This bounds the initial quadratic algorithm without introducing a
more complex matching implementation before it is needed.
includes shares bounded substring search; startsWith/endsWith compare just their
selected range (22.1.3.7/8/24). Empty searches match, including clamped endpoints.
endsWith defaults undefined to the receiver length; other position conversions
map NaN to zero. IsRegExp reads Symbol.match on objects after receiver conversion
and before search-string or position conversion (7.2.6). A defined marker uses
ToBoolean without calling or converting it; truthy markers cause TypeError even
for empty searches. Primitive searches never perform this lookup. Inherited
getters retain the search object as receiver. An undefined marker falls back to
the object's own native RegExp brand; string-keyed lookalikes are not hooks.
String.split (22.1.3.23) checks receiver coercibility, then delegates Symbol.split
only for object separators as required by edition 17. Pass original this/limit
values and return the hook's result without conversion. Fallback converts this
to a String, limit through ToUint32 (undefined defaults to 2^32-1), then separator
to a String even when the limit is zero. Undefined separators return the full
String; empty separators split individual UTF-16 units with no extra empty parts.
Nonempty separators use non-overlapping searches, retaining leading/trailing
empty substrings and stopping at the limit. Checked copies and each search
comparison charge opted-in work. Fresh intrinsic Arrays bypass constructors,
species, and inherited setters. Generic RegExp splitting accepts custom species
constructors and exec methods and shares native literal matching with RegExpExec.

String.replace (22.1.3.19) delegates Symbol.replace only on object searches,
preserving original this/replacement values and returning the hook's result.
Fallback converts the receiver and search, then converts non-callable replacements
before searching even when there is no match. Replace only the first UTF-16 match.
Call functional replacements with undefined this and exactly match/position/full
String; convert their return value to literal replacement text. Non-functional
replacements use GetSubstitution with no captures (22.1.3.19.1): expand $$, $&,
prefix, and suffix tokens once; numeric/named-capture tokens remain literal.
Checked output reservation and copied/scanned units charge opted-in work. Input
Strings remain fixed through callbacks. Generic RegExp replacement shares the
same scanner with capture resolution.

GetSubstitution template scanning (22.1.3.19.1) is a cloneable, allocation-free
UTF-16 iterator in core. It returns borrowed literal units or match/context,
numbered-capture and named-capture references. Decimal references consume at most
two digits and fall back to one digit when the two-digit index exceeds the
capture count; zero references stay literal. Named keys end at the first greater
than unit and preserve arbitrary UTF-16. Without a named captures Object, the
dollar/less-than pair stays literal and later references are scanned normally.
Caching the next delimiter, including its absence, keeps unterminated templates
linear. Consumers resolve references in order without rescanning expansion text.
String replace/replaceAll use this scanner with no captures. Generic RegExp
replacement supplies already-converted numbered captures and performs each named
property read/conversion when its reference is consumed. Custom matches extending
past the input yield an empty suffix without indexing past its UTF-16 units.

String.replaceAll (22.1.3.20) checks object searches through IsRegExp, then reads
and converts flags for a true result and requires a lowercase g before looking up
Symbol.replace. Primitive searches ignore prototype hooks. Ordinary fallback
converts receiver, search, and non-callable replacement in order, then collects
non-overlapping match positions before any replacement callback. Empty searches
match every UTF-16 boundary, including between surrogate halves and at the end.
Each callback receives match/position/full String with undefined this and yields
literal text; String replacements reuse uncaptured GetSubstitution against the
original input for every match. Position storage and output reservation check
platform capacity, and searches, callback copies, and substitutions charge opted-in
work. Generic RegExp replacement hooks are implemented; native RegExp objects
and matching remain pending.

String.raw uses ToObject for its template and raw value, reads length once through
LengthOfArrayLike, and interleaves each indexed literal conversion with the
corresponding available substitution (22.1.2.4). Missing substitutions add no text;
missing indexed literals convert undefined normally. Each iteration and copied
unit consumes work, even for huge lengths and empty literals. This ordinary-call
API completes the String constructor's static properties, so own-key reflection
and integrity operations on that constructor no longer need an incomplete-intrinsic
guard. Tagged templates use the same raw component semantics.

Boolean construction and methods follow 20.3. Calling Boolean applies ToBoolean
without invoking conversion methods; construction also allocates a fresh ordinary
object with an own BooleanData internal slot. Boolean.prototype holds false in
that slot, inherits Object.prototype, and supplies constructor, toString, and
valueOf. The methods accept Boolean primitives or objects with an own slot;
inheriting Boolean.prototype alone does not qualify. The constructor's prototype
property is fixed. The global Boolean property is writable/configurable and does
not control intrinsic identity or receiver boxing after deletion/replacement.
Non-strict ordinary calls and Object.prototype.valueOf use ToObject to allocate
observable wrappers (7.1.18, 10.2.1.2); primitive property operations need no
allocation. Object.prototype.toString recognizes BooleanData independently of
prototype identity. Wrapper slots consume trace work, and wrapper object edges
use the existing heap/root machinery. Bound construction forwards through the
same Boolean constructor algorithm and ignores the bound receiver.

Number support includes construction, valueOf, constants,
and the non-coercing isFinite/isNaN/isInteger/isSafeInteger predicates (21.1).
Explicit Number conversion accepts BigInt, using the integer library's single
correctly rounded conversion; ordinary ToNumber still rejects it. NumberData
preserves negative zero and NaN. Decimal toString uses the existing Number string
conversion after validating the receiver and radix in specification order.
Non-decimal formatting uses exact integer ratios and the binary64 rounding
interval (6.1.6.1.20). Generate significant digits until a candidate lies inside
that interval; this gives the shortest round-tripping representation. Account
for the smaller lower interval at normal powers of two and include midpoint
boundaries only when the input significand is even. If both adjacent candidates
qualify, choose the nearest, breaking ties toward an even significand as the spec
recommends. Normalize carries and use fixed notation in every non-decimal radix.
An approximate exponent may seed normalization, but exact comparisons must
correct it before digit generation. Budget integer work and result length, and
test subnormals, exponent transitions, and maximal finite values independently
of ordinary decimal formatting.
Pinned reference vectors use Python's standard-library Fraction arithmetic and
correctly rounded conversion to float, enumerating candidates by precision rather
than duplicating runtime digit generation. CI verifies their reproducibility.

Number.prototype.toFixed uses the exact binary significand/exponent to scale by
10 to the requested power, divides with integer arithmetic, and rounds ties toward
the larger magnitude (21.1.3.3). It must not use the shortest decimal rendering as
input or Rust's ties-even decimal formatter. Validate the receiver and fraction
argument before special-value handling. Negative zero has no sign, but negative
nonzero values that round to zero retain it. Values at least 1e21 use ordinary
Number string conversion. Integer work and result length remain host bounded.

Number.prototype.toPrecision shares exact rational scaling and ties-up rounding
with toFixed (21.1.3.5). Estimate the decimal exponent from the binary exponent,
then correct it with exact integer comparisons against powers of ten. Cancel
common factors of two while scaling to keep intermediate integers small. Preserve
the requested significant zeros and apply the specification's fixed/exponential
thresholds after rounding can carry into the exponent. Precision conversion
precedes nonfinite handling, while its range check follows nonfinite handling.

Number.prototype.toExponential always emits scientific notation (21.1.3.2).
An explicit fraction argument uses the same exact rounding for 1–101 significant
digits; an absent/undefined argument uses Rust's shortest scientific rendering,
normalizing its exponent sign. Coerce the argument before nonfinite handling and
range checking. An object that converts to undefined still counts as an explicit
fraction argument and requests zero fractional digits.

This host does not implement ECMA-402. Number.prototype.toLocaleString therefore
uses the explicitly permitted ECMA-262 fallback of ordinary Number string
formatting (21.1.3.4), with no locale-specific grouping. It validates NumberData
receivers directly and ignores the reserved locale/options arguments; their
positions are not repurposed. Overrides of the public toString property do not
change this builtin algorithm. This implementation-defined choice is stable across
host operating-system locales.

The global parseFloat and parseInt functions share identity with Number.parseFloat
and Number.parseInt (19.2.4–5, 21.1.2.12–13). Coerce the input to String first;
parseInt then coerces radix with ToInt32. Scan UTF-16 units for the decimal/radix
prefix after ECMAScript leading whitespace, so an unpaired surrogate after valid
digits terminates the prefix rather than invalidating it. Decimal exponents only
join a prefix when they contain digits. parseInt accumulates an exact integer in
every radix and rounds once to binary64; it does not use the optional approximation
allowances. Preserve negative zero and the specified hexadecimal prefix rules.
Input length, scan work, and integer arithmetic remain bounded by realm limits.

Global isFinite and isNaN use ordinary ToNumber (19.2.2–3), including object
coercion with the number hint and TypeError for BigInt. They are separate builtin
objects from Number.isFinite and Number.isNaN, which never coerce their argument.
The global functions ignore their receiver and additional arguments after normal
argument evaluation. Their intrinsic roots survive deletion of public bindings.

String.normalize implements NFC, NFD, NFKC, and NFKD using pinned Unicode 18 data
(22.1.3.15, UAX #15). Require a non-nullish receiver and convert it with the string
hint before converting/validating the form; undefined selects NFC. Generate
decomposition, combining-class, and composition tables with source digests,
acyclic-graph validation, and full composition exclusions. Expand mappings with
an explicit stack, reorder nonstarter runs with stable counting order, and compose
in place with canonical blocking and algorithmic Hangul. Preserve lone surrogates
as class-zero boundaries. Do not apply output string quotas to intermediate scalar
lists; check final UTF-16 output before fallible reservation. Work quotas remain
opt-in and host aborts bypass JavaScript cleanup. The complete unchanged Unicode
NormalizationTest oracle verifies all specified column invariants and scalar
identity outside its Part 1 inventory.

String.localeCompare uses this host's fixed locale-neutral collation without
ECMA-402 (22.1.3.10). RequireObjectCoercible and ToString of the receiver precede
ToString of the comparison value. Compare NFD strings lexicographically by UTF-16
units, charging comparison work; this yields a consistent total ordering and +0
for all canonically equivalent spellings. Preserve lone surrogates and compatibility
distinctions. Ignore the reserved locale/options arguments after their evaluation,
and invoke native normalization without consulting mutable public methods.

String.match, matchAll, and search delegate to object Symbol hooks before receiver
conversion, preserving the original receiver and returning the hook result unchanged
(22.1.3.11–12, 21). MatchAll performs IsRegExp, reads/coerces flags, and requires a
lowercase g before looking up Symbol.matchAll. Edition 17 ignores primitive prototype
hooks. If no hook is available, convert the receiver, then RegExpCreate allocates
from the intrinsic constructor and initializes the pattern directly. It bypasses
IsRegExp, constructor call identity and copying of native original slots. Undefined
patterns become empty; matchAll supplies g while match/search supply undefined
flags. Invoke reads the new object's live Symbol method and calls it with that
object as receiver and the converted String as its sole argument. Missing or
non-callable methods throw TypeError. Default matchAll creates its lazy native
iterator; execution uses the native matcher for supported ordinary literal
Patterns and reports Unsupported for the remaining productions. These native
methods complete the baseline String
property inventory, enabling ordered reflection, enumeration, copying, and integrity
operations independently of the broader native matching gaps.

String toLowerCase/toUpperCase use the full Unicode 18 default case mappings
(22.1.3.29–31, 35). Generate reproducible, digest-pinned tables from UnicodeData,
SpecialCasing, and DerivedCoreProperties; Rust toolchain Unicode versions must not
change results. Preserve lone UTF-16 surrogates. Apply Final_Sigma to original input,
skipping Case_Ignorable points before considering Cased, including overlapping
properties. Do not normalize Unicode. Context scans are iterative and use constant
auxiliary space; checked output growth accounts for multi-point expansions before
fallible reservation. The host's fixed locale-neutral default means
toLocaleLowerCase/toLocaleUpperCase use the same mappings and ignore ECMA-402's
reserved arguments. All four methods remain generic with once-only string-hint
conversion, zero length, and separate retained function identities.

URI encoding (19.2.6.3–5) first performs string-hint ToString. Traverse UTF-16
iteratively, reject every unpaired surrogate with URIError, and encode valid code
points into uppercase percent-encoded UTF-8 octets. Both encoders preserve ASCII
word characters and `-.!~*'()`; only encodeURI also preserves `;/?:@&=+$,#`.
Do not normalize Unicode or preserve existing percent escapes. Ignore the call
receiver and extra values after normal argument evaluation. Retain both intrinsic
function roots independently of replaceable global bindings. Prepay output work
and check optional string quotas/addressable capacity before fallible reservation;
no default output, work, or heap quota is imposed. URIError materialization uses
the retained native prototype.

URI decoding (19.2.6.1–2, 6) performs string-hint ToString once and preserves raw
UTF-16 code units, including lone surrogates. Validate percent triplets with ASCII
hex digits and strict one-to-four-byte UTF-8: reject bad continuation bytes,
overlong sequences, surrogate code points, and values above U+10FFFF with URIError.
Only decodeURI preserves reserved `;/?:@&=+$,#` escapes, retaining the original
hexadecimal spelling. Neither decoder rescans decoded percent signs, normalizes
Unicode, or changes plus signs. Ignore receivers and extra values after argument
evaluation. Retain both intrinsic roots; use the shared checked, fallible output
accumulator and existing opt-in quotas without imposing defaults.

Error objects include Error and the six NativeError constructors,
Error.prototype.toString, Error.isError, and ordered message/cause initialization
(20.5). Only instances carry ErrorData; prototype
objects do not. NativeError constructors inherit Error, and their prototypes
inherit Error.prototype. Constructor calls allocate before message conversion,
then inspect object-valued options for an inherited or own cause property. Keep
ErrorData private and trace cause values through ordinary properties.

AggregateError shares the ErrorData brand and native Error inheritance (20.5.7).
Select the newTarget prototype and allocate first, then convert the message,
install cause, and obtain a synchronous iterator from the errors argument.
Cache next once and collect values without coercion. IteratorToList does not close
the iterator on exhaustion or abrupt next/done/value access. After exhaustion,
create a fresh intrinsic Array with own indexed data properties and install the
writable, non-enumerable, configurable errors property. Public Array constructors,
species hooks, and inherited setters do not participate. Trace list values through
ordinary Array properties; retain the constructor and prototype as intrinsic roots.
Use fallible list reservations and existing opt-in work/heap limits; impose no
default list size or work quota. Host failures propagate without JavaScript cleanup.

Built-in runtime exceptions materialize as Error objects when a catch binding
needs a JavaScript value, using intrinsic prototypes independently of replaced
global bindings. Preserve the host-facing exception category, message, and source
span for uncaught failures. Host Unsupported/Limit failures must still bypass
JavaScript handlers. Standard Error properties are the entire exposed interface;
stack traces and host-specific fields are outside the language baseline.

Embedding hosts can obtain exception values and read properties through checked
realm operations. Exception conversion preserves explicit throw identity and
creates a fresh standard Error for a Rust-described built-in failure; parse and
host failures have no JavaScript value. Property reads preserve Get semantics,
including accessor receivers and abrupt completions. Each operation receives a
fresh allowance when `max_steps` is enabled, and returned values require host roots
across collection.
Foreign/stale embedding handles are checked at the boundary and return a distinct
InvalidObject host failure before internal evaluation can dereference them.
The Test262 runner uses these operations to inspect constructor names for
runtime-negative exceptions. It never treats failures during that inspection as
the original test's expected exception.

Object implements its constructor and mandatory string-keyed prototype
methods (20.1.1, 20.1.3). Nullish arguments create fresh ordinary objects; object
arguments retain identity, and other primitives use the implemented ToObject
wrappers. A distinct newTarget selects its own prototype and ignores the value
argument. Property predicates convert their key before converting their receiver;
isPrototypeOf tests its argument's type before receiver conversion. toLocaleString
invokes the receiver's current toString with no arguments. Object.defineProperty
and getOwnPropertyDescriptor use ToPropertyDescriptor/FromPropertyDescriptor
(6.2.6.4–5), preserving inherited field reads, accessor validation order, omitted
fields, descriptor rejection, and mapped-argument aliasing. Descriptor results
are fresh ordinary objects with mutable enumerable fields. Object.hasOwn converts
its target before its key; Object.is performs bounded SameValue comparisons.
Object.getPrototypeOf uses ToObject; setPrototypeOf validates its prototype before
returning primitive targets unchanged. Ordinary prototype changes preserve
identity checks, immutable/non-extensible invariants, and bounded cycle rejection.
Object.isExtensible returns false for primitives; preventExtensions returns them
unchanged and closes objects without freezing their existing properties.
Object.create allocates with its supplied object/null prototype before optional
property definition. Object.defineProperties snapshots own keys, rechecks each own
descriptor's enumerability, and converts all selected descriptors before the first
definition (20.1.2.3.1). Conversion failure performs no definitions; user getter
side effects remain observable. Definition failure retains earlier successful
definitions. Key copying and sorting consume
bounded work. Enumeration of the incomplete Array constructor reports Unsupported
until its own key set is complete. Object.freeze/seal and
isFrozen/isSealed implement SetIntegrityLevel/TestIntegrityLevel (7.3.15–16).
They preserve accessor identity without invoking getters, perform shallow changes,
and use mapped-argument descriptor rules when freezing indexed properties.
Extensible objects fail integrity predicates before key enumeration.
Object.assign snapshots each source's keys and then rechecks enumerability before
ordinary Get/Set, retaining getter/setter effects and earlier copies on abrupt
completion. Object.getOwnPropertyDescriptors creates ordinary own data properties
containing fresh descriptor objects, preserving accessors without invoking them.
Object.getOwnPropertyNames/getOwnPropertySymbols perform ToObject, snapshot own
keys, and filter by key type without reading descriptors or values (20.1.2.10–11).
Object.keys/values/entries use EnumerableOwnProperties (7.3.23): snapshot once,
skip Symbol keys, and recheck each own descriptor's enumerability immediately
before visiting it. Keys never invokes getters. Values and entries perform live
Get with the original object as receiver; entries create each intrinsic pair
before proceeding. Getter additions outside the snapshot are ignored; deletions
and descriptor changes affect later visits. Abrupt reads stop immediately.
CreateArrayFromList (7.3.17) starts an intrinsic Array at length zero and defines
own writable/enumerable/configurable data elements in order. Results never call
the public Array constructor, species getters, or inherited indexed setters.
Object.fromEntries uses AddEntriesFromIterable with a private data-property adder
(20.1.2.7, 24.1.1.2). Require a non-nullish iterable, then create an ordinary
intrinsic-prototype object before acquiring the iterator. Read each object's
properties 0 and 1 before ToPropertyKey; entries themselves are never iterated.
Create writable/enumerable/configurable own data properties, preserving Symbols,
overwriting duplicate keys in place, and bypassing inherited setters. Invalid
entries and entry-read/conversion/definition throws close the iterator and preserve
the original throw over cleanup language errors. Step failures and host failures
do not close; cleanup host failures remain host failures.
Object.groupBy uses property-key GroupBy (20.1.2.13). Validate nullish inputs and
callability before iterator acquisition, then call with undefined this and exactly
value/index. Convert returned keys before adding each original value to its group.
Preserve first-key and within-group order with string code-unit and Symbol identity
comparisons. Checked temporary storage and comparisons charge opted-in work.
The specification's safe-integer counter bound closes before the next step.
Callback/key-conversion throws close; step/host failures do not. After exhaustion,
create a null-prototype object and dense intrinsic Arrays with own data properties.
Materialization failures occur after grouping and cannot close the exhausted
iterator. All required Object constructor properties are now materialized, so its
own-key reflection, copying, and integrity operations are supported.

Object method/accessor syntax reuses shared function bodies and source capture,
with distinct property kinds for methods, getters, and setters (13.2.5, 15.4).
Method parameters are unique even in sloppy code; getters have no parameters,
and setters take one FormalParameter without a trailing comma or rest marker.
Computed keys parse in the outer new.target context, while parameters and bodies
use a new non-arrow function context. Validation resets labels/loop targets and
checks inherited strictness, parameter/lexical collisions, and non-simple strict
directives. Contextual get/set prefixes must be unescaped; methods named get/set
remain ordinary methods. Only colon-form non-computed __proto__ definitions are
prototype setters. Runtime evaluation converts each key before creating the
closure and captures the current environment without a private function-name
binding. OrdinaryFunctionCreate without MakeConstructor produces non-constructible
methods with no own prototype property (15.4.4–5). MakeMethod retains a traced
HomeObject edge even for detached methods (10.2.7); unrooted cycles are collectible.
SetFunctionName preserves UTF-16 keys and Symbol descriptions, includes get/set
prefixes, and checks output limits before prefix allocation (10.2.9). Accessor
definitions merge omitted getter/setter fields in source order; data/accessor
transitions use ordinary descriptor rules and bypass inherited setters. Calls
reuse ordinary this, arguments, parameter, strictness, and new.target semantics.
Function.prototype.toString retains the complete method definition source.
Async/generator methods and derived-constructor super calls remain separate gaps.

SuperProperty syntax retains a distinct reference node (13.3.7). GetThisBinding
precedes computed-name evaluation; GetSuperBase reads the home object's current
prototype after that expression, before property-key conversion. The reference
retains this value separately from its base (6.2.5). GetValue and PutValue use the
existing receiver-aware internal property operations, so inherited accessors and
calls receive the actual method receiver, while data writes update that receiver.
Simple assignment converts a computed name after its RHS; compound/update forms
cache the key after GetValue. Prototype changes during conversion or the RHS do
not replace the retained base. Null super bases throw TypeError before key
conversion. Delete throws ReferenceError before base/key coercion (13.5.1.2),
while still evaluating a computed-name expression. Arrows and direct eval inherit
the nearest non-arrow function's home object; nested ordinary functions reset
the context. Home-object environment edges are traced with captured closures.

Function.prototype.call forwards thisArg unchanged and consumes the remaining
arguments through iterative tail dispatch (20.2.3.3), avoiding Rust stack growth.
Function.prototype.toString emits `function NAME() { [native code] }` for builtin
functions, using immutable builtin identity for [[InitialName]] (20.2.3.5). It
never reads the public name property. Generated builtin strings respect the realm
length limit, including during implicit coercion. Arrow functions return their exact retained source, including internal comments
and formatting, independently of the public name property.

Function.prototype.apply checks callability before inspecting argArray, treats
nullish lists as empty, and otherwise performs object-only CreateListFromArrayLike
(20.2.3.1, 7.3.19). Length conversion uses ToLength; indexed reads include inherited
properties and getters, preserve the list receiver, and finish before the target
call. Direct calls and apply share an optional argument-list quota, disabled by
default. Opted-in work and argument quotas are checked before building a large
list. Indexed traversal retains ToLength's full width, and reservation is checked
as values are appended.

Bound function exotic records capture an unrooted target handle, a receiver value,
and an argument list (10.4.1). The heap validates and traces every captured edge,
including cycles through ordinary properties. Invocation ignores later receivers,
prepends captured arguments, checks the combined argument limit, and transfers to
the target iteratively. Capture copies consume work before allocation. Bind copies
the target prototype before reading length/name, uses only own Number-valued length,
and preserves exact UTF-16 name units with the standard "bound " prefix (20.2.3.2).
Native source for bound functions is `function () { [native code] }`. Bound objects
inherit constructibility from their target at creation; adding a public prototype
property cannot make an arrow or builtin method constructible.

EvaluateNew evaluates the constructor and all arguments before IsConstructor
(13.3.5.1.1). Ordinary construction reads the current newTarget.prototype after
arguments finish, falls back to the constructor realm's Object.prototype for any
non-object value, and creates a fresh receiver (10.1.13–14, 10.2.2). Parameter and
body evaluation share ordinary call semantics; object returns replace the receiver,
while primitive returns preserve it. Bound construction prepends captures, ignores
bound this, substitutes its target for a matching newTarget, and forwards iteratively
(10.4.1.2). Its own prototype property is never read on this path. Construction shares
call-depth, argument, allocation, and work limits; every abrupt result restores
caller scopes/strictness/depth. Function environments retain a traced newTarget
handle for construction and undefined for calls (9.1.1.3). GetNewTarget finds the
nearest non-arrow function environment, so arrows retain the original invocation
binding through nested scopes and after return (9.4.5, 13.3.12). Bound construction
exposes the substituted target, and nested ordinary calls reset the binding.
Derived constructors remain unimplemented.

Reflect is an ordinary object inheriting Object.prototype, with a configurable,
non-writable Symbol.toStringTag and retained method roots (28.1). Reflect.apply
checks callability before CreateListFromArrayLike and transfers to the existing
tail-call dispatch with the original receiver (28.1.1). Reflect.construct checks
target and present newTarget before reading the array-like list; an absent third
argument defaults to target, while explicit undefined is rejected (28.1.2).
Construction reuses ordinary, builtin, and iterative bound dispatch with an
explicit newTarget, preserving prototype lookup order and bound substitution.
Neither method reads Symbol.iterator. Argument-list copies, opted-in quotas,
platform capacity, and native-stack guards follow existing call rules. All thirteen
Reflect methods are exposed for currently implemented object kinds; Proxy traps
remain pending.

Reflect.getPrototypeOf, setPrototypeOf, isExtensible, and preventExtensions
(28.1.7/9/11/13) require object targets without boxing or conversion. They share
the ordinary/exotic storage operations used by Object methods. SetPrototypeOf
validates an object-or-null prototype and returns the internal boolean rather
than converting rejection into TypeError: equal current prototypes succeed even
after extension prevention; cycles, immutable-prototype changes, and new
prototypes on non-extensible objects return false. PreventExtensions returns true
for the currently exposed object kinds. Proxies remain a separate implementation.

Reflect.get, has, and deleteProperty (28.1.4/5/8) validate object targets before
ToPropertyKey, then share receiver-aware reads, prototype-aware presence checks,
and own-property deletion. Get defaults receiver only when the argument is absent;
explicit undefined, null, and primitives reach accessors unchanged. Data reads
ignore receiver, and has/delete do not invoke property getters. Delete returns
false for non-configurable properties regardless of caller strictness. Exposed
Array, String, and arguments exotics retain their indexed/length and parameter-map
rules. Missing intrinsic values/mutations and host failures remain Unsupported or
Limit, so language handlers cannot conceal them.

Reflect.defineProperty (28.1.3) reuses ordered ToPropertyKey/ToPropertyDescriptor
conversion and returns the internal DefineOwnProperty boolean. Invalid descriptors
and invalid Array lengths still throw; partial Array truncation preserves its
specified state on false. GetOwnPropertyDescriptor (28.1.6) requires an object
target and reuses FromPropertyDescriptor without invoking accessors. OwnKeys
(28.1.10) preserves all String/Symbol keys, including non-enumerables, in internal
key order and creates a fresh intrinsic Array with own data elements. It bypasses
species, constructors, and inherited setters. Incomplete intrinsic key lists and
descriptors remain explicit host gaps.

Reflect.set (28.1.12) shares checked OrdinarySet/OrdinarySetWithOwnDescriptor with
ordinary assignments (10.1.9.1–2). Walk target descriptors before consulting an
object receiver's own descriptor. Target accessors call their setter with the
exact receiver and return true regardless of its result; absent setters and
read-only data return false. Writable data defines or updates the receiver,
bypassing its inherited setters and retaining existing attributes. Primitive
receivers reject data writes but reach setters unchanged. Definition dispatch
preserves Array length/index rules, String read-only characters, and mapped
arguments. Missing intrinsic descriptors remain Unsupported before mutation,
including on a distinct receiver. Reflect now has complete own-key/descriptor
reflection and integrity operations; its thirteen methods and tag remain rooted.

Math is an ordinary Object.prototype object with eight fixed, non-enumerable
mathematical constants and a configurable, non-writable Symbol.toStringTag
(21.3.1). Abs, sign, ceil, floor, round, and trunc perform ToNumber once, ignore
the call receiver, and convert no extra arguments (21.3.2). Abs clears negative
zero; sign preserves either zero and NaN. Ceil/floor/trunc use exact binary64
integral rounding, including signed zero and infinities. Math.round
([sec-math.round](https://tc39.es/ecma262/#sec-math.round)) chooses the nearest
integer with halfway ties toward positive infinity, retains negative zero for
[-0.5, 0], and compares the fractional remainder without adding 0.5 to the input.
This avoids rounding near half a unit and changing odd integral values above
2^52. The object and all standard methods retain intrinsic roots.

Math max/min (sec-math.max/min) convert every argument in order before deciding
whether NaN wins. A running extremum and NaN flag avoid a second argument list;
interleaved comparisons have no observable effect. Equal zeros select +0 for max
and -0 for min. Empty lists retain -Infinity/+Infinity respectively. Clz32 uses
the shared ToUint32 conversion and counts leading zero bits (sec-math.clz32).
Imul converts left then right, uses safe wrapping u32 multiplication, interprets
the result as i32, and converts it exactly to Number (sec-math.imul).

Math fround narrows to binary32 with ties to even and widens exactly. F16round
rounds directly from binary64 using binary16 spacing (sec-math.f16round): a fixed
2^-24 quantum for subnormals and the exact power of two determined by the normal
exponent. Exact scaling, round_ties_even, and exact rescaling retain halfway
behavior without a binary32 intermediate. Magnitudes at or above 65520 overflow
to signed infinity; smaller underflowed results retain the input sign. No new
numeric dependency or unstable Rust float type is required.

Math pow applies ToNumber to the base and then the exponent before using the
same Number::exponentiate algorithm as ** (sec-math.pow). Its JavaScript special
values, signed zero, odd integral exponents, and negative fractional bases follow
that shared implementation; BigInt arguments are rejected by ToNumber. Sqrt
performs one conversion and uses the correctly rounded IEEE binary64 squareRoot
operation, preserving -0 and rejecting negative numbers as NaN (sec-math.sqrt).

Math exp/expm1 and log/log1p/log2/log10 perform one ToNumber conversion
(sec-math.exp through sec-math.log2). Domain endpoints and signed-zero results
are explicit. Remaining finite results use the platform's approximation, with
exp_m1 and ln_1p retaining small-input accuracy instead of subtracting one or
adding one first. Log2 decodes every binary64 power of two exactly, including
subnormal powers, before using the platform logarithm for other finite values.

Math acos/acosh/asin/asinh/atan/atanh/cbrt handle their domains and signed
endpoints before platform finite approximations (sec-math.acos through
sec-math.cbrt). Acosh/asinh magnitudes above 2^28 use log(x)+LN2, avoiding
intermediate doubling overflow in Rust 1.85's standard-library implementations;
the omitted O(1/x^2) correction is below binary64 spacing there. Atanh evaluates
0.5*log1p(2*abs(x)/(1-abs(x))) and restores the sign, preventing a negative
interior argument from rounding to the -1 logarithm endpoint. Atan2 converts y
then x before any numeric shortcut
(sec-math.atan2). NaN wins after both conversions; explicit zero and infinity
quadrants retain the sign of y and distinguish either sign of x, including -0.
Ordinary finite nonzero pairs use atan2 directly without forming y/x.

Math sin/cos/tan and sinh/cosh/tanh perform one ToNumber conversion
(sec-math.cos/cosh/sin/sinh/tan/tanh). Circular functions reject either infinity
as NaN; odd functions retain both zero signs. Cosh maps either infinity to
+Infinity, sinh retains its sign, and tanh maps it to the corresponding unit.
Cos/cosh return exactly one at either zero. Other finite inputs use the platform
approximations, including large-argument reduction and subnormal inputs.

Math hypot converts all arguments in order before returning Infinity or NaN
([sec-math.hypot](https://tc39.es/ecma262/#sec-math.hypot)). Infinity wins over
NaN after successful conversion. Empty/all-zero calls return +0. A running
maximum magnitude scales a compensated sum of squares, preventing intermediate
square overflow/underflow and retaining small contributions beside larger ones.
Rescaling has no observable effect and stores no second argument list. Finite
results are implementation approximations; final overflow may return Infinity.

Math sumPrecise consumes a synchronous iterable without converting its elements
([sec-math.sumprecise](https://tc39.es/ecma262/#sec-math.sumprecise)). Non-Number
elements throw TypeError and close the iterator; IteratorStepValue errors do not
close. All steps continue after NaN or infinity, so later failures remain visible.
The 2^53-1 element bound is checked after each successful step and produces a
RangeError with iterator closing. Host failures remain outside JavaScript cleanup.

Finite sums use separate positive/negative integer accumulators in units of
2^-1074. Each has 34 u64 words: finite magnitudes below 2^1024 and the specification
count bound keep either total below 2^2151 units, within their 2176-bit capacity.
This is a specification-derived storage size, independent of host quotas and
JavaScript BigInt. Subtract the exact magnitudes, then round once using retained,
guard, and sticky bits with ties to even and signed overflow. Subnormal sums
are exact in the accumulator unit. Empty/all-minus-zero inputs return -0;
other exact cancellation returns +0. No external numeric dependency is needed.

Math random uses a realm-specific SplitMix64 state (sec-math.random), initialized
from the standard library's randomized RandomState and a checked process-wide
unique sequence identity. The intentional wrapping Weyl increment and bijective
mixer give a 2^64 state period. The projected Number sequence retains that
period: a checked pair of outputs half a cycle apart differ, ruling out every
proper period (which would divide 2^63). Distinct starting states therefore give
distinct Number sequences. Taking the high 53 bits before exact scaling by
2^-53 yields positive Numbers in [0, 1), with no chance of rounding up to one.
Sequence identities cannot wrap or be reused; exhaustion is a platform capacity
failure. No external dependency or default resource quota is introduced. Receiver
and arguments are ignored. Math's complete method inventory enables ordinary
own-key/descriptor reflection, copying, enumeration, sealing, and freezing.

Instanceof uses relational precedence, evaluates both operands, then follows
InstanceofOperator and OrdinaryHasInstance (13.10.2, 7.3.21). Bound functions
re-enter the operator on their target through an iterative loop. Primitive left
operands return false before reading prototype; object operands require an object
prototype and compare its identity with ancestors. Callable arrows can participate
without being constructors. Non-callable objects inheriting the default hook return
false; non-callable objects without a hook throw TypeError.

Function.prototype owns the fixed, non-writable, non-configurable symbol-keyed
hasInstance callable (20.2.3.6). GetMethod reads custom/inherited hooks before
checking target callability; calls retain the original receiver and left operand,
and results use ToBoolean without coercion. Null/undefined hooks fall back to
OrdinaryHasInstance. Direct intrinsic calls bypass the receiver's own hook, but
bound targets re-enter InstanceofOperator and observe their current hook even for
primitive left operands. Lookup of the exact intrinsic forwards iteratively to
avoid native stack growth across deep bound chains; it still checks argument
limits and charges property traversal. Other handlers use ordinary bounded Call.
Native-injection and Script tests cover symbol access.

Accessor/coercion calls that re-enter execution have a fixed host nesting limit of
32 until explicit frames replace Rust recursion. Every success and abrupt result
restores the counter; iterative call/apply/bound transfers do not increase it.
Host limit failures continue to bypass JavaScript catch/finally handlers.
Native builtin algorithms use a separate non-inlined dispatcher so extending the
standard library does not enlarge every recursive script call's native frame.
Embedders must provide at least a 2 MiB native thread stack. Regression tests run
recursive defaults, ordinary calls, constructors, and coercion on an explicit
2 MiB stack in the Linux/Windows toolchain matrix. A previous 64-call bound could
overflow Windows stable Rust debug test threads before returning a host error.

Arrows share immutable parameter/body syntax and retained source
through safe Rc values. Creating a closure captures an environment handle; invoking
it allocates a fresh parameter environment whose outer is the captured environment,
never the caller’s scope. Missing parameters are undefined, extra arguments are
ignored after evaluation, and parameters remain mutable. Arrows create no arguments
binding or constructor/prototype property. Lexical this follows captured function
environments and otherwise resolves to the realm's global this value.
Every call restores caller strictness and active scopes on success or abrupt exit.
The evaluator bounds combined statement/expression nesting across calls to 64,
as well as call re-entry; ordinary user tail calls await explicit execution frames.
Call expressions and expression statements use separate small dispatch frames.
Indirect eval bypasses the native-algorithm match frame, so recursive Script
compilation/execution does not retain storage for unrelated expression, statement,
or builtin branches. This keeps the existing native-depth guards effective on a
2 MiB embedding-thread stack, including ordinary debug builds. Quotas and guard
thresholds are unchanged.

With simple parameter lists, block bodies instantiate all function vars in the
parameter environment before execution, preserving existing parameter values
and initializing other vars to
undefined (10.2.11). Body lexical declarations begin uninitialized; non-strict
bodies use a separate lexical environment. Nested functions do not contribute
vars to their enclosing scope. A normal body completion produces undefined.
Return completions always carry a value, including undefined for bare returns,
and propagate through statements until invocation consumes them (14.10, 15.3.3).
Finally may replace a language completion, but cannot intercept host failures.

Parameters begin uninitialized and initialize left to right. A default runs only
for an undefined argument, with anonymous function name inference (8.6.3).
Parameter expressions cannot see body declarations, including through captured
closures. When nested defaults, computed keys, or top-level defaults are present,
body vars get a separate environment and
same-named vars copy the initialized parameter value (10.2.11). Function length
counts parameters before the first top-level default or rest (15.1.5). Partial initialization and
escaped default closures remain traced after an abrupt completion.

Ordinary function objects share code/source and environment capture storage with
arrows. They own name and length properties plus a writable, non-enumerable,
non-configurable prototype property; its ordinary object owns a writable,
non-enumerable, configurable constructor backlink (10.2.5). Named expressions
capture an extra environment with an immutable self binding (15.2.5). This binding
ignores non-strict writes and rejects strict writes; const bindings always reject
writes (9.1.1.1.5). Captures, self bindings, and prototype cycles are all traced.
Strict ordinary calls use a function environment with a traced this binding and
an immutable arguments binding. The original receiver is preserved, including
undefined, null, and primitives (10.2.1.2). Arrows create neither binding and
resolve this/arguments through captured outer environments (9.4.4).

Strict arguments objects are unmapped ordinary objects (10.4.4.6): indexed values
are writable/enumerable/configurable, length is writable/configurable, and callee
is a non-configurable accessor using %ThrowTypeError%. Parameter writes do not
alias indices, or vice versa. Argument values and receiver captures remain traced
after returns or abrupt default initialization. Their own Symbol.iterator property
aliases the intrinsic Array values function, independent of public replacements.
Object.prototype.toString recognizes the Arguments tag. Boolean, Number, String,
and Symbol non-strict receivers are boxed; other
primitive receivers still report Unsupported until their wrappers are implemented. Call failures restore strictness, scopes, and nesting counters.

Non-strict simple parameter lists use mapped arguments (10.4.4.1–7). Internal maps
store checked environment handles and parameter names instead of exposing hidden
getter/setter objects. Only the last duplicate occurrence maps, even when it has
no corresponding argument. Reads and complete descriptors consult the current
binding. Writes update the same cell; successful deletion, accessor conversion,
or a non-writable descriptor detaches it. Freezing without a value snapshots the
current parameter value first. Failed descriptors and work limits leave both
storage and aliases unchanged. Writes through other receivers preserve the
original alias, and removing the final alias releases the environment edge.
Non-simple lists use unmapped arguments and a separate parameter environment in
non-strict code. Parameter/body names decide whether an arguments binding is
created; non-strict arguments bindings remain mutable (10.2.11).

NamedEvaluation supplies names for binding initializers, bare identifier assignment
and logical-assignment targets, and ordinary object property values. Parenthesized
RHS anonymous ordinary/arrow function definitions retain inference; parenthesized LHS identifiers do not
(IsIdentifierRef, 8.4.4). The non-computed prototype setter also excludes inference.
Captured environments, functions, and ordinary objects form one traced graph, so
shared bindings survive caller exit and unreachable closure cycles are reclaimed.

Modules use standard module records and host resolution hooks. Promises use a job
queue. Async functions and generators require resumable execution, which can later
use explicit interpreter frames. Module loading, clocks, entropy, and agent creation
are host capabilities. The default host exposes no non-standard JavaScript globals.
Test262 host helpers exist only in the conformance host.

## Validation

Use unit tests for algorithms and integration tests for observable behavior. Use
insta snapshots for syntax trees and diagnostics, with explicit semantic assertions
for execution. Test invalid input, early errors, evaluation order, coercion, signed
zero, NaN, UTF-16, ASI, and resource exhaustion alongside ordinary cases.

Pin Test262 to an exact revision. Run each test in a fresh realm with the required
strict and non-strict modes, includes, and host helpers. Match negative tests by
both phase and error type. Never treat all errors as success. Report unsupported,
skipped, failed, timed-out, and passed cases separately. A supported subset is a
regression gate, not a whole-suite conformance percentage.

The pinned default assert.js and sta.js scripts are executed unchanged for non-raw
positive regressions. Harness files have a separate checksummed manifest mode;
they are support code, not test cases, and their frontmatter is not interpreted as
test metadata. Local controls check successful assertions, Test262Error identity,
and deliberate assertion failures against those exact files. Diagnostic paths
requiring other missing facilities stay Unsupported and never count as passes.
String comparison assertion diagnostics use the implemented JSON serializer.
Additional includes and host capabilities join coverage only when their execution
paths are implemented and reviewed.

Build the runner in stages. First parse a documented subset of Test262 frontmatter
without adding a YAML dependency, rejecting unsupported metadata forms explicitly.
Plan strict, non-strict, module, and raw execution without rewriting test bodies;
raw sources retain their exact bytes. Preserve harness include order. Then match
results by phase and error type, keeping harness/setup failures separate from the
test result. Modules, async execution, and host helpers remain explicitly unsupported
until implemented. While the grammar is partial, parse-negative passes additionally
require a reviewed expected diagnostic span and message; an arbitrary syntax error
is an unverified result, not a conformance pass. Conformance realms reserve missing
host helper globals, so reading or testing their type reports Unsupported rather
than producing an accidental ReferenceError or undefined. Ordinary realms receive
no Test262 globals. Parse-negative tests need no harness evaluation.

Use CI for formatting, Clippy, unit tests, integration tests, documentation tests,
snapshot review, and a dependency-policy check. Keep production crates buildable
without development dependencies. Commit the lockfile. Add deterministic corpus
and fuzz-style tests without requiring an external runtime dependency. Differential
checks against other engines are supplementary evidence, never the specification.

## References

- [Published specification](https://262.ecma-international.org/17.0/)
- [Publication record](https://ecma-international.org/publications-and-standards/standards/ecma-262/)
- [Living specification](https://tc39.es/ecma262/)
- [Test262 execution rules](https://github.com/tc39/test262/blob/main/INTERPRETING.md)

JSON text has a dedicated ECMA-404 parser (25.5.1, ParseJSON). It accepts exactly
JSON whitespace, decimal numbers, double-quoted strings and JSON escapes, and
object/array/literal grammar; ECMAScript extensions are syntax errors. Keep UTF-16
units, including raw and escaped lone surrogates. Preserve source-order duplicate
object entries and exact code-unit ranges for reviver source contexts.
The flat postorder tree uses child indices and explicit container states, so parsing
and dropping deeply nested JSON never recurse on the native stack. Every growing
buffer reserves fallibly; a caller-supplied work callback can abort separately from
syntax errors, without imposing a default quota. Number conversion uses correctly
rounded binary64 decimal parsing and preserves negative zero, overflow, and
underflow. JSON.parse converts text once with the string hint, reports invalid
text as an intrinsic SyntaxError, and materializes the flat tree iteratively.
Objects and dense arrays use retained intrinsic prototypes. Own writable,
enumerable, configurable data properties bypass inherited setters; duplicate
names replace earlier values without changing their initial property order, and
__proto__ is an ordinary JSON key. Noncallable revivers are ignored. Callable
revivers use an immutable flat parse snapshot and explicit traversal frames.
Get precedes creation of each fresh context object; SameValue determines whether
the original primitive lexeme or child snapshot remains available. Duplicate
names use the last source occurrence. Snapshot enumerable string keys or array
length before visiting children, then invoke the reviver in postorder with its
holder as this and exactly three arguments. Read each child live, so mutations,
inherited values, getters, and newly encountered objects retain their specified
effects. Undefined deletes a child; other returns create own data properties,
bypassing setters. Ignore false from those updates while propagating abrupt
completions. The root holder has an own empty-string property, and its callback
result is returned directly. Source ranges and traversal remain iterative even
for deeply nested JSON; user-created cycles can be aborted with opt-in work limits.
JSON.rawJSON performs string-hint conversion once, checks the edition-17 leading
and trailing code-unit restrictions, and validates strict primitive JSON text.
It preserves the original text even when the parsed Number would round or
overflow. Returned objects have null prototypes, an internal IsRawJSON brand,
and an enumerable frozen rawJSON data property. JSON.isRawJSON checks that slot
without property reads or coercion; inheritance and copying cannot transfer it.
The brand contributes one slot to shared heap accounting and survives collection.
JSON.stringify processes callable or array replacers before space conversion.
Deduplicate converted string/number keys while preserving their order, and apply
that list to every ordinary object, including inherited/nonenumerable properties.
Gap conversion uses numeric truncation/clamping or the first ten UTF-16 units.
Serialize each property with live Get, then callable toJSON, then the replacer,
then raw branding or wrapper conversion. BigInt throws unless a hook replaces it;
omitted array values become null, while omitted object values are excluded.
QuoteJSONString uses lowercase escapes for controls and lone surrogates, retaining
valid surrogate pairs. Retained intrinsic prototypes create the root holder.
Explicit frames snapshot object keys or array length and emit into one fallible
UTF-16 buffer. A set of active object identities detects cycles after hooks while
allowing repeated aliases; heap identity and slot generation are part of hashing.
Indentation grows and shrinks with frames, preserving empty-container formatting.
The complete JSON method inventory supports reflection and integrity operations.
No default serialization nesting or resource quota is introduced.
Materialization has no default nesting or resource quota; opted-in work, heap,
and final decoded string/property-name quotas retain their host-abort behavior.

## Map keyed collections

Map (24.1) stores canonical collection keys in a standard randomized HashMap
that indexes a stable vector of ordered entries. Keys compare by primitive type,
UTF-16 string content, normalized BigInt magnitude/sign, Symbol identity, or
validated heap handle identity. All NaNs share a hash identity; signed zero is
stored as positive zero. No key conversion invokes user code. Expected sublinear
lookup meets the specification's performance requirement. Updates preserve an
entry's position, while deletion and clear erase values and retain vacant positions
so suspended iterators and forEach observe later appends correctly. Historical
vacant positions consume addressable storage until the Map is collected; they
retain no keys or values. This representation sets no default host quota.

MapData traces each live key/value. Iterators retain their source Map until
completion and then release it permanently. Storage validates handles, charges
opted-in work, reserves new hash/vector capacity before inserting, and executes
no JavaScript under a heap borrow. BigInt and heap handle hashes agree with their
existing equality rules. Map.prototype is ordinary and has no MapData brand;
methods and size reject inherited or forged slots regardless of public prototypes.

Construction requires NewTarget and reads its prototype before the instance's
set method. Null/undefined input returns before reading set. AddEntriesFromIterable
caches the adder and next method, reads entry zero before one, closes entry or
adder errors, and never closes next/done/value failures. Incoming throw completions
win over closing errors; host aborts skip script cleanup. Computed insertion
validates its callback before lookup, passes a canonical key with undefined this,
and looks up the entry again after callback mutation before assigning the result.
ForEach and Map iterators use live ordered cursors and expose reinsertions/appends.
GroupBy checks its safe-integer index before stepping, closes callback errors,
uses canonical collection keys without coercion, and materializes retained intrinsic
Map and Array objects after exhaustion without reading public constructors or set.
The complete edition-17 Map and Map Iterator inventories support ordinary reflection.

## Set keyed collections

Set (24.2) shares Map's canonical key identity rules and uses its own ordered
hash-indexed SetData slot. Its distinct brand and iterator state reject Map,
prototype, and inherited lookalikes. Deletion and clear retain vacant positions
without retaining values; live iterators and forEach resume the original ordered
list and observe appends/reinsertions. Completed iterators release their source.
Set values/keys/Symbol.iterator share one native function identity. Construction
caches add and next, closes adder throws, and propagates next/done/value failures
without cleanup. Methods mutate frozen instances' internal slots normally.

GetSetRecord reads and converts size, rejects NaN and negative integral sizes,
then reads and validates has before keys. It accepts arbitrary set-like objects,
including Map. Combination/predicate methods check the receiver's brand before
any other-object reads and preserve the exact size-dependent branch and order.
Difference traverses copied data for its has branch, while intersection, subset,
and disjoint predicates traverse live receiver data after observable has calls.
Union and symmetricDifference copy the receiver only after keys and cached-next
acquisition. Other-key duplicates are deduplicated, zero is canonicalized, and
symmetricDifference checks the receiver's live membership at each key. Superset
and disjoint predicates close key iterators on normal short-circuit returns;
cleanup errors replace those returns. Step failures propagate without closing.
Results are allocated with the retained intrinsic Set prototype after traversal,
bypassing public constructors, species, and add. Snapshots preserve vacant positions
and checked capacity/work; there are no default host limits. The complete edition-17
Set/Set Iterator inventories support ordinary reflection and integrity operations.

## Base classes

Base ClassDefinitionEvaluation (15.7.14–16) creates a declarative environment
with an initially uninitialized, immutable internal class-name binding. Computed
names execute in that environment under strict mode, retaining the outer this,
new.target, and super context. The name is initialized only after every method
has been defined. Class declarations separately initialize their mutable lexical
binding in the enclosing environment and produce an empty statement completion.
Every language or host failure restores the enclosing environment and strictness.

Explicit and synthesized empty base constructors use shared ordinary function
code with a distinct class-constructor callable kind and the class prototype as
their home object. Calling a class throws TypeError after evaluating arguments;
its callable metadata still permits apply to read the argument list first.
Construction shares ordinary base allocation, newTarget prototype selection,
strict parameter/body execution, and object-versus-primitive return rules. The
empty default constructor implements the base branch of 15.7.14 and initializes
instance elements. Async/generator methods remain Unsupported. Existing
native-stack guards and opt-in quotas are unchanged.

Class constructors inherit Function.prototype and own a non-writable,
non-enumerable, non-configurable prototype property. Their prototype's constructor
link and instance/static method properties are writable, non-enumerable, and
configurable. Accessor pairs use the shared method descriptor algorithm. Methods
remain non-constructible and retain their home object. Internal environment and
home-object edges are traced, including unreachable constructor/prototype cycles.
Class and constructor stringification retain the complete class source; method
source excludes the static modifier. Anonymous class name inference supplies the
name before computed names and static definitions, allowing a static name method
to replace it without creating an internal binding for the inferred name.

## Class heritage and derived constructors

ClassHeritage evaluates in the strict class-name environment (15.7.14). Null gives
instances a null prototype while the constructor inherits Function.prototype.
Other values must be constructors before their prototype property is read; that
property must be an object or null. Both constructor and instance prototype links
are established before computed class elements run. Every failure restores the
outer class evaluation context and leaves an uninitialized declaration binding.

Derived function environments distinguish uninitialized this from an initialized
undefined receiver. They retain and trace the active derived constructor for
GetSuperConstructor through arrows and direct eval. SuperCall (13.3.7.1) captures
the active constructor's internal prototype before evaluating arguments, validates
constructibility afterward, and forwards the current newTarget. BindThisValue
runs after construction, so repeated super calls still evaluate arguments and
run the superclass before throwing ReferenceError. Reads of this and super
properties throw before initialization, including before computed property names.
Derived constructors return objects directly, reject other non-undefined returns
with TypeError, and resolve the saved function environment's this binding for
undefined returns after restoring the caller context (10.2.2).

Default derived constructors forward argument Lists directly, without invoking
Array's iterator. Bound/default forwarding shares the iterative construction
loop; pending default-constructor fields initialize on the returned object in
superclass-to-subclass order, including private fields. Explicit
constructors, superclass calls, and recursive computed names retain the existing call/evaluation/parser stack guards. The
SuperCall expression uses the small call dispatch path to avoid retaining the
general debug expression frame during recursive construction. Heritage, super,
and default forwarding introduce no default resource quotas.

## Public class fields

ClassFieldDefinitionEvaluation evaluates and converts every public key alongside
method definitions before any static initializer runs (15.7.10/14). Field records
hold immutable shared syntax, the strict class lexical environment, and the instance
prototype or constructor as home object. Constructors retain their instance record
List in shared storage and trace initializer environments/home objects. Every edge
is validated before installing the List; opted-in work exhaustion leaves it intact.
The internal class name is initialized before static fields run, even when a static
initializer subsequently throws and leaves the outer declaration binding in its TDZ.

DefineField evaluates an initializer in its own strict function environment, with
initialized this, an undefined new.target, a super property binding, and the field
name used by NamedEvaluation (7.3.32, 15.2.3). It omits declaration instantiation;
the initializer closure is internal and cannot be obtained by JavaScript. Arrows
inherit that environment. Parsing and direct eval reject ContainsArguments through
arrows and computed method names while ordinary function/method bodies establish
new boundaries. Super calls are forbidden in initializer context. Caller scopes,
strictness, and variable environments restore after language and host failures.

CreateDataPropertyOrThrow defines an enumerable/writable/configurable own property
without invoking inherited setters. Initializers run before descriptor rejection;
fields that precede an abrupt completion remain installed. Base fields initialize
before parameter defaults/body execution. Explicit derived fields initialize after
BindThisValue and before SuperCall returns; failure leaves this initialized, and
repeated super calls never repeat fields. Returning an object without super skips
derived fields. Default derived construction retains pending field Lists in the
iterative forwarding loop and initializes them from the superclass outward on the
actual returned object, without Array iteration or a default resource quota.

## Static initialization blocks

ClassStaticBlockDefinitionEvaluation creates internal strict function code with
empty parameters, the class lexical environment, and the constructor as home
object (15.7.11). Static blocks join static fields in the ordered initialization
List. All computed keys and methods finish first; the internal class name is
initialized before the List runs. An abrupt block preserves preceding effects
and leaves an enclosing declaration binding uninitialized.

EvaluateClassStaticBlockBody performs ordinary function declaration instantiation
with an empty argument List, evaluates the statements, and discards normal values
(15.2.3). Each block has its own lexical/variable scope. Its this is the constructor,
new.target is undefined, and super properties retain the constructor as receiver.
Escaping closures retain those bindings and use existing environment/home-object
tracing. Caller context restores after language and host failures; execution uses
the existing native-call guard and introduces no default quotas.

Static block parsing resets return, loop, and label contexts and enforces strict
declarations, ContainsArguments, SuperCall, and Await early errors. Ordinary
functions/methods establish independent grammar contexts; arrow parameters inherit
the block's Await restriction while their concise bodies reset it. Literal
arguments references are forbidden, including through arrows. Direct eval instead
uses its own grammar context and can observe the block's zero-length unmapped
arguments object, as required by ordinary declaration instantiation.

## Private element grammar

PrivateIdentifier syntax retains its decoded StringValue, including the initial
hash, and its full source range. Class fields/methods/accessors, member references,
optional chains, and private-in expressions use that syntax. Private-in consumes a
ShiftExpression on the right and participates in relational precedence; bare
private identifiers are not ordinary expressions or property names (13.10).

Each class body collects private declarations and references independently of
ordinary bindings. On leaving a body, declarations resolve its references and
unresolved names pass to the containing class. This implements forward references
through functions and nested classes without making a class's names visible in
its own heritage (AllPrivateIdentifiersValid, 16.1.1). Duplicate names fail except
for one getter/setter pair with matching staticness. Escapes normalize before
comparison; private constructor names, deletion, and super access are early errors.

## Private fields and environments

Each ClassDefinitionEvaluation creates fresh opaque private-name identities
(6.2.10/15.7.14). The identities use allocation identity internally and never
become JavaScript Symbols or ordinary property keys. A transparent heap
environment contains the private namespace and links to the class-name lexical
environment. It is created after heritage evaluation, so heritage closures retain
the outer private scope even when the body declares the same spelling. Body code,
computed keys, constructors, methods, fields, and blocks capture the child record;
the internal class binding still belongs to its parent. Private resolution follows
the captured chain iteratively, across ordinary function boundaries.

Private fields occupy separate own storage. PrivateElementFind compares identity
without searching prototypes. Add rejects duplicate names after evaluating the
initializer, and get/set reject missing brands with TypeError (7.3.27–31).
Extensibility and ordinary property descriptors do not affect private fields;
reflection, serialization, and copying never enumerate them. Storage checks object
handles and opt-in work budgets before mutation and traces every stored object
value, including cycles. Private fields do not consume ordinary property capacity.
Public/private instance fields share ordered initialization before base parameters
and after derived BindThisValue; default forwarding initializes superclass fields
before subclass fields. Static fields and blocks retain their source order.

Private references preserve the receiver for calls and defer brand errors to
GetValue/PutValue, so assignment evaluates its right side before a failed write.
Fresh primitive wrappers cannot contain private elements and are omitted under
the GetValue/PutValue notes (6.2.5.5–6). Private-in evaluates its right side, requires
an Object, then resolves the private identity and tests its own storage (13.10).
Field NamedEvaluation uses the private description, including `#`, for inferred
function/class names without exposing the private identity.

Direct eval validates every private identifier against the caller's visible
private spellings before declaration instantiation or execution (19.2.1.3).
Its parsed references resolve through the captured runtime namespace; indirect
eval and dynamic Function inherit no caller private names. Nested class names can
shadow eval's inherited names only within their body. Parser API context booleans
remain separate from the supplied name set.

## Private methods and accessors

Private methods/accessors use the same opaque names and own private storage as
fields (6.2.11/7.3.27–31). A method record retains one shared non-constructible
strict method function. An accessor record retains getter/setter handles; paired
declarations merge at their first list position, in either source order (15.7.14).
SetFunctionName uses `#name`, with `get`/`set` prefixes for accessor functions.
Private methods retain exact source, home objects, ordinary arguments, and
captured lexical/private scope through the existing method call machinery.

InitializeInstanceElements installs every private method/accessor before any
field initializer, before base parameters and after derived BindThisValue
(7.3.33/10.2.2). The default-derived forwarding queue retains both method and
field lists, including classes with methods only, and initializes superclass
elements before subclass elements. Static private methods/accessors install after
internal class-name initialization and before all ordered static fields/blocks.
Duplicate method/accessor installation throws TypeError before field effects.
Ordinary extensibility and property capacity do not restrict private elements.

PrivateGet returns shared method values or calls an accessor getter with the
original object after releasing storage borrows. PrivateSet rejects method writes
and absent setters with TypeError or invokes the setter with that receiver and
the assigned value. An absent getter also throws TypeError; private-in observes
the accessor's brand regardless of which functions exist. Assignment retains
right-side-before-write ordering, while compound/logical updates use normal
GetValue ordering. Object and constructor records trace method/getter/setter
handles. Checked storage validates every callable edge and work budget before
mutation. Recursive calls/accessors use the unchanged native-stack guards.

WeakSet stores a hash index of checked generational object handles and weak
Symbol identities (24.4). Its object handles are unrooted and excluded from Trace;
Symbol keys use weak Arc references with allocation identity preserved after
death. CanBeHeldWeakly consults the internal global Symbol registry, without
calling the public Symbol.keyFor property. Well-known symbols are valid keys.
Hash lookup is sublinear; periodic pruning after a proportional number of new
insertions provides amortized cleanup, with all work/reservation failures checked
before changing the index. Collection charges for weak storage without marking
keys. Presenting a live key preserves identity through cleanup; no enumeration or
liveness observation is exposed. Constructor acquisition and iterator closing
follow the ordinary Set sequence, including host-abort propagation and default
derived forwarding. WeakMap's ephemerons, WeakRef and FinalizationRegistry use the lifetime rules
below.

Object-key WeakMapData uses a dense vector of key/value entries and a hash index
(24.3). Heap handles preserve key identity without rooting their objects. The
ordinary object and heap entry records forward these entries as ephemerons;
values are not enumerated as strong edges. Successful collection removes dead
entries with swap removal and updates moved indices without allocating. Primitive
values, including Symbols, are dropped with inactive entries. Checked storage
validates object keys and values, brands, work and reservations before mutation.
WeakMap exposes its constructor and delete/get/has/set/getOrInsert/
getOrInsertComputed methods for object and non-registered Symbol keys. Receiver
branding precedes CanBeHeldWeakly; computed insertion validates its callback
before lookup. Registered Symbols remain invalid, while boxed Symbols use their
ordinary object identities. Registry checks use internal state. Constructors
reuse the captured adder without prevalidating entry keys.

Computed insertion distinguishes stored undefined from absence and calls its
callback with the original key. It performs a fresh insertion after the callback,
overwriting a same-key callback insertion. Checked storage prepays vector/index
growth before graph mutation. Object and Symbol keys share the conditional
reachability protocol below.

The Symbol-key collector extends conditional reachability to
mixed object/Symbol graphs without treating internal Arc copies as host roots.
A physical census counts every stored Symbol wrapper, including collection
indices and shared class-field/iterator records; marking separately follows
reachable strong fields and activates weak-key values to a fixpoint. External
strong Symbols and recoverable shared Rc records remain roots. Other realms and
permanent Symbol registries conservatively retain their identities. WeakSet keys
have no conditional values and need no candidate census.

A short allocation-free atomic gate orders weak upgrades against logical Symbol
retirement. Final root rechecks happen before any retirement; newly observed host
owners resume marking outside the gate. Allocation, user iterators, callbacks,
formatting, owned drops, cleanup and sweeping stay outside the gate. Checked
scratch allocation, sparse index capacity and all fallible graph validation are
prepaid before commit. A returned collection error preserves entries, handles,
generations and Symbol liveness; reserved capacities may change. The generic heap
Trace/collect path remains unchanged, and Objects uses it until the first
Symbol-key insertion. Existing public JsSymbol/WeakJsSymbol signatures and host
rooting obligations remain unchanged; hidden workspace support and prepared heap
sweeping add an explicit spite_heap::Error::CollectionState variant.

JavaScript Symbol-key insertion activates this joint collector. Collection is
explicitly requested by the host between evaluations. WeakRef uses the same
Symbol retirement protocol; FinalizationRegistry uses the same weak identities.
Resource quotas remain opt-in.

## WeakRef and kept objects

WeakRef construction and deref implement 26.1 over checked, unrooted object handles
or weak Symbol identities. Targets satisfy CanBeHeldWeakly (9.13): objects and
non-registered Symbols, including well-known Symbols, are eligible. Construction
validates the target before reading newTarget.prototype. A non-object prototype
requires GetFunctionRealm validation before intrinsic fallback. Deref requires
the receiver's own internal slot, without unwrapping Proxies or reading public
properties. A stale object generation or retired Symbol produces undefined.

Construction and successful deref append a temporary strong root through
AddToKeptObjects (9.11). The list reserves capacity and charges opted-in work
before using the existing root registry; append has no later fallible operation.
Weak slots themselves are neither tracing edges nor physical Symbol owners.
Atomic Symbol upgrade bridges the transfer into the kept list. Existing root
registry allocation behavior is unchanged.

The host calls Realm::clear_kept_objects after an entire synchronous execution
sequence, including callbacks and reentry, has ended (9.10). A sequence can span
multiple evaluations and host property reads. Those operations and collect never
clear the list automatically, including after abrupt completion. The checkpoint
releases only temporary kept roots, without collection, JavaScript execution,
intrinsic initialization or work-quota reset. Active execution rejects it without
changing roots; detached roots drop outside object borrows and the Symbol gate.
Independent host roots remain live.

The current Realm heaps and execution domains are independent. Hosts grouping
several domains into one synchronous sequence must wait for the entire sequence
before clearing each domain. Shared-Agent and foreign-realm coordination remain
deferred. No automatic collection, default quota or cleanup scheduling is added.

## FinalizationRegistry and explicit cleanup jobs

FinalizationRegistry implements construction, register and unregister (26.2).
Targets and optional unregister tokens are objects or non-registered Symbols,
including well-known Symbols. Held values can be any value except the target
itself. Methods require the receiver's own registry slot; public properties and
Proxy wrappers cannot supply it. Registration does not add targets to kept
objects, and token death alone does not unregister a cell.

A reachable registry strongly retains its callback and held values, while its
targets and tokens remain weak. Successful collection clears dead targets and
queues cleanup captures without running JavaScript. A queued capture keeps the
registry and callback alive. Before removing a selected cell, the driver retains
its held argument through the callback; unregister removes all matching cells,
including cleared cells awaiting delivery.

The host explicitly calls Realm::run_finalization_cleanup_job at an idle turn.
It runs at most one queued registry job and returns NoJob, Completed, or
Reported(Error). NoJob is inert. An outer Err denotes active-execution refusal
or preflight failure before the job starts, preserving queued captures and cells.
A started job reports its first abrupt completion and leaves later pending cells
queued; a removed cell is not retried. Each callback receives undefined as this
and exactly one held value. Object values in returned errors must be rooted by
the host before a later explicit collection.

Cleanup delivery is neither automatic nor guaranteed, and job order is
unspecified. The driver neither collects nor clears kept objects. The embedding
ends the full synchronous sequence with Realm::clear_kept_objects, explicitly
collects, and schedules a later cleanup turn. Kept roots survive every callback
and reentry until that explicit checkpoint. One opt-in job work allowance spans
all callbacks and host reentry, with no per-callback reset. Realm execution
context is restored on normal and abrupt exit.

## ArrayBuffer

Ordinary ArrayBuffer objects retain private byte storage, byte length, and an
optional maximum byte length (25.1). Zero-filled pages are implicit; materialized
4 KiB pages and their sorted index use checked, fallible allocation. Logical
lengths cover the ToIndex domain without eagerly reserving or touching enormous
host allocations. Shrinking clears truncated bytes before subsequent growth can
expose them. Allocation and opted-in work failures leave existing storage intact.

Construction converts length and the optional maximum before consulting
newTarget.prototype. Resize, species-aware slice, transfer, and transferToFixedLength
follow the specification's conversion and detachment order. Slice retains the
original bounds, validates the species result, then rechecks the source and copies
only its remaining bytes. Transfer uses the intrinsic prototype, copies before
detaching, and preserves resizability only when requested. Detached buffers retain
their resizability slot while reporting zero byte and maximum lengths. Freezing
ordinary properties does not freeze these internal slots.

The constructor, species getter, isView, state getters, methods, and toStringTag
have their standard descriptors. isView recognizes DataView and TypedArray
ViewedArrayBuffer slots, including detached and out-of-bounds views, without
reading public properties. Shared buffers and embedding detach keys remain
separate implementation work.

## DataView

Ordinary DataView objects retain a traced buffer handle, a byte offset, and either
a fixed byte length or the specification's auto length (25.3). Omitted length
tracks a resizable buffer's remaining bytes; an explicit length remains fixed.
Shrinking can make either view out of bounds without changing its slots, so
regrowth restores valid views. The buffer getter still returns the same buffer
when it is detached or the view is out of bounds. The byteLength and byteOffset
getters throw TypeError for detached or out-of-bounds views.

Construction checks the buffer brand before offset conversion, then captures
the buffer length before converting an explicit view length (25.3.2.1). That
first range check uses the captured length. After newTarget.prototype lookup,
construction rechecks detachment and current bounds before installing the slots.
GetViewValue and SetViewValue validate the view brand, convert the offset and
setter value, and only then form a live buffer witness (25.3.1.2–6). Conversion
callbacks can resize, detach, or restore a view; the final witness determines
whether access throws TypeError or exceeds the valid view with RangeError.

The complete numeric family covers signed and unsigned 8/16/32-bit integers,
BigInt64/BigUint64, and Float16/Float32/Float64. Explicit byte order supports
unaligned accesses. Integer stores truncate and reduce modulo their width;
BigInt conversion never passes through Number or platform-sized integers.
Budgeted native-word helpers preserve all 64 bits, including signed minima.
Float16 stores reuse direct binary64 rounding to avoid binary32 double rounding.
Float encodings preserve signed zero and use consistent canonical NaN encodings.

Numeric reads use a fixed eight-byte result. Stores stage at most two sparse
pages, reserve the page index, and charge opted-in work before changing bytes.
Successful stores replace at most eight buffer bytes without cloning the whole
buffer; allocation or work failure preserves the existing bytes. Lazy page allocation
failure remains a host failure because SetValueInBuffer does not allocate a new
Data Block in the specification. The constructor, three getters, 22 numeric
methods, and toStringTag have their standard descriptors. No dependency or
default resource quota is added.

## TypedArray indexed properties

The internal TypedArray boundary implements all eight exotic methods in 10.4.5
for twelve element kinds, including Uint8Clamped and both BigInt kinds. Each
view retains a traced buffer, an aligned byte offset, an element kind, and either
a fixed element count or auto length. Live witnesses reject a whole fixed view
when it extends beyond the buffer; auto length floors remaining bytes to complete
elements. Original slots survive shrinking and detachment, allowing recovery
after regrowth when the buffer remains attached.

CanonicalNumericIndexString uses the existing ECMAScript Number parser and
formatter and preserves negative zero (7.1.21). Canonical invalid indices stop
Get and HasProperty from consulting prototypes; noncanonical strings and Symbols
keep ordinary property behavior. The maximum canonical numeric spelling has
25 code units, so longer keys can take the ordinary path without an extra copy.
This is a classification rule, not a property-key length limit.

Same-receiver Set converts its value before rechecking current index validity.
Distinct receivers follow the specification's separate validity and ordinary
receiver rules. DefineOwnProperty rejects invalid indices or incompatible
descriptors before conversion, then commits against current bounds. Element
descriptors report writable, enumerable, and configurable true, but valid elements
cannot be deleted. Own keys contain current indices followed by ordinary strings
and Symbols. PreventExtensions rejects views backed by resizable ordinary
buffers; Object and Reflect preserve that Boolean result and integrity failures.

Element access shares DataView's checked byte I/O and numeric codec. Uint8Clamped
rounds ties to even. BigInt reads retain opted-in magnitude and work limits;
HasProperty, deletion, and key enumeration do not decode element values. Numeric
writes retain the existing transactional sparse-page behavior and host failure
classification. ArrayIterator.next uses a TypedArray's current internal length,
throws for out-of-bounds views, and ignores public length properties (23.1.5.2.1).
No dependency or default resource quota is added.

## TypedArray construction and intrinsics

All twelve concrete constructors share the overload algorithms in 23.2.5.1.
Primitive inputs pass through ToIndex before prototype lookup; object inputs
resolve the destination prototype before accessing source hooks. Buffer inputs
check offset alignment before converting an explicit length, then inspect live
detachment and bounds. Omitted lengths over resizable buffers track complete
elements; fixed buffers require divisible byte lengths.

TypedArray sources use internal slots instead of public properties or iterators.
Same-kind copies clone the selected source bytes into a fresh fixed buffer at
offset zero, preserving NaN payloads and sparse zero regions. Other compatible
kinds convert internal numeric values; Number and BigInt content types cannot
mix, including empty sources. Iterable inputs finish iteration before allocation
or element conversion. Array-like inputs capture length once and read each index
in order.
Scratch element lists use checked allocation independently of call-argument
quotas. Byte lengths retain CreateByteDataBlock's maximum safe integer bound.

The shared from/of factories construct their receiver, require a live TypedArray
result of sufficient length, then use ordinary Set for each element (23.2.2).
The from mapper receives the value and index with the supplied thisArg. Its
iterable branch finishes collecting before construction or mapping; its
array-like branch captures length before construction and reads values afterward.
Later callbacks can resize or detach a custom result without an extra validation
step beyond the specified Set operations.

The abstract %TypedArray% constructor is available through the intrinsic graph
and has no global binding. Concrete constructors inherit its from/of and species
getter. State getters, the tag getter, and keys/values/entries use internal slots;
detached or out-of-bounds state getters return zero where required, while
iteration rejects those views. Symbol.iterator aliases values, and toString
aliases the initial Array.prototype.toString function. Intrinsics remain rooted
even after public properties change.

Deferred prototype algorithms and Uint8Array encoding methods have their
standard callable metadata and descriptors. Calling them reports a host
Unsupported result. Their presence does not count as algorithm support or turn
unsupported operations into passing negative conformance tests. Shared buffers
and agents remain separate implementation work.

## TypedArray traversal and species results

Seventeen prototype methods implement indexed access, search, callbacks,
reductions, string conversion, map, and filter (23.2.3). Entry validation captures
the internal element kind and length, ignoring public length properties. Later
indexed operations inspect live buffer state. A callback can detach or resize
the source without changing the captured iteration range or triggering an extra
validation step. Includes reads invalid indices as undefined; indexOf and
lastIndexOf use live presence checks and skip them.

Callback methods preserve their specified direction, arguments, thisArg, and
early returns. Find methods retain the value read before the predicate runs.
Reductions distinguish an omitted accumulator from explicit undefined. Join
converts the separator even for an empty view. The existing implementation
without ECMA-402 supplies comma-separated toLocaleString through live primitive
method lookup and zero-argument calls. String accumulation uses checked,
fallible UTF-16 storage and the embedding's optional quotas.

Map constructs its species result before callbacks; filter collects selected
values before species construction (23.2.4.3, delegating to 23.2.4.1). Both
validate the result's live TypedArray brand, minimum length, and Number/BigInt
content type. Custom species
may return an aliased view or another compatible element kind. Subsequent
writes retain ordinary Set conversion and live bounds behavior, including
reentrant detachment or resizing. Filter's scratch list is independent of
call-argument quotas. Sorting and Uint8Array encoding algorithms remain
separate work.

## TypedArray mutation

copyWithin, fill, reverse, and set implement 23.2.3.6/.9/.25/.26 using captured
internal lengths and live buffer witnesses. copyWithin converts all bounds
before a second witness, and skips that witness only when the original copy
count is zero. Resizing a still-valid auto view clips the original byte range.
Its raw sparse memmove preserves NaN payloads and overlap direction; all
fallible storage work finishes on a private clone before replacing the block.

fill converts its value once before either bound and always takes the second
witness, including for an empty range. reverse performs numeric Get/Get/Set/Set
for each pair, retaining the specified conversion and partial-effect behavior.
These two methods do not substitute raw byte operations for numeric semantics.

set checks the receiver brand before offset conversion, rejects negative
offsets before its target witness, and validates typed source state in order.
The range check precedes Number/BigInt content mismatch, even for empty sources.
Every same-buffer typed source uses a complete CloneArrayBuffer snapshot before
writing, including nonoverlapping and different-kind copies. Same-kind copies
retain raw bytes; other compatible kinds decode the chosen source buffer and
perform ordered numeric writes.

Array-like set captures source length after the target witness, then reads and
converts each element live. Getters and conversions can resize or detach the
target; conversion still precedes each current-bounds write check. Earlier
writes and user effects remain on later failure. Normative backing allocation
failures retain RangeError; internal staging failures remain host failures.
Native capacity and optional work checks stay separate from JavaScript errors;
no default limit, synthetic view, or extra intrinsic object is introduced.

## TypedArray copies and views

slice, subarray, toReversed, and with implement 23.2.3.27/.30/.32/.36. The shared
TypedArrayCreateFromConstructor helper accepts the exact argument list, checks
brand and live bounds after construction, and enforces minimum length only for
a single Number argument (23.2.4.1). Existing from/of and map/filter count-based
wrappers retain that check. Species results also require matching Number/BigInt
content type; subarray's two/three-argument form has no minimum-length check.

slice captures length, converts bounds, constructs species for the original
requested count, then revalidates source only if that request was positive.
A later shrink clips the copied prefix without reducing result length or
changing its existing tail. Same-kind copies retain raw bytes. An aliased
same-kind result requires live forward byte copying: a snapshot or memmove
would suppress the specified propagation of earlier writes. Different kinds
use increasing numeric Get/Set, preserving their own alias behavior.

subarray checks only the entry brand; detached or out-of-bounds source state
contributes length zero while retaining original buffer slots. An auto view
with missing/undefined end supplies exactly two constructor arguments; all
other forms supply the third count. No later source validation is added.
Custom species may return an unrelated shorter live view of the same content
type; the built-in constructor applies its own ordered buffer/range checks.

toReversed and with construct fresh results through the retained intrinsic
constructor for the source kind (23.2.4.2), without consulting source species or
replaced global bindings. Both copy numeric values instead of raw NaN payloads.
with converts the replacement before testing the normalized index against live
source bounds. Auto growth can validate an index beyond captured length while
the result still retains that captured length. Later reads after shrinkage keep
normal undefined-to-Number/BigInt behavior; the replaced index is not read.
Checked byte arithmetic, sparse storage and optional host limits are unchanged.

## TypedArray numeric sorting

sort and toSorted validate comparator callability before the receiver
(23.2.3.29/.33). Both capture the validated length and read every numeric element
before invoking comparisons. toSorted allocates through the retained intrinsic
constructor before those reads, without consulting public constructors or species.
The captured values use stable, iterative merging with fallible scratch allocation;
merging moves values between buffers and does not recurse on the native stack.

CompareTypedArrayElements (23.2.4.7) orders Numbers numerically, places NaN last,
and places negative zero before positive zero; BigInts use exact comparison.
A supplied comparator receives two values and undefined this, and its result
undergoes ToNumber. NaN and either zero compare equal. Callback and conversion
exceptions propagate immediately. Tests preserve stability without prescribing
the implementation-dependent sequence of comparator calls.

Only after sorting finishes are values written in increasing index order.
Comparator hooks may resize or detach the source: writes retain the captured
range and ordinary live indexed-set behavior, without an extra validation or
deletion phase. Copies use numeric values rather than promising raw NaN payload
preservation. Work, heap, and BigInt quotas remain caller-selected.

## Uint8Array hexadecimal conversion

fromHex, setFromHex, and toHex implement the hexadecimal operations in
23.3.1.2, 23.3.2.2/.4, and 23.3.3.8. Decoders require a primitive String
without coercion. Instance methods require the exact Uint8 element kind,
including subclasses, and use live internal bounds rather than public state.
The static decoder ignores its this value and uses the retained intrinsic
constructor after successful decoding.

Input parity is checked before destination capacity, even at capacity zero.
Only ASCII hexadecimal pairs are accepted. A setter decodes up to its current
capacity, writes complete valid pairs before an examined invalid pair, then
throws SyntaxError. An unexamined even-length suffix is ignored. Normal setter
results are ordinary objects with ordered, writable, enumerable, configurable
read and written properties. Options are unused.

toHex emits lowercase ASCII. Checked doubled length and any caller-selected
string limit are tested before scanning a large sparse view. Scratch storage
and byte I/O use checked sizes, fallible reservation, and bounded work.
Malformed input remains a JavaScript exception; optional host work, allocation,
and output limits retain their separate error channel and ordinary defaults.

## Uint8Array Base64 conversion

fromBase64, setFromBase64, and toBase64 implement the Base64 operations in
23.3. Decoders require primitive Strings. Instance methods check the exact
Uint8 brand, including subclasses, before reading options; the live buffer
witness follows option getters so their resize and detach effects are observed.
The static decoder ignores its this value and allocates through the retained
intrinsic only after decoding succeeds.

Options use GetOptionsObject, including its ordinary allocation for an omitted
argument. Decoding reads alphabet before lastChunkHandling; encoding reads
alphabet before applying ToBoolean to omitPadding. Standard and URL alphabets
are distinct, only the five ASCII whitespace characters are skipped, and
loose, strict, and stop-before-partial modes retain their separate tail rules.
Strict padded tails require zero unused bits.

Decoding respects destination capacity while scanning, with read positions in
UTF-16 code units. Syntax errors preserve writes from preceding complete
quanta; invalid padding or trailing junk does not write the current quantum.
Normal setters return ordinary objects with ordered writable, enumerable,
configurable read and written properties. Encoding uses checked output sizes,
fallible scratch storage, and bounded byte I/O. Host quotas remain opt-in and
use their separate error channel.

## Date time values

Shared Date arithmetic implements TimeClip, MakeTime and MakeDate (21.4.1).
TimeClip enforces the specification's inclusive ±8.64×10¹⁵-millisecond domain,
truncates finite fractions, and canonicalizes zero. MakeTime truncates each input
and preserves the specified floating-point operation order; MakeDate rejects
non-finite inputs/results and leaves clipping to its caller. These are Date's
numeric semantics, not embedding quotas.

UTC decomposition uses exact integer Euclidean division for pre-epoch days and
time within a day. YearFromTime uses binary search between January boundaries
containing both TimeClip endpoints; DayFromYear uses the specified Gregorian
leap-cycle formula with floor division for negative years. Month/day and weekday
fields follow the proleptic Gregorian calendar with 86,400 seconds per day and
no leap-second instants. The shared helper accepts clipped integral milliseconds;
Numeric MakeDay normalization is shared; local zones remain a separate step.

The separate from_epoch_milliseconds helper decomposes every native i64
millisecond value without TimeClip, including local calendar intermediates just
outside Date's range. It uses the usual narrow year bracket for clipped inputs
and a wider bracket containing both native integer endpoints otherwise. All
Gregorian day arithmetic remains exact in i64 and all resulting years fit i32.
This integer helper does not substitute for Number division or rounding in wide
MakeDay witness checks. Public Date formatters and from_time_value still enforce
the clipped domain; zone conversion must precede final clipping.

Shared TransitionRule arithmetic computes exact epoch milliseconds for recurring
political time-zone transition dates. TZif's POSIX date forms distinguish Julian
days with or without February 29 and numbered weekdays within a month; the fifth
week means the last occurrence. Signed transition seconds can carry into another
day or year. Wall clocks subtract the offset before the transition, standard
clocks subtract the standard offset, and UTC clocks need no adjustment. Native
i32 years and offsets fit the i128 result without floating-point rounding,
TimeClip or a narrower backend calendar range. Malformed date/time rule fields
are rejected before indexing. Loading zone data and resolving complete offset
histories remain separate implementation steps.

The borrowing POSIX time-zone parser accepts complete TZif version 3/4 rule
strings with ASCII designations, quoted numeric/sign designations, explicit
standard offsets and optional daylight offsets. Written POSIX offsets have the
opposite sign from seconds east of UTC; absent daylight offsets add one hour to
standard time. Daylight transition dates must be explicit and omitted transition
times default to wall 02:00. Extended signed times span −167:59:59 through
+167:59:59. Field widths/ranges, punctuation and complete consumption are checked
without allocating names or applying a name-length quota. Missing rules never
acquire platform defaults. Loading TZif histories remains pending.

RecurringTimeZone compiles recurring offsets into the exact 400-year Gregorian
cycle of 146,097 days. Two annual rules contribute at most 800 transitions;
this is a consequence of the calendar, not a resource quota. Ordered daylight
intervals spanning a whole year omit standard-time transitions, implementing
TZif's year-round daylight extension. Years with no transitions preserve the
offset from neighboring years; southern seasons and negative daylight offsets
retain their ordered UTC changes. Transition times normalize into the cycle,
sort by exact epoch milliseconds, and reject contradictory simultaneous changes.
Lookup uses integer Euclidean remainder and binary search without allocating or
clipping, including every native i128 time. Invalid native records, conflicting
rules and failed native allocation have distinct errors. This calendar
periodicity applies only to recurring rules; explicit historical TZif changes
must still be preserved, rather than mapped into another year.

TzifTimeZone loads UTC offset histories from TZif versions 1–4. Header counts,
checked block lengths, type indices, strict transition ordering, designation
termination and Boolean indicators are validated before storing a history.
Modern files skip the legacy 32-bit body and use the authoritative 64-bit block.
Offsets and transition seconds remain exact native integers; queries use
Euclidean millisecond-to-second division, including negative subsecond times.
Type zero applies before the first transition, and without recurring rules the
last historical offset persists. A footer applies after the last transition or
to all times in a history without transitions; its offset must agree with the
final historical type. Version 2 transition syntax excludes signed/extended
times. Trailing extension data is ignored. Candidate offsets include historical
and recurring types without assuming every type occurs at the query time.
Lookup makes no allocations and applies no Date/backend calendar bound. Unknown
versions, leap-second time scales, malformed data, contradictory recurring rules
and native allocation failures remain distinct. Display names and local gap/fold
resolution are separate implementation steps.

Recurring and TZif histories now resolve nominal local milliseconds with UTC's
gap/fold policy (21.4.1.26). Each candidate subtracts a possible offset and is
checked against the actual UTC offset at that epoch. Repeated times choose the
earliest valid epoch. For a skipped time, the resolver finds the greatest valid
local millisecond before the requested value, selects that local value's latest
possible epoch and uses its offset. This remains correct when closely spaced
changes produce overlapping local images; choosing the nearest UTC transition
alone would lose the required preceding local value. TZif loading indexes sorted
local interval ends once, while recurring endpoints use exact calendar-cycle
arithmetic and respect the historical cutoff. Queries allocate nothing. Native
i128 overflow and inconsistent data remain separate errors, and TimeClip stays
with the Date caller. Display names remain a separate step.

The immutable public TimeZone host object owns fixed, recurring or checked TZif
histories behind a shared Arc. Named loading canonicalizes database spelling,
searches TZDIR and conventional Unix roots in order, and uses the explicitly
pinned IANA bundle only when files are absent. Existing malformed or unreadable
files return typed native failures. Named identifiers exclude traversal and
absolute paths; explicit host TZ settings may separately select a TZif file.
System loading honors empty, named, POSIX and file TZ settings. Unix localtime
files retain exact raw histories even without a name; Jiff performs platform
discovery elsewhere. A realm loads its system zone lazily on first request and
keeps that immutable history until the host selects an override or resets it.
Queries allocate nothing and impose no year, work or heap quota. Local Date
getters use these histories; construction and mutation follow in separate steps.

Local calendar getters (21.4.4.2–11) check the Date slot and return NaN before
loading a zone for an invalid time. Finite reads add the exact whole-second
offset to the clipped millisecond value, then decompose that unclipped local
intermediate. Every native i32 offset preserves i64 capacity and exact Number
integers below 2^53 throughout Date's domain. getTimezoneOffset retains the
specified subtraction/division, so a zero offset produces positive zero. Host
configuration/data failures become Error::Host and bypass JavaScript catch and
finally. Unsupported TZif time scales/versions and allocation failures remain
Unsupported and Limit respectively. The conformance runner classifies host
configuration failure as setup failure, never a successful negative test.

Finite numeric Date construction and local interchange parsing share a fused
TimeClip(UTC(t)) helper (21.4.1.26/30). Non-finite Number inputs return NaN before
loading host configuration. Finite values retain their original nominal local
milliseconds until gap/fold resolution, including values outside Date's domain.
Every supported native offset fits i32 whole seconds, so an input outside
MAX_TIME_VALUE + 2^31 * 1000 cannot produce an in-range final epoch for any zone.
That is an output-range proof, not an input quota. Remaining integral inputs and
resolved epochs stay below 2^53, preserving exact Number milliseconds. Historical
and recurring resolution precede final clipping. Interchange strings retain
literal years and their UTC/date-only/local distinction; numeric construction
retains MakeFullYear before month normalization. All seven present numeric
components convert before zone resolution and new-target prototype lookup.

MakeDay (21.4.1.28) truncates components, divides the mathematical integer month
by 12 with floor, converts that quotient to Number, adds the year using Number
arithmetic, and retains the exact Euclidean month remainder. Native integers
handle common inputs; the existing workspace BigInt implementation handles
larger finite Numbers. DayFromYear's Gregorian expression also uses mathematical
integer arithmetic and rounds once. No year or month input cap is imposed.

For an i32 normalized year, a finite first-day time exists: neighboring time
values are less than one UTC day apart and the calendar day number is exact.
Larger years explicitly check the rounded month boundary and its neighboring
time values against the requested day and adjacent year boundaries. These checks
handle skipped day numbers and rounded year-boundary plateaus using Day's Number
division and YearFromTime's largest-integral-Number rule. A valid month boundary
can exceed TimeClip, and a huge day component can cancel it before final clipping.
MakeDay retains ordered Number addition/subtraction and rejects non-finite input
or an impossible calendar witness with NaN. Its native integer work budget is
optional; runtime callers must leave intermediate magnitude unlimited rather
than apply a JavaScript BigInt value quota to Number calendar arithmetic.

Date Time String Format parsing borrows UTF-16 code units and accepts exactly the
edition-17 date-only/time forms, four-digit or signed six-digit years, and numeric
offsets (21.4.1, 21.4.3.2). It rejects negative expanded zero, non-ASCII digits,
nonconforming separators/widths, leap seconds and out-of-bounds elements. Hour 24
requires zero minutes, seconds and milliseconds. Parsed fields retain defaults
and the UTC/local distinction: absent zones mean UTC for date-only forms and
unresolved local time for date-time forms. Expanded years can exceed TimeClip's
domain at this syntax stage; calendar/zone conversion and time-value clipping
remain separate. Only standard interchange and this implementation's required
own-output formats are recognized; no heuristic legacy parser is added.

UTC and explicit-offset interchange fields convert with exact widened integer
arithmetic through calendar rollover and zone adjustment. Only then enforce
TimeClip's range and convert the accepted integral milliseconds to Number;
nominal fields outside the range can still describe an in-range UTC instant after
their offset. Invalid native field records and out-of-range instants yield NaN,
while valid local forms report that zone resolution is required. MakeFullYear
separately truncates numeric input and maps years 0–99 to 1900–1999; interchange
years preserve their literal year, including zero and 1–99.

DateTimeString exposes nominal_epoch_milliseconds to validate and normalize its
calendar fields before interpreting the zone. It returns exact i128 milliseconds
for every native i32 year and valid day/time record, with day and end-of-day
rollover and no TimeClip or floating rounding. UTC/offset conversion uses that
same result, then applies the offset and range check. Local parsing can therefore
share the identical calendar normalization before host zone resolution.
Invalid zone offsets remain separate from calendar validation.

Canonical ISO formatting decomposes a clipped integral time value and emits the
finite toISOString form (21.4.4.36): four year digits for 0–9999, otherwise a sign
and six digits, followed by full UTC date/time fields, three millisecond digits,
and Z. Output has 24 or 27 UTF-16 code units and preserves every supported
millisecond. Invalid native time values are rejected before formatting.

Standard UTC formatting follows toUTCString (21.4.4.42), with English weekday/
month names, seconds, GMT, and a minimum four-digit year prefixed only when
negative. It omits milliseconds. Date.parse also accepts that canonical output,
validating the calendar and weekday directly from UTF-16 without allocation.
Years 0–99 remain literal, and negative instants round-trip to the containing
second using floor division. This satisfies the standard UTC-output invariant
in 21.4.3.2; other legacy formats remain outside the parser.

Local strings use DateString, TimeString and TimeZoneString (21.4.4.41), with
unclipped calendar fields, English weekday/month names and minimum four-digit
literal years. The GMT hours/minutes follow the required 24-hour wrap and omit
offset seconds. Historical seconds and offsets of a day or more are preserved
in the permitted optional timezone name `(UTC+HH:MM:SS)` or its negative form;
other offsets use an empty name. The canonical parser borrows UTF-16, validates
calendar/weekday and consistent offset fields, subtracts the exact full offset
and checks the final UTC range. Its own whole-second output round-trips even if
the realm's zone changes or host loading fails. Date() formats the current time
without coercing supplied values; argument expressions still evaluate normally.

The runtime materializes the complete edition-17 Date constructor/prototype
property graph. Instances have a distinct [[DateValue]] slot containing a clipped
Number or NaN; Date.prototype has no slot. Timestamp/copy/calendar construction, Date.now, Date.UTC,
UTC/offset/local Date.parse, UTC/local getters and setters, getTime/valueOf, setTime,
toISOString, local strings, toUTCString, toJSON and Date() output
and @@toPrimitive are implemented. Constructor input conversion precedes
new-target prototype lookup; copying another Date bypasses its conversion hooks.
setTime checks the receiver before converting input and stores only after that
conversion succeeds. UTC getters and ISO formatting use the shared exact helpers.
ISO, local and standard UTC output check exact optional string/work quotas before allocation;
an invalid Date throws RangeError for ISO and yields "Invalid Date" for UTC.
Object.prototype.toString recognizes the slot independently
of prototype identity and still observes @@toStringTag.

Date's generic @@toPrimitive treats default as string and enters
OrdinaryToPrimitive directly, avoiding redispatch through its own hook. Generic
toJSON boxes its receiver, requests a numeric primitive and returns null for
non-finite Numbers before looking up toISOString; other primitive results invoke
that method on the boxed original object with no arguments. Invalid Date string
methods return "Invalid Date". The three locale string methods use the edition-17
non-Intl fallback (21.4.4.38–40). This host's fixed English conventions use the
corresponding local date/time formatters directly, without invoking overridable
receiver methods. Both reserved parameter positions are ignored without reading
or coercing their values; ECMA-402 remains outside the target. Branding, invalid
values, exact offsets, host errors and output quotas follow the shared formatter.
UTC hour/minute/second/millisecond setters capture the time value
before ordered argument conversion and retain omitted fields from that captured
instant. They normalize rollover with MakeTime/MakeDate and then apply TimeClip.
For a captured NaN time, all present arguments still convert before returning
NaN without writing the slot; conversion hooks may have installed a valid time.
Numeric constructors share Date.UTC's ordered conversion and calendar/time
arithmetic, including later abrupt completions after earlier NaN results. UTC
returns NaN for a non-finite intermediate before querying a time zone, so these
inputs create a branded invalid Date with ordinary new-target prototype lookup.
Finite local intermediates resolve before TimeClip because an offset can bring a
boundary instant back into range. Nine local getters return NaN for an invalid time.
setDate, setMonth, setHours, setMinutes, setSeconds and setMilliseconds convert
all present arguments before returning NaN for a captured invalid time without
writing the slot. Hooks can revive that Date. All finite local setters derive
omitted fields from the captured local calendar and resolve folds/gaps before
final clipping. Local setFullYear converts the year, uses +0 as the local calendar
for a captured invalid time or obtains LocalTime for a valid time, then converts
optional month/date fields. Its invalid fallback revives from January 1 at local
midnight, preserving literal short years. Slots are written only after successful
conversion, calendar arithmetic and zone resolution.

setUTCDate (21.4.4.27) likewise captures the original timestamp before ToNumber
and returns NaN without writing when that captured value was invalid. Its year
and month are already normalized, so the first day of the month follows exactly
from the captured UTC day number and day of month. Add the truncated date and
subtract one using ordered Number arithmetic, retain the captured time of day,
and clip only the final timestamp. The intermediate month boundary can lie
outside TimeClip, including April 1 at the minimum supported timestamp; it is
still finite. This implements setUTCDate completely without imposing extra
input bounds or substituting for general numeric MakeDay.

Date.UTC (21.4.3.4) converts the mandatory year and every present optional
component in order, including conversions after an earlier NaN. Absent month/
time fields default to zero and absent date defaults to one; explicit undefined
still converts to NaN. MakeFullYear adjusts the input year before month carry,
then shared MakeDay/MakeTime/MakeDate preserve the specified arithmetic order
and TimeClip applies only to the final combined timestamp. Excess arguments
are evaluated by the caller and ignored by the intrinsic. Native calendar
integer work shares the optional evaluation budget, with no BigInt magnitude
quota on Number intermediates and no default resource quotas.

setUTCMonth and setUTCFullYear (21.4.4.32 and 28) capture the stored time before
ordered numeric conversion and retain its omitted calendar fields and time of
day. setUTCMonth converts all present fields before returning NaN for a captured
invalid time, preserving any value installed by a conversion hook. setUTCFullYear
instead substitutes the epoch for a captured invalid time,
so successful conversion revives the object using January 1 and midnight as
defaults. Year setters preserve literal years 0–99 without MakeFullYear adjustment.
Both use shared MakeDay/MakeDate and clip only the final timestamp. The stored
slot changes only after all conversions and optional native integer work succeed;
an abrupt conversion or work failure preserves mutations made by hooks.

## Character-only UnicodeSets operations

The checked v-mode class compiler recognizes complete character-only classes
with arbitrarily nested unions, intersections, left-subtractions and complements
(22.2.1, 22.2.2.9). Each bracket level uses a union or one homogeneous operator
chain; nesting retains the chosen structure. Supported operands are characters,
class escapes, character-property escapes and nested classes. Raw character
endpoints form ranges before folding. Optional line assertions, single-class
quantifiers and existing whole-body capturing/noncapturing wrappers reuse the
Unicode matching plans and original UTF-16 boundaries. Runtime validates the
complete Pattern before selecting this local compiler.

Case-sensitive v uses raw code-point intervals including lone surrogates. In vi,
raw leaves are closed under the pinned full simple-fold preimages before Boolean
operations or complement. Completed children are already closed; a child's
complement is applied before its parent operation. For example,
`[A--[a--A]]/vi` matches A and a, while `[A--a--A]/vi` is empty.
Complements use the complete code-point universe. Unions close their raw leaves
once and normalize the combined child intervals once; a sole completed child
moves through enclosing classes without copying or repeating the fold-table scan.

One source cursor and fallibly reserved flat frames evaluate nesting without
native recursion.
Source, frame and interval work is charged before it is performed. Checked
reservations report new frame, interval and scratch capacity failures as host
resource limits, distinct from JavaScript exceptions and unsupported syntax.
Work-callback errors pass through unchanged. Final Vec-to-Arc publication retains
the existing infallible allocation boundary of shared matcher storage; these
constructors do not promise process-wide recovery from every allocation failure.
No default depth, work or heap limit is introduced.

String operands, every `\q` operand and binary properties of strings remain
Unsupported, including when surrounding set operations would produce an empty
set or an all-character set. Additional consuming concatenation, inner quantified
captures and assertions inside repeated classes remain separate proofs. Standalone
flat classes still use the legacy constructors; their checked entry point returns
None unless a structural nested class or set operator occurs. Existing u-mode
semantics, narrow legacy Core compiler domains and their work callbacks remain
unchanged.

## Word assertions around checked UnicodeSets classes

A separate checked v/vi constructor admits word assertions around one complete,
unquantified nested/operator class (22.2.2.4, 22.2.2.9). Its exact order is an
optional ^, at most one leading \b or \B, the class, at most one trailing \b or
\B, and an optional $. At least one word assertion is required. Runtime validates
the complete Pattern and tries this adapter after the legacy class-word path,
only in v mode. Bare flat classes and all earlier Core constructor domains and
work callbacks retain their existing behavior.

The adapter borrows the complete class units and records the outside assertions
without allocating a stripped source buffer. It requires the retained body to
start with [ and end with ], then delegates to the checked class compiler and
attaches assertions only after successful complete compilation. Other valid
orders such as \b^CLASS and CLASS$\b remain outside this new entry point, as do
duplicate or contradictory assertion sequences. Existing whole-body wrapper
proofs retain complete-character captures and indices; partial captures, word
assertions inside repeated atoms, inner quantified captures, consuming continuations and
string operands remain separate implementation work.

Word assertions inspect the original input neighbors at the consumed character's
start and end. Word characters are ASCII letters, digits and underscore, plus
long-s and Kelvin under i; other Unicode letters and surrogate units are
non-word. The existing Unicode start normalization runs before assertion checks,
including the approved leading-boundary behavior for an offset inside a surrogate
pair. Line assertions retain their original endpoint and multiline context;
matching never substitutes synthetic folded input. Class algebra and folding
continue through the existing checked compiler.

The existing search-work bound includes each attached word predicate. Work(E)
retains the first callback error and Capacity remains a host resource failure,
distinct from Unsupported and JavaScript exceptions. Any new construction buffer
requires checked, fallible reservation; the existing final matcher Arc and Runtime
normalization allocations retain their documented boundaries. No default work,
heap or depth limit, dependency or matching kernel is added.

## Word assertions around checked UnicodeSets class repetitions

A separate typed v/vi constructor admits optional outside word assertions around
one complete checked nested/operator class with one greedy or lazy quantifier
(22.2.2.3, 22.2.2.4, 22.2.2.9). The exact order is optional ^, at most one leading
\b or \B, the class and its quantifier, at most one trailing \b or \B, optional $.
At least one word assertion is required. Existing decimal bounds retain their
zero, finite, open and native-overflow semantics without expanding counts into
storage. Complete class compilation remains mandatory even for {0} or an
impossible minimum; string leaves are never admitted through an empty result.

The wrapper borrows the retained units and delegates to the existing checked
class/quantifier proof. Retained inner line gates are rejected before outside
flags are attached, so \bCLASS+$\b cannot lose its dollar assertion. Word gates
apply to the complete repetition, never to each class iteration. Other assertion
orders, duplicate gates, inner groups/captures, per-iteration word assertions,
string operands and consuming continuations retain their separate proof domains.
Existing complete enclosing wrappers may retain whole-repetition captures.

The existing monotone boundary-search kernel selects the earliest start and its
longest or shortest eligible endpoint. A lazy count can grow until its trailing
word gate succeeds; a greedy count can retain an earlier eligible end. Empty
results from zero minimum still check both gates at the same original boundary.
WordCharacters uses the original input and the actual i flag; class folding does
not replace it. Unicode normalization precedes boundary checks, retaining the
approved leading-boundary behavior for starts inside a surrogate pair.

The borrowed typed helper preserves the old line-only constructor's complete
callback sequence and narrow domain. New construction Work(E) and Capacity keep
the existing host-error mapping; search and all legacy callbacks are unchanged.
No new source-copy buffer, recursion, dependency or default quota is introduced.
Existing Runtime preprocessing/legacy source copies and final matcher Arc
allocation retain their documented allocation boundaries.

## Checked UnicodeSets class with one literal continuation

A separate `RegExpUnicodeClassLiteralMatcher::compile_v_set_with_work` entry
point admits one complete character-only nested/operator v/vi class
followed by exactly one literal Unicode atom (22.2.2.3, 22.2.2.9). Optional ^/$
assertions constrain the whole sequence. Runtime uses it only after complete
Pattern validation, with existing complete enclosing wrappers retaining captures
of the whole sequence. Legacy Core constructors keep their narrower domains and
callback contracts.

The class must pass the existing checked set compiler in full, and the literal
continuation must pass its complete Unicode-atom proof. Every string operand,
including every \q leaf and property of strings, remains Unsupported even when
set algebra would eliminate it. Quantified components, multiple continuation
atoms, a literal prefix, a second class, internal assertions and captures of only
part of the sequence remain outside this entry point.

Matching consumes one class character and one literal character in source
order. Class algebra and literal comparison use the actual i flag and pinned
common/simple folding. Candidate starts and both consumed characters retain the
existing Unicode decoding and start normalization; match/capture ranges and
lastIndex remain UTF-16 code-unit offsets. A class match with a failing literal
continuation does not end the search early. Optional line assertions use the
original input and m flag at the whole sequence's endpoints.

Construction work and checked capacity failures retain distinct host-error
handling. Search accounts for class checks, literal checks and attached line
assertions before publishing success or lastIndex. Existing Runtime coercion,
consumer and host-error boundaries remain in force. This extension adds
no default work, heap or depth limit, dependency, profile or fixture change.

## One literal prefix before a checked UnicodeSets class

A separate typed `compile_literal_then_v_set_with_work` constructor admits one
literal Unicode atom followed by one unquantified checked character-only v/vi
class (22.2.2.3, 22.2.2.9). Optional outside ^/$ constrain the complete pair.
Runtime tries it only after the existing class-then-literal constructor and all
earlier dispatch paths. Both constructors use one private source-order enum in
the same zero-inner-capture plan. Existing complete whole-body wrappers retain
their separate capture proof; partial captures are excluded.

The prefix boundary comes from the existing Unicode literal-atom decoder, after
source-sized work is charged. It is never found by scanning for an unescaped
bracket. Borrowed prefix units must compile as exactly one literal, with only an
optional leading ^. The entire remaining suffix must compile as a checked class,
with only an optional trailing $. A nested class or set operator is required.
Every string leaf remains unsupported, including leaves behind empty algebra.
Internal assertions, word gates, quantifiers and additional atoms or classes
remain outside this proof.

Matching checks the literal first and the class sticky at its end. A failed
class resumes search at the literal's complete-character end. Folding, line
context and UTF-16 ranges use the original input; the existing approved Unicode
start normalization is unchanged. The existing two-character search bound and
four-unit sticky bound also apply to this order. The class-first constructor's
domain and work-callback sequence remain unchanged.

The adapter adds no source buffer, input-sized state, native recursion, default
quota, dependency or profile change. Existing checked class reservations and
Work(E)/Capacity distinctions remain in force; no existing final shared-storage
allocation boundary changes.

## Outside word assertions around checked class/literal pairs

A separate typed v/vi constructor admits canonical outside word assertions
around either existing two-atom class/literal order (22.2.2.3, 22.2.2.4). Its
grammar is optional ^, at most one leading \b/\B, one complete bare pair, at most
one trailing \b/\B, optional $. At least one word gate is required. The class
must contain a nested class or set operator and remain character-only. Existing
complete whole-body wrappers retain their capture proof; component captures,
quantifiers, additional atoms and other assertion orders remain unimplemented.

Framing and both component proofs borrow the original units. Escape parity
distinguishes an outside $ or word gate from literal escapes. The complete bare
pair is proved before outside flags are attached, so retained internal ^/$ or
duplicate/reordered word gates cannot lose meaning. Every checked class compiles
fully even when empty; hidden string leaves never become admitted by an
impossible match or endpoint condition. The two old public constructors retain
their exact recognition and work-callback bodies, with absent outside predicates.

The existing plan checks outside line and word predicates only at the complete
pair's original endpoints. WordCharacters uses actual i, including Kelvin and
long-s, independently of whether a consuming class is fold-invariant. If either
word gate fails after both characters match, search resumes at the first
character's end, preserving overlapping candidates. Original line context,
UTF-16 ranges and approved Unicode start normalization remain unchanged.

Only new plans add word/line predicate allowances to the existing precharged
search passes; the four-unit sticky consumption bound and entry callback remain.
There is no new source buffer, input-sized state, recursion, allocation boundary,
dependency, default quota or stack setting. Existing checked Work(E)/Capacity
failures keep their host-error distinction.

Two unquantified checked character-only UnicodeSets classes can form one typed
v/vi sequence (22.2.2.3, 22.2.2.7.3). Each component independently satisfies the
existing nested-class/set-operator compiler gate. Exact borrowed class boundaries
prove there are two complete classes and optional outside ^/$, with no remaining
atom, repetition, inner assertion or capture. Flat components remain outside
this constructor's domain. Full checked compilation of both components rejects
string leaves even when the first class is empty.

The immutable plan contains two existing Unicode class matchers. The first
searches normally; the second is sticky at the first character's end. A failed
second match retries at the first end, preserving overlapping candidates. Both
consume original complete input characters and retain original line context,
pinned i folding, lone surrogates and the documented leading-boundary behavior
for initial offsets inside a pair. Work bounds sum both existing predicates plus
pair traversal; one fallible entry charge and the four-unit sticky allowance are
retained. No source/input buffer, dependency, quota, profile or stack change is
introduced. Whole enclosing captures are proved separately by Runtime.

A separate typed constructor admits canonical outside \b/\B around the two
checked character-only v/vi classes (22.2.2.3,22.2.2.4). It requires at least one
word gate, independently retains each existing checked nested/operator gate,
and proves the exact borrowed order ^? word? CLASS CLASS word? $?. Full component
compilation rejects string leaves even behind empty sets. Inner/reordered gates,
flat components, repetitions, extra atoms and partial captures remain separate
proofs. The previous bare-pair constructor/domain/callbacks are unchanged.

Optional local endpoint predicates test original whole-pair ranges using actual
i WordCharacters and original m line context. Failure after two consumed
characters retries the first end, preserving the overlapping match1..3 of
\B[[a]][[a]]\b on aaa!. Whole enclosing captures, the existing Runtime variant,
one entry charge and four-unit sticky bound are reused. Only new-plan predicate
allowances are added; no source/input buffer, dependency, default quota, profile
or stack change is introduced.

A separate constructor matches a checked character-only v/vi class followed by
one flat class, with optional outside ^/$ (22.2.2.3, 22.2.2.7.3). The prefix keeps
the original nested-class/set-operator gate. A crate-private borrowed flat adapter
preserves the public flat compiler's framing and initial source charge with
UnicodeSets enabled, then uses the unchanged flat parser. Complete class
boundaries prove there are exactly two atoms. The suffix rejects nested classes,
operators and string leaves, including behind an empty checked prefix. Outside
word assertions, extra atoms, repetitions and partial captures remain separate
work. Previous constructors retain their domains and callbacks.

The existing two-matcher plan and zero-inner-capture Runtime variant are reused
with absent outside-word metadata. The second match remains sticky at the first
end; search retries at that first end after failure. Original character and line
boundaries, pinned case folding, opted-in work charges and the approved
initial-low-pair behavior remain in use. No source/input buffer, dependency,
default quota, profile or stack change is introduced.

A separate constructor admits the reverse order: one flat UnicodeSets class
followed by one checked character-only v/vi class (22.2.2.3, 22.2.2.7.3). Exact
borrowed boundaries prove two unquantified atoms with optional outside ^/$.
The existing flat adapter compiles the prefix before the checked compiler
validates the suffix, retaining source-order work callbacks. An empty flat
prefix still requires complete suffix validation, including rejection of
string leaves. Earlier constructors retain their domains and callbacks.

This order reuses the same pair plan with absent outside-word predicates,
sticky second match and retry at the first end. Complete original characters,
line context, folding, whole enclosing captures, Runtime representation and
work bounds retain their existing behavior. Outside word assertions, extra
atoms, repetition and partial captures remain separate work.

A separate typed constructor admits canonical outside word assertions around a
checked-then-flat v/vi pair (22.2.2.3, 22.2.2.4). The exact borrowed order is
^? word? CHECKED FLAT word? $?, with at least one \b/\B. The prefix retains the
nested-class/set-operator gate; the suffix retains the existing flat proof.
Both compile fully before predicates are attached, including behind an empty
prefix. String leaves, reverse order, flat-flat, inner/reordered assertions,
extra atoms, repetition and partial captures remain outside this constructor.
Previous constructors retain their domains and exact callback bodies.

The existing endpoint record checks actual i WordCharacters, including Kelvin
and long-s, and original m line context. Fold-invariant class membership does
not remove the i word-fold requirement. A failed outside gate after two matched
characters retries at the first end, preserving the 1..3 overlap of
\B[[a]][a]\b on aaa!. Complete original UTF-16 characters and the documented
initial-low-pair boundary remain unchanged. Borrowed framing precharges 3N;
component callbacks retain checked-then-flat source order. Existing predicate
allowances, one entry charge, four-unit sticky bound, Runtime representation and
whole-body capture proof are reused. No source/input buffer, parser refactor,
dependency, default quota, profile or native-stack change is introduced.

A separate typed constructor admits canonical outside word assertions around a
flat-then-checked v/vi pair (22.2.2.3, 22.2.2.4). Its exact borrowed order is
^? word? FLAT CHECKED word? $?, with at least one \b/\B. The prefix retains the
existing flat proof; the suffix retains the nested-class/set-operator gate.
Both components compile fully in source order, even behind an empty flat set.
String leaves, flat-flat, inner/reordered assertions, extra atoms, repetition
and partial captures remain outside this constructor. Earlier pair constructors
retain their separate domains and exact callback bodies.

WordCharacters uses actual i, including Kelvin and long-s, independently of
fold-invariant Any membership. Line predicates use original m context. The
existing endpoint record and search kernel retry at the first character's end
after a failed outside gate, preserving overlaps and original UTF-16 ranges.
The documented leading-boundary behavior for an initial low surrogate remains
unchanged. Construction charges 3N before borrowed framing, then flat and checked
callbacks in source order; Work and checked Capacity failures retain their host
error distinction. The same Runtime representation, complete enclosing capture
proof, endpoint allowances, one entry charge and four-unit sticky bound are
reused. No source/input buffer, parser refactor, dependency, default quota,
profile, native-stack or worker-policy change is introduced.

A separate typed constructor admits exactly two flat character-only v/vi classes
in sequence (22.2.2.3, 22.2.2.7.3). Borrowed boundaries prove the complete order
^? FLAT FLAT $?, with no remaining atom. Construction charges 2N before the two
class-end walks, then invokes the unchanged flat compiler first and second in
source order. Both components compile fully even when the first set is empty;
nested classes, set operators and every string leaf fail the flat proof.
Earlier checked and mixed constructors retain their domains and exact callback
bodies. Outside word assertions around flat-flat pairs, inner assertions or
captures, extra atoms and repetitions remain separate work.

The optional leading ^ belongs to the first component and trailing $ to the
second; both inspect the original input and m line context. The existing pair
plan matches the second character sticky at the first end and retries at that
end after failure, preserving overlaps and complete original UTF-16 characters.
Pinned folding, lone surrogates and the documented initial-low-pair boundary
remain unchanged. Runtime reuses the same representation and separately proved
complete whole-body captures. The existing entry charge, work bounds and
four-unit sticky allowance remain in use. No source/input buffer, parser
refactor, dependency, default quota, profile, stack or worker change is added.

A separate typed constructor admits canonical outside word assertions around
exactly two flat character-only v/vi classes (22.2.2.3, 22.2.2.4). The borrowed
order is ^? word? FLAT FLAT word? $?, with at least one \b/\B. Both existing
flat proofs run fully in source order, including validation of the second behind
an empty first set. Nested/operator classes, string leaves, inner/reordered
assertions, extra atoms, repetitions and partial captures remain outside this
constructor. Earlier constructors retain their domains and callback bodies.

Construction charges 3N before borrowed framing and both class-end walks. The
existing endpoint record uses actual i WordCharacters even when a flat Any
predicate is fold invariant, and original m line context. Failure after both
characters match retries at the first end, preserving the 1..3 overlap of
\B[a][a]\b on aaa!. Complete original UTF-16 characters, lone surrogates and
the documented initial-low-pair boundary retain their existing behavior.
Runtime reuses the same representation and complete whole-body capture proof.
Existing endpoint allowances, one entry charge and four-unit sticky bound remain
in use. No source/input buffer, parser refactor, dependency, default quota,
profile, stack or worker-policy change is introduced.

A separate typed plan matches one decoded Unicode literal followed by exactly
two checked character-only v/vi classes (22.2.2.3, 22.2.2.7.3). An optional ^
belongs to the literal prefix and optional $ to the second class; line assertions
inspect the original input and m context. The existing literal decoder proves
one complete atom, and each class independently retains the nested-class/set-
operator gate. Borrowed framing charges 3N before literal decoding and the two
class-end walks. Complete literal, first-class and second-class proofs then run
in source order, including rejection of hidden string leaves behind an empty
first class. Flat components, word gates, quantifiers, additional atoms, inner
assertions and partial captures remain separate proofs. Earlier APIs are unchanged.

Matching checks both classes sticky after the literal. If either fails, search
retries at the literal's complete-character end, preserving the 1..4 overlap of
a[[a]][[b]] on aaab. All ranges use original UTF-16 characters, folding and lone-
surrogate semantics. The documented leading-boundary behavior for an initial
low surrogate remains in use. Runtime adds a dedicated zero-inner-capture variant
and retains the existing proof of complete whole-body captures.

Three complete characters consume at most six UTF-16 units. The new Runtime
work path adds the possible one-unit initial-pair rewind to the remaining suffix
before applying the six-unit sticky cap; nonsticky work retains that rewind
allowance. Entry charging and summed predicate/traversal bounds remain fallible.
The preceding four-unit paths and their callbacks are unchanged. No source/input
buffer, parser refactor, dependency, default quota, profile, stack or worker-policy
change is introduced.

A separate typed plan matches exactly two checked character-only v/vi classes
followed by one decoded Unicode literal (22.2.2.3, 22.2.2.7.3). An optional ^
belongs to the first class and optional $ to the literal suffix; line assertions
inspect the original input and m context. Each class independently retains the
nested-class/set-operator gate, and the existing literal decoder proves one
complete suffix atom. Borrowed framing charges 3N before the two class-end walks
and literal decoding. Complete first-class, second-class and literal proofs then
run in source order, including rejection of hidden string leaves in the second
class behind an empty first class. Flat components, word gates, quantifiers,
extra atoms, inner assertions and partial captures remain separate proofs.

Matching checks the second class and then the literal sticky after the first
class. If either fails, search retries at the first class's complete-character
end, preserving the 1..4 overlap of [[a]][[a]]c on aaac. All ranges use original
UTF-16 characters, folding and lone-surrogate semantics, including the documented
leading-boundary behavior for an initial low surrogate. Runtime uses a separate
zero-inner-capture variant and the existing complete whole-body capture proof.

The suffix variant shares the literal-prefix variant's six-unit work path. The
possible one-unit initial-pair rewind is added to the remaining suffix before the
sticky cap; nonsticky work retains the rewind allowance. Entry charging and summed
predicate/traversal bounds remain fallible. Earlier APIs, the literal-prefix
proof and callbacks, and the old four-unit paths retain their behavior. No
source/input buffer, parser refactor, dependency, default quota, profile, stack
or worker-policy change is introduced.

A separate typed plan matches exactly two checked character-only v/vi classes
separated by one decoded Unicode literal (22.2.2.3, 22.2.2.7.3). An optional ^
belongs to the first class and optional $ to the second; line assertions inspect
the original input and m context. Both classes independently retain the nested-
class/set-operator gate, and the existing literal decoder proves one complete
middle atom. Borrowed framing charges 3N before either class-end walk or literal
decoding. Complete first-class, literal and second-class proofs then run in
source order, including rejection of hidden string leaves in the second class
behind an empty first class. Flat components, word gates, quantifiers, extra
atoms, inner assertions and partial captures remain separate proofs.

Matching checks the literal sticky after the first class and the second class
sticky after the literal. If either fails, search retries at the first class's
complete-character end, preserving the 1..4 overlap of [[a]]a[[c]] on aaac. All
ranges use original UTF-16 characters, folding and lone-surrogate semantics,
including the documented leading-boundary behavior for an initial low surrogate.
Runtime uses a separate zero-inner-capture variant and the existing complete
whole-body capture proof.

The middle variant shares the existing three-character six-unit work path. The
possible one-unit initial-pair rewind is added to the remaining suffix before
the sticky cap; nonsticky work retains the rewind allowance. Entry charging and
summed predicate/traversal bounds remain fallible. Earlier APIs, callback bodies,
prefix/suffix plans and four-unit paths retain their behavior. No source/input
buffer, parser refactor, dependency, default quota, profile, stack or worker-policy
change is introduced.

A separate typed plan matches exactly three unquantified checked character-only
v/vi classes (22.2.2.3, 22.2.2.7.3). Each class independently retains the nested-
class/set-operator gate. Optional outside ^/$ belong to the first/third class and
inspect original input and m context. Borrowed framing charges 3N before the
three class-boundary walks. Complete first-, second- and third-class proofs run
in source order, including rejection of a hidden string leaf in the third class
behind an empty first or second set. Complete whole-body captures remain a
separate Runtime caller proof; the typed plan has no inner captures.

The first class searches for a candidate; the second and third then match sticky
at the preceding complete-character end. Either later failure retries the first
class's end, preserving the 1..4 overlap of [[a]][[a]][[c]] on aaac. Original
UTF-16 ranges, actual i/m behavior, lone surrogates and the documented leading-
boundary behavior at an initial low surrogate are retained. Three complete
characters consume at most six UTF-16 units. The Runtime variant shares the
existing six-unit work path, initial-pair rewind and fallible entry/predicate
charging.

Flat components, word gates, extra atoms, repetitions, string-valued leaves and
inner assertions or partial captures remain outside this three-class proof.
Earlier typed APIs retain their closed domains, callbacks and search behavior,
including their existing triple-class None controls. The direct Runtime work-
bound unit gains assertions without a new test function. No source/input buffer,
dependency, default quota or build-profile change is introduced.

A separate typed wrapper matches canonical outside word assertions around
exactly three checked character-only v/vi classes (22.2.2.3, 22.2.2.4). Its
borrowed order is ^? word? CHECKED CHECKED CHECKED word? $?, with at least one
\b/\B. Each class independently retains the nested-class/set-operator gate.
The wrapper lives beside the plain triple matcher without changing its stored
fields, methods, callbacks or closed domain. Complete whole-body captures remain
a separate Runtime caller proof. Flat components, string leaves, extra atoms,
quantifiers, inner/reordered assertions and partial captures remain separate work.

Construction charges 4N before canonical framing and the three class-end walks,
then runs all three complete bare-class proofs in source order, including
hidden-string rejection after an empty first or second set. No source slice or
input buffer is copied. The plain triple constructor retains its original 3N
work transcript. Outside endpoint checks use actual i WordCharacters independently
of fold-invariant class predicates, and original m line context. After a later
class or outside gate fails, search retries the first class's complete-character
end. This preserves the 1..4 overlap of \B[[a]][[a]][[a]]\b on aaaa!.

All ranges and neighboring assertions inspect original UTF-16 characters,
including lone surrogates and the documented initial-low-pair leading boundary.
The wrapper adds the established endpoint allowances to the plain triple search
bound and keeps one fallible entry charge. Its separate zero-inner-capture
Runtime variant joins the unchanged six-unit work path and complete enclosing-
capture proof. Existing rewind/saturation arithmetic, four-unit paths and prior
six-unit assertions remain in use. No dependency, default quota, profile, stack
or worker-policy change is introduced.

A separate typed wrapper composes canonical outside word assertions with one
decoded Unicode literal followed by exactly two checked character-only v/vi
classes (22.2.2.3, 22.2.2.4). Its borrowed order is
^? word? LITERAL CHECKED CHECKED word? $?, with at least one \b/\B. Each class
independently retains the nested-class/set-operator proof. Complete whole-body
captures remain a separate Runtime caller proof. Flat classes, extra atoms,
quantifiers, inner/reordered assertions, partial captures and string leaves
remain outside this domain.

Construction charges 4N before framing, literal decoding and both class-boundary
walks. It then proves the literal and both complete bare classes in source order,
including rejection of a hidden string leaf after an empty first set. The plain
literal-prefix matcher retains its original fields, methods, closed domain and
3N callback transcript. No source slice or input buffer is copied. Actual i
WordCharacters govern the outside endpoints independently of fold-invariant
membership; optional ^/$ use the original input and m line context.

A failed class or outside gate retries the literal's complete-character end,
preserving the 1..4 overlap of \Ba[[a]][[a]]\b on aaaa!. Sticky failure returns
no match. Ranges and neighboring assertions retain original UTF-16 characters,
lone surrogates and the documented initial-low-pair leading boundary. The new
zero-inner-capture Runtime variant shares the existing three-character six-unit
work path and rewind/saturation arithmetic. It shares the existing search precharge
and one fallible entry charge. Earlier four-unit paths and complete enclosing
captures remain unchanged.

A separate typed wrapper composes canonical outside word assertions with exactly
two checked character-only v/vi classes followed by one decoded Unicode literal
(22.2.2.3, 22.2.2.4). Its borrowed order is
^? word? CHECKED CHECKED LITERAL word? $?, with at least one \b/\B. Both classes
independently retain the nested-class/set-operator proof. Complete whole-body
captures remain a separate Runtime caller proof. Flat classes, extra atoms,
quantifiers, inner/reordered assertions, partial captures and string leaves
remain outside this domain.

Construction charges 4N before canonical framing, both class-boundary walks and
literal decoding. Both classes and the literal are then proved completely, in
source order, including hidden-string rejection after an empty first or second
set. The plain checked-pair/literal matcher retains its original fields, methods,
closed domain and 3N callback transcript. No source slice or input buffer is
copied. Actual i WordCharacters govern the outside endpoints independently of
fold-invariant membership; optional ^/$ use original input and m line context.

A failed second class, literal or outside gate retries the first class's end at
an original character boundary, preserving the 1..4 overlap of \B[[a]][[a]]a\b on aaaa!. Sticky
failure returns no match. Ranges and neighboring assertions retain original
UTF-16 characters, lone surrogates and the documented initial-low-pair leading
boundary. The separate zero-inner-capture Runtime variant shares the existing
three-character six-unit work path and rewind/saturation arithmetic, with the
established search precharge and one fallible entry charge. Earlier APIs retain
their closed domains and callbacks; four-unit paths and complete enclosing
captures remain unchanged.

A separate typed wrapper composes canonical outside word assertions with one
checked character-only v/vi class, one decoded Unicode literal and a second
independently checked class (22.2.2.3, 22.2.2.4). Its borrowed order is
^? word? CHECKED LITERAL CHECKED word? $?, with at least one \b/\B. Both classes
retain the nested-class/set-operator proof. Complete whole-body captures remain
a separate Runtime caller proof. Flat classes, extra atoms, quantifiers,
inner/reordered assertions, partial captures and string leaves remain outside
this domain.

Construction charges 4N before canonical framing, class-boundary walks and
literal decoding. The first class, literal and second class are proved completely
in source order, including hidden-string rejection after an empty first
set. The plain middle-literal matcher retains its original fields, methods,
closed domain and 3N callback transcript. No source slice or input buffer is
copied. Actual i WordCharacters govern outside endpoints independently of
fold-invariant membership; optional ^/$ use original input and m line context.

A failed literal, final class or outside gate retries the first class's end at
an original character boundary, preserving the 1..4 overlap of \B[[a]]a[[a]]\b
on aaaa!. Sticky failure returns no match. Ranges and neighboring assertions retain original
UTF-16 characters, lone surrogates and the documented initial-low-pair leading
boundary. The separate zero-inner-capture Runtime variant shares the existing
three-character six-unit work path and rewind/saturation arithmetic, with the
established search precharge and one fallible entry charge. Earlier APIs retain
their closed domains and callbacks; four-unit paths and complete enclosing
captures remain unchanged.

A separate typed matcher compiles a finite source run of at least four
independently checked, unquantified character-only v/vi classes
(22.2.2.3, 22.2.2.4, 22.2.2.7.3). It accepts a plain run or canonical framing
^? word? CHECKED...CHECKED word? $?, with at most one \b/\B at each endpoint.
Both line and word assertions are optional. The source supplies the finite term
count; no arbitrary term ceiling is introduced. Complete whole-body captures
remain a separate Runtime caller proof. Earlier one-, two- and three-term APIs
retain their closed domains, fields, callbacks and search behavior.

Construction charges 4N before borrowed framing and both complete class-boundary
passes. The first pass validates the entire class-only shape and count before
program allocation or component compilation. Class storage is then reserved
fallibly, and every complete character-only class proof runs in source order.
An empty early set never skips later string-leaf rejection or source validation.
Only a fully constructed immutable shared program is published; capacity and
work failures retain their host-error categories. No source or input buffer is
copied, and no general parser refactor or recursive term search is required.

The first class searches for a candidate; every later class matches sticky at
the preceding original complete-character end. A later class or outside-gate
failure retries the first class's end in nonsticky mode. Sticky failure returns
no match. Original UTF-16 ranges, lone surrogates and the documented initial-low-
pair leading boundary remain in use. Outside words use actual i WordCharacters
independently of fold-invariant class predicates; line assertions use original
input and m context. Whole captures retain the selected complete range.

Search-pass bounds conservatively sum all component predicates, term traversal
and endpoint allowances with saturating arithmetic and one fallible entry
charge. A dedicated Runtime branch uses the proved term count: sticky search is
bounded by min(2 * count, remaining + 1), with saturating arithmetic; nonsticky
search retains remaining + 1. The extra unit covers initial pair rewind. Source
length is not a substitute for this character-derived bound. Existing two-,
four- and six-unit paths and their tests remain unchanged. A separate direct
unit verifies the new dynamic bound.

A separate flat constructor on the same finite class-sequence matcher admits at
least three unquantified flat character-only v/vi classes (22.2.2.3, 22.2.2.4,
22.2.2.7.3). Framing is ^? word? FLAT...FLAT word? $?, with at most one
\b/\B at each endpoint; line and word assertions are optional. Every component
uses the existing borrowed flat-class proof with actual i/m and UnicodeSets
semantics. Nested or operator classes, mixed proof domains, literal atoms,
quantifiers, reordered/inner assertions, partial captures and string leaves
remain outside this constructor. Complete whole-body captures remain a separate
Runtime caller proof. Earlier flat-pair and checked-sequence constructors retain
their exact closed domains and callbacks.

Construction charges 4N before borrowed framing and both class-boundary passes.
The complete shape and finite source-derived count are validated before fallible
storage reservation. Every component proof then runs in source order, including
later hidden-string rejection behind an empty first class. The new constructor
publishes the existing immutable shared program only after full success; work
and capacity failures retain their host categories. No source or input buffer
is copied, and no new count ceiling or resource default is introduced.

The existing sequence search remains unchanged: later terms match sticky at the
previous original-character end, and a late term or outside-gate failure retries
the first class's end in nonsticky mode. Sticky failure returns no match. Outside
words inspect actual i WordCharacters; line assertions inspect original input
and m context. Ranges and complete captures preserve original UTF-16, lone
surrogates and the documented initial-low-pair leading boundary. The same Runtime
variant retains its term-derived saturating sticky min(2 * count, remaining + 1)
bound, nonsticky remaining + 1 and fallible entry charge. The separate flat-bound
unit verifies counts 3/4/17 without changing the checked bound or older fixed
width paths.

A separate mixed constructor on the same finite class-sequence matcher admits
at least three unquantified character-only v/vi classes with both a proved flat
class and a proved checked nested/operator class (22.2.2.3, 22.2.2.4, 22.2.2.7.3).
Canonical framing remains ^? word? CLASS...CLASS word? $?, with optional line
assertions and at most one \b/\B at each endpoint. Complete whole-body captures
remain a separate Runtime caller proof. Homogeneous sources remain outside this
new entry; their earlier constructors retain their exact domains and callbacks.
Mixed pairs, literals, repetition, alternatives, inner/reordered assertions,
partial captures and string leaves retain their earlier separate treatment.

Construction retains the borrowed 4N precharge and complete class-only shape/
count pass before fallible exact storage reservation. The second boundary pass
proves every component in source order. Each component tries the existing flat
primitive first; only Ok(None) tries the checked primitive. Flat work errors and
checked work/capacity errors propagate immediately, and failed-flat charges
remain part of the callback transcript. If both proofs return None, the source
is unsupported. A successful empty class still records its proved kind and
never skips later validation or hidden-string rejection. After all components
succeed, both kinds are required before the existing immutable shared program
is published. No additional classifier walk, kind vector, source/input copy,
count ceiling, representation or default resource limit is introduced.

The existing sequence search, captures, GC ownership and Runtime dynamic work
branch remain unchanged. Later terms match sticky at the preceding original
character end; a late term or outside-gate failure retries the first class end
in nonsticky mode. Sticky failure returns no match. Word gates retain actual i
WordCharacters, and line gates retain original input and m context. Whole ranges
and complete captures preserve original UTF-16, lone surrogates and the approved
initial-low-pair leading boundary. Sticky work retains the saturating
min(2 * count, remaining + 1) bound, and nonsticky work retains remaining + 1,
independent of source length or capacity. A separate mixed-bound unit verifies
counts 3/4/17 without changing any preceding bound unit or fixed-width path.

## Flat UnicodeSets class followed by one literal

A separate constructor proves one unquantified flat character-only v/vi class
followed by exactly one decoded Unicode literal, with optional outside ^/$.
Complete whole-body captures retain the existing caller proof. Outside word
gates, partial captures, quantifiers, extra terms and string leaves keep their
separate treatment; all earlier constructor domains and callbacks are unchanged.

Construction borrows original source units and charges saturating 3N before
boundary walks or literal decoding. The complete two-term shape is proved
before component compilers run. The flat class is then compiled first and the
literal second in source order, with actual i/m flags and immediate work-error
propagation. A successful empty class never skips complete literal validation.
Escaped brackets, dollars and braced Unicode atoms retain their exact framing.
No new source buffer, representation, allocation boundary or resource limit is
introduced.

The existing ClassThenLiteral representation, original-character matching,
complete captures, GC ownership and four-unit Runtime work path are reused.
A late literal failure retries the first class's original character end;
sticky matching attempts only its selected candidate. Original UTF-16 spans,
lone surrogates, line context and the approved initial-low-pair leading boundary
are retained. Sticky work remains min(4, remaining + 1) and nonsticky work remains
remaining + 1, with saturating arithmetic independent of source length. A new
separate bound unit leaves all preceding bounds and tests unchanged.

## Outside words around a flat class and one literal

A separate constructor proves canonical optional ^, optional word gate, one
flat character-only v/vi class, one decoded literal, optional word gate and
optional $. At least one outside \b/\B is required. The existing caller may
prove complete whole-body captures. Literal-first order, checked classes,
partial captures, repetitions, extra atoms, repeated/reordered gates and string
leaves remain separate proofs. All preceding constructors and callbacks retain
their exact domains and order.

Construction borrows original units and charges saturating 3N before complete
framing. It then proves the flat class and literal in source order with actual
i/m and immediate work-error propagation. Neither an empty class nor an
impossible endpoint gate skips later validation. The existing ClassThenLiteral
representation and OutsideAssertions retain actual i WordCharacters, including
Kelvin/long-s classification when class membership itself is invariant, and
original m line context.

Matching preserves complete original characters, UTF-16 spans, lone surrogates
and the approved initial-low-pair leading boundary. A late literal or outside-
gate failure retries the first class end in nonsticky mode; sticky failure does
not retry. Thus \B[a]a\b over aaa! matches original 1..3 after the 0..2 pair
fails its gates. Existing complete captures, GC ownership, search-pass charges
and the Runtime four-unit branch are reused: sticky work is saturating
min(4, remaining + 1), and nonsticky work is remaining + 1, independent of source
length. No new representation, source buffer, allocation boundary or default
resource limit is introduced.

## One literal followed by a flat UnicodeSets class

A separate constructor proves exactly one decoded Unicode literal followed by
one unquantified flat character-only v/vi class, with optional canonical outside
^/$ and the existing caller proof for complete whole-body captures. Outside or
inner word gates, partial captures, quantifiers, extra terms, nested/operator
classes and string leaves retain their separate treatment. All five preceding
constructor domains and callbacks remain unchanged.

Construction borrows original source units and charges saturating 3N before
boundary walks or decoding. The complete literal boundary and full suffix
class_end framing are checked before component compilers run. The literal is
then proved first and the complete flat class second in source order, with
actual i/m flags and immediate work-error propagation. An empty suffix class
still receives its complete proof. Original component slices retain their
outside anchors, escaped brackets/dollars and braced Unicode atoms.

The existing literal-first representation, complete captures, GC ownership and
original-character search are reused without outside-word assertions. A late
class failure retries the first literal's original character end in nonsticky
mode; sticky matching attempts only its selected candidate. Thus a[b] over
aab! selects original 1..3 after the suffix class fails at the first candidate.
Original UTF-16 spans, supplementary/lone characters, original m line context
and the approved initial-low-pair leading boundary are retained. The shared
Runtime path keeps saturating sticky min(4, remaining + 1) and nonsticky
remaining + 1, independent of source length. A separate direct bound test
preserves every earlier bound test and work formula.

## Outside words around one literal and a flat UnicodeSets class

A separate constructor proves canonical optional ^, optional word gate, one
consuming decoded literal, one complete flat character-only v/vi class, optional
word gate and optional $. At least one outside \b/\B is required. Complete
whole-body captures retain the existing caller proof. Inner or reordered gates,
partial captures, quantifiers, extra terms, nested/operator classes and string
leaves remain separate proofs. All six preceding APIs and callbacks are unchanged.

Construction borrows original source and charges saturating 3N before framing
or decoding. The complete literal boundary and full suffix class are framed
before component proofs run in literal-then-class source order, with immediate
work-error propagation. An empty suffix or impossible endpoint gate never skips
complete validation. The existing literal-first representation and
OutsideAssertions retain the caller's actual i/m flags, even when both consuming
predicates are fold-invariant. Thus `^\B\$[\p{Any}]\b$` matches `$K` or `$ſ`
only with i: the consuming predicates stay the same while the final word gate
changes. Original input and multiline context determine every assertion.

Matching preserves original complete-character UTF-16 ranges, supplementary and
lone characters, enclosing captures and the approved initial-low-pair leading
boundary. A late class or outside-gate failure retries the first literal's end;
sticky failure does not retry. In particular, \Ba[a]\b over aaa! selects 1..3
after the first pair fails its gates. The existing search precharge and one
fallible entry charge are shared. Sticky work remains saturating
min(4, remaining + 1) and nonsticky work remains remaining + 1, independent of
source length. No new representation, source buffer, allocation boundary,
dependency or default resource limit is introduced.

## Two flat UnicodeSets classes separated by one literal

A separate constructor proves exactly two independently flat character-only
v/vi classes separated by one decoded Unicode literal. Canonical outside line
anchors and word gates are optional: optional ^, optional \b/\B, the complete
class/literal/class body, optional \b/\B and optional $. The existing caller may
prove complete whole-body captures. Checked or mixed class kinds, inner or
reordered assertions, partial captures, quantifiers, extra atoms and string
leaves remain separate proofs. Every preceding constructor and callback retains
its closed domain.

Construction borrows original source units and charges saturating 4N before
framing or decoding. The complete frame is proved before full flat-class,
literal and flat-class proofs run in source order. A successful empty first
class cannot skip the later literal or class proof, and work errors propagate
immediately. The existing class/literal/class word-wrapper representation also
admits the bare form with absent word gates; no new fields or matching plan are
needed.

Outside assertions use actual i WordCharacters and original m line context,
even when all three consumers are case-invariant. Thus
`^\B[\p{Any}]\$[\p{Any}]\b$` matches `!$K` or `!$ſ` only with i: Any, the
literal dollar and Any retain their membership while the final word boundary
changes. A leading-only gate does not imply a trailing gate, so `\b[a]b[c]`
still matches original 0..3 in abcx.

Matching retains complete original UTF-16 characters, supplementary and lone
surrogates, complete captures and the approved initial-low-pair boundary.
A late literal, last-class or outside-gate failure retries the first class end
in nonsticky mode; sticky failure does not retry. In particular, `[a]a[a]\b`
over aaaa! selects 1..4 after the first candidate fails its final gate. The
existing entry charge, search-pass accounting, Runtime variant and six-unit
branch are unchanged: sticky work remains saturating min(6, remaining + 1)
and nonsticky work remains remaining + 1, independent of source length.
Earlier two-atom four-unit bounds and all earlier direct units remain exact.
No new representation, source or input buffer, dependency or default resource
limit is introduced.

## One literal followed by two flat UnicodeSets classes

A separate constructor proves exactly one decoded Unicode literal followed by
two independently flat character-only v/vi classes. Canonical outside line
anchors and word gates are optional: optional ^, optional \b/\B, the complete
literal/class/class body, optional \b/\B and optional $. The existing caller
proof handles complete whole-body captures. Checked or mixed class kinds,
string leaves, inner or reordered assertions, partial captures, quantifiers
and extra atoms remain separate proofs. Every preceding constructor and
callback retains its closed domain.

Construction borrows original source units and charges saturating 4N before
framing or decoding. The complete frame is proved before the literal, first
flat class and second flat class are fully compiled in source order. A
successful empty first class cannot skip the final class proof, including a
late string-property rejection. An explicit \q string leaf instead rejects
during framing before component callbacks. Work errors propagate immediately.
The existing matcher stores the literal, two class predicates and outside
assertions; absent word assertions also represent the bare form.

Outside assertions retain actual i WordCharacters and original m line context,
even when all three consumers are case-invariant. Thus
`^\B\$[\p{Any}][\p{Any}]\b$` matches `$!K` or `$!ſ` only with i: the literal
dollar and both Any classes retain their membership while the final word gate
changes. A leading-only gate does not imply a trailing gate, so `\bc[a][b]`
still matches original 0..3 in cabx.

Matching retains complete original UTF-16 characters, supplementary and lone
surrogates, complete captures and the approved initial-low-pair boundary.
Failure in either class or an outside gate retries the literal end in
nonsticky mode; sticky failure does not retry. In particular, `a[a][b]` over
aaab! and `a[a][a]\b` over aaaa! both select 1..4 after the first candidate
fails. The existing entry charge, search-pass accounting, Runtime variant and
six-unit branch remain unchanged: sticky work is saturating
min(6, remaining + 1) and nonsticky work is remaining + 1, independent of
source length. Earlier two-atom four-unit bounds and direct units remain exact.
No new representation, source or input buffer, dependency or default resource
limit is introduced.

## Two flat UnicodeSets classes followed by one literal

A separate constructor proves exactly two independently flat character-only
v/vi classes followed by one decoded Unicode literal. Canonical outside line
anchors and word gates are optional: optional ^, optional \b/\B, the complete
class/class/literal body, optional \b/\B and optional $. The existing caller
proof handles complete whole-body captures. Checked or mixed class kinds,
string leaves, inner or reordered assertions, partial captures, quantifiers
and extra atoms remain outside this constructor. Every preceding constructor
and callback retains its closed domain.

Construction borrows original source units and charges saturating 4N before
framing or decoding. The complete frame is proved before the first flat class,
second flat class and literal are fully compiled in source order. A successful
empty first or second class cannot skip later component proofs. In particular,
a string property in the second class is still rejected after a successful
empty first class; no literal proof follows that rejection. An explicit \q
string leaf instead rejects during framing before component callbacks. Work
errors propagate immediately. The existing matcher stores two class predicates,
the literal and outside assertions; absent word assertions also represent the
bare form.

Outside assertions retain actual i WordCharacters and original m line context,
even when all three consumers are case-invariant. Thus
`^\b[\p{Any}][\p{Any}]\$\B$` matches `K!$` or `ſ!$` only with i: both Any
classes and the literal dollar retain their membership while the leading word
gate changes. A leading-only gate does not imply a trailing gate, so
`\b[a][b]c` still matches original 0..3 in abcx.

Matching retains complete original UTF-16 characters, supplementary and lone
surrogates, complete captures and the approved initial-low-pair boundary.
Failure in the second class, literal or an outside gate retries the first
class end in nonsticky mode; sticky failure does not retry. In particular,
`[a][b]c` over aabc, `[a][a]b` over aaab! and `[a][a]a\b` over aaaa! each
select 1..4 after the first candidate fails. The existing entry charge,
search-pass accounting, Runtime variant and six-unit branch remain unchanged:
sticky work is saturating min(6, remaining + 1) and nonsticky work is
remaining + 1, independent of source length. Earlier two-atom four-unit bounds
and direct units remain exact. No new representation, source or input buffer,
dependency or default resource limit is introduced; resource limits remain
explicitly opt-in.

## Literal, checked UnicodeSets class and flat class

A separate constructor proves one decoded Unicode literal followed by a
character-only v/vi class with the existing nested-class or set-operator proof,
then one independently flat class. Canonical optional assertions are ordered
as ^, outside \b/\B, the complete three-term body, outside \b/\B and $.
The existing caller proof admits complete whole-body captures; absent word
assertions also represent the bare form.

Borrowed whole-body framing charges saturating 4N before component proofs.
The literal, checked class and flat class are then compiled fully in source
order, including the flat proof behind a successfully empty checked class.
Explicit \q leaves reject during framing; a hidden string property instead
reaches the corresponding component proof. Work errors propagate immediately.
Existing predicates, storage and Runtime variant retain complete original
UTF-16 ranges, supplementary and lone surrogates, initial-low-pair handling,
actual-i WordCharacters and original-m line context. A failed class or outside
gate retries literal.end in nonsticky mode; sticky failure does not retry.
Complete captures and the existing six-unit work bound remain unchanged.

The reverse flat/checked order, other mixed literal placements, extra atoms,
string-set leaves, inner or reordered assertions, partial captures and
quantified bodies remain outside this entry. Earlier constructors preserve
their closed domains. General unsupported RegExp syntax receives no passing
conformance credit, and resource limits remain explicitly opt-in.

## Literal, flat UnicodeSets class and checked class

A separate entry proves one decoded Unicode literal followed by one flat
character-only v/vi class and one checked nested or set-operator class.
Canonical optional outside assertions retain the order ^, word gate, complete
three-term body, word gate and $. The caller proves complete whole-body
captures; the same representation also admits the bare form.

Construction charges saturating 4N before borrowed whole-frame validation,
then proves the literal, flat class and checked class fully in source order.
A successfully empty flat class cannot skip the checked proof or its errors.
Explicit \q string leaves reject during framing; a hidden string property
reaches its component compiler. Work errors propagate immediately.

The existing matcher retains original UTF-16 ranges, initial-low-pair
handling, supplementary and lone surrogates, actual-i WordCharacters and
original-m line context. A failed class or outside gate retries literal.end
in nonsticky mode; sticky failure does not retry. Captures, storage, search
and the six-unit work bound are shared without changes. Earlier entries keep
their closed domains. Other mixed literal placements, extra atoms, string-set
leaves, inner or reordered assertions, partial captures and quantified bodies
remain outside this entry; resource limits remain explicitly opt-in.

## Checked UnicodeSets class, flat class and literal

A separate entry proves one checked character-only v/vi class, one flat class
and one decoded literal, with canonical optional outside word/line assertions
and caller-proved whole-body captures. Borrowed whole framing charges
saturating 4N before the checked, flat and literal proofs run fully in source
order. A successful empty checked class cannot hide a later flat-class error;
explicit \q leaves reject framing before components, while a hidden string
property reaches its corresponding compiler. Work errors propagate immediately.

The existing ClassPairLiteralWord representation retains original UTF-16,
actual-i word and original-m line context. Later-class, literal or gate failure
retries first.end in nonsticky mode; sticky failure does not retry. Complete
captures and six-unit Runtime work accounting are unchanged. Earlier entries
keep their domains; reverse mixed suffix order, mixed middle placements,
string leaves, extra atoms, inner assertions, partial captures and quantifiers
remain outside this entry. Resource limits remain explicitly opt-in.

## Flat UnicodeSets class, checked class and literal

A separate entry proves one flat character-only v/vi class, one checked nested
or set-operator class and one decoded literal. Canonical optional outside word
and line assertions and complete caller-proved whole-body captures retain the
existing ClassPairLiteralWord representation. Construction charges saturating
4N before borrowed whole framing, then proves flat class, checked class and
literal fully in source order. A successful empty flat class cannot skip the
checked proof; explicit \q leaves reject framing early while a hidden string
property reaches its component compiler. Work errors propagate immediately.

Original UTF-16 ranges, supplementary and lone surrogates, initial-low-pair
handling, word context under the original i flag and line context under the
original m flag are retained. Later-class, literal or gate failure retries
first.end in nonsticky mode; sticky failure does not retry. Earlier entries, complete captures, search and six-unit work
accounting remain unchanged. Mixed middle placements, string leaves, extra
atoms, inner assertions, partial captures and quantified bodies remain outside
this entry; resource limits remain explicitly opt-in.

## Checked UnicodeSets class, literal and flat class

A separate entry proves one checked character-only v/vi class, one decoded
literal and one flat class. Canonical optional outside word and line
assertions and complete caller-proved whole-body captures reuse the existing
ClassLiteralClassWord representation. Construction charges saturating 4N
before borrowed whole framing, then proves checked class, literal and flat
class completely in source order, including after a successful empty checked
class. Explicit \q leaves reject framing early; a later flat string property
still reaches its component proof. Work errors propagate immediately.

Matching retains original UTF-16 text and indices, supplementary and lone
surrogates, initial-low-pair handling, word context under the original i flag
and line context under the original m flag. A later literal, class or outside
gate miss retries first.end in nonsticky mode; sticky calls do not retry.
Earlier typed entries, storage, search, complete captures and six-unit Runtime
accounting remain unchanged. Flat-literal-checked forms, string leaves, extra
atoms, inner assertions, partial captures and quantified bodies remain outside
this entry. Resource limits remain explicitly opt-in.

## Flat UnicodeSets class, literal and checked class

A separate entry proves one flat character-only v/vi class, one decoded
literal and one checked class. Canonical optional outside word and line
assertions and complete caller-proved whole-body captures reuse the existing
ClassLiteralClassWord representation. Construction charges saturating 4N
before borrowed whole framing, then proves flat class, literal and checked
class completely in source order, including after a successful empty flat
class. Explicit \q leaves reject framing early; a later checked string property
still reaches its component proof. Work errors propagate immediately.

Matching retains original UTF-16 text and indices, supplementary and lone
surrogates, initial-low-pair handling, word context under the original i flag
and line context under the original m flag. A later literal, class or outside
gate miss retries first.end in nonsticky mode; sticky calls do not retry.
Earlier typed entries, storage, search, complete captures and six-unit Runtime
accounting remain unchanged. Checked-literal-flat forms, string leaves, extra
atoms, inner assertions, partial captures and quantified bodies remain outside
this entry. Resource limits remain explicitly opt-in.

## Standalone one-character UnicodeSets string disjunctions

A separate v/vi entry proves a standalone [\q{...}] or [^\q{...}] whose
alternatives each decode to exactly one Unicode character. Canonical optional
outside word and line assertions and complete caller-proved enclosing captures
reuse the existing UnicodeClass representation and two-unit Runtime bound.
Construction charges saturating 3N before borrowed framing and branch decoding,
then uses the existing fallible charged interval normalization and case closure.
Every branch is proved before deduplication or folding can hide a late empty or
multi-character alternative. Folding precedes outer inversion.

ClassString \b is backspace. Escaped class punctuation, raw supplementary
characters and adjacent fixed Unicode surrogate escapes retain their character
values; braced or separated lone-surrogate alternatives remain distinct.
Matching retains original UTF-16 text, initial-low-pair handling and actual i/m
assertion context. Existing parser MayContainStrings errors remain SyntaxError.
Empty or multi-character positive alternatives, mixed members, nesting, set
operations, repetition, continuations and properties of strings remain outside
this entry. Earlier typed compilers and composite \q rejection are unchanged;
resource limits remain explicitly opt-in.

Edition 17 [CompileToCharSet](https://262.ecma-international.org/17.0/#sec-compiletocharset)
applies [MaybeSimpleCaseFolding](https://262.ecma-international.org/17.0/#sec-maybesimplecasefolding)
to class-string disjunctions before matching and complementing them. Thus
`[\q{K|s}]` with `iv` accepts Kelvin sign, and its complement rejects Kelvin
sign and long s.
Node v24.19.0, which bundles V8 13.6.233.17, omits the singleton class-string
case-equivalence closure: `[\q{K}]` with `iv` matches lowercase k but rejects
uppercase K and Kelvin sign. Spite retains the specification behavior. V8
fixed this defect in
[bca5c4eb1db2](https://github.com/v8/v8/commit/bca5c4eb1db2dc36cffd133eb5c9d2ce8b54f1f6),
including Kelvin, long-s and complement regressions. Differential evidence
retains the original Node results and records equivalent ordinary flat-class
checks separately; the handwritten native expectations stay unchanged.

## One-character UnicodeSets string operands in checked class algebra

A separate v/vi entry admits one complete checked character class containing
at least one ClassStringDisjunction operand whose alternatives each decode to
one character. Existing unions, nested complements, intersection chains and
subtraction chains combine these operands with the supported character-only
leaves. Canonical optional outside word and line assertions and complete
caller-proved enclosing captures retain the UnicodeClass representation and
two-unit Runtime search bound.

The shared iterative compiler keeps its earlier mode unchanged and enables
string operands only for the new entry. Every q alternative and every operand
is proved in source order, including after an empty intermediate result. The
new entry charges N before canonical framing, then uses the existing 4N checked
algebra and fallible charged interval paths. The standalone q entry retains its
3N framing charge and exact callback transcript. Resource limits remain opt-in.

[CompileToCharSet](https://262.ecma-international.org/17.0/#sec-compiletocharset)
applies simple case folding to each operand before Boolean operations and
complement. Each admitted q operand therefore has the same character set as
its equivalent ordinary character class. Matching preserves original UTF-16
text, the approved initial-low-surrogate behavior and actual i/m assertion
context. Multi-character or empty q alternatives, consuming continuations,
quantifiers and properties of strings remain outside this entry.

## Repetition of checked one-character UnicodeSets string operands

A separate v/vi entry compiles one supported checked class containing at least
one single-character ClassStringDisjunction operand and one existing greedy or
lazy quantifier. Optional outside line and word assertions apply to the whole
repetition. Complete enclosing captures remain the Runtime caller's proof.
The same class algebra, compact decimal bounds and UnicodeRepeatedAtom storage
handle empty sets, zero iterations and counts beyond the native integer range;
the implementation does not expand exact counts.

The new route charges N before borrowed canonical assertion framing, then uses
the shared repetition constructor's existing 8N parsing charge and the checked
class proof. Its private q-class wrapper adds no callback to the earlier class
entry. Previous constructors retain their accepted domains and work callbacks.
Search retains incremental work over the original input suffix, including long
runs and greedy/lazy endpoint retries. Host quotas remain opt-in.

[CompileQuantifier](https://262.ecma-international.org/17.0/#sec-compilequantifier)
and [CompileToCharSet](https://262.ecma-international.org/17.0/#sec-compiletocharset)
govern count selection and folded character membership. Matching preserves the
earliest candidate, original UTF-16 spans and the approved initial-low-surrogate
behavior. Empty or multi-character q alternatives and properties of strings
decline even under zero or impossible bounds. Inner groups, additional consuming
atoms and reordered assertions remain outside this entry.

## One literal followed by a checked one-character UnicodeSets string class

A separate v/vi entry proves one decoded Unicode literal followed by one
complete checked class containing at least one ClassStringDisjunction operand
whose alternatives each decode to one character. Optional outside line and word
assertions occur in canonical order and apply to the complete pair. Complete
whole-body captures retain the existing Runtime caller proof. The shared
UnicodeClassLiteral representation and matching path remain unchanged.

Construction charges saturating 3N before borrowed framing and atom decoding,
then compiles the exact literal prefix and the complete class suffix in source
order. The retained suffix must start with [ and end with ]; the internal class
compiler cannot absorb an additional line gate. No q-aware boundary scanner is
introduced, and every earlier constructor keeps its domain and work callbacks.
All q alternatives and later class operands are proved even after an empty
intermediate set or an impossible outside gate. Resource quotas remain opt-in.

[CompileToCharSet](https://262.ecma-international.org/17.0/#sec-compiletocharset)
and [CompileSubpattern](https://262.ecma-international.org/17.0/#sec-compilesubpattern)
retain operand-wise simple folding and two adjacent original characters.
Matching preserves original UTF-16 text, supplementary and lone surrogates,
actual i/m assertion context and the approved initial-low-surrogate boundary.
A suffix-class or outside-gate miss retries the first literal's end in nonsticky
mode; sticky calls attempt only their selected candidate. Existing search work
remains saturating min(4, remaining + 1) for sticky calls and remaining + 1 for
nonsticky calls, independent of source length. Class-first q forms, partial
captures, additional atoms, quantifiers, reordered assertions, empty or longer
q alternatives and properties of strings remain outside this entry.

## Complete inner captures around one-character UnicodeSets string repetition

A separate v/vi entry admits complete nested ordinary capturing/noncapturing
wrappers around one checked class containing singleton-character q operands,
followed by one existing greedy/lazy quantifier. Canonical optional outside
line and word predicates apply to the complete repetition. Each inner capture
selects the final original character, or remains undefined after zero selected
iterations. Complete enclosing captures retain the caller's whole-match scope.

Construction borrows the original source. A saturating 16N charge bounds outside
framing, both existing quantifier interpretations and wrapper walks before any
scan. Exact prefix openers and a corresponding closing run surround a bare
[...]-class. Bounds are parsed first; zero or impossible bounds cannot skip the
complete checked q compiler's proof of every leaf. Its existing fallible
work/storage charges remain unchanged, including after empty results and under
unrepresentable counts. No class/group scanner, copied Pattern, quantifier
expansion or default quota is introduced.

[RepeatMatcher](https://262.ecma-international.org/17.0/#sec-repeatmatcher)
retains final-iteration capture effects and undefined zero-iteration slots.
The existing UnicodeRepeatedAtom representation, original-character matching,
whole-body captures and incremental full-suffix accounting are reused, including
supplementary/lone characters and the approved initial-low-surrogate boundary.
Actual i/m predicates retain original input context. Inner assertions, partial
captures, alternatives, extra atoms, nullable branches and string-valued leaves
remain outside this entry. Earlier constructors keep their domains/callbacks.

## Raw empty alternatives in one-character UnicodeSets string repetition

A separate v/vi entry proves one raw empty branch and one complete singleton-q
checked class, in either source order, enclosed by complete capturing/noncapturing
wrappers and followed by *, + or ? with optional laziness. Canonical outside
line/word gates and caller-proved whole captures retain their original scopes.
The consuming leaf is fully proved even if an empty path could succeed first.

Borrowed framing checks exact prefix openers, a matching closing run, exactly
one edge bar and a complete bare class. It adds no q scanner or copied Pattern.
A conservative 10N source charge precedes all framing; braced tokens decline
before the quantifier helper, and existing class work/capacity callbacks remain
separate and unchanged. Prior constructor domains and callbacks are preserved.

[RepeatMatcher](https://262.ecma-international.org/17.0/#sec-repeatmatcher)
rejects optional zero-progress iterations, leaving inner captures undefined
when no iteration participates. A mandatory empty plus iteration instead defines
an empty capture. Source branch order and lazy continuation priority matter:
character-first lazy plus can prefer a consuming path over an empty path at the
same start, while the earliest candidate still wins. Existing nullable order,
Atom::Class storage, final-capture projection and incremental full-suffix search
implement these effects over original UTF-16 boundaries, including the approved
initial-low-surrogate handling. No new matcher/search model or default limits
are added. Branch-local captures/assertions, grouped empty branches, braced
counts, extra alternatives and string-valued leaves remain outside this entry.

## One-character UnicodeSets string classes followed by one literal

A separate checked v/vi entry composes one complete class containing at least
one ClassStringDisjunction operand with one decoded Unicode literal. Every q
alternative must consume exactly one character. Canonical outside ^, leading
\b/\B, trailing \b/\B and $ apply to the complete pair; enclosing whole
captures remain the caller's proof.

The new private boundary walk borrows validated source. ClassString syntax
forbids raw brackets inside q payloads, and escaped brackets/backslashes remain
opaque pairs. Complete class compilation independently proves every q leaf and
set operator. The earlier boundary scanner continues to reject q. A conservative
4N charge precedes both outside parity walks, the boundary scan and suffix
predecoding. The complete class and literal then compile in source order, even
behind an empty class or an impossible gate, retaining fallible class storage.

The existing ClassThenLiteral order, first-character-end retries and original
UTF-16 line/word context implement search. Sticky work remains bounded to four
code units, including the approved initial-low-surrogate behavior. No new
matcher storage, copied Pattern, default host limit or dependency is introduced.
Partial captures, extra atoms, repetition and string-valued alternatives remain
outside this entry. Earlier constructors retain their domains and callbacks.

## A checked one-character UnicodeSets string class followed by a flat class

A separate checked v/vi entry composes one complete class containing singleton
ClassStringDisjunction operands with one ordinary flat class. Every q alternative
must decode to one character. The suffix retains its independent flat proof;
nested set algebra and string-valued suffixes remain outside this entry.

Complete v validation precedes the borrowed boundary proof. A private prefix
scanner skips escaped pairs; valid q payloads cannot contain raw brackets. The
unchanged legacy scanner proves the suffix boundary and continues to reject q.
The full checked prefix and full flat suffix compile in source order, including
when the first class is empty or an outside assertion cannot succeed. Existing
fallible work and class storage errors propagate.

A conservative 4N charge precedes both outside parity walks and both class
boundary scans. Canonical outside ^, leading \b/\B, trailing \b/\B and $
apply to the complete pair. The existing pair matcher preserves original UTF-16
context and retries at the first character's end; sticky precharge remains four
code units, including the approved initial-low-surrogate handling. Complete
whole-body capture proof and result projection remain with the caller.

Earlier constructors and search plans retain their domains and callbacks. No
matcher storage, copied Pattern, default limit or dependency is added. Reverse
order, two q classes, repetition, extra terms and partial captures remain outside
this entry.

## A flat class followed by a checked one-character UnicodeSets string class

A separate validated v/vi entry composes one ordinary flat class with one
complete checked class containing single-character ClassStringDisjunction
operands. The flat prefix and checked suffix compile completely in source
order, including the suffix after an empty prefix or an impossible outside
assertion. Each q alternative must decode to one original Unicode character;
operand-wise simple folding precedes set operations and complement.

The adapter borrows validated source and shares the pair matcher's private
checked-class boundary scanner. Its observable initial 4N charge covers both
outside parity walks and both class-boundary walks. Child compilation retains
its fallible work and capacity checks. The legacy scanner and earlier compiler
entry points retain their existing domains and callbacks.

Canonical outside line/word assertions and complete enclosing captures reuse
the existing pair plan, original UTF-16 context and overlapping retry from the
first character's end. Sticky accounting remains four code units, including
the approved initial-low-surrogate behavior. Two checked q classes, repetition,
additional atoms and partial captures remain outside this entry.

## Two checked one-character UnicodeSets string classes

A separate v/vi entry proves exactly two complete checked classes. Each class
must contain at least one ClassStringDisjunction operand, and every alternative
must decode to one original Unicode character. Operand-wise simple folding
precedes set operations and complement
([CompileToCharSet](https://262.ecma-international.org/17.0/#sec-compiletocharset)).

The adapter borrows validated source and uses the shared checked-class boundary
walk for both components. An observable 4N framing charge precedes both outside
parity walks and both class-boundary walks. Both complete classes compile in
source order through the existing fallible work and capacity checks, including
when the first set is empty or an outside assertion cannot succeed.

Canonical outside ^, leading \b/\B, trailing \b/\B and $ apply to the complete
pair. The existing pair matcher preserves original UTF-16 context, overlapping
retry from the first character's end and four-unit sticky accounting, including
the approved initial-low-surrogate behavior. Complete enclosing captures and
result projection retain the caller's existing proof.

Earlier entries keep their domains and callbacks. This entry requires q
operands in both classes; ordinary non-q checked operands use separate entries.
Repetition, additional atoms and partial captures remain outside this entry.
Matcher storage and dependencies are unchanged; host resource quotas remain
opt-in.

## An ordinary checked class followed by a one-character UnicodeSets string class

A separate v/vi pair entry requires a non-q checked prefix with a nested class
or set operator, then a complete q-aware checked suffix. Every q alternative
must decode to exactly one original character. The borrowed 4N framing proof
finishes before the legacy checked prefix and q-aware suffix compile fully in
source order, even after an empty first set or an impossible outside gate.
Operand-wise simple folding precedes set operations and complement
([CompileToCharSet](https://262.ecma-international.org/17.0/#sec-compiletocharset)).

Canonical outside line/word assertions and complete enclosing captures reuse
the existing pair representation, original UTF-16 context, first-end overlap
retry and four-unit sticky accounting, including the approved initial low-pair
rewind. Earlier eleven APIs, scanners, callbacks, storage and search stay
unchanged. Reverse q-to-ordinary-checked order, partial captures, repetition,
extra atoms, noncanonical assertions and empty or multicharacter q alternatives
remain outside this entry. Parser string-property early errors remain distinct
from unsupported valid syntax and optional host limits; dependencies and default
resource policy are unchanged.

## A one-character UnicodeSets string class followed by an ordinary checked class

The reverse v/vi pair entry proves a complete q-aware prefix followed by an
ordinary non-q checked suffix with a nested class or set operator. Every q
alternative must decode to one original Unicode character. A prepaid 4N framing
scan borrows both component boundaries; both components then compile in source
order, including the suffix after an empty prefix or impossible outside gate.
Operand-wise simple folding precedes algebra and complement
([CompileToCharSet](https://262.ecma-international.org/17.0/#sec-compiletocharset)).

Canonical outside line/word assertions and complete enclosing captures reuse
the existing pair representation and result projection. Search preserves original
UTF-16 context, overlapping retry, four-unit sticky accounting, and the approved
initial low-surrogate rewind. The Runtime tries this proof after earlier v/vi
proofs. Existing public compilers, scanner bodies, callbacks, storage, and search
remain unchanged. Flat suffixes, partial captures, repetition, extra atoms,
noncanonical assertions, empty or longer q alternatives, and string properties
remain outside this entry. Default resource policy and dependencies are unchanged.

## Empty and multi-character UnicodeSets string alternatives

Core provides a separate checked matcher for one complete positive v/vi class
whose sole operand is a ClassStringDisjunction containing an empty or
multi-character member. It shares the existing ClassSetCharacter decoder,
including fixed-escape surrogate pairing and separate braced surrogate escapes.
Each character receives simple folding before insertion into a flat prefix trie.
Shared prefixes and duplicate alternatives reuse nodes; construction and drop
do not recurse. Earlier character-only compilers retain their proof domains.

At each original input position the matcher retains the longest member whose
outside assertions succeed. A rejected longer member leaves shorter members
and the empty member eligible, as required by
[CompileAtom](https://262.ecma-international.org/17.0/#sec-compileatom).
Canonical outside line and word assertions inspect original UTF-16 context.
Initial low-surrogate offsets retain the approved leading-boundary behavior,
including empty matches. Search allocates no state.

Construction prepays borrowed framing/decoding and charges folding, trie
lookups, writes, and possible vector movement or hash-index rebuilding before
fallible reservations. The final Arc retains the existing shared-allocation
boundary. Sticky work is bounded by one longest-member walk; searching work
includes that walk at every candidate, including the final empty position.
Runtime selects this proof after earlier v/vi entries, projects complete enclosing
captures through the existing result representation, and reuses RegExpBuiltinExec
and the String consumers. Sticky work includes one candidate and bounded member
consumption; searching work includes the final empty candidate and a possible
initial pair rewind. Opted-in precharge failures precede search and lastIndex
writes. Class algebra, properties of strings, additional consuming terms, inner
captures, and repetition remain outside this entry. Dependencies and default
resource policy are unchanged.

V8 13.6 (Node v24.19.0) joins adjacent braced surrogate escapes inside q strings:
`[\q{\u{D800}\u{DC00}}]` matches U+10000 there, while its ordinary
`\u{D800}\u{DC00}` concatenation correctly fails. Edition 17
[CompileClassSetString](https://262.ecma-international.org/17.0/#sec-compileclasssetstring)
concatenates two character sequences; CompileAtom invokes CharacterSetMatcher
for each character. The first surrogate cannot match the decoded supplementary
input character. This matcher keeps that specified failure. Adjacent fixed
Unicode escapes still form one character through the grammar's RegExpUnicodeEscapeSequence.

The same V8 version also omits case-fold closure for singleton q members beside
longer strings: `[\q{a|ab}]` with vi fails on A, while `[a\q{ab}]` matches it.
CompileAtom's singleton CharacterSetMatcher must apply Canonicalize to both
pattern and input characters. The trie preserves that closure, including Kelvin
and long-s equivalences and shorter/empty assertion continuations.

## Adjacent UnicodeSets string operands

A separate checked constructor admits a positive v/vi class consisting of at
least two adjacent ClassStringDisjunction operands, with an empty or longer
member somewhere in the union. It recognizes an unescaped closing brace and
immediately following q opener between operands. The shared character decoder
consumes escaped braces and reverse solidus first, so escaped punctuation cannot
create operand boundaries. Every operand is proved in source order, including
after an empty member. The original single-operand constructor retains its exact
proof and construction-work transcript.

All operands insert into the same flat trie: duplicates and shared prefixes reuse
nodes, and longest-member priority applies across operand boundaries. Folding
precedes membership, as required by
[CompileToCharSet](https://262.ecma-international.org/17.0/#sec-compiletocharset).
Outside assertions and shorter/empty continuations retain original input context.
Runtime selects this proof after the single-operand entry and reuses its capture
projection, search bounds, RegExpBuiltinExec, and String consumers. Ordinary
character/property operands, nesting, intersection/subtraction, further atoms,
and repetition remain separate proofs. No dependencies or default quotas change.

## Ordinary literal characters beside UnicodeSets strings

A separate checked entry admits positive v/vi unions of ordinary literal
ClassSetCharacters and q operands, with at least one literal operand and an empty
or longer q member. The shared decoder preserves escaped delimiters, fixed-pair
and braced-surrogate distinctions, backspace, null, and raw constructor line
terminators. Each literal becomes a one-character member; q alternatives retain
their own boundaries. Both enter the same trie in source order.

The trie builder centralizes checked folding, prefix insertion, terminal marking,
and fallible growth. Earlier single-q and q-only-union framing and callback
sequences are unchanged. All members follow
[CompileAtom](https://262.ecma-international.org/17.0/#sec-compileatom)'s
longer-string, singleton, empty priority. Outside assertions can still reject a
longer member and select a shorter literal or the empty member.

The Runtime tries this entry after q-only proofs and reuses the existing variable
string search, capture projection, and work accounting. Raw intersection is
explicitly rejected; class escapes, properties, ranges, nested classes,
intersection/subtraction, further atoms, and repetition remain separate proofs.
Dependencies, shared-allocation boundaries, and default resource policy are unchanged.

## Character ranges and properties beside UnicodeSets strings

Core adds a separate proof for one complete positive flat v/vi class containing
q strings and a range, character escape, or supported Unicode character property.
Literal characters may also appear. At least one q member must be empty or
multi-character. A borrowed pass proves all flat operands, validates q members,
and compiles ordinary operands through the existing checked interval builder.
A second prepaid pass builds the q trie and skips ordinary terms with the same
scalar-term decoder. No source is rewritten or copied.

The scalar portion batches duplicate escapes and properties, normalizes ranges,
and applies the existing simple-fold/complement rules. Full Unicode ranges stay
as intervals. Search compares scalar membership at the first input character
and all eligible trie terminals, retaining the longest successful member under
the original outside assertions. Empty q members remain last. The bounds include
scalar membership lookup and a two-unit consuming candidate even when every q
member is empty. The existing shared-allocation boundaries and fallible interval,
trie-node, and hash-index reservations remain explicit.

Runtime selects this plan after the earlier q-only and literal-union entries.
Complete enclosing captures reuse the existing result projection. Search work
includes interval membership, a possible initial surrogate rewind, and the final
empty candidate. Opt-in failures precede search and lastIndex writes. Nested
classes, set operations, properties of strings, further atoms, and repetition
remain separate proofs. Earlier public constructors retain their domains and
work callbacks. Dependencies and default resource policy are unchanged.

## Positive nested unions containing UnicodeSets strings

A separate Core proof admits positive nested v/vi unions containing an empty or
longer q member. Positive union associativity lets every scalar operand share one
interval set and every q alternative share one trie. A prepaid borrowed pass
checks nesting with a depth counter, proves every q member and scalar term, and
rejects negative child classes and all set operators. A second borrowed pass
builds q members while skipping the already proved class boundaries and scalar
terms. Character decoders consume escaped brackets before boundary handling.

Construction, storage, search, and drop do not recurse with source depth. Empty
child classes contribute no members. An empty scalar portion is omitted from
the new plan, preserving zero consumption bounds for q-only empty unions.
Earlier flat constructors keep their exact domains and work callbacks. Outside
assertions, simple folding, longest-member priority, and original UTF-16 ranges
reuse the existing interval/trie matcher. Runtime selects this proof after the
earlier flat entries and reuses their whole-body capture projection, search-work
bounds, RegExpBuiltinExec, and String consumers. Negated child classes, set
operations, string properties, further atoms, and repetition remain deferred;
dependencies and default quotas are unchanged.

## Intersections and subtractions of UnicodeSets string operands

Core adds a separate positive flat proof for homogeneous chains of q-only
intersections or subtractions. At least one operand member must be empty or
multi-character. Shared framing retains the earlier constructors' canonical
outside assertions and work sequence. The first operand builds a trie with
unique terminal/length records. Later operands decode and fold completely while
probing that trie; right-only words never allocate nodes or retain source copies.

Checked membership marks and a reusable list deduplicate successful right-hand
probes. Subtraction clears those terminal bits; intersection retains marked
active members. Marks are cleared through the matched list, rather than by
resetting the full trie for each operand. Every later operand is proved after an
empty intermediate result. Final maximum length considers retained members, so
discarded long words do not enlarge search bounds. Search and whole-body ranges
reuse the existing immutable matcher.

Runtime selects this proof after the earlier union entries, projects complete
whole-body captures, and reuses RegExpBuiltinExec and String consumers. Empty
results and surviving empty members share zero consumption bounds but retain
distinct failure/match behavior. Scalar or nested operands, mixed operators,
negation, string properties, further atoms, and repetition remain separate proofs.
Buffer growth is fallible and charged before mutation; the final shared allocation
retains its existing boundary. Dependencies and default quotas are unchanged.

## Finite nested and literal operands in string-set operations

A separate Core proof extends homogeneous positive operation chains to literal
characters and positive nested finite unions of characters and q disjunctions.
At least one operand uses the extended shape and a q member is empty or longer.
A borrowed event lexer emits points and member ends without constructing source
strings. Positive nesting uses a counter. Empty classes emit no member event;
empty q alternatives emit an end event, preserving their distinct set semantics.

The first operand inserts unique words into the trie. Later operands stream
probes and reuse checked membership marks, active terminals, and retained-length
bounds. Shared operation application keeps the earlier q-only proof's work
sequence. Every operand is proved after an empty intermediate result. Ranges,
class/property escapes, negation, inner operations, further atoms, and repetition
remain separate proofs.
Storage and drop remain flat, growth is fallible and prepaid, and dependencies
and default quotas are unchanged.

Runtime selects this proof after q-only string operations. It uses the existing
UnicodeStringSet body, whole-body capture projection, and precharged search work;
no new matcher dispatch or resource policy is needed. All operands are proved
even when a prior operation leaves an empty set. An empty class contributes no
member, while an empty q branch contributes the empty word; both may have zero
maximum width but produce different match results.

## Character-only right operands in finite string-set operations

A separate Core proof admits character-only right operands in a positive,
homogeneous intersection/subtraction chain with a finite left operand. At least
one operand requires the scalar proof and a q member is empty or longer. Later
finite operands remain eligible. Scalar operands include class/property escapes
and q-free classes with ranges, nested algebra, or negation. Scalar-left sets,
mixed scalar/q unions within a right operand, inner string operations, additional
atoms, and repetition remain separate proofs.

Borrowed operand boundaries reuse the checked character compiler. Its new scalar
entry admits flat classes without changing the earlier checked entry domains or
callbacks. All new interval and frame buffers grow fallibly with prior work
charges. Ranges stay compact; the trie retains only first-operand words.

Character-only sets contain one-character words. Filtering therefore visits
active terminal root edges and leaves empty/longer words out of scalar membership,
even when a longer word starts with an admitted character. Matching uses canonical
member equality before each operation, including property folding and complement
order under vi. Hash-map traversal work uses capacity, scalar lookup/folding work
is prepaid, and checked marks reuse the existing operation application. Search
retains the final active-word width, including zero for an empty result. Every
later operand is still proved after an empty intermediate set. Search, clone,
and drop keep their flat representation; default quotas and dependencies are unchanged.

Runtime selects the scalar-right proof after the finite-only operation proof.
It reuses the UnicodeStringSet body, enclosing capture projection, and existing
search-work accounting. Zero-width bounds continue to distinguish an empty set
from a retained empty member. Unsupported operands are still proved after an
empty intermediate result and preserve the host-error completion category.

## Scalar-left string operations and character projection

A separate Core entry proves a positive class whose first operand is q-free and
character-only, followed by intersection or subtraction. The root's result is
therefore a subset of that first character set. Later operands may contain nested
unions, intersections, subtractions, scalar properties, ranges, and q disjunctions.
At least one q member must be empty or longer. Whole-Pattern v/vi validation is
required; string properties, root unions or negation, additional atoms, and
repetition remain separate proofs.

Character projection commutes with set union, intersection, and subtraction.
Every q branch is decoded to its end, but only branches containing one character
contribute intervals. Simple folding preserves character count, so empty and
longer words can be discarded before canonical interval operations without
turning a string into a character. Nested complements retain the normal character
universe; full validation proves that each complemented result contains no strings.
Separate braced surrogate escapes remain two characters and cannot become a
projected supplementary character.

The checked parser has separate no-string, singleton-string, and projection
modes. Earlier entries retain their existing domains and callback sequences.
The projection mode reuses charged, fallible interval/frame storage and never
stores a long word or expands scalar ranges. Scalar-first framing borrows source
and preserves canonical outside word/line gates. Empty projected results remain
consuming classes that never match, including on empty input. Deep parsing,
search, clone, and drop remain iterative, and quotas and dependencies are unchanged.

Runtime selects this proof after the string-set operation proofs and reuses the
UnicodeClass matcher body, capture projection, and charged search dispatch. Every
successful result consumes one complete original input character. Empty projected
sets never produce an empty match, and retained q empty members cannot make a
scalar-left result nullable. String consumers and cloned matchers share the same
immutable interval plan.

## Quantified scalar-left string operations

A separate Core repetition entry compiles one projected scalar-left class with
one existing greedy/lazy quantifier. The class retains its positive q-free first
operand and at least one empty/long q member. Canonical optional outside line/word
gates apply to the entire repetition. Atom-internal wrappers, further atoms,
string properties, and other root set shapes remain separate proofs. Whole-body
captures remain a caller proof.

Character projection proves the atom consumes one complete original character.
An empty q member cannot introduce an empty iteration. An empty projected class
cannot consume, so it only permits a repetition with a zero minimum to finish
empty. Exact decimal bounds reuse the existing nonexpanding count representation
and matching, including huge impossible minima and overflowing maxima. Even a
zero count still requires complete class proof.

Shared framing now has a borrowed-unit entry, and the scalar-left class proof
exposes a matching internal unit entry. The repetition constructor adds no source
copy, new atom representation, search algorithm, dependency, or default quota.
Its checked/singleton-string/scalar-left class selection preserves older entry
domains and callback sequences. Interval/frame growth and deep parsing retain
their checked work/capacity contracts, and search, clone, and drop stay flat.

Runtime selects the quantified proof after the unquantified scalar-left proof.
It reuses the UnicodeRepeatedAtom body, whole-body capture projection and charged
incremental search. Result arrays, named indices, lastIndex and String consumers
retain their existing contracts for consuming and zero-iteration results.

## Static character results of string-set expressions

A separate Core proof accepts a whole validated v/vi class whose static
MayContainStrings result is false and that contains an empty or longer q member.
The checked interval compiler tracks containment in each iterative class frame:
union uses OR, intersection uses AND, and subtraction retains its left operand's
flag, following Static Semantics: MayContainStrings. The Parser uses the same rule;
the compiler adds no crate dependency. A legal complement requires a false operand flag and yields a
character-only set. Both root and nested complements are admitted.

When the final result is proved character-only, character projection can discard
empty/long q members throughout the expression before canonical interval
algebra. This admits string-first intersections and nested unions whose string
members are eliminated by character operands. The analysis remains conservative:
this proof declines a q-only intersection marked true even if its actual result
is empty. Every operand is still decoded and compiled after empty intermediates;
string properties remain deferred even when static containment is false.

The new mode reuses existing q decoding, fold closure, complements, interval
algebra, checked frame growth, and original-input search. Older modes retain
their selection and callback sequences. Canonical optional outside line/word
gates and caller-projected whole-body captures retain their scope. A paired
repetition entry uses the same proof with existing greedy/lazy exact bounds;
projected classes remain consuming character atoms and zero bounds still require
complete proof. Inner repeated wrappers and further atoms remain separate.
Source stays borrowed, all storage stays flat,
and dependencies, allocation boundaries, and default quotas are unchanged.

Runtime selects character-result class and repetition proofs after the earlier
scalar-left proofs. They reuse UnicodeClass/UnicodeRepeatedAtom bodies, enclosing
capture projection and charged search. Empty character sets remain nonnullable
atoms; only zero-iteration repetition yields empty results. Names, indices,
lastIndex, String consumers, and direct-eval completions keep their existing
contracts. Former host Unsupported completions execute finalizers normally.

## Captures inside repeated character-result classes

A separate Core entry proves complete capturing/noncapturing wrappers around one
bare character-result v/vi class, followed by one existing greedy/lazy quantifier.
The caller validates the entire Pattern and normalizes named captures. The leaf
must have a false static MayContainStrings result and an empty/long q member.
Canonical optional outside word/line gates retain whole-repetition scope. Inner
assertions, partial captures, alternatives, additional atoms, string properties,
and conservative string results remain separate proofs, including under zero
or impossible bounds.

Every complete inner capture selects the final original consumed character.
Zero iterations leave those captures undefined, including when the projected
class is empty. Noncapturing wrappers add no capture. The existing repeated-atom
representation stores a capture count and one final-iteration range; captures
and decimal counts never expand into matching state. Complete enclosing captures
remain a caller proof and may cover the entire repeated range instead.

The wrapped constructor shares borrowed framing/wrapper scans with the earlier
singleton-q constructor. Explicit class proof selection retains that entry's
domain, callback sequence, and capture results. Existing checked interval/frame
storage, error categories, flat search/clone/drop, allocation boundaries,
dependencies and resource defaults are unchanged.

Runtime selects the wrapped proof after unwrapped character-result repetition.
The existing UnicodeRepeatedAtom body combines final-iteration inner captures
with complete enclosing captures. Normalized named groups retain their original
keys, including supplementary escaped names. Zero-iteration inner captures stay
undefined in arrays, groups and indices, with named keys present. lastIndex,
String consumers, cloned matchers and collection reuse their existing paths.

## Raw empty alternatives beside character-result classes

A separate Core entry proves one raw empty alternative and one bare
character-result v/vi class branch inside complete ordinary/noncapturing wrappers.
The class must have a false static MayContainStrings result and an empty/long q
member. Both `class|` and `|class` preserve their source order under *, + and ?,
including lazy forms. Canonical optional outside line/word gates retain whole
repetition scope. Named captures must already be normalized. Branch-local
captures/assertions, extra/grouped empty alternatives, braced counts, string
properties, conservative string results, and further atoms remain separate proofs.

The explicit raw branch introduces nullability; projected q empty members cannot
make the character branch nullable. RepeatMatcher's progress guard rejects
optional empty iterations. A required empty plus iteration retains defined empty
inner captures, whereas zero iterations leave them undefined. Successful complete
wrappers share the final selected iteration's original range, including an empty
range. Every class operand is compiled even behind an empty-first alternative.

The constructor shares borrowed edge/quantifier/wrapper scans with the earlier
singleton-q entry and selects the character-result leaf proof. Existing nullable
branch-order state, linear matching, captures, checked interval/frame growth,
allocation boundaries, clone/drop, and old constructor domains/callbacks are
reused. Dependencies and resource defaults are unchanged.

Runtime selects this proof after wrapped character-result repetition. The
existing repeated-atom body combines final-iteration captures with complete
enclosing captures. Required empty captures remain empty strings and zero-count
captures remain undefined in numeric/named results and indices. String consumers,
replacement callbacks, clones, collection, lastIndex and host-abort propagation
reuse their existing paths.

## Zero-minimum braced nullable string classes

Separate Core entries admit braced counts with a zero lower bound around one raw
empty alternative and one bare singleton-q or character-result v/vi class. The
class proofs retain their separate domains. Complete ordinary/noncapturing
wrappers and canonical outside word/line gates retain their existing scope.
Positive lower bounds, simple quantifiers, branch-local captures/assertions,
grouped/extra empty alternatives, and additional atoms remain separate proofs.

Exact zero, padded counts, finite/unbounded upper bounds, upper bounds exceeding
native integers, and lazy forms reuse the existing nullable repetition and
capture representation. All iterations are optional, so RepeatMatcher's progress
check rejects the raw empty branch before captures can escape. Zero iterations
leave inner captures undefined; successful iterations retain the final original
character range. Every class operand is still compiled under exact zero and
behind empty-first alternatives. A projected q empty member never makes the
character branch nullable.

The shared borrowed framing/wrapper scan selects the new count proof explicitly
and precharges its decimal scans. Counts, wrappers and captures never expand
into matching state. Earlier constructors retain their domains, callback traces
and results. Checked class storage, allocation boundaries, flat search/clone/drop,
dependencies and resource defaults are unchanged.

Runtime selects the two braced-count proofs after raw nullable character-result
repetition. The existing repeated-atom body combines final-iteration captures
with complete enclosing captures and preserves undefined zero-iteration entries
in numeric/named arrays and indices. String consumers, replacement callbacks,
lastIndex, clones, collection and host-abort propagation reuse their existing
paths. Former Unsupported exact-zero expressions complete finalizers normally.

## One-minimum open braced nullable string classes

Separate Core entries prove {1,} repetition of one raw empty alternative beside
one bare singleton-q or character-result v/vi class, including padded lower counts
and lazy forms. Complete ordinary/noncapturing wrappers and canonical outside
word/line gates retain their earlier scope. Finite upper bounds, including those
exceeding native integers, other lower bounds, simple quantifiers, branch-local
captures/assertions, grouped/extra empty alternatives and additional atoms remain
separate proofs. Every class operand is compiled even behind an empty-first
alternative which can satisfy the mandatory iteration without consuming input.

The existing nullable matcher preserves the required branch's source priority
and applies RepeatMatcher's progress check to later optional iterations. A
required empty iteration leaves defined empty captures; rejected optional empty
iterations cannot erase them. Successful consuming iterations retain the final
original character range. Existing linear search and capture storage are reused
without count expansion or per-capture matching state.

The borrowed framing/wrapper scan selects this braced-count proof explicitly
and precharges decimal scans. Earlier simple/zero-minimum constructors retain
their domains, exact callback traces and results. Checked class storage,
allocation boundaries, flat clone/drop, dependencies and resource defaults are
unchanged.

Runtime selects the two one-minimum open-count proofs after the zero-minimum
braced proofs. Existing repeated-atom capture projection combines required
defined-empty or final-character inner captures with complete enclosing captures,
including normalized supplementary names. String consumers, replacement
callbacks, lastIndex, clones, collection and host-abort propagation reuse their
existing paths. Former Unsupported {1,} expressions complete finalizers normally.

## Exactly-one braced nullable string classes

Separate Core entries prove {1} and {1,1} repetition of one raw empty alternative
beside a bare singleton-q or character-result v/vi class, including padded and
lazy forms. Complete ordinary/noncapturing wrappers and canonical outside
word/line gates retain their earlier scope. Other counts, simple quantifiers,
branch-local captures/assertions, grouped/extra empty alternatives and additional
atoms remain separate proofs. Every class operand is compiled even when the
empty-first branch can satisfy the only required iteration.

Every iteration is mandatory, so the quantifier's greedy flag cannot reorder
the body's choices. The existing matcher finds consuming and empty candidates
in separate linear passes: the earliest input start wins and source branch order
breaks a tie. Successful complete inner captures retain the selected original
character range or a defined empty range. Existing representation, linear search
and capture projection are reused without per-capture matching state.

The shared borrowed framing/wrapper scan selects this braced-count proof and
precharges decimal scans. Counts never expand into matching state. Earlier
simple, zero-minimum and one-minimum-open constructors retain their domains,
exact work traces and results. Checked class storage, allocation boundaries,
flat clone/drop, dependencies and resource defaults are unchanged. Runtime
selects the two exactly-one proofs after one-minimum open braced repetition.
Existing capture projection combines selected empty/character inner ranges with
complete enclosing captures. Normalized supplementary names, named indices,
String consumers, replacement callbacks, lastIndex, clones, collection and host
aborts reuse their existing paths. Former Unsupported {1}/{1,1} expressions
complete finalizers normally.

## Positive open-count nullable string classes

Separate Core entries prove positive {m,} repetition of a raw empty alternative
beside a bare singleton-q or character-result v/vi class. Padded lower bounds,
lower bounds exceeding native integers, and lazy forms share the existing
nullable search. Complete ordinary/noncapturing wrappers and canonical outside
word/line gates retain their earlier scope. Zero lower bounds, finite upper
bounds, simple quantifiers, branch-local captures/assertions, grouped/extra empty
alternatives and additional atoms remain separate proofs. Every class operand
is compiled even behind mandatory empty-first choices.

Character-first mandatory iterations prefer characters before empty choices.
When fewer characters are consumed than the minimum, the last required empty
iteration defines an empty capture at the whole-match end. This count uses
decoded characters, not UTF-16 width. Empty-first mandatory iterations instead
prefer empty choices; successful later optional characters can define the final
capture. Lazy search retains mandatory source priority before optional stopping,
and optional empty iterations remain rejected by RepeatMatcher's progress check.

Unrepresentable positive minimums exceed every finite input character count.
The existing count representation and linear matcher preserve their required
empty choices and final original ranges without expanding counts or captures
into matching state. The shared borrowed framing/wrapper scan precharges
decimal scans. Earlier constructors retain their domains, exact work traces and
results. Checked class storage, allocation boundaries, flat clone/drop,
dependencies and resource defaults are unchanged.

Runtime selects the positive open-count proofs after exactly-one repetition.
Existing capture projection combines final mandatory empty or original character
ranges with complete enclosing captures, including supplementary names and
indices. String consumers and replacement callbacks retain empty final captures
even when the whole match consumed characters. lastIndex, cloning, collection
and host-abort propagation reuse their existing paths. Former Unsupported {m,}
expressions with larger minimums complete finalizers normally.

## Representable positive exact nullable string-class counts

Separate Core entries prove {m} and {m,m} repetition with a represented bound
greater than one around a raw empty alternative and a bare singleton-q or
character-result v/vi class. Padded bounds and lazy forms preserve complete
ordinary/noncapturing wrappers and canonical outside word/line gates. Other
counts, simple quantifiers, branch-local captures/assertions, grouped/extra empty
alternatives and further atoms remain separate proofs. Every class operand is
compiled even behind an empty-first mandatory choice.

All iterations are mandatory. At each candidate input start, the existing matcher
selects the longest accepted character run for character-first bodies and the
shortest for empty-first bodies, independently of quantifier greediness. Fewer
consumed decoded characters than the exact count leave a final required empty capture for
character-first bodies. Empty-first bodies retain the last selected character
when outside gates require consumption. Original supplementary ranges and
complete enclosing captures keep their separate meanings.

The shared borrowed framing/wrapper scan precharges decimal scans and retains
exact bounds without expanding them. Earlier constructors keep their domains,
work traces and results. Existing linear search, checked class storage,
allocation boundaries, flat clone/drop, dependencies and resource defaults are
unchanged.

Runtime selects the positive exact-count proofs after positive open counts.
Existing capture projection combines final mandatory empty or original character
ranges with complete enclosing captures. Padded and largest represented bounds
keep their original source metadata. Named indices, supplementary escaped names,
String consumers, replacements, lastIndex, cloning, collection and host-abort
propagation retain their existing paths. Former Unsupported {m}/{m,m} expressions
with represented bounds above one complete finalizers normally.

## Unrepresentable exact nullable string-class counts

Separate Core entries prove {m}/{m,m} when the exact bound exceeds native count
storage around a raw empty alternative and a bare singleton-q or character-result
v/vi class. A borrowed decimal comparison trims leading zeros and requires equal
range endpoints; open and unequal overflowing ranges retain separate proofs.
Padded and lazy forms preserve complete wrappers and canonical outside word/line
gates. Other bounds, simple quantifiers, branch-local captures/assertions,
grouped/extra empty alternatives and further atoms remain separate proofs.
Every class operand is compiled behind empty-first choices.

Every possible consumed character is mandatory below this exact overflowing
count. The constructor therefore encodes source branch order in the existing
greedy search field, independently of the source quantifier's greedy flag.
Character-first bodies choose the longest accepted run at a candidate start and
retain a final required empty capture. Empty-first bodies choose the shortest;
when outside gates require characters, the last selected character defines the
inner capture. Original source metadata remains a caller responsibility.

The shared borrowed framing/wrapper scan precharges decimal parsing and exact
equality checks without expanding counts or captures. Existing linear matching,
checked class storage, allocation boundaries and flat clone/drop are reused.
Earlier constructors retain their domains, exact work traces and results.
Dependencies and resource defaults are unchanged.

Runtime selects the overflowing exact-count proofs after represented positive
exact counts. Original padded endpoints and lazy markers stay in source metadata
while internal search encodes mandatory branch order. Existing capture projection
combines final empty or original character ranges with enclosing captures, named
indices and supplementary names. String consumers, replacements, lastIndex,
cloning, collection and host-abort propagation retain their existing paths.
Former Unsupported overflowing exact expressions complete finalizers normally;
unequal overflowing ranges retain Unsupported results.

## Represented positive finite nullable string-class ranges

Separate Core entries prove {m,n} around a raw empty alternative and a bare
singleton-q or character-result v/vi class when both bounds are represented,
the minimum is positive and the maximum is greater. Padded/lazy forms preserve
complete wrappers and canonical outside word/line gates. Other bounds, simple
quantifiers, partial captures, grouped/extra empty alternatives and further atoms
remain separate proofs. Every class operand is compiled behind empty-first
mandatory choices.

The existing matcher preserves mandatory source priority and the exact optional
budget n-m. Greedy empty-first bodies try the optional character budget before
backtracking mandatory empty choices to longer runs. Separate linear passes
compare both phases by input start, so a later first-phase match cannot hide an
earlier candidate found through mandatory backtracking. Character-first and lazy
plans retain their existing mandatory/optional ordering and progress checks.
Required empty or final original character captures reuse the existing projection.

The shared borrowed framing/wrapper scan precharges decimal parsing without
expanding counts or captures. Existing linear search, checked class storage,
allocation boundaries and flat clone/drop are reused. Earlier constructors
retain their domains, exact work traces and results. Dependencies and resource
defaults are unchanged.

Runtime selects the represented finite-range proofs after overflowing exact
counts. Existing capture projection combines final mandatory empty or original
character ranges with complete enclosing captures and named indices. Phase
comparison retains earlier matches found through mandatory backtracking, including
sticky and supplementary starts. String consumers, replacements, source metadata,
lastIndex, cloning, collection and host-abort propagation retain their existing
paths. Former Unsupported represented finite ranges complete finalizers normally;
overflowing unequal ranges retain separate proofs.

## Nullable string-class ranges with overflowing finite maxima

Separate Core entries prove a raw empty alternative and a bare singleton-q or
character-result v/vi class under {m,n}, where m is positive and represented and
n exceeds native storage. Complete ordinary/noncapturing wrappers and canonical
outside word/line gates reuse the existing borrowed proof. Every class operand
is compiled before matching, including empty-first choices and discarded words.
Other bounds and partial capture shapes retain separate proofs.

An overflowing maximum can still leave a small optional budget: native maximum
m and mathematical n=m+1 allow one optional iteration. Borrowed decimal
subtraction preserves n-m when represented; an overflowing difference exceeds
every possible input character count. Empty-first greedy matching retains the
exact optional phase and compares it with mandatory backtracking for the earliest
start. Character-first mandatory priority and lazy stopping reuse existing
linear search and final empty/original character capture projection (22.2.2.3).

The constructor precharges decimal comparison/subtraction in addition to existing
framing and count parsing. Counts never expand into iterations or captures, and
flat clone/drop, prior constructor domains/work traces, allocation boundaries,
dependencies and resource defaults are unchanged.

Runtime selects these two proofs after represented finite ranges. Existing result
projection retains mandatory empty captures and original supplementary character
ranges, including the leading boundary when a Unicode start lies inside a pair.
String consumers, replacements, names/indices, source metadata, lastIndex, clones,
collection and opt-in host-abort propagation reuse existing paths. Former
Unsupported overflowing-maximum expressions complete finalizers normally; ranges
whose lower and upper bounds both overflow retain separate proofs.

## Finite nullable string-class ranges with two overflowing bounds

Separate Core entries prove {m,n} around a raw empty alternative and a bare
singleton-q or character-result v/vi class when both bounds exceed native storage
and are numerically unequal. Borrowed endpoint comparison trims leading zeros;
decimal subtraction retains the exact optional budget n-m when represented.
The existing finite optional-phase state also covers one-unit differences across
arbitrarily long decimal borrows. An overflowing difference exceeds every possible
input character count and needs no additional limit state.

Every consumed character can be mandatory below the overflowing minimum.
Character-first priority therefore chooses descending accepted endpoints
independently of source laziness. Empty-first choices preserve optional progress,
lazy stopping and earliest cross-phase candidates. Existing capture projection
retains final required empty or original character ranges (22.2.2.3). Complete
wrappers and canonical outside word/line gates reuse existing linear search.
Every class operand is compiled before matching, including discarded words and
empty-first alternatives; unknown leaves and partial captures remain separate.

The constructor precharges comparison/subtraction and reuses borrowed decimal
storage, checked class allocation and flat clone/drop. Earlier constructor domains,
exact work traces, dependencies and resource defaults are unchanged.

Runtime selects these two proofs after overflowing-maximum ranges. Existing
capture projection, named indices and String consumers retain mandatory empty
captures, original supplementary character ranges and earliest cross-phase
matches. Original padded/lazy source metadata, lastIndex, cloning, collection and
opt-in host-abort propagation reuse existing paths. Earlier unequal overflowing
range expressions now complete their exact results and finalizers; partial
capture/empty-branch shapes and unknown class leaves retain separate proofs.

## Nullable string classes beside nested noncapturing empty groups

Separate Core entries prove one bare singleton-q or character-result v/vi class
and an empty branch made only of one or more nested noncapturing groups under
*, + or ?, including lazy forms. Complete capturing/noncapturing wrappers and
canonical outside word/line gates retain their existing scope. Sibling empty
sequences, empty choices/quantifiers, branch-local captures/assertions, braced
counts, further atoms and unknown class leaves retain separate proofs.

The constructor walks initial openers once. When empty-first groups belong to
the branch rather than complete repeated wrappers, one closing run recovers
their noncapturing prefix without repeatedly probing each depth. Edge branch
scans then prove the complete empty group and the entire remaining class,
including q payload separators and discarded words. Construction precharges the
additional linear scans and adds no recursion or expanded capture storage.

Existing RepeatMatcher search rejects optional empty iterations before exporting
captures. A required plus iteration keeps defined empty captures; skipped zero
iterations keep inner captures undefined, and consumed iterations retain the
last original character range (22.2.2.3). Branch priority, earliest starts and
whole-repetition gates reuse existing kernels. Prior constructor domains/work
traces, flat clone/drop, dependencies and resource defaults are unchanged.

Runtime selects the grouped-empty proofs after raw nullable count proofs.
Existing result projection preserves defined empty versus undefined inner
captures and original supplementary ranges. Source/flags, names and indices,
String consumers and replacements, lastIndex, clones/collection and opt-in
host-abort propagation reuse existing paths. The former grouped-empty rejection
completes with its exact value and finalizer effects; empty sequences/choices,
quantified empty branches, braced counts and partial captures remain separate.

## Zero-minimum braced nullable string classes beside nested empty groups

Separate Core entries admit {0}, {0,n} and {0,} around a bare singleton-q or
character-result v/vi class and a nested noncapturing empty alternative, including
padded/lazy forms and overflowing finite upper bounds. Empty-branch shape and
count proof are independent constructor options; previous entries retain their
domains and exact work traces. Complete wrappers and canonical outside word/line
gates reuse the existing borrowed group scan and linear search.

RepeatMatcher rejects optional empty iterations before exporting captures;
zero selected iterations leave inner captures undefined while consumed iterations
retain the last original character range (22.2.2.3). Exact zero bounds still
compile every class operand, including unknown or string-valued leaves. Positive
minima, empty sequences/choices/quantifiers, partial captures and further atoms
retain separate proofs.

Construction precharges the grouped-branch scans and decimal parsing separately.
Counts and nested groups do not expand matching state. Checked class allocation,
flat clone/drop, dependencies and resource defaults are unchanged.

Runtime selects the grouped-empty zero-minimum proofs after simple grouped-empty
counts. Existing result projection and String consumers retain undefined inner
captures after zero iterations, defined enclosing empty captures, original
supplementary ranges and named-key presence. Original source/flags, lastIndex,
clones/collection and opt-in host-abort propagation reuse existing paths. The
earlier exact-zero grouped-empty rejection now completes with its exact value and
finalizer effects; positive counts and other empty-branch shapes remain separate.

## One-minimum open string-class counts beside grouped empty alternatives

Separate Core entries admit {1,} around a bare singleton-q or character-result
v/vi class and a nested noncapturing empty alternative, including padded/lazy
forms. Complete wrappers and canonical outside word/line gates reuse the
independent empty-shape and count proofs. Exact/finite and other positive minima,
empty sequences/choices/quantifiers, partial captures and further atoms remain
separate proofs; every class leaf is compiled behind empty-first choices.

RepeatMatcher retains a mandatory empty iteration's defined empty captures.
Optional empty iterations fail the progress guard before overwriting a consumed
iteration's original character capture (22.2.2.3). Source branch priority,
greedy/lazy stopping, UTF-16 boundaries and outside gates reuse existing linear
search. Constructor work precharges grouped-branch and decimal scans separately;
counts and groups never expand matching state. Earlier domains/work traces,
checked class storage, flat clone/drop, dependencies and resource defaults are
unchanged.

Runtime selects the grouped-empty {1,} proofs after grouped-empty zero-minimum
counts. Existing capture projection and String consumers preserve mandatory empty
versus skipped undefined captures, original supplementary ranges, names and
indices, source metadata, lastIndex and clone/collection behavior. Opt-in host
aborts retain atomic effects. Earlier {1,} grouped-empty rejections complete with
their exact values and finalizer effects; exact/finite counts and other branch
shapes remain separate proofs.

## Exactly-one string-class counts beside grouped empty alternatives

Separate Core entries admit {1} and {1,1} around a bare singleton-q or
character-result v/vi class and a nested noncapturing empty alternative,
including padded/lazy forms. The one mandatory iteration follows source branch
priority independently of quantifier greediness. Existing linear consuming and
empty candidate passes retain earliest starts, outside word/line gates and
defined empty or original character captures (22.2.2.3).

Complete wrappers, borrowed group recovery, checked class storage, work callbacks
and flat clone/drop are reused. Every class leaf is compiled behind empty-first
choices. Other counts, partial captures, empty sequences/choices/quantifiers and
further atoms retain separate proofs. Counts do not expand matching state and
earlier constructor domains/work traces, dependencies and resource defaults are
unchanged.

Runtime selects the grouped-empty exact-one proofs after grouped-empty {1,}
counts. Existing capture projection and String consumers preserve mandatory
source priority, defined empty captures and original supplementary ranges.
Padded/lazy source metadata, names/indices, lastIndex, clones/collection and
opt-in host-abort propagation reuse existing paths. Earlier exact-one rejections
complete with exact values and finalizer effects; larger counts and other
empty-branch/partial capture shapes retain separate proofs.

## Positive open string-class counts beside grouped empty alternatives

Separate Core entries admit {m,} with positive represented or overflowing m
around a bare singleton-q or character-result v/vi class and a nested
noncapturing empty alternative, including padded/lazy forms. Complete wrappers
and canonical outside word/line gates reuse the existing borrowed proof and
linear search. Every class leaf is compiled behind empty-first choices.

With character-first mandatory choices, too few consumed characters leave a final
required empty iteration and defined empty captures. Empty-first choices retain
the last consumed original character, or a required empty capture when nothing
consumes. Greedy/lazy priority and original UTF-16 boundaries reuse existing
RepeatMatcher kernels (22.2.2.3). An overflowing minimum exceeds every finite
input character count and never expands matching state.

Exact/finite counts, partial captures, other empty-branch shapes and further atoms
retain separate proofs. Earlier constructor domains/work traces, checked class
storage, flat clone/drop, dependencies and resource defaults are unchanged.

Runtime selects the grouped-empty positive open proofs after grouped-empty
exact-one counts. Existing result projection retains final required empty
captures or original supplementary character ranges, including when the minimum
overflows native storage. Names/indices, padded/lazy source, String consumers and
replacements, lastIndex, clones/collection and opt-in host-abort propagation reuse
existing paths. Earlier grouped {2,} rejections complete with exact values and
finalizer effects; finite/exact counts and other branch shapes remain separate.

## Represented positive exact counts beside grouped empty alternatives

Separate Core entries admit {m} and {m,m} with represented m greater than one
around a bare singleton-q or character-result v/vi class and a nested
noncapturing empty alternative, including padded/lazy forms. All iterations are
mandatory, so source branch priority determines accepted endpoints independently
of quantifier greediness. Character-first choices retain a final required empty
capture when fewer than m characters consume; empty-first choices retain a
required empty or last original character capture (22.2.2.3).

Existing linear search retains earliest starts, canonical outside word/line gates
and original supplementary ranges, including candidate trimming at actual input
ends. Every class leaf is compiled behind empty-first choices. Complete wrappers,
borrowed group recovery, work callbacks, checked class storage and flat clone/drop
are reused. Other counts and partial/other-empty shapes retain separate proofs.
Counts never expand matching state and earlier domains/work traces, dependencies
and resource defaults are unchanged.

Runtime selects the grouped-empty represented exact proofs after grouped-empty
positive open counts. Existing capture projection and String consumers retain
mandatory source order, final required empty captures and original supplementary
ranges, including actual-end trimming and largest represented counts. Padded/lazy
source, names/indices, lastIndex, clones/collection and opt-in host-abort
propagation reuse existing paths. Earlier exact-count rejections complete with
exact values and finalizer effects; unequal finite and overflowing exact counts
and other branch shapes remain separate proofs.

## Overflowing exact counts beside grouped empty alternatives

Separate Core entries admit {m} and {m,m} with overflowing m around a bare
singleton-q or character-result v/vi class and a nested noncapturing empty
alternative, including padded/lazy forms. Borrowed decimal comparison trims
leading zeros and proves range endpoints numerically equal. Unequal finite
ranges and open counts retain separate proofs.

Every possible consumed character is mandatory below this count. Source branch
priority determines endpoints independently of greediness, retaining a final
required empty or original character capture (22.2.2.3). Existing linear search
retains earliest starts, canonical outside word/line gates and original UTF-16
ranges. Every class leaf is compiled behind empty-first choices.

Complete wrappers, borrowed group recovery, comparison work callbacks, checked
class storage and flat clone/drop are reused. Counts never expand matching state.
Earlier constructor domains/work traces, dependencies and resource defaults are
unchanged.

Runtime selects the grouped-empty overflowing exact proofs after represented
exact counts. Existing result projection and String consumers retain numeric
endpoint equality, mandatory source priority and required final empty captures
or original supplementary ranges. Source/flags, names and indices, lastIndex,
clones/collection and opt-in host-abort propagation reuse existing paths. The
earlier overflowing exact rejection completes with its exact value and finalizer
effects; unequal overflowing ranges and other branch shapes remain separate.

## Represented finite ranges beside grouped empty alternatives

Separate Core entries admit {m,n} with represented positive m and greater
represented n around a bare singleton-q or character-result v/vi class and a
nested noncapturing empty alternative, including padded/lazy forms. Complete
wrappers and canonical outside word/line gates reuse the borrowed proof and
existing linear search. Every class leaf is compiled behind empty-first choices.

Mandatory source priority and the exact optional budget n-m retain the existing
greedy/lazy phases. Earliest-start comparison allows mandatory backtracking to
expose a candidate before the preferred optional phase's candidate. Final required
empty or last original character captures retain decoded character counts and
original UTF-16 ranges, including supplementary actual-end trimming (22.2.2.3).

Exact/open and overflowing bounds, partial captures, other empty-branch shapes
and further atoms retain separate proofs. Counts and captures never expand
matching state. Earlier domains/work traces, checked class storage, flat
clone/drop, dependencies and resource defaults are unchanged.

Runtime selects the grouped-empty represented finite proofs after overflowing
exact counts. Existing capture projection and String consumers retain exact
optional budgets, earliest cross-phase starts, required empty captures and
original supplementary ranges. Padded/lazy source, names/indices, lastIndex,
clones/collection and opt-in host-abort propagation reuse existing paths. Earlier
finite-range rejections complete with exact values and finalizer effects;
overflowing upper bounds and other branch shapes retain separate proofs.

## Overflowing upper bounds beside grouped empty alternatives

Separate Core constructors prove a represented positive lower bound and a finite
upper bound beyond native storage around a singleton-q or character-result v/vi
class beside nested noncapturing empty groups. The same complete wrapper, class
and outside-gate proofs apply. Borrowed decimal subtraction preserves an exact
optional budget when the difference fits native storage, including a difference
of one across the native boundary. Larger differences cannot constrain the
available input. Mandatory branch priority, greedy/lazy phases, earliest starts
and required empty/original character captures follow RepeatMatcher (22.2.2.3).

The constructors reuse the existing flat matcher and charge the nested-empty
scan and decimal proof independently. Counts, groups and discarded strings do not
expand state. Unknown class leaves remain unproved behind empty-first choices;
both-overflowing finite bounds, partial captures and other empty shapes retain
separate proofs. Prior constructor domains and work traces remain unchanged.
Runtime selects these proofs after grouped-empty represented finite ranges.
Existing capture projection and consumers preserve exact optional budgets,
mandatory priority, earliest starts and original UTF-16 ranges. Earlier upper-bound
rejections complete with their expected values and finalizer effects. Names,
indices, lastIndex, clones, collection and opt-in host aborts reuse existing paths.

## Two overflowing finite bounds beside grouped empty alternatives

Separate Core constructors prove numerically unequal finite bounds that both
exceed native storage around a bare singleton-q or character-result v/vi class
beside nested effect-free noncapturing empty groups. Borrowed decimal subtraction
retains exact optional budgets, including a difference of one after a long borrow
through zeros. Every consumed character can remain mandatory: character-first
source priority is independent of laziness, while empty-first choices preserve
optional progress and earliest cross-phase starts (RepeatMatcher, 22.2.2.3).

Padded endpoints, outside word/line gates and required empty/original character
captures reuse the existing flat matcher. Every class leaf must be proved behind
empty-first choices. Counts, groups and discarded words never expand state.
Earlier constructor domains/work traces, class storage and resource defaults are
unchanged. Partial captures and other empty shapes retain separate proofs.
Runtime selects these proofs after grouped-empty overflowing upper bounds.
Capture projection and String consumers retain exact optional budgets, mandatory
priority, earliest starts, required empty captures and original supplementary
ranges. Former unequal-range rejections complete with exact results and eval
finalizer effects. Padded/lazy source, names/indices, lastIndex, clones, collection
and opt-in host aborts reuse existing paths.

## Noncapturing empty sequences beside string classes

Separate Core constructors prove a bare singleton-q or character-result v/vi
class beside a nonempty sequence of effect-free noncapturing empty groups under
*, + or ?, with an optional lazy marker. The empty sequence may contain arbitrary
nesting and siblings. Its complete iterative proof rejects captures, assertions,
quantifiers and choices. The whole class is compiled even behind empty-first
choices; unknown leaves remain unsupported. Mandatory branch priority, optional
progress, required empty/undefined captures, original character ranges and
canonical outside word/line gates reuse RepeatMatcher (22.2.2.3).

Initial openers can belong to an empty-first branch. A bare class ends in ], so
trailing body closes identify the complete wrappers in that case. The compiler
recovers discarded openers once and proves they are noncapturing. Both branch
scans and the empty proof remain linear; deeply nested groups and capture slots
never expand matching state or native recursion. Earlier constructor domains and
work traces are unchanged. Braced counts, quantified/choice empty groups and
partial captures retain separate proofs.

Runtime selects the empty-sequence proofs after grouped-empty finite ranges.
Existing capture projection and String consumers preserve required empty versus
undefined captures, original supplementary/lone-surrogate ranges, names/indices,
lastIndex, replacement values, clones and collection. Earlier sibling-sequence
rejections complete with exact values and eval finalizers. Construction and
matching host aborts preserve atomic effects under opt-in limits.

## Braced counts beside noncapturing empty sequences

Separate Core constructors combine the empty-sequence proof with every braced
count family. They classify represented zero, exactly-one, positive exact,
positive open and finite ranges from parsed bounds. Open overflowing minima
remain open; finite overflowing endpoints use significant decimal digits to
separate exact equality from unequal ranges. Existing count proofs then retain
exact optional budgets, including a difference of one across the native boundary
or after a long decimal borrow. No count is expanded.

The same complete class and effect-free empty-sequence proofs apply to every
family. Mandatory branch priority, optional progress, earliest cross-phase starts,
required empty/undefined captures and original character ranges reuse
RepeatMatcher (22.2.2.3). Endpoint classification has a separate linear work
charge; prior constructor domains and callback traces are unchanged. Simple
quantifiers, quantified/choice empty groups and partial captures retain separate
entries or proofs.

Runtime selects the counted empty-sequence constructors after simple sequence
quantifiers. Capture projection and String consumers preserve every count
family's required empty/undefined and original character captures, optional
budgets, earliest starts, named indices and lastIndex. Former braced sequence
rejections complete with exact results and eval finalizers. Source metadata,
clones, collection and opt-in host aborts reuse existing paths.

## Quantified empty sequences beside string classes

Separate Core constructors extend the complete empty-sequence proof to validated
greedy/lazy quantifiers on effect-free noncapturing empty groups. The groups may
nest, have siblings and contain overflowing counts. Since they contribute no
captures, assertions or input consumption, their repetition always has the same
empty result; the proof does not expand required or optional iterations
(RepeatMatcher, 22.2.2.3). Simple outer quantifiers and every braced outer count
family have separate singleton-q and character-result v/vi entries.

The iterative empty proof and one-pass wrapper recovery reuse existing helpers.
Every class leaf must still be compiled behind empty-first choices. Existing
matching state retains mandatory priority, optional progress, exact optional
budgets, earliest starts, original character ranges and outside word/line gates.
Quantified-empty recognition has its own conservative linear work reservation;
prior constructor domains and callback traces are unchanged. Empty choices,
capturing empty groups, assertions and partial captures retain separate proofs.

Runtime selects the simple and counted quantified-empty constructors after the
unquantified sequence entries. Existing capture projection and String consumers
preserve all outer-count families, required empty/undefined captures, exact
optional budgets, earliest starts, original UTF-16 ranges, names/indices,
lastIndex, clones and collection. Huge inner empty counts remain effect-free and
flat. Former quantified-empty rejections complete with exact values and eval
finalizers; opt-in host aborts preserve atomic state.

## Effect-free empty choices beside string classes

Separate Core constructors prove a singleton-q or character-result v/vi class
beside a nonempty sequence of effect-free noncapturing empty groups that may
contain choices, nesting, siblings and validated greedy/lazy or huge counts.
Every alternative must be empty and contribute no captures or assertions, so
inner choice priority cannot change its empty result (RepeatMatcher, 22.2.2.3).
Simple and every braced outer count family reuse existing matching state.

Balanced scans from each edge locate a depth-zero separator. Only an edge that
passes the complete empty proof supplies a delimiter; parentheses and bars in
the opposite class payload cannot supply an accepted empty edge. The whole
remaining class is compiled independently, including escaped parentheses in q
payloads and unknown leaves behind empty-first choices. One-pass wrapper recovery
preserves capture slots. Linear work charges cover edge/depth tracking and the
empty proof without expanding counts or groups. Mandatory priority, progress,
optional budgets, earliest starts, original ranges and outside word/line gates
remain unchanged. Prior constructor domains/work traces remain fixed; consuming,
asserted and capturing empty branches retain separate proofs.

Runtime selects simple and counted empty-choice constructors after quantified
empty sequences. Existing capture projection and String consumers retain all
outer-count families, exact optional budgets, earliest starts, required empty or
undefined captures, original UTF-16 ranges, names/indices, source metadata and
lastIndex. Former pure-choice rejections complete with exact results and eval
finalizers; capturing or effectful choices retain separate proofs. Huge inner
counts, clones, collection and opt-in host aborts reuse existing flat paths.

## Captures confined to a nullable class's consuming branch

A separate Core matcher proves complete capturing/noncapturing wrappers around
one nullable singleton-q or character-result v/vi class with a raw empty
alternative. Groups confined to the consuming branch are tracked independently
from groups enclosing the complete atom. The flat parser records each original
capture in source order and skips whole validated classes, including q payload
punctuation. Captured empty branches, extra alternatives/atoms, inner assertions
or quantifiers retain separate proofs; enclosing whole-match groups belong to
the caller's existing prefix projection.

A canonical complete-atom capture feeds the existing simple/counted matcher,
retaining original class and count spelling and outside word/line gates. Its last
iteration is a positive-width original character or a required empty selection.
Complete-atom groups receive either range; character-only groups receive a range
only for the consuming selection. No iterations leaves every inner capture
undefined. This implements RepeatMatcher's clearing of all atom captures before
each iteration, including a required final empty after earlier characters
(22.2.2.3). The immutable role table and parser stack are source-sized and flat;
matching adds no input-sized capture storage or native recursion. Prior matcher
layouts, domains and work traces remain unchanged.

Runtime selects this matcher after existing nullable empty-choice proofs. The
synthetic iteration marker stays private: whole-match enclosures retain their
prefix ranges, and each original atom capture uses its source-order role.
Undefined character-only slots remain present in named groups and indices.
RegExpBuiltinExec and String consumers preserve original UTF-16 text, source and
flags, lastIndex, replacement callback arguments and clone/collection behavior.
Opt-in constructor and search aborts preserve atomic state through existing
work callbacks. Other empty-branch shapes retain separate capture proofs.

## Captures confined to a nullable class's empty branch

A separate Core constructor adds complete capturing/noncapturing empty groups on
the alternative opposite one singleton-q or character-result v/vi class. The
empty branch may contain nested and sibling groups but no inner quantifiers,
choices, assertions or consuming terms. At least one empty-only capture is
required; character-only captures are optional. Empty groups beside the class
retain a separate proof because their boundary depends on placement.

A flat group-span table resolves empty-first groups after the class is known.
Every group must enclose the class or lie wholly on the empty side of the sole
separator. Source-order roles distinguish complete iteration, character-only and
empty-only captures. The existing canonical marker selects the last branch:
empty-only groups receive its empty point and remain undefined after consuming
iterations; character-only groups receive only positive-width ranges. All atom
captures remain undefined when no iteration occurs. Required final empties clear
prior characters and define empty groups as RepeatMatcher requires (22.2.2.3).

Simple and every braced outer count, optional budgets, outside word/line gates,
UTF-16 ranges and source spellings reuse existing matching. Linear construction
charges cover the extra span walk; captures, clone and drop remain flat and
source-sized. Prior constructor domains and exact work traces remain unchanged.
Runtime selects this proof after raw-empty character-only captures. Existing
projection keeps whole-match enclosures separate and preserves source-order
named and unnamed slots for complete iterations and either branch. String
consumers, callback arguments, original source/flags and UTF-16 values, indices,
lastIndex, clones and collection reuse the same ranges. Required empty-only
groups and undefined character-only groups remain distinct through finalizers.
Opt-in constructor/search aborts retain atomic state under unchanged limits.

## Character captures beside noncapturing empty sequences

A separate Core constructor extends character-only captures to a nonempty
sequence of unquantified noncapturing empty groups on the opposite alternative.
Nesting and siblings contribute no captures or effects, so the existing final
iteration marker and character-capture projection apply unchanged
(RepeatMatcher, 22.2.2.3). The flat group-span proof records an actual grouped
empty side, requires a character-only capture and rejects any empty-side capture.
Groups beside the character, choices, inner quantifiers/assertions and unknown
class leaves retain separate proofs.

Simple and every braced outer count, outside word/line gates, mandatory priority,
optional budgets, original UTF-16 ranges and required empty clearing reuse the
same matcher. Construction precharges cover the span walk and role table;
source-sized storage, clone and drop remain flat. Prior constructors keep their
domains and exact work traces. Runtime selects this constructor after the raw-empty and empty-capture proofs.
Existing projection preserves source-order character slots, undefined results
after required final empties, enclosing captures and named/indices aliases.
RegExp and String consumers retain original text, source/flags, lastIndex,
clones and collection. Former grouped-empty rejections complete their exact
results and eval finalizers; opt-in host aborts preserve atomic state.

## Character captures beside quantified noncapturing empty groups

A separate Core constructor proves quantified nested/sibling noncapturing empty
groups opposite one class with character-only captures. At least one empty-side
group carries a validated simple or braced greedy/lazy quantifier. Such a group
has no captures, assertions, choices or consuming terms, so its count cannot
change the branch's empty result (RepeatMatcher, 22.2.2.3).

The flat parser scans each count token forward, checks it through the existing
borrowed quantifier parser, and records which group carries it. Every quantified
group must lie wholly on the empty side; quantified character wrappers and
empty-side captures retain separate proofs. The canonical empty marker drops
these effect-free inner counts and preserves original class/outer-count spelling,
outside gates and source-order capture roles. Earlier outer matcher semantics
therefore retain mandatory priority, progress, optional budgets, required empty
clearing and original ranges. Linear precharges cover count scans and span walks
without expanding counts; clone/drop and storage remain flat and source-sized.
Earlier constructors preserve their domains and work traces. Runtime selects this proof after unquantified noncapturing empty sequences.
Existing projection and String consumers preserve character-only participation,
required final empty clearing, source-order names/indices, original source and
UTF-16 values, lastIndex, clones and collection. Former quantified-empty
rejections complete their exact results and eval finalizers; opt-in host aborts
preserve atomic state. Huge inner counts remain effect-free and flat.

## Character captures beside effect-free empty choices

A separate Core constructor proves character-only captures opposite noncapturing
empty choices with nesting, siblings and validated simple/braced counts. Every
alternative is empty and contributes no captures or assertions, so inner branch
priority cannot change the empty selection (RepeatMatcher, 22.2.2.3).

The parser marks active group ancestors when it skips the sole complete class.
A bar owned by a marked group is the consuming-versus-empty separator; exactly
one such bar is permitted. All remaining bars must lie wholly on the empty side.
Class payload punctuation never enters that scan. Group spans still require
transparent unquantified character wrappers, and empty-side captures/effects
retain separate proofs. Linear precharges cover source-sized owner markers,
count scans and group walks. Canonical projection retains existing outer count
priority, progress, optional budgets, required clearing, original ranges and
outside gates without expanding inner choices or counts. Prior constructor
domains/work remain unchanged; clone/drop stay flat. Runtime selects this constructor after quantified noncapturing empty sequences.
Projection preserves source-order character groups and aliases, undefined slots
after required final empties, original UTF-16 text/indices and enclosing ranges.
RegExp and String consumers retain source metadata, lastIndex, clones and
collection. Former pure-choice rejections complete their exact results and eval
finalizers; huge inner counts and opt-in host aborts stay flat and atomic.

## Captures in quantified empty branches and choices

A separate Core constructor proves captured empty branches containing nested or
sibling quantifiers and choices opposite one character class. At least one
empty-only capture and one empty-side quantifier/choice are required. Every
empty alternative contains only groups and validated counts; assertions,
references, consuming terms and character-side empty groups/quantifiers retain
separate proofs.

Within the empty side, a zero-minimum empty group cannot pass RepeatMatcher's
progress check, so it and its descendants never participate. Required empty
group iterations select the first empty alternative. Every alternative has the
same input point and the continuation reads only input gates, so later choices
cannot turn a failed continuation into a successful capture profile (22.2.2.3).

Flat group-end and capture-start tables resolve source intervals disabled by
zero-minimum groups and later empty alternatives. One ordered sweep merges
these intervals and marks their capture roles permanently undefined. Other
empty-only roles receive the final empty iteration point; character-only roles
receive only positive-width ranges. Complete iterations and enclosing captures
retain their separate projection. All atom captures clear before each outer
iteration, including required final empties after earlier characters.

Linear precharges cover owner markers, count scans, interval tables and sweeps.
Counts and choices never expand, clone/drop remain flat and source-sized, and
matching adds no input-sized capture history. Existing outer counts, mandatory
priority, optional budgets, gates and original UTF-16 ranges reuse the same
matcher. Prior constructors retain their domains and exact work traces. Runtime selects this constructor after capture-free empty choices. Existing
projection preserves required empty parents, undefined optional/later children,
source-order names/indices and enclosing ranges. String consumers and callbacks
retain every lexical capture slot even when all branch captures are undefined.
Original UTF-16 text, source metadata, lastIndex, clones and collection remain
shared. Huge inner counts and opt-in constructor/search aborts stay flat and
atomic. Former captured-effect rejections complete exact results and finalizers.

## Ownership of nullable empty captures

An empty capture can use the empty-branch role only when its group lies inside
the group owning the class/empty separator. A sibling outside that owner executes
independently of the chosen alternative. For example, the capture in
(?:(?:class|)())+ is defined after consuming characters and denotes the last
iteration's end point; treating it as empty-branch-only would incorrectly leave
it undefined (22.2.2.3).

The parser records the separator's owner and resolves its closing boundary from
the flat group table. Captured empty groups outside those bounds retain a
separate point-projection proof. Capture-free empty siblings remain transparent,
and nested wrappers around the complete alternative keep their iteration roles.
This guard adds one linear span walk under existing construction charges and
preserves valid constructor work traces, counts, matching and source metadata.

## Empty point captures beside nullable alternatives

A separate Core constructor proves captured empty siblings before or after the
class or the group owning its nullable alternative. At least one such boundary
capture is required. Empty terms may contain nested groups, counts and choices,
but no assertions, references or consuming terms; class-containing wrappers
remain unquantified and exactly one alternative separates class and empty sides.

The owner bounds distinguish unconditional siblings from siblings in the
character branch. Unconditional captures use the selected iteration's start or
end point on either branch; character-side empty groups use those points only
when the class consumes. Empty-branch groups retain their existing empty role.
Optional groups and later epsilon alternatives still receive the permanently
undefined role through the flat interval sweep (22.2.2.3).

The synthetic final-iteration marker supplies both endpoints even when there are
no class captures. Thus a prefix before the nested alternative denotes the last
iteration's start, and a suffix after it denotes the last iteration's end, rather
than a whole-run boundary inferred from capture position. Zero outer iterations
leave all inner groups undefined. Existing outer counts, optional budgets,
mandatory branch priority, line/word gates and original UTF-16 ranges reuse the
same matcher. Linear precharges cover owner, count and interval scans; matching
adds no capture history and clone/drop remain flat and source-sized. Earlier
constructor domains and work traces remain unchanged. Runtime selects this proof
after conditional empty-capture effects. Projection
keeps source-order named and unnamed slots, before/after UTF-16 points and
character-only participation through RegExp and String consumers, callbacks,
clones and collection. The nested-alternative suffix reproducer now returns its
correct empty value and end-point indices. Original metadata and lastIndex
remain shared; opt-in constructor/search aborts stay flat and atomic.

## Global intrinsic metadata and enumeration

The edition-17 global object materializes Promise, Proxy, SharedArrayBuffer,
WeakRef, FinalizationRegistry, and Atomics with their standard property attributes
and retained intrinsic identities. SharedArrayBuffer is the host's selected
optional exposure. Constructors, prototypes, methods, tags, species accessors,
and function name/length properties follow their specification inventories.
Proxy has no prototype property; Atomics is a namespace, not a function.

Promise and SharedArrayBuffer retain
explicit Unsupported results for their deferred bodies after ordinary argument
evaluation and the specified call/new prefix checks. Proxy algorithms are
described below; WeakRef and FinalizationRegistry lifetimes are described above.
Ordinary-buffer Atomics algorithms are described in the following section.
The two species
accessors return their original receiver. The
metadata adds 45 rooted intrinsic objects; global mutation or deletion cannot
unroot them.

The complete global own-key set supports reflection, enumeration, copying, and
integrity operations through ordinary object behavior. Standard bindings can be
replaced, shadowed through prototypes, or deleted. Deleted names follow ordinary
lookup and typeof rules. Lexical bindings remain separate from global properties.
Configured host reservations still report explicit host gaps and install no
properties. Names outside the selected edition receive no implicit reservation.
The incomplete Array constructor retains its separate fromAsync barrier.

## Atomics on ordinary buffers

Atomics implements the ordinary ArrayBuffer paths of 25.4 for all eight
unclamped integer and BigInt TypedArray kinds. ValidateIntegerTypedArray and
ValidateAtomicAccess require real internal slots, capture the original length,
and convert the index before converting values. RevalidateAtomicAccess checks
the live buffer only after every operand conversion, including both operands of
compareExchange. Detached or out-of-bounds views throw TypeError; a lost index
in an otherwise valid tracking view throws RangeError. Public properties,
prototype inheritance, and Proxy wrappers cannot substitute for internal slots.

Load, store, exchange, compareExchange, add, sub, and, or, and xor share the
existing native-endian element codecs and checked sparse-buffer access. Modular
fixed-width arithmetic preserves all 64 BigInt bits. Read-modify-write methods
return the old element; store returns ToIntegerOrInfinity or ToBigInt before
element-width truncation, without reading or decoding the overwritten element.
No JavaScript executes between the final buffer read and write. Optional work,
BigInt, and backing-page allocation failures remain host failures.

Notify accepts Int32Array and BigInt64Array, converts index and count, and returns
positive zero for ordinary buffers without post-conversion revalidation. Wait
and waitAsync reject those buffers before index, value, or timeout conversion.
IsLockFree applies ToIntegerOrInfinity once and uses fixed native target flags
for widths one, two, and eight; width four always returns true. Shared buffers,
wait queues, agents, and their memory-model events remain deferred.

## Private Proxy state and dispatch boundaries

Proxy records retain a checked target/handler pair and the target's callable and
constructible capabilities (10.5 and 28.2). A single optional pair represents
revocation, so the two slots cannot be partially cleared. Revoker functions trace
their captured Proxy, and collection follows live target/handler handles without
introducing physical Symbol cells. Clearing the capture releases those edges.

Revocation validates the handles and charges optional work before changing any
state. The capture and pair are then cleared without allocation, callbacks, or
fallible operations; repeated revocation is inert. Cached call/construct
capabilities survive revocation and do not copy Array or other target brands.

Ordinary property operations stop at direct or inherited Proxy records and
require Realm dispatch. Revoked records throw TypeError; live records execute
the trap algorithms described below. Existing
read-only, accessor, TypedArray, and Array-length decisions keep their required
ordering. Ordinary prototype-cycle traversal stops at a Proxy candidate.
Public construction, revocation, and trap dispatch use these checked records.

## Proxy exotic objects

Proxy construction and Proxy.revocable implement 28.2 over the checked storage
and revocation foundation. All thirteen internal methods implement the Proxy
algorithms in 10.5, including missing-trap forwarding, trap receiver/argument
ordering, and target invariants checked after the trap returns. Target and handler
are captured before trap lookup; later invariant queries inspect live target
state. Descriptor conversion preserves HasProperty/Get ordering, and ownKeys
finishes the array-like key list before checking duplicates and target keys.

Object-method queries use explicit continuations. Missing call and construct
traps forward iteratively, preserving this, arguments, and newTarget across long
Proxy chains. Actual JavaScript callbacks still use the existing native-stack
safety boundary. Argument and receiver copies charge opted-in work before cloning
large values; property-key lists do not inherit a function argument-count limit.

Callability and constructibility remain cached after revocation. Array detection
unwraps Proxy targets and rejects revoked records where required by Array, JSON,
and Object algorithms; direct built-in brand checks retain their own rules.
Reflection, integrity operations, binding, prototype traversal, instanceof, and
super property access use Proxy internal methods at their specified boundaries.

Constructor prototype fallback validates Proxy and bound-function chains after a
non-object prototype result. Revoked records throw before allocation; object-valued
prototypes skip the realm query. Array species selection validates a constructor
before reading @@species. The traversal uses checked internal slots and an
iterative loop without observable traps or copying bound arguments. The current
implementation exposes one realm; foreign-realm intrinsic normalization remains
deferred.
Resource quotas remain opt-in, and the initial realm retains the same 549
intrinsic objects.


## Empty point captures around consuming class repetitions

A separate Core matcher proves a repeated class with captured empty sequences
before or after it. Exactly one class consumes on every selected iteration;
class-containing alternatives and inner counts remain outside this proof. The
same validated singleton-q and static character-result class checks apply even
under zero or impossible counts. At least one original empty capture is required.

The flat group and interval tables retain character wrappers, before/after
points and optional/later-alternative undefined slots. A synthetic complete
capture around the class records the last consuming iteration. Canonicalization
retains the original outer count and outside word/line gates without introducing
an empty branch or expanding numeric bounds. Zero iterations leave all inner
slots undefined; impossible positive counts fail through the existing consuming
repetition matcher (22.2.2.3). Construction, clone and drop remain source-sized,
and matching retains no input-sized capture history. Earlier nullable
constructors retain their domains and callback traces. Runtime selects this proof
after the nullable capture constructors. Source-order projection retains named
aliases, String consumer captures, callback arguments and original metadata.
Enclosing whole-match captures remain independent; opted-in aborts preserve
lastIndex and finalizers through the shared execution path.


## Empty point captures around a single consuming class

A separate Core entry point proves captured empty sequences around one class
without an outer quantifier. Empty prefixes/suffixes need no complete enclosing
wrapper. The constructor extracts only canonical outside line/word gates, wraps
the borrowed body in a noncapturing group and delegates an exact single
iteration to the existing consuming capture proof (22.2.2 and 22.2.2.3).

The synthetic wrapper adds no lexical captures. Original character captures and
empty before/after points retain actual UTF-16 ranges; optional groups and later
empty alternatives remain undefined. No nullable alternative is introduced.
Class-containing choices/counts, references, internal assertions and additional
consuming terms still decline. Strict class validation proves every leaf even
when its character result is empty. Source copies are precharged before
allocation, all group tables remain flat, and numeric counts never expand.
Earlier repeated/nullable entry points retain their domains and work traces.
Runtime selects the single-iteration proof after repeated consuming captures and
uses the same source-order projection, metadata and RegExp/String execution
paths. No new result representation or capture history is required.


## Empty point captures around a single nullable class group

A separate Core entry point applies exactly one required iteration to a nullable
class group with captured empty siblings. Complete enclosing wrappers are
optional. Character-side points remain conditional, outside siblings retain
unconditional start/end points, and optional/later-alternative groups stay
undefined. Required first-branch selection, UTF-16 projection and strict
character-class validation reuse the established nullable capture proof.

A flat precharged scan declines top-level alternatives before outside assertions
are extracted. In an unwrapped disjunction, a leading/trailing assertion belongs
to only one branch; moving it outside a synthetic wrapper would change its scope
(22.2.2). Grouped alternatives retain their original whole-pattern gates. The
class-versus-empty separator therefore belongs to an explicit original group.
Source framing/copies share the existing exact-single-iteration helper without
changing earlier API work traces. Counts, clone/drop and role masks remain flat
and source-sized; matching adds no capture history. Runtime selects this proof
after unquantified consuming captures. It passes the complete normalized source
to Core: removing complete wrappers could expose the original grouped alternative
to the root-alternative guard. The Core table includes those captures, so Runtime
adds no separate enclosing prefix. Named/indices projection, String consumers,
empty-match advancement, clones and collection retain the shared execution path.


## Captures confined to one unquantified nullable group's branches

A separate Core constructor proves character-only and empty-only captures for
one required nullable iteration. At least one original branch capture is
required, including optional/later empty captures that never participate.
Complete enclosing wrappers remain optional. Captured empty points beside the
class or its owning alternative retain the separate boundary constructor.

The flat parser enables existing empty-group count and choice effects, then
requires a branch-only role table with no boundary capture. Complete iteration
wrappers retain the original one-character or empty range; branch captures are
undefined when the opposite branch succeeds. Existing interval masks preserve
required, optional and later-choice participation without expanding counts
(RepeatMatcher, 22.2.2.3). Source framing, root assertion-scope guards, strict
class leaves and UTF-16 projection remain shared. Earlier API domains and work
traces remain unchanged. Runtime selects the branch-only proof after boundary
captures using the complete normalized source and no extra enclosing prefix.
Named/indices projection and RegExp/String execution reuse the same result path.


## Unquantified nullable groups with complete captures only

A separate Core constructor proves one required nullable class group with no
branch-local or boundary captures. There may be no original captures, or only
complete enclosing captures. The flat role table must contain only complete
iteration roles; its empty form retains a zero-capture pattern. Capture-free
empty groups, choices and counts may surround the class or occupy its empty
branch. Strict class checks, original alternative ownership, outside gates and
exact-single framing remain shared (22.2.2.3).

The synthetic marker still selects the final character or empty branch. Complete
captures project that whole range, and zero-capture patterns expose only the
whole match. Branch/boundary capture constructors retain their separate domains.
Source counts, clone/drop and matching state remain flat; no input-sized capture
history is introduced. Earlier API domains and work traces remain unchanged.
Runtime selects the complete/zero-capture proof after branch captures using the
original normalized source. Result arrays retain zero-capture length, undefined
unnamed group maps and complete named/indices aliases through String consumers.


## Ungrouped class-versus-empty alternatives

A separate Core constructor proves one root alternative between a checked
character class and empty terms. Original captures may be absent, character-only,
empty-only, conditional points or permanently undefined slots. Unlike grouped
single-iteration framing, this constructor first wraps the entire source and
extracts no boundary assertions. A branch-local leading/trailing assertion
therefore stays inside its original branch and retains a separate proof
(22.2.2). Exactly one root alternative and one class-bearing branch are required;
empty nested choices/counts retain existing flat masks.

The synthetic noncapturing wrapper introduces no capture slots. One required
iteration preserves original first-branch selection, source-order captures and
UTF-16 ranges. Strict classes still prove every leaf even when the empty branch
comes first. Source copies and inspection are precharged; clone/drop remain flat,
and matching retains no capture history. Earlier constructor domains and work
traces remain unchanged. Runtime selects root alternatives after the grouped
single-iteration proofs using the original normalized source and no extra
enclosing prefix. Existing named/indices projection, String consumers and Unicode
empty advancement retain their shared execution path.


## Root string-class alternatives with nonempty class results

A separate Core matcher proves root string-class/empty alternatives with flat
original capture roles. Existing single-q, union, nested-union, finite-operation
and scalar-filtered string-set constructors compile the complete class. A shared
internal query checks the compiled set's empty member; plans whose class can
consume an empty string decline. This also admits empty operands removed by set
operations, while keeping class-branch and empty-branch identity unambiguous.

The consuming branch selects the existing trie's longest member, preserving
original UTF-16 ranges across variable-width words (22.2.2.7). The empty branch
retains first-branch priority. Character/point/empty/undefined role projection
shares the existing source-order table through a common range helper. Root
assertions are never moved and retain separate proofs. Every class leaf is
compiled before branch selection, including empty-first alternatives.

Matching normalizes the initial Unicode boundary before selection and charges
the bounded class prefix before querying it. The empty branch guarantees success
at that first valid boundary, so searching and sticky modes need no candidate
history. Class programs share flat trie storage; captures clone/drop as flat
source-sized data. Earlier constructor/search work traces remain unchanged.
Runtime selects the string-class proof after character-only root alternatives.
A separate matcher body delegates bounded work and source-order projection;
metadata, named indices, String consumers, clones and collection remain shared.


## Explicit selection for string classes containing empty members

A separate Core matcher requires an empty member in the compiled class result
and returns an explicit class-selected flag with the whole UTF-16 range. A
zero-width class member therefore defines character-side captures and points,
while the root empty alternative defines only its own captures. Equal-width
results do not erase that distinction (22.2.2.3 and 22.2.2.7).

Both string-root constructors share complete class compilation and flat source
framing, then require opposite empty-member outcomes. The established nonempty
constructor retains its fields, callbacks and projection. Longest class members
precede its empty member; original root empty-first order still skips the class
query. Tagged projection retains lexical slots and optional/later masks without
allocating a capture vector or input history per match. Root assertion/reference/
count scopes remain separate. Source-sized clone/drop and bounded-prefix work
precharges are unchanged.

Runtime selects the empty-member proof after the nonempty proof on the complete
normalized source. The match stores class selection in the existing branch
selector, so zero-width class captures project from the whole-match range
without a per-match capture allocation. RegExp results, named indices, String
consumers, clones and collection continue through their shared paths. Earlier
successful constructors retain their work traces; new attempts and matching
remain covered by opt-in host work limits.


## Grouped unquantified string-class alternatives

Both string-class capture matchers expose a separate grouped constructor. A
flat source scan requires no root disjunction, then the same noncapturing frame
retains the original inner separator owner. Complete enclosing captures cover
either selected branch; empty sibling captures retain unconditional points,
while class-side points and empty-branch slots retain conditional participation
(22.2.2.3, 22.2.2.7). Nested captures, choices and empty counts share the existing
flat role/mask proof. Class-containing counts, assertions, references and further
consuming terms remain separate proofs.

The constructors share complete class compilation, opposite empty-membership
checks and bounded-prefix matching with root alternatives. Existing root
constructors retain their exact accepted domain and callback traces. No source
wrapper is peeled, captures are not renumbered and no per-match capture history
is allocated.

Runtime attempts grouped proofs after both root proofs on the full normalized
source. Existing matcher bodies retain lexical capture slots, including complete
original wrappers and unwrapped sibling points. The nonempty range or explicit
class-selection flag continues through shared result/indices construction and
String consumers without adding per-match storage.


## Empty captures around one variable-width string class

A separate single-class Core matcher proves one unquantified string class with
complete enclosing captures and empty capturing/noncapturing siblings. The
existing flat empty-group parser runs in consuming mode, preserving start/end
points and optional/later undefined masks. Complete class compilation is shared
with both nullable string-class matchers without changing their constructor
callbacks. Assertions, references, class-bearing counts, nullable alternatives
and additional consuming terms retain separate proofs.

Matching delegates earliest-start/longest-member selection to the string-set
matcher and projects every capture from the original UTF-16 class range. Class
selection is unconditional, so empty members still define complete captures and
point groups (22.2.2.3, 22.2.2.7). Work precedes boundary and prefix/search scans;
empty-member classes need only the first boundary's bounded prefix. Storage,
clone and drop remain flat and source-sized, and matching retains no capture
history.

Runtime selects the single-class proof after grouped alternatives on the full
normalized source. A distinct body delegates search work and projects captures
from the selected class range. Shared result/indices construction and String
consumers retain lexical names, empty advancement and lastIndex behavior.
Search work is charged before matching; opt-in search aborts preserve state.


## Whole-pattern assertions around single string-class captures

A separate constructor admits canonical leading ^/word gates and trailing word
gates/$ around one class and its empty sibling captures. It rejects original
root disjunctions before extracting gates, preserving assertion ownership. The
flat capture proof sees only the body; the class compiler receives the same
whole-pattern gates around the complete original class. Inner assertions,
references, class-bearing counts and further consuming terms stay separate.

The existing string-set matcher filters each possible member by its continuation
assertions before selecting a longest success (22.2.2.4, 22.2.2.7). A failed longer
member therefore leaves shorter members eligible, and their selected original
UTF-16 endpoints supply complete and point capture ranges. Empty membership no
longer guarantees success at the first boundary when gates are present, so work
uses the full search bound in that case. Plain single-class constructors retain
their domain and callbacks.

Runtime selects the assertion proof after the plain single-class proof on the
full normalized source. The existing single-class body delegates gated search
and projects captures from the selected member. Shared RegExp/String paths
retain names/indices, empty advancement, lastIndex and opt-in host aborts.


Unicode failed-search candidates advance by code point (RegExpBuiltinExec and
AdvanceStringIndex, 22.2.7.2, 22.2.7.3). For an empty-capable class followed by
\B on `😀a😀!`, searching from index 2 fails at the word boundary and advances
past the second complete pair to index 5. V8 instead tests index 4 inside that
pair. This is separate from the approved initial-offset leading-boundary rule
and is not adopted. Core and Runtime regressions retain index 5 and reject the
corresponding sticky start. Differential controls use independent native sticky
matching with explicit Unicode candidate advancement and initial normalization.


## Complete wrappers around asserted single string classes

A separate Core constructor recovers complete ordinary capturing/noncapturing
wrappers around an asserted single-class body. A v-aware delimiter scan keeps
all nested class brackets and escape pairs opaque. Three constant-space scans
recover the maximum complete wrapper chain, then the entire retained body must
pass the existing outside-assertion proof. Quantified wrappers, continuations,
branch-local gates, inner assertions and unknown leaves remain separate proofs.

Capturing wrappers precede every retained-body capture in lexical order, so a
flat prefix of whole-range roles preserves original capture slots and defined
empty ranges (22.2.2.7). No wrapper is peeled repeatedly; clone/drop remain flat.
Construction precharges the original source scans, copy and prefix roles before
allocation, then delegates body work unchanged. Earlier constructors retain their
accepted domains and callbacks.

Runtime selects complete-wrapper proof after ordinary outside assertions on the
full normalized source. Prefix roles already own every original enclosing slot,
so no generic enclosing captures are added again. Existing single-class result
projection, named aliases, String consumers and opt-in abort propagation remain
shared.


## Line gates surrounding complete asserted-class wrappers

A separate Core constructor combines canonical outer ^/$ assertions with
complete ordinary wrappers whose retained body already has word or line gates.
Original root disjunctions decline before any framing movement. The v-aware
wrapper scan recovers prefix captures; outer line gates move through only that
complete chain. Equal inner line gates are idempotent at the same input position
(22.2.2.4), while canonical inner word gates retain their order.

The complete combined body must pass the existing outside-assertion proof. Its
selected original class range supplies unchanged whole-prefix and point captures
(22.2.2.7), including empty members. Quantified/partial chains, inner assertions,
references, nullable root alternatives and unknown leaves remain separate proofs.
Original-source work is precharged before recovery, copying and role allocation;
body compilation, flat storage and earlier constructor work remain shared.
Runtime selects this proof after enclosed assertions on the full normalized
source. Prefix slots remain owned by the same matcher body, with shared named
indices, String consumers, Unicode progress and opt-in abort propagation.


## Word gates surrounding complete asserted-class wrappers

A separate Core constructor combines canonical outer word/line assertions with
complete wrappers and their retained canonical gates. Equal word predicates on
the same side are idempotent; opposite predicates on that side are a false
conjunction at the same input position (22.2.2.4). Line gates retain their existing
conjunction semantics, and lexical prefix captures retain their original ranges.

Contradictions still require the entire original class/body proof, including
every leaf, reference and count check. Only after that proof does the plan replace
the class with a fixed empty-word atom guarded by opposite word predicates. This
reuses the existing matcher without adding per-match state or skipping unknown
leaves. Consistent gates delegate ordinary assertion compilation. Flat wrapper
recovery, source precharges and earlier constructor callbacks remain unchanged.
Runtime selects this proof after surrounding line gates on the full normalized
source. The same capture body retains lexical prefix names and original source
metadata, including when contradictory gates select a never-matching plan.
RegExp/String consumers and opt-in abort propagation stay shared.


## Canonical assertion layers between complete class wrappers

A separate Core constructor indexes group-closing positions with one v-aware
pass, then advances inward through canonical assertion frames and complete
ordinary wrappers. Nested classes and escape pairs remain opaque. Group lookup
avoids rescanning bodies when gates interrupt the wrapper chain; all framing,
source copies and prefix roles retain source-sized storage and bounded work.

Predicates on each side accumulate as a conjunction (22.2.2.4). Complete wrapper
captures retain lexical whole-range roles (22.2.2.7), even when a complete class
capture itself becomes part of the recovered prefix. The entire remaining body
must still pass the single-class proof. Contradictory word gates are lowered only
after full class validation. Noncanonical layers, partial/quantified wrappers,
root choices, inner assertions, references, counts and unknown leaves stay closed.
Earlier constructors retain their domains and callbacks. Runtime selects nested
framing after the earlier surrounding-gate proofs on the full normalized source.
The same capture body owns all recovered lexical slots and keeps shared named
indices, String consumers, Unicode progress and opt-in host abort behavior.


The Core `compile_unconditional_assertions_with_work` proof admits word predicates
and canonical-side line assertions in required empty siblings of one unquantified
v/vi string class. For example, `(\b)([\q{a|ab}])(\b)` retains both defined
empty capture slots and the longest class member whose end satisfies the final
boundary. A flat source scan collects start/end conjunctions while copying an
assertion-free shadow that preserves every group, choice, count and capture slot.
The existing single-class body parser proves that shadow and assigns its lexical
capture ranges. Nested v classes and escaped code units stay opaque to assertion
recovery. Every assertion-bearing ancestor must be unconditional: it cannot own
an alternative or have a zero-minimum quantifier. Positive empty-group counts
repeat the same pure predicates at the same position, so their magnitude never
expands construction, matching state or work. Class-bearing counts remain closed.

ECMA-262 edition 17 22.2.2.3 CompileSubpattern, 22.2.2.4 CompileAssertion and
22.2.2.3.1 RepeatMatcher determine capture entry/exit, conjunction and required
empty repetition behavior. Before/after positions are exact because the proved
spine contains one class and only pure empty terms. Only ^ before that class and
$ after it are admitted; reversed line gates, assertion-bearing choices and
optional assertion groups need different proofs. Duplicate predicates are
idempotent. Contradictory word gates lower to a fixed never matcher only after
the entire shadow and original class compile successfully; unknown leaves,
references and extra consuming terms cannot become supported through false gates.
The new constructor precharges source-sized work and reuses the established
allocation-free search and capture projection. Earlier public constructors retain
their accepted domains and callback transcripts. Runtime selects the sibling
proof after the earlier nested-wrapper proof on the full normalized source. The
same body owns all lexical slots, named index aliases, String consumers, Unicode
progress and opt-in host abort behavior. Optional assertion groups and
assertion-bearing choices continue to report separate host unsupported errors.


Core `compile_reversed_line_assertions_with_work` extends the required-sibling
proof to ^ after the class and $ before it, requiring at least one such predicate.
The two constructors share a source scan while the earlier entry retains its
canonical-side domain and exact charge callbacks. Freshly compiled string-set
programs acquire two immutable endpoint flags before sharing: $ at the match
start and ^ at each possible member end. Existing constructors initialize both
flags to false. Search work accounts for each new predicate, and empty-member
search still advances by Unicode code point when a gate rejects a candidate.
No state is allocated or mutated during matching.

ECMA-262 edition 17 22.2.2.4 CompileAssertion applies line predicates at their
actual positions even when they surround an empty term. With multiline matching,
`($)([\q{\n|\na}])(^)` on `\na!` rejects the longer member and retains the
newline member ending at the next line start. The matcher tests these predicates
before choosing the longest eligible member. Without multiline matching, $
before a class restricts its start to input end, while ^ after it restricts its
end to zero; neither justifies discarding the original class proof. Both line
predicates can hold at one empty line boundary, so they are conjoined without
word-style contradiction lowering. Required empty capture counts and lexical
point ranges retain the established proof; optional assertions, assertion choices,
references, class counts and unknown leaves remain closed, including under false
word predicates. Runtime selects this proof after the earlier canonical sibling
proof on the full normalized source. The shared capture body retains original
metadata, lexical indices, String consumer behavior and opt-in host aborts.


Core `RegExpUnicodeAssertedStringClassAlternativeMatcher` proves exactly two root
alternatives: one supported unquantified string class with pure empty terms, and
one entirely pure empty alternative. Either alternative may carry mandatory
word/line predicates; at least one original predicate is required. A flat scan
finds the sole root separator with nested v classes and escaped code units opaque.
Both complete branches must compile before matching. The class branch reuses the
single-class proof. Appending a synthetic sole `[\q{}]` outside the other
branch's groups proves its original terms are all pure empty, retains its capture
slots, and rejects any original class, reference or extra consuming term. Both
plans remain immutable and source-sized.

ECMA-262 edition 17 22.2.2.3 CompileSubpattern gives source-order alternatives
priority at each candidate; 22.2.2.4 CompileAssertion retains predicates on their
own branches. For `^([\q{a|ab}])$|(\b)` on `ab!`, the class alternative fails
its end assertion and the empty alternative captures the boundary at zero.
Captures in the unselected alternative remain undefined. Explicit class-selected
identity distinguishes an empty class member from the original empty alternative,
so their equal ranges cannot erase capture participation. Lexical slot offsets
are checked sums of the two proved branches; no capture history is retained.

Matching precharges initial Unicode normalization, then searches the original
first alternative. A success at the initial candidate wins without searching the
other alternative. Otherwise the second bounded search supplies its earliest
success, and original source order resolves equal starts. This preserves Unicode
candidate progress, longer/shorter eligible class members and flat immutable
state without allocating during matching. Full constructor proof still rejects
unknown leaves in false or unselected alternatives. Callback failures abort
before a result is returned. Grouped root choices, assertion-bearing optional
terms/choices and class repetition remain separate proofs. Earlier constructor
domains and callback transcripts are unchanged. Runtime selects the asserted-root
proof after the earlier single-class endpoint proofs on the full normalized
source. A dedicated body stores the selected class/empty tag and projects all
original lexical slots and named index aliases without capture history. String
consumers, cloning, collection and opt-in host aborts share existing paths.


Core `compile_grouped_with_work` on the asserted string-class alternative matcher
proves a complete ordinary capturing/noncapturing wrapper chain around an
otherwise accepted asserted root body. Three source-sized scans recover only
complete wrappers with nested v classes and escaped q data opaque. The retained
body must still pass the entire root proof; assertions remain on their original
alternatives. Outside predicates, partial wrappers, outside empty siblings and
choice counts remain separate proofs.

ECMA-262 edition 17 22.2.2.7 CompileAtom gives each enclosing capture the complete
selected range, including a selected empty alternative. A scalar prefix count
and checked total slot count retain those original lexical positions ahead of
both alternatives. Prefix projection always returns the whole range; only the
inner alternative slots depend on the selected class/empty tag. For
`(^([\q{a|ab}])$|(\b))` on `ab!`, the outer capture is defined empty and the
class capture is undefined. Prefix construction, cloning and dropping remain
flat, with no per-match storage added. Root construction initializes the prefix
to zero and retains its accepted domain and exact callbacks; searching and
branch-order decisions are unchanged. Runtime selects grouped alternatives after
the earlier root proof on the full normalized source. The same body owns prefix
and branch slots, with generic enclosing slots reset to zero. Original source,
named index aliases, String consumers, collection and host aborts share existing
paths.


Core `compile_surrounding_line_assertions_with_work` on the asserted alternative
matcher admits canonical common ^/$ predicates outside a complete ordinary
wrapper chain. Original root disjunctions decline before any gate is extracted;
`^C|E$` keeps its existing branch-specific meaning. Flat frame and wrapper scans
retain all original lexical capture slots. Only common pure predicates are
conjoined with both original alternatives, which then pass the full existing
root proof. This also covers a plain class/empty body whose only original
predicate is outside the wrapper. Outside word gates, noncanonical layers,
partial wrappers, outside empty siblings and choice counts remain separate.

ECMA-262 edition 17 22.2.2.3 CompileSubpattern, 22.2.2.4 CompileAssertion and
22.2.2.7 CompileAtom require the common continuation to succeed for the selected
alternative and member. On `ab`, `^(()|([\q{a|ab}]))$` rejects the first empty
alternative at zero and captures `ab` through the class alternative instead.
Each class endpoint must satisfy the common suffix before longest-member
selection; failing longer members can expose shorter eligible strings or the
original empty alternative. The synthetic empty class keeps all empty-branch
predicates at one point. Both line predicates can hold on an empty line with
multiline matching, so no word-style contradiction lowering is applied.

Common predicates add no capture slots. Prefix counts and checked total counts
retain whole ranges for every complete enclosing capture, including a selected
empty alternative. Both original branches and class leaves must still prove,
even when common predicates reject all input candidates. Construction, clone,
drop and matching stay flat with source-sized storage and existing bounded
searches. Older root/grouped constructors retain their domains and callbacks.
Nonsticky asserted branches keep their conservative suffix work precharge;
sticky queries retain the established member-width bound. Default quotas remain
unset. Runtime selects the common-line proof after root and complete-wrapper
proofs on the full normalized source. The shared capture body keeps original
source metadata, prefix/branch aliases, String consumers, cloning, collection and
opt-in abort behavior.

Core `compile_surrounding_word_assertions_with_work` on the asserted alternative
matcher admits canonical common word predicates outside complete ordinary
wrappers, with optional canonical ^/$ predicates. It declines original root
disjunctions before extracting any frame. Each original alternative receives
the common start/end predicates and passes the full root proof; lexical prefix
counts are restored without adding slots. Earlier constructors retain their
domains and exact work callbacks.

ECMA-262 edition 17 22.2.2.3–4 requires each alternative's continuation to hold at
its actual endpoints. On `ab!`, `\b(()|([\q{a|ab}]))\B` rejects the first empty
alternative and the longer `ab` member, then selects `a`. Opposite common word
predicates can hold at different boundaries of a consuming member even though
they contradict at the empty alternative's single point. Same-boundary local
contradictions still require proof of every original class leaf and term. A
class-selected empty member retains its branch identity and capture slots.

All enclosing captures retain the whole selected range, while original branch
slots remain conditional. Construction, matching, cloning and dropping use the
existing flat plans. Noncanonical outside layers, partial wrappers, siblings,
choice counts, optional assertions, references and unknown leaves remain
separate proofs. Runtime selects this proof after the common-line constructor on
the full normalized source. The shared capture body retains original source,
flags, lexical names and aliases, String consumers, cloning, collection and
opt-in constructor/exec abort behavior.


Core `compile_nested_assertion_wrappers_with_work` on the asserted alternative
matcher admits canonical assertion layers interleaved with complete ordinary
wrappers. One flat v-aware delimiter pass indexes group ends. Recovery commits
a frame's predicates only when its retained body is one complete wrapper;
otherwise it keeps the original payload boundaries. This preserves local root
assertions without repeatedly scanning nested bodies. Partial and quantified
wrappers retain separate proofs.

Common line flags and same-side word conjunctions are applied to each original
alternative at its actual start/end. ECMA-262 edition 17 22.2.2.3–4 and 22.2.2.7
require the selected member and its continuation to succeed before captures are
projected. Opposite start/end words can hold across a consuming class, while the
empty alternative checks both at one point. Same-side contradictions still pass
both complete original branches and every class leaf through the existing root
proof before they become never matches. Original root predicates keep their
branch-specific scope.

Every peeled capture owns the whole selected range in lexical prefix order;
inner branch slots retain conditional participation and class-selected empty
members keep their explicit identity. Source-sized construction and delimiter
storage remain flat, with no per-match capture history. Earlier constructors
retain their domains and exact callbacks. Noncanonical single layers, outside
siblings, choice counts, optional assertions, references and unknown leaves
remain separate proofs. Runtime selects the layered proof after the common-word
constructor on the full normalized source. The existing capture body retains
original source, flags, lexical name/index aliases, String consumers, cloning,
collection and opt-in constructor/exec abort behavior.


Core `compile_reversed_surrounding_line_assertions_with_work` on the asserted
alternative matcher admits common $ before and/or ^ after complete ordinary
wrappers, with optional common word predicates beside them. It declines original
root choices before recovering any outside frame. Flat wrapper scans retain
lexical prefixes; common predicates are conjoined with each original branch,
then both complete branches pass the existing root proof. Earlier constructors
retain their domains and exact callbacks.

ECMA-262 edition 17 22.2.2.3–4 and 22.2.2.7 require predicates at the actual
selected endpoints. On `x\nab`, `$(()|([\q{\n|\na}]))^` with multiline matching
rejects the empty alternative at index one and the longer newline-plus-a member,
then selects the newline at 1..2. Both reversed line predicates can succeed
across different consuming boundaries or together at an empty-line point. The
existing Single matcher records $ at the start and ^ at the end before selecting
the longest eligible member; no global false-pattern shortcut is introduced.

Every original class leaf and term must prove even when common/local predicates
conflict. Original class-selected empty members retain their explicit branch
identity. Enclosing captures keep whole selected ranges and inner slots keep
conditional lexical participation. Construction, searches, cloning and dropping
remain flat with source-sized storage and the existing work bounds. Duplicate or
noncanonical outside layers, partial wrappers/siblings, choice counts, optional
assertions, references and unknown leaves retain separate proofs. Runtime
selects the reversed common-line proof after the layered constructor on the
full normalized source. The existing capture body retains original source,
flags, lexical name/index aliases, String consumers, cloning, collection and
opt-in constructor/exec abort behavior.


Core `compile_nested_endpoint_assertion_wrappers_with_work` on the asserted
alternative matcher admits canonical and reversed line/word layers interleaved
with complete ordinary wrappers. At least one common $ before or ^ after a
wrapper is required. A flat v-aware delimiter map bounds recovery. Each frame
admits one line predicate and one word predicate per side; recovery commits them
only after identifying a complete wrapper, preserving the original local root
source when it stops. Earlier constructors retain their domains and callbacks.

ECMA-262 edition 17 22.2.2.3–4 and 22.2.2.7 require every shared continuation at
the selected alternative's actual endpoints. Canonical and reversed line flags
form four independent requirements, with word conjunctions at each side. On a
newline, `^($(([\q{\n|\na}])|())^)$` with multiline matching selects the newline
and defines both enclosing captures. On newline-plus-a, both consuming members
fail their continuation and the original empty alternative wins instead. All
line predicates can hold at an empty-line point. Opposite start/end words may
hold across a consuming member, while the empty alternative uses one point.

Only predicates are duplicated into the original root branches. Both complete
branches and every original class leaf must prove before same-side conflicts
become never matches. Checked lexical prefixes retain whole selected ranges;
branch-local slots and class-selected empty members retain their identity.
Construction, cloning, dropping and matching remain flat with source-sized
storage and existing bounded searches. Same-layer duplicates, noncanonical
orders, partial wrappers/siblings, choice counts, optional assertions, references
and unknown leaves remain separate proofs. Runtime selects the mixed-layer
proof after the reversed common-line constructor on the full normalized source.
The shared capture body retains original source, flags, lexical name/index
aliases, String consumers, cloning, collection and opt-in host abort behavior.


Core `RegExpUnicodeSurroundingAssertedStringClassAlternativeMatcher` proves a
two-way asserted class/empty choice with required empty siblings around it. A
flat v-aware scan finds the closest class ancestor owning one disjunction and
records the lexical capture insertion point. The original payload passes the
complete root proof with all outside predicates conjoined at its endpoints.
The full surrounding source separately passes the existing Single spine proof,
with one synthetic empty class replacing only that payload. Neither proof can
skip unknown original terms or class leaves under false predicates.

The synthetic class proves outside structure and capture participation only;
its empty width never decides match eligibility. Opposite start/end words may
reject the placeholder while a consuming original class still succeeds. Required
assertion groups and positive empty counts contribute pure predicates at one
point. Assertion-bearing choices, zero-minimum assertion groups, quantified
class ancestors, extra classes, consuming outside terms and references remain
separate proofs. A required predicate may originate in the root or its spine.

ECMA-262 edition 17 22.2.2.3–4 and 22.2.2.7 require lexical capture slots to
retain their participation and actual ranges. On `ab!`,
`(\b)(([\q{a|ab}])|())(\B)` selects `a`: the first capture is 0..0, the whole
wrapper and class captures are 0..1, the empty branch is undefined, and the final
capture is 1..1. Outside whole/start/end/never roles are stored flat; original
branch slots are inserted between the outside lexical slots. Projection uses
the selected root range and explicit branch tag, including class-selected empty
members. Searching delegates to the bounded ordered root finder. Construction,
cloning and dropping use source-sized flat storage. Older constructors and exact
callbacks remain unchanged. Runtime selects the surrounding proof after the
mixed-layer constructor on the full normalized source. Its dedicated body
retains explicit class/empty tags, lexical point/whole/branch projection, original
source and names, String consumers, cloning, collection and opt-in host aborts.


Core `compile_exact_single_counts_with_work` on the surrounding asserted
alternative matcher proves exact-one counts on the sole class atom and all
class-containing ancestors. A flat v-aware scan removes only {1} and {1,1},
including leading zeros and lazy forms, before invoking the unchanged surrounding
proof. It requires at least one such count and rejects every other count on
that path. Counts on outside empty/predicate groups remain original. Every
original class leaf, branch and outside term still passes the full proof.

ECMA-262 edition 17 22.2.2.3 RepeatMatcher executes an exact-one body once, even
when it matches an empty string. Within this reference-free single-choice spine,
the body's capture slots are initially undefined and clearing them for that one
iteration preserves the unquantified result. No capture delimiter or branch is
removed, so lexical roles, original source order, class-selected empty identity
and whole/start/end ranges remain unchanged. On `ab!`,
`(\b)(([\q{a|ab}]){1}|()){1}(\B)` still selects `a` with outside points at zero
and one. Other minima/maxima, optional/open counts, references and class-bearing
repetition retain separate proofs. Existing constructors retain their exact
callbacks and accepted domains. Construction, normalization and dropping remain
flat with source-sized storage; matching reuses the bounded root finder.
Runtime selects the exact-one constructor after the original surrounding
constructor on the full normalized capture source. The same body retains
original count spelling, names, flags, point/whole/branch aliases, String
consumers, cloning, collection and opt-in host abort behavior.


### Exact-one counts in asserted root string-class alternatives

A separate root constructor accepts {1}/{1,1}, padded and lazy, on the sole
class atom and ordinary ancestors within its branch. The same flat normalizer
removes only class-path count tokens. It retains every capture and assertion in
its original branch, then runs the full existing root proof. RepeatMatcher's
mandatory single execution includes empty bodies; initial capture clearing has
no additional effect in this reference-free, unquantified choice. Empty-branch
counts retain their original participation rules. Other class-path counts,
unknown leaves even behind impossible assertions, and references still decline.
Existing constructors retain their domains and callbacks. Runtime selects the
root exact-one constructor after the preceding surrounding proofs, retaining
original source/flags/names, branch aliases, String consumers, cloning and host
abort behavior. Construction, clone/drop and matching reuse the flat existing plans
(22.2.2.3, RepeatMatcher).


### Exact-one counts around a single string class

A separate Single constructor reuses the flat class-path normalizer for {1} and
{1,1}, including padded and lazy forms, on the sole atom and its ordinary
ancestors. It then tries the original plain, mandatory canonical and reversed
assertion proofs. Every class and outside term still requires complete proof,
including false predicates. Removing only exact-one count tokens preserves all
capture delimiters and endpoint roles; mandatory single execution includes
empty members and only clears initially undefined inside captures in this
reference-free spine (22.2.2.3, RepeatMatcher). Original empty sibling counts
retain optional/later undefined slots and required point participation. Other
class counts, references, class-bearing alternatives and unknown leaves remain
separate proofs. Existing constructors retain exact domains/callbacks and flat
storage. Runtime selects the exact-one Single constructor after preceding
proofs on the full normalized capture source, preserving original metadata,
names/indices, String consumers, cloning, collection and host-abort behavior.


### Exact-one counts in unasserted string-class alternatives

The existing nonempty-result and empty-member string-class alternative types
have separate exact-one constructors. A shared helper removes only {1}/{1,1},
padded/lazy, from the sole class atom and its ancestors, then selects the original
root or grouped proof without moving any capture or alternative. The complete
original class and both branches require proof even when an empty-first branch
wins. Result-emptiness guards retain their distinct domains. The empty-member
type keeps explicit selected-branch identity for defined class empties versus
unselected captures. Mandatory one-iteration execution only clears initially
undefined captures in this reference-free choice; outside empty counts retain
all required/optional/later capture roles (22.2.2.3, RepeatMatcher). Existing
constructors retain domains and exact callbacks, and storage/clone/drop remain
flat. Other counts, assertions, references and unknown leaves stay separate
proofs. Runtime selects the new constructors after the earlier Single proof on
the complete normalized capture source. Existing result types retain original
metadata, names/indices, branch identity, String consumers, cloning, collection
and opt-in host-abort behavior.


### Simple repetition of variable-width string classes

A separate matcher proves one class atom or ordinary class/empty group under
*, + or ?, greedy or lazy, with only pure-empty outside terms. Full Single or
grouped alternative proofs retain every original class/branch and capture role.
A flat delimiter scan separates inside captures from enclosing/point slots;
a synthetic empty class proves the outside spine and is never matched. Other
class-path exact-one counts may normalize through the existing helper. Other
counts, nested repetitions, assertions, references and extra consumption stay
separate proofs.

The outside continuation always succeeds. Once a body succeeds, matching never
needs to retry an earlier class member: each consuming iteration uses the first
preferred longest member. This preserves source priority rather than maximizing
whole-match length; `[\q{aa|aaa}]*` on `aaaa` chooses `aaa`. A plain class + may search
for its first mandatory member; nullable bodies always succeed at the initial
Unicode boundary. Mandatory + permits an empty body and retains its captures.
Optional iterations reject empty progress and retry the consuming class even
when the empty alternative comes first. Rejected empty attempts preserve the
previous accepted iteration. Lazy zero-minimum forms execute no body. These are
RepeatMatcher's minimum/progress rules (22.2.2.3).

Match state contains only the whole range, optional final iteration and its
explicit class/empty identity. Inside slots project the final iteration, or stay
undefined when none executes; outside slots project the whole range. A capturing
group enclosing a repeated class atom therefore differs from a repeated
capturing group. Storage/clone/drop remain flat and source-sized. Class-prefix
and search work is charged before inspection; no default quota or per-iteration
capture history is added. Runtime stores the same whole range, final iteration
and selected-class tag through a dedicated body, preserving original source,
names/indices, lastIndex, String consumers, cloning and collection.


Core braced string-class repetition admits one count with minimum zero or one,
with any upper bound, around the same class atom or ordinary class/empty group.
Additional class-path exact-one wrappers retain their lexical scopes. Exact-one
principal counts use the existing single-class and nullable-alternative proofs.
The new constructor fully proves the body and pure-empty outside continuation
before applying its bound, including an upper bound of zero. Construction charges
the original source before scanning; normalization replaces the principal count
with star or plus before the existing exact-one normalization, retaining every
original group and leaf.

ECMA-262 edition 17 22.2.2.3 RepeatMatcher limits accepted iterations, including a
required empty-first body. Optional empty bodies fail the progress check before
capture state commits. Thus `(()|([\q{a|ab}])){1,2}` on `ababa` accepts a required
empty iteration followed by one `ab` iteration. The final body range is `0..2`,
while an outside capture encloses the whole repetition. A zero upper bound leaves
all body slots undefined. Greedy matching stops at the upper bound without an
extra class query; lazy matching accepts the minimum with the same source
priority and empty-branch identity as simple repetition.

Upper-bound decimal parsing saturates at usize::MAX: every optional accepted
iteration advances in the allocated UTF-16 input, so a larger upper bound cannot
limit any observable execution. No iteration history or count-sized storage is
allocated. A larger minimum remains a separate proof: `[\q{aa|aaa}]{2}` on `aaaa`
needs to retry the shorter first member. The existing simple constructor retains
its domain and exact work callbacks through the shared private finder. Runtime
selects the braced proof after the earlier simple and exact-one constructors on
the full capture source. Its dedicated body retains the whole/final ranges and
class-selected tag, preserving original source, names/indices, lastIndex, String
consumers, cloning and collection.


Core larger braced minima use a separate counted-nullable string-class proof.
After the same full simple-repetition body/outside proof, the class must contain
an empty member or the body must contain a pure-empty alternative. Required
bodies therefore cannot fail at an input boundary, so preferred class members
need no retry history. Plain nonnullable classes with a minimum above one retain
separate proofs for required failures and possible word backtracking. Earlier constructors and match/work
contracts remain unchanged.

ECMA-262 edition 17 22.2.2.3 RepeatMatcher accepts empty iterations while the
minimum remains positive. These required empty bodies replace earlier capture
state. On `ab`, `([\q{|a|ab}]){2}` retains a final empty class capture at `2..2`,
whereas the optional empty attempt in `([\q{|a|ab}])+` leaves the last `ab`
capture intact. An empty-first required body repeats its deterministic empty
capture profile; a class-first body tries preferred class members before falling
back to its class empty member or explicit empty branch. After the first required
empty body, all remaining required copies at that point share the same profile.

The finder collapses this required empty tail, then admits only consuming optional
bodies within the upper bound. It stores the whole range, final body range and
class-selected tag, with no capture history or storage proportional to a count.
Every class query is charged before inspection; an empty-first exact huge count
needs only the initial finder charge. Construction charges the original source
before scans, copying and decimal arithmetic. Minimum parsing may saturate at
usize::MAX because positive required iterations are bounded by the input length.
The optional allowance is computed by decimal subtraction before saturation:
huge adjacent lower/upper bounds can still leave exactly one optional iteration.
Leading zeros and unbounded uppers preserve that arithmetic. Original capture
scopes and every class leaf remain proved, including unselected empty-first
classes. Runtime selects larger nullable counts after the earlier simple and
zero/one-minimum proofs on the full capture source. A dedicated body retains
whole/final ranges and the class-selected tag, preserving original source,
names/indices, lastIndex, String consumers, cloning and collection.


Core larger plain string-class counts admit a separate prefix-free proof. The
canonical finite trie must have no empty member, no live member that prefixes
another live member, and no scalar fallback. The existing full simple body and
pure-empty outside proofs retain every group and class leaf before this check.
Empty alternatives, assertions and nested class-path loops remain separate.
The property is checked after case canonicalization, so `A|ab` is prefix-free
without i and overlapping under i. Set algebra can disable a terminal while
retaining dead suffix edges: `ab|abc` minus `abc` therefore remains eligible.

The property scan charges the trie capacity and node passes before inspection
and fallible temporary allocation. Flat parent/live arrays propagate active
terminals from newer children to their unique older parents. A live child of a
terminal identifies a strict prefix. The shared trie remains immutable, and
storage/clone/drop stay flat and bounded by the compiled class size. Earlier
public matcher domains, callbacks and capture projections remain unchanged.

ECMA-262 edition 17 22.2.2.3 RepeatMatcher can retry a shorter class member after
a required continuation fails. Prefix-free members have at most one accepted
endpoint at each input position, so those word retries cannot change a result.
The finder executes the required run, then greedy/lazy optional members within
the exact decimal upper-minus-lower allowance. Nonsticky failure retries the next
Unicode boundary after the failed run's start, including positions inside its
first word. On `abcda`, `[\q{abcd|bc|da}]{2}` fails at zero and succeeds at one
with `bc` then `da`; its final body range is `3..5`, while outside scopes enclose
`1..5`. Sticky execution tests its initial Unicode boundary. Each class query and suffix
search is charged before inspection. A minimum larger than the remaining UTF-16
length fails with constant finder work, after complete construction proof.
Only the whole/final ranges and class-selected tag are retained. Runtime selects
this proof after the earlier nullable-count constructors on the full capture
source. Its dedicated body preserves original source, lexical names/indices,
lastIndex, String consumers, cloning and collection.


Core separately proves a nonnullable plain string-class body with minimum exactly
two. This admits overlapping finite words and scalar fallback while retaining
the existing full proof of capture scopes, empty outside terms and class leaves.
Empty members, empty alternatives, assertions and larger required counts use
separate proofs. Optional counts retain exact decimal upper-minus-lower arithmetic.

ECMA-262 edition 17 22.2.2.3 RepeatMatcher retries class alternatives when the
second required iteration fails. The finder walks accepted first-member endpoints
from shortest to longest and retains the latest endpoint admitting a second
member. It queries the preferred second member at each endpoint. This yields the
same preferred successful pair as longest-first backtracking: `[\q{aa|aaa}]{2}`
on `aaaa` must choose `aa` then `aa`; `[\p{Any}\q{ab}]{2}` on `ab` must choose
scalar `a` then scalar `b`. Greedy or lazy optional iterations follow only after
the required pair succeeds, with no failing continuation beyond the proved pure
empty suffix. The whole range and final body range suffice for capture projection.

Construction charges and counts active trie terminals plus one possible scalar
endpoint. Each pair search precharges its trie walk and all second-member probes
using the lesser of the candidate count and input inspection bound. Execution
allocates no endpoint lists or capture history. Failed nonsticky pairs retry the
next Unicode boundary after their start, including starts within a prior word.
All earlier public matcher domains, work callbacks and projections stay unchanged.
Runtime selects this proof after the earlier prefix-free constructor on the full
capture source. A dedicated matcher body retains original source, names/indices,
lastIndex, String consumers, cloning and collection.


Core proves a separate nonnullable plain string-class body with three or more
required iterations. The full existing simple-body proof retains lexical capture
scopes and pure-empty outside terms; empty class members, empty alternatives,
assertions and nested class-path loops remain separate. Count bounds retain original
decimal digits for exact upper-minus-lower arithmetic before the resulting
optional allowance saturates at the platform index limit.

ECMA-262 edition 17 22.2.2.3 RepeatMatcher may revisit earlier member choices
when a later required iteration fails. The finder stores one selected range per
required iteration in a flat vector. It tries the preferred next member, and on
failure pops ranges until an earlier member has a strictly shorter accepted
endpoint. It then retries the required continuation. Thus `[\q{aa|aaa}]{3}` on
`aaaaaa` selects three `aa` members after longer first/second choices fail.
The endpoint helper considers scalar fallback and canonical finite words in the
same original UTF-16 coordinate space as the preferred class matcher.

Every transition, shorter-endpoint inspection and class search is charged before
inspection. The vector reserves fallibly after a charge, and holds at most the
required count, bounded by remaining input units because every member consumes.
Its storage is reused across candidate starts; construction, cloning and drop
remain flat. A separate RegExpUnicodeMatchError distinguishes callback failures
from matching-storage capacity failures; both are host failures. Impossible huge
minima fail with constant finder work before allocation. Once the required run
succeeds, the proved pure-empty continuation admits greedy/lazy optional members
without further required-history search. Only the whole range and final body
range escape for capture projection. Nonsticky failed runs retry the next Unicode
boundary after their starting position. Earlier matcher APIs retain their exact
domains, callback work and projection. Runtime selects this proof after the
earlier two-required constructor on the full capture source. Its dedicated body
preserves original source, lexical names/indices, lastIndex, String consumers,
cloning and collection. Runtime dispatch carries matching work and storage errors
separately, mapping capacity failures to host heap errors.


Core separately proves unconditional assertions around a nonnullable string-class
repetition. The principal count may be *, +, ?, or a braced count other than
exact-one forms handled by earlier proofs. The repeated body contains one fully
proved class and empty capture terms, with no empty class member, empty arm or
body assertion. Outside ^/$/word/nonword assertions are collected at the whole
match's start or end. Line predicates retain m semantics; word predicates retain
Unicode i folding. Contradictory predicates fail after the full class proof.

A flat outside-scope scan records assertion participation and alternation. A
scope containing both remains closed, as does a non-exact-one repetition of an
assertion-containing group. Assertion-free empty groups may retain alternatives
and counts, with their original lexical capture profiles. Assertions are replaced
with empty noncapturing groups in a shadow source; the original class body and
all remaining terms pass the existing simple capture proof. Both shadow storage
and outside-scope storage reserve fallibly after charging the original source.
Hidden body predicates, references, consuming siblings, additional classes and
unknown leaves remain closed, including beneath an exact-zero principal count.

ECMA-262 edition 17 22.2.2.3 RepeatMatcher may retry both a prior class member
and an optional repetition count after an outside suffix fails. Each flat
continuation frame records its input point, last tried endpoint, and whether its
body and stopping continuation are exhausted. Greedy frames try members before
the suffix; lazy frames try the suffix before members once the minimum is met.
Member retries use strictly descending original UTF-16 endpoints. For example,
`^[\q{aa|aaa}]*$` on `aaaa` retries a shorter first member after the suffix
fails, then succeeds with two `aa` members and final range `2..4`.

Live continuation depth is bounded by the input point count plus its root frame.
Growth reserves fallibly after a charge; no Rust recursion or capture-history
copies are needed. Frame transitions, class inspections and predicate checks are
charged before inspection. Matching work and capacity failures use the separate
host error variants. Zero iterations leave repeated captures undefined while
outside scopes remain defined at their lexical points. A short lazy empty
success retains bounded matching work on long input. Failed nonsticky starts
advance one Unicode point, retaining the established leading-boundary policy.
Earlier public matcher domains, callback work and projections stay unchanged.
Runtime selects this proof after the larger-required-count constructor on the full
capture source. A dedicated body retains original source, names/indices,
lastIndex, String consumers, cloning and collection, with matching-capacity and
work failures retained as separate host errors.


Core separately proves nullable string-class bodies with unconditional outside
assertions and a zero/one minimum. The body may contain an empty class member,
a class/empty alternative in either order, or both. The shared constructor still
proves every class leaf and capture scope before checking this new domain. The
earlier nonnullable constructor applies its original filter after the shared
proof, retaining its exact public domain and callback sequence. Outside predicate
collection and conditional-assertion exclusions remain unchanged. Larger nullable
minima remain separate because their required empty runs need additional handling.

ECMA-262 edition 17 22.2.2.3 RepeatMatcher accepts a required empty iteration,
resets captures for that iteration, and rejects optional empty progress. At a
one-minimum root, an empty-first arm is tried before class members. A class-first
body tries its consuming endpoints before its preferred empty result. Failure of
later assertions/count continuations may retry a required consuming member after
the empty-first path fails: the required empty iteration already uses one count.
Optional iterations try consuming class members only, including when an earlier
empty arm must be rejected before reaching them.

A nullable continuation frame retains whether its required empty choice was tried
and which branch entered it. Only the one required root may add a zero-width
child; later children consume input, so live depth is bounded by input points plus
two. Class-empty and separate empty arms that reach the same input/count state
share possible continuations: captures do not feed predicates or references in
this proof. The preferred empty branch therefore retains its capture identity;
a lower-priority equivalent empty branch cannot add a successful continuation.
Storage grows fallibly after a charge, with separate work/capacity host failures.
No positive input-length lower bound is applied to a nullable required iteration.

Zero iterations leave repeated captures undefined. A required class-empty result
retains defined empty class captures; a required separate empty arm retains its
own captures and clears class captures. Later consuming iterations overwrite
both profiles with the final accepted class member. Original lexical projection
therefore uses the final range plus its class-versus-empty tag, including a
zero-width final required range. Huge upper counts retain exact decimal bounds
and bounded short lazy matching work. Unknown leaves remain unproved even under
zero counts or a preferred empty arm. Runtime selects this proof after the
nonnullable asserted constructor on the full capture source. Its dedicated body
retains original source, lexical names/indices, lastIndex, String consumers,
cloning and collection, including defined required-empty captures and undefined
zero-iteration captures.


Core separately proves asserted nullable string-class bodies with minimum exactly
two. It retains the same complete class/capture proof and unconditional outside
predicates. The shared nullable continuation finder now receives its proved plan;
the earlier zero/one-minimum API retains its original domain and exact callback
sequence. Nullable minima above two remain separate.

ECMA-262 edition 17 22.2.2.3 RepeatMatcher accepts each required empty iteration,
resetting captures for the newly entered body. A consuming first iteration may
therefore be overwritten by a required empty second iteration. Class-first and
empty-first bodies retain their original ordering during word, branch and count
retries. A required empty second result is retained as a defined zero-width final
range with its class/empty tag; optional empty copies remain rejected.

Positive-body pruning tests available input independently of the remaining
required count: other required copies may be empty. Thus one input point can
satisfy a consuming first copy and an empty second copy, preserving the latter's
capture profile. At most two required zero-width children can be added; later
children consume input. Live continuation depth is bounded by input points plus
three, with fallible growth and charged transitions. The final range/tag still
suffices for projection because all prior body captures reset on entry.
Huge upper bounds retain exact decimal allowance and bounded lazy empty work.
Matching work/capacity failures remain host errors. Unknown zero-width paths and
unselected class leaves remain fully proved. Runtime selects this proof after
the zero/one-minimum nullable asserted constructor on the full capture source.
Its dedicated body retains source, lexical names/indices, lastIndex, String
consumers, cloning and collection, including required-empty capture resets.


Core separately proves finite asserted nullable string-class counts with minimum
at least three. It reuses the complete class/capture and unconditional outside
assertion proof, then excludes scalar fallback. Finite trie terminal counts prove
an empty-only body; every original class leaf remains checked before shortcuts.
The exact decimal upper-minus-lower allowance is retained before saturating the
individual bounds. Smaller nullable APIs retain their domains and charged work.

ECMA-262 edition 17 22.2.2.3 RepeatMatcher accepts required empty iterations and
rejects optional empty progress. When the minimum fits within remaining input
units, a shared flat continuation stack executes required empties directly; its
live depth is bounded by twice remaining input units plus one. Larger minima
contain more required copies than any consuming path can use. Class-first paths
then enumerate consuming words greedily and finish with the preferred required
empty body, independently of outer count greediness. Empty-first paths enumerate
zero, one, then more consuming required copies; each tier retains longest-member
order, the original optional allowance and greedy/lazy continuation order.
Search preserves leftmost candidate order before advancing the consuming tier.

These reductions omit duplicate empty padding, whose continuation cannot inspect
prior captures: references and conditional/body assertions are outside the proof.
The last required empty preserves its class/empty identity; a final consuming
body preserves its own range and identity. Empty-only bodies directly check the
suffix predicate and project the preferred required empty. Large decimal bounds
therefore allocate no count-sized stack. Transitions and fallible continuation
growth remain charged, with work/capacity failures separate from JS exceptions.
Scalar fallback remains a separate proof. Runtime selects the finite larger-count
proof after the two-required nullable asserted constructor on the full normalized
capture source. Its dedicated body preserves source, flags, lexical names and
indices, lastIndex, String consumers, cloning and collection. Required-empty and
consuming final bodies retain their distinct capture profiles.


Core separately proves larger asserted nullable string-class counts with scalar
fallback. It shares the finite larger-count constructor through a private
scalar-presence parameter; the finite-only API retains its domain, callback
sequence and Debug representation. The scalar API requires fallback presence,
keeps complete class/capture and unconditional outside-predicate validation,
and preserves the exact decimal optional allowance before bound saturation.

A scalar endpoint consumes one Unicode point and at least one UTF-16 unit.
Thus ECMA-262 edition 17 22.2.2.3 RepeatMatcher's required-empty count reduction
applies unchanged: small minima execute directly on bounded flat state; larger
class-first runs finish on their preferred required empty; empty-first runs
retain consuming-count tiers and the original optional count/greediness.
Scalar and finite-word endpoints keep descending end order before class EPS.
The finite candidate bound counts a scalar fallback as potentially consuming,
so an unproved empty scalar language cannot trigger the empty-only shortcut.
No storage depends on a huge count's decimal value. Required-empty capture
identity, Unicode boundaries/folding and host work/capacity failures retain
the shared semantics. Runtime selects the scalar proof after the finite larger-
count constructor on the full normalized capture source. Its dedicated body
preserves source/flags, lexical names and index aliases, lastIndex, String
consumers, cloning and collection, including the final required-empty identity.


Core separately proves exact-zero outside assertion scopes around an already
proved string-class repetition. A flat scope pass identifies skipped groups and
rejects a skipped scope containing the repeated class. Two fallible source-sized
masks mark zero-scope starts and ends. A second linear pass replaces only known
^/$/word predicates inside those scopes with noncapturing empty terms; original
captures, counts and the entire class body remain present. Escaped literals,
unknown terms, references, lookaround and additional classes acquire no proof.

ECMA-262 edition 17 22.2.2.3 RepeatMatcher invokes the continuation directly for
maximum zero; the body and its captures never participate. Thus assertions inside
a skipped scope, including nested alternatives and counts, impose no predicate,
and their lexical captures remain undefined. The transformed source preserves
those capture slots under their original zero counts. Other outside assertions
must still satisfy the unconditional proof, including at least one live outside
predicate. The existing nonnullable, zero/one-minimum nullable, two-required,
larger finite and larger scalar plans retain live count/word/branch behavior.
Both preprocessing and delegated compilation/search remain charged and fallible;
no storage depends on huge decimal counts. Runtime selects the skipped-scope
proof after larger nullable scalar counts on the full normalized capture source.
Its dedicated body preserves original source/flags, names and index aliases,
lastIndex, String consumers, cloning and collection, including undefined captures
in every skipped scope. All-inert outside predicates remain a separate proof.


Core separately proves string-class repetition when every outside predicate is
inside an exact-zero scope. It reuses skipped-scope validation and masking without
changing the asserted skipped-scope API. A charged linear scan of the transformed
source skips complete classes and rejects any remaining ^/$/word predicate,
including body predicates. The remaining repetition and pure-empty continuation
select existing simple, bounded, counted-nullable, two-required or larger-required
plans. Every original lexical capture remains present; skipped scopes are
undefined, while enclosing scopes retain their normal participation.

ECMA-262 edition 17 22.2.2.3 RepeatMatcher skips a maximum-zero body before testing
its predicate. Consequently an original ^/$/word term inside that scope cannot
constrain start/end search. Matching retains leftmost Unicode search, original
member/count/branch ordering, required-empty resets and optional-empty rejection.
Huge nullable minima use the existing count reductions; impossible nonnullable
minima fail against the input bound. Callback errors from fixed-state finders
map to Work; the larger required plan retains Work and Capacity distinctions.
Live predicates, body predicates, references, unknown terms and additional class
paths remain outside this proof. Runtime selects this proof after the live-
predicate skipped-scope constructor on the full normalized capture source. Its
dedicated body preserves original source/flags, names/index aliases, lastIndex,
String consumers, cloning and collection, including defined enclosing empty
captures around skipped children and undefined captures in the children.


Core separately proves one string class whose outside predicates are all skipped.
The shared skipped-scope scan now takes a private occurrence shape; existing
repetition APIs retain their original shape, domains and charged work. The single
shape locates the class leaf directly, accepts bare and exact-one class counts,
and still rejects a zero ancestor containing that leaf. A shared charged scan
rejects any remaining live predicate while leaving nested classes opaque.
The existing plain/exact-one single-class capture proof validates the complete
transformed source, including every class and all original lexical slots.

ECMA-262 edition 17 22.2.2.3 enters an exact-one class body even when it matches an
empty member. The class is therefore selected and its final occurrence is a
defined range, including a zero-width range. Skipped scopes remain undefined;
ordinary enclosing captures around skipped children retain defined empty points.
Longest-member and Unicode start/search semantics remain the single-class
semantics. Other class counts, class-bearing choices, references, live/body
predicates and unknown terms retain separate proofs. Allocation and callback
failures keep their constructor/work distinction. Runtime selects the single-
class skipped-predicate proof after the assertion-free repetition proof on the
full normalized capture source. Its dedicated body preserves source/flags,
lexical name/index aliases, lastIndex, String consumers, cloning and collection.
A required empty class member stays defined beside undefined skipped children.


Core separately proves a single class with skipped outside scopes and remaining
live unconditional predicates. It shares the single-occurrence skipped-scope
scan without changing earlier API domains or charged work. At least one live
predicate remains after masking; exact-one class-path counts normalize before
the existing unconditional/reversed single-class capture proof validates the
complete retained source. All original lexical slots remain present.

ECMA-262 edition 17 22.2.2.4 and 22.2.2.7 preserve outside predicate continuations
when selecting a class member. A longer member failing a live end/word condition
leaves shorter members eligible. Skipped predicates impose no constraint, and
their captures remain undefined; ordinary surrounding empty captures retain
the selected start/end points. Empty members enter the required class body and
retain a defined final class range. Reversed line gates, Unicode search/folding,
callbacks and constructor/capacity failures retain their existing semantics.
Conditional predicate paths, references, additional classes and unknown terms
remain unproved. Runtime selects the live-predicate single-class proof after the
single-class skipped-only proof on the full normalized capture source. Its
separate body retains named/escaped-name aliases, indices, source/flags, Unicode
search, lastIndex, String consumers, cloning and collection. A failed longer
member's live suffix still permits a shorter member at the same start.


Core separately proves a root or grouped class/empty alternative whose outside
predicates all occur in skipped exact-zero scopes. The existing single-leaf
masking scan preserves every original group and rejects a zero-count ancestor
containing the class. Remaining live predicates decline before complete
nonempty/empty-class alternative proofs validate both branches. Bare/exact-one
class paths preserve original choice ownership, source order and lexical slots.

ECMA-262 edition 17 22.2.2.3 and 22.2.2.7 preserve alternative order at the first
Unicode candidate. An empty-first alternative suppresses the class query; an
empty member selected from a class-first branch retains class identity and its
captures. Pure-empty siblings and enclosing captures retain their own points
and ranges, while skipped scopes and unselected branch captures stay undefined.
A class containing only unknown leaves still declines when an empty branch
would win. Construction and searches retain charged work and callback recovery. Runtime
selects this alternative proof after the two skipped-predicate single-class
proofs on the full normalized source. Dedicated capture projection retains
source/flags, named aliases, lastIndex, String consumers, cloning and collection;
empty-first choices keep class slots undefined and class-first empty members
keep their selected captures defined.


Core separately proves class/empty alternatives combining skipped outside scopes
with remaining live predicates. After masking skipped predicates and validating
bare/exact-one class paths, complete asserted-root or required outside-spine
proofs retain original choice ownership. The selected class/empty branch remains
explicit even when both could match the empty string. Every original class leaf
and conditional branch is validated before selecting a result.

ECMA-262 edition 17 22.2.2.3–4 and 22.2.2.7 apply common predicates at each branch's
actual endpoints. A longer class member rejected by a live continuation permits
a shorter member, and a failed empty-first branch permits the class branch.
Earliest Unicode candidates and source order resolve successful branch ties.
Skipped and unselected captures remain undefined; enclosing and point captures
retain their own participation. Reversed line gates, clone/drop, charged work,
callback cancellation and constructor/capacity failures keep their existing
semantics. Other class-path counts, references and unknown leaves remain
unproved. Runtime selects this alternative proof after the skipped-only
alternative proof on the full normalized capture source. Its dedicated body
retains original source/flags, named capture aliases, lastIndex, String consumers,
cloning and collection. An empty-first branch failing a live suffix permits the
class branch; a class-first empty member retains defined class captures.


Core separately proves live-asserted string-class repetition with outside
predicate scopes whose minimum count is zero. The skipped-scope scanner admits
optional, star and zero-minimum braced counts in this separate constructor.
Earlier exact-zero APIs keep their original scan mode, domains and charged
work. The class-bearing repetition and all remaining terms still pass complete
asserted finite/scalar/nullable-body proofs.

ECMA-262 edition 17 22.2.2.3 RepeatMatcher rejects an optional iteration whose
body ends at its starting index. A scope containing only predicates and empty
terms therefore takes zero iterations, for greedy and lazy forms alike. Its
predicates impose no constraint and its captures remain undefined, including
positive-count descendants inside that skipped scope. Ordinary surrounding
empty captures retain their own points. Masking erases only known predicates;
remaining literals, references, additional classes and unknown terms still
must be proved and decline. A zero-minimum class-containing ancestor also
declines. Required predicates outside skipped scopes remain live and retain
actual endpoint continuations. Construction and storage remain flat and
source-sized; constructor/capacity errors and callback cancellation retain their
existing distinction. Runtime selects the zero-minimum outside-scope repetition
proof after the skipped class/empty alternative proofs on the full normalized
capture source. Its dedicated body retains original source/flags, lexical named
aliases, lastIndex, String consumers, cloning and collection. Optional predicate
scope captures remain undefined beside defined ordinary outside point captures.


Core separately proves class repetition when every outside predicate belongs to
a zero-minimum scope. It shares the optional-scope masking scan without changing
earlier API domains or charged work, rejects any remaining live predicate, and
validates the complete class/body through the five assertion-free repetition
proofs. Nonprogressing predicate scopes retain undefined captures; ordinary
outside points and required class/empty iterations preserve their own identity.

ECMA-262 edition 17 22.2.2.3–4 still controls greedy/lazy count, longest-member and
ordered class/empty continuation. With no live endpoint predicate, a bounded
nullable repetition may stop after its optional consuming allowance, and a lazy
required-empty path may succeed without consuming. Search, Unicode folding and
leading surrogate boundaries keep their existing behavior. Unknown leaves,
references, additional classes and live/conditional predicates remain separate
proofs. Constructor/capacity failures and callbacks retain their distinction. Runtime
selects this assertion-free zero-minimum-scope proof after its live-predicate
counterpart on the full normalized source. Its dedicated capture body retains
source/flags, lexical named aliases, lastIndex, String consumers, cloning and
collection, including defined ordinary points beside undefined skipped slots.


Core separately proves one required class whose outside predicates all belong
to zero-minimum scopes. The single-occurrence masking scan validates the full
bare/exact-one class path before the existing single-class proof preserves its
lexical slots. Optional predicates impose no constraint and their captures stay
undefined. Ordinary outside point scopes retain defined empty ranges.

ECMA-262 edition 17 22.2.2.3 and 22.2.2.7 still require entering the sole class
body, including a selected empty member, so its class captures remain defined.
Longest-member priority and Unicode search remain unconstrained by masked
predicates. A zero-minimum class-containing ancestor, root alternatives, other
class-path counts, live predicates, references and unknown leaves retain their
separate proofs. Cloning, charged work, callback cancellation and constructor/
capacity failures keep their existing semantics. Runtime selects this required
single-class proof after the assertion-free repetition proof on the full
normalized capture source. Its dedicated body retains source/flags, named
aliases, lastIndex, String consumers, cloning and collection, including defined
empty class captures beside undefined optional predicate scope slots.


Core separately proves one required class combining zero-minimum outside scopes
with remaining live predicates. Optional predicates keep undefined captures;
mandatory canonical/reversed line and word predicates retain their actual
endpoint conjunctions. Bare/exact-one class paths normalize before the existing
unconditional/reversed single-class proofs validate all original terms.

ECMA-262 edition 17 22.2.2.3–4 and 22.2.2.7 preserve required empty class entry,
lexical capture identity and continuation priority. A rejected longer class
member leaves shorter members eligible for a live suffix. Skipped scopes impose
no constraint and ordinary enclosing points stay defined. Search, folding,
leading surrogate boundaries, cloning, charged work, cancellation and constructor/
capacity failures retain their existing semantics. Other class-path counts,
alternatives, references and unknown leaves remain separate proofs. Runtime
selects this live-predicate single-class proof after its assertion-free
counterpart on the full normalized capture source. Its dedicated body retains
source/flags, named aliases, lastIndex, String consumers, cloning and collection,
including shorter-member retries and defined required empty class captures.


Core separately proves one class/empty alternative when every outside predicate
belongs to a zero-minimum scope. The shared single-occurrence scan masks only
known optional predicates and preserves all original groups. Complete root,
grouped and bare/exact-one alternative proofs validate both branches, including
an unselected class. Remaining live predicates decline before selection.

ECMA-262 edition 17 22.2.2.3–4 and 22.2.2.7 retain earliest Unicode candidates and
original branch order. An empty-first choice keeps class captures undefined;
a selected empty class member in a class-first branch keeps them defined.
Ordinary outside point scopes retain their own ranges, while optional scopes
and unselected branches stay undefined. Unknown leaves, references, additional
classes and other class-path counts retain separate proofs. Clone/drop, charged
work, cancellation and constructor/capacity failures retain their distinction.
Runtime selects this assertion-free alternative proof after the two required
single-class proofs on the full normalized capture source. Its dedicated body
retains source/flags, lexical named aliases, lastIndex, String consumers, cloning
and collection, including distinct empty-first and class-first empty captures.


Core separately proves one class/empty alternative combining zero-minimum
outside scopes with remaining live predicates. Masking preserves original groups
and lexical slots, while complete asserted-root or required outside-spine proofs
retain both branch conditions and original alternative ownership. Bare/exact-one
class paths remain mandatory when their branch is selected.

ECMA-262 edition 17 22.2.2.3–4 and 22.2.2.7 apply common predicates to each branch's
actual endpoints. A failed empty-first branch permits the class branch, and a
longer class member failing a live suffix permits shorter members. Selected
empty class members stay distinct from empty alternatives. Skipped and
unselected captures remain undefined; ordinary enclosing and point captures
keep their own participation. Every class leaf is proved before selection,
including unknown leaves in an unselected branch. Other class counts,
references and consuming siblings retain separate proofs. Clone/drop, charged
work, cancellation and constructor/capacity failures retain their distinction.
Runtime selects this live-predicate alternative proof after its assertion-free
counterpart on the full normalized capture source. Its dedicated body retains
source/flags, named aliases, lastIndex, String consumers, cloning and collection,
including endpoint retries and distinct empty-first/class-first captures.


Core separately proves string-class repetition beside required predicate choices
such as `((\b)|(\B)){1}`, `(\b|())` and `(^|())`. A flat parser proves every
outside leaf and records success for all eight start-line/end-line/word-boundary
truth profiles. Only complete ordinary scopes that always succeed are projected;
alternatives or additional counts enclosing the consuming class retain separate
proofs. Source-sized fallible storage and flat traversal, cloning and destruction
retain deep nesting without recursive owned trees.

ECMA-262 edition 17 22.2.2.3–4 preserves the first successful alternative and
resets captures in unselected branches. Positive zero-width counts preserve the
last required iteration; zero-minimum counts perform no iteration. Per-capture
truth masks therefore retain required empty points and undefined skipped slots.
Replacing only the proved scope predicates leaves original group/count syntax
and capture indexes for the complete existing class-body proof. Matching uses
that continuation and records actual endpoint truth, including multiline and
Unicode ignore-case word aliases. Other predicates remain live, so endpoint
retry and class-member priority remain with the proved matcher. Every unknown
leaf is rejected before matching, including leaves in skipped branches. Charged
work, cancellation and capacity failure remain distinct from JavaScript errors.
Runtime selects this proof after the existing optional-scope constructors on the
full normalized capture source. Its dedicated result preserves both endpoint
truth profiles alongside class selection and the final iteration; named aliases,
indices, lastIndex, String consumers, cloning and collection retain original
source and capture identity.

Core also proves a required single string class beside always-successful outside
predicate choices. The same flat truth-mask proof preserves original captures;
only a bare class or exact-one counts on its required spine qualify. Complete
single-class proofs validate remaining live predicates and every class leaf,
including required empty class members and shorter successful continuations.
The result records actual endpoint truth alongside the required class range.
Zero-minimum outside scopes remain undefined and required alternatives retain
source priority (ECMA-262 edition 17 22.2.2.3–4). Repetition proof domains and work
accounting are unchanged. Runtime selects this proof after the repetition
choice proof on the full normalized capture source. The dedicated body retains
endpoint truth, source/flags, named aliases, indices, lastIndex and String
consumer behavior through cloning and collection.

Core also proves mandatory always-successful predicate choices around one
class/empty alternative. The flat parser keeps the class-bearing choice explicit
and marks both of its branch subtrees conditional. Only scopes outside every
such choice receive endpoint masks, so a skipped branch cannot gain captures.
Every remaining branch and class leaf retains its full asserted or assertion-free
proof. Bare/exact-one class paths preserve ordered class/empty selection,
required empty class identity and shorter successful endpoint continuations.
Line/word choices project captures at actual whole-match endpoints, including
empty-first retries after failed live gates (ECMA-262 edition 17 22.2.2.3–4).
Branch-local required predicate choices, other class counts and unknown leaves
retain separate proofs. Repetition and single-class domains and charged work
remain unchanged. Runtime selects the alternatives proof after required
repetition and single-class choice proofs on the complete normalized capture
source. Its dedicated body preserves both endpoint truth profiles, class/empty
selection, source/flags, named aliases, indices, lastIndex and String consumers
through cloning and collection.

Core separately proves predicate-only outside choices whose success equals a
conjunction of line/word assertions, including choices that can fail. The flat
truth-mask proof selects a complete scope only when its success mask equals
start-line, end-line, word/nonword predicates or a conjunction of them. A live
canonical gate precedes that entire zero-width scope; the original internal
predicates become empty groups while all group/count syntax and lexical slots
remain. Conditional capture masks retain the original first successful branch.
The complete existing repetition continuation therefore tests real endpoint
gates and can retry shorter class members or earlier repetition endpoints
(ECMA-262 edition 17 22.2.2.3–4). Zero-minimum scopes still perform no iteration.

Contradictory choices retain a proved never gate, without hiding unknown class
leaves or other terms. Disjunctions that cannot be represented by conjunctions,
references, consuming siblings and conditional class scopes remain separate
proofs. The preceding three choice APIs retain their exact domains and charged
work. Allocation, traversal and clone/drop remain flat and source-sized; host
limits and cancellation remain distinct from JavaScript errors. Runtime selects
this proof after the three always-successful choice proofs on the full normalized
capture source. Its dedicated body retains endpoint truth, last-iteration state,
source/flags, named aliases, indices, lastIndex and String consumers through
cloning and collection.

Core separately proves one required class beside predicate choices equivalent to
live conjunction gates. Bare and exact-one class paths retain required empty
class identity and original lexical capture slots. The complete single-class
proof sees canonical outside gates, so a failed suffix can retry a shorter
member before search advances. Endpoint truth projects the original ordered
predicate captures; zero-minimum scopes still remain undefined (ECMA-262 edition
17 22.2.2.3–4). Other class counts, consuming siblings, unknown leaves and
nonconjunctive choices retain their separate proofs. Earlier choice APIs retain
exact domains and charged work. Runtime selects this single-class proof after
the conjunctive repetition proof on the full normalized capture source. The
dedicated body retains endpoint truth, required class identity, source/flags,
named aliases, indices, lastIndex and String consumers through clone/collection.

Core separately proves mandatory conjunctive predicate choices around a required
class/empty alternative. Endpoint masks apply only to scopes outside every
class-bearing branch. Canonical live gates therefore preserve empty-first
retries and shorter successful class members, while original source order and
required empty class identity select the correct captures (ECMA-262 edition 17
22.2.2.3–4). Bare/exact-one class paths retain lexical slots; skipped scopes stay
undefined. Unknown leaves are still proved even behind never gates, and
nonconjunctive or branch-local conditional scopes retain separate proofs.
Earlier choice APIs retain exact domains and charged work. Runtime selects this
alternatives proof after conjunctive repetition and single-class proofs on the
full normalized capture source. Its dedicated body retains endpoint truth,
class/empty identity, source/flags, named aliases, indices, lastIndex and String
consumers through cloning and collection.

Core separately proves arbitrary predicate-only outside choices beside one
string-class repetition. Complete scopes compile to eight start-line/end-line/
word-boundary truth profiles, including disjunctions such as `(^|\b)` and
`($|\B)`. The flat source proof preserves every original group, count and lexical
slot while removing only the selected scope predicates. Remaining bare
assertions retain their original compiled conditions. Profile gates run inside
every actual candidate-start and repetition-endpoint probe, so a failed gate
permits shorter members, earlier count endpoints or later search starts
(ECMA-262 edition 17 22.2.2.3–4). Conditional capture masks preserve original
first-successful branch participation at the accepted endpoints.

Private assertion-free compilation is restricted to this complete outside-scope
proof and still validates every class/body term. Nonnullable bodies retain
existing counts; nullable bodies currently require minimum zero, one or two.
Larger nullable minima need a separate required-empty reduction proof. Extra
profile probes charge sixteen work units each; preceding matchers use no extra
profile and retain exact domains and callback sequences. Unknown terms,
references and conditional class spines remain separate proofs even behind
never profiles. Source traversal, captures and continuation storage remain flat
and host limits remain opt-in. Runtime selects this truth-profile proof after
the three conjunctive choice proofs on the complete normalized capture source.
The dedicated body retains actual endpoint truth, final iteration identity,
source/flags, named aliases, indices, lastIndex and String consumers through
cloning and collection.

Core separately proves nullable repetitions with a minimum at least three and
arbitrary outside predicate-choice profiles. Complete class/body compilation
precedes exact optional-count extraction and finite/scalar prefix bounds. The
existing required-empty reduction retains class-first versus empty-first tier
order, final iteration identity and exact optional allowances even when decimal
minima exceed machine integers (ECMA-262 edition 17 22.2.2.3 RepeatMatcher).
Profile gates run inside the empty-only and reduced continuation probes, so failed
endpoint disjunctions can retry shorter members or different consuming tiers.
A zero profile is a proved never gate after complete construction.

No matcher storage or iteration scales with the numeric required-empty count.
Conditional captures use accepted whole-match endpoint truth; unknown class
leaves, references and other class spines retain separate proofs even behind
never gates. Earlier constructors and matching callbacks retain exact domains
and charged work. Runtime selects this counted truth-profile proof after the
smaller-minimum proof on the complete normalized capture source. Its dedicated
body retains final iteration identity, endpoint truth, source/flags, named
aliases, indices, lastIndex and String consumers through cloning and collection.


Core separately proves a required single class beside arbitrary outside
predicate-choice profiles. Complete bare/exact-one source validation retains
required empty class identity and original lexical slots. A private member
continuation checks both truth masks before longest-member selection, so rejected
longer or empty members leave shorter members and later starts eligible
(ECMA-262 edition 17 22.2.2.4, 22.2.2.7). Existing string-set find uses an
infallible accepting continuation and retains exact behavior and charged work.

The new proof charges bounded prefix/search work and sixteen units per actual
profile probe. An empty member does not restrict searching to the first start
when profiles can reject it. Accepted endpoint truth projects ordered predicate
captures; all class/body terms are validated even behind never profiles. Other
counts, class-bearing choices, references and consuming siblings retain their
separate proofs. Source, storage, clone/drop and capture projection remain flat.
Runtime selects this required-class truth-profile proof after the larger
nullable-count proof on the complete normalized capture source. Its dedicated
body preserves endpoint truth, required class identity, source/flags, named
aliases, indices, lastIndex and String consumers through cloning and collection.


Core separately proves arbitrary outside predicate-choice profiles around a
required class/empty alternative. Only mandatory scopes outside every
class-bearing choice contribute masks. Complete original branch proofs retain
line/word assertions, exact-one counts and lexical capture slots. A private
member continuation enforces both truth masks inside asserted branches; plain
alternatives test empty branches in source order at each Unicode candidate.
Failed empty-first continuations can therefore retry the class, and failed
longer members retain shorter members and later starts (ECMA-262 edition 17
22.2.2.3–4).

Explicit class selection preserves required empty class identity, while accepted
whole-match endpoint truth projects ordered predicate captures. Earlier public
find methods and their charged work remain unchanged. New continuation probes
charge sixteen work units; flat source-sized storage and complete class/body
validation precede matching even behind never profiles. Branch-local conditional
scopes, other class counts, references and consuming siblings retain separate
proofs. Runtime selects this alternatives truth-profile proof after the
required-single proof on the full normalized capture source. The dedicated body
retains actual endpoint truth, class/empty selection, source/flags, named aliases,
indices, lastIndex and String consumers through cloning and collection.


Core separately proves two root alternatives with branch-local predicate
choices: one bare/exact-one string-class branch and one pure-empty predicate
branch. Each complete branch reuses the single-class truth-profile proof. A
noncapturing tautology permits branches with bare predicates; an appended empty
class proves the pure-empty branch at one position without adding capture slots.
Complete validation rejects original extra classes, references and unknown terms
even when a preceding branch always succeeds or a profile is never true.

Each immutable branch search retains its own endpoint continuations. The earliest
start wins; ties follow original root order (ECMA-262 edition 17 22.2.2.3–4).
Explicit branch selection preserves required empty class identity and leaves all
unselected branch captures undefined. Lexical offsets project the original
slots; neither synthetic proof term appears in captures or Runtime source.
Source, branch storage and clone/drop remain flat. Enclosing root-choice wrappers,
additional branches, other class counts and consuming empty-branch terms retain
separate proofs. Earlier APIs retain exact domains and charged work. Runtime
selects this root branch-local proof after the mandatory outside-choice
alternatives proof on the complete normalized capture source. Its dedicated body
preserves branch ownership, endpoint truth, source/flags, named aliases, indices,
lastIndex and String consumers through cloning and collection.


Core separately proves complete ordinary wrappers around two root alternatives
with branch-local predicate choices. A flat enclosing-group scan removes only
complete unquantified capturing/noncapturing wrappers before the full root
branch proof. Mandatory enclosing captures project the accepted whole range,
including empty matches, while shifted inner slots retain original branch
ownership (ECMA-262 edition 17 22.2.2.7).

The grouped proof reuses immutable branch continuations and source-priority
selection without additional matching work. Construction charges before wrapper
inspection and retains flat source-sized storage, clone/drop and capture
projection. Unknown terms behind never gates still undergo complete branch
validation. Counted/optional wrappers, outside predicates, lookarounds and other
class spines retain separate proofs. Earlier APIs retain exact domains and
charged work. Runtime selects the complete-wrapper proof after the root
branch-local proof on the full normalized capture source. Its dedicated body
retains complete outer captures, inner branch ownership, endpoint truth, original
source/flags, named aliases, indices, lastIndex and String consumers through
cloning and collection.


Core separately proves exact-one counts on class-bearing paths through grouped
branch-local alternatives. The existing complete class-path scan removes only
`{1}`/`{1,1}` counts and their padded/lazy spellings. Every retained class, group
and branch still passes the complete grouped truth-profile proof. One required
execution retains complete and branch-local captures even when the selected
branch is empty (ECMA-262 edition 17 22.2.2.3 RepeatMatcher).

Count recognition examines decimal digits without converting them to a machine
integer, so arbitrarily padded values of one retain the same meaning. Pure
predicate counts off the class path retain their original participation.
Construction charges before normalization; matching and capture projection reuse
the immutable grouped proof without extra matching work. Other class-path
counts, references and unknown leaves behind never profiles retain separate
proofs. Earlier APIs retain exact domains and charged work. Runtime selects the
exact-one wrapper proof after the unquantified complete-wrapper proof on the
full normalized capture source. Its dedicated body retains required complete
captures, branch ownership, endpoint truth, source/flags, named aliases, indices,
lastIndex and String consumers through cloning and collection.


Core separately proves bare outside line/word assertions surrounding complete
grouped branch-local alternatives. Flat prefix/suffix scans collect conjunctions
into the two endpoint truth masks; escaped literals remain body terms. The
complete body still passes the grouped proof, optionally after exact-one
class-path normalization. Each original root branch conjoins the common masks
with its own profile before any member or empty continuation succeeds. Failed
common gates therefore retain shorter members, empty-first retries and later
starts (ECMA-262 edition 17 22.2.2.3–4).

Outside assertions add no capture slots. Complete enclosing ranges and original
conditional branch captures retain accepted endpoint truth. A contradictory
common mask becomes a bounded-work no-match only after complete body/class
construction; unknown terms behind that gate remain unsupported. Construction
charges before source inspection and normalization. Normal matching reuses the
profiled branch searches; storage and clone/drop remain flat. Outside captured
scopes, consuming siblings and other class-path counts retain separate proofs.
Earlier APIs retain exact domains and charged work. Runtime selects the common
outside-gate proof after exact-one grouped alternatives on the complete
normalized capture source. The dedicated body retains complete ranges, branch
ownership, endpoint truth, source/flags, named aliases, indices, lastIndex and
String consumers through cloning and collection.


Core separately proves captured pure-predicate/empty scopes around a grouped
class/empty choice. A flat choice-hole proof isolates the complete inner root
alternative while preserving its original lexical insertion point. Both inner
branches compile before outside profile extraction. The outside parser treats
that proved choice as one opaque payload, retains every original slot, and
combines pure predicate scopes and bare gates into endpoint masks. Predicates
inside an ordered selected scope retain its disjunctive success mask; they are
not individually conjoined.

A synthetic empty class builds only outside capture roles, without adding slots.
Accepted endpoint truth projects conditional outside points, whole-range roles
project enclosing groups, and the original numeric interval delegates inner
branch captures (ECMA-262 edition 17 22.2.2.3–4, 22.2.2.7). Both inner branches
conjoin outside profiles before member/empty continuations succeed, retaining
shorter-member, empty-first and later-start retries. Optional zero-width scopes
remain undefined. Contradictory common profiles use bounded matching work only
after complete inner and outside validation.

Existing projection callers use the unchanged parser mode and retain exact
domains and callback sequences. All traversal, storage and clone/drop remain
flat and source-sized; class-path counts still require exact one. Further
class-bearing choices, references, consuming outside siblings and unknown terms
even behind skipped/never scopes retain separate proofs. Runtime selects the
captured outside-scope proof after bare common-gate alternatives on the full
normalized capture source. The dedicated body retains outside point roles,
complete ranges, inner branch ownership, actual endpoint truth, source/flags,
named aliases, indices, lastIndex and String consumers through cloning and
collection.


Core separately proves one optional direct root group containing a class/empty
alternative with branch-local predicate choices. `?`/`{0,1}`, including padded and
lazy forms, retain minimum zero and maximum one. RepeatMatcher rejects every
zero-width body iteration at minimum zero: an empty-first alternative must still
retry consuming class members, and an empty class member cannot establish a
participating iteration (ECMA-262 edition 17 22.2.2.3).

Greedy matching probes only consuming class members at the current normalized
Unicode start; lazy matching chooses zero iterations immediately. No later start
can precede the valid zero-iteration result in this complete direct pattern. Both
original branches undergo full construction even when lazy/never paths would
skip them. A missing final iteration leaves every original capture, including
the optional root group, undefined; consuming iterations retain original lexical
branch offsets and complete ranges. Defined empty matches remain distinct from
participating empty bodies.

Prefix work and endpoint probes charge before inspection; storage and clone/drop
remain flat. Outside terms, enclosing choice wrappers and other root counts
retain separate proofs. Earlier APIs retain exact domains and charged work.
Runtime selects this complete optional-root proof after captured outside-scope
alternatives. Its dedicated body preserves final-iteration absence, actual
endpoint truth, branch ownership, source/flags, named aliases, indices, lastIndex
and String consumers through cloning and collection.


Core separately proves complete unquantified ordinary wrappers around an optional
branch-local class/empty root. A flat scan removes only complete wrappers before
the existing optional proof validates both original branches. Mandatory enclosing
captures cover the accepted whole range, including a zero-iteration empty match;
every skipped optional-root capture remains undefined (ECMA-262 edition 17
22.2.2.3, 22.2.2.7). Lexical inner slots shift by the checked enclosing count.
Matching delegates progress and member retry rules unchanged. Storage, traversal
and clone/drop remain flat and source-sized. Outside terms and quantified
wrappers retain separate proofs; earlier API domains and charged work remain
unchanged. Runtime selects this grouped optional-root proof after direct optional
alternatives. Its dedicated body retains mandatory enclosing ranges separately
from final-iteration absence, actual endpoint truth, original lexical slots,
source/flags, named aliases, indices, lastIndex and String consumers through
cloning and collection.


Core separately proves exact-one class-path counts around grouped optional root
alternatives. A flat normalization retains exactly one optional group count and
removes only proved `{1}`/`{1,1}` counts, including padded/lazy forms. The complete
grouped optional-root proof subsequently validates every retained original term
and both branches. Optional class leaves and additional optional class-path
groups remain outside this proof. Mandatory enclosing captures participate even
when the optional root takes zero iterations; skipped inner captures remain
undefined (ECMA-262 edition 17 22.2.2.3, 22.2.2.7).

Exact decimal recognition avoids integer conversion for arbitrarily long leading
zeros. Source-sized normalization and checked capture projection remain flat;
matching delegates the existing optional progress/member retries unchanged.
Earlier exact-one normalization uses its original mode and retains exact API
domains, charged work and captures. Other counts, outside terms and unknown
leaves even in lazy/never bodies retain separate proofs. Runtime selects this
exact-one grouped optional proof after ordinary grouped optional alternatives.
The dedicated body retains mandatory outer ranges, final-iteration absence,
actual endpoint truth, source/flags, named aliases, indices, lastIndex and String
consumers through cloning and collection.


Core separately proves bare outside line/word gates around an optional
branch-local class/empty root. The complete optional body, including ordinary
and exact-one enclosing paths, validates both original branches before any
zero-iteration shortcut. Common profiles stay separate for zero iterations and
join the live class endpoint masks for consuming continuations. An outside gate
can reject zero iterations and force a lazy optional root to consume
(ECMA-262 edition 17 22.2.2.3–4).

The matcher compares the earliest valid zero and nonempty starts, using original
greediness only when they coincide. A valid zero continuation at the requested
start bounds greedy class probing to that start; lazy matching can return zero
immediately. Rejected member endpoints still retry shorter members. Later zero
starts advance by Unicode scalar boundaries, with leading-boundary normalization
for requested trailing surrogates. Mandatory enclosing captures cover accepted
empty ranges; all skipped optional-root slots remain undefined.

Source-sized proof, storage and clone/drop stay flat. Prefix scans and each truth
probe charge before inspection; contradictory common gates use bounded matching
work only after full construction. Captured outside scopes, consuming siblings,
other counts and unknown leaves even in lazy/never bodies retain separate
proofs. Earlier API domains and charged work remain unchanged. Runtime selects
this outside-gate optional proof after exact-one grouped optional alternatives.
The dedicated body retains zero/consuming identity, mandatory enclosing ranges,
actual endpoint truth, original lexical slots, source/flags, named aliases,
indices, lastIndex and String consumers through cloning and collection.


Core separately proves captured pure-predicate/empty outside scopes around one
optional class/empty truth root. The opaque atom includes the original ordinary
root group and its complete zero-to-one count. Full optional-root construction
validates both branches before the outside projection skips that atom, reserving
all original inner lexical slots. Capturing/noncapturing root openers retain the
correct outside insertion point; exact-one class-path normalization keeps the
optional count intact.

The existing flat outside-scope proof provides conditional point masks and
mandatory whole-range roles. Its endpoint profiles feed the optional zero and
consuming continuations, retaining forced lazy consumption, shorter-member and
later-start retries. Outside captures can participate when the optional body
skips; every skipped original inner slot remains undefined, while mandatory
enclosing captures retain the accepted empty range (ECMA-262 edition 17
22.2.2.3–4, 22.2.2.7).

Storage, traversal and clone/drop stay flat and source-sized. All original inner
and outside terms validate even under skipped/never scopes; contradictory common
profiles use bounded matching work after construction. References, extra classes,
consuming outside siblings and other class-path counts retain separate proofs.
Earlier API domains, charged work and captures remain unchanged. Runtime selects
this captured outside-scope proof after bare outside-gate optional alternatives.
Its dedicated body preserves conditional outside points, mandatory enclosing
ranges, final-iteration absence, original inner slots, actual endpoint truth,
source/flags, named aliases, indices, lastIndex and String consumers through
cloning and collection.


Core separately proves a direct class/empty truth root with an exact zero count,
including padded `{0}`/`{0,0}` and lazy forms, beside captured outside predicate
scopes. Full construction validates both original root branches before erasing
all possible iterations. The outside projection reserves the original complete
root capture interval and retains conditional point masks and mandatory enclosing
roles. Every root/body capture stays undefined; mandatory enclosing captures can
still hold an accepted empty range (ECMA-262 edition 17 22.2.2.3, 22.2.2.7).

The existing outside continuation kernel executes only zero-width paths: class
profiles are disabled after complete body validation. Outside gates can require
later Unicode scalar positions; sticky matching retains its requested normalized
start. Deep groups, long finite words, storage and clone/drop stay flat and
source-sized. Truth probes charge before inspection, and contradictory outside
profiles use bounded matching work after construction. Unknown terms in either
region, including the unexecuted body and skipped/never scopes, references,
additional classes, consuming outside siblings and other root counts retain
separate proofs. Earlier API domains and charged work remain unchanged. Runtime
selects this zero-count proof after captured outside optional-root scopes. The
dedicated body preserves undefined body slots, conditional outside points,
mandatory enclosing ranges, actual endpoint truth, source/flags, named aliases,
indices, lastIndex and String consumers through cloning and collection.


Core separately proves an optional outer root containing complete nested
ordinary/exact-one wrappers around a branch-local class/empty truth choice. A
flat balanced scan identifies the complete outer group and its padded/lazy
zero-to-one count. Exact-one normalization and complete-wrapper projection then
retain every original lexical capture while both original branches validate.
All captures inside the optional root, including mandatory inner wrappers, stay
undefined on zero iterations. A nonempty iteration gives each complete inner
wrapper the accepted iteration range (ECMA-262 edition 17 22.2.2.3, 22.2.2.7).

Matching delegates the existing optional progress and endpoint retry kernel with
a checked enclosing-capture count. Empty-first branches and empty class members
cannot establish participating minimum-zero iterations; no later start precedes
a valid zero-iteration result. Storage, traversal and clone/drop remain flat and
source-sized. Unknown terms even in lazy/never bodies, additional optional/zero
inner counts, outside terms and other outer counts retain separate proofs.
Earlier API domains and charged work remain unchanged. Runtime selects this
nested optional proof after zero-count root alternatives. The dedicated body
retains complete enclosed ranges, zero-iteration absence, original lexical slots,
actual endpoint truth, source/flags, named aliases, indices, lastIndex and String
consumers through cloning and collection.


Core separately proves an exact zero-count outer root containing nested
ordinary/exact-one wrappers around a class/empty truth choice. The complete
outer count and all inner wrappers validate before both original branches compile.
Every original capture inside the root remains undefined, including otherwise
mandatory inner wrappers. Matching returns an empty range at the normalized
Unicode start with bounded work and no body queries (ECMA-262 edition 17
22.2.2.3, 22.2.2.7).

The existing flat optional kernel executes only its zero-iteration path; the
zero-count capture projection always returns undefined. Deep wrappers, long
finite bodies, source-sized storage and clone/drop remain flat. Unknown terms
in the unexecuted body, additional optional/zero inner counts, outside terms and
other outer counts retain separate proofs. Earlier API domains, charged work
and captures remain unchanged. Runtime selects this nested zero-count proof after
nested optional alternatives. The dedicated body preserves every undefined inner
slot, normalized starts, source/flags, named aliases, indices, lastIndex and String
consumers through cloning and collection.


Core separately proves captured outside predicate/empty scopes around a nested
optional truth root. A flat ancestor scan finds the nearest optional class-owning
group and its original capture insertion point before the opener. Class data
stays opaque; the complete nested optional atom and both original branches
validate before outside projection reserves all original inner slots. Ordinary
and exact-one inner wrappers remain inside the optional capture interval.

The existing outside role/mask proof and live optional continuation kernel retain
conditional outside points, mandatory enclosing ranges, forced lazy consumption,
shorter-member retries and earliest complete starts. Every skipped inner slot,
including mandatory wrappers inside the optional root, remains undefined while
outside captures can participate (ECMA-262 edition 17 22.2.2.3–4, 22.2.2.7).
Storage, traversal and clone/drop remain flat and source-sized. Unknown terms in
either region even under skipped/never scopes, additional optional class-path
counts and consuming outside siblings retain separate proofs. Earlier API
domains and charged work remain unchanged. Runtime selects this captured-scope
nested optional proof after nested zero-count alternatives. Its dedicated body
retains outside point/whole roles, every nested optional slot, actual endpoint
truth, source/flags, named aliases, indices, lastIndex and String consumers through
cloning and collection.


Core separately proves captured outside predicate/empty scopes around a nested
zero-count truth root. A flat zero-count ancestor scan preserves the original
capture prefix; the complete nested body and both branches validate before the
root becomes one opaque outside-proof atom. Every inner ordinary/exact-one
wrapper stays inside the zero-count capture interval and remains undefined.
Outside conditional points and mandatory enclosing ranges can still participate
(ECMA-262 edition 17 22.2.2.3, 22.2.2.7).

After construction, class profiles are disabled and the existing outside kernel
searches only zero-width continuations, retaining normalized Unicode starts,
sticky behavior and later positions required by outside gates. Traversal,
storage and clone/drop remain flat and source-sized; all truth probes charge
before inspection. Unknown terms in either region even under zero/skipped/never
scopes, additional optional/zero class-path counts and consuming outside siblings
retain separate proofs. Earlier API domains, charged work and captures remain
unchanged. Runtime selects this zero-count nested outside-scope proof after the
corresponding optional proof. Its dedicated body preserves outside participation,
every undefined inner slot, actual endpoint truth, source/flags, named aliases,
indices, lastIndex and String consumers through cloning and collection.


Core separately proves a complete optional truth root with bounded zero/one
counts along its sole class path. A flat scan preserves all original groups and
removes only class-path counts whose upper bound is at most one. A nonempty
outer iteration forces every such scope to run once: inner lazy zero paths fail
the outer RepeatMatcher progress check before consuming retries. A zero upper
bound disables consumption only after both original branches validate fully
(ECMA-262 edition 17 22.2.2.3, 22.2.2.7).

The existing optional continuation kernel retains outer greediness, live endpoint
checks and shorter-member retries. Every inner capture remains undefined on zero
iterations; a consuming iteration retains the original lexical participation.
Pure predicate scopes keep their original counts and conditional captures.
Traversal, storage and clone/drop remain flat and source-sized. Unknown terms
even under skipped/never scopes, additional consuming paths, unbounded or larger
class-path counts and outside terms retain separate proofs. Earlier API domains,
charged work and captures remain unchanged. Runtime selects this bounded-path
optional proof after captured outside nested zero-count scopes. Its dedicated
body retains original lexical slots, zero-iteration absence, live endpoint truth,
source/flags, named aliases, indices, lastIndex and String consumers through
cloning and collection.


Core separately proves a complete zero-count truth root with bounded zero/one
class-path counts. The flat bounded-path normalization preserves every original
group while both original branches compile fully before the root loses every
iteration. Every capture remains undefined, including mandatory wrappers and
nested optional/zero scopes (ECMA-262 edition 17 22.2.2.3, 22.2.2.7).

The existing optional kernel runs only its zero-iteration path, with disabled
class profiles and no body queries. Empty matches retain normalized Unicode
starts and sticky bounds with bounded matching work. Deep groups, long finite
words, decimal count scanning, storage and clone/drop remain flat and
source-sized. Unknown terms even under zero/lazy/never paths, larger or unbounded
inner counts and outside terms retain separate proofs. Earlier API domains,
charged work and captures remain unchanged. Runtime selects this bounded-path
zero-count proof after the corresponding optional proof. Its dedicated body
preserves every undefined inner slot, normalized starts, original source/flags,
named aliases, indices, lastIndex and String consumers through cloning and
collection.


Core separately proves larger and unbounded class-path counts inside a complete
zero-count truth root. A flat scan removes only quantifiers on its sole class
path, preserving every original group. Decimal lower/upper ordering validates
before any count is erased, including values beyond native counters. At least
one path count must exceed the earlier zero/one grammar; both original branches
then validate completely before every root iteration is disabled (ECMA-262
edition 17 22.2.2.3, 22.2.2.7).

All original captures remain undefined. Matching uses only the existing
zero-iteration path, retaining normalized Unicode starts and sticky bounds
without executing inner counts or querying the class body. Unknown terms even
under erased/never paths, additional consuming paths and outside terms retain
separate proofs. Traversal, decimal scanning, storage and clone/drop remain flat
and source-sized. Earlier API domains, work and captures remain unchanged.
Runtime selects this erased-count proof after bounded zero-count class paths.
Its dedicated body preserves every undefined inner slot, original source/flags,
named aliases, indices, lastIndex and String consumers through cloning and
collection.


Core separately proves captured outside predicate/empty scopes around a
zero-count truth root with larger or unbounded inner class-path counts. The flat
nearest-zero-ancestor scan retains the original capture insertion point while
class data stays opaque. The complete quantified body and both branches validate
before the root becomes one outside-proof atom. Every erased capture remains
undefined; outside conditional points and mandatory enclosing ranges can still
participate (ECMA-262 edition 17 22.2.2.3, 22.2.2.7).

The existing outside continuation kernel searches only zero-width paths, with
class profiles disabled after full construction. Normalized Unicode starts,
sticky behavior and later starts required by outside gates remain intact. Deep
groups, decimal scanning, storage and clone/drop stay flat and source-sized.
Unknown terms even under erased/skipped/never scopes, consuming outside siblings
and patterns requiring a different zero-ancestor selection retain separate
proofs. Earlier API domains, work and captures remain unchanged. Runtime selects
this captured-scope erased-count proof after the bare quantified root proof. Its
dedicated body retains conditional outside points, mandatory enclosing ranges,
every undefined inner slot, source/flags, named aliases, indices, lastIndex and
String consumers through cloning and collection.


Core separately proves captured outside predicate/empty scopes around a
zero-count truth root with bounded zero/one class-path counts. The existing flat
nearest-zero-ancestor scan retains the original capture prefix. The complete
bounded-path body and both branches validate before outside projection reserves
every original inner slot. All erased captures remain undefined while outside
conditional points and mandatory enclosing ranges can participate (ECMA-262
edition 17 22.2.2.3, 22.2.2.7).

Disabled class profiles keep matching on zero-width outside continuations.
Normalized Unicode starts, sticky bounds and later starts required by outside
gates remain intact. Traversal, storage and clone/drop stay flat and
source-sized. Unknown terms even under erased/skipped/never scopes, larger or
unbounded inner counts, consuming outside siblings and patterns requiring a
different zero-ancestor selection retain separate proofs. Earlier API domains,
work and captures remain unchanged. Runtime selects this bounded erased-path
outside-scope proof after larger/unbounded erased paths. Its dedicated body
retains conditional outside points, mandatory enclosing ranges, every undefined
inner slot, source/flags, named aliases, indices, lastIndex and String consumers
through cloning and collection.


Core separately proves captured outside predicate/empty scopes around an optional
truth root with bounded zero/one class-path counts. The flat nearest-optional
ancestor scan retains the original capture prefix; the complete bounded-path
body and both branches validate before outside projection reserves all original
inner slots. Nonempty outer iterations force every bounded inner path to run
once, including lazy class counts. Zero upper bounds disable consumption only
after full validation (ECMA-262 edition 17 22.2.2.3–4, 22.2.2.7).

The existing live outside continuation kernel preserves forced lazy consumption,
shorter-member retries and earliest complete starts. Outside conditional points
and mandatory enclosing ranges can participate while every skipped inner slot
stays undefined. Normalized Unicode starts, sticky behavior, flat traversal and
source-sized storage/clone/drop remain intact. Unknown terms even under
zero/lazy/skipped/never paths, larger or unbounded inner counts, consuming outside
siblings and patterns requiring a different optional-ancestor selection retain
separate proofs. Earlier API domains, work and captures remain unchanged.
Runtime selects this bounded optional-path outside-scope proof after bounded
erased paths. Its dedicated body retains forced lazy consumption, zero-body
absence, outside participation, original lexical slots, source/flags, named
aliases, indices, lastIndex and String consumers through cloning and collection.


Core separately proves captured outside scopes around an optional bounded-path
root when nearer optional class scopes contain no structural choice. A flat
balanced scan propagates a choice flag through ordinary groups while keeping
class data opaque. It skips those nearer scopes, retains the original lexical
capture prefix and selects the nearest optional class ancestor containing a
choice. Full bounded-path body compilation and outside projection remain
mandatory; a structural flag alone never establishes a matching proof.

The constructor requires a different atom from the preceding nearest-ancestor
proof, preserving that API's domains and charged work. Both scans, capture
projection, storage and clone/drop remain flat and source-sized. Nonempty outer
iterations force bounded inner groups to participate once; skipped/zero-bound
bodies retain undefined slots while outside points and enclosing ranges can
participate (ECMA-262 edition 17 22.2.2.3–4, 22.2.2.7). Unknown terms even under
zero/lazy/skipped/never scopes, consuming outside siblings and other ancestor
selection cases retain separate proofs. Runtime selects this choice-guided
bounded-group proof after the preceding optional-path proof. Its dedicated body
retains inner group participation, outside roles, original lexical slots,
source/flags, named aliases, indices, lastIndex and String consumers through
cloning and collection.


Core separately proves captured outside scopes around a zero-count root above
nearer choice-free zero-count class groups. A flat choice-flag scan skips those
nearer scopes while retaining the original capture prefix and opaque class data.
It requires a different atom from the preceding zero-ancestor proof. The complete
body validates through the existing larger/unbounded or bounded zero-count proof
before outside projection reserves every original inner slot.

Every body capture remains undefined; outside conditional points and mandatory
enclosing ranges can participate. Disabled class profiles prevent body queries
while outside gates retain normalized Unicode starts, sticky bounds and later
complete starts (ECMA-262 edition 17 22.2.2.3, 22.2.2.7). Traversal, decimal count
scanning, storage and clone/drop remain flat and source-sized. Unknown terms even
under erased/skipped/never scopes, consuming outside siblings and other ancestor
selection cases retain separate proofs. Earlier API domains, charged work and
captures remain unchanged. Runtime selects this choice-guided zero-root proof
after the corresponding optional-root proof. Its dedicated body preserves every
undefined inner slot, conditional outside points, mandatory enclosing ranges,
source/flags, named aliases, indices, lastIndex and String consumers through
cloning and collection.


A final whole-Pattern zero-group fallback relies on the parser's completed
Pattern and flag validation, including early errors. It recognizes only an
entire group followed by an exact zero upper bound (`{0}` or `{0,0}`, with
padded decimal zeros and an optional lazy marker). RepeatMatcher invokes its
continuation immediately when max is zero, without executing the body
(ECMA-262 edition 17 22.2.2.3.1); every lexical capture therefore remains undefined
(22.2.7.2). Validated lookarounds, references, modifiers and Unicode class bodies
need no matching implementation within this erased group. Invalid syntax,
properties, names, references and quantifiers still fail full parser validation.

The Core API explicitly requires validated source and normalized capture heads.
An iterative structural scan counts captures, skips escapes and distinguishes
ordinary classes from nested Unicode Sets classes. Construction charges
source-sized work before scanning; matching charges constant work before input
inspection and uses the shared Unicode leading-boundary rule. Storage and
clone/drop are constant. Runtime tries this fallback after all older successful
plans, retains the complete original capture layout, and uses the usual named
aliases, indices, lastIndex and String operations. The fallback does not erase
outside terms, a larger repetition bound or only part of a Pattern; those
patterns still require their own matching proof. The preceding specialized APIs
retain their existing acceptance domains and work accounting.


Core also exposes a separate enclosing-group proof for the whole-pattern
zero-group plan. It removes only complete, unquantified ordinary capturing and
noncapturing wrappers, using a mode-aware flat delimiter scan. First-closure
tracking prevents later siblings from replacing a leading group's boundary.
The retained body must pass the original whole-pattern zero-group proof. The
returned enclosing-capture count precedes every retained lexical slot: enclosing
captures select the empty whole-match span, while all captures inside the
zero-count group remain undefined (22.2.2.2, 22.2.2.3.1, 22.2.2.7). The API
charges source-sized scanning and temporary body storage before allocation;
matching, retained storage and clone/drop stay constant. Outside terms, choices
and quantified wrappers remain outside this additional proof.

Runtime selects the enclosing zero-group proof after the whole-pattern zero
fallback. The common match layout defines the returned capture prefix at the
empty whole-match span and retains undefined body slots. Named aliases, indices,
lastIndex, String consumers, cloning and collection use the existing machinery.
Original Pattern validation preserves enclosing reference bindings before
projection; unexecuted references are never revalidated in an isolated,
renumbered body.


A separate whole-class zero-count plan accepts a complete validated character
class followed by an exact zero bound, including padded/lazy forms. It requires
full Pattern and flag validation before its structural scan. Character classes
have no captures, and RepeatMatcher with max zero never evaluates membership
(22.2.2.3.1), including string-valued properties whose matching data is not yet
implemented. Invalid negation, property names, set operations and string escapes
still fail the parser's ordinary grammar and early-error checks.

The flat scan distinguishes ordinary/u classes from nested v classes and skips
escaped delimiters. Source work is charged before scanning; the retained plan
and clone/drop are constant-sized, and matching charges constant actual work
before checking the normalized start. Runtime selects this fallback after the
existing zero-group proofs, retaining all earlier successful plans. The common
match result supplies source/flags, indices, lastIndex and String consumers with
no capture slots. Continuations, nonzero upper bounds and enclosing groups
remain outside this particular proof.


Core separately proves mandatory ordinary enclosing groups around a whole
zero-count class. The group and class proofs share the same flat, mode-aware
first-closure scan. The class proof returns the enclosing capture prefix; every
capture selects the empty match span because the retained class has no captures
(22.2.2.7). Full original Pattern validation precedes normalized-head projection.
Construction charges source scanning and temporary body storage before copying;
matching and retained storage remain constant. Earlier group and class APIs
retain their exact acceptance domains, results and work callbacks.

Runtime selects the separate enclosing-class zero proof after the bare-class
fallback. Its existing class-match body uses the common enclosing capture
prefix, so every capture selects the empty whole-match range. Original named
and escaped-name aliases, index-array identity, source/flags, lastIndex and
String consumers remain on the standard result path through cloning and
collection. Class membership is never evaluated by the retained matching plan.


A separate Unicode property-escape constructor on the zero-class plan proves a
complete `\p{...}` or `\P{...}` atom followed by an exact zero bound. Full
original u/v grammar, property-name and string-property validation remains a
prerequisite. Property names stay opaque to this proof, and no matching data is
queried by its retained plan (22.2.2.3.1). Runtime tries the constructor only for
Unicode patterns after the existing zero-class proofs. The original bracket
class APIs retain their domains and work. Enclosing groups, alternatives,
continuations and nonzero upper bounds remain outside this additional proof.


Core separately proves complete mandatory ordinary enclosing groups around a
zero-count Unicode property escape. It reuses the flat mode-aware first-closure
scan, then the complete property-escape proof. Full original u/v validation and
normalized capture heads remain prerequisites. Every returned capture belongs
to the mandatory enclosing prefix and selects the empty match span (22.2.2.7).
Construction charges source scanning and temporary body storage before copying;
matching and retained storage remain constant. Existing group, bracket-class
and bare-property proof domains and work are unchanged.

Runtime selects the enclosing-property proof after the bare-property fallback.
The existing class-match body and common capture-prefix layout define every
enclosing capture at the empty whole-match span. Original named and escaped-name
aliases, index-array identity, source/flags, lastIndex and String consumers
remain on the standard result path through cloning and collection. Property
membership is never evaluated by the retained matching plan.


Core also proves a single class/property atom within a complete quantified group
spine. The spine may contain ordinary capturing/noncapturing and ims modifier
groups. Full original grammar/flag validation and normalized capture heads are
required. An exact zero upper bound on the atom or any ancestor proves that the
entire Pattern succeeds empty without querying membership. Modifier flags have
no observable matching effect on the erased body.

Required empty repetitions define their captures; optional empty repetitions
fail their zero-progress continuation and roll their capture writes back
(22.2.2.3.1). Since all groups lie on one spine, every capture before the first
optional group is defined empty and every subsequent capture is undefined. The
plan stores only this prefix boundary and the full lexical count. Bounds are
scanned symbolically without expansion, including oversized positive minima.
Temporary frames and source work are charged before the flat scan; retained
storage, clone/drop and matching remain constant. Assertions, alternatives,
consuming siblings and expressions without a proven zero bound require other
proofs. Earlier APIs and their work are unchanged.

Runtime selects the quantified zero-spine proof after the earlier zero-atom
proofs. Its dedicated body carries the complete original lexical count and
queries Core for each defined or undefined capture. The common result path
preserves own undefined slots, named aliases, index-array identity, source/flags,
lastIndex and String consumers through cloning and collection. Actual matching
work is charged by the plan before input access.


The zero-spine plan has a separate arbitrary-group constructor. A zero-count
group may contain any fully validated body, including lookarounds, references,
alternatives and matching data not yet implemented. A flat delimiter scan
reserves every original lexical slot and records each leading group's first
closure. A linear suffix pass admits the candidate only when its outside
ancestors form a complete quantified ordinary/modifier spine, without other
terms or alternatives. It avoids rescanning suffixes for every nested zero
group. The erased group's own and inner slots remain undefined; outside
required captures are empty, with optional ancestors clearing their suffix
(22.2.2.3.1, 22.2.2.7). Source/frame work is charged before allocation, and the
retained capture boundary and matching work remain constant. Existing APIs and
matching behavior are unchanged.

Runtime selects the arbitrary-group spine proof after the class/property spine
proof. It reuses the complete-layout spine body, retaining the original lexical
count, defined outside prefix and erased inside suffix. Named aliases (including
duplicate names in erased alternatives), references inside erased bodies,
indices, lastIndex and String consumers keep the standard result path through
cloning and collection. No matching body data is queried by the retained plan.


A separate zero-count sequence plan composes at least two complete top-level
class/property or arbitrary-group spines. Each constituent proof matches empty
without input conditions or executed references, so concatenation preserves
each term's capture state independently (22.2.2.2, 22.2.2.3.1). A flat mode-aware
term scan preserves escaped and class delimiters; projected terms retain their
original validated bindings, including references to other lexical slots inside
erased bodies. No projected term is validated or rebound in isolation.

The plan appends each term's defined prefix and undefined suffix to a lexical
capture mask. Unlike a single spine, a sequence can alternate these roles any
number of times. Construction, retained storage and cloning are source-sized;
matching and individual capture queries are constant. Source and temporary
storage work is charged before allocation, and actual matching charges before
input access. Shared enclosing groups, choices, assertions and unproved
consuming terms require additional proofs. Earlier APIs are unchanged.


Runtime selects the zero-erased sequence proof after the single-spine proofs,
using the complete original normalized capture source. Its dedicated body
queries the lexical mask for each slot; the shared result path preserves named
and numbered values, own undefined slots, index aliases, source/flags and
lastIndex. String consumers, cloning and collection retain the same mask. Actual
matching work is charged before input access, without a precharged input scan.
Original early errors are validated before any projection or erasure.


The lexical-mask plan has a separate first-alternative constructor. At the root
of a complete Pattern, an unconditional empty first alternative invokes the
final successful continuation, so no later alternative can run (22.2.2.2). The
first alternative may be empty or a proven zero-erased spine/sequence; later
arms may contain any fully validated body. A full flat scan finds the first
root separator and counts every original capture while respecting escapes,
groups and mode-specific classes. The first arm retains its capture mask and
all later arms append undefined slots. Original bindings remain unchanged,
including references across alternative scopes inside erased bodies. Source
and temporary-storage work precede allocation; retained storage and clone/drop
are source-sized, with constant matching work and capture queries. Conditional
or consuming first arms and choices inside shared enclosing groups require
other proofs. Earlier constructors and Runtime behavior are unchanged.


Runtime selects the unconditional first-alternative proof after the erased
sequence proof. The existing lexical-mask body preserves selected first-arm
values, undefined unselected slots and duplicate-name aliases across root
alternatives. Full original syntax and named-reference validation precedes the
proof, including early errors in arms that never execute. String consumers,
cloning, collection and actual-work accounting share the sequence result path.


A separate enclosing constructor applies the shared flat ordinary-group scope
proof to a complete zero sequence or unconditional first choice. Every
mandatory enclosing capture is defined empty (22.2.2.7), preceding the body's
interleaved capture mask. The plan stores the complete original layout, so
Runtime need not shift or reconstruct lexical slots. Projected bodies keep
original reference bindings, including enclosing targets inside erased terms.
Source/copy/mask work is charged before allocation, and the shared first-closure
scan excludes outside terms and later siblings without rescanning nested
suffixes. Construction, clone/drop and retained storage are source-sized;
matching and capture queries remain constant. Quantified/modifier enclosing
groups require additional proofs. Existing constructors are unchanged.


Runtime selects the mandatory enclosing sequence/choice proof after the root
proofs. Its complete-layout lexical mask includes the defined enclosing prefix,
so named aliases and indices keep original slot numbers without an additional
Runtime offset. References to enclosing groups inside erased terms are never
revalidated in isolation. String consumers, cloning, collection and actual-work
accounting use the existing sequence result path.


The lexical-mask plan also proves quantified ordinary/ims modifier spines around
a complete empty sequence or choice whose every alternative is proved empty.
A flat delimiter scan records each
leading group's first closure, and a linear suffix pass selects the deepest
complete outside spine. Required empty iterations retain the body's mask; the
first optional outside group rolls all descendant captures back (22.2.2.3.1).
The complete mask combines the defined outside prefix, any cleared suffix and
the retained inner mask. Modifier flags cannot change these unconditional empty
results, and decimal bounds stay symbolic without expanding required empty
iterations. Source/frame work is charged before allocation; construction,
storage and clone/drop are source-sized, while matching and capture queries
remain constant. Outside siblings, conditions and unproved consuming bodies
require other proofs. Earlier APIs and their work are unchanged.


A root first-alternative proof depends on its final successful continuation.
Inside RepeatMatcher, a rejected empty iteration can backtrack into later arms:
`(|a)+` consumes `a` on that input despite its empty first arm. The quantified
enclosing constructor therefore checks every later arm independently as an
erased spine/sequence or an empty arm before retaining the first arm's mask.
All backtracking paths then stay at the same input position, and optional
attempts roll every capture write back. Root and mandatory-enclosing first
choices still use their final-success proof and may skip arbitrary later bodies.


Runtime selects the quantified enclosing proof after the mandatory enclosing
proof and reuses the complete-layout lexical-mask body. Required outside
captures, optional rollback, modifier scopes and safe empty alternatives retain
original named/index aliases through String consumers, cloning and collection.
Choices with unproved later arms retain unsupported matching, including cases
where a failed empty iteration could backtrack into consuming membership.
Original early errors and opt-in host aborts still precede matching effects.


A separate tree constructor composes empty groups, sequences, choices and
quantifiers at arbitrary nesting depth. Every active alternative must be
independently empty, so failed empty iterations cannot expose consuming
backtracking paths (22.2.2.2, 22.2.2.3.1). Exact-zero literal, dot, class,
property and known short-escape atoms erase their matching work. Exact-zero
groups erase arbitrary validated bodies, including unsupported membership,
assertions and references. Active assertions, references and unproved escapes
remain outside this proof. Named capture heads must already be normalized,
with complete original grammar/flag/binding validation retained by the caller.

A flat frame stack reduces the source once. Required captures start defined;
optional or erased groups and later choice arms mark lexical capture intervals
undefined (22.2.2.7). Each interval records its exclusive end at its starting
slot. One prefix-maximum pass applies all overlapping clears without repeatedly
walking descendants. Decimal bounds stay symbolic. Source/frame work is charged
before allocation; construction, retained storage and clone/drop are
source-sized. The existing lexical-mask plan supplies constant matching work
and capture queries. Earlier constructors and their work remain unchanged.


Runtime selects the composable tree proof after the earlier enclosing proofs.
The complete lexical mask preserves live captures before and after erased
subtrees, selected branch values and undefined unselected slots without
rebinding original references. Named/index aliases, String consumers, cloning
and collection use the existing sequence result path. Original syntax errors
still precede erasure; actual matching work is charged before input access.


The tree reducer has a separate constructor for positive lookahead/lookbehind
whose complete bodies always succeed empty (22.2.2.4). Such assertions export
their capture masks without input access. Every active alternative still needs
an independent empty proof. Reversing zero-width term evaluation in lookbehind
preserves the mask: admitted terms have no consuming work or active reference
dependencies, and empty repetition rollback keeps the same capture roles in
either direction. Negative assertions and unproved consuming or conditional
bodies remain unsupported by this proof.

The earlier tree constructor retains its original assertion exclusions and
source work. Both constructors use the same flat stack, capture-clear markers
and constant-work retained plan; the new constructor explicitly enables only
proved positive assertions. Source/frame work precedes allocation, and deeply
nested assertions do not add native recursion.


Runtime selects the positive empty-tree proof after the earlier tree proof,
retaining the complete original lexical capture layout. Forward/reverse
assertion imports, optional rollback and unselected slots use the shared named
and numbered result path, including index-array aliases. Erased references
retain their original bindings, and original early errors precede proof
selection. String consumers, cloning, collection and actual-work accounting
reuse the existing sequence body.


A separate escaped-atom tree constructor recognizes complete hexadecimal and
non-surrogate Unicode literal escapes before applying an exact-zero bound
(22.2.1, 22.2.2.3.1). Fixed hexadecimal/Unicode spellings retain their complete
source widths; Unicode braced spellings require u/v and scan leading zeros
without expansion. Encoded delimiters remain literal atoms rather than group
or choice syntax. Surrogate escapes remain unproved because adjacent escaped
code units can combine into one Unicode atom. Erased enclosing groups can
still suppress such bodies without reinterpreting them.

The new constructor enables the positive empty-tree proof as well. Earlier
tree constructors retain their escape exclusions and work. Escape boundaries
are scanned only during construction, without matching data; the retained
lexical mask, capture-clear markers and actual matching work are unchanged.


Runtime selects the escaped-atom tree proof after the earlier positive-tree
proof. Complete escape spellings retain their original source text and lexical
capture layout. Encoded delimiters, optional rollback, assertion imports and
erased reference bindings use the same named/index result path through String
consumers, cloning and collection. Original escape syntax and binding errors
precede erasure; actual matching work is charged before input access.


A separate surrogate-aware tree constructor applies the edition-17 atom
boundaries for escaped UTF-16 code units (22.2.1). Adjacent fixed lead/trail
Unicode escapes combine into one atom only in u/v mode. Non-Unicode fixed
escapes and braced or mixed literal/escaped forms keep separate boundaries.
The proof consumes a fixed pair together before reading its following count;
it never erases an unquantified leading atom in a separate spelling.

Individual surrogate atoms can also be proved empty under an exact-zero bound,
including inside positive assertions. Earlier constructors retain their
surrogate exclusions and work. Only construction reads the escape spellings;
the lexical capture mask and constant-work matcher need no surrogate matching
data. Flat capture clearing and source-sized storage are unchanged.


Runtime selects the surrogate-aware tree proof after the earlier escaped-tree
proof. Atom boundaries follow the original Unicode mode before zero-count
erasure, while complete source spellings and capture numbering remain intact.
Fixed-pair/lone/braced atoms, optional rollback and assertion imports use the
shared named/index result path through String consumers, cloning and collection.
Original syntax and binding errors still precede proof selection.


A separate impossible-match plan proves a complete root sequence containing a
direct negative lookahead/lookbehind whose body independently always succeeds
empty (22.2.2.4). That assertion must fail at every position; other validated
root terms cannot rescue the sequence. Root alternatives and negative
assertions inside enclosing groups require other proofs. Original grammar,
flags and named bindings must be validated before this proof.

A flat mode-aware scan counts every original capture and locates direct root
assertion bodies. Projected bodies keep their original bindings and use the
existing empty-tree proof. Once one assertion proves failure, remaining source
is still scanned for capture inventory and root choices, without proving
irrelevant bodies. Source/copy work precedes allocation. Construction is
source-sized; retained storage, clone/drop and actual no-match work are constant.
Matching charges before any possible input access and needs no matching data
from otherwise consuming or unsupported surrounding terms.


Runtime selects the impossible-match proof after the empty-tree proofs, using
the full original normalized capture source. Its dedicated body retains every
lexical slot for constructor consistency, charges one unit of actual matching
work and returns no match without input access. The normal execution path
keeps input/index coercion order, global/sticky lastIndex resets, source/flags
and String consumer behavior. Clones and collection retain the constant-size
plan; original syntax errors still precede proof selection.


The impossible plan has a separate mandatory-enclosing constructor. The shared
flat ordinary-group scope proof projects a complete impossible root sequence
without rebinding its original references. Every enclosing capture is added to
the original lexical inventory even though no successful result can expose it.
Quantified/modifier wrappers, outside terms and outer choices require other
proofs. Source/copy work precedes allocation; the retained plan and no-match work
remain constant, with no recursive wrapper processing.


Runtime selects the mandatory-enclosing impossible proof after the direct root
proof. The returned inventory includes every original outside and inside slot,
so constructor capture checks use the full source without an extra offset.
Erased references to outside/later names keep their original bindings. Shared
execution preserves source/flags, lastIndex, String consumers, cloning and
collection while the retained matcher still performs constant no-match work.


A separate impossible-choice constructor admits a root disjunction only when
every alternative independently proves no match. A flat mode-aware scan splits
root separators while retaining escaped, class and nested choice boundaries.
Each arm uses the direct or mandatory-enclosing proof with its original binding
context; empty, possible or unproved arms reject the constructor. Original
capture inventories are summed without renumbering references. Source/copy
work precedes allocation, each projected arm is scanned independently and total
construction stays source-sized. Retained storage and actual no-match work
remain constant; earlier constructors and their work are unchanged.


Runtime selects the impossible-choice proof after the direct/enclosing proofs.
Every root arm must independently fail before the constant no-match body is
selected. Full capture inventories preserve cross-arm and duplicate-name
bindings for original validation; no arm's result is fabricated or exposed.
Shared execution keeps lastIndex, source/flags, String consumers, cloning and
collection behavior. Original errors in later arms still precede proof selection.


A separate impossible constructor accepts complete enclosing ordinary/modifier
group spines only when every outside minimum is positive (22.2.2.3.1,
22.2.2.4). The first required body attempt must fail; any optional wrapper
rejects the proof. Counts remain symbolic, including very large minima.
The shared flat scope scan projects a direct impossible sequence or a choice
whose every arm independently fails, preserving original reference bindings
and adding all outside captures. Source/frame/copy work precedes allocation;
construction stays source-sized and the retained no-match plan is constant.


Runtime selects the required-enclosing impossible proof after the root-choice
proof, using the complete normalized original source. Positive-minimum counts
and ordinary/modifier wrappers keep their original lexical inventories and
bindings. The constant no-match body continues to use normal lastIndex,
source/flags, String consumer, clone and collection behavior.


A separate positive-assertion impossibility constructor proves root sequences
containing a direct positive lookahead/lookbehind whose complete operand is
independently impossible (22.2.2.4). The shared flat scanner retains every
original capture and rejects root choices. Projected operands preserve their
original bindings and use only the established negative-empty direct,
enclosing, root-choice or required-spine proofs. This finite proof chain never
re-enters the positive constructor. Callback type erasure at the private proof
boundary prevents recursive generic expansion; earlier negative construction
work and domains stay unchanged. Source/copy work precedes allocation, total
construction is source-sized and the retained no-match plan stays constant.


Runtime selects the positive-impossible-assertion proof after the earlier
negative-empty impossibility proofs. Both positive assertion directions use
the full normalized original capture source and the existing constant no-match
body. Original errors, lastIndex, String consumers, cloning and collection keep
the shared execution behavior.


A separate empty-match constructor admits a whole negative lookahead/lookbehind
whose complete operand independently proves impossible (22.2.2.4). Both
negative directions succeed empty without exporting any operand capture. The
finite operand chain uses the established negative-empty and positive-impossible
constructors and never calls this new empty proof. Original validation and
normalized heads are required; siblings, outside groups and quantifiers require
other proofs. Source/copy work precedes allocation, and a separate capture-mask
charge precedes allocating undefined slots. The existing empty matcher preserves
Unicode leading boundaries, constant actual work and lexical capture indexing.


The empty negative-impossible proof has a separate enclosing ordinary/modifier
spine constructor. The shared flat scope scan keeps original bindings and
symbolic counts. Each inner attempt stays empty, so required enclosing captures
are defined empty and optional repetitions roll descendant writes back
(22.2.2.3.1, 22.2.2.7). All assertion-operand slots remain undefined. Source,
projection and full-mask work precede allocation; construction and retained
storage are source-sized, without recursive wrapper processing.


Runtime selects the whole negative-impossible empty proof after the existing
empty-tree proofs. It uses the complete normalized original capture source,
retains every operand slot as undefined and reuses normal empty execution.
Unicode boundaries, global empty advancement by String consumers, callback
arguments, split captures, source/flags, lastIndex, cloning and collection stay
on the shared paths. Original errors still precede proof selection.


A separate root-choice empty proof accepts a first arm proved by the whole or
enclosing negative-impossible constructor. With the final successful
continuation that arm succeeds empty, so later validated arms are skipped
(22.2.2.2). The first arm's mask is preserved and later lexical slots are
undefined, with original bindings and no renumbering. This proof is restricted
to the root and must not be used as a repetition operand: an empty iteration's
failing continuation could backtrack into a consuming later arm. Source/copy
and full-mask work precede allocation; construction and storage are source-sized.


Runtime selects the enclosing negative-impossible empty proof after the whole
assertion proof. It uses the complete original capture source and exports
required empty enclosing captures while optional repetitions retain undefined
descendant slots. The shared empty execution path preserves Unicode boundaries,
source/flags, lastIndex, String callbacks and split captures, cloning and collection.


A separate empty-sequence constructor composes negative-impossible assertions
or spines with independently empty tree terms (22.2.2.2, 22.2.2.4). Each complete
root term is proved independently with its original binding context; at least
one must use the negative-impossible proof. Root choices and unproved consuming
terms reject this constructor. A shared safe-atom boundary helper reuses the
existing surrogate-aware tree scanner without changing earlier tree modes.
Interleaved masks remain in lexical order, including undefined assertion slots.
Source/copy and mask work precede allocation; disjoint segments keep construction
and storage source-sized. Actual empty matching and individual capture queries
reuse the existing constant-work implementation.


Runtime selects the first-negative-impossible root-choice proof after the whole
and enclosing empty assertion proofs. It retains the first arm's capture mask
and every skipped later lexical slot, using the complete original normalized
source. Duplicate names across original alternatives and erased forward bindings
keep their original validation context. Shared empty execution preserves Unicode
boundaries, lastIndex, String callback/split captures, cloning and collection.


A separate enclosing-sequence constructor applies the flat ordinary/modifier
scope proof to independently empty negative-impossible sequences. Every body
term stays empty, so required repeat continuations cannot expose a consuming
alternative. Required enclosing captures are defined empty; optional repeats
roll the complete descendant mask back (22.2.2.3.1, 22.2.2.7). Original bindings,
interleaved body masks and symbolic bounds are preserved. Scope, projection and
full-mask work precede allocation; construction and retained storage remain
source-sized without recursive wrapper processing.


Runtime selects the negative-impossible empty-sequence proof after the whole,
enclosing and first-root-choice empty assertion proofs. Complete original
capture source preserves interleaved defined/undefined slots and erased forward
bindings. The shared empty path keeps Unicode boundaries, source/flags,
lastIndex, String callback/split captures, cloning and collection behavior.


A separate composite-negative impossibility constructor proves root sequences
blocked by a negative lookahead/lookbehind over the newer empty assertion,
spine, root-choice or sequence proofs (22.2.2.4). A complete assertion operand
uses its final successful continuation, so the first-empty-root-arm proof is
valid there; outer negative failure does not retry that successful operand.
Original bindings and full capture inventories remain intact. The shared flat
scanner has separate modes preserving earlier domains and work. Its finite
operand chain never re-enters the new proof; callback type erasure keeps the
compiler boundary finite as well. Source/copy work precedes allocation,
construction stays source-sized and retained no-match storage stays constant.


Runtime selects the enclosing negative-impossible empty-sequence proof after
the direct sequence proof. Full original normalized source preserves enclosing
and interleaved body capture roles: required wrappers export empty captures,
while optional repetitions clear descendant writes. Shared empty execution
retains Unicode boundaries, source/flags, lastIndex, String callback/split
captures, cloning and collection behavior.


A separate wrapped-composite impossibility constructor applies the shared flat
ordinary/modifier scope scan to a complete composite-negative failing sequence.
Every enclosing minimum must be positive; an optional wrapper could skip the
failing body (22.2.2.3.1). Bounds stay symbolic, references retain their original
bindings and every enclosing capture joins the full lexical inventory.
Source/frame/copy work precedes allocation. Construction stays source-sized,
without recursive wrapper processing; retained storage and actual no-match
work reuse the constant impossible plan. Earlier constructor work is unchanged.


Runtime selects the composite-negative impossibility proof after the earlier
negative-empty and positive-impossible proofs. Complete original normalized
capture source preserves all nested, cross-arm and duplicate-name bindings for
validation. The shared constant no-match body keeps source/flags, lastIndex,
String consumers, cloning and collection behavior. Original errors still
precede proof selection, including errors in otherwise skipped operand arms.


A separate composite-alternatives impossibility constructor admits a root
choice only when every arm independently proves a direct or required-enclosing
composite-negative failure (22.2.2.2). Empty, possible or unproved arms reject
the proof. The earlier negative-empty alternative constructor and this new
constructor share a flat mode-aware separator scanner, with distinct arm-proof
modes preserving earlier domains and charges. Every projected arm keeps its
original binding context and contributes its full lexical capture inventory.
Source/copy work precedes allocation; disjoint arms keep construction
source-sized, while retained storage and actual no-match work stay constant.


Runtime selects required ordinary/modifier spines around composite-negative
failures after the direct composite-negative proof. Complete original normalized
capture source retains every outside/inside slot and erased enclosing binding.
The constant no-match body preserves source/flags, lastIndex, String consumers,
cloning and collection behavior. Original early errors still precede selection.


A separate enclosing-composite-alternatives constructor applies the flat
ordinary/modifier scope proof to a complete impossible composite root choice.
Every enclosing minimum must be positive, and every body arm must independently
prove impossible (22.2.2.2, 22.2.2.3.1). Optional wrappers, outside siblings and
unproved body arms reject the proof. Original bindings and all outside/inside
capture slots remain intact, with symbolic bounds. Source/frame/copy work
precedes allocation; the finite body proof never recursively processes outside
wrappers. Construction stays source-sized and retained no-match storage constant.


Runtime selects impossible composite root alternatives after the direct and
required-enclosing composite-negative proofs. Every arm must independently
fail. Full original normalized source preserves cross-arm, duplicate-name and
erased forward bindings for validation. The shared constant no-match body
retains source/flags, lastIndex, String consumers, cloning and collection.


A separate optional-impossible empty proof admits complete ordinary/modifier
spines with at least one zero-minimum wrapper around an independently impossible
body (22.2.2.3.1). Every attempted body fails, so the first optional wrapper can
skip it without exporting operand captures. Required enclosing captures before
that wrapper remain defined empty; all later wrapper/body slots are undefined.
The finite operand chain uses the existing impossible constructors and never
calls this new empty proof. Original bindings and symbolic bounds stay intact.
Source/frame/copy and full-mask work precedes allocation; construction and
retained storage are source-sized. Shared empty matching keeps Unicode
boundaries, constant actual work and lexical capture indexing.


Runtime selects complete required ordinary/modifier spines around impossible
composite alternatives after the direct root-alternative proof. Full original
normalized source preserves every enclosing/arm slot, duplicate-name bindings
and erased outside/cross-arm references for validation. Shared constant no-match
execution keeps source/flags, lastIndex, String consumers, cloning and collection.


Runtime selects optional spines over impossible operands after the existing
empty negative-impossible sequence/spine proofs. Complete original normalized
source preserves required outside captures and undefined skipped descendants.
Normal empty execution retains Unicode boundaries, source/flags, lastIndex,
global String advancement, named indices, replacement/split captures, cloning
and collection. Original early errors still precede proof selection.


A separate optional-impossible sequence constructor composes complete optional
impossible spines with independently empty tree or negative-impossible terms.
At least one term must use the optional-impossible proof; root choices and
consuming/unproved terms reject the constructor (22.2.2.2, 22.2.2.3.1). Original
bindings and interleaved masks remain in lexical order. A shared flat term
scanner has separate earlier-negative and new-optional modes, preserving earlier
domains and charges. Source, copy and mask work precedes allocation; disjoint
terms keep construction and storage source-sized. Actual empty matching keeps
the existing constant work and Unicode boundary behavior.


A separate enclosing optional-impossible sequence proof applies the flat
ordinary/modifier scope scan to a body whose every term independently stays
empty. Required wrappers retain the body's interleaved mask; optional repeats
roll all descendant writes back (22.2.2.3.1, 22.2.2.7). Original bindings and
symbolic counts remain intact. Source/frame/copy and full-mask work precede
allocation; construction and storage stay source-sized without recursive
outside-wrapper processing. Earlier constructors and their work are unchanged.


Runtime selects optional-impossible empty sequences after the whole optional
spine proof, using the complete normalized original capture source. Interleaved
required and skipped slots, including erased forward references, retain their
original roles. Shared empty execution preserves Unicode boundaries,
source/flags, lastIndex, String callback/split captures, cloning and collection.


A separate root-choice empty proof accepts a first arm proved by the optional,
optional-sequence or enclosing optional-sequence constructor. With the final
successful continuation that arm always succeeds empty, so later validated
arms are skipped (22.2.2.2). The first mask is preserved and every later lexical
slot is undefined, with original reference bindings. This root-only proof must
not be used as a repetition operand: an empty iteration's failing continuation
could backtrack into a consuming later arm. Source/copy and full-mask work
precedes allocation; construction and retained storage remain source-sized.


Runtime selects ordinary/modifier spines around optional-impossible empty
sequences after the direct sequence proof. Complete original normalized capture
source preserves enclosing and interleaved descendant roles: required repeats
retain the body mask and optional repeats clear it. Shared empty execution keeps
Unicode boundaries, source/flags, lastIndex, String callback/split captures,
cloning and collection behavior.


A separate positive-assertion empty proof admits a whole lookahead/lookbehind
over independently empty optional-impossible forms (22.2.2.4). The operand uses
the optional, sequence, enclosing-sequence or first-root-choice proof with its
original bindings. Complete assertion operands have a final successful
continuation, so the first-empty-choice proof is valid there. Both directions
export the operand's empty/undefined mask without consuming input. Siblings and
outside wrappers require other proofs. Source/copy work precedes allocation;
the finite operand chain never calls this new constructor. Retained masks,
Unicode boundaries and actual matching reuse the existing empty plan.


Runtime selects root choices starting with independently empty optional-impossible
arms after the whole/sequence/enclosing empty optional proofs. The first arm's
mask is retained and all skipped later slots remain undefined, using the complete
original normalized source. Duplicate names select the participating first-arm
capture even when it is empty. Shared execution preserves Unicode boundaries,
source/flags, lastIndex, String callback/split captures, cloning and collection.


A separate negative-nullable-group proof admits root sequences containing a
negative assertion over a complete ordinary/modifier spine with a zero-minimum
wrapper. Such an operand always has a successful path (22.2.2.3.1), so its
negative assertion must fail at every legal position (22.2.2.4). A greedy operand
may consume input; this is an impossibility proof and must never fabricate an
empty operand result or capture mask. Arbitrary validated matching data inside
the skippable group is unnecessary. The shared flat scanner has a separate mode
preserving earlier domains and work. Source/copy/frame work precedes allocation;
construction is source-sized and retained storage/actual no-match work constant.


Runtime selects whole positive assertions over optional-impossible empty forms
after the first-root-choice optional proof. Complete original normalized source
preserves exported body masks, erased forward bindings and duplicate-name
selection, including participating empty captures. Shared empty execution keeps
Unicode boundaries, source/flags, lastIndex, String callbacks/split captures,
cloning and collection behavior. Original errors still precede proof selection.


A separate negative-nullable-atom proof accepts a complete safe literal,
class/property atom with a zero minimum, optionally inside ordinary/modifier
spines. Its operand always has a successful path (22.2.2.3.1), so its negative
assertion fails (22.2.2.4), even if greedy matching could consume. The established
surrogate-aware tree scanner supplies complete atom boundaries, including
Unicode fixed-surrogate pairs and escaped delimiters. Multiple terms, required
atoms and unproved boundaries reject the proof. This proves impossibility only;
operand matching data and capture values are unnecessary. Source/copy/frame
work precedes allocation; construction is source-sized and the retained plan
and actual no-match work remain constant. Earlier scanner modes are unchanged.


Runtime selects negative assertions over nullable ordinary/modifier group
spines after the existing composite impossibility proofs. Complete original
normalized source retains every capture and incoming reference for validation.
The body may consume greedily, but the guaranteed successful operand path makes
the outside negative assertion fail, so matching data is unnecessary. The
constant no-match body keeps source/flags, lastIndex, String consumers, cloning
and collection. Original errors still precede proof selection.


A separate negative-nullable-sequence proof accepts complete sequences whose
every term independently has a successful zero-length path, optionally inside
ordinary/modifier spines (22.2.2.2, 22.2.2.3.1). Group and safe atom proofs compose
without reading matching data; greedy execution may still consume. The negative
assertion must fail (22.2.2.4), so no operand result or capture mask is fabricated.
The shared flat group/atom boundary scanners split disjoint terms, preserving
original bindings and full inventories. Required/conditional terms and root
choices reject this constructor. Source/frame/copy work precedes allocation;
construction stays source-sized and the retained no-match plan constant.


Runtime selects negative assertions over complete nullable literal/class/property
atoms and enclosing spines after the nullable-group proof. Full original
normalized source preserves every lexical capture and mode-aware atom boundary.
A successful operand path is guaranteed even if greedy matching could consume,
so the constant no-match body avoids otherwise necessary matching data. Shared
execution retains source/flags, lastIndex, String consumers, cloning and collection.


A separate negative-nullable-choice proof accepts complete root operand choices
with at least one empty or independently nullable arm, optionally inside complete
ordinary/modifier spines (22.2.2.2). Earlier arms may consume; one guaranteed
successful path is sufficient to make the negative assertion fail (22.2.2.4).
All original capture slots and incoming bindings remain in the inventory, but
no operand captures are fabricated. A flat mode-aware scan separates root arms
without interpreting class/escaped/nested separators. Source/frame/copy work
precedes allocation; disjoint arm proofs stay source-sized and the retained
no-match plan and actual work remain constant.


Runtime selects negative assertions over complete independently nullable
sequences after the nullable-group and nullable-atom proofs. Complete original
normalized source preserves full inventories and incoming reference bindings.
Every term has a successful path, so matching data and operand capture values
are unnecessary for the outside negative failure. Shared constant no-match
execution retains source/flags, lastIndex, String consumers, cloning and collection.


A separate negative-over-successful-positive proof accepts a complete positive
assertion over a nullable group, atom, sequence or choice, optionally inside
ordinary/modifier spines. Both positive assertion directions always have a
successful operand path and therefore succeed; the outside negative assertion
must fail (22.2.2.4). Operand captures may contain input data and are never
fabricated. The finite proof handles one positive assertion layer without
recursing through arbitrary assertion nesting. Source/frame/copy work precedes
allocation, construction stays source-sized and retained no-match storage and
actual work remain constant. Earlier scanner modes and work are unchanged.


Runtime selects negative assertions over nullable operand choices after the
group, atom and sequence proofs. One empty or independently nullable arm is
sufficient; earlier arms may consume and no capture values are fabricated.
Full original normalized source retains every arm's inventory, incoming
references and duplicate-name bindings for validation. Shared constant no-match
execution keeps source/flags, lastIndex, String consumers, cloning and collection.


A separate wrapped-nullable-negative constructor applies the shared flat scope
proof to required ordinary/modifier spines around nullable-operand negative
failures. Every outside minimum must be positive; optional wrappers could skip
the failing body (22.2.2.3.1). The body uses only the established nullable group,
atom, sequence, choice or successful-positive assertion proofs. Original
bindings, symbolic counts and all outside/inside capture slots remain intact.
Source/frame/copy work precedes allocation; no recursive wrapper processing or
matching data is needed. Retained no-match storage and actual work stay constant.


Runtime selects negative assertions over guaranteed-successful positive
assertions after the nullable choice proof. Complete original normalized source
retains incoming references, duplicate-name bindings and every capture slot.
Variable operand capture values are never exposed because the outside assertion
fails. Shared constant no-match execution keeps source/flags, lastIndex, String
consumers, cloning and collection, with original errors before proof selection.


A separate known-impossible root-choice proof allows established and nullable-
guard impossibility families to mix across arms. Every arm must independently
prove impossible; empty, possible or unproved arms reject the constructor
(22.2.2.2). Earlier alternative constructors retain distinct scanner modes and
their original work/domains. The closed arm-proof helper never re-enters the new
constructor, and callback type erasure bounds compiler expansion through older
alternative proofs. Original projected bindings and full lexical inventories
remain intact. Source/copy work precedes allocation; disjoint arm scans stay
source-sized and the retained no-match plan and actual work remain constant.


Runtime selects required ordinary/modifier spines around nullable-operand
negative failures after the direct nullable-guard proofs. Every outside minimum
must be positive. Complete original normalized source retains all outside/inside
slots and incoming bindings; operand matching data is unnecessary. Shared
constant no-match execution keeps source/flags, lastIndex, String consumers,
cloning and collection, with original errors before proof selection.


A separate required-spine proof wraps complete established impossible operands,
including mixed impossible root choices, in ordinary/modifier groups. Every
outside minimum must be positive (22.2.2.3.1). Optional wrappers and outside
siblings reject this proof. The finite earlier-operand chain preserves original
bindings and the full lexical inventory; symbolic bounds and flat wrapper scans
avoid recursion or count expansion. Source/frame/copy work precedes allocation;
the retained no-match plan and actual work remain constant.


Runtime selects mixed established impossible root alternatives after the earlier
nullable-guard proofs. Every arm must independently fail; complete normalized
source keeps original bindings, duplicate-name alternatives and all lexical
capture slots. Shared constant no-match execution preserves source/flags,
lastIndex, String consumers, cloning and collection; original validation errors
precede proof selection.


A separate optional-spine empty proof accepts complete established impossible
operands, including nullable guards and mixed known choices. At least one outside
zero minimum skips the failing body (22.2.2.3.1). Required captures before the
first optional wrapper stay defined empty; every skipped descendant stays
undefined. The finite earlier-operand chain preserves original bindings and
full lexical slots. Flat scopes and symbolic bounds keep construction source-
sized; source/frame/copy and mask work precede allocation, while matching and
individual capture queries stay constant. Nullable operands themselves never
produce fabricated empty captures.


Runtime selects complete required ordinary/modifier wrappers around established
impossible operands, including mixed known root choices. Full normalized source
keeps incoming bindings and every lexical slot. Shared constant no-match
execution preserves source/flags, lastIndex, String consumers, cloning and
collection; original validation errors precede proof selection.


A separate sequence proof concatenates optional established-impossible spines
with independently empty negative-impossible or empty-tree terms. Every root
term must independently stay empty, and at least one uses the broader optional
proof (22.2.2.2, 22.2.2.4). Original term bindings and interleaved masks survive.
Root choices and consuming/unproved terms reject the constructor. Earlier
sequence modes retain their domains and work. Disjoint source/copy/mask work
precedes allocation, construction and retained masks stay source-sized, and
matching and capture queries remain constant.


A separate enclosing-sequence constructor wraps optional established-impossible
concatenations in complete ordinary/modifier spines. Every body term stays empty,
so required wrappers preserve the body mask and optional repetitions roll back
descendants (22.2.2.3.1, 22.2.2.7). Original bindings and symbolic counts remain
intact. Flat scope/body-copy/full-mask work precedes allocation; retained masks
stay source-sized and actual matching and capture queries remain constant.


Runtime selects optional ordinary/modifier spines over complete established
impossible operands, including nullable guards and mixed choices. Full original
normalized source keeps bindings and every lexical slot. Required-prefix groups
export empty captures while skipped descendants remain undefined. Shared empty
execution preserves Unicode boundaries, indices/groups, lastIndex, String
consumers, cloning and collection; original errors precede proof selection.


A separate root-choice constructor accepts an independently empty first arm
proved by the optional-established-impossible, sequence or enclosing-sequence
constructors. Its final successful continuation selects that arm and leaves
every later lexical slot undefined (22.2.2.2). This proof remains root-only;
repeat operands may retry a later consuming arm after continuation failure.
Original bindings survive projection; source/copy/full-mask work precedes
allocation and actual matching and capture queries stay constant.


A separate complete-positive-assertion constructor exports masks from optional
established-impossible operands, sequences, enclosing sequences and root-first
choices in either assertion direction (22.2.2.4). The complete operand's final
successful continuation makes the first-arm proof valid. Outside siblings,
wrappers and additional positive layers require other proofs. Original bindings
and all lexical slots survive; the finite earlier-operand chain never re-enters
the new constructor. Source/copy work precedes allocation, with constant actual
matching and capture queries over a source-sized retained mask.


A separate negative-assertion mode accepts independently empty optional
established-impossible operands, sequences, enclosing sequences, root-first
choices and complete positive assertions. A guaranteed-successful operand makes
the negative assertion fail (22.2.2.4); its final successful continuation admits
the first-arm proof. Root disjunctions and unproved/nonroot guards reject this
constructor. The finite closed earlier-operand chain never re-enters the new
mode, and erased callback types bound compiler expansion. Full original
bindings/inventories survive, source/copy work precedes allocation, and retained
no-match storage and actual matching work remain constant.


A separate required-spine constructor wraps negative assertions over optional
established-impossible empty forms. Every outside minimum must be positive,
keeping the failing body mandatory (22.2.2.3.1). Original bindings, complete
lexical slots and symbolic counts remain intact. Optional wrappers, outside
siblings and unproved bodies reject this constructor. Flat scope/body-copy work
precedes allocation; no-match storage and actual matching remain constant.


A separate optional-spine constructor skips established negative failures over
optional-empty forms, including their complete required wrappers. At least one
outside zero minimum must skip the failing operand (22.2.2.3.1). Required-prefix
captures stay empty and all skipped descendants remain undefined. The finite
closed earlier-operand chain preserves original bindings and symbolic counts.
Flat source/frame/body-copy/full-mask work precedes allocation; retained masks
stay source-sized and actual matching and capture queries remain constant.


Runtime selects optional established-impossible concatenations with complete
original normalized source. Interleaved empty/undefined term masks retain their
original bindings and lexical slots. Shared empty execution preserves Unicode
boundaries, indices/groups, lastIndex, String consumers, cloning and collection;
original validation errors precede proof selection.


A separate sequence mode composes optional established-negative spines with
independently empty optional-known, negative-impossible or empty-tree terms. At
least one root term must use the new optional-established-negative proof and
every term must stay empty (22.2.2.2, 22.2.2.4). Interleaved masks and original
bindings survive. Root choices and consuming/unproved terms reject the proof;
earlier scanner modes retain their domains and work. Disjoint source/copy/mask
work precedes allocation, with source-sized retained masks and constant actual
matching and capture queries.


A separate enclosing-sequence proof wraps optional established-negative
concatenations in complete ordinary/modifier spines. Independently empty body
terms let required wrappers preserve the body mask while optional repeats clear
descendant captures (22.2.2.3.1, 22.2.2.7). Original bindings, full lexical slots
and symbolic counts remain intact. Flat scope/body-copy/full-mask work precedes
allocation; retained masks stay source-sized and actual matching and capture
queries remain constant.


Runtime selects complete ordinary/modifier spines around optional established-
impossible concatenations. Full normalized source keeps every original binding
and lexical slot. Required wrappers retain body masks while optional repeats
clear descendants; shared empty execution preserves Unicode boundaries,
indices/groups, lastIndex, String consumers, cloning and collection, with
original validation errors before proof selection.


Runtime selects root choices whose first arm uses independently empty optional
established-impossible forms. The final successful continuation selects the
first arm, preserving its mask and leaving every later lexical slot undefined
(22.2.2.2). Full normalized source keeps duplicate names and original bindings.
Shared empty execution preserves Unicode boundaries, indices/groups, lastIndex,
String consumers, cloning and collection, with original errors before proof
selection. This root-only proof is not composed under repeat continuations.


Runtime selects complete positive lookahead/lookbehind over independently empty
optional established-impossible forms, including root-first choices with their
final successful continuation (22.2.2.4). Full normalized source keeps original
bindings and lexical slots. Both directions export exact operand masks. Shared
empty execution preserves Unicode boundaries, indices/groups, lastIndex, String
consumers, cloning and collection, with original errors before proof selection.


Runtime selects root negative assertions over independently empty
optional-established-impossible operands, sequences, enclosing sequences,
root-first choices and complete positive assertions. The complete operand
always succeeds, so either negative assertion direction fails (22.2.2.4).
Root-first proofs use the operand’s final successful continuation. Operand
matching data is unnecessary. Full normalized source retains every original
binding and lexical slot. Shared execution preserves Unicode boundaries,
indices/groups where matches exist, source/flags, lastIndex, String consumers,
cloning and collection, with original validation errors before proof
selection.


Runtime selects complete required ordinary/modifier spines around negative
assertions over independently empty optional-established-impossible forms.
Every outside minimum is positive, keeping the failing body mandatory
(22.2.2.3.1). Original bindings, complete lexical inventories and symbolic
counts remain intact. Full normalized source retains every original binding
and lexical slot. Shared execution preserves Unicode boundaries,
indices/groups where matches exist, source/flags, lastIndex, String consumers,
cloning and collection, with original validation errors before proof
selection.


Runtime selects optional ordinary/modifier spines over established negative
failures and their complete required wrappers. At least one outside zero
minimum skips the failing operand (22.2.2.3.1). Required-prefix captures stay
empty; every skipped descendant remains undefined. Full normalized source
retains every original binding and lexical slot. Shared execution preserves
Unicode boundaries, indices/groups where matches exist, source/flags,
lastIndex, String consumers, cloning and collection, with original validation
errors before proof selection.


Runtime selects concatenations of optional established-negative spines and
independently empty earlier terms. Every term stays empty; at least one term
uses the optional-established-negative proof (22.2.2.2, 22.2.2.4). Interleaved
masks and original bindings survive. Full normalized source retains every
original binding and lexical slot. Shared execution preserves Unicode
boundaries, indices/groups where matches exist, source/flags, lastIndex,
String consumers, cloning and collection, with original validation errors
before proof selection.


Runtime selects complete ordinary/modifier spines around independently empty
optional established-negative concatenations. Required wrappers retain body
masks while optional repetitions clear descendant captures (22.2.2.3.1,
22.2.2.7). Original bindings, full lexical slots and symbolic bounds remain
intact. Full normalized source retains every original binding and lexical
slot. Shared execution preserves Unicode boundaries, indices/groups where
matches exist, source/flags, lastIndex, String consumers, cloning and
collection, with original validation errors before proof selection.


A separate flat static Pattern constructor composes independently empty,
impossible and otherwise unproved facts across ordinary/modifier groups,
sequences, choices, both assertion directions and symbolic quantifiers
(22.2.2.2–4, 22.2.2.7). A successful empty choice retains its first possible
arm and clears failed/later arms. Every successful arm must independently stay
empty before a mask can be exported; a merely nullable operand may consume or
capture data and only proves an outside negative assertion impossible. Unknown
escape boundaries and conditional consuming forms remain unproved. Earlier
constructors retain their exact domains and work.

The constructor scans complete original normalized source without rebinding or
projecting references. Safe literal/class/property/surrogate boundaries reuse
the existing tree scanner. Flat frames and prefix maxima for overlapping capture
clears keep construction source-sized at arbitrary depth, without recursive
proof calls or count expansion. Source-sized work precedes frame/mask
allocation. The public result reuses existing empty/no-match plans: an empty
plan retains a source-sized mask, an impossible plan retains only its full
lexical capture count, and actual matching and capture queries remain constant.


Runtime selects the flat compositional static proof after earlier matchers. It
passes complete original normalized source and maps the result to the existing
empty or impossible execution plan, with no capture-offset projection. Failed
alternative arms retain undefined slots while the first successful empty arm
exports exact masks, including duplicate-name groups. Shared execution preserves
source/flags, Unicode boundaries, indices/groups, lastIndex, String consumers,
cloning and collection; full original validation precedes proof selection.

RegExp allocation/prototype lookup and ToString conversions finish in a small
non-inlined frame before the large matcher-selection helper is entered
(22.2.4.1). Reentrant getters/conversions therefore use the existing native
stack guard without retaining the compiler's large stack frame. Pattern
validation, budget charges, original slots and lastIndex initialization retain
the same observable order. No default quota or native reentry allowance changes.


A separate static-proof mode treats a guaranteed-successful capture-free positive
assertion as independently empty in either direction (22.2.2.4). Its operand may
consume, but the assertion consumes zero and contains no lexical capture slots
whose values could escape. Outside required/optional groups retain exact empty/
undefined masks. Capturing operands keep the earlier conservative facts. The
original public constructor retains its domains and work through a separate
scanner mode; flat frames, symbolic counts, capture-clear invariants and source
charges remain the same.


Runtime selects the separate capture-free positive static mode after the earlier
static proof. Complete guaranteed-successful positive operands with no lexical
captures compose as empty in both directions, while outside groups keep exact
empty/undefined masks. Consuming capturing operands keep their earlier matching
behavior. Full normalized source preserves original bindings and slots; shared
execution retains Unicode boundaries, indices/groups, source/flags, lastIndex,
String consumers, cloning and collection, with original errors before proof
selection and compilation outside reentrant conversion frames.


A third static mode composes guaranteed-successful positive operands whose
captures always remain undefined (22.2.2.2–4). Flat frames conservatively count
lexical captures that may participate in any successful operand path. Impossible
arms, successful negative assertions, exact-zero groups and skipped independently
empty/impossible groups contribute no exports. Other capturing groups contribute
their own slot and possible child exports. Choice counts include every possible
arm, so conditional later-arm mask clears cannot incorrectly prove capture
absence. Defined empty captures remain unproved when consumption could move
their indices.

A positive assertion with a guaranteed-successful operand and zero possible
exports consumes zero and clears its entire body capture interval. This retains
all lexical slots as undefined without exposing consumed data or spans. Counter
merges and prefix clear markers avoid scanning nested masks; construction stays
source-sized at arbitrary depth. Earlier static modes retain their domains and
source-work charges, with original full validation and bindings intact.


Runtime selects the undefined-capture positive mode after the earlier static
constructors. Guaranteed-successful operands can consume while all their
lexical captures remain undefined, including negative-assertion, erased and
skipped groups. Both directions export exact undefined body slots and retain
required/optional outside masks. Full normalized source preserves original
bindings; shared execution keeps Unicode boundaries, indices/groups, source/
flags, lastIndex, String consumers, cloning and collection, with original errors
and coercions before proof execution. The large compiler remains outside
reentrant conversion frames.


A dedicated Core matcher compiles a complete positive Unicode lookahead over one
quantified character atom (22.2.2.3.1, 22.2.2.4). Complete unquantified ordinary
and noncapturing groups may surround the assertion and its operand. The
existing repeated-atom plan selects the operand with its original greedy/lazy
counts, line anchors, flags, input context and initial surrogate boundary. The
assertion's whole-match span remains empty at that selected candidate. Outside
captures use that empty span; operand wrappers use the full selected operand
span, including a defined empty run. Both ranges remain available independently
without allocating or rescanning a capture vector.

Original u/v Pattern validation and normalized capture heads are required before
projection. Internal atom captures, quantified outside wrappers, modifiers,
continuations, alternatives and lookbehind retain separate compiler domains.
Source work precedes all wrapper scans and operand copying. Construction,
cloning and dropping retain flat source-sized storage; symbolic counts and
wrapper depth add no native recursion. Underlying construction and actual-search
work callbacks preserve their existing events and errors. Positive ASCII
property folding follows the existing edition-17 rule, including the documented
Node iv discrepancy.


Runtime selects the complete Unicode positive-lookahead plan after its earlier
proofs, using the complete validated normalized source. It resets the projected
outer-capture offset only after successful selection, so every original lexical
slot retains its role. Actual search work delegates to the operand plan. Each
result caches the empty assertion range and one separate operand range; outside
slots and operand slots resolve independently in constant work, without a
capture vector or input copy. Named groups and indices, ordinary/global/sticky
lastIndex, String consumers, cloning and collection share the existing result
path. Source/flags validation and conversions still precede compilation, whose
large frame stays outside reentrant getters and conversions.


A separate positive-lookahead constructor composes leading/trailing operand
word assertions with optional line gates (22.2.2.4). The repeated-atom endpoint
matcher chooses the consumed extent that passes both word predicates, including
retrying lazy endpoints. Outside captures remain empty and operand wrappers
retain that selected extent. Original Unicode input context and i-mode word
folds apply at both boundaries; empty operands keep defined empty captures.
The earlier line-only constructor retains its proof domain and work events.
Both public entries borrow their callbacks into a shared flat wrapper parser,
so callback types do not duplicate its delegated compiler graph.

Unicode searches enumerate complete-character candidate boundaries. V8 can
return an interior surrogate position for an empty nonboundary search, for
example `\Bſ*?\B` between the halves of an emoji. The implementation follows
RegExpBuiltinExec's complete-character advancement. This differs from the
approved initial-offset rule: an explicitly supplied trailing-pair start still
uses the leading boundary. A native sticky-candidate oracle enumerates legal
search positions independently and checks operand ranges and captures at each
position.


Runtime selects the word-boundary lookahead constructor after the line-only
constructor, with the same complete validated source and successful-selection
capture-offset reset. Its original-context word predicates and lazy endpoint
retries use the shared actual-work execution path and independently cached
operand extent. Named groups, indices, stateful execution, String consumers,
cloning and collection retain those ranges. The compilation frame remains
separate from user getters and conversions.


A separate Core entry compiles positive Unicode lookahead over one unquantified
character atom (22.2.2.3–4). After full original validation and normalized capture
heads, the flat lexical scanner proves exactly one non-reference atom between
optional line assertions. The projected operand inserts the constant `{1}`
bound immediately after that atom and before its optional end assertion, then
reuses the existing consuming atom compiler. No projected source is independently
validated or rebound. Unknown escapes, additional terms, word predicates and
other counts retain separate proof domains.

Required outside wrappers remain empty; complete operand wrappers retain the
selected original character range. Raw astral characters and adjacent fixed
surrogate escapes preserve complete-character boundaries; lone surrogates match
only lone input characters. Source work precedes lexical scans and the constant
three-unit projection growth. Earlier repeated and word-boundary entries keep
their domains and charges through the shared flat parser.


Runtime selects the unquantified-atom lookahead constructor after its earlier
line and word lookahead proofs. The full normalized source preserves lexical
slots; successful selection resets only the projected outer offset. The
constant single-iteration operand projection remains internal: exposed source,
flags and cloning retain the original Pattern. Cached empty and consuming ranges
feed named captures, indices and String consumers independently. Four former
Unsupported controls now assert exact positive execution for captured classes,
folded literals, astral literals and lone surrogates.


A separate Core entry admits an unquantified lookahead atom between outside
word predicates, with optional leading ^ and trailing $. A fixed lexical
envelope proves at most one leading and trailing word predicate and exactly
one non-reference atom; at least one word predicate is required. The constant
single-iteration bound is inserted after the atom, preserving both predicate
scopes, then the existing actual-work word endpoint matcher selects its
original input range. A class-contained backspace escape remains part of the
atom rather than becoming a predicate.

Outside and operand captures retain independent empty and one-character ranges,
including astral widths and Unicode-i word folds. Unicode search advances over
complete characters; explicitly supplied trailing-pair starts retain the
approved initial rewind. Earlier line-only, repeated-word and unquantified
constructors keep their domains and work.


Runtime selects the single-atom word-predicate constructor after the earlier
lookahead proofs. It uses full normalized source and resets the projected
capture offset only on successful selection. Existing empty/operand range
caching preserves names, indices, source/flags, state, String consumers, copies
and collection. Word and line predicates retain original input context, and
ordinary search excludes interior surrogate candidates. Compilation remains
outside reentrant user conversions.


A separate Core constructor compiles complete positive Unicode lookbehind over
one unquantified non-reference character atom (22.2.2.4). It shares the flat
wrapper inventory and lexical single-iteration projection with forward
assertions, while keeping the original assertion direction in the plan.
Optional operand ^/$ assertions retain full original input context. Other
counts, alternatives, references, modifiers and word predicates retain separate
proof domains.

Search visits complete-character candidate boundaries. Each candidate derives
at most one complete predecessor and probes the proved fixed-count operand
stickily at that predecessor's beginning. Only an operand ending at the
candidate can succeed. Outside wrappers capture the empty candidate position;
operand wrappers capture the original predecessor range before it. The fixed
count bounds each probe to at most two character attempts, including failing
end assertions, so repeated candidate retries remain linear rather than
rescanning input suffixes. Actual work precedes all input access. Initial
trailing-pair offsets retain the approved leading-boundary policy. Construction,
cloning and dropping stay flat, with no input-sized capture storage. Earlier
forward constructors retain their domains and work.


Runtime selects the positive single-character lookbehind constructor after the
existing lookahead proofs, using full validated normalized source. On success
it resets the projected outer offset and uses the existing independently cached
empty/operand ranges. Operand slots may precede the whole-match position;
outside slots remain empty at that position. Named indices and String consumers
slice captures from the full original input, so replacement and split preserve
predecessor values. Source/flags, copies, collection, original coercions and
stateful execution retain the shared result path. The compiler frame remains
separate from reentrant user conversions.


A separate Core lookbehind entry admits outside word predicates around one
unquantified predecessor atom, with optional ^/$ gates. The lexical envelope
requires at least one word predicate and preserves class-contained backspace
escapes. Its fixed single iteration evaluates both predicate boundaries against
the full original input. For one non-reference character, predicate truth is
independent of matching direction; only the assertion position and operand
extent differ.

The existing complete-predecessor loop retains linear actual work, original
Unicode-i word folds and the approved initial-pair rewind. Outside wrappers
stay empty at the candidate, while operand wrappers retain the preceding
character. Earlier lookahead and line-only lookbehind constructors keep their
proof domains and work.

Runtime selects the word-predicate lookbehind entry after the existing positive
assertion proofs. It retains full normalized source, resets the projected
capture offset on success and shares the cached empty/predecessor ranges.
Original word and line context therefore survives result construction, named
indices, String consumers, copies and collection. Ordinary search visits only
complete Unicode candidates, independently of native-engine surrogate search.

A separate negative Unicode assertion plan proves complete lookahead over one
unquantified non-reference character atom, with optional operand ^/$ gates
(22.2.2.4). After charging source work, it identifies complete outside wrappers
and changes only the assertion sign for the existing positive single-atom proof.
That proof rejects references before projection; no projected source is
independently validated or rebound. Other counts, word predicates, alternatives,
continuations and assertion directions remain separate proof domains.

Search probes the positive operand stickily at each complete Unicode candidate
and succeeds exactly when that probe fails. A fixed single iteration bounds
each probe, keeping retries linear without suffix rescans. Original line,
dotAll, fold and surrogate semantics remain in the shared operand matcher.
Outside captures select the empty assertion span; every operand slot remains
undefined, as required when negative lookahead succeeds. The plan stores only
capture counts and the shared operand, with no input-sized capture allocation.
Source and search cancellation precede their work, and callbacks share private
compiler/search bodies. Runtime integration remains separate.

Runtime selects the negative single-character entry after the existing positive
assertion proofs, using full normalized source and resetting the projected
capture offset on success. Its common result cache stores the empty match only;
outside captures derive from that position and operand slots stay undefined.
Named result/indices properties, replacement callbacks, split, copies and
collection all use the shared capture accessor. The compiler remains separate
from reentrant input conversions. Native V8 search can select an interior-pair
candidate for `(?!😀)`; independent expectations and sticky probes at legal
Unicode boundaries verify the complete-character search required here.

A separate negative-lookahead entry admits one unquantified operand atom with
outside word predicates and optional ^/$ gates. It changes only the proved
assertion sign before delegating to the established positive word-atom proof;
at least one word predicate is required. Negation applies to the conjunction of
the operand and both predicates, retaining their full original input context
(22.2.2.4). Class-contained backspace escapes remain atom data. Outside captures
remain empty and all operand captures undefined. The fixed-count sticky probe
and complete-character candidate loop keep linear actual work and the approved
initial-pair rewind. Earlier line-only negative proofs keep their domains and
charges. Runtime integration remains separate.

Runtime selects the negative word-atom proof after the line-only negative entry,
using full normalized source and the same successful offset reset. Its empty
result and undefined operand slots use the shared accessor, preserving own
named indices properties, original source and flags, String consumers, copies
and collection. Original-context word/line gates and complete-character search
remain in Core; reentrant conversions remain outside the compiler frame.

A separate negative Unicode lookbehind entry proves one unquantified
non-reference predecessor atom with optional ^/$ gates (22.2.2.4). Its charged
flat scanner identifies complete outside wrappers and changes only the proved
assertion sign before delegating to the positive single-character lookbehind
proof. No projected Pattern is independently validated or rebound. Other
counts, word predicates, continuations, alternatives and references retain
separate compiler domains.

The shared negative candidate loop invokes that positive assertion stickily and
succeeds exactly when its one complete predecessor fails. This keeps direction,
original line/dot/fold context and complete-character advancement, with linear
actual work and the approved initial-pair rewind. At input start, the absent
predecessor makes negative lookbehind succeed; at EOF the preceding character
can still make it fail. Outside wrappers capture the empty assertion position;
all operand captures remain undefined. Construction, clone and drop stay flat,
and cancellation precedes source/input work. Runtime integration remains separate.

Runtime selects the negative single-character lookbehind constructor after the
existing negative-lookahead proofs. Full normalized source preserves original
capture slots, and successful selection resets the projected offset. The
existing negative result path retains empty outside ranges and undefined
operand slots through named indices, String consumers, copies and collection.
Original predecessor/line context and complete-character candidates stay in
Core, while input conversion and stateful execution retain their shared paths.

A separate negative single-character lookbehind entry accepts canonical operand
word predicates with optional ^/$ gates. Its assertion-sign projection delegates
to the established positive word-predecessor proof; at least one word predicate
is required. Negation applies to the complete conjunction of the predecessor
atom and both predicates, preserving original input context (22.2.2.4).
Class-contained backspace escapes stay atom data. The shared complete-character
candidate loop and bounded sticky predecessor probe retain linear actual work,
initial-pair rewind, empty outside captures and undefined operand slots. Earlier
line-only and forward entries keep their domains and charges. Construction,
cloning and dropping remain flat. Runtime integration remains separate.

Runtime selects the negative word-predecessor constructor after the line-only
lookbehind entry. It retains full normalized source and the existing successful
offset reset, then shares empty outside/undefined operand capture access with
the other negative assertions. Original-context predicates, named indices,
String consumers, copies, collection and complete-character stateful search
retain the shared paths. User conversions remain outside the compiler frame.

A separate checked positive-lookahead constructor admits one unquantified
v-mode class containing ClassStringDisjunction operands whose alternatives all
decode to one character (22.2.2.4, 22.2.2.9). Charged flat scans retain complete
ordinary outside/operand wrappers, prove exactly one class between optional
^/$ gates and insert one fixed iteration immediately after that class. The
existing checked single-character string-class compiler proves every member
and supported nested operation. Empty/long members, other counts, word gates,
references, continuations and other assertion directions remain separate domains.

The projection is internal and never independently validated or rebound. Full
original v/vi validation and normalized capture heads remain prerequisites.
Outside captures retain the empty assertion position, while operand wrappers
capture the complete original character, including an astral pair. Line gates,
folding, lone surrogates and candidate advancement use the shared operand matcher.
Checked work and capacity errors retain their existing contracts. Construction,
clone/drop and cancellation stay flat; earlier entry points retain their domains
and charges. Runtime integration remains separate.

Runtime selects the single-character q-class lookahead proof only in v mode,
after the earlier assertion entries. It uses full normalized source, resets the
projected outer offset on successful selection and maps checked construction
work/capacity failures through the existing host error path. The public source
and flags retain the original class syntax. Shared empty/operand range caching
preserves named indices, String consumers, copies and collection. User
conversions remain outside the compiler frame, and existing proof domains and
work charges remain separate.

A separate checked q-class lookahead entry admits leading/trailing operand word
predicates with optional ^/$ gates. The flat envelope requires at least one word
predicate and exactly one class whose q alternatives each decode to a character.
Its fixed single iteration stays between those predicates, so the existing
checked class matcher evaluates original-context word/line truth and retains
the original complete-character capture extent (22.2.2.4, 22.2.2.9). Class data,
including a backspace member, stays distinct from outside word assertions.
Outside wrappers remain empty; checked work/capacity errors retain their host
contracts. The line-only q-class entry keeps its proof domain and callback
sequence through the shared flat parser. Runtime integration remains separate.

Runtime selects the word-predicate q-class lookahead entry after the line-only
q-class proof, gated by v mode and using full normalized source. Successful
selection resets the projected outer offset; checked construction errors retain
the existing host mapping. Shared empty/operand range caching preserves names,
original q source, indices, String consumers, copies and collection. Original
context and complete-character search remain in Core; reentrant conversions
remain outside the compiler frame.

A separate checked positive lookbehind entry admits one unquantified v-mode
class whose q alternatives each decode to a character, with optional ^/$ gates
(22.2.2.4, 22.2.2.9). The flat assertion parser retains original direction while
sharing complete wrapper inventories and the fixed single-iteration class
projection. Checked class compilation proves the scalar domain before matching;
empty/long q members, other counts, word gates, continuations and references
remain separate proofs.

The existing complete-predecessor search probes that class stickily at each
candidate and requires it to end at the candidate. Outside captures remain
empty there; operand wrappers retain the original preceding character. Fold,
line and surrogate context, initial-pair rewind and bounded actual-work retries
retain the shared path, without suffix rescans or input-sized capture storage.
Construction, clone/drop and work/capacity errors keep their existing contracts.
Earlier q-lookahead entries retain their domains and charges. Runtime integration
remains separate.

Runtime selects the single-character q-class lookbehind constructor after the
q-lookahead entries, gated by v mode and using full normalized source. Successful
selection resets the projected outer offset, and checked work/capacity failures
retain the existing host mapping. The shared result cache retains independent
empty outside and predecessor capture ranges, so named indices and String
consumers slice the original preceding character. Original q source/flags,
copies, collection and stateful execution keep their existing paths. User
conversions remain outside the compiler frame.

A separate checked q-class lookbehind entry admits leading/trailing operand
word predicates with optional ^/$ gates. At least one word predicate is
required around a class whose q alternatives each decode to one character.
The fixed single iteration stays within those gates; the existing complete
predecessor loop retains their original input context, empty outside captures
and preceding operand spans (22.2.2.4, 22.2.2.9). Backspace class data stays
distinct from outside predicates. Checked work/capacity errors, bounded retries,
initial-pair rewind and flat construction/clone/drop keep their existing paths.
Earlier q-class entries retain their domains and charges. Runtime integration
remains separate.

Runtime selects the word-predicate q-class lookbehind entry after the line-only
q-lookbehind proof, gated by v mode and using full normalized source. Successful
selection resets the projected outer offset, and checked construction failures
use the existing host mapping. The common result cache preserves empty outside
and preceding operand ranges through named indices and String consumers.
Original q source/flags, word/line context, copies, collection and stateful
Unicode candidates retain the shared paths. User conversions remain outside
the compiler frame.

A separate checked negative-lookahead constructor admits one unquantified
v-mode class whose q alternatives each decode to a character, with optional
^/$ gates (22.2.2.4, 22.2.2.9). Charged flat scans retain the complete outside
wrapper inventory and change only the proved assertion sign before delegating
to the checked positive q-class proof. Projected source is never independently
validated or rebound. Other widths, counts, word predicates, directions and
binding roles remain separate compiler domains.

The shared negative loop succeeds exactly when its bounded sticky class probe
fails at a legal complete-character candidate. Outside wrappers capture that
empty position and all operand slots remain undefined, including through deep
scopes. Original line/fold/surrogate context, initial-pair rewind, linear actual
work and checked work/capacity errors retain their existing paths. Earlier
constructors keep their domains and charges. Runtime integration remains separate.

Runtime selects checked negative q-class lookahead after the positive q-class
entries, gated by v mode and using full normalized source. Successful selection
resets the projected outer offset, and checked construction failures retain
the existing host mapping. The shared negative result path keeps outside ranges
empty and every operand slot undefined through names, own indices properties,
String consumers, copies and collection. Original q source/flags, coercion
ordering and stateful execution remain in their shared paths; user conversions
stay outside the compiler frame.

A separate checked negative q-class lookahead entry accepts leading/trailing
operand word predicates with optional ^/$ gates. It projects only the assertion
sign and delegates to the checked positive word-q proof, requiring at least one
word predicate and one character per q alternative (22.2.2.4, 22.2.2.9). Negation
applies to the conjunction of the atom and its original-context predicates.
Outside captures stay empty; all operand slots remain undefined. Backspace
class data stays distinct from outside word assertions. The fixed-count sticky
probe preserves linear actual work, complete-character search, initial-pair
rewind and checked construction errors. Earlier q-class entries keep their
domains and charges. Runtime integration remains separate.

Runtime selects checked negative word-q lookahead after its line-only q-class
entry, gated by v mode and using full normalized source. Successful selection
resets the projected outer offset; checked construction failures use the
existing host mapping. Shared empty outside/undefined operand capture access
preserves own named indices properties, original q source/flags, String
consumers, copies and collection. Original-context word and line predicates
stay in Core, and user conversions remain outside the compiler frame.

A separate checked negative q-class lookbehind entry proves one unquantified
predecessor character with optional ^/$ gates (22.2.2.4, 22.2.2.9). Charged flat
scans retain outside wrappers and project only the proved assertion sign before
delegating to positive q-class lookbehind. Complete original v/vi validation and
normalized capture heads remain prerequisites; projected source is never
independently validated or rebound. Other widths, counts, word predicates and
binding roles retain separate domains.

The shared negative candidate loop negates one bounded sticky predecessor
probe, preserving absent-predecessor and EOF behavior, original line/fold/
surrogate context, complete-character search and initial-pair rewind. Outside
captures remain empty and all operand slots undefined. Linear actual work,
checked work/capacity errors and flat construction/clone/drop keep their shared
paths. Earlier q-class entries retain their domains and charges. Runtime
integration remains separate.

Runtime selects checked negative q-class lookbehind after its negative
q-lookahead entries, gated by v mode and using full normalized source. Successful
selection resets the projected outer offset, and checked construction failures
keep the existing host mapping. Empty outside and undefined operand ranges use
the shared capture accessor, preserving own named indices properties, String
consumers, copies and collection. Original q source/flags, predecessor context
and stateful execution retain their shared paths; user conversions remain
outside the compiler frame.

A separate checked positive-lookahead entry admits one unquantified scalar
v-mode class containing a nested class or set operator (22.2.2.4, 22.2.2.9).
The charged flat assertion parser retains complete outside/operand wrapper
inventories, optional original-context ^/$ gates and one fixed class iteration.
Checked scalar-set compilation proves every leaf is character-valued; q strings,
other string leaves, counts, word predicates and additional terms retain
separate proof domains. Full original v/vi validation and normalized capture
heads remain prerequisites; projected source is never independently validated
or rebound.

The shared character matcher retains intersection, subtraction, complement,
folding, empty-set and surrogate semantics without expanding ranges. Outside
captures stay empty and operand wrappers retain the original complete character.
Checked construction errors and actual-work search keep their existing paths.
The shared assertion parser selects scalar-set versus one-character-q backends
explicitly; earlier q-class domains and work sequences remain separate. Runtime
integration remains separate.

Runtime selects the checked scalar-set lookahead proof after the q-class
assertion entries, gated by v mode and using full normalized source. Successful
selection resets the projected outer offset; checked construction failures keep
the existing host mapping. Shared empty/operand range caching preserves names,
original set syntax, indices, String consumers, copies and collection. Nested
set algebra, original line/fold context and complete-character search stay in
Core. User conversions remain outside the compiler frame.

A separate checked scalar-set lookahead entry admits leading/trailing operand
word predicates with optional ^/$ gates. The flat envelope requires at least
one word predicate and a nested/operator class with character-valued leaves;
its fixed iteration stays inside those predicates (22.2.2.4, 22.2.2.9). The
existing checked word-set compiler preserves original line/word context,
folded set algebra and backspace class data. Outside wrappers remain empty;
operand wrappers capture the original complete character. Actual-work search,
checked construction errors, initial-pair rewind and flat storage retain their
shared paths. Earlier scalar/q entries keep their domains and charges. Runtime
integration remains separate.

Runtime selects checked word-scalar-set lookahead after its line-only scalar
entry, gated by v mode and using full normalized source. Successful selection
resets the projected outer offset, and checked construction errors retain their
host mapping. Shared empty/operand range caching preserves original set syntax,
names, indices, String consumers, copies and collection. Original-context word
and line predicates stay in Core, while user conversions remain outside the
compiler frame.

A separate checked scalar-set lookbehind entry accepts one unquantified
nested/operator class with character-valued leaves and optional ^/$ gates
(22.2.2.4, 22.2.2.9). It shares the flat charged assertion envelope and checked
scalar-set compiler while retaining the backward direction. A bounded sticky
probe must consume exactly the complete predecessor ending at the candidate;
line gates inspect original input context. Outside wrappers remain empty at
that end and operand wrappers capture the predecessor. Strings, counts and
word predicates retain separate proof domains. Checked errors, complete Unicode
advancement, initial-pair rewind and actual-work cancellation keep their shared
paths. Runtime integration remains separate.

Runtime selects checked scalar-set lookbehind after its scalar-lookahead
entries, gated by v mode and using full normalized source. Successful selection
resets the projected outer offset; checked errors retain their host mapping.
The shared empty/operand range cache preserves original set source, named
indices, String consumers, copies and collection with captures preceding the
empty root match. Original-context predecessor and line checks stay in Core;
user conversions remain outside the compiler frame.

A separate checked scalar-set lookbehind entry admits leading/trailing operand
word predicates with optional ^/$ gates. It requires at least one word
predicate and delegates the fixed character iteration to the existing checked
word-set compiler (22.2.2.4, 22.2.2.9). A bounded predecessor probe retains
original line/word/fold context and backspace class data; outside wrappers stay
empty while operand wrappers capture the complete predecessor. Nested scalar
algebra, checked errors, complete Unicode advancement, initial-pair rewind and
actual-work cancellation keep their shared paths. Other domains retain their
existing proofs. Runtime integration remains separate.

Runtime selects checked word-scalar-set lookbehind after its line-only
lookbehind entry, gated by v mode and using full normalized source. Successful
selection resets the projected outer offset; checked errors keep their host
mapping. Shared empty/predecessor range caching preserves original set syntax,
named indices, String consumers, copies and collection. Word/line/predecessor
context and stateful Unicode execution remain in Core, while user conversions
stay outside the compiler frame.

A separate checked negative q-class lookbehind entry accepts leading/trailing
operand word predicates with optional ^/$ gates. It projects only the proved
assertion sign and delegates to checked positive word-q lookbehind, requiring
at least one word predicate and one character per q alternative (22.2.2.4,
22.2.2.9). Negation applies to the original-context predecessor conjunction;
outside wrappers remain empty and all operand captures undefined. Backspace
class data stays distinct from outside predicates. The bounded sticky probe,
complete-character search, initial-pair rewind, checked errors and flat storage
retain their shared paths. Earlier entries keep their domains and charges.
Runtime integration remains separate.

Runtime selects checked negative word-q lookbehind after the line-only negative
q-lookbehind entry, gated by v mode and using full normalized source. Successful
selection resets the projected outer offset, and checked construction failures
keep the existing host mapping. The shared negative capture accessor retains
empty outside/undefined operand slots through named indices, String consumers,
copies and collection. Original q source, word/line/predecessor context and
stateful Unicode execution retain their shared paths; user conversions remain
outside the compiler frame.

A separate checked negative scalar-set lookahead entry proves one nested/
operator class with character-valued leaves and optional original-context ^/$
gates (22.2.2.4, 22.2.2.9). The charged flat negative envelope changes only the
proved assertion sign before delegating to the checked positive scalar proof;
full original v/vi validation and normalized bindings remain prerequisites.
Negation applies to the complete original-context conjunction. Outside wrappers
capture the empty assertion position, while every operand slot stays undefined.
Bounded sticky probes, complete-character advancement, initial-pair rewind,
checked errors and actual-work cancellation retain their shared paths. The
shared checked envelope explicitly selects scalar versus q-class proofs;
earlier q domains and charges remain unchanged. Runtime integration remains
separate.

Runtime selects checked negative scalar-set lookahead after the positive
scalar assertion entries, gated by v mode and using full normalized source.
Successful selection resets the projected outer offset; checked errors retain
their host mapping. Shared negative capture caching preserves empty outside
and undefined operand slots through named indices, String consumers, copies
and collection. Original source, set algebra, line context and complete Unicode
execution stay in Core, while user conversions remain outside the compiler
frame.

A separate checked negative scalar-set lookahead entry admits leading/trailing
operand word predicates with optional ^/$ gates, requiring at least one word
predicate (22.2.2.4, 22.2.2.9). The flat negative envelope projects only the
assertion sign and delegates to checked positive word-scalar lookahead.
Negation applies to the original-context class and predicate conjunction;
outside wrappers remain empty and all operand slots undefined. Backspace class
data, folded set algebra, checked errors, complete Unicode advancement,
initial-pair rewind and actual-work cancellation retain their shared paths.
Earlier entries keep their domains and charges. Runtime integration remains
separate.

Runtime selects checked negative word-scalar-set lookahead after its line-only
negative scalar entry, gated by v mode and using full normalized source.
Successful selection resets the projected outer offset; checked errors keep
their host mapping. Shared negative capture caching retains empty outside and
undefined operand slots through named indices, String consumers, copies and
collection. Original set syntax, word/line context and complete Unicode search
remain in Core; user conversions stay outside the compiler frame.

A separate checked negative scalar-set lookbehind entry proves one nested/
operator class with character-valued leaves and optional ^/$ gates
(22.2.2.4, 22.2.2.9). It projects only the assertion sign and delegates to
checked positive scalar lookbehind, so negation applies to the original-context
complete predecessor conjunction. Outside wrappers remain empty and all operand
slots undefined. Bounded sticky predecessor probes, folded set algebra, checked
errors, complete Unicode advancement, initial-pair rewind and actual-work
cancellation retain their shared paths. Earlier entries keep their domains
and charges. Runtime integration remains separate.

Runtime selects checked negative scalar-set lookbehind after the negative
scalar-lookahead entries, gated by v mode and using full normalized source.
Successful selection resets the projected outer offset; checked errors retain
their host mapping. Shared negative capture caching keeps outside ranges empty
and all operand slots undefined through named indices, String consumers, copies
and collection. Original set syntax, predecessor/line context and stateful
Unicode execution remain in Core; user conversions stay outside the compiler
frame.


A separate checked negative scalar-set lookbehind entry accepts leading/trailing
operand word predicates with optional ^/$ gates, requiring at least one word
predicate (22.2.2.4, 22.2.2.9). The charged flat envelope changes only the
assertion sign and delegates to checked positive word-scalar lookbehind.
Negation applies to the complete original-context predecessor conjunction;
outside wrappers remain empty and all operand slots undefined. Backspace class
data, folded set algebra, bounded sticky probes, checked errors, complete Unicode
advancement, initial-pair rewind and actual-work cancellation retain their
shared paths. Earlier entries keep their domains and charges. Runtime
integration remains separate.


Runtime selects checked negative word-scalar-set lookbehind after its line-only
negative scalar lookbehind entry, gated by v mode and using full normalized
source. Successful selection resets the projected outer offset; checked errors
retain their host mapping. Shared negative capture caching keeps outside ranges
empty and operand slots undefined through named indices, String consumers,
copies and collection. Original set syntax, word/line/predecessor context and
stateful Unicode execution remain in Core; user conversions stay outside the
compiler frame.


A separate checked positive-lookahead entry admits one unquantified v-mode
class with an empty or longer q member and MayContainStrings false
(22.2.2.4, 22.2.2.9). It shares the charged flat assertion envelope, but selects
the existing static character-result compiler for its fixed iteration. Every
successful class match consumes one complete character; an empty set result
never matches. Optional ^/$ gates inspect original input context, outside
wrappers remain empty and operand wrappers retain the complete selected
character. Conservative string results, counts, word predicates and additional
terms retain separate proofs. Full original v/vi validation and normalized
capture heads remain prerequisites; projected source is never independently
validated or rebound.

The shared assertion parser selects q-member, scalar-set and character-result
proofs explicitly. Earlier q/scalar domains, work sequences and checked error
contracts remain unchanged. Character-result set projection, folded algebra,
complete Unicode advancement, initial-pair rewind, flat capture storage and
actual-work cancellation reuse their established paths. Runtime integration
remains separate.


Runtime selects checked character-result lookahead after the scalar assertion
entries, gated by v mode and using full normalized source. Successful selection
resets the projected outer offset; checked errors retain their host mapping.
Shared empty/operand range caching preserves original q/set syntax, named
indices, String consumers, copies and collection. The static character-result
proof, original line context and complete Unicode search remain in Core, while
user conversions stay outside the compiler frame.


A separate checked character-result lookahead entry admits leading/trailing
operand word predicates with optional ^/$ gates, requiring at least one word
predicate (22.2.2.4, 22.2.2.9). A class must contain an empty or longer q member
and have MayContainStrings false. The existing checked character-result
repetition compiler retains one fixed iteration inside those gates. Original
line/word/fold context, character-result set algebra and complete input
characters stay intact; outside wrappers remain empty and operand wrappers
capture the selected character. Checked errors, complete Unicode advancement,
initial-pair rewind and actual-work cancellation retain their shared paths.
Earlier entries keep their domains and charges. Runtime integration remains
separate.


Runtime selects checked word character-result lookahead after its line-only
character-result entry, gated by v mode and using full normalized source.
Successful selection resets the projected outer offset; checked errors retain
their host mapping. Shared empty/operand range caching preserves original q/set
syntax, named indices, String consumers, copies and collection. Original word/
line context and complete Unicode search remain in Core; user conversions stay
outside the compiler frame.


A separate checked positive-lookbehind entry admits one unquantified v-mode
class with an empty or longer q member and MayContainStrings false
(22.2.2.4, 22.2.2.9). The charged flat assertion envelope selects the checked
character-result compiler for a fixed iteration and retains the backward
direction. A bounded sticky probe must consume the complete predecessor ending
at the candidate; optional ^/$ gates inspect original input context. Outside
wrappers remain empty there, while operand wrappers retain the predecessor.
An empty class result never matches. Other widths, counts and word predicates
retain separate proofs. Checked errors, complete Unicode advancement,
initial-pair rewind and actual-work cancellation keep their shared paths.
Runtime integration remains separate.


Runtime selects checked character-result lookbehind after its character-result
lookahead entries, gated by v mode and using full normalized source. Successful
selection resets the projected outer offset; checked errors retain their host
mapping. Shared empty/predecessor range caching preserves original q/set
syntax, named indices, String consumers, copies and collection. The static
character-result proof, original line context and complete Unicode search
remain in Core; user conversions stay outside the compiler frame.


A separate checked character-result lookbehind entry admits leading/trailing
operand word predicates with optional ^/$ gates, requiring at least one word
predicate (22.2.2.4, 22.2.2.9). A class contains an empty or longer q member
and has MayContainStrings false; the checked character-result compiler retains
one fixed iteration inside those gates. A bounded predecessor probe preserves
original line/word/fold context. Outside wrappers remain empty at its end, while
operand wrappers capture the complete predecessor. Character-result algebra,
checked errors, complete Unicode advancement, initial-pair rewind and
actual-work cancellation retain their shared paths. Earlier entries keep their
domains and charges. Runtime integration remains separate.


Runtime selects checked word character-result lookbehind after its line-only
character-result lookbehind entry, gated by v mode and using full normalized
source. Successful selection resets the projected outer offset; checked errors
retain their host mapping. Shared empty/predecessor range caching preserves
original q/set syntax, named indices, String consumers, copies and collection.
Original word/line/predecessor context and complete Unicode execution remain
in Core; user conversions stay outside the compiler frame.


A separate checked negative-lookahead entry admits one unquantified v-mode
class with an empty or longer q member and MayContainStrings false
(22.2.2.4, 22.2.2.9). The charged flat negative envelope changes only the
proved assertion sign before delegating to checked positive character-result
lookahead. Optional ^/$ gates keep original input context; negation applies
to the complete conjunction. Outside wrappers capture the empty assertion
position and every operand slot stays undefined, including when the proved
class result is empty. Other widths, counts and word predicates retain
separate proofs. Full original v/vi validation and normalized capture heads
remain prerequisites; projected source is never independently validated or
rebound.

The shared checked negative envelope selects q-member, scalar-set and
character-result proofs explicitly. Earlier q/scalar domains, work sequences
and checked error contracts remain unchanged. Bounded sticky probes, complete
Unicode advancement, initial-pair rewind, flat capture storage and actual-work
cancellation retain their established paths. Runtime integration remains
separate.


Runtime selects checked negative character-result lookahead after its positive
character-result assertion entries, gated by v mode and using full normalized
source. Successful selection resets the projected outer offset; checked errors
retain their host mapping. Shared negative capture caching preserves empty
outside and undefined operand slots through named indices, String consumers,
copies and collection. Original q/set syntax, line context and complete Unicode
search remain in Core; user conversions stay outside the compiler frame.


A separate checked negative character-result lookahead entry admits leading/
trailing operand word predicates with optional ^/$ gates, requiring at least
one word predicate (22.2.2.4, 22.2.2.9). A class contains an empty or longer q
member and has MayContainStrings false. The flat negative envelope changes
only the assertion sign before delegating to checked positive word
character-result lookahead. Negation applies to the original-context class
and predicate conjunction; outside wrappers remain empty and all operand
slots undefined. Checked errors, complete Unicode advancement, initial-pair
rewind, flat storage and actual-work cancellation retain their shared paths.
Earlier entries keep their domains and charges. Runtime integration remains
separate.


Runtime selects checked negative word character-result lookahead after its
line-only negative character-result entry, gated by v mode and using full
normalized source. Successful selection resets the projected outer offset;
checked errors retain their host mapping. Shared negative capture caching
keeps outside ranges empty and operand slots undefined through named indices,
String consumers, copies and collection. Original q/set syntax, word/line
context and complete Unicode search remain in Core; user conversions stay
outside the compiler frame.


A separate checked negative-lookbehind entry admits one unquantified v-mode
class with an empty or longer q member and MayContainStrings false
(22.2.2.4, 22.2.2.9). The flat negative envelope changes only the proved
assertion sign before delegating to checked positive character-result
lookbehind. Optional ^/$ gates inspect original input context; negation applies
to the complete predecessor conjunction. Outside wrappers remain empty and
all operand slots undefined. Bounded sticky probes, checked errors, complete
Unicode advancement, initial-pair rewind, flat storage and actual-work
cancellation retain their established paths. Earlier entries keep their
domains and charges. Runtime integration remains separate.


Runtime selects checked negative character-result lookbehind after its negative
character-result lookahead entries, gated by v mode and using full normalized
source. Successful selection resets the projected outer offset; checked errors
retain their host mapping. Shared negative capture caching preserves empty
outside and undefined operand slots through named indices, String consumers,
copies and collection. Original q/set syntax, predecessor/line context and
complete Unicode execution remain in Core; user conversions stay outside the
compiler frame.


A separate checked negative character-result lookbehind entry accepts leading/
trailing operand word predicates with optional ^/$ gates, requiring at least
one word predicate (22.2.2.4, 22.2.2.9). A class contains an empty or longer q
member and has MayContainStrings false. The charged flat envelope changes
only the assertion sign before delegating to checked positive word
character-result lookbehind. Negation applies to the original-context
predecessor conjunction; outside wrappers remain empty and all operand slots
undefined. Bounded sticky probes, checked errors, complete Unicode advancement,
initial-pair rewind, flat storage and actual-work cancellation retain their
shared paths. Earlier entries keep their domains and charges. Runtime
integration remains separate.


Runtime selects checked negative word character-result lookbehind after its
line-only negative character-result lookbehind entry, gated by v mode and using
full normalized source. Successful selection resets the projected outer offset;
checked errors retain their host mapping. Shared negative capture caching keeps
outside ranges empty and operand slots undefined through named indices, String
consumers, copies and collection. Original q/set syntax, word/line/predecessor
context and complete Unicode execution remain in Core; user conversions stay
outside the compiler frame.


Unicode string-set matching exposes a separate cancellable traversal for
callers that meter visited search work. Each candidate and traversed trie step
charges the existing conservative sticky per-step weight, rather than charging
unvisited input or prefixes. The flat walk preserves earliest-start and
longest-accepted-member priority, including empty members and shorter members
whose endpoint predicates succeed after longer members are rejected
(22.2.2.7). Scalar union membership, original word/line context, folding and
initial-pair rewind retain their established semantics. Search allocates no
state, and errors return before additional traversal. A private erased callback
body shares code across callers. Existing find/endpoint-filter entries retain
their matching, callback and precharged-work contracts.


### Positive lookahead over variable-length q classes

Core now proves complete positive lookahead over one unquantified single-q
class containing an empty or longer member, with optional operand ^/$ gates
and complete ordinary/noncapturing wrappers around the assertion and operand.
Full original v/vi Pattern validation and capture-head normalization precede
this proof. It delegates decoded/folded class construction to the checked
string-set compiler; source work is charged before scans or fallible projection.
Other class operands, word gates, repetitions, alternatives, continuation terms
and references remain separate proofs.

CompileAtom's forward class-string priority selects the longest member whose
continuation succeeds at the earliest legal Unicode candidate (22.2.2.4,
22.2.2.7, 22.2.2.9). A failed trailing line gate can select a shorter member.
An empty alternative at an earlier candidate precedes a consuming member at a
later candidate. The assertion exports an empty root range and independent
whole-operand wrapper ranges, including defined empty captures. Outside
wrappers capture the root range. Matching uses cancellable visited trie steps,
original input context and UTF-16 extents without input-sized capture state.
This forward proof does not establish reverse class-string priority.


Runtime selects the variable-length single-q lookahead proof only after full
original v-mode Pattern/flags validation and earlier proofs, using normalized
capture heads and the complete lexical scope. It maps checked work/capacity
errors through the existing host boundary and resets enclosing offsets only
when the proof succeeds. The matcher caches the operand as Some(range),
including Some(empty), separately from the empty root and without dynamic
capture storage. RegExpBuiltinExec and String consumers retain original names,
indices, source, UTF-16 advancement and lastIndex behavior. Reentrant coercions
remain outside the compilation frame.


A separate variable-length single-q lookahead constructor admits canonical
operand word predicates before/after the class, with optional ^/$ gates. It
requires at least one predicate, retaining the earlier constructor's domain
and charge sequence. Endpoint predicates inspect the original input: a longer
member can fail a boundary while a shorter or empty member succeeds. The
selected whole-member range remains independent of the empty assertion range.
Checked decoding/folding, legal Unicode candidates, cancellation and lexical
capture accounting reuse the established string-set plan (22.2.2.4, 22.2.2.7).


Runtime selects the word-predicate single-q lookahead constructor as a separate
v-mode fallback after full original validation, using the complete normalized
capture scope. It retains the same checked work/capacity boundary and cached
operand representation, including defined empty captures. Named indices, copies,
String consumers and collection preserve the selected endpoint extent and
original context, including shorter-member retries.


### Negative lookahead over variable-length q classes

A separate checked Core proof accepts complete negative lookahead over one
variable-length single-q class with optional operand ^/$ gates and complete
ordinary/noncapturing wrappers. Full original v/vi validation and normalized
capture heads precede the proof. It charges source work before scans, reserves
the source projection fallibly, and changes only the proved assertion sign to
delegate complete operand/scope validation to the positive proof. References,
other class domains, word gates, repetitions and continuation terms decline.
The projection is never independently validated or rebound.

At each legal Unicode candidate, a cancellable sticky positive probe tests the
whole original-context conjunction. Successful negation exports the candidate's
empty range, empty outside captures and undefined operand captures (22.2.2.4).
An accepted empty member makes negation fail at that candidate. Failed probes
do not search a suffix internally; candidate advancement uses complete input
characters, with only the approved initial low-surrogate rewind. Long prefix
traversal remains interruptible, and no input-sized capture state is allocated.


Runtime selects negative variable-length single-q lookahead only as a checked
v-mode fallback after full original Pattern/flags validation, retaining the
complete normalized capture scope and source. Work/capacity errors use the
existing host boundary; enclosing offsets reset only on proof success. Search
caches the empty root with no operand range or dynamic capture state. Capture
lookup retains outside empty spans and named undefined operand slots through
RegExpBuiltinExec, String consumers, copies and collection. User coercions
remain outside the compilation frame.


A separate negative single-q lookahead constructor admits operand word predicates
with optional ^/$ gates. At least one word predicate is required. Sign-only
projection delegates to the checked positive word-predicate proof, so negation
applies to the whole original-context conjunction, including all longer/shorter/
empty endpoint retries. Failure of a boundary can make negation succeed even
when the class has an empty member. Successful negative matches retain empty
outside spans and undefined operand slots; candidate traversal and cancellation
remain shared with the earlier negative string proof (22.2.2.4).


Runtime selects negative variable-length single-q lookahead with word predicates
as a separate checked v-mode fallback after original validation. It uses the
complete normalized capture scope, retains the earlier checked host errors and
caches only the empty assertion range. Named undefined operand slots and empty
outside captures retain their indices through consumers, copies and collection;
original-context endpoint conjunctions and complete Unicode candidates remain
Core's responsibility.


### Positive lookahead over adjacent q operands

A separate positive string-class constructor proves complete lookahead over a
class containing at least two adjacent ClassStringDisjunction operands, with an
empty or longer member overall. It reuses the checked string-union compiler,
whose shared trie implements set union and longest-member priority across all
operands (22.2.2.7, CompileToCharSet and CompileAtom). Source ordering across
operands does not override longer-member priority; duplicate members retain
ordinary membership semantics. An earlier empty choice still precedes a later
search candidate.

Optional ^/$ gates and complete wrappers preserve original context, empty
assertion spans and independent selected-member captures. Full original v/vi
validation and normalized capture heads remain prerequisites. The shared wrapper
compiler uses an explicit class-domain selector; earlier single-q constructors
retain their domains and work sequences. Scalar operands, nesting, operations,
word gates, repetition, continuation terms and references require other proofs.
Checked source copying, decoding/folding, cancellation and capacity errors keep
the established boundaries.


Runtime selects adjacent-q-union lookahead as a separate checked v-mode fallback
after full original Pattern/flags validation and earlier proofs. The complete
normalized capture scope retains original bindings; checked errors use the
existing host boundary and enclosing offsets reset only on proof success. The
existing positive string-class cache retains independent selected-member spans,
including defined empty ranges, through named indices, consumers, copies and
collection. Reentrant conversions remain outside compilation.


A separate q-union lookahead constructor admits operand word predicates with
optional ^/$ gates, requiring at least one predicate and at least two q operands
with an empty or longer member overall. It reuses the same checked union trie
and complete-wrapper proof. Endpoint predicates inspect original input, so a
longer member in a later operand can fail while a shorter member in another
operand succeeds. The selected whole-member capture extent remains independent
of the empty assertion span; empty alternatives retain ordinary candidate
priority (22.2.2.4, 22.2.2.7). Earlier constructor domains and work are unchanged.


Runtime selects q-union lookahead with word predicates as a separate checked
v-mode fallback after full original validation, retaining complete normalized
capture scopes and the established host error boundary. The existing positive
string cache keeps original selected-member extents, including shorter retries
across operands and defined-empty spans, through named indices, consumers,
copies and collection. Original source and reentrant conversion frames retain
their existing boundaries.


### Negative lookahead over adjacent q operands

A separate checked negative constructor accepts complete lookahead over at least
two adjacent q operands, with an empty or longer member overall and optional
operand ^/$ gates. Full original v/vi validation and normalized capture heads
precede sign-only projection. The checked positive union proof validates the
complete class and wrapper scope without independently validating or rebinding
the projection. Successful negation means no member satisfies the original-
context conjunction at that candidate (22.2.2.4, 22.2.2.7).

The existing cancellable sticky-probe search retains complete Unicode candidates,
empty outside captures and undefined operand slots, including when union members
share prefixes or the class contains an empty string. An explicit private class-
domain selector keeps earlier single-q constructor domains and work sequences
unchanged. Other class domains, word gates, repetition, continuation terms and
references require separate proofs.


Runtime selects negative q-union lookahead as a checked v-mode fallback after
full original validation, retaining complete normalized capture scopes and
original source. It uses the existing negative string-class matcher and cache:
only the empty assertion range is retained, with outside empty captures and
named undefined operand slots. Checked host errors, enclosing-offset reset on
proof success, consumers, copies and collection retain the established behavior.
Reentrant conversions remain outside the compilation frame.


A separate negative q-union constructor admits operand word predicates with
optional ^/$ gates, requiring at least one predicate and two q operands with
an empty or longer member overall. Sign-only projection delegates to the
checked positive word-union proof, so successful negation means every member
fails the complete original-context conjunction, including shorter endpoint
retries across operands. Empty members alone do not guarantee failure when
predicates reject their endpoints. Outside spans remain empty and every operand
slot remains undefined; candidate search and work cancellation retain the
earlier behavior (22.2.2.4, 22.2.2.7).


Runtime selects negative q-union lookahead with word predicates as a checked
v-mode fallback after full original validation. Complete normalized capture
scopes retain original bindings; checked errors and enclosing-offset reset on
proof success use the established boundaries. The existing negative string
cache retains only the empty assertion range, with outside empty spans and
named undefined operand slots through indices, consumers, copies and collection.
Original-context conjunctions and complete Unicode candidates remain Core's
responsibility.


### Positive lookahead over literal characters and q strings

A separate positive string-class constructor proves complete lookahead over
ordinary literal characters beside q operands, requiring an empty or longer
q member overall. The checked character-union compiler shares prefix storage
and longest-member priority across one-character literals and strings; a literal
member can supersede an empty alternative or satisfy an endpoint rejected by a
longer string (22.2.2.4, 22.2.2.7). Source ordering does not override membership
priority.

Optional ^/$ gates and complete wrappers preserve original context, empty
assertion spans and independent selected-member captures. Full original v/vi
validation and normalized heads remain prerequisites. The private class-domain
selector retains earlier domains and work sequences. Character-class escapes,
properties, ranges, nesting, operations, word gates, repetitions, continuations
and reference roles require separate proofs. Checked construction and cancellable
search retain the existing host boundaries and original UTF-16 extents.


Runtime selects literal-character/string-union lookahead as a separate checked
v-mode fallback after full original validation. Complete normalized capture
scopes retain original bindings; checked errors and enclosing-offset reset on
proof success use the existing boundaries. The positive string-class cache
retains selected literal or string extents, including defined-empty members,
through named indices, consumers, copies and collection. Original source and
reentrant conversion frames retain their established behavior.


A separate character/string-union constructor admits operand word predicates
with optional ^/$ gates, requiring at least one predicate, a literal character
and an empty or longer q member. The checked union trie retains membership
priority across literals and strings while predicates inspect original
endpoints. A rejected longer string can select a shorter literal or an empty
member, retaining independent operand capture extents and original UTF-16
spans (22.2.2.4, 22.2.2.7). Earlier domains and work remain unchanged.


Runtime selects character/string-union lookahead with word predicates as a
separate checked v-mode fallback after full original validation. Complete
normalized capture scopes retain original bindings; checked errors and
enclosing-offset reset on proof success use the established boundaries. The
positive string cache retains selected literal/string extents and defined-empty
members through named indices, consumers, copies and collection, with original
source and reentrant conversion behavior preserved.


### Negative lookahead over literal characters and q strings

A separate checked negative constructor accepts a literal-character/string
union with an empty or longer q member and optional operand ^/$ gates. Full
original v/vi validation and normalized capture heads precede sign-only
projection. The positive character-union proof validates complete membership
and wrapper scope without independently validating or rebinding projected source.
Successful negation means neither a literal nor a string member satisfies the
original-context conjunction at that candidate (22.2.2.4, 22.2.2.7).

The existing cancellable sticky-probe search retains complete Unicode candidates,
empty outside captures and undefined operand slots. The private domain selector
keeps earlier single-q and q-union constructors and their work unchanged. Word
gates, scalar escapes/properties/ranges, nesting, operations, repetition,
continuations and references retain separate proof requirements.


Runtime selects negative character/string-union lookahead as a checked v-mode
fallback after full original validation. Complete normalized capture scopes
retain original bindings; checked errors and enclosing-offset reset on proof
success use the existing boundaries. The negative string-class matcher and
cache retain only the empty assertion range, outside empty spans and named
undefined operand slots through indices, consumers, copies and collection.
Literal and string membership both participate in the original-context probe.


A separate negative character/string-union constructor admits operand word
predicates with optional ^/$ gates, requiring at least one predicate, a literal
character and an empty or longer q member. Sign-only projection delegates to
the checked positive proof. Successful negation means every literal and string
member fails the complete original-context conjunction, including shorter
endpoint retries. Empty membership does not guarantee failure when predicates
reject. Outside captures remain empty, operand slots remain undefined, and
candidate/work behavior is shared with earlier negative modes (22.2.2.4,
22.2.2.7).


Runtime selects negative character/string-union lookahead with word predicates
as a checked v-mode fallback after full original validation. Complete normalized
capture scopes retain original bindings; checked errors and enclosing-offset
reset on proof success keep the established boundaries. The negative string
cache retains only the empty assertion range, with outside empty captures and
named undefined operand slots through indices, consumers, copies and collection.
Original-context literal/string conjunctions and Unicode candidates remain
Core's responsibility.


### Positive lookahead over scalar operands and q strings

A separate checked positive constructor proves complete lookahead over scalar
class escapes, properties and ranges beside q operands, requiring an empty or
longer q member. It delegates membership to the existing scalar-union compiler,
which combines folded scalar intervals with shared string-prefix storage. The
longest accepted member wins at the earliest candidate; an empty member can win
before a later consuming candidate (22.2.2.4, 22.2.2.7, 22.2.2.9).

Optional ^/$ gates inspect original input and can reject a longer string in
favor of a scalar member. Complete wrappers preserve empty outside spans and
independent whole-operand extents, including defined-empty captures. Full original
v/vi validation and normalized capture heads precede projection; source is never
independently rebound. The private class-domain selector preserves earlier
constructor domains and work sequences. Word predicates, nesting, operations,
repetition, continuations, references and reverse matching require separate
proofs. Checked construction and cancellable traversal use existing host
boundaries without input-sized capture state.


Runtime selects scalar/string-union lookahead as a separate checked v-mode
fallback after full original validation. Complete normalized capture scopes
retain original bindings; checked errors and enclosing-offset reset on proof
success use established boundaries. The positive string cache preserves
selected scalar/string extents and defined-empty members through named indices,
String consumers, copies and collection. Original source and reentrant conversion
frames retain their established behavior.


A separate scalar/string-union constructor admits operand word predicates and
optional ^/$ gates, requiring at least one predicate, a scalar operand and an
empty or longer q member. Folded scalar intervals and shared string prefixes
retain longest accepted membership while predicates inspect original endpoints.
Rejected longer strings can select shorter scalar or empty members, with
independent whole-operand captures and original UTF-16 spans (22.2.2.4, 22.2.2.7).
Earlier constructor domains and work sequences remain unchanged.


### Negative lookahead over scalar operands and q strings

A separate checked negative constructor accepts scalar class escapes, properties
and ranges beside q operands, with an empty or longer q member and optional
operand ^/$ gates. Full original v/vi validation and normalized capture heads
precede sign-only projection. The checked positive proof validates folded
scalar/string membership and complete wrapper scope without independently
validating or rebinding projected source. Negation succeeds only when every
member fails the original-context conjunction at that candidate (22.2.2.4,
22.2.2.7, 22.2.2.9).

The existing cancellable sticky-probe search preserves complete Unicode
candidates, empty outside spans and undefined operand captures. The private
domain selector preserves earlier constructors and their work sequences. Word
gates, nesting, operations, repetition, continuations and reference roles
retain separate proof requirements.


A separate negative scalar/string-union constructor admits operand word
predicates and optional ^/$ gates, requiring at least one predicate, a scalar
operand and an empty or longer q member. Sign-only projection delegates to
the checked positive proof. Successful negation means every scalar and string
member fails the complete original-context conjunction, including shorter
endpoint retries. Empty membership does not guarantee failure when predicates
reject. Outside captures remain empty and operand slots remain undefined;
candidate and work behavior are shared with earlier negative modes
(22.2.2.4, 22.2.2.7).
