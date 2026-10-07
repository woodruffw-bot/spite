# Implemented coverage

spite is an initial executable subset, not a conforming ECMAScript engine.
The target is ECMAScript 2026, edition 17. This file records implementation status,
not an alternative language specification.

## Available now

| Area | Implemented behavior |
| --- | --- |
| Source | UTF-8 Scripts and lossless UTF-16 eval/Function input, byte spans, distinct syntax, unsupported, and limit diagnostics |
| Strings | UTF-16 code units, lone surrogates, ordinary escapes, Unicode escapes, line continuation |
| Lexical grammar | On-demand parser lookahead, explicit scanner goals and RegExp token boundaries, ECMAScript whitespace and line terminators, comments, initial hashbang, Unicode 18.0.0 identifiers, Unicode identifier escapes |
| Numbers | Decimal, binary, octal, hex, numeric separators, binary64 rounding and overflow |
| Legacy literals | Leading-zero octal and decimal numbers, octal/decimal string escapes in non-strict code, and strict early errors including escapes before a use-strict directive |
| BigInt | Exact literals in all four radices, signed arithmetic, truncating division and remainder, exponentiation, arithmetic shifts, infinite sign-extension bitwise operations, updates and compound assignment |
| BigInt coercion | Boolean and decimal string conversion, integer-string equality and ordering, exact mixed Number comparisons, TypeError for mixed numeric arithmetic and unary plus, RangeError for zero division and negative exponents |
| BigInt APIs | Calls with exact integral Number and integer-string conversion, signed/unsigned width reduction, wrappers and boxed receivers, branded valueOf/toString/toLocaleString, and observable prototype tags |
| Expressions | Primitive, object, and array literals, untagged/tagged templates with substitutions, identifiers, this, parentheses, simple/compound and object/array destructuring assignment, prefix/postfix updates, conditional/comma expressions and optional property/call chains |
| Arrays | Calls/new, of, from, isArray, literal holes/spread and trailing commas, sparse indexed properties, ordered length coercion, read-only length, partial truncation, generic at/join/push/pop/shift/unshift/reverse/fill/copyWithin and includes/indexOf/lastIndexOf, forEach/every/some and reduce/reduceRight callbacks, species-aware map/filter/slice/concat/flat/flatMap/splice, find/findIndex/findLast/findLastIndex, sort/toSorted and toReversed/with/toSpliced copies, dynamic toString, and toLocaleString |
| Object literals | Literal and computed keys, shorthand, ordered data properties and spread, duplicate-key replacement, required prototype initializers, identity equality and truthiness |
| Object coercion | Realm-level ordered method lookup, TypeError for objects without callable conversion methods, left-to-right operand conversion, Object.prototype toString/valueOf dispatch |
| Calls | Builtin, arrow, and ordinary calls, standard Function intrinsic graph/metadata/branding, strict/global/boxed receivers, ordered callee/argument evaluation, member receivers, Object.prototype conversion methods |
| Arrow functions | Expression/block bodies, identifier/pattern/default/rest parameters, closures, local declarations, return completions, strict directives, name/length metadata, and source stringification |
| Instance checks | instanceof for ordinary/bound functions with ordered prototype lookup; materialized Symbol.hasInstance and custom hooks tested through native symbol injection |
| Construction | new with optional arguments and nested/member precedence, ordinary and bound constructors, prototype selection, object/primitive return rules, and ordered evaluation |
| Reflect | All thirteen methods for exposed object kinds, exact call/write receivers, custom newTarget, boolean rejection, complete own reflection, and standard tag/attributes |
| Math | Fixed constants/tag, abs/sign, integral and binary32/binary16 rounding, extrema, clz32/imul, pow/sqrt/cbrt/hypot, logarithmic/exponential, trigonometric/hyperbolic methods, exact iterable sumPrecise, realm-specific random sequences, and complete own reflection |
| Function syntax | Ordinary named/anonymous function expressions and named declarations, identifier/pattern/default/rest parameters, body early errors, and variable versus block scope, declaration instantiation/hoisting, and standard prototype/name/length properties; Boolean, Number, String, Symbol, and BigInt receivers box |
| Classes | Base/derived declarations and expressions, ordered heritage and prototype checks, explicit/default constructors, uninitialized this and super calls, strict instance/static methods and accessors, class-name TDZs, immutable internal names, inferred names, public/private fields and static blocks, private-element grammar/early errors, private brands and eval, descriptors, newTarget, and exact source retention |
| Properties | Ordinary own/inherited and super references, separate super receivers, ordered reads/writes/updates/deletion, primitive property operations, UTF-16 String indices and length, strict write/delete failures |
| Operators | Arithmetic, exponentiation, bitwise, shifts, equality, primitive comparison, logical and nullish operators, typeof, void, delete, and in |
| Statements | Empty and expression statements, let, const, and var, blocks, if/else, while, do-while, three-clause for, synchronous for-of, and for-in over complete prototype chains with assignment/var/lexical bindings, non-strict with, switch, labels, break/continue with optional targets, function-body return, throw, try with catch and/or finally; catch identifiers and object/array binding patterns bind supported throws |
| Static semantics | Implemented ASI rules, strict directives, duplicate lexical bindings, strict binding and assignment restrictions, escaped reserved-word checks, enclosing-loop/switch checks, duplicate labels, control-target validation, duplicate defaults and case-block lexical names |
| Runtime | Persistent realm state, lexical scope, declaration instantiation, per-iteration scopes, temporal dead zones, immutable bindings, ordered evaluation and synchronous iterator closing |
| Completions | Empty versus undefined, statement-list values, if-statement UpdateEmpty, loop body values, return and break/continue propagation through blocks, conditionals, nested loops, and switch fall-through, primitive and object throws, finalizer preservation and overrides of language completions |
| Global values | Ordinary global object, globalThis, Script/arrow global this, undefined, NaN, Infinity, and property-backed global bindings |
| Eval | Standard intrinsic metadata, non-String identity returns, direct/indirect String execution, caller/global lookup and declaration checks, fresh lexical/strict var environments, deletable eval-created bindings, inherited this/new.target, exact UTF-16 source, and restored caller contexts |
| URI handling | String-hint conversion, exact URI/component escape sets, UTF-8 encoding and strict decoding, reserved-escape preservation, and intrinsic URIError exceptions |
| Set | Canonical values, ordered hash storage, construction and closing, live iteration/forEach, union/intersection/difference/symmetricDifference, set-like predicates, and full reflection |
| WeakSet | Object and non-registered Symbol identities, non-retaining hash storage, iterable construction/closing, branded add/has/delete, subclassing, and complete reflection |
| Map | Canonical keys, hash-indexed ordered storage, construction and closing, branded keyed methods and size, live keys/values/entries and forEach, computed insertion, intrinsic groupBy, and full reflection |
| RegExp | String-only escape encoding; native construction and literal evaluation, original slots, getters and branding; ordinary-mode literal matching, fixed class/escape/dot sequences and top-level alternatives with outer anchors and word boundaries, single-atom greedy/lazy quantifiers with fixed ordinary continuations, ordered captures, lastIndex state and match/indices Arrays; generic test/match/search/replace/split/matchAll and a branded iterator; broader Pattern matching remains pending |
| JSON | Exact ECMA-404 parsing, intrinsic value creation, iterative revivers with primitive source contexts, frozen raw JSON values, and iterative stringify with replacers, indentation, raw embedding, cycles, and full reflection |
| Limits | Opt-in source size, evaluation/arithmetic work, string code units, BigInt magnitude bits, shared object/environment heap slots, properties per object, and call argument count; checked platform capacity and native-stack guards |
| Tests | Algorithm and integration tests, AST and diagnostic snapshots, 13494 reviewed Test262 variants, ten pinned harness files, plus 13 identifier lexer and 6 statement parser fixtures |

Debugger statements parse with ordinary ASI and produce an empty completion.
This host has no active debugging facility (ECMA-262 14.16.1).

Five eval regressions cover non-String primitives and object identity without
conversion, direct/indirect and call/apply/bound calls, metadata/descriptors,
construction rejection after argument effects, global preservation/replacement
and deletion, shadowing, collection, and unsupported eval syntax results that
bypass JavaScript handlers. Four UTF-16 Script parser regressions and two inspected
insta snapshots cover shared Script grammar, exact source values, constructor and
control-flow early errors, and super's method/arrow context boundaries. Eleven
indirect-eval runtime regressions cover call classification, global lookup and this,
own strictness, completion values, declaration descriptors and preflight failures,
fresh lexical environments and TDZs, captured closures, live initializer references,
source preservation, caller restoration after throws, collection, unlimited default
source sizes, shared opt-in work/source/heap quotas, and native re-entry guards.
Two direct-eval parser regressions and two inspected insta snapshots cover inherited
strictness, new.target, invalid return, and method/arrow super context boundaries.
Twelve runtime regressions cover direct reference forms, ordered arguments, caller
lookup, strict local vars, fresh lexicals and TDZs, preflight lexical/catch/parameter
conflicts, deletable declarations, preserved mapped parameters, live with references,
ordinary/arrow default and body environments, this/new.target, nested eval, abrupt
caller restoration, exact UTF-16 source, collection, and shared opt-in limits.
The unchanged pinned inventory contains eight eval intrinsic, 50 indirect eval,
and 132 direct eval sources: 290 harness-positive and four runtime-negative
variants. The direct selection reviews whole synchronous ordinary/arrow/method
programs, including default-parameter arguments conflicts, lexical and variable
scope, declaration preflight, deletable bindings, this/new.target, and early
errors. Its 154 excluded whole files require async/generator execution (145),
modules (two), classes (two), or complete global reflection through the unchanged
property helper (five). Thirteen other intrinsic/indirect candidates remain
excluded without credit, including private
identifiers and cross-realm execution. Historical descriptions and Annex-B labels
are preserved; strict block/switch controls exercise core lexical function scope.
All three runtime-negative originals have valid outer Scripts and require
SyntaxError during eval compilation or declaration instantiation. The inventory
preserves each runtime phase, and the runner requires the original expected
constructor name. Both stable and MSRV pass all 294 eval variants under
ordinary unlimited defaults; the pin and eight helpers are unchanged.

Left-associated binary expression chains now parse and evaluate without native
recursion along their left spine (13.6–13.13, 13.16). Three parser regressions
include an inspected ten-record insta snapshot for grouping and prefix spans,
strict/nullish early errors, and parsing, cloning, comparison and dropping of
100,000 operations. Two existing precedence snapshots were inspected after the
AST representation changed. Seven runtime regressions cover precedence,
right-associated exponentiation, evaluation/coercion order, short circuiting,
comma call receivers and indirect eval, String/BigInt results, 100,000 arithmetic
operations, 1,000 string concatenations, and opted-in host aborts. Ten independent
Node programs confirm the semantic expectations. Genuine recursive syntax retains
the existing stack guards; no default quota or test-only allowance is added.
Stable and MSRV pass the complete 13494-variant corpus, workspace and documentation
tests, and the debug-symbol stack checks. Formatting, Clippy, dependency policy,
fixture inventory and the pinned Unicode/numeric generator checks also pass.

With statements now use live Object Environment Records (14.11, 9.1.1.2).
Four parser regressions and two inspected insta snapshots cover grammar, strict
early errors, nested declaration conflicts, and control targets. Nine runtime
regressions cover own/inherited/nonenumerable bindings, live unscopables truthiness
and receiver-aware getters, single-resolution typeof, mutation between lookup and
Get/Set/delete, ordinary/optional/tagged call receivers, closure capture and GC,
global dynamic Function scope, primitive boxing, and completion/scope restoration
after language exceptions and opted-in host aborts. Strict closures distinguish
disappeared bindings from non-writable properties. Annex B extensions are excluded.
An additional heap regression validates foreign/stale/wrong-kind binding objects,
opted-in work exhaustion, and tracing of the binding object and outer environment.

Catch binding patterns now implement ordered computed keys, shorthand, nested
patterns/defaults, array elisions, and object/array rest (14.3.3, 14.15.2). Seven
parser regressions and two inspected insta snapshots cover grammar, BoundNames,
duplicates, strict/default validation, lexical/var conflicts, and native depth.
Nine runtime regressions cover GetV receiver/order, temporal dead zones, default
function names, exact String/Symbol rest exclusions and live descriptors, primitive
and nullish sources, cached iterator methods, skipped elision value getters,
normal/abrupt closing and nested throw precedence, captured scopes and collection,
a 12,000-name pattern with ordinary unlimited defaults, and opted-in host aborts.
Formal and assignment patterns use the implementations described below.
The unchanged pinned corpus adds 77 catch-destructuring sources and five reviewed
catch-parameter early errors: 142 harness-positive and 20 parse-negative variants.
All 93 destructuring originals were reviewed; fourteen require generators and
two require classes and remain excluded as whole files without credit. The other
top-level try sources remain a separate review. Exact diagnostic spans identify
strict binding names, duplicate/conflicting names, rest initializers, and non-final
rest elements. Both stable and MSRV pass all 162 new variants with ordinary
unlimited defaults and the existing pin and eight unchanged helpers.

Lexical declaration patterns share the catch pattern grammar and initialization
engine. Let/const declaration lists, three-clause loops, and for-in/of headers
support nested/default/rest bindings. All bound names participate in early
conflicts, declaration instantiation, RHS TDZs, and fresh iteration environments;
const names are immutable. Six parser regressions and two inspected new insta
snapshots cover these contexts, initializer requirements, BoundNames, conflicts,
strictness, grammar parameters, and depth. Existing identifier snapshots retain
their names and spans in the shared pattern AST. Nine runtime regressions cover
ordered mixed declarations, named defaults, primitive receivers/rest, skipped
elision getters and iterator closing, immutable names, arguments shadowing,
per-iteration closures, partial initialization and captured TDZs across collection,
global conflict checks, a 12,000-name default pattern, and opted-in host aborts.
The unchanged pinned corpus adds 154 lexical declaration pattern sources: 284
harness-positive and 24 reviewed parse-negative variants. All 186 let/const
destructuring originals were reviewed; 28 require generators and four require
classes and remain excluded as whole files without credit. Exact let/const pair
comparison and upstream Git blob hashes verify the original bytes. Negative
diagnostics identify forbidden rest initializers or following elements at their
original tokens. Both stable and MSRV pass the new 308 variants and the existing
162 catch and 238 loop variants with ordinary unlimited defaults. Lexical loop
pattern fixtures are recorded in the synchronous loop inventory below.

Var declaration patterns now share the pattern engine while using ordered
ResolveBinding/PutValue. Script/function hoisting collects every nested bound
name, including repeats; statements and all synchronous loop declarations accept
patterns. Six parser regressions and two inspected new insta snapshots cover
grammar, missing initializers, strictness, every-name lexical/catch conflicts,
VarDeclaredNames order, loop restrictions, and depth. Nine runtime regressions
cover hoisting, repeated names, mapped arguments, RHS/reference order, computed
keys, unscopables, references retained across source mutation, elisions, nested
and rest bindings, strict writes and iterator closing, shared var loop scopes,
partial writes and collection, early global conflicts, a 12,000-name pattern with
ordinary unlimited defaults, and opt-in host aborts.
The unchanged pinned corpus adds 77 var declaration pattern sources: 142 harness
positives and twelve reviewed parse negatives. All 97 originals were reviewed;
fourteen require generators and six require classes and remain excluded as whole
files without credit. The class exclusions include four static-block await cases.
Exact body/metadata comparison with the reviewed let originals and original Git
blob hashes verify the source inventory; var's assignment-mode initialization is
reviewed separately. Both stable and MSRV pass the 154 new variants, 308 lexical
declaration variants, 162 catch variants, 238 loop variants, and 104 with/strict
Function variants with the existing pin, helpers, and unlimited defaults. Var
loop-pattern fixtures are recorded in the synchronous loop inventory below.

Formal binding patterns now execute in ordinary functions, arrows, methods,
setters, constructors, and dynamic Function compilation. Six parser regressions
and two insta snapshots cover nested/rest grammar, all-name early errors,
ContainsExpression versus top-level defaults, and native-depth diagnostics.
Nine runtime regressions cover ordered binding/default evaluation, parameter
TDZ, anonymous names, rest Arrays and observable iterator hooks, unmapped
arguments and arguments-name suppression, length/source metadata, closures from
computed keys and nested defaults, iterator close/error precedence, escaped
partial initialization and collection, and a 12,000-name parameter pattern under
ordinary unlimited defaults.
The unchanged pinned corpus adds 507 formal-pattern sources: 154 function
declarations, 154 function expressions, and 199 arrows. Their 1,006 variants
comprise 852 harness positives and 154 reviewed parse negatives. All 603 originals
were reviewed; 84 generator-dependent and twelve class-dependent originals remain
excluded as whole files without credit. Exact comparison verifies the repeated
context templates, metadata, and case algorithms against the reviewed originals.
The six rest grammar cases reject the intended equals/comma in each default and
non-default context; 45 additional arrow negatives reject reserved identifiers,
with the original eight onlyStrict flags preserved. Both stable and MSRV pass
these variants plus 921 catch, declaration, and Function control variants under
the existing pin, helpers, and unlimited defaults. Method/setter parameter and
other formal rest fixtures remain separate reviews.

Destructuring assignments now support object/array patterns, nested/default/rest
targets, repeated names, member references, and synchronous for-in/of assignment
heads. Six parser regressions and two inspected insta snapshots cover supplemental
cover refinement, ordinary literal grammar, strict and invalid targets, rest
restrictions, grammar parameters, unavailable syntax, and native depth. Nine
runtime regressions cover RHS identity, ordered reference/GetV/step/default/write
effects, edition-17 deferred target-key conversion, retained references across
source mutation and with lookup, dense array rest, exact object rest exclusions
and live descriptors, default function names, partial writes and strict errors,
nested iterator closing and incoming-throw precedence, outer loop closing,
collection, a 12,000-name pattern under ordinary unlimited defaults, and opted-in
host aborts without JavaScript cleanup.
The unchanged pinned corpus adds 335 assignment-pattern sources: 258 harness
positives and 77 reviewed parse negatives, for 445 positive and 130 negative
variants. All 368 originals were reviewed; thirty require generators and three
require classes and remain excluded as whole files without credit. One ordinary
function-name original has a historical class feature annotation but contains no
class syntax; it is preserved unchanged and executed normally. Original Git blob
hashes verify every selected file. Negative spans identify invalid nested/member
targets, optional chains, rest separators/initializers, strict names, and escaped
reserved shorthand references. Both stable and MSRV pass all 575 new variants and
2,269 binding, iteration, with, and Function control variants under the existing
pin, eight unchanged helpers, and ordinary unlimited defaults.

The synchronous loop pattern inventory adds 32 for-in parse-negative sources and
482 for-of sources, producing 970 variants: 826 harness positives and 144 reviewed
parse negatives. The for-of inventory includes 245 assignment sources and 79 each
of var, let, and const binding sources. Complete programs, context algorithms,
metadata, and exact negative diagnostics were reviewed; template comparisons and
Git blob hashes verify the unchanged originals. All 602 candidates were reviewed.
Seventy-nine generator-dependent and nine class-dependent whole files remain
excluded without credit. Six excluded class programs lack the class feature tag;
one retained ordinary function-name original has that tag but no class syntax.
Both stable and MSRV pass all 970 selected variants with the existing pin, eight
helpers, and ordinary unlimited defaults. Unsupported syntax cannot satisfy a
negative.

Identifier names retain their exact decoded code point sequence. Canonically
equivalent spellings are distinct bindings. Escapes cannot turn reserved words
into identifiers or stand in for grammar keywords. Unicode property tables are
generated from pinned data and do not depend on the Rust toolchain version.

The parser now owns the scanner and requests cached lookahead only as needed.
All Script, lossless UTF-16, eval, and dynamic Function entry points propagate
encountered lexical failures, including failures that appear syntactically to
terminate input or occur during cover/computed-name lookahead. Three regressions
and an inspected diagnostic snapshot exercise these boundaries and preserve
Unsupported for recognized unavailable syntax before unrequested invalid input.
Consumed tokens retain strict legacy checks, exact source spans, escapes, and
line-terminator trivia. The existing native-depth guard is unchanged.

The scanner now accepts all five ECMA-262 lexical goals explicitly. Parser-owned
substitution context selects division/template-tail goals for buffered lookahead;
initial input uses HashbangOrRegExp. Primary-expression solidus tokens rescan with
a RegExp goal, discarding diagnostics from Div-goal scans of literal contents.
Boundary scanning follows 12.9.5 for escaped characters, classes, solidus,
non-terminator body characters, and IdentifierPartChar flags. Raw bodies and flags
preserve supplementary and unpaired UTF-16 source code points; escaped flags are
not consumed as IdentifierPartChar. Comments, hashbang placement, trivia, division
and division-assignment retain their own grammar. Four scanner regressions and
five parser regressions cover goal selection, exact text, every line terminator,
unfinished tokens, templates, cover/computed-key contexts, and distinct diagnostics.
Two diagnostic snapshots were inspected. Computed class accessor lookahead retains
its parsed key and private-name uses for the accessor or following ASI field;
nested names are parsed once. A runtime regression verifies ordered single key
evaluation, getters/setters, ASI fields, outer arguments, and private captures.
Valid RegExp literals now produce AST values; broader native matching remains
Unsupported at runtime and receives no execution credit.

Arrow heads now use the parameter and expression grammars for lookahead.
Four public regressions and three parser-internal regressions cover nested
literal boundaries, division, templates, computed class names, Script/UTF-16/eval
and dynamic Function goals, invalid flags/Patterns, line terminators, private
scope rollback and exact replay, contextual memo reuse, scanner errors, and
existing native depth guards on a two-mebibyte stack. The inspected, retained
19-record syntax snapshot and pinned rejection expectations keep their exact
parameter diagnostics, including forbidden rest initializers,
rest continuations and escaped reserved shorthand names. Reserved statement
keywords cannot begin expressions; dynamic import retains its Unsupported boundary.
Valid RegExp literals retain their AST through head recognition. The obsolete
Unsupported-only snapshot is retired. Both compiler versions run the complete
corpus with unchanged variants and ordinary unlimited defaults.

Destructuring assignment and for-in/of covers now probe the supplemental grammar
instead of balancing division-goal punctuation. Four public regressions and three
parser-internal regressions cover computed/default/nested keys, literal bodies
containing closers and template delimiters, division, member/rest references,
methods/accessors, preserved prototype/shorthand/spread rules, Script/UTF-16/eval
and dynamic Function goals, private-use rollback/replay, both memo-table
invalidations, and existing guards on a two-mebibyte stack. The inspected, retained
16-record syntax snapshot preserves every rejection. Lexically complete invalid
Patterns/flags retain their RegExp rejection point through both arrow and
destructuring probes. All retained snapshots and 1469 reviewed parse-negative variants keep their
exact expectations. Both compiler versions pass the complete unchanged corpus
with ordinary unlimited defaults. The obsolete Unsupported-only snapshot is
retired; the retained syntax snapshot keeps all sixteen exact rejection records.

Core RegExp Canonicalize shares the pinned Unicode 18 default uppercase data and
adds all 1533 simple/common case-folding mappings from the digest-verified official
CaseFolding.txt. Three regressions cover every UTF-16 code unit's ordinary-mode
range, ASCII barrier and idempotence, every folding entry, exact-case preservation,
full/Turkic exclusions, supplementary points and surrogate identity. An inspected
39-record insta snapshot covers the mode differences, including long s, Kelvin,
ohms, sigma, sharp s, Cherokee, Georgian, Cyrillic Tje, Deseret and Garay.
Independent parsing of the raw UCD files verifies every snapshot value and confirms
that all four existing casing/property tables are byte-for-byte unchanged. The
existing generator check covers the additional input locally and in CI. Both
compiler versions pass the full unchanged corpus with ordinary
unlimited defaults; broader native matching remains pending and this core step
adds no new fixture credit.

Core ordinary-mode literal Pattern compilation and matching have four regressions.
An inspected 92-record insta snapshot covers empty and overlapping matches,
sticky offsets, past-end offsets, all implemented character escapes, surrogate
units, supplementary pairs, line terminators, pinned ignore-case boundaries and
explicit unsupported productions. An independent JavaScript engine confirms all
352 search/sticky results in its 44 supported records. Exhaustive short UTF-16
inputs and Patterns agree with a separate sliding-window oracle in both case
modes. A 60,001-unit Pattern against 120,001-unit repeated-prefix inputs exercises
successful and failed linear searches, sticky failure and shared program lifetime
with ordinary unlimited defaults. Both compiler versions pass the complete
unchanged corpus; this core step adds no new fixture credit.

