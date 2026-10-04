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
| Expressions | Primitive and object literals, untagged templates with substitutions, identifiers, this, parentheses, simple and compound assignment, prefix/postfix updates, conditional and comma expressions |
| Object literals | Literal and computed keys, shorthand, ordered data properties, duplicate-key replacement, required prototype initializers, identity equality and truthiness |
| Object coercion | Realm-level ordered method lookup, TypeError for objects without callable conversion methods, left-to-right operand conversion, Object.prototype toString/valueOf dispatch |
| Calls | Builtin, arrow, and ordinary calls with strict, object, global, or Boolean/Number boxed receivers, function typeof, name/length descriptors, ordered callee/argument evaluation, member receivers, Object.prototype conversion methods |
| Arrow functions | Expression/block bodies, identifier/default parameters, closures, local declarations, return completions, strict directives, name/length metadata, and source stringification |
| Instance checks | instanceof for ordinary/bound functions and inherited intrinsic hooks, with ordered prototype lookup; custom Symbol.hasInstance hooks await symbols |
| Construction | new with optional arguments and nested/member precedence, ordinary and bound constructors, prototype selection, object/primitive return rules, and ordered evaluation |
| Function syntax | Ordinary named/anonymous function expressions and named declarations, identifier/default parameters, body early errors, and variable versus block scope, declaration instantiation/hoisting, and standard prototype/name/length properties; Boolean and Number receivers box; other non-strict primitive receivers remain Unsupported |
| Properties | Ordinary own/inherited data references, ordered reads/writes/updates/deletion, primitive property operations, UTF-16 String indices and length, strict write/delete failures |
| Operators | Arithmetic, exponentiation, bitwise, shifts, equality, primitive comparison, logical and nullish operators, typeof, void, delete, and in |
| Statements | Empty and expression statements, let, const, and var, blocks, if/else, while, do-while, for with expression, lexical, or var headers, switch, labels, break/continue with optional targets, function-body return, throw, try with catch and/or finally; catch identifiers bind supported throws |
| Static semantics | Implemented ASI rules, strict directives, duplicate lexical bindings, strict binding and assignment restrictions, escaped reserved-word checks, enclosing-loop/switch checks, duplicate labels, control-target validation, duplicate defaults and case-block lexical names |
| Runtime | Persistent realm state, lexical scope, declaration instantiation, per-iteration let scopes, temporal dead zones, immutable bindings, ordered evaluation |
| Completions | Empty versus undefined, statement-list values, if-statement UpdateEmpty, loop body values, return and break/continue propagation through blocks, conditionals, nested loops, and switch fall-through, primitive and object throws, finalizer preservation and overrides of language completions |
| Global values | Ordinary global object, globalThis, Script/arrow global this, undefined, NaN, Infinity, and property-backed global bindings |
| Limits | 1 MiB source, depth 64, configurable evaluation/arithmetic work, string code units, BigInt magnitude bits, shared object/environment heap slots, properties per object, and call argument count |
| Tests | Algorithm and integration tests, AST and diagnostic snapshots, 156 reviewed Test262 variants, two pinned harness files, plus 13 identifier lexer and 6 statement parser fixtures |

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

The `spite-heap` foundation provides capacity-bounded generational storage,
checked cross-heap identity, stale-handle rejection, and bounded iterative
collection from explicit roots. Handle or budget failures occur before sweeping.
Runtime object records add string-keyed data/accessor descriptors, extensibility,
deletion, array-index ordering, and tracing of prototype handles. Frozen
properties use SameValue and preserve equivalent NaN payloads. Accessor descriptors
validate callable handles, trace getter/setter edges, preserve omitted attributes,
and enforce non-configurable identity and kind invariants. Reads/writes dispatch
accessors with the original receiver after releasing storage borrows. These records
have a host property-capacity limit. A heap context adds bounded iterative
prototype traversal, cycle-checked mutation, inherited data reads and presence
checks, and receiver-sensitive writes. Rust values carry object identity and
trace object-valued property edges, including cycles. Object literals parse with
literal/computed keys, shorthand, trailing commas, and duplicate prototype-setter
early errors. Evaluation preserves key/value order and implements ordinary data
properties and prototype initializers. Methods, accessors, and spread remain
unsupported. Explicit collection between evaluations retains persistent bindings,
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
descriptors. Object.prototype has an immutable null prototype, toString, and
valueOf. Default ordinary-object conversion is supported. Intrinsic initialization
is atomic and the twenty-two objects remain rooted. Function.prototype caller/arguments
accessors use the shared, non-extensible %ThrowTypeError% with frozen name/length
metadata. Their reads/writes throw catchable TypeError in both modes. String/BigInt/Symbol
wrappers, spread arguments, and optional calls remain open; missing operations report Unsupported. Function.prototype
call passes receivers unchanged through bounded iterative dispatch. Native function
toString uses the original builtin name even after public name changes; generated
strings obey host limits. Apply accepts ordinary array-like objects, converts
length with ToLength, and reads inherited/indexed getters before the target call.
Argument lists have a configurable host limit shared with direct calls and bound
arguments. Bind implements callable receiver/argument capture, target prototype
selection, standard length/name metadata, and iterative invocation. Captured
object edges survive collection and unreachable cycles are reclaimed. Re-entrant
getter/coercion calls have a host nesting limit of 64; tail transfers are iterative.

