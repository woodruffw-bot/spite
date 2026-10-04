# Implemented coverage

spite is an initial executable subset, not a conforming ECMAScript engine.
The target is ECMAScript 2026, edition 17. This file records implementation status,
not an alternative language specification.

## Available now

| Area | Implemented behavior |
| --- | --- |
| Source | UTF-8 input, byte spans, distinct syntax, unsupported, and limit diagnostics |
| Strings | UTF-16 code units, lone surrogates, ordinary escapes, Unicode escapes, line continuation |
| Lexical grammar | ECMAScript whitespace and line terminators, comments, initial hashbang, Unicode 18.0.0 identifiers, Unicode identifier escapes |
| Numbers | Decimal, binary, octal, hex, numeric separators, binary64 rounding and overflow |
| Legacy literals | Leading-zero octal and decimal numbers, octal/decimal string escapes in non-strict code, and strict early errors including escapes before a use-strict directive |
| BigInt | Exact literals in all four radices, signed arithmetic, truncating division and remainder, exponentiation, arithmetic shifts, infinite sign-extension bitwise operations, updates and compound assignment |
| BigInt coercion | Boolean and decimal string conversion, integer-string equality and ordering, exact mixed Number comparisons, TypeError for mixed numeric arithmetic and unary plus, RangeError for zero division and negative exponents |
| BigInt APIs | Calls with exact integral Number and integer-string conversion, signed/unsigned width reduction, wrappers and boxed receivers, branded valueOf/toString/toLocaleString, and observable prototype tags |
| Expressions | Primitive, object, and array literals, untagged templates with substitutions, identifiers, this, parentheses, simple and compound assignment, prefix/postfix updates, conditional and comma expressions |
| Arrays | Calls/new, of, from, isArray, literal holes/spread and trailing commas, sparse indexed properties, ordered length coercion, read-only length, partial truncation, generic at/join/push/pop/shift/unshift/reverse/fill/copyWithin and includes/indexOf/lastIndexOf, forEach/every/some and reduce/reduceRight callbacks, species-aware map/filter/slice/concat/flat/flatMap/splice, find/findIndex/findLast/findLastIndex, sort/toSorted and toReversed/with/toSpliced copies, dynamic toString, and toLocaleString |
| Object literals | Literal and computed keys, shorthand, ordered data properties and spread, duplicate-key replacement, required prototype initializers, identity equality and truthiness |
| Object coercion | Realm-level ordered method lookup, TypeError for objects without callable conversion methods, left-to-right operand conversion, Object.prototype toString/valueOf dispatch |
| Calls | Builtin, arrow, and ordinary calls with strict, object, global, or Boolean/Number/String boxed receivers, function typeof, name/length descriptors, ordered callee/argument evaluation, member receivers, Object.prototype conversion methods |
| Arrow functions | Expression/block bodies, identifier/default/rest parameters, closures, local declarations, return completions, strict directives, name/length metadata, and source stringification |
| Instance checks | instanceof for ordinary/bound functions with ordered prototype lookup; materialized Symbol.hasInstance and custom hooks tested through native symbol injection |
| Construction | new with optional arguments and nested/member precedence, ordinary and bound constructors, prototype selection, object/primitive return rules, and ordered evaluation |
| Function syntax | Ordinary named/anonymous function expressions and named declarations, identifier/default/rest parameters, body early errors, and variable versus block scope, declaration instantiation/hoisting, and standard prototype/name/length properties; Boolean, Number, String, Symbol, and BigInt receivers box |
| Properties | Ordinary own/inherited data references, ordered reads/writes/updates/deletion, primitive property operations, UTF-16 String indices and length, strict write/delete failures |
| Operators | Arithmetic, exponentiation, bitwise, shifts, equality, primitive comparison, logical and nullish operators, typeof, void, delete, and in |
| Statements | Empty and expression statements, let, const, and var, blocks, if/else, while, do-while, three-clause for, synchronous for-of, and for-in over complete prototype chains with assignment/var/lexical bindings, switch, labels, break/continue with optional targets, function-body return, throw, try with catch and/or finally; catch identifiers bind supported throws |
| Static semantics | Implemented ASI rules, strict directives, duplicate lexical bindings, strict binding and assignment restrictions, escaped reserved-word checks, enclosing-loop/switch checks, duplicate labels, control-target validation, duplicate defaults and case-block lexical names |
| Runtime | Persistent realm state, lexical scope, declaration instantiation, per-iteration scopes, temporal dead zones, immutable bindings, ordered evaluation and synchronous iterator closing |
| Completions | Empty versus undefined, statement-list values, if-statement UpdateEmpty, loop body values, return and break/continue propagation through blocks, conditionals, nested loops, and switch fall-through, primitive and object throws, finalizer preservation and overrides of language completions |
| Global values | Ordinary global object, globalThis, Script/arrow global this, undefined, NaN, Infinity, and property-backed global bindings |
| Limits | Opt-in source size, evaluation/arithmetic work, string code units, BigInt magnitude bits, shared object/environment heap slots, properties per object, and call argument count; checked platform capacity and native-stack guards |
| Tests | Algorithm and integration tests, AST and diagnostic snapshots, 2850 reviewed Test262 variants, three pinned harness files, plus 13 identifier lexer and 6 statement parser fixtures |