Native ordinary-mode literal RegExpBuiltinExec has ten runtime regressions and an
inspected 32-record result snapshot independently verified against a separate
JavaScript engine. Coverage includes ordered input/lastIndex coercions, ToLength
without g/y, strict success/failure/past-end writes, read-only lastIndex and
reentrant conversions. Results and d indices have fresh intrinsic Array
prototypes, exact own data descriptors, undefined groups and UTF-16 ranges,
including individual surrogate halves. Poisoned prototypes, replaced Array and
public source/flag forgeries cannot change native execution. Non-callable exec
uses the native fallback; custom exec retains live lookup. Native test, match,
search, replace, split and matchAll share the matcher, including empty matches and
lazy iteration. A 120,001-unit search, copied matcher and collection regression
exercise unlimited defaults; an explicitly opted-in work abort skips JavaScript
cleanup. The existing slot-lifetime regression also retains a compiled matcher
through collection. Broader Patterns and u/v still report Unsupported after the
specified coercions. Both compilers pass the complete unchanged corpus; literals,
new upstream fixtures and broader matching receive no additional pass credit.

A focused RegExp exec fixture review reads twenty whole programs at the existing
pin. Eighteen newly vendored originals add 36 normal/strict positives: native
literal search and result Array stringification, exec metadata/descriptors,
non-construction and brand rejection across ordinary objects, functions,
wrappers, primitives and undefined. The prior not-a-constructor fixture is
unchanged; one program requiring broader matching remains
outside the corpus. This is not a complete directory review. Original bytes,
assertions and metadata are unchanged, and both compiler versions pass every
selected variant with ordinary unlimited defaults. The pin, ten helpers and all
prior exact negative expectations are unchanged. Corpus totals are 7027 fixtures,
7008 reviewed Script sources and 13494 variants, including 12017 harness positives.

A subsequent focused exec review reads 25 further whole programs at the same pin.
Twelve unchanged originals add 24 normal/strict positives for abrupt input
coercion, numeric/boolean/absent input conversion, ignore-case matching, global
failure and past-end resets, nonglobal lastIndex reads without writes, sticky
failure and successful end-index writes, and non-writable lastIndex rejection.
At that stage, thirteen reviewed originals required alternatives, classes,
quantifiers or wildcard matching and remained excluded; the directory-wide
inventory below records the current selection. Both compilers pass the full selected
cohort and complete corpus with unchanged helpers and unlimited defaults. Totals
are 7027 fixtures, 7008 Script sources and 13494 variants, including 12017 harness
positives. All prior negative expectations and upstream bytes remain unchanged.

The exec-directory review covers all 79 whole programs at the same pin. After
ordinary top-level literal alternatives became available, seven further unchanged
originals add fourteen normal/strict positives for source-order ties,
earliest-position selection, original match Arrays and input coercion. Fifty-three
directory programs are now vendored, including the prior non-construction
original; 26 still need broader execution and remain excluded. Both compilers
pass every added variant with unlimited defaults and the ten original helpers.
Assertions, metadata, upstream bytes, the pin and every prior negative expectation
remain unchanged. Totals are 7027 fixtures, 7008 reviewed Script sources and 13494
variants, including 12017 harness positives.

The ordinary dot matcher enables three further unchanged exec originals and
six normal/strict positives for lastIndex conversion, global writes, nonglobal
reads without writes and sticky starting positions. Both compilers pass all six
added variants with unchanged assertions, metadata, the ten helpers and unlimited
defaults. The directory inventory remains 79 reviewed programs, with 53 vendored
and 26 excluded. Iterative binary-chain parsing lets all three previously
depth-limited concatenation programs parse normally. Fixed sequences enable
S15.10.6.2_A3_T2; quantified literal continuations enable T3. T4 still needs
nested choices and remains excluded without passing credit.
Corpus totals are 7027 fixtures, 7008 reviewed Script sources and 13494 variants,
including 12017 harness positives; the pin and every negative expectation remain
unchanged.

Fixed ordinary sequences add four unchanged whole exec programs and eight
normal/strict positives: S15.10.6.2_A1_T13, T20, T21 and A3_T2. They verify
Boolean/undefined input conversion, class ranges, original match Array contents,
index/input properties and eleven successive global matches through the complete
original poem. Both compilers pass all eight selected variants with the ten
unchanged helpers, original assertions/metadata and unlimited defaults. The exec
inventory is 53 of 79 whole programs vendored, with 26 exclusions receiving no
passing credit. Totals are 7027 fixtures, 7008 reviewed Script sources, 13494
variants and 12017 harness positives. The pin and every prior negative expectation
remain unchanged.

Single-atom quantifiers add three unchanged whole exec programs and six
normal/strict positives: S15.10.6.2_A1_T19, A3_T5 and A3_T7. They check undefined
input conversion with exact repetition, successive global digit runs and ten
nonglobal calls returning the same first run. All original source blobs were
verified by Git SHA-1 and the manifest retains their SHA-256 digests. Both
compilers pass all six selected variants with unlimited defaults, the same ten
helpers and original assertions/metadata. The exec-directory inventory is now
53 of 79 whole programs vendored, with 26 exclusions receiving no credit.
Totals are 7027 fixtures, 7008 reviewed Script sources, 13494 variants and 12017
harness positives. The pin and every prior negative expectation are unchanged.
Stable and MSRV pass all 13494 corpus variants and workspace/documentation tests.
Formatting, Clippy, policy, fixture inventory and pinned generator checks pass.

Quantified literal continuations add the unchanged whole S15.10.6.2_A3_T3 exec
program and its two normal/strict positives. Its original complete poem requires
twelve successive global [Nn]?evermore matches, including the occurrence without
an initial N. Both compilers pass both variants with unlimited defaults, original
assertions/metadata and the same ten helpers. Git blob SHA-1 and manifest SHA-256
verify the source bytes. The exec inventory is now 53 of 79 whole programs
vendored, with 26 exclusions receiving no credit. Totals are 7027 fixtures, 7008
reviewed Script sources, 13494 variants and 12017 harness positives. The pin and
every prior negative expectation are unchanged. Both compilers pass all 13494
corpus variants and workspace/documentation tests. Formatting, Clippy, policy,
fixture inventories and pinned generator checks pass.

Quantified captures add the unchanged whole S15.10.6.2_A3_T6 exec program and its
two normal/strict positives. The original global digit-capture pattern finds
three successive runs through the complete original loop and assertions. Both
compilers pass both variants with unlimited defaults, original metadata and the
same ten helpers. Git blob SHA-1 and manifest SHA-256 verify the source bytes.
The exec inventory is now 53 of 79 whole programs vendored, with 26 exclusions
receiving no credit. Totals are 7027 fixtures, 7008 reviewed Script sources,
13494 variants and 12017 harness positives. The pin and every prior negative
expectation are unchanged. Stable and MSRV pass all 13494 corpus variants and
workspace/documentation tests. Formatting, Clippy, policy, fixture inventories
and all pinned generator checks also pass.

Literal prefixes add the unchanged whole S15.10.6.2_A1_T3 and T4 exec programs
and four normal/strict positives. The original greedy and lazy bounded patterns
verify boxed String and custom toString input conversion, match Array contents,
index and input. Both compilers pass all four variants with unlimited defaults,
original assertions and metadata, and the same ten helpers. Git blob SHA-1 and
manifest SHA-256 verify the exact bytes. The exec inventory is now 53 of 79 whole
programs vendored, with 26 exclusions receiving no credit. Totals are 7027
fixtures, 7008 reviewed Script sources, 13494 variants and 12017 harness positives.
The pin and every prior negative expectation are unchanged. Stable and MSRV pass
all 13494 corpus variants and workspace/documentation tests. Formatting, Clippy,
policy, fixture inventories and all pinned generator checks also pass.

RegExp literals now produce validated AST values and call intrinsic RegExpCreate
on each evaluation. Three parser regressions and an inspected 15-record insta
snapshot preserve Pattern/flag text, original flag order, escaped solidus,
classes, names, u/v productions, paired/lone/reversed surrogates and encoded byte
spans. An independent engine verifies every record and span. Eight runtime
regressions cover fresh instances and +0 lastIndex, literal metadata, shadowed or
deleted RegExp bindings, Symbol.match poisoning, source spelling, UTF-16 matching,
generic consumers, arrow/destructuring/template contexts, collection and existing
native stack guards. Shared grammar-goal and Pattern tests now require valid ASTs;
all retained syntax snapshots keep their exact contents. Three obsolete
Unsupported-only snapshots are retired, and the flag-error snapshot is renamed
without changing its bytes. Eval regressions keep invalid literals before effects;
valid unsupported matching aborts after preceding effects and skips handlers and
finalizers. Both compilers pass the unchanged complete corpus with unlimited
defaults. No fixture, expected negative diagnostic, dependency or quota is added.

Ordinary literal compilation now also flattens unquantified noncapturing groups.
Three core regressions cover a reviewed 50-record insta snapshot, an independent
sliding-window oracle over nested groups, and iterative compilation, matching and
drop of 100,000 nested groups. The earlier snapshot changes only the two empty
group records. An independent engine agrees on all 280 supported search/sticky
ranges and nine runtime programs. Four runtime regressions preserve original
source and flag text, absence of capture/group slots, UTF-16 indices, global and
sticky lastIndex state, empty matches and advancement through generic consumers.
Other group productions, quantified groups and Unicode matching remain explicitly
unsupported. Both compilers pass the complete unchanged corpus and the native
stack checks with ordinary unlimited defaults. No fixture, dependency, default
quota or parse-negative expectation changes.

Ordinary IdentityEscape compilation now covers punctuation and whitespace using
the pinned Unicode ID_Continue exclusion, including '$', constructor line
terminators and lone surrogate units. Two core regressions inspect a 34-record
insta snapshot and check matching across every supported code unit in the full
UTF-16 domain. An independent engine agrees on all twenty compiled snapshot
matches and seven runtime programs in the shared grammar; it is not used as an
oracle for Annex B identity-escape extensions. Four runtime regressions cover
punctuation and whitespace literals, arbitrary constructor units, surrogate
halves, generic consumers and the preserved identifier/Unicode/class/assertion/
backreference boundaries. Both compilers pass the complete unchanged corpus and
native stack checks with ordinary unlimited defaults. No fixture, dependency,
quota or expected parse diagnostic changes.

The Pattern validator's capture-count bound now rejects the first forbidden
total rather than admitting one extra group. An internal boundary regression
checks the largest permitted count and rejection without counter mutation or
overflow in ordinary, u and v modes, without allocating a huge source. This is
the 22.2.1.1 grammar early error, not a default resource quota. Both compiler
versions pass the complete unchanged corpus; fixture inventories and every
prior negative expectation remain unchanged.

Ordinary unquantified capturing groups now retain relative UTF-16 ranges in
opening-parenthesis order and produce native capture strings and MatchIndices
pairs. Three core regressions inspect a 36-record insta snapshot, decoded offsets
and flat capture order, and iterative compilation/clone/drop of 100,000 nested
captures. Two earlier snapshots change only their two ordinary-capture records.
An independent engine agrees on 44 capture/indices records and fifteen runtime
programs. Eight runtime regressions cover nested/empty groups, original source,
input case and surrogate halves, global/sticky/strict lastIndex, intrinsic own
result properties, replacement callback arguments/substitutions, split/match/
matchAll, constructor copies, collection and 1,000 flat empty captures without
default count quotas. Existing host-gap regressions now use grouped alternatives or
named groups. Both compilers pass the complete unchanged corpus and native stack
checks; no fixture, dependency, quota or negative expectation changes.

Top-level ordinary literal alternatives now execute with earliest-position and
source-order selection, including empty choices, noncapturing branches, pinned
ignore-case rules and arbitrary UTF-16 units. Three core regressions inspect a
42-record insta snapshot, compare all offsets against an independent
position-first oracle, and exercise long repeated prefixes and 100,001 flat
branches without default caps. An independent engine agrees on all 448 supported
search/sticky ranges and thirteen runtime programs. Nine runtime regressions
cover tie ordering, global/sticky lastIndex, empty-match advancement, indices,
all generic consumers and String fallbacks, copies and collection, whole-plan
unsupported boundaries and opted-in per-branch work accounting, including empty
input branch visits. Nested choices, other Pattern productions and Unicode-mode matching
remain pending. No fixture, dependency, default quota or negative expectation
changes.

Top-level ordinary alternatives now retain captures in global source order.
Unselected branch groups produce own undefined match and indices elements; empty
participating groups produce empty Strings and zero-length pairs. Two new core
regressions inspect a 96-record insta snapshot and check iterative compilation,
shared clones and drop of 100,001 flat capturing branches and 100,000 nested
captures without default quotas. The existing independent position-first oracle
also checks selected branch identities for capturing alternatives. The earlier
alternatives snapshot changes only its two ordinary-capture records. An
independent engine agrees on all 96 capture/indices records and sixteen runtime
programs. Eight runtime regressions cover source-order ties, nested/empty groups,
UTF-16 and input case, global/sticky state and strict writes, intrinsic own
undefined descriptors, substitution/callback/split/matchAll behavior, copies and
collection. Both compilers pass the complete unchanged corpus and native stack
checks. Broader choices and other Pattern productions remain pending; no fixture,
dependency, quota or negative expectation changes.

Leading ^ and trailing $ now execute around ordinary literal sequences and
unquantified groups. Three core regressions inspect a 34-record insta snapshot,
compare every start/sticky choice against an independent position/boundary oracle,
and retain overlapping candidates and linear search across long repeated
prefixes. An independent engine agrees on all 440 supported anchor ranges and
twelve runtime programs. Eight runtime regressions cover exact non-multiline
boundaries, all LineTerminators and CRLF positions, sticky state, empty global
advancement, original case/surrogates and capture indices, escaped punctuation,
shared generic consumers, strict lastIndex writes, long copies and collection,
and explicit unsupported boundaries. Existing snapshots remain unchanged. Both
compilers pass the complete unchanged corpus and native stack checks. Internal
assertions, anchored choices and Unicode-mode matching remain pending; no fixture,
dependency, quota or negative expectation changes.

Ordinary single character-set atoms now execute bracket classes, ranges, outer
inversion and empty sets, all six class escapes, class backspace and dot/DotAll.
Four core regressions inspect a 62-record insta snapshot, independently verify
the six class-escape sets across every UTF-16 unit, distinguish opted-in
construction work failures from unsupported syntax, and exercise 100,000 flat
class characters, shared clones and long searches without default quotas. An
independent engine agrees on all 1,012 supported snapshot ranges and twelve
runtime programs. Nine runtime regressions cover pinned ordinary case rules and
inversion, whitespace/word/digit sets, line terminators and surrogate halves,
global/sticky/strict lastIndex and intrinsic results, every shared generic
consumer, large copies and collection, opted-in construction/search aborts and
explicit unsupported boundaries. At that stage, former literal and direct-eval
single-atom host-gap tests used unsupported concatenations; direct eval also
checks successful native matching. Existing snapshots remain unchanged. Both
compilers pass the complete unchanged corpus and native stack checks. Quantifiers
and Unicode-mode sets remain pending; no fixture, dependency, default
quota or negative expectation changes.

Fixed ordinary sequences now combine literal characters, bracket classes, class
escapes and dot atoms with iterative capturing/noncapturing groups. Four core
regressions cover an inspected 52-record insta snapshot, an independent exhaustive
candidate/sticky oracle, 10,000 nested groups and 100,000 literal terms, and distinct
construction work failures. Node agrees on 4,818 range/capture comparisons and
thirteen runtime programs. Ten runtime regressions cover earliest/global/sticky
execution, nested and empty captures/indices, per-atom case and DotAll rules,
surrogate units, generic String consumers, original-plan copies and collection,
opted-in construction/search aborts and linear single-candidate sticky accounting.
Older gap regressions now use unsupported quantifiers and retain their host-error
expectations; identity-escape regressions also check successful fixed sequences.
The corpus and pin are unchanged. Choices, internal assertions, quantifiers, backreferences,
named/scoped groups and Unicode-mode sequences remain pending.
Both compilers pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventory and
the pinned Unicode/numeric generator checks pass. Existing snapshots are unchanged.

Fixed-sequence compilation now shares identical character-set plans within the
Pattern. Two further core regressions cover repeated sets under opted-in work,
distinct sets exhausting that allowance, independent capture positions/inversion,
and compiling, cloning and matching 100,000 repeated atoms with unlimited defaults.
A runtime regression executes 1,002 repeated atoms with capture indices after
collection under an opted-in allowance. All 4,818 independent Node range/capture
comparisons remain identical. In one local unoptimized 20,000-identical-atom probe,
peak process memory fell from 163880 KiB to 8448 KiB; both versions matched the
same input. Existing snapshots and the corpus are unchanged.
Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventory and
all pinned generator checks also pass.

Outer input and multiline anchors now wrap fixed ordinary class/escape/dot
sequences while preserving captures and independent ignore-case/DotAll settings.
Three further core regressions cover an inspected 45-record insta snapshot,
independent candidate/boundary/sticky enumeration and distinct construction host
failures. Node agrees on 6,138 range/capture comparisons and thirteen runtime
programs. Seven runtime regressions cover complete-input boundaries, all four
LineTerminators and CRLF positions, sticky resets, captures/empty groups/indices,
surrogate units, trailing escape parity, shared String consumers and opted-in
construction/search aborts. Existing literal anchor and sequence snapshots retain
their values; literal scans keep their prefix-failure matcher. Internal assertions,
choices, quantifiers and Unicode modes remain pending. The corpus, pin and default
quota policy are unchanged.
Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventory and
all pinned generator checks also pass.

Top-level ordinary alternatives now combine fixed class/escape/dot sequences and
outer input/multiline anchors with literal branches. Source order still breaks
earliest-position ties, independently of match length. The delimiter scan keeps
escaped units and class-contained bars/parentheses within their branch. Global
capture slots retain own undefined match/indices elements for unselected groups,
distinct from empty participating groups. Three core regressions cover an
inspected 51-record insta snapshot, an independent exhaustive position/branch/
capture oracle and separate opted-in construction/search accounting. The earlier
literal disjunction snapshot changes only four newly supported class/dot rows.
Node agrees on all 7,200 range/capture-slot comparisons and thirteen runtime
programs. Nine runtime regressions cover ordering, captures and intrinsic own
slots, global/sticky state, per-branch boundaries, DotAll and pinned case rules,
surrogates and class delimiters, generic String consumers, copies and collection,
host aborts and whole-plan unsupported boundaries. Six earlier gap regressions
now use unsupported quantifiers. Internal/nested choices, quantifiers,
backreferences, named/scoped groups and Unicode-mode matching remain pending.
No additional reviewed whole Test262 programs become eligible; the corpus and
pin are unchanged.
Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventory and
all pinned generator checks also pass.

Single ordinary literal/class/escape/dot atoms now execute greedy/lazy *, +, ?,
exact, lower-bounded and bounded-range quantifiers. Compact decimal bounds retain
mathematical meaning beyond platform capacity without expanding the Pattern.
Three core regressions cover an inspected 105-record insta snapshot, an independent
exhaustive candidate/repetition-order oracle, 100,000-digit bounds and 300,000-unit
runs, shared clones and distinct optional construction failures. Node agrees on
all 16,362 range comparisons and fifteen runtime programs. Eight runtime
regressions cover earliest/greedy/lazy/empty runs, bounded and empty sets,
global/sticky/strict lastIndex, DotAll and pinned case rules, escapes/surrogates,
intrinsic own results, generic consumers and empty advancement, huge bounds,
100,000-unit copies after collection and optional constructor/sticky-search host
aborts. Sticky work accounting covers variable-length consuming runs even when
the Pattern text is short. Four older gap regressions now use quantified
concatenations. Quantified groups, continuations, assertions, choices,
backreferences and Unicode-mode quantifiers remain pending. The corpus and pin
are unchanged; three further whole reviewed exec programs are eligible for a
separate unchanged-fixture commit recorded in the inventory above.
Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventory and
all pinned generator checks also pass. Earlier matcher snapshots retain their
values.

Transparent noncapturing groups now wrap a single consuming atom or its one
quantified run at arbitrary depth. Two further core regressions cover an
inspected 48-record insta snapshot, 100,000 nested wrappers with different
quantifier placements, shared clones and complete-subset rejection before set
construction. The prior quantifier snapshot changes only its three newly
supported noncapturing rows. Node agrees on all 23,028 combined range comparisons
and six runtime programs. Five runtime regressions cover inner/outer greedy/lazy
placement, bounded/empty sets, flags and UTF-16 units, absent capture slots,
original source and global/sticky state, generic consumers and empty advancement,
copies after collection and constructor validation of 100,000 nested groups with
unlimited defaults. Three older gap regressions retain unsupported multi-atom
groups, including direct eval's ordered host-abort effects. Capturing/multi-atom
groups, multiple quantifiers, continuations, assertions, choices and Unicode
modes remain pending. The corpus and pin are unchanged.
Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventory and
all pinned generator checks also pass.

Quantified consuming prefixes now execute with complete capture-free literal
continuations, including overlapping suffixes and empty noncapturing groups.
Three core regressions cover an inspected 87-record insta snapshot, an independent
exhaustive candidate/repetition-order oracle, compact oversized bounds and
300,000-unit repeated-prefix success/failure cases. Node agrees on all 21,120
range comparisons and fifteen runtime programs. Nine runtime regressions cover
earliest-before-length and greedy/lazy ordering, bounded/empty sets and failed
prefixes, global/sticky/strict lastIndex, pinned case rules and DotAll, escapes and
surrogates, own whole-match indices, generic consumers and callbacks, long copies
after collection, optional constructor/sticky-search host aborts and complete
unsupported continuations. Four older gap regressions now use character-set
continuations. The two forward scans preserve linear search without allocation,
expanded repetitions or a recursive/backtracking stack. Captures, set
continuations, multiple quantifiers, enclosing concatenation groups, assertions,
choices and Unicode modes remain pending. Existing snapshots, the corpus and pin
are unchanged. One further reviewed whole exec poem program is vendored in the
separate unchanged-fixture inventory above.
Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventory and
all pinned generator checks also pass.

Top-level alternatives now also accept quantified atoms and quantified prefixes
with capture-free literal continuations, alongside the existing fixed and
anchored branches. Three core regressions cover an inspected 60-record insta
snapshot, exhaustive independent branch/capture selection and search accounting.
Five newly supported rows change in two earlier snapshots. Node agrees on all
11,730 range/global-capture comparisons and twelve runtime programs. Eight runtime
regressions cover earliest starts and source-order ties, independent greedy/lazy
branch order, own undefined global capture/indices slots, global/sticky state and
strict writes, pinned flags and UTF-16 units, generic consumers and empty
advancement, copies after collection and optional constructor/sticky-search host
aborts. Existing gap regressions now retain unsupported character-set
continuations and quantified captures. A compiled constant flag preserves full
remaining-input charges for sticky quantified branches; default quotas remain
unlimited. Plans remain immutable, and searches allocate nothing or expand
repetitions. Nested choices, captured quantified groups, anchored quantified
branches and Unicode modes remain pending. The corpus and pin are unchanged.
Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventories
and all pinned generator checks also pass.

Outer input/multiline anchors now surround quantified atoms and quantified
prefixes with capture-free literal continuations. Three core regressions cover
an inspected 114-record insta snapshot, exhaustive independent candidate and
repetition ordering, compact oversized bounds and 300,000-unit success/failure
cases. Five newly supported rows change in two earlier anchor snapshots; every
other snapshot is unchanged. Node agrees on all 30,240 range comparisons and
eighteen runtime programs. Nine runtime regressions cover assertion-constrained
greedy/lazy lengths, exact ends and multiline/CRLF boundaries, sticky/global
state, literal continuations and escape parity, pinned case/DotAll and surrogate
units, transparent groups, capture slots in mixed alternatives, intrinsic Arrays
and strict writes, generic consumers and empty advancement, long copies after
collection, optional constructor/sticky-search aborts and ordered unsupported
host effects. Prior gap regressions now retain character-set continuations. The
three monotone scans remain linear without allocation, expanded counts or native
recursion. Optional sticky work covers the entire remaining input, including
anchored branches in choices; defaults remain unlimited. Captured quantified
groups, nested choices, internal assertions, multiple quantifiers and Unicode
modes remain pending. The corpus and pin are unchanged.
Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventories
and all pinned generator checks also pass.