Arrows with identifier parameters and expression or block bodies execute with shared lexical captures,
fresh mutable parameter bindings, missing/extra argument handling, inherited and
body-local strictness, unique names, source retention, and bounded nesting.
Block bodies support local var/lexical instantiation and return completions through
loops, labels, switch, try/catch/finally, including return ASI and early errors. Metadata includes
standard name inference and exact Function.prototype.toString source. Captured
per-iteration/catch/block environments survive collection; unreachable cycles are
reclaimed. Default parameters support ordered initialization, parameter TDZ,
separate body var environments, closure capture, name inference, and length. Rest
parameters, patterns, and async arrows remain gaps.

Strict ordinary functions execute with preserved receivers, shared lexical captures,
hoisting, defaults, and return completions. Unmapped arguments expose original
indices, length, and restricted callee; indices and parameters do not alias. Arrows
capture this and arguments from enclosing functions, including across collection.
Non-strict functions with object, Boolean, Number, or nullish/global receivers also execute. Simple parameter lists
use mapped arguments, including last-duplicate rules, live descriptor values,
receiver-sensitive writes, and detachment on deletion, accessor conversion, or
non-writable changes. Default parameters use unmapped arguments. Parameter/body
arguments declarations shadow or suppress the implicit binding as specified.
The arguments Symbol.iterator hook and String/BigInt/Symbol primitive wrappers remain unimplemented.
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
toString, radix validation, and exact toFixed formatting are implemented; finite
nonzero formatting in other radices reports Unsupported. toFixed uses exact
binary values and rounds ties to the larger magnitude for 0–100 fraction digits.
Other formatting methods and parseInt/parseFloat
remain gaps, including their callable values. NumberData preserves negative zero
and NaN independently of the object's current prototype.

Symbols, String/BigInt wrapper APIs,
arrays, derived construction, classes, destructuring, regular
expressions, tagged templates, for-in/of, catch patterns, generators,
async functions, promises, modules, standard library objects, eval, agents, shared
memory, and automatic garbage collection remain open. See the roadmap for their order.
The BigInt constructor and its prototype/static methods remain part of standard
library work. The command-line host displays BigInt completion values in exact
hexadecimal notation with an `n` suffix; JavaScript string conversion is decimal.

Global lexical bindings and Script var declarations persist between evaluations.
New global vars are non-deletable. A var declaration for an existing global value
preserves its descriptor and value. Global identifiers and globalThis properties
share storage; lexical bindings stay separate. Declaration checks use actual own
property attributes and extensibility. Global accessors and inherited properties
retain correct receivers, and strict assignments recheck bindings deleted by RHS
evaluation. Replacing/deleting globalThis does not change the realm's this identity.
Realm initialization creates eleven retained entries under a separate fixed work
budget before Script execution; allocation/property limits still apply.
Built-in error categories are represented in Rust. Catch clauses without a parameter handle language throws and built-in
exceptions. Catch binding identifiers now parse with scope and strict-mode early
errors, including the required non-browser rejection of conflicting var names.
Supported throws initialize a mutable catch binding without coercion, preserving object identity. The catch
binding and block scopes are restored across every completion and host failure.
Binding a built-in exception currently reports Unsupported because JavaScript
Error objects are not yet implemented. Catch binding patterns remain unsupported.

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
executes for nine positive function/capture tests, twelve Boolean tests, and nineteen
Number tests in both Script modes. Controls
verify successful assertions and explicit assertion failures. Some comparison
failure formatting still requires missing String/JSON APIs and remains Unsupported;
arrays, built-in Error constructors, other includes, async completion, and agents
remain gaps. CI runs the reviewed corpus on Linux and Windows with MSRV and stable
Rust. Its 156 variants are four raw positives, 80 positives using the upstream
harness, and 72 reviewed parse negatives. Component fixtures and harness files do
not enter this count; it is not a whole-suite conformance measurement.

Number-to-string formatting uses Rust's shortest round-trip decimal conversion
with ECMAScript presentation rules. Primitive numeric operations have boundary
regressions. Exhaustive numerical and cross-platform conformance audits remain
part of the roadmap.

The resource limits bound specific work and value sizes. They do not provide a
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