Debugger statements parse with ordinary ASI and produce an empty completion.
This host has no active debugging facility (ECMA-262 14.16.1).

Identifier names retain their exact decoded code point sequence. Canonically
equivalent spellings are distinct bindings. Escapes cannot turn reserved words
into identifiers or stand in for grammar keywords. Unicode property tables are
generated from pinned data and do not depend on the Rust toolchain version.

Each production crate currently depends only on std and workspace crates. The
design permits regex and jiff when needed. Neither has been added. insta remains
a test-only dependency. Rust unsafe code is forbidden through inherited workspace lints.

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

The `spite-heap` foundation provides capacity-bounded generational storage,
checked cross-heap identity, stale-handle rejection, and bounded iterative
collection from explicit roots. Handle or budget failures occur before sweeping.
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
primitive boxing, nullish skipping, closures, TDZ, and completion values. Reaching
incomplete String/Function/shared-Iterator prototypes remains Unsupported; fully
enumerating those chains awaits their missing methods. The non-strict Annex B
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
are traced through ordinary properties. AggregateError remains unimplemented.

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
if a later definition is rejected. Enumeration of incomplete Object/Function/String/Array/Iterator/global
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
names, abrupt key evaluation, and setter scope. Async/generator methods, super,
and parameter patterns remain unsupported.
Object.fromEntries/groupBy remain explicit gaps, including descriptor inspection
or mutation of an unimplemented intrinsic property.

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
RegExpMatcher brand fallback awaits RegExp objects. String.raw processes
ordinary array-like templates with ordered raw/length/index reads and interleaved
substitution conversion. All baseline String constructor static properties are
installed, and its own-key enumeration is supported. Remaining prototype methods and enumeration of the
incomplete String.prototype remain Unsupported; tagged template syntax is still open.

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
lookup, and preserve ordered live reads. With never reads its replaced index. Array
assignment patterns remain pending.
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
attributes. The shared Iterator constructor and helpers remain Unsupported.
Symbol-keyed access is tested through native injection, Script integration tests,
and reviewed upstream Symbol/iterator fixtures.

Remaining String methods,
Array.fromAsync, derived construction, classes, destructuring, regular
expressions, tagged templates, for-await-of, catch patterns, generators,
async functions, promises, modules, standard library objects, eval, agents, shared
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
Realm initialization creates 170 retained entries outside the per-Script work
allowance; opted-in allocation/property quotas still apply.
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
Catch binding patterns remain unsupported.

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
tests, 30 call/construction object-spread tests, twelve Boolean tests, 63
Number tests, ten numeric parsing tests, five global numeric predicate tests, and
22 Error construction, conversion, and prototype tests, 48 BigInt constructor,
width reduction, formatting, and receiver-brand tests, plus 140 Object descriptor,
prototype, extensibility, creation, copying, key enumeration, integrity, and SameValue tests, and
153 String wrapper, raw construction, character, search, sequence, trimming, repetition, padding, Unicode
well-formedness, conversion, and String iteration tests in both Script modes. Another 677 Array and Array iterator
files cover construction, of, isArray, literal elisions and spread (including
fifteen nested object-spread files), length/index boundaries,
truncation, generic at/join/push/pop/shift/unshift/reverse/fill/copyWithin and includes/indexOf/lastIndexOf, toString/toLocaleString, find/findIndex/findLast/findLastIndex, and ordered forEach/every/some
and reduce/reduceRight callbacks, species-aware map/filter/slice/concat/flat/flatMap/splice, plus sort/toSorted, toReversed/with/toSpliced copies,
and keys/values/entries iteration, including mapped/unmapped arguments. The 34
Symbol files and 23 object method/accessor files run in their prescribed Script modes.
Another 53 positive for-of files and 22 reviewed for-of parse-negative files cover
iteration, bindings, header grammar, and closing precedence. Seven rest-parameter
positives and twelve parameter parse negatives cover Arrays, length, argument
aliasing, call/apply, and non-simple parameter syntax. Another 36 for-in positives
and 20 reviewed negatives cover ordered enumeration, mutation, bindings, ASI,
control flow, and header/body early errors. Controls
verify successful assertions and explicit assertion failures. Some string comparison
failure formatting still requires missing JSON and remains Unsupported;
Array.fromAsync, other includes, async completion, and agents
remain gaps. CI runs the reviewed corpus on Linux and Windows with MSRV and stable
Rust. Its 2850 variants are four raw positives, 2676 positives using the upstream
harness, and 170 reviewed parse negatives. Component fixtures and harness files do
not enter this count; it is not a whole-suite conformance measurement.
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