Ordinary capturing wrappers now surround a quantified atom or repeated prefix.
Three core regressions cover an inspected 99-record insta snapshot, independent
nested capture/repetition ordering, 100,000 nested groups with compact layouts,
continuation endpoints and global alternative slots. Fifteen newly supported
rows change in four earlier snapshots. Node agrees on all 43,710 range/capture
comparisons, all 1,782 stored match/capture arrays and fifteen runtime programs.
Nine runtime regressions cover whole-run versus final-iteration captures,
zero-iteration undefined versus empty groups, greedy/lazy continuation ranges,
anchors and global/sticky state, alternative offsets, generic consumers and
callbacks, intrinsic capture/indices Arrays and strict writes, pinned flags and
surrogates, copies after collection, constructor validation of 100,000 groups and
1,000-capture results with unlimited defaults, optional host aborts and complete
unsupported bodies. Prior capture gap regressions retain unsupported multi-atom
groups. Capturing layout uses two counts and the source's implicit opening stack;
search still allocates nothing, expands no repetition counts and uses no native
recursion. Captured suffixes, multi-atom/named groups, nested choices, multiple
quantifiers and Unicode modes remain pending. The corpus and pin are unchanged.
The further reviewed whole exec program is vendored in the separate unchanged-
fixture inventory above.
Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventories
and all pinned generator checks also pass.

Fixed literal continuations now accept ordinary nested and empty capturing
groups after quantified prefixes. Three core regressions cover an inspected
87-record insta snapshot, independent candidate/repetition/capture ordering,
100,000 nested suffix groups with shared clones and anchored/alternative slots.
Three newly supported rows change in the prior continuation snapshot; every
other snapshot retains its values. Node agrees on all 43,710 range/capture
comparisons, all 1,656 stored match/capture arrays and fifteen runtime programs.
Eight runtime regressions cover prefix versus suffix ranges, greedy/lazy order,
empty participating suffix groups versus undefined prefix captures, anchors and
global/sticky state, pinned case/DotAll and surrogate units, generic consumers,
callbacks and matchAll, intrinsic Arrays and strict writes, copies after
collection, constructor validation of 100,000 suffix groups and 1,000-capture
results with unlimited defaults, optional host aborts and complete unsupported
suffixes. Prior capture gap regressions retain character-set suffixes. Fixed
suffix ranges resolve from the chosen prefix endpoint without allocation or a
capture matrix during search; the same two linear scans retain their existing
work accounting. Enclosing concatenation groups, character-set continuations,
nested choices, named groups, multiple quantifiers and Unicode modes remain
pending. The diagnostic fixture review enables no further whole programs. The
corpus, inventories and pin are unchanged.
Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventories
and all pinned generator checks also pass.

Complete ordinary outer noncapturing groups now compile around supported
Pattern bodies. Three core regressions cover an inspected 40-record insta
snapshot, 2,940 independently paired group-boundary comparisons and 100,000
nested wrappers without native recursion. Eight runtime regressions cover
quantified continuations and choices, assertion-constrained repetition, source-
order capture/indices slots and unselected alternatives, global/sticky state and
strict writes, original source/flags and constructor conversion order, escapes,
classes and surrogate units, pinned case/DotAll, generic consumers and callbacks,
copies after collection, unlimited nesting and optional host aborts. Two prior
gap regressions retain unsupported embedded choices and capturing concatenation
groups. Node agrees on all 70,680 match/capture range comparisons and eighteen
runtime programs. Group unwrapping uses constant space and two linear scans;
search reuses the existing immutable plans without changing work accounting or
allocating capture matrices. Capturing outer concatenation/choice groups, named
groups, nested choices within concatenations, multiple quantifiers and Unicode
modes remain pending. The diagnostic fixture review enables no further whole
programs. The corpus, inventories and pin are unchanged.
Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventories
and all pinned generator checks also pass.

Complete ordinary enclosing captures now compile around every supported
Pattern body, including quantified continuations, anchored bodies and top-level
choices. Three core regressions cover an inspected 42-record insta snapshot,
4,320 independent mixed-group boundary/count comparisons and 100,000 mixed
wrappers with 50,000 whole-match captures. The prior noncapturing snapshot keeps
all values. Eight runtime regressions cover whole-match prefix slots versus
final iterations and fixed suffixes, source-order alternative slots, own
undefined elements and intrinsic Arrays, zero runs and empty groups, assertion-
constrained repetition, global/sticky state and strict writes, original slots,
pinned flags and UTF-16 ranges, generic consumers and callbacks, constructor
validation of 100,000 enclosing captures, 1,000-capture results, copies after
collection with unlimited defaults and optional host aborts. Earlier rejection
regressions retain unsupported embedded choices/assertions and character-set
continuations. All 130 constant RegExp gap-loop programs retain Unsupported.
Node agrees on all 106,020 range/capture comparisons and twenty runtime
programs. Matcher storage adds only scalar prefix/total counts to the existing
body plan, without native recursion, a wrapper tree or search allocations.
Backreferences remain unsupported after normalization. The fixture review
enables no further whole programs. The corpus, inventories and pin are unchanged.
Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventories
and all pinned generator checks also pass.

Complete ordinary capturing/noncapturing groups now enclose supported top-level
alternative bodies. Three core regressions cover an inspected 93-record insta
snapshot, independent position/source/repetition/capture ordering, 100,000
branch wrappers and shared clones, preserved fixed capture APIs and optional
host charge separation. All prior snapshots keep their values. Eight runtime
regressions cover earliest starts and source-order ties, greedy/lazy branch
selection, input/multiline assertions, whole-branch versus inner capture slots,
global/sticky state and strict writes, participating empty groups and unselected
slots, pinned case/DotAll and UTF-16 ranges, generic consumers and callbacks,
intrinsic Arrays, original sources and copies after collection, 100,000-group
constructor validation and 1,000-capture results without default quotas, optional
host aborts and complete unsupported bodies. All 137 reviewed constant RegExp
gap-loop programs retain Unsupported. Node agrees on all 36,270 range/capture
comparisons, all 1,800 stored snapshot match/capture arrays and seventeen runtime
programs. Scalar branch prefix/total counts retain immutable plan sharing and
existing search bounds without recursion, a wrapper tree or search allocation.
Nested choices, quantified multi-atom groups, embedded continuations/assertions,
named groups, backreferences and Unicode modes remain pending. The fixture
review enables no further whole programs. The corpus, inventories and pin are
unchanged.
Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventories
and all pinned generator checks also pass.

Complete ordinary capturing/noncapturing consuming groups now compile inside
outer input/multiline anchors. Three core regressions cover an inspected
87-record insta snapshot, independent boundary/repetition/capture ordering over
all four LineTerminators, 100,000 enclosing groups with shared clones, long
successful/failed input runs, unchanged fixed capture APIs and optional host
charge separation. Three newly supported noncapturing rows change in the prior
quantified-anchor snapshot; other snapshots retain their values. Eight runtime
regressions cover whole and final-iteration ranges, lazy repetition constrained
by assertions, exact input ends and multiline boundaries, participating empty
groups and undefined captures, global/sticky state and strict writes, composition
with branch and outer capture prefixes, original slots, pinned case/DotAll and
UTF-16 ranges, generic consumers and intrinsic Arrays, deep copies after
collection, unlimited constructor/result sizes and optional host aborts. One
older gap regression retains an unsupported character-set continuation. All
145 reviewed constant RegExp gap-loop programs retain Unsupported. Node agrees
on all 34,410 range/capture comparisons, all 1,716 stored snapshot match/capture
arrays and sixteen runtime programs. Scalar counts retain the existing
nonrecursive body plan and search bounds without a wrapper tree or capture
matrix. Choices inside anchored bodies, quantified multi-atom groups, embedded
assertions/continuations, named groups, backreferences and Unicode modes remain
pending. The fixture review enables no further whole programs. The corpus,
inventories and pin are unchanged.
Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventories
and all pinned generator checks also pass.

Capture-free literal prefixes now precede one quantified atom and optional
fixed literal continuation. Three core regressions cover an inspected 132-record
insta snapshot, independent prefix/candidate/repetition/capture ordering across
overlapping literals and exact bounds, 100,000-unit prefixes, long successful
and failed overlaps, shared clones, decoded escape lengths, raw surrogate units
and optional host construction aborts. Existing snapshots keep their values.
The literal search cursor retains prefix-failure state between occurrences;
every cursor remains monotone and search allocates no candidate list or capture
matrix. Eight runtime regressions cover earliest starts before greedy/lazy
lengths, body/suffix captures excluding the prefix, zero-run undefined versus
empty groups, global/sticky state and strict writes, anchors and branch/outer
capture composition, pinned flags, decoded escapes and ordinary-mode surrogate
quantification, generic consumers and intrinsic Arrays, long copies after
collection with unlimited defaults and optional work aborts. Earlier gap
regressions retain multiple quantifiers. Node
agrees on all 68,328 range/capture comparisons, all 3,264 stored snapshot
match/capture arrays and twenty runtime programs. Sticky accounting covers the
complete remaining suffix; prefix search keeps its linear bound. The whole
fixture review enables two unchanged greedy/lazy bounded-prefix programs,
recorded in the separate fixture inventory above. The pin is unchanged.
Multiple consuming quantifiers, nested choices, named
groups, backreferences and Unicode modes remain pending.

Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventories
and all pinned generator checks also pass.

Literal prefixes now include ordinary capturing and noncapturing groups,
including nested and empty groups. Three core regressions cover an inspected
90-record insta snapshot, an independent prefix/candidate/repetition/capture
oracle with participating empty groups, 100,000 flat prefix captures, shared
clones, invalid ranges and opt-in preparation aborts. Six existing prefix
snapshot records gain matching support; every earlier accepted value is retained.
Eight runtime regressions cover prefix/body/suffix source order and indices,
undefined versus participating empty groups, global/sticky and empty-match
progress, strict writes, branch/outer capture and anchor composition, decoded
escapes and UTF-16 ranges, pinned flags, generic consumers, intrinsic Arrays,
100,000-group compilation and 1,000-group results after collection with unlimited
defaults. Optional host failures remain outside JavaScript handlers. Node agrees
on all 110,376 match/capture comparisons, all 5,547 stored arrays across both
prefix snapshots, and twenty runtime programs. The existing gap regressions
retain nonliteral prefixes and multiple quantifiers. The whole fixture review
enables no further programs; the corpus, inventories and pin are unchanged.
Grouped multi-atom repetition, nonliteral
continuations, nested choices, named groups, backreferences and Unicode modes
remain pending.

Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventories
and all pinned generator checks also pass.

Fixed prefixes now include ordinary classes, class escapes and dots, with
static captures across nested groups. Three core regressions cover an inspected
93-record insta snapshot, an independent position/repetition/capture oracle with
empty and inverted sets, overlapping candidates, 1,000 shared set atoms, clones
and optional construction/search bounds. Nine prior prefix snapshot records
gain support; every earlier accepted record retains its match/capture values.
The shared fixed-sequence occurrence cursor preserves the existing sequence
oracle and snapshots. Eight runtime regressions cover earliest starts before
greedy/lazy lengths, prefix/body/suffix and empty capture slots, global/sticky
state and strict writes, branch/outer capture and anchor composition, classes
and dots with pinned flags, surrogate offsets, generic consumers, intrinsic
Arrays, shared sets after collection and unlimited defaults. Optional budgets
account for every nonsticky prefix atom; sticky runs check one prefix and keep
full-body accounting. Node agrees on 151,110 range/capture comparisons, all
8,748 stored arrays across the three prefix snapshots, and nineteen runtime
programs. Earlier gap tests retain multiple quantifiers. The whole fixture review enables no additional programs; the
corpus, inventories and pin are unchanged. Repeated multi-atom groups, multiple
quantifiers, nested choices, named groups,
backreferences and Unicode modes remain pending.

Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventories
and all pinned generator checks also pass.

Quantified continuations now include fixed ordinary classes, class escapes
and dots, with nested and empty captures. Three core regressions cover an
inspected 99-record insta snapshot, an independent endpoint/repetition/capture
oracle with empty and inverted sets, bounded greedy/lazy counts, 1,000 shared
sets and captures, clones and optional preparation/search bounds. Twenty-seven
records across nine earlier snapshots gain matching support; every earlier
accepted match/capture value is unchanged. Eight runtime regressions cover
earliest candidates, greedy/lazy endpoints, whole-run/final-iteration and fixed
capture slots, undefined versus empty participation, global/sticky state and
strict writes, prefix/outer capture, anchor and branch composition, pinned flags,
UTF-16 offsets, generic consumers, intrinsic Arrays and 1,000-capture results
after collection with unlimited defaults. Sticky work covers every continuation
atom over all candidate endpoints. Node agrees on 391,572 independent
match/capture comparisons, all 23,375 stored arrays across affected snapshots,
and nineteen runtime programs. Earlier gap tests retain their coercion and host
ordering assertions with unsupported multi-quantifier Patterns. The whole
fixture review enables no further programs; the corpus, inventories and pin are
unchanged. Repeated multi-atom groups, multiple quantifiers, embedded choices,
named groups, backreferences and Unicode modes remain pending.

Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventories
and all pinned generator checks also pass.

Outer ordinary word assertions now compile with supported fixed and quantified
bodies. Three core regressions cover an inspected 114-record insta snapshot,
an independent exhaustive start/endpoint/capture oracle, 100,000 duplicate
assertions on each side, cloned plans, long inputs and surrogate positions.
Eleven records across three older snapshots gain matching support; every earlier
accepted value is unchanged. Eight runtime regressions cover ASCII WordCharacters
with ordinary IgnoreCase, empty and contradictory assertions, greedy/lazy
selection, participating/undefined captures, source-order alternatives, enclosing
groups, multiline and sticky boundaries, UTF-16 offsets, zero-width lastIndex,
strict writes, generic consumers, intrinsic Arrays, collection and optional host
aborts with unlimited defaults. Node agrees on all 14,944 stored results across
four affected snapshots, 140,160 independent match/capture comparisons and
21 runtime programs. Four earlier gap programs now exercise assertions embedded
within consuming sequences. The whole-fixture review enables no further
Test262 programs; inventories and the pin are unchanged. Embedded assertions,
assertions inside consuming groups within outer assertion bodies, and
Unicode-mode matching remain pending.

Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventories
and all pinned generator checks also pass.

Word assertions now also compile within fixed consuming sequences and groups,
and in fixed prefixes/continuations of one quantified atom. Three core regressions
cover an inspected 99-record insta snapshot, independent neighbor/capture-offset
checks, 100,000 collapsed duplicate assertions, 1,000 shared sets and captures,
clones and conservative optional work. All earlier stored snapshots are unchanged.
Eight runtime regressions cover empty assertion captures, overlapping and
assertion-only prefixes, greedy/lazy continuation endpoints, anchors and complete
groups, source-order alternatives, sticky state, pinned flags and surrogate units,
generic consumers, intrinsic results, collection and optional work with unlimited
defaults. Node agrees on 21,424 stored results across five snapshots, 196,662
independent match/capture comparisons and twenty runtime programs. Existing gap
programs now exercise embedded input/line assertions or repeated multi-atom groups.
The whole-fixture review enables no further programs; inventories and the pin are
unchanged. Input/line assertions embedded within consuming bodies, assertions
within a repeated atom, multiple quantifiers, named groups, backreferences and
Unicode modes remain pending.

Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventories
and all pinned generator checks also pass.

Input/line assertions now compile within fixed consuming bodies and ordinary
groups, with explicit multiline flags. Three core regressions cover an inspected
175-record insta snapshot, an independent exhaustive position/capture oracle,
100,000 collapsed input assertions, cloned plans and exact sticky positions.
Three records in the earlier complete-group snapshot gain support; all earlier
accepted values are unchanged. Eight runtime regressions cover interior empty
capture slots, all four LineTerminators and CRLF positions, independent flags,
escape/class boundaries, sticky state and strict writes, zero-width groups,
outer anchors and alternatives, generic consumers, intrinsic results, collection
and long searches with unlimited defaults and optional host work. Node agrees
on all 34,896 stored results across seven snapshots, 278,568 independent
match/capture comparisons and 21 runtime programs. Ten earlier gap programs now
exercise input/line assertions in variable bodies. The whole-fixture review
enables no further programs; inventories and the pin are unchanged. Input/line
assertions within variable consuming bodies or repeated atoms, multiple
quantifiers, named groups, backreferences and Unicode modes remain pending.

Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventories
and all pinned generator checks also pass.

Input/line assertions now compose with fixed prefixes/continuations of one
quantified atom. Three core regressions cover an inspected 145-record insta
snapshot, an independent exhaustive repetition/endpoint/capture oracle,
100,000 collapsed prefix assertions, cloned plans, large runs and explicit
multiline behavior. Every earlier stored snapshot is unchanged. Eight runtime
regressions cover greedy/lazy and bounded endpoints, assertion-only prefix
candidates, exact empty/undefined slots, consuming continuations and CRLF,
outer and enclosing groups, source-order alternatives, global/sticky state,
pinned flags and surrogate offsets, generic consumers, intrinsic results,
collection, long runs with unlimited defaults and optional host aborts. Node
agrees on 43,896 stored results across eight snapshots, 349,524 independent
match/capture comparisons and 21 runtime programs. Twelve earlier gap programs
now exercise multiple quantifiers or assertions inside repeated atoms. The
whole-fixture review enables no further programs; inventories and the pin are
unchanged. Assertions inside a repeated atom, repeated multi-atom groups,
multiple quantifiers, named groups, backreferences and Unicode modes remain
pending.
Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventories
and all pinned generator checks also pass.

Standalone repeated literal groups have three core regressions: an inspected
116-record insta snapshot, an independent exhaustive candidate/repetition/capture
oracle, and exact border comparisons for compact virtual-word failure tables,
large bounds, 100,000 nested captures, cloned plans and optional host work.
Every earlier stored snapshot is unchanged. Eight runtime regressions cover
earliest greedy/lazy and bounded runs, overlapping literal occurrences,
last-iteration and empty/undefined slots, complete enclosing/branch captures,
source-order ties, global/sticky state, decoded escapes, pinned case behavior,
surrogate offsets, generic consumers and callbacks, intrinsic result arrays,
collection, large minimums with unlimited defaults and optional host aborts.
Node agrees on 46,392 stored match/capture arrays across nine snapshots, 93,480
additional match/capture comparisons and 23 runtime programs. Eleven earlier
gap programs now cover repeated choices or sets instead. Assertions and fixed
terms around repeated literal groups, repeated sets/choices/empty groups,
multiple quantifiers, named groups, backreferences and Unicode modes remain
pending. The whole-fixture review enables no additional programs; inventories
and the existing Test262 pin are unchanged.
Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventories
and all pinned generator checks also pass.

Standalone repeated fixed class/dot groups have three core regressions: an
inspected 150-record insta snapshot, an independent exhaustive phase-run and
capture oracle, and large minimums, shared sets, 100,000 nested captures,
100,000-unit sticky groups, cloned plans and optional construction work. Earlier
stored snapshots remain unchanged. Eight runtime regressions cover earliest
greedy/lazy and bounded iterations, overlapping candidates, exact nested/empty
and undefined slots, complete enclosing/branch captures, source-order ties,
global/sticky state, class escapes, DotAll, pinned case behavior, surrogate
offsets, generic consumers and callbacks, intrinsic results, collection, large
minimums with unlimited defaults and optional host aborts. Node agrees on 49,200
stored results across ten snapshots, 118,560 additional match/capture comparisons
and 22 runtime programs. Eight earlier gap programs now exercise choices inside
repeated groups. Whole-fixture review enables no new programs; inventories and
the Test262 pin are unchanged. Assertions inside or around repeated groups,
surrounding fixed terms, repeated choices/empty groups, additional quantifiers,
named groups, backreferences and Unicode modes remain pending.
Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks. Formatting, Clippy, policy, fixture inventories
and all pinned generator checks also pass.

Fresh RegExp result/indices initialization has three object regressions for
descriptors, key order, length/truncation and traced edges; rejection of invalid
Array state and foreign/stale edges; and optional capacity/work failure before
mutation. Four runtime regressions cover intrinsic prototypes and setter
bypass, all property attributes and key order, sparse mutations and own undefined
slots, distinct indices pairs, 100,000 captures with unlimited defaults and
post-result collection, and optional property aborts after lastIndex writes.
The unchanged eight-group repeated-assertion prototype suite drops from 156.29
seconds to 0.76 seconds after this fix; no case or coverage is removed. A separate
100,000-capture result/indices regression, including collection, completes with
the other three runtime regressions in 1.34 seconds. Matching grammar, corpus
inventories and the pinned Test262 revision are unchanged.
Stable and MSRV pass the complete 13494-variant corpus, workspace/documentation
tests and debug-stack checks, including the fresh-array object regressions.
Formatting, Clippy, policy, fixture inventories and all pinned generator checks
also pass.

Standalone ParsePattern validation now accepts arbitrary UTF-16 constructor
Strings and returns exact capture counts through the parser's public API. Three
regressions cover constructor/literal boundary differences, ordered flag errors,
100,000 nested capturing groups and 100,000 alternatives without default quotas.
An inspected 89-record diagnostic snapshot covers all three Pattern modes, raw
line terminators, surrogate units, capture counts and syntax failures. This shares
the existing grammar implementation; the corpus inventory is unchanged.

Core EscapeRegExpPattern source serialization handles empty Patterns, solidus,
all four raw line terminators and existing escape parity without allocating.
Two inspected snapshots cover 46 UTF-16 records, including ordinary Pattern
syntax, UnicodeSets strings and lone/paired surrogates. A parser regression
revalidates constructor inputs and their serialized forms, checks capture counts
and confirms the output passes literal boundary scanning. Native source getter
exposure and matching remain pending; no fixture or matching credit is added.

Native RegExp construction now validates constructor Patterns, preserves original
source/flags, sets the specified lastIndex descriptor and supports subclasses.
Thirteen focused runtime regressions cover call identity, regexp-like getter order,
String hints, newTarget prototype lookup, private-slot copying after public
overrides, syntax errors, source serialization, own brands, custom exec and the
explicit native-matching boundary, argument evaluation and native stack guards.
Two storage regressions verify foreign/stale handle rejection and slot values.
Slot storage is not inherited and survives
collection. A 120,000-unit source exercises unlimited defaults; optional output
quotas still abort outside language handlers. Generic splitting with a zero limit
and lazy matchAll now proceed after default intrinsic construction. Exec validates
the native brand before input conversion; unbranded receivers throw TypeError.
Broader Pattern matching remains pending. The
full unchanged corpus is checked on both compilers; fixture counts and the pin
are unchanged by this implementation step.

A separate scoped review reads 19 constructor programs, all 12 source getter
programs and seven prototype not-a-constructor programs. Twenty-eight unchanged
originals add 56 passing normal/strict variants on both compilers for constructor
identity/copying, ordered regexp-like properties, errors, flag validation,
lastIndex, source metadata/brands and prototype non-construction. Ten whole files
remain excluded for literal execution, native matching, cross-realm support or
global own-key reflection. The root RegExp directory is not claimed as completely
reviewed. Older historical descriptions retain edition-17-compatible assertions.
The pin, ten helpers, original bytes and unlimited defaults are unchanged; no
native matching coverage is claimed.

Core now streams RegExp.escape's edition-17 encoding without allocating. Two
inspected snapshots cover every ASCII code point in leading and later positions,
all non-ASCII ECMAScript WhiteSpace/LineTerminator values, ordinary Unicode,
surrogate endpoints, paired and unpaired adjacency, and the empty string. An
independent implementation of the published algorithm verifies all 271 snapshot
records. Runtime exposure, RegExp objects and matching remain pending; this
component addition changes no Test262 inventory or execution count.

