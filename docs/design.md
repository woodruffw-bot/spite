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

Production crates use Rust's standard library and workspace crates. Two direct
external production dependencies are approved when needed: `regex` for regular
expressions and `jiff` for time APIs. No other new direct dependencies are approved.
External build dependencies remain forbidden. The existing `insta` development
dependency remains allowed. Review features and transitive dependencies when
introducing either exception. Neither dependency is added preemptively.

These libraries are implementation tools, not alternative language specifications.
RegExp syntax, matching, and UTF-16 behavior must still follow ECMA-262, including
features that `regex` does not implement. Time operations must likewise expose
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
| `spite-core` | Source locations, UTF-16 strings, shared language primitives | std |
| `spite-parser` | Lexical grammar, AST, parsing, static semantics and early errors | core |
| `spite-runtime` | Values, abstract operations, environments, objects, execution | core, bigint, heap, parser |
| `spite` | Small embedding facade and command-line host | core, parser, runtime |
| `spite-test262` | Test262 metadata, execution modes, harness and result accounting | engine, development tooling |

Create crates when they first have real functionality. Keep parser internals in
one crate. Keep built-ins inside the runtime until an actual dependency boundary
justifies another crate. Integer arithmetic lives in `spite-bigint`. A regular
expression implementation may later need its own crate and may use `regex`.

## Syntax

Use a handwritten lexer and recursive descent parser with binding powers for
expressions. Source positions are UTF-8 byte offsets into the supplied Rust source
string. JavaScript strings are sequences of UTF-16 code units, including lone
surrogates. Source adapters must document their decoding behavior.

The parser controls lexical goals for division, regular expressions, and template
tails. Preserve whether trivia contains a line terminator. Do not approximate ASI
by splitting lines. Carry grammar parameters for strict mode, await, yield, return,
and context-sensitive syntax. Keep parentheses where early errors depend on them.

The current eager scanner tracks braces in template substitutions. Before adding
RegExp literals, move lexical-goal selection into the parser so braces and
backticks inside a RegExp body cannot affect template scanning.

Use generated, versioned Unicode tables for identifiers. The current tables use
Unicode 18.0.0, with a pinned source digest and reproducible generation. Rust's alphabetic predicate
is not ECMAScript's ID_Start or ID_Continue. Numeric conversion, separators, escapes,
and line terminators need focused boundary tests.

Parsing returns an owned, inspectable syntax tree or a diagnostic with a source
span. A separate validation step can handle non-local early errors. Finish all
parsing and early-error checks before executing any part of a Script or Module.
Parser depth and evaluator work limits report host resource errors, not JavaScript
exceptions. Syntax or semantics that are not implemented must be recorded as gaps.
Never use unsupported syntax rejection as evidence of conformance to negative tests.

Arrow parsing supports non-async arrows with identifier parameters, optional
defaults, and assignment-expression or block bodies (15.3). A bounded token lookahead refines the
parenthesized parameter cover without changing ordinary parenthesized expressions.
Parameters are unique in both modes, strict binding rules are inherited, and no
line terminator may precede the arrow. Expression bodies inherit the In grammar
parameter; block bodies reset In and enable Return. Each function resets control
targets and labels. Its directive prologue enables strict parameter/body checks,
including legacy tokens before the directive and in nested functions. An own Use
Strict Directive is forbidden with defaults, while inherited strictness is allowed.
Defaults parse as AssignmentExpression with In enabled. Top-level
lexical declarations cannot conflict with parameters (15.3.1). Rest/pattern
parameters and async arrows remain explicit gaps. Function source ranges share an owned source allocation and preserve exact
text for Function.prototype.toString. Arrow instantiation captures the current environment identity and strictness.

Ordinary function expressions and declarations share the identifier parameter and
function-body parser with arrows (15.2). Names, parameters, bodies, and source text are retained.
Only simple lists in non-strict ordinary functions permit duplicate parameters;
strict or non-simple lists require unique names. A function's own strict directive
also constrains its optional name, parameters, and nested code. Function
expressions may appear in call/member positions, and nested bodies reset control
targets. Function heads and bodies each charge parser depth; declarations cannot
bypass expression recursion limits. Ordinary functions instantiate; strict calls
and non-strict calls with object, global, Boolean, or Number receivers execute. Ordinary
construction also executes; other primitive wrappers remain a runtime gap.
Generators, async functions, rest parameters, and patterns remain separate steps.

