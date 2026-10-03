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
| Expressions | Primitive and object literals, untagged templates with substitutions, identifiers, parentheses, simple and compound assignment, prefix/postfix updates, conditional and comma expressions |
| Object literals | Literal and computed keys, shorthand, ordered data properties, duplicate-key replacement, required prototype initializers, identity equality and truthiness |
| Properties | Ordinary own/inherited data references, ordered reads/writes/updates/deletion, primitive property operations, UTF-16 String indices and length, strict write/delete failures |
| Operators | Arithmetic, exponentiation, bitwise, shifts, equality, primitive comparison, logical and nullish operators, typeof, void, delete, and in |
| Statements | Empty and expression statements, let, const, and var, blocks, if/else, while, do-while, for with expression, lexical, or var headers, switch, labels, break/continue with optional targets, throw, try with catch and/or finally; catch identifiers bind supported throws |
| Static semantics | Implemented ASI rules, strict directives, duplicate lexical bindings, strict binding and assignment restrictions, escaped reserved-word checks, enclosing-loop/switch checks, duplicate labels, control-target validation, duplicate defaults and case-block lexical names |
| Runtime | Persistent realm state, lexical scope, declaration instantiation, per-iteration let scopes, temporal dead zones, immutable bindings, ordered evaluation |
| Completions | Empty versus undefined, statement-list values, if-statement UpdateEmpty, loop body values, break/continue propagation through blocks, conditionals, nested loops, and switch fall-through, primitive and object throws, finalizer preservation and overrides of language completions |
| Global values | undefined, NaN, Infinity, and simple sloppy-mode global assignment |
| Limits | 1 MiB source, depth 64, configurable evaluation/arithmetic work, string code units, BigInt magnitude bits, object slots, and properties per object |
| Tests | Algorithm and integration tests, AST and diagnostic snapshots, 31 reviewed Test262 variants from 11 hashbang and 10 BigInt files, plus 13 identifier lexer and 6 statement parser fixtures |

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
Runtime object records add string-keyed data descriptors, extensibility,
deletion, array-index ordering, and tracing of prototype handles. Frozen
properties use SameValue and preserve equivalent NaN payloads. These records
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
temporary and pending-completion root lifetimes are implemented.

Dotted and computed property references parse and evaluate for ordinary objects,
including assignment, compound assignment, update, and delete targets. Tests cover
IdentifierName spelling, precedence, strict targets, ASI, inherited reads/writes,
and edition-17 deferred key conversion. Nullish bases produce TypeError at
GetValue/PutValue/delete. Primitive property operations use ephemeral wrapper
semantics; String indices and length are read-only and non-configurable. Writes
do not persist on primitives, and strict failures throw TypeError. Reading a
missing standard prototype method reports Unsupported, while ordinary object
properties can shadow it. Optional Annex B prototype accessors and String methods
are not installed.

The `in` operator includes inherited and undefined-valued properties, rejects
primitive right operands before converting the left key, and recognizes standard
Object.prototype method presence. The parser applies the In grammar parameter to
for initializers and nested expressions. For-in/of iteration remains unsupported.

Symbols, primitive wrapper constructors,
arrays, functions, calls, closures, constructors, classes, destructuring, regular
expressions, tagged templates, for-in/of, catch patterns, generators,
async functions, promises, modules, standard library objects, eval, agents, shared
memory, and automatic garbage collection remain open. See the roadmap for their order.
The BigInt constructor and its prototype/static methods remain part of standard
library work. The command-line host displays BigInt completion values in exact
hexadecimal notation with an `n` suffix; JavaScript string conversion is decimal.

Global lexical bindings and Script var declarations persist between evaluations.
New global vars are non-deletable. A var declaration for an existing global value
preserves its value and deletability. There is no observable global object or
full property model yet. Built-in error categories are represented in
Rust. Catch clauses without a parameter handle language throws and built-in
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
Script execution passes. The fixture suite does not run the general harness,
parse arbitrary Test262 YAML, report a whole-suite pass rate, or cover all of the
implemented semantics. The new `spite-test262` crate reads a documented metadata
subset, plans strict/non-strict/module/raw modes, and preserves harness include
order. Its Script runner uses fresh realms, separates harness failures from test
results, and matches negative tests by phase and error type. Parse-negative passes
require a reviewed diagnostic range and message; unreviewed syntax errors remain
unverified. Unsupported features, missing host helpers, and resource limits are
separate non-passing results. Modules, async completion, agents, and the full
upstream harness remain unsupported. CI runs the reviewed corpus on Linux and
Windows with the minimum supported Rust version and stable Rust. Its 31 variants
are four raw positive evaluations and 27 reviewed parse-negative variants, not a
whole-suite conformance measurement. Component fixtures do not enter this count.

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