Runtime now exposes RegExp.escape with a String-only input check, ignored this
and extra values, the shared encoding and exact opt-in output/work quotas. Three
regressions cover poisoned conversion hooks, wrappers and primitive types,
surrogate adjacency, evaluated extra arguments, empty and large default output,
exact quota boundaries and host aborts that bypass catch/finally. The RegExp
constructor and ordinary, unbranded prototype have the complete edition-17
constructor/prototype property metadata, including all ten getters and five
symbol methods. The generic species getter returns its receiver. Reflection and
collection checks cover names, lengths, descriptors, non-constructible methods,
the fixed prototype property and intrinsic rooting after global deletion. These
22 intrinsic records bring the initialized-realm entry count to 385. Construction,
matching, other prototype bodies and the string iterator remain pending and
receive no pass credit; no regex dependency or default quota is activated.

A separate escape review reads all twenty unchanged whole programs at the same
Test262 pin. Nineteen originals add 38 passing normal/strict variants on both
compilers for leading letters/digits, syntax and other punctuators, controls,
white space/line terminators, surrogates, Unicode preservation, String-only
arguments and intrinsic metadata. The cross-realm program needs the pending
createRealm hook and remains excluded with no pass credit. The ten harness
helpers and unlimited defaults are unchanged; construction and matching receive
no coverage credit from this cohort.

Generic RegExp flags and toString now preserve the specified property/coercion
order. Flags applies ToBoolean without invoking conversion hooks and can return
both u and v for an ordinary object. toString rejects primitives before lookup,
uses String hints, preserves borrowed source units, and checks its exact combined
output only after both conversions. Flag/source getters recognize the intrinsic
ordinary prototype by identity and reject other unbranded objects without reading
public lookalike properties. Generic test converts its input before exec lookup,
preserves receiver and argument count, and validates the Object/null result.
Four regressions cover these paths, abrupt getters/conversions, live mutation,
prototype exceptions, quotas and recursive getters/exec callbacks on the native
stack guards. Native instances, constructors, matching, remaining symbols and the string
iterator remain pending; the inventory, dependency set and unlimited defaults
are unchanged.

A separate generic-operation review reads all sixteen flags programs and all
nine toString programs. Twenty-two unchanged originals add 44 passing
normal/strict variants on both compilers for ordered reads, truthiness, abrupt
completions, primitive receivers, the ordinary prototype and intrinsic metadata.
Two flags programs and one toString program require native RegExp instances and
remain excluded with no pass credit. The existing pin, ten helpers and unlimited
defaults are unchanged; no native matching coverage is claimed.

Generic RegExp Symbol.match and Symbol.search now share the validated custom-exec
operation with test. Non-global match returns the exact exec result; global match
uses live exec lookup, intrinsic arrays and own data elements, with exact result
counts and ToLength-based empty-match advancement for both u and v. Search
preserves signed zeros, NaNs, symbols and object identity in lastIndex, restores
only after normal exec completion and returns result.index without coercion.
Seven native regressions cover ordered coercion, live exec mutation, poisoned
inherited setters, surrogate pairs/unpaired units, end and safe-integer index
boundaries, strict write failures, abrupt completion effects, String hook
delegation, opted-in work aborts and recursive callbacks on the native stack
guards. No native matcher, fixture count, dependency or default quota is added.

A separate match/search review reads all 76 whole programs at the existing pin.
Seven unchanged match and fourteen unchanged search originals add 42 passing
normal/strict variants on both compilers for generic operation ordering,
custom-exec validation, lastIndex preservation, abrupt completions and metadata.
The other 55 whole programs require native instances or matching, including two
not-a-constructor programs that first construct a RegExp; they remain excluded
with no source rewriting or pass credit. The pin, ten helpers and unlimited
defaults are unchanged; this cohort claims no native matcher coverage.

Core GetSubstitution template scanning now preserves borrowed UTF-16 units and
recognizes literal dollar text, match/prefix/suffix references, exact one/two-digit
capture fallback and named-capture keys. A cached closing delimiter prevents
quadratic scans of unterminated names. String replace/replaceAll share this
scanner without changing their no-capture semantics. Two inspected insta
snapshots cover 70 numeric and named-reference records, including zero/out-of-range
indices, literal dollars, nested marker text, empty/missing delimiters, astral
keys and unpaired surrogates; an independent reference calculation verifies every
record. A third regression covers cloned iterators and 120,000 unterminated
markers with unlimited defaults. Capture-value resolution now feeds the shared
scanner through generic RegExp replacement; native matching remains pending and
inventories are unchanged.

Generic RegExp Symbol.replace collects live custom-exec result objects before
replacement callbacks, then follows ordered length/match/index/capture/groups
reads and conversions. It clamps positions, preserves undefined captures, passes
uncoerced groups to callbacks, boxes groups for text templates and resolves named
references in order without rescanning their expansion text. Backward/overlapping
results still perform every read, callback and conversion before their output is
ignored. Ten native regressions cover these paths, exact callback arguments and
receivers, empty results, oversized custom matches, Unicode empty advancement,
poisoned public libraries, String hooks, 120,000-unit unlimited output, opted-in
output/argument/work aborts and recursive exec/capture/named/replacer callbacks.
Native instances, construction, matching, matchAll/split and the string iterator
remain pending; the 385 intrinsic entries and corpus counts are unchanged.

A separate replacement review reads all seventy whole programs at the existing
pin. Six unchanged originals add twelve passing normal/strict variants on both
compilers for receiver checks, flags/exec getter failures, global lastIndex
initialization and metadata. The other sixty-four programs need native instances
or matching, including not-a-constructor.js, which constructs a RegExp first;
they remain excluded without rewriting or pass credit. Original bytes, the pin,
ten helpers and unlimited defaults are unchanged; no native matching coverage is
claimed.

Generic RegExp Symbol.split follows ordered species/flags construction and
ToUint32 limits, creates intrinsic Arrays with own data elements and copies
capture values unchanged. Empty input executes once without lastIndex writes;
nonempty input uses strict lastIndex updates, live exec lookup, clamped ToLength
ends and u/v code-point advancement. Eleven native regressions cover conversion
and constructor ordering, intrinsic default selection, exact sticky flags,
primitive receiver/species errors, early limit returns, capture identity and live
mutation, empty/failing/backward custom matches, poisoned public libraries,
String delegation, 120,001-unit unlimited flag output, opted-in output/work aborts
and recursive species/flags/exec/index/length/capture hooks on native stack guards.
Native constructors, matching, matchAll and the string iterator remain pending;
the 385 intrinsic entries, dependencies and corpus counts are unchanged.

A separate splitting review reads all forty-four whole programs at the existing
pin. Twenty-one unchanged originals add 42 passing normal/strict variants on both
compilers for custom species, flags conversion, zero limits, empty input,
lastIndex and length coercion, abrupt captures, receiver checks and metadata.
The other twenty-three whole programs require native instances or matching,
including species-ctor-ctor-non-obj.js's initial baseline construction and
not-a-constructor.js's RegExp creation; one also needs cross-realm support. They
remain excluded without rewriting or pass credit. Original bytes, the pin, ten
helpers and unlimited defaults are unchanged; no native matching is claimed.

Shared immutable String storage avoids copying complete inputs for every
custom exec call. A native split regression processes 120,000 unsuccessful exec
calls and preserves its complete input with unlimited defaults. Core checks
verify content-based hashing and UTF-16 ordering across separate allocations,
unpaired-surrogate concatenation and ownership across threads. Existing snapshots,
fixture inventories, optional work accounting and default quotas are unchanged.

Generic RegExp Symbol.matchAll now performs ordered species/flags construction,
ToLength lastIndex copying and strict matcher initialization before creating a
lazy branded iterator. The RegExp String Iterator retains traced matcher/input
slots, inherits the shared Iterator prototype and returns exact custom exec
results in fresh IteratorResults. Non-global success or null permanently completes;
global empty matches advance by UTF-16 unit/code point for u/v. Abrupt operations
leave completion unchanged, and reentrant next calls preserve nested completion.
Twelve native integration tests cover these paths, exact metadata/own-slot brands,
constructor defaults, conversion ordering, live mutation, strict write errors,
String hooks and iterator helpers with poisoned public libraries, 120,000-unit
input, opted-in aborts and recursive creation/next hooks. Two storage
regressions verify tracing through completion, collection, foreign/stale handles
and non-inherited slots. The two new rooted intrinsic records bring the initialized
realm to 387 entries. Native instances, construction and matching remain pending;
corpus inventories and dependencies are unchanged.

A separate review reads all 26 whole matchAll and all 17 RegExp String Iterator
programs at the existing pin. Four unchanged matchAll originals add eight passing
normal/strict variants on both compilers for metadata and primitive receivers.
The other 39 whole programs require native construction, instances or matching,
including every iterator original and not-a-constructor.js's initial RegExp
creation; they remain excluded without rewriting or pass credit. Historical draft
descriptions do not change the selected edition-17 assertions. Original bytes,
the pin, ten helpers and unlimited defaults are unchanged; no native construction
or matching coverage is claimed.

Scanner checkpoints now retain input positions and parser-owned template context
before every token's trivia. Primary-expression RegExp rescans restore those
checkpoints, truncate the division suffix, and cache the replacement token so
subsequent scanning resumes after the literal. Failures retain an authoritative
diagnostic and EOF sentinel. Five internal scanner/parser regressions exercise
failed Div-goal lookahead, continuation after replacement, nested template tails,
line-terminator trivia, lossless UTF-16, hashbang placement and failed RegExp
rescans. An additional public regression and inspected diagnostic snapshot cover
exact property errors in arrow, object, computed class and template contexts
across Script, UTF-16, eval and dynamic Function entry points. Grammar-driven
arrow/destructuring covers, matching and RegExp objects remain pending. Corpus
inventories, runtime defaults, native guards and dependencies are unchanged.

Literal flag validation implements IsValidRegularExpressionLiteral's allowed
`d g i m s u v y` code points and duplicate rejection, followed by ParsePattern's
u/v exclusion. Unknown or duplicate flags take precedence over incompatible
Unicode modes; flag rejection precedes Pattern validation. Two parser
regressions cover every flag subset, reordered valid flags, non-ASCII and astral
flags, validation order, templates/computed keys, and UTF-16, eval, and dynamic
Function entry points. The new flag diagnostic snapshot and the reduced Pattern
gap snapshot were inspected. An eval regression verifies that flag errors become
SyntaxError before source effects, while valid-flag Pattern gaps bypass JavaScript
catch/finally as host Unsupported. Runtime defaults and native guards are unchanged.
Two unchanged literal flag originals add four parse-negative variants for uppercase
`G` and repeated `g` in `gig`, with exact literal spans and messages. Both whole
programs and metadata were reviewed. These checks precede Pattern parsing and
provide no RegExp object or matching execution credit; original bytes, assertions,
the pin, eight helpers, and unlimited defaults are preserved.

The core Pattern validator implements unnamed capturing and noncapturing groups,
alternatives, lookahead/lookbehind and boundary assertions, quantifier placement,
lazy suffixes, exact arbitrary-length decimal bounds, scoped i/m/s modifiers and
their duplicate/overlap early errors, character/control/hexadecimal/Unicode
escapes, identity escapes, and numbered backreferences including forward uses.
It follows the edition-17 core grammar without Annex B extensions. An iterative
group stack accepts 20,000 nested captures; decimal comparisons do not convert
bounds to machine integers. Unicode modes decode paired UTF-16 while retaining
unpaired surrogates; ordinary mode uses separate units. The specification's
capture-count early error is distinct from a host quota. Six parser regressions
cover valid and malformed productions, all modifier-set combinations, large
bounds and references, surrogate inputs, grammar entry points, and unsupported
features that must not receive negative credit. The new diagnostic snapshot and
updated boundary snapshot were inspected. Eval rejects malformed supported
patterns as SyntaxError before effects; matching and unimplemented grammar gaps
remain host Unsupported and bypass catch/finally. No default quota, native guard,
fixture bytes, corpus count, intrinsic or dependency changes are introduced.

Ordinary character classes now validate empty and inverted contents, literal and
escaped dashes, backspace and character/class escapes, range endpoint kinds, and
numeric ordering. Set-valued escapes cannot serve as range endpoints in either
ordinary or Unicode mode; Annex B extensions remain disabled. Character values
decode adjacent hexadecimal lead/trail surrogate escapes only in Unicode mode,
including the nearest eligible pair. Brace escapes and raw units do not join
hexadecimal escapes. Three parser regressions cover these distinctions, grammar
entry points, raw unpaired UTF-16, and a 100,000-character class without a default
quota. The new range diagnostic snapshot and updated lexical-goal snapshot were
inspected. Eval rejects reversed and set-valued ranges as SyntaxError before
effects; valid classes still reach host Unsupported for matching. Unicode property
escapes were excluded from this initial step. Corpus inventories,
runtime defaults, native guards and dependencies are unchanged.

Named captures validate IdentifierStartChar/IdentifierPartChar with Unicode
escapes and astral code points, including paired raw units without u/v. Group-name
escapes always use UnicodeMode as required by their productions. Names retain
their decoded spelling without normalization or case folding. Named forward
references resolve against all captures; named captures also contribute to the
numbered capture count. Alternative name sets implement MightBothParticipate:
a separating disjunction permits duplicates, while successive terms and names
enclosing their own disjunction conflict. Sets move through the iterative group
stack without repeatedly copying all inner names. Five parser regressions cover
these rules, malformed names, raw lone surrogates, all grammar entry points,
20,000 nested named captures and 20,000 alternatives without default quotas. The
new diagnostic snapshot was inspected. Eval rejects invalid and duplicate names
before source effects; valid alternative-separated names still produce host
Unsupported for matching. Unicode property validation was excluded from this
initial step. Corpus inventories, defaults, native guards and dependencies are
unchanged.

Flat UnicodeSetsMode classes now validate ClassUnion operands and scalar ranges,
ClassSetCharacter's syntax and reserved double punctuation, the additional
reserved-punctuation escapes, and ClassStringDisjunction. Their MayContainStrings
result distinguishes empty, single-code-point and longer strings, including paired
surrogate escapes, without a default string-length quota. Inverted classes reject
empty or multi-character alternatives; the leading inversion caret is distinct
from a literal subsequent caret. Four parser regressions cover these rules,
ordinary/u/v distinctions, exact diagnostics across grammar entry points, and
100,000-character class strings. The new diagnostic snapshot was inspected.
Eval rejects supported v-class errors before effects, while valid unions retain
host Unsupported for matching. Nested classes, set operators and Unicode property
escapes were excluded from this initial step. Corpus inventories, runtime defaults,
native guards and dependencies are unchanged.

UnicodeSetsMode nested classes, intersections and subtraction now use an iterative
class frame stack. Operator expressions require operands, excluding ranges and
implicit unions; mixed intersection/subtraction and a third unescaped ampersand
are rejected. MayContainStrings follows the specification's OR for unions, AND
for intersections and left operand for subtraction, independently of whether
actual matching could cancel string sets. Inverted inner classes reject strings
before enclosing operators combine their results. Three additional parser
regressions cover nested grammar, the complete empty/single/multiple string
containment matrix, exact diagnostic entry points, 20,000 nested classes and
20,000 intersection operands without a default quota. The new diagnostic snapshot
was inspected. Eval rejects missing operands and mixed operators before effects;
valid nesting and operators retain host Unsupported for matching. Unicode
property validation was excluded from this initial step. Corpus inventories,
runtime defaults, native guards and dependencies are unchanged.

Unicode property expressions now validate ASCII-only name/value grammar and
exact, case-sensitive edition-17 aliases. General_Category and Script values come
from digest-pinned Unicode 18 alias data; Script_Extensions shares Script values.
Binary properties use the specification's explicit whitelist, rejecting additional
UCD properties, loose matching, Is-prefixes and WSpace. Seven string properties
require v mode, reject P negation, and propagate MayContainStrings through nested
classes and operators. Five parser regressions cover all six nonbinary property
aliases, extra UCD value aliases, Unicode 18 scripts, binary alias boundaries, all
seven string properties, source grammar entry points and 20,000 property atoms
without a default quota. The new diagnostic snapshot was inspected. Eval rejects
property errors as SyntaxError before effects; valid properties retain host
Unsupported for matching. CI checks the new generator against its pinned sources.
Corpus inventories, runtime defaults, native guards and dependencies are unchanged.

136 unchanged Unicode property escape originals add 272 reviewed parse-negative
variants for exact alias spelling, expression grammar, prohibited binary values,
excluded properties, missing nonbinary values and class-range endpoints. All
144 whole programs and metadata in the property-escapes directory were reviewed.
Two positives require matching; six double-backslash negatives are excluded
because their unrelated quantifier error masks the intended unknown property-value
rejection. Every selected literal has exact reviewed diagnostic spans/messages on
both toolchains. Original bytes, assertions, metadata, the existing pin, harness,
native guards, runtime defaults and dependencies are unchanged. This provides no
matching or RegExp object execution credit.

106 unchanged core Pattern originals add 212 reviewed parse-negative variants
covering scoped modifiers, quantifier placement/bounds, assertion quantifiers,
character/control/Unicode escapes, and out-of-bounds numbered references. All
112 whole candidate programs and metadata were reviewed. Four class-range files
were excluded from this core selection; two escaped overlap files are excluded
because their forbidden modifier escape masks the intended overlap error. Exact literal spans/messages
are required for every selected rejection on both toolchains. This adds no
matching or RegExp object execution credit. Historical descriptions and modifier
placeholder comments remain unchanged, along with source bytes, assertions,
metadata, the pin, eight helpers, unlimited defaults and native guards.

Four unchanged ordinary class-range originals add eight reviewed parse-negative
variants for set-valued first, second and paired endpoints, including a literal
dash endpoint. These are the four files excluded from the preceding core Pattern
selection. All complete sources and metadata were reviewed; both toolchains
verify the exact literal spans and expected endpoint diagnostic. Historical
runtime-semantics wording and Annex B descriptions retain their original bytes.
This provides no matching or RegExp object execution credit. The pin, assertions,
metadata, eight helpers, runtime defaults and native guards are unchanged.

55 unchanged named-capture originals add 110 variants: 108 reviewed parse
negatives and two harness positives for eval's SyntaxError on lone surrogate
names. Reviews require exact literal spans and diagnostics for malformed names,
identifier/Unicode escapes, duplicates and missing references, plus one invalid
identity escape within a named capture. All 56 whole programs and metadata in
the literal named-groups directory were reviewed; the forward-reference execution
original remains excluded because it needs matching. Native regressions validate
forward-reference syntax independently. Historical descriptions and clause names
retain their original bytes. Both toolchains pass every selected variant without
matching or RegExp object execution credit. Sources, assertions, metadata, the
pin, eight helpers, defaults and native guards are unchanged.

34 unchanged RegExp lexical-boundary originals add 68 Script/StrictScript variants:
36 harness positives and 32 reviewed parse negatives. Programs verify comment
boundaries and eval's SyntaxError for forbidden line terminators. Negative reviews
identify actual malformed literals, the unterminated comment, and leading dots
after empty-literal comments with exact messages and byte spans. Historical
CR-labelled files containing LF retain their bytes and are checked at that LF.
The focused 53-program review excludes eighteen whole files requiring RegExp
execution, including one negative with an initial complete literal, and one
paragraph-separator file containing an earlier line separator that would mask its
intended error. This does not claim complete directory coverage or Pattern, flag,
matching, or RegExp object execution. Both toolchains pass every selected variant;
the source bytes, assertions, metadata, pin, eight helpers, and unlimited defaults
remain unchanged.

Production crates use std and workspace crates, with jiff for host time-zone
discovery and pinned jiff-tzdb for portable raw IANA data in spite-runtime.
The design also permits regex or regress for matching; neither is added yet.
insta remains a test-only dependency. Rust unsafe code is forbidden through
inherited workspace lints.

## Not implemented

The core crate provides immutable, shared Symbol identities and distinct
string/symbol PropertyKey values, with identity-based equality and hashing.
Descriptions preserve UTF-16, including lone surrogates and absent versus empty
text. Object storage handles both key kinds, orders symbols after string keys,
and preserves descriptor, prototype, and GC behavior. Symbols never alias Array
length/indices, String wrapper indices, or mapped arguments by description.
Borrowed lookup keys avoid allocation; symbol comparison/copy work is constant.
Runtime Symbol values support identity equality, truthiness, typeof, and abrupt
numeric/implicit string conversions. String(symbol) produces its descriptive
UTF-16 string, with bounded allocation. The 13 edition-17 well-known identities
are shared across realms and host threads. Realm property operations, descriptors,
copying, integrity operations, and enumeration preserve both key kinds. Computed
keys retain Symbol identity, and anonymous function names use bounded bracketed
descriptions. ToPrimitive observes symbol-keyed hooks, exact hints, original
receivers, and abrupt completions before ordinary conversion. Native and Script
tests cover this foundation. Symbol wrappers, branded methods, description access,
fresh intrinsic calls, and the 13 fixed constructor properties are implemented.
Symbol.prototype has no SymbolData, and new rejects the intrinsic before coercing
the description. Symbol.for/keyFor share an append-only registry across realms
and threads, preserving exact UTF-16 keys and identity through realm destruction.
Coercion runs before locking; copying runs after unlock. Registry operations and
constructor enumeration have native and Script coverage. The JavaScript Symbol
global now exposes the retained intrinsic with standard attributes. Thirty-four
reviewed Symbol files exercise construction, identity, boxing, registry access,
descriptions, branded methods, and conversion hooks through the upstream harness.