New expressions retain the constructor and optional argument list (13.3.5).
Constructor parsing consumes member access but leaves call parentheses to the new
expression, distinguishing `new F.x(a)` from `new F(a).x`. Nested new forms bind
inner argument lists first. Calls and construction share argument parsing with In
enabled; spans, strict validation, and depth budgets cover all children. Spread
and derived construction remain separate gaps. NewTarget is permitted in ordinary
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
a budget that bounds result bits and charges word operations before doing the
work. The runtime translates budget exhaustion into a host limit, while division
by zero and negative exponents become JavaScript RangeError exceptions.
Comparisons between BigInt and Number must compare mathematical values without
rounding the integer first. Literal grammar, string coercion, and mixed-type
operator rules remain in the parser and runtime rather than the arithmetic crate.
The AST stores validated BigInt digits and their radix. Evaluation converts them
under the realm's budget, so parsing never performs unbounded integer arithmetic.
The evaluator shares its step budget with integer arithmetic and conversion and
also bounds BigInt magnitude bits (65,536 by default). BigInt/Number comparisons
inspect the binary64 significand and exponent without rounding the integer.
The integer library also supplies explicit conversion to binary64 for the Number
constructor (21.1.1.1). It retains the leading 53 bits and rounds once using guard,
sticky, and parity bits; halfway results choose the even significand. Rounding
can carry into the exponent or overflow to signed infinity. Conversion allocates
no intermediate integer and charges work before examining its bits.
ToNumber is fallible; it rejects BigInt with TypeError. Language ToString produces
decimal digits. Host value display uses exact hexadecimal BigInt notation to keep
diagnostic formatting linear and independent of the evaluator's remaining budget.

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
handler runs. Thrown values, including object identities, are bound without
coercion. Built-in exceptions materialize as Error instances when a binding needs
their JavaScript value; catch clauses without a parameter need no such value. The optional
Annex B rule permitting var to redeclare a catch parameter is not enabled.

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

Realm initialization runs on first evaluation, after parsing succeeds. It creates
the global lexical environment, implemented intrinsics, and an ordinary global
object whose prototype is Object.prototype (9.3.1). This fixed graph uses a separate
100,000-unit initialization budget; `max_steps` applies to each Script. The host
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

Slots have a fixed host-configured upper bound. Reuse increments the generation;
a generation that cannot increment retires its slot permanently. Every access
checks heap identity, generation, and occupancy. Heap-owned values cannot be
reached through stale handles, including after slot reuse.

Add a non-moving mark-and-sweep collector with caller-supplied roots and iterative
edge traversal. Charge collection work before each scan or edge traversal. If
tracing sees a foreign/stale handle or exceeds its budget, return without sweeping.
Trace iterators yield an optional handle for each inspected field, including
primitive-valued fields, so a scan cannot evade its budget by filtering out all
non-reference properties. Deep object graphs use an explicit work stack.
Allocation initially never invokes collection implicitly. Before integrating
collection into evaluation, explicitly root environment bindings, intrinsics,
pending completions, suspended frames, expression temporaries, and host-held
values. No allocation-triggered collection or JavaScript-visible GC hook may run
until those root lifetimes are implemented and tested.

Property descriptors, key ordering, prototypes, internal methods, and exotic
objects are semantic requirements. Add ordinary objects first. Implement arrays,
functions, proxies, typed arrays, and other exotic objects on explicit internal
method boundaries rather than ad hoc evaluator branches.

Ordinary records store string-keyed properties in a creation-ordered
vector. Partial descriptors preserve omitted fields, enforce non-configurable
and non-writable invariants, and use SameValue. A successful equivalent-value
definition on a frozen property preserves its stored value, including NaN bits
(10.1.6.3). Enumeration sorts array indices numerically before other strings in
creation order. Deletion followed by re-creation gives a string a new position.
Property capacity failures remain distinct from descriptor rejection. Storage
exposes checked internal operations; realms supply JavaScript execution and
exception semantics. Symbol keys remain a separate implementation boundary.

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
a primitive or JavaScript exception. Realm-level OrdinaryToPrimitive performs
ordered method lookups and calls using the requested hint and original receiver.
Objects without a method yielding a primitive throw TypeError. Object.prototype
toString and valueOf provide ordinary default conversion. Missing intrinsics
remain Unsupported. Add Symbol hooks before exposing Symbol keys.
Arithmetic and comparisons convert original operands from left to right after
both expressions evaluate; templates and property names use the string hint.

