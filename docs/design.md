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
Conversions between BigInt and Number must compare mathematical values without
rounding the integer first. Literal grammar, string coercion, and mixed-type
operator rules remain in the parser and runtime rather than the arithmetic crate.
The AST stores validated BigInt digits and their radix. Evaluation converts them
under the realm's budget, so parsing never performs unbounded integer arithmetic.
The evaluator shares its step budget with integer arithmetic and conversion and
also bounds BigInt magnitude bits (65,536 by default). BigInt/Number comparisons
inspect the binary64 significand and exponent without rounding the integer.
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
handler runs. Thrown values, including object identities, are bound without coercion. Until the object
model supplies Error instances, binding a built-in exception reports Unsupported;
catch clauses without a parameter can still handle those exceptions. The optional
Annex B rule permitting var to redeclare a catch parameter is not enabled.

An engine owns realms, environments, execution contexts, and the object heap.
Lexical bindings distinguish uninitialized from undefined and preserve mutability.
Declarations are instantiated before evaluation. Closures retain environment
identities. Global object bindings and lexical bindings have distinct semantics.

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

Ordinary records start with string-keyed data properties in a creation-ordered
vector. Partial descriptors preserve omitted fields, enforce non-configurable
and non-writable invariants, and use SameValue. A successful equivalent-value
definition on a frozen property preserves its stored value, including NaN bits
(10.1.6.3). Enumeration sorts array indices numerically before other strings in
creation order. Deletion followed by re-creation gives a string a new position.
Property capacity failures remain distinct from descriptor rejection. Accessors,
Symbols, inherited lookup, and prototype mutation arrive on explicit boundaries;
the record layer does not yet expose objects to JavaScript.

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
ordered method lookups using the requested hint. Every currently materialized
heap object is non-callable, so ordinary objects without callable methods throw
TypeError. Missing callable intrinsics remain Unsupported. Add IsCallable/Call
dispatch before exposing function objects, and Symbol hooks before Symbol keys.
Arithmetic and comparisons convert original operands from left to right after
both expressions evaluate; templates and property names use the string hint.

Realm allocation is bounded by object slots and per-object property counts. Object
literals create data properties in source order, convert computed keys before
evaluating values, and implement the required non-computed `__proto__` initializer.
The intrinsic Object prototype has a stable, retained identity; its callable
properties remain incomplete. A lookup that reaches an unimplemented intrinsic
method reports Unsupported; own or nearer inherited data properties can shadow
that method normally. Object coercion hooks and primitive wrapper constructors
remain explicit implementation gaps.

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
fail without persisting a property; strict mode throws. Missing standard prototype
methods report Unsupported, while absent properties produce undefined. The
optional Annex B String methods and prototype accessor are not installed.

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
parentheses. Spread arguments and optional calls remain unsupported. Add builtin
function objects first, then captured environment storage and user functions.
All currently materialized values are non-callable; unavailable standard functions
report Unsupported during lookup rather than reaching a false TypeError result.

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