The `spite-heap` foundation provides generational storage with opt-in capacity
limits, checked cross-heap identity, stale-handle rejection, and bounded iterative
collection from explicit roots. Handle, work or allocation failures occur before
sweeping. Ephemeron entries retain values only through reachable containers and
keys; waiting values are indexed by validated key slots and activated once.
Unreachable conditional cycles remain collectible, and stale weak keys cannot
activate reused slots. Nine heap regressions cover all root combinations, value
back-edges, reverse 20,000-key chains, newly activated containers/strong edges,
primitive values, invalid handles, and every work failure during delayed
activation. Scratch state and sweep free-list growth use checked reservations
before removing any values. Runtime WeakMap intrinsics and Symbol-key reachability
remain pending; corpus inventories, runtime defaults and native guards are
unchanged.
Successful marking additionally supports allocation-free removal of inactive
entries from reachable containers. Six cleanup regressions check primitive value
ownership, stale generations alongside live replacements, late key activation,
all insufficient work budgets, invalid handles, and unreachable containers.
Cleanup runs after the complete reachability fixpoint and all fallible checks;
failed collection preserves entries as well as heap values and generations.
Runtime object records now include private object-key WeakMap storage, with a
dense entry vector and identity hash index. Nine regressions cover replacement,
deletion and moved indices, both container/key root requirements, conditional
cycles and back-edges, newly activated maps, host roots, Symbol value release,
generation reuse, invalid handles/brands, and opt-in failures before mutation.
The object and heap-entry trace adapters forward conditional edges and cleanup;
JavaScript WeakMap exposure and Symbol keys remain pending.
The shared Date foundation implements TimeClip, MakeTime, MakeDate and exact UTC
calendar decomposition for clipped integral milliseconds. Eight regressions
cover signed zero, fractions, non-finite input and overflow, both ±8.64×10¹⁵
endpoints, pre-epoch fields, year zero and leap centuries, complete positive and
negative 400-year cycles, and observable floating-point operation order. Fixed
UTC examples and 4,109 full-domain sample/boundary timestamps were also checked
against Node's Date getters. Local time zones remain pending.
Date interchange string syntax now parses directly from UTF-16 with exact
element widths, expanded years, defaults, explicit offsets and unresolved local
date-times. Five regressions include two reviewed insta snapshots for accepted
forms and rejected syntax, all absent-zone form combinations, offset boundaries,
and unpaired surrogates. UTC and explicit-offset fields now convert through exact
calendar normalization and offset adjustment before final range validation.
Six additional regressions cover numeric MakeFullYear adjustment, preserved
interchange years 0–99, signed offsets, leap/calendar-day and hour-24 rollover,
both clipped endpoints with offsets, unresolved local forms, and invalid native
records/extreme years without overflow. Local zone resolution remains pending.
A separate reference check round-tripped 4,103 Node ISO strings spanning the
clipped domain to their exact original time values.
Canonical ISO formatting now emits full UTC fields, millisecond precision and
four-digit/signed six-digit years across the clipped domain. A reviewed insta
snapshot covers exact strings at epoch/year-width/range boundaries and invalid
native values; a 4,096-case regression checks parse/format round trips and fixed
ASCII output widths. All 4,103 reference ISO strings also matched Node's formatter
output exactly.
Date runtime regressions cover the complete constructor/prototype property graph,
timestamp clipping, absent/undefined construction, copy bypass of hooks, default
versus string/number conversion hints, input/new-target ordering, subclass slots
and collection roots, branded receivers independent of prototype identity, UTC
fields and both range endpoints, transactional setTime conversion, expanded ISO
strings, generic toJSON and JSON serialization, invalid-date strings and
RangeErrors, optional output quotas, and call-state recovery. Pending local time,
numeric calendar construction, calendar setters and legacy string operations
report Unsupported; regressions verify that host failures bypass JavaScript
catch/finally and that numeric constructor conversions retain their order.
The realm's complete Date metadata graph adds 49 objects, bringing the shared
initialized-realm entry count to 363. No host quotas are enabled by default.
162 unchanged pinned Date programs add 324 positive normal/strict Script variants
for timestamp/copy construction, metadata, coercion and abrupt completions,
Date.now, UTC/offset parsing and range boundaries, UTC fields, setTime, ISO
branding/RangeErrors, generic JSON/primitive conversion, and invalid-date strings.
The focused review read 177 whole programs: fourteen still require pending Date
operations and one requires the unchanged helper's global own-key enumeration.
Those files remain outside the corpus. The eight harness helpers and existing pin
are unchanged; this is not a complete Date suite review.
UTC time setters now cover hours, minutes, seconds and milliseconds with captured
time-value semantics, once-only ordered conversions, omitted versus explicit
undefined fields, negative/calendar-boundary rollover, TimeClip endpoints,
floating-point cancellation and non-finite inputs. Regressions exercise hooks
that mutate the Date, including invalid dates revived during conversion, later
abrupt completions and ignored excess arguments; recursive conversion hooks
remain subject to the existing native-stack guard.
An independent reference check matched Node's UTC setter return values and final
timestamps for 4,289 random, range-boundary, non-finite and cancellation cases.
28 unchanged upstream UTC time setter programs add 56 normal/strict positives for
metadata, conversion order and captured-time mutation. The review read all 36
whole files in those four setter directories; eight required numeric Date.UTC
construction and were held out for follow-up. The pin and eight harness
files are unchanged.
Standard toUTCString formatting and parsing of its own output cover the complete
clipped domain, English day/month names, year zero and signed minimum-four-digit
years. A reviewed insta snapshot covers names, year widths and both endpoints;
4,096 samples plus short years and boundary cases check exact containing-second
round trips. Regressions reject malformed UTF-16, invalid calendar days, weekday
mismatches and noncanonical forms. Five runtime regressions cover receiver
branding, ignored hooks/arguments, literal short years, string-hint conversion,
abrupt completion and native-stack recovery. A private regression checks exact
opt-in output quotas at every year width; default quotas remain disabled.
An independent reference check matched 4,217 Node UTC strings and their exact
containing-second round trips. Local Date strings and other legacy forms remain
pending.
Seven unchanged pinned toUTCString programs add 14 normal/strict positives for
weekday/month names, negative-year widths and metadata. All nine whole files
were reviewed: invalid-date was already included, and format requires pending
RegExp.exec. The pin and eight harness files are unchanged.
setUTCDate now preserves the captured year/month/time through day conversion,
normalizes day rollover with ordered Number arithmetic and clips only the final
timestamp. Five regressions cover leap centuries and year zero, signed/fractional
days, both clipped endpoints and an out-of-range intermediate month boundary,
mutating/throwing hooks, invalid-date revival, receiver branding, frozen slots,
ignored extra arguments and normal-stack recovery. A reference check matched
Node's return values and final timestamps for 4,216 random, boundary and
non-finite cases. Numeric Date construction, other calendar setters
and local zones remain pending; default quotas and native-stack guards are unchanged.
All seven whole upstream setUTCDate programs were reviewed and vendored unchanged,
adding 14 normal/strict positives for metadata, conversion order and captured-time
mutation. The pin and eight harness files are unchanged.
Shared MakeDay now implements exact mathematical month division and Gregorian
year arithmetic before Number rounding, finite first-day witness checks, and
ordered day addition without premature TimeClip or input caps. Six regressions
cover literal short years, fractions and negative month rollover, leap centuries,
range endpoints and intermediate boundaries, overflow and impossible witnesses,
huge month/year and year/day cancellation, exact Gregorian rounding, and opt-in
work failures distinct from NaN. The existing workspace BigInt crate supplies
wide intermediate integers; no external dependency or default quota is added.
An independent check matched 4,096 Node calendar cases, 1,024 exact huge-month
cancellations and 2,048 expanded calendar-witness searches. Remaining calendar
setters and local numeric construction are pending.
Date.UTC now converts all present numeric components in order, applies absent
defaults and input-year adjustment, and clips only after shared calendar/time
arithmetic. Six runtime regressions cover undefined/omitted fields, short years,
fractions and rollover, both range endpoints, floating evaluation order, NaN
and abrupt coercion ordering, ignored arguments, huge valid cancellations and
normal-stack recovery. A private regression verifies that Number calendar work
ignores BigInt value-magnitude quotas and reports opt-in work failure as a host
limit. A reference check matched 4,152 Node results spanning ordinary, random,
non-finite, clipped-boundary and floating-order cases. The intrinsic property
graph, default quotas and native-stack guards are unchanged.
25 unchanged pinned programs add 50 normal/strict positives: all 17 whole
Date.UTC programs and eight previously excluded UTC time setter rollover files.
The review covers ordered coercion/errors, absent defaults, short years,
non-finite/fractional fields, clipped boundaries, floating arithmetic, metadata
and rollover. All 36 programs in the four UTC time setter directories are now
included. The pin and eight harness files are unchanged.
setUTCMonth and setUTCFullYear now preserve captured calendar fields and time
through ordered conversion, retain literal short years and normalize rollover
with final clipping. Eight runtime regressions cover omitted/undefined fields,
leap centuries and year zero, signed/fractional months, invalid-date revival,
mutating/throwing hooks, coercion order, branding, frozen slots, ignored arguments,
range endpoints, huge valid cancellations and normal-stack recovery. A private
regression verifies transactional opt-in work failure and independence from
JavaScript BigInt magnitude quotas. An independent check matched 4,336 Node
return values and final stored timestamps. Local calendar setters and numeric
local construction remain pending; default quotas and native-stack guards are unchanged.
All nine whole upstream setUTCMonth programs and six setUTCFullYear programs were
reviewed and vendored unchanged, adding 30 normal/strict positives for ordered
coercion, captured-time mutation, literal short years, rollover and metadata.
The pin and eight harness files are unchanged.
Numeric Date construction now shares Date.UTC's ordered component conversion
and calendar/time arithmetic. Non-finite intermediates create invalid Dates
before zone resolution; finite intermediates remain unresolved and unclipped.
Nine local getters and six local setters implement their captured-invalid-time
branches. Setter coercion still runs in order and preserves hook revival or
abrupt completion without writing the slot. Seven runtime regressions cover all
component positions, arithmetic overflow/impossible calendar witnesses, ignored
arguments, new-target prototypes and subclass slots, coercion/error ordering,
branding and frozen Dates, finite boundary values, and normal-stack recovery.
Finite local operations and local setFullYear remain pending. The property graph,
default quotas and native-stack guards are unchanged.
29 reviewed unchanged pinned programs add 58 normal/strict positives: two
constructor-infinity ISO RangeErrors previously held out, nine invalid local
getters, and eighteen local setter programs covering invalid results, ordered
coercion and captured-time mutation. This focused review uses the existing pin
and eight harness files without claiming finite local-time support.
Unclipped native calendar decomposition now accepts every i64 epoch millisecond
without TimeClip while preserving the existing clipped Date entry points.
Two regressions cover both integer endpoints, neighboring milliseconds, Date
range neighbors and 8,192 integer round trips. An independent civil-from-days
reference matched all eight fields for 8,203 native integer times. Existing UTC
formatting snapshots and clipped-domain tests retain their original expectations. This prepares
calendar fields for local zone arithmetic; finite local operations remain pending.
Date string records now expose exact nominal calendar milliseconds before zone
conversion. UTC/offset conversion shares that widened helper, preserving final
range checks after offset adjustment. Two regressions cover local/UTC/offset
equivalence, literal year zero, day/hour rollover, both clipped boundaries,
native i32 year endpoints, invalid day/time records and separate zone validation.
An independent Gregorian reference matched 4,102 native field records, including
both i32 year endpoints. Existing parse and formatting snapshots are unchanged;
local zone resolution remains pending.
Recurring transition calendar arithmetic now covers TZif's Julian and
month/week/weekday forms and signed transition times using wall, standard or UTC
clocks. Five regressions cover leap-year conventions, fourth/fifth weekday
occurrences, New York and southern-hemisphere half-hour transitions, negative
daylight adjustments, cross-year times, Date boundaries, native integer year and
offset endpoints, and malformed rules. Complete zone histories and finite local
Date operations remain pending. An independent Gregorian reference matched
8,624 native transition records, including all native year and offset endpoints.
The TZif POSIX rule parser borrows names and normalizes written offsets and
daylight defaults without loading a zone. Five regressions and an inspected
insta snapshot cover standard/daylight designations, Julian and weekday rules,
extended transition times, offset sign/default boundaries, malformed and
truncated input, non-ASCII mutations and long borrowed names. Jiff agrees on
acceptance and normalized offsets/rules for 3,759 strings, including all 93
distinct nonempty rule footers in the installed IANA 2026b data. Complete
zone histories and finite local Date operations remain pending.
Compiled recurring offsets now use the exact Gregorian cycle for integer
lookups without allocations or a backend year cap. Seven regressions cover exact
millisecond changes, southern half-hour seasons, negative daylight adjustments,
fixed/perpetual daylight zones, leap conventions, cross-year times, skipped
transition years, Date/native integer endpoints, malformed records and
contradictory simultaneous changes. An independent equivalent-calendar mapping
matched Jiff offsets for 94,581 queries over all 93 installed IANA rule footers,
including native i128 endpoints. The mapping is specific to recurring Gregorian
rules; complete historical zone data and finite local Date operations remain
pending.
TZif offset loading now preserves explicit historical transitions and recurring
footers without a native calendar cap. Nine regressions cover versions 1–4,
exact signed 32/64-bit transition boundaries, negative subsecond history, Date
boundaries, fixed/recurring/no-footer behavior, legacy-body skipping, footer
version syntax and consistency, the full 256-type format, invalid ordering and
flags, malformed/truncated data and adversarial counts. Unknown versions and
leap-second time scales remain explicit unsupported results. An independent
comparison matched 349,734 historical/recurring queries over 598 bundled IANA
2026c zones and 436 installed IANA 2026b files. It queries Jiff at the containing
integral second to avoid its negative-subsecond TZif truncation; raw records and
Python zoneinfo independently confirm the Abidjan millisecond regression.
Display names and finite local Date operations remain pending.
Local offset resolution now applies UTC's earliest-fold and preceding-gap
policies to both recurring and historical histories without final clipping.
Seven regressions cover exact gap/fold milliseconds, half-hour and negative
daylight adjustments, Date and native year boundaries, native arithmetic
overflow, historical second precision, overlapping local images, the latest
epoch at a shared local endpoint and recurring/historical cutoff separation.
An independent Python zoneinfo comparison matched 773,138 local resolutions
over 1,034 bundled/system TZif files, including historical gap/fold boundaries.
Finite local Date construction/mutation and display names remain pending;
default quotas and native-stack guards are unchanged.

The public immutable TimeZone loader now preserves host TZif data and falls back
to the explicitly pinned IANA 2026c jiff-tzdb 0.1.8 bundle only for missing named
files. Jiff 0.2.37 provides platform discovery, with only std, system discovery,
host databases and the shared bundle features enabled. No Jiff calendar queries
replace the checked full-range arithmetic. Explicit inputs support fixed native
offsets, complete POSIX rules and unnamed TZif bytes. System loading honors TZ
and reads unnamed Unix localtime histories directly. Failed host reads or data
validation never silently substitute UTC or another database version.
Realm loading is lazy and a native host can install or reset an independent
override; immutable histories can be shared. Eight regressions cover all 598
bundled zones at both Date endpoints, historical second precision, gaps/folds,
wide recurring years, native overflow, ordered file precedence, malformed and
unreadable files, path traversal, aliases and independent realms. UTC/invalid
Date branches still leave system-zone discovery untouched. No fixture inventory,
pin, default quota or native-stack guard changes in this step.

All nine local Date getters now use the realm's checked history and exact native
calendar decomposition after applying its whole-second offset. Local fields
remain unclipped at both Date endpoints, and timezone offsets preserve historical
seconds and positive zero in UTC. Invalid time values still return NaN before
host discovery. Six native regressions cover spring/autumn and half-hour changes,
historical offsets, aliases, branded receivers, ignored arguments, independent
realm changes and recursive reads on a 2 MiB stack. Twenty independently computed
Gregorian reference records verify 180 fields at Date boundaries and full native
offset extremes. Two CLI regressions cover explicit POSIX/empty settings and an
inspected insta snapshot for malformed zone data. Host configuration failures
use Error::Host, skip JavaScript catch/finally and become Test262 setup failures;
unsupported data formats and failed native allocation retain separate categories.
UTC and invalid-Date reads continue to work with malformed host configuration.
Local setters and local/legacy string output remain separate steps.

Numeric Date calendar construction and local interchange parsing now resolve
through the selected realm history before TimeClip. Eight regressions cover
ordered conversions/defaults, short input years and literal interchange years,
calendar rollover, gap/fold milliseconds, half-hour shifts, Apia's skipped day,
expanded years, all twenty independent native-offset boundary records, ignored
arguments, branded subclass construction, prototype lookup and recursive re-entry
on a 2 MiB stack. Huge finite calendar cancellations remain valid even with an
opted-in JavaScript BigInt magnitude quota; no such quota applies to Number
calendar intermediates. A CLI control verifies that non-finite construction,
invalid syntax, date-only forms and explicit UTC strings do not load malformed
host configuration, while finite local construction reports a host failure.
The fused TimeClip(UTC(t)) helper rejects only finite values whose final result
cannot fit Date's domain for any supported native offset; it does not impose an
input year, work or heap allowance. Local setters and string output remain pending.

The following local calendar corpus review adds 119 unchanged originals and 238
normal/strict positives at the existing pin. They cover constructor conversion
and calendar normalization, prototype selection, date-only UTC versus local
date-time parsing, and all nine complete local getter directories. All 238
variants pass under UTC, New York, Lord Howe and Paris. The two original upstream
assertRelativeDateMs.js and dateConstants.js helpers bring the current inventory
to ten harness files. Of 127 candidate whole programs, three cross-realm files,
three Date() files, one own-local-string parsing file and one global property
helper file remain excluded for their pending capabilities. Original assertions
and bytes, the pin, unlimited defaults and native-stack guards are unchanged.

All seven local Date setters now derive defaults from the captured local calendar
and resolve the resulting local time through the selected realm history before
final clipping. Ordered conversions preserve hook mutations on abrupt completion
and on the six captured-invalid branches that return NaN without writing. Only
setFullYear revives an invalid Date from the local epoch calendar; its year
conversion precedes local lookup and optional month/date conversions follow it.
Literal short years are preserved. Eight native regressions cover omitted versus
undefined fields, truncation and rollover, DST folds/gaps, half-hour changes,
Apia's skipped day, historical seconds, captured-slot mutation, ordered abrupt
conversions, branding, frozen slots, huge finite cancellations and recovery after
recursive coercion. All seven setters round-trip twenty independent native-offset
boundary calendars, including both Date endpoints without clipping local fields.
A CLI regression verifies conversion/host-error ordering with malformed zone
data and independent invalid-Date branches. Local/legacy strings and Date()
function output remain pending; the fixture inventory is unchanged in this step.

A separate local setter corpus review adds all 105 remaining unchanged originals
and 210 normal/strict positives at the existing pin. All 123 whole files in the
seven local setter directories are now included, covering coercion/errors,
rollover/defaults, captured slot values, invalid-Date revival, final clipping,
receiver branding and metadata. The new cohort passes on both compilers under
UTC, New York, Lord Howe and Paris. The ten harness helpers, original source bytes
and unlimited runtime defaults are unchanged.

The shared Date core now formats DateString, TimeString, TimeZoneString and finite
ToDateString output with unclipped local fields and minimum-width literal years.
The normative GMT hours/minutes wrap at 24 hours and omit seconds; historical
seconds and offsets of a day or more are retained in the permitted optional
timezone name `(UTC+HH:MM:SS)` or its negative form. Other offsets use an empty
name. The borrowed UTF-16 parser validates the actual calendar/weekday and both
offset representations, subtracts the exact offset, then checks only the final
UTC range. This preserves own whole-second round trips independently of host
configuration, including both Date boundaries and every native offset.
Four core regressions cover all short years, 4,096 dispersed timestamp/offset
pairs, boundary seconds, malformed calendars/suffixes and non-ASCII code units
at every output position. The full insta snapshot was inspected; an independent
Gregorian calculation confirms 28 offset/calendar strings and both native i64
endpoints. Runtime string methods and Date() still await wiring in the next step;
the pin, fixture inventory and unlimited defaults are unchanged.

Date(), toString, toDateString and toTimeString now use the shared local string
helpers. Finite output uses the realm's exact offset and unclipped local calendar;
invalid values return before host loading. Date() ignores supplied values while
the caller still evaluates argument expressions. Branded methods ignore receiver
conversion properties and arguments. Date.parse and one-argument construction
accept required own-output strings without loading the current zone. Six native
regressions cover both sides of folds, historical seconds, negative years, all
twenty independent offset/boundary records, zone changes after saving output,
ignored hooks, branding, frozen instances, exact opt-in output limits and recovery
after recursive primitive conversion. A CLI control verifies malformed-host
failures and independent invalid/saved-string branches. The three locale methods
remain Unsupported; the fixture inventory is unchanged in this implementation step.

A separate local string corpus review adds nineteen unchanged originals and 38
normal/strict positives for Date() output/ignored values, own-string parsing,
negative-year widths, branding and metadata. All 21 whole files in the three
string directories and four earlier held-out programs were reviewed. Three
invalid-Date files were already present; the three RegExp.exec-dependent format
files remain excluded without pass credit. The cohort passes on both compilers
under UTC, New York, Lord Howe and Paris. The pin, ten helpers, original assertions
and unlimited defaults are unchanged.

The three Date locale string methods now use the edition-17 non-Intl fallback.
This host's fixed English conventions map to the corresponding local date/time
formatter; no receiver formatting method is invoked. Both reserved argument
positions are ignored without coercion or property reads, as required without
ECMA-402. Two regressions cover poisoned options and locale values, evaluated
extra arguments, overridden methods, branding, frozen instances and invalid Date
values across UTC, New York, Lord Howe and Abidjan. The CLI host control now also
covers all three locale methods, including malformed zone data and invalid
branches that return without loading it. Exact offsets and opted-in output
limits reuse the already verified shared helper. All edition-17 Date bodies now
dispatch explicitly; Annex B and ECMA-402 remain outside the target. The fixture
inventory, pin, unlimited defaults and native-stack guards are unchanged.

A separate locale-method review vendors all twelve unchanged programs across
toLocaleString, toLocaleDateString and toLocaleTimeString at the same pin. All 24
normal/strict positives pass on both compilers under UTC, New York, Lord Howe and
Paris. They verify intrinsic length, name, descriptors and non-constructibility,
without claiming ECMA-402 coverage. All four metadata files in each directory
are included; the ten harness helpers and unlimited defaults are unchanged.

Runtime object records add string/symbol data/accessor descriptors, extensibility,
deletion, array-index ordering, and tracing of prototype handles. Frozen
properties use SameValue and preserve equivalent NaN payloads. Accessor descriptors
validate callable handles, trace getter/setter edges, preserve omitted attributes,
and enforce non-configurable identity and kind invariants. Reads/writes dispatch
accessors with the original receiver after releasing storage borrows. These records
support an opt-in host property-capacity quota. A heap context adds bounded iterative
prototype traversal, cycle-checked mutation, inherited data reads and presence
checks, and receiver-sensitive writes. Rust values carry object identity and
trace object-valued property edges, including cycles. Object literals parse with
literal/computed keys, shorthand, trailing commas, and duplicate prototype-setter
early errors. Evaluation preserves key/value order and implements ordinary data
properties, prototype initializers, ordinary methods, and getters/setters. Object
spread skips nullish sources, boxes primitives, snapshots ordered own keys, and
reads descriptors and values live. Enumerable string and Symbol keys become fresh
data properties, bypassing setters; spread `__proto__` keys remain data. Regressions
cover getter receivers, key deletion and enumerability changes, ordered overwrites,
abrupt getters, and opt-in copy limits. Explicit collection between evaluations retains persistent bindings,
intrinsics, and host roots. Returned/thrown object values need a host root to
survive collection; tokens release roots when their last clone is dropped. Root
registry storage is reused. Collection during evaluation remains disabled until
temporary and pending-completion root lifetimes are implemented. Lexical
environments now share the generational heap, carry stable identities and outer
links, and trace object-valued bindings. Per-iteration environments preserve
previous identities. `max_heap_entries` replaces the earlier object-only slot
limit name; explicit collection reclaims abandoned scopes too.

Dotted and computed property references parse and evaluate for ordinary objects,
including assignment, compound assignment, update, and delete targets. Tests cover
IdentifierName spelling, precedence, strict targets, ASI, inherited reads/writes,
and edition-17 deferred key conversion. Nullish bases produce TypeError at
GetValue/PutValue/delete. Primitive property operations use ephemeral wrapper
semantics; String indices and length are read-only and non-configurable. Writes
do not persist on primitives, and strict failures throw TypeError. Reading a
missing standard prototype method reports Unsupported, while ordinary object
properties can shadow it. Optional legacy Object.prototype accessors/helpers (20.1.3.8–9) and Annex B
String methods are not installed.

The `in` operator includes inherited and undefined-valued properties, rejects
primitive right operands before converting the left key, and recognizes standard
Object.prototype method presence. The parser applies the In grammar parameter to
for initializers and nested expressions. For-in/of iteration remains unsupported.

Call expressions parse, retain member receivers, and evaluate callee then arguments
in order. Non-callable values throw TypeError after argument evaluation. Builtin
function objects inherit callable Function.prototype and expose standard name/length
descriptors. Object.prototype has an immutable null prototype and its mandatory
string-keyed methods. Default ordinary-object conversion is supported. Intrinsic
initialization is atomic and the initialized objects remain rooted. Function.prototype caller/arguments
accessors use the shared, non-extensible %ThrowTypeError% with frozen name/length
metadata. Their reads/writes throw catchable TypeError in both modes. Optional
calls remain open; missing operations report Unsupported. Ordinary calls and new
support iterable spread arguments, consuming each iterator before later arguments
and retaining the original callee/member receiver. Next is cached once; failures
in next/done/value do not close. Non-callable/constructible checks follow argument
evaluation. Argument storage checks platform capacity without a default quota.
Function.prototype
call passes receivers unchanged through bounded iterative dispatch. Native function
toString uses the original builtin name even after public name changes; generated
strings obey host limits. Apply accepts ordinary array-like objects, converts
length with ToLength, and reads inherited/indexed getters before the target call.
Argument lists have a configurable host limit shared with direct calls and bound
arguments. Bind implements callable receiver/argument capture, target prototype
selection, standard length/name metadata, and iterative invocation. Captured
object edges survive collection and unreachable cycles are reclaimed. Re-entrant
getter/coercion calls have a host nesting limit of 32; tail transfers are iterative. Embedding threads require
at least a 2 MiB native stack, exercised by recursion regressions in CI.
Those regressions include direct and indirect eval through property, optional,
call/apply/Reflect, and bound forms, conditional/sequence execution, mixed function
calls, and nested eval/Function compilation. They verify host errors before stack
exhaustion, handler/finalizer bypass, and usable realm state afterward.