Realm allocation is bounded by shared heap slots and per-object property counts. Object
literals create data properties in source order, convert computed keys before
evaluating values, and implement the required non-computed `__proto__` initializer.
The intrinsic Object prototype has a stable, retained identity and its mandatory
string-keyed methods. A lookup that reaches an unimplemented intrinsic
method reports Unsupported; own or nearer inherited data properties can shadow
that method normally. Symbol coercion hooks and BigInt/Symbol
wrapper constructors remain explicit implementation gaps.

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
parentheses. Spread arguments and optional calls remain unsupported. Builtin
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

Error objects include Error and the six NativeError constructors,
Error.prototype.toString, Error.isError, and ordered message/cause initialization
(20.5). Only instances carry ErrorData; prototype
objects do not. NativeError constructors inherit Error, and their prototypes
inherit Error.prototype. Constructor calls allocate before message conversion,
then inspect object-valued options for an inherited or own cause property. Keep
ErrorData private and trace cause values through ordinary properties. AggregateError
depends on iteration and remains a separate increment.

Built-in runtime exceptions materialize as Error objects when a catch binding
needs a JavaScript value, using intrinsic prototypes independently of replaced
global bindings. Preserve the host-facing exception category, message, and source
span for uncaught failures. Host Unsupported/Limit failures must still bypass
JavaScript handlers. Standard Error properties are the entire exposed interface;
stack traces and host-specific fields are outside the language baseline.

Embedding hosts can obtain exception values and read properties through bounded
realm operations. Exception conversion preserves explicit throw identity and
creates a fresh standard Error for a Rust-described built-in failure; parse and
host failures have no JavaScript value. Property reads preserve Get semantics,
including accessor receivers and abrupt completions. Each operation receives a
fresh work budget, and returned values require host roots across collection.
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
bounded work. Enumeration of incomplete Object/Function/String/global intrinsics reports
Unsupported until their own key sets are complete. Object.freeze/seal and
isFrozen/isSealed implement SetIntegrityLevel/TestIntegrityLevel (7.3.15–16).
They preserve accessor identity without invoking getters, perform shallow changes,
and use mapped-argument descriptor rules when freezing indexed properties.
Extensible objects fail integrity predicates before key enumeration.
Object.assign snapshots each source's keys and then rechecks enumerability before
ordinary Get/Set, retaining getter/setter effects and earlier copies on abrupt
completion. Object.getOwnPropertyDescriptors creates ordinary own data properties
containing fresh descriptor objects, preserving accessors without invoking them.
Remaining Object static methods are explicit gaps.

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
call. Direct calls and apply share a configurable argument-list limit (16,384 by
default). Work and argument limits are checked before allocating a large list.

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

Instanceof uses relational precedence, evaluates both operands, then follows
InstanceofOperator and OrdinaryHasInstance (13.10.2, 7.3.21). Bound functions
re-enter the operator on their target through an iterative loop. Primitive left
operands return false before reading prototype; object operands require an object
prototype and compare its identity with ancestors. Callable arrows can participate
without being constructors. Non-callable objects inheriting the default hook return
false; non-callable objects without a hook throw TypeError.

Until Symbol keys are exposed, hook lookup recognizes Function.prototype's fixed,
non-writable, non-configurable @@hasInstance method along the actual prototype
chain (20.2.3.6). This path does not expose a symbol-keyed callable or descriptor.
Symbol support must materialize that property and implement custom-hook lookup
before symbol properties or reflection become accessible. Prototype getter calls
retain their receiver and abrupt completions, and traversal consumes host work.

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
closures. When defaults are present, body vars get a separate environment and
same-named vars copy the initialized parameter value (10.2.11). Function length
counts parameters before the first default (15.1.5). Partial initialization and
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
after returns or abrupt default initialization. The required Symbol.iterator hook
is deferred until Symbol/Array iteration is exposed; Object.prototype.toString
recognizes the Arguments tag. Boolean, Number, and String non-strict receivers are boxed; other
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
requiring missing JSON and other facilities stay Unsupported and never count as passes.
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
