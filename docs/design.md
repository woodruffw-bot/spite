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
Private fields, super, async contexts, and eval await their own implementations.

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
String prototype enumeration awaits its remaining APIs.
The Annex B initialized-var extension remains Unsupported in non-strict code and
is a SyntaxError in strict code; it never changes the core binding algorithm.

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
lexical declarations cannot conflict with parameters (15.3.1). Pattern parameters
and async arrows remain explicit gaps. Function source ranges share an owned source allocation and preserve exact
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
Generators, async functions, and patterns remain separate steps.

Identifier rest parameters are final and have no initializer or trailing comma
(15.2.3). They collect the remaining internal argument values into a fresh dense
intrinsic Array, without invoking JavaScript iterators, setters, species, or the
global Array binding. Function length stops before the first default or rest
parameter. Rest makes a parameter list non-simple, requiring unique names,
rejecting an own Use Strict Directive, and selecting unmapped arguments. Parameter
expressions separately determine whether body vars need their own environment
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
Initial sparse truncation uses a conservatively charged quadratic algorithm;
iteration over absent indices up to the logical length is unnecessary.

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
assignment patterns remain Unsupported until their semantics exist.

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
Missing intrinsics remain Unsupported. Each new object kind must implement its
required Symbol hooks alongside its string-keyed API.
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
call/construct metadata for branding and newTarget validation, while invoking it
returns Unsupported until dynamic global-scope compilation is implemented.
This gap is never converted to a JavaScript TypeError or accepted as a passing
negative. The constructor and prototype remain intrinsic roots after deletion.

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
getters retain the search object as receiver. The RegExpMatcher brand fallback
remains false until RegExp objects exist; string-keyed lookalikes are not hooks.
String.split (22.1.3.23) checks receiver coercibility, then delegates Symbol.split
only for object separators as required by edition 17. Pass original this/limit
values and return the hook's result without conversion. Fallback converts this
to a String, limit through ToUint32 (undefined defaults to 2^32-1), then separator
to a String even when the limit is zero. Undefined separators return the full
String; empty separators split individual UTF-16 units with no extra empty parts.
Nonempty separators use non-overlapping searches, retaining leading/trailing
empty substrings and stopping at the limit. Checked copies and each search
comparison charge opted-in work. Fresh intrinsic Arrays bypass constructors,
species, and inherited setters. RegExp-specific splitting awaits RegExp objects.

String.replace (22.1.3.19) delegates Symbol.replace only on object searches,
preserving original this/replacement values and returning the hook's result.
Fallback converts the receiver and search, then converts non-callable replacements
before searching even when there is no match. Replace only the first UTF-16 match.
Call functional replacements with undefined this and exactly match/position/full
String; convert their return value to literal replacement text. Non-functional
replacements use GetSubstitution with no captures (22.1.3.19.1): expand $$, $&,
prefix, and suffix tokens once; numeric/named-capture tokens remain literal.
Checked output reservation and copied/scanned units charge opted-in work. Input
Strings remain fixed through callbacks. RegExp captures remain a separate step.

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
work. RegExp objects and their replacement hooks remain pending.

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
bounded work. Enumeration of the incomplete String prototype, Array constructor, and global
object reports Unsupported until their own key sets are complete. Object.freeze/seal and
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
Async/generator methods, parameter patterns, and super remain separate gaps.

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