The Function global exposes the standard intrinsic constructor/prototype graph,
name/length/prototype descriptors, inherited branding, and complete own reflection.
Tests cover prototype constructor links, restricted accessors, integrity operations,
newTarget validation, and collection after public deletion. Calling or constructing
Function compiles ordinary bodies from arbitrary UTF-16 source with ordered input
conversion, global-environment capture, body-derived strictness, and prototype
selection after successful parsing. The parser independently validates parameter and
body grammar goals, combine their early errors with body-derived strictness, and
retain the standard anonymous source text. Insta snapshots cover its AST/source
and syntax injection diagnostics. Regressions cover calls/new/bound/Reflect,
default/rest/duplicate parameters, arguments aliasing, caller scope isolation,
conversion/error/prototype order, constructor behavior, exact source/metadata,
collection and deleted intrinsic links, sources exceeding one MiB under defaults,
and opt-in quotas. Unsupported compiled syntax remains a host abort;
catch/finally cannot disguise that gap as a JavaScript exception.
Lossless source storage preserves lone surrogates in literals, comments, template
raw/cooked values, and exact nested function source ranges. Four parser regressions,
two inspected insta snapshots, and six runtime regressions cover all 2,048 lone
surrogate code points, their invalid identifier diagnostics, valid pairs, real
replacement characters, identity escapes, line-ending normalization, separate
grammar goals, strict early errors, conversion/prototype order, collection, and
exact opt-in encoded-byte boundaries. Scalar spans retain UTF-8 lengths; each lone
surrogate occupies three encoded bytes. `FunctionSource::as_str` is fallible, while
`to_js_string` returns every original UTF-16 unit. All host quotas remain opt-in.

Arrows with identifier parameters and expression or block bodies execute with shared lexical captures,
fresh mutable parameter bindings, missing/extra argument handling, inherited and
body-local strictness, unique names, source retention, and bounded nesting.
Block bodies support local var/lexical instantiation and return completions through
loops, labels, switch, try/catch/finally, including return ASI and early errors. Metadata includes
standard name inference and exact Function.prototype.toString source. Captured
per-iteration/catch/block environments survive collection; unreachable cycles are
reclaimed. Default parameters support ordered initialization, parameter TDZ,
separate body var environments, closure capture, name inference, and length. Rest
identifiers collect remaining arguments into fresh dense intrinsic Arrays. Tests
cover defaults before rest, TDZ, function length/source, unmapped arguments,
body binding rules without parameter expressions, constructors, methods, and
opt-in output quotas. Parameter patterns and async arrows remain gaps.

Synchronous for-of accepts reference targets and single var/let/const bindings.
Tests cover live Array lengths, String code points, cached next methods, RHS TDZ,
fresh lexical environments, and assignment references evaluated after each value.
Iterator closing handles break, return, nonmatching continue, and body/assignment
throws with the required cleanup precedence. Step failures and host failures do
not close; cleanup host errors remain host errors. Completion values, labels,
scope restoration, and captured bindings after collection have regressions.

For-in snapshots own keys separately at each reached prototype, reads descriptors
live, skips Symbols, and suppresses inherited names even when a present own key
is non-enumerable. Deleted keys do not suppress inherited names. Values and custom
iterator hooks are never read. Tests cover mutation, ordered Array indices,
primitive boxing, nullish skipping, closures, TDZ, and completion values. Complete
String and shared Iterator
chains use ordinary ordered enumeration. The non-strict Annex B
initialized-var extension remains a separate unsupported feature.

Strict ordinary functions execute with preserved receivers, shared lexical captures,
hoisting, defaults, and return completions. Unmapped arguments expose original
indices, length, and restricted callee; indices and parameters do not alias. Arrows
capture this and arguments from enclosing functions, including across collection.
Non-strict functions with object, Boolean, Number, String, Symbol, BigInt, or nullish/global receivers also execute. Simple parameter lists
use mapped arguments, including last-duplicate rules, live descriptor values,
receiver-sensitive writes, and detachment on deletion, accessor conversion, or
non-writable changes. Default and rest parameters use unmapped arguments. Parameter/body
arguments declarations shadow or suppress the implicit binding as specified.
Arguments objects now own the intrinsic Array values callable at Symbol.iterator.
Ordinary new expressions create fresh receivers from the current constructor
prototype (or the realm default), run parameters/bodies, and honor object returns.
Bound constructors forward arguments and newTarget while ignoring bound this and
their own prototype property. Arrows and builtin methods remain non-constructible.
Construction shares call nesting and argument limits; deep bound chains are iterative.
new.target is validated in ordinary functions, binds undefined on calls or the
constructor on construction, and is lexically captured/traced through arrows.

Reflect exposes apply, construct, and Symbol.toStringTag on an ordinary object.
Apply validates callability before reading length and ordered live indices, then
calls with the original receiver. Construct validates target and an explicitly
present newTarget before reading arguments, preserves custom prototype selection,
and forwards bound targets iteratively. Explicit undefined newTarget is rejected;
an absent argument defaults to target. Tests cover sparse lists, abrupt getters,
constructor return rules, builtin and bound construction, metadata, collection,
large default argument lists, and opt-in host failures. All thirteen Reflect
methods are exposed for currently implemented object kinds; Proxy traps remain
pending.

Reflect prototype and extensibility methods reject every primitive target without
conversion. SetPrototypeOf returns false for cycles, immutable-prototype changes,
and non-extensible changes, but true for an unchanged prototype. PreventExtensions
returns true for exposed object kinds and remains idempotent. Regressions cover
ordinary/exotic receivers, dormant getters, existing property updates, boolean
rejections, metadata, and collection. Proxy traps remain pending.

Reflect get/has/deleteProperty share ordered ToPropertyKey conversion and reject
primitive targets before converting keys. Get preserves explicit accessor
receivers; has checks prototypes without getter calls; delete returns booleans for
own-property rejection independently of caller strictness. Regressions cover
mutation during key conversion, symbols, absent and inherited properties, dormant
getters, Array/String/arguments exotics, metadata, collection, and host gaps.

Reflect defineProperty/getOwnPropertyDescriptor/ownKeys reuse descriptor conversion
and storage key order. Definition returns false for rejected descriptors while
conversion failures and invalid Array lengths still throw. Own descriptors are
fresh objects without getter calls; own-key Arrays include symbols and
non-enumerables without property-value reads or species/constructor hooks. Tests
cover conversion order, SameValue redefinitions, Array partial truncation,
String/arguments exotics, all key types, metadata, collection, and host gaps.
Other incomplete intrinsic lists remain Unsupported.

Reflect.set shares checked OrdinarySet with assignments, preserving exact setter
receivers and defining writable data on the receiver with boolean rejection.
Receiver prototype setters are bypassed; existing own attributes are retained.
Tests cover primitive receivers, read-only and accessor rejection, ordered key
conversion, setters, Array length/index failures and partial truncation,
String/arguments exotics, ordinary assignments, and missing intrinsic descriptors
before mutation. Reflect has complete own-property reflection and supports
freezing, copying, and for-in over its exposed ordinary prototype chain. Metadata,
deletion, collection, and host-failure regressions retain its intrinsic methods.

Math exposes eight immutable constants, its standard tag, abs/sign, and
ceil/floor/round/trunc. Each of these unary methods converts one argument once through
ToNumber and ignores receiver and extra argument conversions. Tests cover signed
zeros, NaN/infinities, subnormal inputs, fractional boundaries, halfway ties toward
positive infinity, and large odd integral values without adding 0.5. Constants,
method metadata, conversion failures, collection, and deleted public bindings are
covered. All standard Math methods and own reflection are now implemented.

Math max/min convert every argument in order, including those following NaN or
infinite extrema. Abrupt conversion stops at the failing argument; NaN wins only
after all conversions succeed. Empty calls return opposite infinities, and zero
ties prefer +0 for max and -0 for min. Clz32 applies ToUint32 before counting bits;
imul performs ordered conversions and exact multiplication modulo 2^32 with a
signed result and positive zero. Regressions cover conversion side effects and
failures, large argument lists with default limits, integer boundaries, ignored
extra arguments for clz32/imul, metadata, and intrinsic retention.

Fround/f16round use direct ties-to-even binary32/binary16 conversion. Regressions
cover both signs, NaN, infinities, underflow and overflow, normal/subnormal
transitions, adjacent halfway values, conversion order, extra-argument bypass,
metadata, host aborts, and collection. Binary16 tests enumerate every finite
representable magnitude and every intervening halfway boundary with its nearest
binary64 neighbors, for both signs. Direct binary64 rounding avoids errors from
a binary32 intermediate.

Pow shares Number exponentiation after ordered base/exponent conversion; sqrt
uses correctly rounded binary64 square root. Regressions cover IEEE special
values, odd/even exponent signs, large exponents, negative fractional bases,
correct square-root rounding and range endpoints, coercion before special-value
results, skipped extra arguments, metadata, collection, and host aborts.

Exp/expm1 and log/log1p/log2/log10 handle domain endpoints and signed zero
explicitly, with platform approximations for ordinary finite inputs. Log2
returns every representable power-of-two exponent exactly, including subnormal
powers. Regressions cover that full exponent range, accuracy near zero without
intermediate addition/subtraction, infinity/NaN and domain failures, underflow and
overflow, ordered coercion, ignored extra arguments, metadata, and collection.

Inverse trigonometric/hyperbolic functions and cbrt preserve required domain and
signed endpoint results before finite approximations. Large acosh/asinh inputs
avoid intermediate overflow, and atanh retains finite results at the interior
neighbors of both domain endpoints on Rust 1.85 and stable. Atan2 converts y then x,
even when y is NaN, and handles every zero/infinity quadrant explicitly without
forming a ratio. Regressions cover endpoints, all zero/infinity sign combinations,
ordinary finite values, very large and subnormal inputs, ordered conversions,
type errors, extra-argument bypass, metadata, host aborts, and collection.

Circular and hyperbolic functions sin/cos/tan/sinh/cosh/tanh preserve signed
zeros and exact infinity endpoints before finite approximations. Regressions
cover NaN and missing arguments, finite values, symmetry, large and subnormal
inputs, ordered coercion, ignored receiver/extra arguments, type and host errors,
standard metadata, and intrinsic retention after public deletion.

Hypot converts every argument before choosing Infinity over NaN, and returns
positive zero for empty/all-zero calls. Scaled, compensated square summation
avoids intermediate range failures and retains small contributions. Regressions
cover very large/subnormal norms, many small terms, argument permutations,
ordered abrupt conversion, primitive type errors, host aborts, standard function
descriptors, and collection.

SumPrecise validates Number elements without coercion, continues iteration after
nonfinite results, closes on element/count errors, and leaves step failures open.
Finite values accumulate exactly and round once with ties to even, including
cancellation after transient overflow and sticky bits below a halfway boundary.
Regressions cover signed zero, iterator receivers/lookup order, closing precedence,
primitive type rejection, host failures and opt-in work limits, independence from
BigInt magnitude quotas, metadata, and collection. The numerical suite checks
464 independently generated Fraction references in both argument orders and
round-trips representative significands through every normal exponent and every
subnormal single-bit magnitude. CI verifies the reference fixture generator.

Math random produces positive values in [0, 1) with separate realm sequences.
Unit tests cover published SplitMix64 vectors, exact interval endpoints, state
wrapping, concurrent realm sequence allocation, and checked identity capacity.
Runtime tests cover ignored argument/receiver coercion, metadata, collection,
and complete Math reflection/integrity behavior without invoking property values.

Boolean calls, construction, prototype methods, descriptors, and boxed receivers
are implemented. Boolean.prototype itself holds false. Methods validate own
BooleanData slots; ordinary objects inheriting the prototype fail that check.
Boolean primitive property access walks the actual prototype chain and retains
the primitive receiver for inherited accessors. Object.prototype.valueOf boxes
Booleans, and toString recognizes Boolean wrappers. Truthiness, ordinary coercion,
bound construction, collection, and host-limit behavior have regression coverage.

Number calls/construction, NumberData wrappers, valueOf, constants, and
isFinite/isNaN/isInteger/isSafeInteger are implemented. Explicit Number conversion
accepts BigInt with nearest-even binary64 rounding and signed overflow; implicit
ToNumber still rejects it. Number receivers box in non-strict calls. Decimal
toString, radix validation, exact shortest formatting for bases 2–36, and
toFixed/toPrecision/toExponential formatting are implemented. Non-decimal output
uses fixed notation, exact binary64 rounding intervals, and nearest/ties-even
selection among shortest candidates. Independent rational reference vectors cover
all 34 non-decimal radices, subnormals, maximal values, and radix power boundaries.
toFixed uses exact
binary values and rounds ties to the larger magnitude for 0–100 fraction digits.
toPrecision retains 1–100 significant digits and uses exact exponent correction
and rounding before selecting fixed or exponential notation. toExponential uses
the same rounding for explicit 0–100 fraction digits and shortest scientific
notation when the argument is undefined. toLocaleString uses the explicitly
permitted ECMA-262 fallback of ordinary numeric formatting, ignores reserved
locale/options arguments, and provides no ECMA-402 locale services.
Global parseInt/parseFloat and their Number aliases share identity, implement
ordered coercion and longest-prefix parsing, and preserve negative zero. Integer
prefixes accumulate exactly in every radix before one binary64 rounding. UTF-16
surrogate tails terminate a valid numeric prefix. NumberData preserves negative zero
and NaN independently of the object's current prototype.
Global isFinite/isNaN perform ordinary ToNumber coercion, propagate abrupt
conversions, and reject BigInt. The distinct Number predicates remain non-coercing.

Error and all six NativeError constructors support call/new/bound construction,
ordered message and cause initialization, standard descriptors, and their required
constructor/prototype inheritance. Error.prototype.toString is generic and orders
property reads and conversions. Error.isError recognizes own ErrorData independently
of prototype identity; Error prototypes do not carry that slot. Cause references
are traced through ordinary properties. AggregateError adds ordered prototype,
message, cause, and synchronous iterable processing. Its errors property is a fresh
intrinsic dense Array, with values preserved by identity and without coercion.
IteratorToList caches next once and leaves the iterator unclosed on step failures.
Native regressions cover descriptors, ErrorData branding, call/new/bound/Reflect
construction, string iteration, abrupt ordering, setter/species bypass, collection,
unrestricted default list sizes, and opted-in host aborts.

Object calls/new preserve object identity, create fresh nullish-argument objects,
and box Boolean/Number/String/Symbol/BigInt values. Object.prototype provides hasOwnProperty,
propertyIsEnumerable, isPrototypeOf, toLocaleString, constructor, toString, and
valueOf with ordered conversions and receiver handling. Object.prototype.toString reads Symbol.toStringTag
after selecting its fallback, accepts only String tags, and bounds UTF-16 output.
Tag getters receive the original object or a fresh primitive wrapper. BigInt and
Symbol receivers use their wrappers and observable prototype tags. Own-property
predicates inspect string/symbol descriptors without invoking accessors. Object.defineProperty converts
inherited descriptor fields in order and applies data/accessor changes, including
mapped-argument alias updates. Object.getOwnPropertyDescriptor returns fresh,
mutable descriptor objects; Object.hasOwn boxes before key conversion, and
Object.is uses SameValue without coercion. getPrototypeOf/setPrototypeOf expose
prototype identity and reject cycles and immutable/non-extensible prototype
changes; isExtensible/preventExtensions retain their primitive special cases.
Object.create selects an object/null prototype and optionally defines properties.
Object.defineProperties snapshots own keys in specification order, converts every
enumerable descriptor before defining properties, and retains earlier definitions
if a later definition is rejected. Enumeration of incomplete Function/String/Array/Iterator/global
intrinsics remains Unsupported. Object.freeze/seal close extensibility and tighten
own descriptors without invoking accessors or recursively freezing values.
isFrozen/isSealed inspect integrity, with primitive and empty-object special cases;
freezing mapped arguments detaches their parameter aliases. Object.assign copies
enumerable own values in source/key order through ordinary Get/Set, including
getters, inherited setters, and rejected-write TypeErrors. getOwnPropertyDescriptors
copies all own descriptors into a fresh object without invoking property getters.
getOwnPropertyNames/getOwnPropertySymbols return own keys of the requested type,
including non-enumerables, without invoking getters. Object.keys/values/entries
snapshot string keys and recheck each own descriptor's enumerability in order.
Keys skips value reads; values/entries perform live Get and preserve earlier
getter effects on failure. Entries creates dense intrinsic pairs; all result
arrays bypass public constructors, species, and inherited setters. Forty-nine
additional upstream Object files cover key reflection, primitive wrapping,
symbol exclusion, intrinsic arrays, and metadata. Eight more upstream files cover
getter additions, deletions, enumerability changes, and abrupt reads.
Ordinary method/getter/setter literals now parse with source retention, arity,
strictness, unique-parameter, scope, and depth checks. Their non-constructible
closures capture the current environment and trace their home objects; names,
lengths, receiver binding, arguments, and exact source text follow ordinary method
semantics. Accessor definitions merge pairs and replace data descriptors in source
order. Twenty-three upstream files cover computed names, escaped/reserved method
names, abrupt key evaluation, and setter scope; four additional originals cover
super properties and forbidden calls. Parameter patterns and super
property references now use the shared function and reference semantics.
Three super parser regressions and two inspected insta snapshots cover names,
assignment/update/destructuring/loop targets, method/arrow context boundaries,
strict computed expressions, optional calls, tags, constructors, and depth guards.
Twelve runtime regressions cover home-prototype lookup, receiver-preserving reads,
writes/calls/accessors, detached/bound methods, strict primitive receivers,
deferred computed-key conversion, base retention across mutation, null bases,
deletion without coercion, logical/update/destructuring/loop assignments,
arrow/default/eval inheritance, and collection of captured environments.
Five additional recursive super call/get/set/key/eval controls run on a two-MiB
native stack. Twenty-seven unchanged pinned originals add 50 variants: 46
harness positives and four reviewed parse negatives. Twenty-two super-expression
programs cover dot/computed references, receivers, strictness, null bases, abrupt
keys, arrows/eval, internal prototype lookup, and four modern key-conversion order
controls. Four object-method originals cover body/default super properties and
forbidden SuperCall tokens; another direct-eval original checks live home
prototypes. Seventy-two class programs (one also cross-realm) and eight
async/generator method programs remain excluded as whole files without credit.
Both stable and MSRV pass all 50 variants with ordinary unlimited defaults.
Async/generator methods and derived-constructor super calls remain
unsupported.
Object.fromEntries creates fresh ordinary objects from iterables, requires object
entries, reads 0 then 1 before key conversion, preserves Symbol keys, and defines
own writable/enumerable/configurable data properties. Duplicate keys overwrite in
place; inherited setters and entry iterator hooks are bypassed. Entry failures
close the iterator with incoming-throw precedence; step/host failures do not close.
Tests cover evaluation order, boxed Strings, inherited entry fields, mutation,
metadata, collection, and opt-in quotas.
Object.groupBy validates callbacks before iterator lookup, groups original values
by converted string/Symbol keys, and preserves ordered callback/coercion effects.
It creates a null-prototype result with dense intrinsic Arrays only after iterator
exhaustion. Callback/key failures close with original-throw precedence; step/host
failures do not. Tests cover receivers, live mutations, sparse inputs, Strings,
Symbols, collection, the safe-integer counter boundary, and opt-in host failures.
All required Object constructor properties are materialized; full own-key
reflection, copying, and freeze/seal operations on it now execute.

String calls/new, StringData wrappers, and branded toString/valueOf are supported.
Length and indices use UTF-16 code units, including lone surrogates. Wrapper index
properties are read-only, enumerable, and non-configurable; length is also
non-enumerable. String receivers box for non-strict calls, while strict accessors
retain primitive receivers. Own primitive indices take precedence over inherited
accessors. Wrappers currently materialize indexed descriptors, so length plus one
consumes the per-object property capacity; primitive index reads do not allocate.
fromCharCode/fromCodePoint construct UTF-16 with ordered numeric conversions;
at/charAt/charCodeAt/codePointAt apply generic receiver conversion and distinguish
relative indexing, out-of-range results, and surrogate pairs. isWellFormed and
toWellFormed detect unpaired surrogates and replace each with U+FFFD while retaining
valid pairs and all other code units. concat converts each argument in order;
slice and substring extract UTF-16 code units with their respective relative-index
and endpoint-swapping rules. trim/trimStart/trimEnd use ECMAScript WhiteSpace and
LineTerminator definitions, retaining other code units and internal whitespace.
repeat/padStart/padEnd preserve code-unit sequences, conversion order, empty-string
short circuits, and standard invalid-count RangeErrors. Their generated output
has checked length/capacity and bounded copying. indexOf/lastIndexOf search UTF-16
units with ordered conversions, clamped positions, and the distinct NaN defaults;
each candidate comparison is charged to the work budget. includes shares this
bounded search; startsWith/endsWith compare the selected code-unit range once.
These predicates perform IsRegExp's Symbol.match lookup between receiver and
search conversion, rejecting truthy markers without invoking them. Native-injected
symbols test lookup order, abrupt completion, and bounded recursion. The actual
RegExpMatcher brand fallback checks each object's own native slots. String.raw processes
ordinary array-like templates with ordered raw/length/index reads and interleaved
substitution conversion. All baseline String constructor static properties are
installed, and its own-key enumeration is supported. The baseline String prototype
property inventory is complete; ordered reflection and integrity operations are
supported. Full native RegExp matching remains open.
String.split uses non-overlapping UTF-16 separator matches, preserves empty parts,
and splits code units for an empty separator. Edition 17 looks up Symbol.split
only for object separators, passing the original receiver/limit and returning the
hook result unchanged. Fallback conversion reads the receiver, limit, and separator
in that order; separator conversion still runs for zero limits. Results are dense
intrinsic Arrays with own data elements. Tests cover surrogate halves, generic
receivers, conversion failures, uint32 wrapping, metadata, collection, and opt-in
host failures. Generic RegExp splitting supports custom species and exec; native RegExp matching
remains pending.
String.replace delegates object Symbol.replace hooks before conversion and skips
primitive prototype hooks. Ordinary searches convert receiver/search/replacement
in order, replace the first UTF-16 match, and preserve unchanged Strings on misses.
Functional replacements receive match/position/full String with undefined this;
their converted return text is literal. String replacements expand $$, $&, prefix,
and suffix tokens; capture tokens remain literal without RegExp captures. Tests
cover hook and conversion ordering, callbacks, surrogate boundaries, substitution,
metadata, collection, large default outputs, and opt-in host failures.

String.replaceAll collects all non-overlapping positions before callbacks and
handles empty searches at every UTF-16 boundary. Each match uses the original
String for callback arguments and prefix/suffix substitutions. Object searches
perform IsRegExp and the required global-flag checks before Symbol.replace lookup;
primitive hooks are ignored. Regressions cover flags and hook ordering, fixed
converted inputs, callback result conversion and failure, literal dollar patterns,
metadata, collection, large default outputs, and opt-in host failures. Actual
Native matching and capture production remain pending.

Sparse Array storage now enforces indexed length growth, read-only length, and
descending partial truncation behind the low-level Objects API. Numeric length
descriptors must be preconverted there; work checks precede any truncation.
Realm definitions now perform both ArraySetLength conversions, and assignments
defer them until after receiver/writability checks. Array calls/new and Array.isArray
are implemented, including Array.prototype identity, sparse numeric construction,
and non-coercing single-element construction. Array literals preserve holes,
trailing commas, and element evaluation order, using own data definitions and
the intrinsic prototype independently of the global binding. Join and toString
support ordered generic conversion and intrinsic fallback; recursive conversion
remains bounded by host limits. Generic at snapshots length before index
conversion, supports relative indexing, and performs live property reads. Push/pop
perform ordered Set/DeletePropertyOrThrow operations and retain partial effects
when later writes or deletions fail. ForEach/every/some snapshot length, validate
callability, and visit live own or inherited elements in index order, skipping
holes. Every/some short-circuit on ToBoolean results. Find/findIndex and their
Last variants instead visit every index, including holes, in the specified
direction and preserve the value read before each predicate call. Includes uses
SameValueZero and reads holes as undefined; indexOf/lastIndexOf skip absent
properties and use strict equality. Searches preserve the initial length,
coerce fromIndex only for nonempty ranges, and read elements live. Reduce/reduceRight
use a supplied initial value or the first present element, skip holes, and pass
the previous accumulator, current value, index, and receiver to each callback.
Reverse preserves holes, observes getters before strict writes/deletions, and
keeps partial effects on failure without assigning length. Fill/copyWithin
convert relative indices in order, including for empty ranges; fill uses strict
writes, while copyWithin handles overlap and deletes targets for source holes.
Shift/unshift move sparse elements in the specified direction before a final
strict length assignment, including on empty or zero-argument paths.
Sort/toSorted validate comparators before reading length, collect indexed values
before comparison, and use stable bounded merge sorting. Custom comparators use
undefined this and numeric return coercion; default comparisons use ordered
string conversion and UTF-16 order. Undefined sorts last without comparator
calls. Sort skips holes, strictly writes values, then deletes the remaining
range; toSorted reads through holes into a dense intrinsic array. Abrupt
comparisons stop before writeback, and later property failures retain prior
effects. Comparison order for inconsistent comparators is implementation-defined.
Array toLocaleString uses the ECMA-262 algorithm without ECMA-402, a fixed comma
separator, live element reads, original receivers, and zero-argument method
calls. It ignores the reserved arguments, converts each result before the next
read, and bounds output and recursion. BigInt elements use their exposed branded
locale fallback.
Array.of constructs through constructor receivers or creates an intrinsic Array
for non-constructors. It defines own data elements before a final strict length
assignment, including on empty results, and retains partial effects on failure.
Array.from accepts iterable and array-like sources, uses ordered zero/one-argument
construction without species, and maps with exactly value/index and the supplied
receiver. Holes become own undefined elements; array-like length is snapshotted,
while iterator state remains live. Iterator methods preserve original receivers;
next is cached once, and done is read before value. Mapping/definition failures
close the iterator and preserve the original throw over cleanup exceptions.
Step failures and final length failures do not close. Host failures remain distinct
and do not trigger JavaScript cleanup. Regressions also cover custom result identity,
source aliasing, partial effects, astral/lone-surrogate strings, and retained intrinsics.
Array literal spread consumes each iterable before evaluating later elements,
preserves original iterator receivers, and caches next. Yielded undefined values
become own data properties while literal elisions remain holes. Iterator-step
failures propagate without closing. Output always uses the intrinsic Array and
bypasses inherited element setters. Elisions assign length immediately; the final
length write follows all elements. Native boundary regressions verify full
iterator exhaustion before a failing Array length write and Number-rounded keys.
ToSpliced preserves omitted versus undefined arguments, skips discarded getters,
and checks both safe-integer and Array length bounds before copying. It reads
retained elements in ascending order and supports full-width source indices
when deletion shrinks the output to a valid Array length.
ToReversed/with create intrinsic arrays with dense own elements, skip constructor
lookup, and preserve ordered live reads. With never reads its replaced index.
Map/filter validate callbacks before ArraySpeciesCreate, which consults constructor
and Symbol.species only for genuine Arrays. Null/undefined species select an
intrinsic Array; custom constructors receive one Number length (source length
for map, zero for filter). Map preserves absent indices; filter packs the values
read before each callback. Both use live HasProperty/Get, define own data
properties without invoking setters, retain partial results on abrupt completion,
and do not assign a final length to custom results. Checked heaps cannot expose
foreign-realm constructors; cross-realm normalization and Proxy traversal remain
pending with those object kinds. Sparse scans and value copies charge host work.
Slice snapshots length before ordered start/end conversion, then creates its
species result and copies present properties with live reads. Holes preserve
any existing custom-result properties. A final strict length assignment runs
even for empty results, observing inherited setters and read-only properties.
Generic sources retain the full safe-integer index range; ArrayCreate rejects
oversized results before element reads. Failed copies retain earlier definitions.
Concat constructs its species result before reading spreadability or input lengths.
Symbol.isConcatSpreadable overrides the Array brand for objects; primitive arguments
remain single elements. Each spread input snapshots length and visits properties
live, preserving holes and inherited values. Definitions bypass setters and keep
partial results. Safe-integer overflow throws TypeError before indexed reads;
the final strict length write follows traversal. Large sparse scans consume an
opted-in work budget, and custom results can alias inputs without hiding subsequent mutations.
The pinned 4,000-hole sparse-object concat fixture runs unchanged in both Script
modes under the default runtime configuration, with host resource quotas disabled.
Flat/flatMap use an explicit traversal stack, snapshot each entered Array's length,
and read presence and values live. Only branded Arrays flatten; holes compact at
each flattened level. Depth conversion follows source length conversion and precedes
species construction. FlatMap validates its callback before species lookup and maps
only original-source elements. Custom results receive own data properties without a
final length assignment; failed definitions retain earlier effects. Regressions cover
safe-integer index overflow, species aliasing, and 4,000 nested Arrays on a 2 MiB
native stack. Cyclic infinite flattening can be stopped with an opted-in work quota.
Splice preserves omitted versus explicit undefined deletion counts, converts ranges
before species construction, and copies present deleted elements with own data
definitions. It strictly assigns the result length before moving source indices,
including when species aliases the source. Shrinking copies forward and deletes
obsolete indices backward; growing copies backward. Source moves and insertions
use strict Set, delete targets for source holes, and retain partial effects on
failure. A final strict source length assignment runs even with no arguments.
Full-width safe-integer bounds precede species and indexed reads. Every Array
prototype method is now materialized, enabling full key enumeration, descriptor
reflection, and integrity operations on that prototype.
Array's Symbol.species getter and Array.prototype's Symbol.unscopables table have
their standard attributes. The getter preserves its receiver; the table has a
null prototype and all 16 specified entries. With environments remain pending.
Array keys/values/entries create branded iterators that read live lengths and
advance their index before reading values. Keys skip element reads; entries
create intrinsic two-element arrays. Length errors retry the current index,
element errors retain the increment, and exhaustion permanently releases the
source. Reentrant getters preserve these ordered state changes. Array and
arguments Symbol.iterator properties alias the original values callable.
String iteration captures ToString's result synchronously at creation, then
yields complete surrogate pairs or individual unpaired UTF-16 units. Each next
copies at most two units, checks the String iterator brand, and creates a fresh
result. Completion releases the captured text. The original receiver is not
retained. Iterator prototypes supply the identity method and Array/String
Iterator tags. The shared Iterator tag getter is generic. Its setter rejects
primitive receivers and the intrinsic prototype, creates an own data property
when absent, and strictly updates existing own properties without changing their
attributes. The Iterator constructor has standard metadata and an immutable
prototype property; ordinary calls and construction with itself as newTarget
throw TypeError. Distinct newTargets select an object-valued prototype or the
intrinsic fallback without executing their bodies or coercing ignored arguments.
Regressions cover bound forwarding, ordered prototype access and errors, generic
constructor getters, protected setter updates and inherited-property bypass,
global replacement, and intrinsic retention. The complete edition-17 shared
prototype supports ordered own-key reflection, copying, enumeration, and integrity
operations without invoking constructor or tag getters. Regressions cover every
method descriptor, accessor identity/protection, Symbol order, inherited iterator
chains, deleted-link retention, and post-baseline names remaining absent.
Symbol-keyed access is tested through native injection, Script integration tests,
and reviewed upstream Symbol/iterator fixtures.

Iterator.from supports object iterables, direct iterators, and String primitives,
preserving existing intrinsic instances and wrapping other iterators. Regressions
cover primitive rejection before hooks, original hook receivers, ordered
acquisition, captured next methods, live return lookups, exact result/receiver
forwarding without arguments, no implicit completion state, reentrant calls,
retry after throws, internal-slot validation, and cached/source retention during
collection. Wrapper prototype reflection and integrity cover its complete two
method inventory and the shared prototype's complete edition-17 properties.

Iterator.concat captures object iterable methods in order, then opens and steps
sources lazily. Native helper resumes cover fresh results, done-before-value,
permanent completion, closing only the active yielded source, strict close-result
validation, empty-source traversal, and reentry rejection. Regressions cover
live source mutation, captured methods/next, receiver/argument counts, closing
precedence and failures, large default inputs, host aborts without cleanup,
internal-slot brands, and capture retention/release during collection. The helper
prototype's next/return/tag and Iterator's static inventory support complete own
reflection and integrity operations. Language generators remain unimplemented.

Iterator.prototype.toArray consumes a generic object receiver through its cached
next and materializes a fresh intrinsic dense Array after exhaustion. Regressions
cover native/wrapped/helper iterators, ignored iterator hooks and extra arguments,
primitive rejection before next lookup, done-before-value, exact receivers and
zero arguments, error identity without closing, element identity/undefined,
species and inherited-setter bypass, metadata, deleted-link retention, large
default results, and opt-in host failures.

Iterator.prototype.forEach validates the callback before reading next and closes
on invalid callbacks or callback language throws. Direct stepping errors never
close. Regressions cover ordered acquisition, cached next, live callback mutation,
undefined callback receivers, exact value/index arguments, incoming-error
precedence, host aborts without cleanup, native/wrapped/helper iteration, metadata,
collection, and large default inputs. Internal tests verify correctly rounded
indices beyond 2^53 and u64 using exact counters, independently of a BigInt-value
magnitude quota, and preserve work-failure behavior.

Iterator.prototype.every/some/find consume direct iterators with exact mathematical
indices and short-circuit closing. Regressions cover callback validation before
next lookup, cached next and live return, strict/sloppy callback receivers,
original found-value identity across mutation, all primitive truthiness categories,
object results without conversion hooks, normal-close error precedence, callback
throw preservation, exhaustion and step errors without cleanup, native/helper
iteration, metadata/collection, large default inputs, and opt-in host aborts.
Internal tests verify indices beyond u64 under a zero BigInt-value magnitude quota.

Iterator.prototype.reduce distinguishes an omitted initial value from a present
undefined, retains arbitrary accumulator identities, and uses exact mathematical
callback indices. Regressions cover empty/singleton inputs, first-value selection,
callback arguments/receivers, direct acquisition and cached next, live mutation,
primitive/invalid reducer validation, incoming throw precedence, initial/later
step errors without closing, exhaustion, ignored extra arguments, metadata and
collection, native/helper iteration, large default inputs, and opt-in host aborts.

Lazy Iterator.prototype.map/filter capture direct next once and defer source steps
and callbacks until next. Regressions cover validation/closing order, lazy cached
traversal, strict/sloppy callback receivers, exact indices including skipped filter
values, arbitrary mapper results, truthiness without coercion, original filtered
value identity, early/yielded return, completion-before-cleanup reentry, normal and
incoming-throw closing precedence, permanent completion, step errors without
cleanup, shared helper branding, chained helpers, capture tracing/release,
metadata/collection, large default pipelines, and opt-in host aborts. Internal
tests verify indices beyond u64 under a zero BigInt-value quota and reject foreign
or noncallable captures before allocating native helper objects.

Lazy Iterator.prototype.flatMap regressions cover direct outer capture, delayed
mapper/inner acquisition, cached inner next, iterable/direct fallback and boxed
Strings, primitive rejection before hooks, one-level identity preservation, empty
inners and exact outer indices, mapper/acquisition/inner-step closing, outer-step
failures without closing, done-before-value, early completion before cleanup,
inner-before-outer return and error precedence, reentry, metadata/brands, traced
capture retention/release, large default inputs, and host aborts without cleanup.
Internal traversal verifies indices beyond u64 with a zero BigInt-value quota.

Lazy Iterator.prototype.take/drop validate their object receivers, convert counts
before capturing next, and preserve exact finite countdowns. Regressions cover
conversion/validation closing precedence, signed zero and fractional truncation,
positive infinity and counts beyond 2^53/u64, cached next/receivers/argument counts,
discarded-value getter bypass, take's zero/count-exhaustion closing, observed done
without cleanup, early/yielded return, reentry, step/host failures, native pipelines,
metadata/collection, capture tracing/release, large default inputs, and zero
BigInt-value quotas. Internal tests verify exact decrements beyond Number precision
and u64, work-failure atomicity, and foreign-capture rejection before allocation.
At the existing pin, some upstream take/drop tests enforce a post-edition-17
safe-integer cap; those originals remain excluded, without rewriting or pass credit.

Optional-chain regressions cover nullable/falsy primitives, skipped keys and
arguments throughout ungrouped suffixes, grouping boundaries, exact method/primitive
receivers, once-only getters and key conversion, late callable checks, abrupt
arguments, final-reference deletion without getters, strict frozen-property errors,
identifier/TDZ failures, spread/newTarget/grouped construction, large default flat
chains, syntax retention through collection, and opt-in host aborts. Parser AST
and diagnostic snapshots cover boundaries, early errors, and strict syntax.

Tagged template regressions cover raw/cooked UTF-16, invalid cooked escapes,
member/call/new precedence, chained tags, exact receivers and unconverted argument
identity/order, tag getters and substitution errors, late callable checks, frozen
Array/raw descriptors, constructor/species/setter bypass, repeated parse-site
identity across functions and syntax clones, separate parses/realms, cache tracing
through collection, and opt-in host failures without partial cached templates.
Parser AST/diagnostic snapshots cover syntax, strict errors, and invalid targets.
Proper tail calls and JavaScript cross-realm hooks remain separate work.

Global encodeURI/encodeURIComponent use string-hint ToString once, preserve the
specified ASCII escape sets, and emit uppercase UTF-8 percent escapes without
Unicode normalization. Unpaired UTF-16 surrogates throw URIError, including through
catch/finally and the host exception-value API, using the retained intrinsic
prototype even after global replacement. Regressions cover Unicode/UTF-8 boundaries,
coercion and abrupt order, ignored receivers/extra arguments, descriptors,
non-construction, deletion/collection, large default output, and opt-in host aborts.
Global decodeURI/decodeURIComponent validate ASCII percent triplets and strict
UTF-8, preserving literal UTF-16 code units and lone surrogates. decodeURI retains
reserved escapes with their original spelling; decodeURIComponent decodes them.
Regressions cover UTF-8 boundaries, malformed/overlong/surrogate/out-of-range
sequences, plus signs and non-recursive percent handling, normalization avoidance,
once-only coercion and abrupt results, intrinsic URIError identity, metadata,
non-construction, deletion/collection, large default inputs, and opt-in host aborts.

String case conversion uses full pinned Unicode 18 mappings, including expansions
and context-sensitive final sigma against original text. Lone UTF-16 surrogates
remain unchanged and Unicode normalization is not performed. Locale casing uses
the documented fixed locale-neutral host fallback and ignores reserved arguments.
Regressions cover combining and overlapping Cased/Case_Ignorable properties,
supplementary mappings, Unicode 18 additions, generic coercion/abrupt results,
metadata, intrinsic retention, large default output, and opted-in host failures.
Generated tables are checked against digest-pinned UCD sources independently of Rust.

String.normalize supports all four Unicode 18 forms, recursive canonical and
compatibility mappings, stable combining-class ordering, excluded compositions,
and algorithmic Hangul. It preserves lone surrogates and orders receiver/form
conversion and invalid-form errors. Native regressions cover canonical blocking,
leading marks, generic coercion, intrinsic RangeError retention, descriptors,
collection, large default inputs, and opted-in host aborts. The complete unchanged
Unicode normalization suite supplies 20,171 independent vectors, with all twenty
column invariants per vector and identity checks on every other Unicode scalar.
Intermediate decomposition does not consume the final output string quota.

String.localeCompare uses the documented locale-neutral NFD UTF-16 ordering.
It honours canonical equivalence with positive zero and preserves compatibility
distinctions and lone surrogates. Regressions cover the specification examples,
supplementary decompositions, antisymmetry/transitivity, ordered generic conversion,
reserved argument evaluation, public normalization hooks, metadata, collection,
large default inputs, and opted-in host aborts.

String.match/matchAll/search support custom object Symbol hooks with original
receivers, exact arguments, and unchanged results. MatchAll checks IsRegExp and
global flags before hook lookup. Regressions cover coercion/getter order, abrupt
identity, non-callable hooks, ignored primitive hooks, function metadata, collection,
and host aborts. The direct RegExpCreate fallback now converts the receiver before
allocating and initializing its pattern through the intrinsic, bypassing constructor
identity and native-copy rules. It invokes the newly created object's live Symbol
method with the converted String; matchAll supplies g and creates a lazy iterator.
Thirteen integration tests cover fresh native objects, property/conversion order,
exact receivers and arguments, syntax and method errors, ignored primitive hooks,
lazy custom exec, collection, a 120,000-unit retained input and native stack guards.
Native matching remains Unsupported; no substring approximation is counted as
matching. The complete String property
inventory supports ordered reflection, copying, enumeration, and integrity operations.

A separate scoped review reads all 25 String.matchAll programs and thirteen
match/search fallback programs at the same pin. Six originals already belong to
the corpus. Four unchanged whole programs add eight passing normal/strict
variants on both compilers for direct creation, native source/flags/lastIndex,
live Symbol invocation, exact results and converted receivers. The other 28
whole programs require literal execution or native matching, including primitive
hook programs that subsequently perform matching. They remain excluded without
rewriting or pass credit. Match/search directories are not claimed as completely
reviewed. Older descriptions retain edition-17-compatible assertions. The pin,
ten helpers, original bytes and unlimited defaults are unchanged; no native
matching coverage is claimed.

The standalone ECMA-404 JSON parser preserves UTF-16 strings, duplicate names,
source lexemes, and correctly rounded Numbers. Insta snapshots cover flat trees
and strict grammar diagnostics. Regressions check every raw noncontrol UTF-16 unit,
JSON escapes, decimal boundary rounding, and 20,000 nested arrays/objects with
iterative parsing and dropping. Host work aborts remain distinct from syntax errors;
no default work or nesting quota is introduced. JSON.parse now creates intrinsic
objects and dense arrays, preserves own data descriptors and duplicate-key order,
and treats __proto__ as an ordinary key. Runtime regressions cover ordered text
conversion, ignored noncallable revivers, retained prototypes and SyntaxError
identity, inherited setter bypass, function metadata, 10,000 nested arrays with
ordinary defaults, and opted-in work/heap/final string aborts. Reviver regressions
cover postorder numeric/string key ordering, root holders and bound callbacks,
exact primitive source lexemes and descriptors, duplicate-key sources, SameValue
checks after mutation, replaced containers, snapshot keys/length, inherited values,
setters and rejected updates, abrupt identity, nested parsing, and 10,000-level
traversal. Cyclic forward mutations abort with an explicitly opted-in work quota.
Raw JSON regressions cover primitive validation and boundary code units, exact
lexemes including large integers and lone surrogates, single ordered conversion,
intrinsic SyntaxError retention, frozen/null-prototype descriptors, mutation
rejection, getter-free branding, inheritance/copying, metadata, collection,
100,000-character default text, and opted-in work aborts. Serialization regressions
cover omissions/null substitution, property order and live reads, toJSON/replacer
ordering and holders, wrapper conversion, replacer lists, indentation, BigInt and
raw embedding, aliases/cycles, abrupt identity, nested serialization, complete
reflection, collection, 10,000-level default nesting, and opt-in output/work aborts.
Every UTF-16 unit roundtrips through quoting and the independent strict parser;
exact assertions verify lowercase escapes and surrogate pairing. Handle hashing
tests distinguish heap owners and reused generations while deduplicating clones.

Native String RegExp fallbacks,
Array.fromAsync,
regular expressions, for-await-of, generators,
async functions, promises, modules, standard library objects, agents, shared
memory, and automatic garbage collection remain open. See the roadmap for their order.
The BigInt constructor converts with the number hint, accepts finite integral
Numbers exactly, and accepts Boolean/BigInt/integer-string values through ToBigInt.
It rejects construction before coercion. Invalid integer strings throw SyntaxError;
nonintegral/nonfinite Numbers throw RangeError; incompatible types throw TypeError.
The arithmetic crate decodes binary64 significands/exponents exactly; regressions
cover every integral exponent, both signs, zero, and opted-in quotas.
BigInt wrappers have a retained internal value and the intrinsic prototype;
non-strict calls box them freshly. BigInt.prototype itself has no BigIntData.
ValueOf/toString/toLocaleString validate brands without coercing receivers.
ToString validates the receiver before radix conversion and emits exact lowercase
radix digits. ToLocaleString uses the specified non-ECMA-402 decimal fallback and
ignores reserved arguments. Object.prototype.toString reads the observable tag
through ToObject/Get; deleting it exposes the ordinary Object fallback. Wrapper
brands survive prototype changes and explicit collection. BigInt.asIntN/asUintN
complete ToIndex before ToBigInt, reject Number values, and reduce modulo powers
of two with exact signed/unsigned results. Fitting values accept even the maximum
safe-integer width without width-sized storage. Constructor enumeration and
integrity operations now cover all standard own properties. Host display stays exact hexadecimal
with an `n` suffix; JavaScript string conversion is decimal.

Global lexical bindings and Script var declarations persist between evaluations.
New global vars are non-deletable. A var declaration for an existing global value
preserves its descriptor and value. Global identifiers and globalThis properties
share storage; lexical bindings stay separate. Declaration checks use actual own
property attributes and extensibility. Global accessors and inherited properties
retain correct receivers, and strict assignments recheck bindings deleted by RHS
evaluation. Replacing/deleting globalThis does not change the realm's this identity.
The process-wide Symbol registry has no default identity or text quota and never
evicts entries. Opted-in work/output quotas and platform capacity failures are
host limits and preserve prior registrations.
Realm initialization creates 309 retained entries outside per-Script work
accounting; opted-in allocation/property quotas still apply.
Built-in error categories are represented in Rust. Catch clauses without a parameter handle language throws and built-in
exceptions. Catch binding identifiers now parse with scope and strict-mode early
errors, including the required non-browser rejection of conflicting var names.
Supported throws initialize a mutable catch binding without coercion, preserving object identity. The catch
binding and block scopes are restored across every completion and host failure.
Built-in exceptions materialize as Error objects for catch bindings, using
intrinsic prototypes even after global constructor bindings are replaced. Rethrows
preserve object identity. Allocation/string limits during materialization remain
host aborts and skip pending handlers and finalizers. Uncaught built-in failures
retain their host-facing Rust category and source span.
Catch patterns use the shared destructuring binding implementation described above.

Recognized missing features return Unsupported. Because the grammar is incomplete,
a syntax diagnostic alone does not prove arbitrary input violates ECMA-262. The
current parser must not be used to score general Test262 negative tests.

The Test262 regression fixtures use a reviewed manifest and match the specific
rejection point for each negative fixture. Identifier component tests compare
literal and escaped spellings at the lexer boundary. Statement parser fixtures
check contextual `let` lookahead and ASI. Neither component group is counted as
Script execution passes. The suite does not parse arbitrary Test262 YAML, report a
whole-suite pass rate, or cover all implemented semantics. The `spite-test262` crate reads a documented metadata
subset, plans strict/non-strict/module/raw modes, and preserves harness include
order. Its Script runner uses fresh realms, separates harness failures from test
results, and matches negative tests by phase and error type. Parse-negative passes
require a reviewed diagnostic range and message; unreviewed syntax errors remain
unverified. Unsupported features, missing host helpers, and resource limits are
separate non-passing results. The unchanged pinned assert.js/sta.js harness now
executes for nine positive function/capture tests, 40 call/construction iterable-spread
tests, 30 call/construction object-spread tests, 133 Reflect call, construction, prototype, extensibility, descriptor, and property tests, 325 Math numeric/metadata tests, 154 Iterator constructor/acquisition/sequencing/consumption/reflection tests, twelve Boolean tests, 63
Number tests, ten numeric parsing tests, five global numeric predicate tests, 58 URI encoding and 107 URI decoding tests, and
43 Error and AggregateError construction, conversion, and prototype tests, 48 BigInt constructor,
width reduction, formatting, and receiver-brand tests, plus 173 Object descriptor,
prototype, extensibility, creation, copying, key enumeration, integrity, and SameValue tests, and
430 String wrapper, raw construction, character, search, sequence, trimming, repetition, padding, Unicode
well-formedness, conversion, and String iteration tests in both Script modes. Another 713 Array and Array iterator
files cover construction, of, isArray, literal elisions and spread (including
fifteen nested object-spread files), length/index boundaries,
truncation, generic at/join/push/pop/shift/unshift/reverse/fill/copyWithin and includes/indexOf/lastIndexOf, toString/toLocaleString, find/findIndex/findLast/findLastIndex, and ordered forEach/every/some
and reduce/reduceRight callbacks, species-aware map/filter/slice/concat/flat/flatMap/splice, plus sort/toSorted, toReversed/with/toSpliced copies,
and keys/values/entries iteration, including mapped/unmapped arguments. The 34
Symbol files and 23 object method/accessor files run in their prescribed Script modes.
Nineteen unchanged tagged-template sources contribute thirty-four prescribed
variants covering site caching, raw/cooked text and invalid escapes, unconverted
arguments/receivers, chains and constructor precedence, freezing, and property
attributes. Originals requiring eval, dynamic Function construction, cross-realm
hooks, or proper tail calls remain excluded, without rewriting or pass credit.
Twenty-seven unchanged optional-chaining sources add fifty-four variants: sixteen
positives cover lazy nullish checks, receivers, ungrouped suffixes, grammar,
decimal lookahead, new.target calls, and iteration. Eleven reviewed negatives
reject template tags (including newline forms) and write/update targets at their
intended byte ranges. Originals requiring classes/super, async/promises, RegExp,
or eval remain excluded, without rewriting or pass credit.
One hundred seven unchanged URI decoding sources add 214 variants covering strict
escaped UTF-8, exhaustive literal UTF-16 and supplementary scalar loops, malformed
percent/Unicode sequences, reserved escape handling, conversions and abrupt
results, metadata, and non-construction. Four global enumeration/descriptor sources
remain excluded pending complete global own-key reflection. They receive no credit.
The original pin, helper bytes, and runtime defaults are unchanged.

Fifty-eight unchanged URI encoding sources add 116 variants covering
surrogate validation, escape sets, URLs/Unicode/control text, object conversion,
metadata, and non-construction. Their pinned decimal hexadecimal helper brings
the harness inventory to seven files. Twelve additional sources now execute
their case-conversion paths unchanged. Four global enumeration/descriptor sources
remain deferred without rewriting or pass credit.
One hundred two unchanged String case-conversion sources add 204 variants across
all four methods, covering full/contextual mappings, generic and abrupt conversion,
unnormalized Unicode text, and metadata. Eight eval/RegExp sources remain excluded
and receive no credit. The existing pin, helper bytes, and runtime defaults remain
unchanged.
Twenty-one unchanged AggregateError sources add 42 variants for constructor,
prototype, ordered arguments/iterators, abrupt results, and errors Array descriptors.
The complete original promiseHelper.js brings the harness inventory to eight; the
selected order test calls its synchronous sequence helper, without Promise support.
Four Proxy/dynamic Function/cross-realm/global reflection sources remain excluded
and receive no credit. The pin and original bytes are unchanged.
Fourteen unchanged String normalization sources add 28 variants for form selection,
ordered generic conversion, errors, and descriptors, including full prototype
reflection through the original helper. The independent Unicode oracle verifies
all normalization column and scalar-identity invariants.
The pin, original bytes, helpers, and unlimited defaults are unchanged.
Thirteen unchanged String locale comparison sources add 26 variants for canonical
equivalence, generic receivers, omitted arguments, and function metadata. Native
regressions additionally verify positive zero and consistent total ordering.
The pin, original bytes, helpers, and unlimited defaults are unchanged.
Fifteen unchanged String matching-hook sources add 30 variants for object hook
lookup/invocation, abrupt results, metadata, and non-construction. Native RegExp
fallback/literal sources remain excluded without credit.
The pin, original bytes, helpers, and unlimited defaults are unchanged.
Fifty-two unchanged JSON.parse sources add 104 variants for exact grammar,
whitespace and escapes, text coercion, negative zero, duplicate/__proto__ keys,
metadata, and non-construction. This cohort omits reviver and serialization cases, which are reviewed separately.
Thirteen unchanged JSON reviver sources add 26 variants for callback order,
source contexts, wrappers, inherited values, rejected updates, and abrupt errors.
Originals requiring Proxy or destructuring remain excluded without credit.
Twelve unchanged raw JSON sources add 24 variants for primitive validation,
null-prototype objects, slot branding, metadata, and non-construction. Serialization
and full-inventory descriptor cases are reviewed separately; destructuring remains open.
Sixty-three unchanged JSON serialization and reflection sources add 126 variants
for replacers, ordered hooks, live property reads, indentation, omission, cycles,
BigInt errors, raw embedding, surrogate quoting, and complete JSON descriptors.
Proxy, cross-realm, RegExp, destructuring, and global-reflection originals remain
excluded without credit. The reviewed JSON inventory is 140 sources; its existing
pin, exact bytes, helpers, and unlimited defaults are unchanged.
One hundred eighty-seven unchanged Map and Map Iterator sources add 371 variants
for ordered construction/closing, canonical keys, live mutation, computed insertion,
intrinsic grouping, branding, metadata, and full reflection. Three sources specify
a single Script mode. Twelve Set receiver originals are added with the Set cohort
below. Whole files requiring other unavailable receivers or syntax,
cross-realm support, or global reflection remain excluded without credit.
The existing pin, original bytes, helpers, and unlimited defaults are unchanged.
Three hundred fifty-three unchanged Set and Set Iterator sources, plus twelve Map
receiver-brand originals using Set instances, add 728 variants. Two Set forEach
sources prescribe a single Script mode. The cohort covers canonical identities,
construction/closing, live callbacks/cursors, all seven set-like algorithms,
observable conversions/getters, mutation and duplicate handling, intrinsic results,
branding, species, metadata, and reflection. Forty-one reviewed Set originals need
generators, weak/typed collections, WeakRef, cross-realm support, or global
reflection and remain excluded without credit, including dormant generator syntax.
The original pin, exact bytes, eight helpers, and unlimited defaults are unchanged.
The class/WeakSet revisit contributes 29 originals and 58 variants for WeakSet
receiver brands, class set-like acquisition order, overridden subclass methods,
ignored species, and intrinsic result construction. Its 48 whole-program Set/Map
reviews exclude 17 generator programs and two mixed value/key programs requiring
typed arrays, WeakMap, or WeakRef. No Map original is added in this revisit.
One hundred ninety unchanged Function sources and one Map computed-callback
source add 297 variants for ordered conversion, independent grammar goals,
combined early errors, global scope, calls/construction, newTarget prototypes,
parameters/arguments, metadata, and exact source retention. Eighty-five sources
prescribe a single Script mode. The 108 other reviewed Function originals require
eval, optional non-strict caller extensions, unavailable syntax or objects, foreign
realms, global reflection, or the unchanged RegExp-based nativeFunctionMatcher.js
helper. Whole originals remain excluded without execution credit. The existing
pin, exact bytes, eight helpers, and unlimited defaults are unchanged.
One hundred one unchanged with-statement sources and one formerly deferred strict
Function source add 104 variants: 87 harness positives and 17 reviewed parse
negatives from sixteen originals. Negative reviews identify actual strict-with,
Statement declaration/label, and let-array lookahead failures with exact byte
ranges and diagnostics. Eighty-five with positives cover variable scope, closure
capture, live lookup, boxing, nullish conversion errors, unscopables getters and
truthiness, mutation during resolution, and restored completions. The 80 other
reviewed with originals reference eval (71), Proxy (seven), or typed arrays (two)
and remain excluded as whole files without credit. The pin, original bytes, eight
helpers, and unlimited defaults are unchanged.
Another 53 positive for-of files and 22 reviewed for-of parse-negative files cover
iteration, bindings, header grammar, and closing precedence. Seven rest-parameter
positives and twelve parameter parse negatives cover Arrays, length, argument
aliasing, call/apply, and non-simple parameter syntax. Another 36 for-in positives
and 20 reviewed negatives cover ordered enumeration, mutation, bindings, ASI,
control flow, and header/body early errors. Controls
verify successful assertions and explicit assertion failures. String comparison
failure formatting now uses JSON.stringify and reports ordinary assertion failures;
Array.fromAsync, other includes, async completion, and agents
remain gaps. CI runs the reviewed corpus on Linux and Windows with MSRV and stable
Rust. Its 13494 variants are four raw positives, 12017 positives using the upstream
harness, 1469 reviewed parse negatives, and four runtime negatives. Component fixtures and harness files do
not enter this count; it is not a whole-suite conformance measurement.
The unchanged propertyHelper.js verifies the installed Math functions' name and
length descriptors in 148 variants. Local controls also cover accessors, Symbol
keys, descriptor restoration, captured primordial functions, incorrect attributes,
and unsupported operations that must remain non-passing. All thirty-seven Math
methods now have unchanged name, length, and Math-property descriptor fixtures;
complete Math reflection enables the original helper's enumeration checks.
The runner uses the runtime defaults, with every host resource quota disabled.
Hosts can opt into quotas with `Limits` fields such as `max_steps: Some(units)`
and `max_heap_entries: Some(slots)`.
Runtime-negative tests inspect the thrown object's constructor name through
checked realm property reads, including explicit Errors and rethrows. Inspection
failures remain non-passing; the original exception category cannot mask them.

Number-to-string formatting uses Rust's shortest round-trip decimal conversion
with ECMAScript presentation rules. Primitive numeric operations have boundary
regressions. Exhaustive numerical and cross-platform conformance audits remain
part of the roadmap.

Opted-in resource quotas bound specific work and value sizes. They do not provide a
complete memory budget or an operating-system sandbox. State is not rolled back
after runtime failure. Parsing and early errors occur before any execution.
Host failures abort evaluation without running further JavaScript finalizers.
Language throws and built-in exceptions do run finalizers and can be overridden
by an abrupt finalizer.

## Intentionally outside scope

Node.js and browser APIs, CommonJS, ECMA-402, syntax extensions, and unstandardized
proposals are outside the target. Annex B's optional browser compatibility
extensions are not enabled for this non-browser host. They are standardized
optional behavior, not evidence that Node.js compatibility is required.
The `Legacy` numeric and string productions in edition-17 sections 12.9.3–4 are
required language syntax, distinct from those optional Annex B features.

Map regressions cover all primitive key types, distinct Object/Symbol identities,
NaN payloads, canonical positive zero, wide BigInt keys, unchanged insertion order,
deleted/reinserted entries, frozen instances, prototype/adder/iterator ordering,
entry-closing precedence, step failures, own branding, live and exhausted iterators,
exact callback arguments/receivers, callback mutation/reentry, computed insertion,
intrinsic group materialization, metadata, garbage-collection retention/release,
foreign/stale handle rejection, a 20,000-entry default Map, and opt-in host aborts.
The GroupBy safe-integer guard is checked directly before the next step.

Set regressions cover primitive types and Object/Symbol identities, NaN and positive
zero, wide BigInt values, ordered deletion/reinsertion, frozen instances, cached
adder/iterator order, closing precedence, own brands and distinct iterator brands,
live callbacks/cursors, set-like size conversion and getter order, exact branch
selection, duplicate other keys, snapshot/live mutation rules, normal short-circuit
closing errors, unclosed step failures, intrinsic result allocation, metadata,
garbage-collection retention/release, foreign/stale values, a 20,000-value default
Set, and opted-in aborts. Storage snapshots retain vacant positions without retaining
deleted values and fail before live mutation when their work allowance is exhausted.

Base-class regressions cover explicit/default construction, primitive and object
returns, custom newTarget prototypes, class-call argument/apply ordering, bound
construction, strict receivers and unmapped arguments, non-constructible methods,
exact descriptors, mutable declarations and immutable internal names, computed-name
TDZs and ordering, name inference before static overrides, accessor merging,
Symbol/lone-surrogate keys, exact class/method source, eval/Function compilation,
super properties, abrupt context restoration, collection, checked home-object
handles, and opt-in name limits. Parser AST/diagnostic snapshots cover constructor,
strictness, scope, and declaration errors. Unsupported async/generator methods
retain no conformance credit.
Five additional recursive class construction/method/computed-name/eval cases run
on two-mebibyte native stacks with the existing 32-call, 64-evaluation, and
64-parser nesting guards.

Twenty-three unchanged class definition/name-binding originals add 46 variants:
44 harness positives and two reviewed parse negatives. The cohort covers base
construction, strict code, instance/static method/accessor descriptors and names,
prototype rejection, restricted properties, and immutable internal class bindings.
All 73 originals in the three reviewed definition/name-binding/strict-mode
directories were read as complete programs. Thirty-four whole files remain excluded
without credit: 22 require generator methods and twelve async methods, including
nine async parse negatives and six generator parse negatives. The sixteen heritage
originals are now retained below.
Two retained accessor originals carry historical generator annotations but use
ordinary class accessors only. The retained ordinary duplicate-parameter negative
checks its exact original token and message in both Script modes. Every Git blob,
source, assertion, metadata field, pin, and harness helper is unchanged.

Derived-class regressions cover superclass/prototype validation order, class-name
TDZs in heritage, extends null, primitive/object/undefined return distinctions,
uninitialized this before defaults/arrows/super computed names, repeated and
nested super construction order, constructor-prototype mutation during arguments,
custom newTarget, direct eval, escaped arrows and collection, checked derived
constructor handles, and native Array/Map/Set/Error subclasses. Default forwarding
runs with a poisoned Array iterator and through a hundred-class chain. Parser
AST/diagnostic snapshots check inherited constructor context, method/function
boundaries, heritage precedence, and forbidden new/assignment/optional super calls.
Four additional recursive derived construction/default/eval cases exercise the
existing native-stack guards on two-mebibyte threads with normal debug profiles.
Host-abort controls use unimplemented generators as their source gap.

Array species construction is integrated with class constructors for map, filter,
slice, concat, flat, flatMap, and splice. Regressions cover inherited species and
fresh Array subclasses, constructor arguments/new.target, superclass and derived
private-element order, default forwarding with a poisoned Array iterator, custom
class species returning ordinary objects, and intrinsic fallback for null species.
Returned non-extensible objects still receive private elements before failed
indexed definitions. Abrupt field initialization preserves prior fields and method
brands without visiting source elements; collection retains escaped results and
private captures, then reclaims their cycles.

Thirty-six more unchanged Array originals add 72 harness-positive variants:
two each map/filter/slice, eleven concat, five flat, six flatMap, and eight splice.
They cover species constructor rejection, custom construction/new.target, result
descriptors and configurable read-only replacement, inherited indices, method
metadata/non-construction, non-Array class receivers, spreadable functions, and
splice creation failures before source mutation. Whole-program review of 65
candidates excludes ten foreign-realm, fourteen Proxy, four typed-array, and one
RegExp originals. The concat/flat/flatMap/splice directory inventories are complete
at the pin; the map/filter/slice revisit covers constructor/species candidates.
Both stable and MSRV pass all 72 variants with unchanged bytes, helpers, flags,
pin, and unlimited default quotas.

Sixteen unchanged class heritage/derived originals add 32 variants: 30 harness
positives and two reviewed strict-mode parse negatives. They cover constructor and
instance prototype links, superclass and prototype validation/effects, explicit and
default construction, uninitialized and repeated super calls, name-binding heritage
TDZs, super methods, inherited strict code, and restricted arguments properties.
The original nested with statement is checked at its exact byte range and diagnostic
in both Script modes. The three reviewed class directories now retain 39 of 73
whole sources, producing 78 variants: 74 positives and four parse negatives.
The remaining 34 sources require generator or async methods and receive no credit.
All source bytes, Git blob hashes, assertions, metadata, helpers, and the pin are
unchanged; both toolchains pass the 32 added variants under ordinary defaults.

Public-field regressions cover literal/computed/Symbol/UTF-16 names, undefined
initializers, own data descriptors and inherited setter bypass, key/method ordering
before static initialization, internal class-name initialization and inferred names,
base initialization before parameter defaults, derived initialization after super
binds this, abrupt partial initialization and repeated super calls, constructor
object returns, default forwarding through a hundred classes, and bound/custom
newTarget construction. Initializer environments preserve strict this, super,
undefined new.target, lexical captures, function/class name inference, and direct
eval's arguments restriction through arrows and computed method names. Native
controls check descriptor rejection after initializer effects, collection, invalid
field edges before mutation, host-abort restoration, and unchanged opt-in quotas.
AST/diagnostic snapshots cover field/method lookahead, ASI, initializer boundaries,
forbidden literal names, arguments, and super calls. Four more recursion cases
exercise instance/static/derived/eval initializers on two-mebibyte debug stacks.

Sixty-six unchanged public-field execution/ASI originals add 131 variants: 115
harness positives and sixteen parse negatives. The focused whole-program review
covers 72 declaration/expression originals; the six private-field dependencies
are now vendored in the private-field cohort. This does not claim complete elements-directory coverage.
Selected programs cover descriptor creation, inherited setter bypass, frozen
receivers, definition/initialization order, key conversion and abrupt completion,
repeated fields, instance values, base construction, static this/eval/arrows,
class names, repeated super calls, and ASI. Two decorator-annotated originals use
ordinary newline-separated fields. Two generator-annotated ASI negatives parse
the star as multiplication and reject the following brace; no unsupported method
is credited. All eight negative originals check exact original ranges/messages in
both Script modes. Both toolchains pass every added variant with ordinary defaults;
Git blobs, source bytes, assertions, metadata, helpers, and the pin are unchanged.

Static-block regressions cover source ordering among static fields and blocks,
computed keys/methods before initialization, isolated lexical/var/function scopes,
hoisting, strict constructor this, super receiver preservation, undefined
new.target, internal class-name initialization and outer declaration TDZs, and
construction from a block after all instance fields have been collected. Direct
eval checks strict isolation and the zero-length arguments object; escaped arrows
retain bindings and home objects through collection. Abrupt language/host failures
preserve prior effects and restore caller context. AST/diagnostic snapshots cover
empty blocks, function boundaries, forbidden return/arguments/super calls/await,
lexical conflicts, and reset loop/label targets. Three additional recursive block,
eval, and construction cases run on two-mebibyte stacks with the unchanged guards.

Twenty-four unchanged static-init originals add 48 variants: 28 harness positives
and twenty reviewed parse negatives. All 29 declaration/expression root originals
were read in full; four whole files requiring generator or async features
remain excluded without credit. The private-scope original is now in the private-field
cohort. Execution covers empty blocks, ordered/abrupt
initialization, isolated scopes, class-name capture, this, new.target, super, and
await grammar boundaries. Every negative checks its intended original token and
message in both Script modes. Both toolchains pass all added variants with ordinary
defaults; source bytes, Git blobs, assertions, metadata, helpers, and pin are unchanged.

Private-element parser AST/diagnostic snapshots cover decoded names, private
fields/methods/accessors, optional access and brand-check precedence, forward
references through functions and nested classes, heritage scope, getter/setter
pairs, escaped duplicates, private constructor names, deletion, and super access.
Assignment, update, destructuring, loop, call, construction, and tagged-template
targets retain private references. Async/generator private methods remain
Unsupported and receive no positive execution credit.
Nested private class scopes exercise the unchanged parser guard on two-mebibyte
threads. Dynamic Function and UTF-16 Script goals accept the same private grammar;
host-abort controls retain unimplemented generator sources after this parser step.

Forty-eight unchanged private-element originals add 96 reviewed parse-negative
variants for duplicates, accessor staticness, constructor names, undeclared
computed/heritage references, malformed hashes, and super access. All 52 focused
originals were read completely; four unparenthesized-arrow heritage programs remain
excluded because an earlier grammar rejection would not verify their intended
private-scope error. Every selected negative checks its exact original token and
message in both Script modes. Both toolchains pass every variant under ordinary
defaults; all bytes, Git blobs, assertions, metadata, helpers, and the pin are unchanged.

Private-field regressions cover instance/static values, undefined defaults, own
brands, fresh class identities, inheritance and shadowing, escaped names,
assignments/updates/destructuring, logical assignment, private call receivers,
construction/tags, optional access, reflection isolation, frozen/non-extensible
receivers, and inferred function/class names. Construction checks base parameters,
derived BindThisValue, object returns, skipped super calls, bound/custom newTarget,
and 100 levels of iterative default forwarding. Static tests check field/block
order, all computed keys before initialization, and access before installation.
Duplicate stamping evaluates the initializer before rejecting the duplicate;
abrupt initialization preserves earlier private fields and skips later fields.
Heritage closures retain outer private names and the internal class binding.
Direct eval checks reads/writes/brands, ordinary function capture, shadowing,
preflight undeclared-name errors, and indirect/dynamic Function isolation; its
diagnostic snapshot was inspected. Collection retains private values and escaped
eval closures, then releases cycles and replaced values. Storage tests reject
foreign/stale/non-object handles and opt-in work failures before mutation and
verify that private fields ignore ordinary property capacity. Four recursive
instance/derived/static/eval cases run on two-mebibyte normal-debug stacks with
the unchanged guards. Language and host failures restore caller scope and
strictness.

Fifty-six unchanged private-field originals add 112 harness-positive variants.
The focused review read 65 complete programs, including six earlier public-field
dependencies and one static-block dependency. Two private-method dependencies are
now in the method/accessor cohort; seven whole files remain excluded:
four async programs, one host-specific historical
non-extensibility extension, and two programs whose TypeError assertion invokes
an absent method instead of exercising the intended brand check. Selected sources
cover instance/static access, nested/ordinary/arrow capture, direct eval, optional
chains, name/length metadata, reflection separation, fresh brands, inheritance,
duplicate stamping, missing/primitive receivers, and assignment/destructuring/loop
ordering. No unsupported feature receives credit. Both toolchains pass every
added variant with ordinary defaults; all source bytes, Git blobs, assertions,
metadata, helpers, and the pin remain unchanged.

Private-method/accessor regressions cover shared identities, strict receivers,
non-construction, name/length/exact source, accessor metadata and reversed pairs,
own/fresh brands, inheritance/shadowing, nested class capture, method writes,
missing getters/setters, receiver preservation, compound/logical updates,
destructuring/loops, abrupt exception identity, super/new.target, and direct eval.
Instance/static brands install before all fields/blocks; partial field failure
retains every method brand, while duplicate stamping rejects before initializer
effects. Default-derived forwarding covers 100 methods-only classes, bound/custom
newTarget, base parameter timing, explicit derived super, and object returns that
skip super. Frozen receivers and reflection remain independent of private storage.
Collection retains methods, accessors, captures, and home objects, then reclaims
cycles. Checked-edge tests reject foreign/stale/non-callable/non-object handles
and work failures before replacing class records or adding an instance brand.
Four private method/getter/setter/eval recursion cases run on two-mebibyte
normal-debug stacks with the unchanged guards. Async/generator private methods
retain host-failure controls.

128 unchanged private-method/accessor originals add 256 harness-positive variants.
The focused review read 133 complete programs, including two earlier private-field
dependencies. Four whole files requiring Annex B lookup helpers remain excluded;
one static-setter program calls an absent getter method, so its TypeError cannot
verify the intended setter brand check. Selected sources cover instance/static
methods/accessors, shared/fresh identity, inheritance/receivers, nested/ordinary/
arrow capture, cross-kind shadowing, direct eval and initializer visibility,
name/length metadata, ordinary-property separation, repeated stamping, method
writes, missing getters/setters, computed names, and abrupt calls. Both toolchains
pass every added variant with ordinary defaults. No unsupported feature receives
credit; all source bytes, Git blobs, assertions, metadata, helpers, and the pin
remain unchanged. This does not claim complete elements-directory coverage.

WeakSet implements edition-17 CanBeHeldWeakly for objects and non-registered
symbols, including well-known symbols. Its hash index stores untraced checked
object identities and weak Arc-backed Symbol identities; neither keeps keys
reachable. Amortized pruning removes dead entries without scanning on every
access. Storage regressions check foreign/stale handles, work failures before
mutation, generation reuse, failed collection before sweep, and Symbol release.
Ten runtime regressions cover identity and absent coercion, invalid keys and own
brands, ordered/cached adder and iterator acquisition, abrupt iterator closing,
partial escaped sets, bound/derived/custom-newTarget construction, metadata and
full reflection/integrity, collection of key cycles, large unlimited collections,
opted-in host aborts, and native re-entry on two-mebibyte threads. WeakMap,
WeakRef, and FinalizationRegistry remain separate implementation work.

83 unchanged WeakSet originals add 166 harness-positive variants, all in both
Script modes. The complete 85-file directory review excludes one cross-realm
newTarget program and one global property-helper program requiring complete
global enumeration. Selected originals cover construction and iterator failures,
object/non-registered/well-known Symbol keys, registered Symbol rejection,
identity and duplicate additions, add/has/delete return values and receiver
brands, metadata, descriptors, and method construction rejection. Both toolchains
pass every new variant with the existing unchanged pin and eight helpers. GC and
host-abort behavior are exercised by the native storage/runtime regressions;
the upstream programs and assertions are preserved byte for byte.
