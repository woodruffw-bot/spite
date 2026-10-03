# Implemented coverage

spite is an initial executable subset, not a conforming ECMAScript engine.
The target is ECMAScript 2026, edition 17. This file records implementation status,
not an alternative language specification.

## Available now

| Area | Implemented behavior |
| --- | --- |
| Source | UTF-8 input, byte spans, distinct syntax, unsupported, and limit diagnostics |
| Strings | UTF-16 code units, lone surrogates, ordinary escapes, Unicode escapes, line continuation |
| Lexical grammar | ECMAScript whitespace and line terminators, comments, initial hashbang, Unicode 18.0.0 identifiers |
| Numbers | Decimal, binary, octal, hex, numeric separators, binary64 rounding and overflow |
| Expressions | Primitive literals, identifiers, parentheses, simple assignment, conditional and comma expressions |
| Operators | Arithmetic, exponentiation, bitwise, shifts, primitive equality and comparison, logical and nullish operators, typeof and void |
| Statements | Empty and expression statements, let and const, blocks, if/else, throw |
| Static semantics | Implemented ASI rules, strict directives, duplicate lexical bindings, strict binding and assignment restrictions |
| Runtime | Persistent realm state, lexical scope, declaration instantiation, temporal dead zones, immutable bindings, ordered evaluation |
| Completions | Empty versus undefined, statement-list values, if-statement UpdateEmpty, uncaught primitive throws |
| Global values | undefined, NaN, Infinity, and simple sloppy-mode global assignment |
| Limits | 1 MiB source, depth 64, configurable evaluation steps and string code-unit limits |
| Tests | Algorithm and integration tests, AST and diagnostic snapshots, 11 pinned Test262 hashbang fixtures |

Each production crate depends only on std and workspace crates. insta is a test
only dependency. Rust unsafe code is forbidden through inherited workspace lints.

## Not implemented

Identifier escapes, BigInt, Symbols, objects, properties,
arrays, functions, calls, closures, constructors, classes, destructuring, regular
expressions, templates, var, loops, labels, switch, try/catch/finally, generators,
async functions, promises, modules, standard library objects, eval, agents, shared
memory, and garbage collection remain open. See the roadmap for their order.

Global lexical and simple value bindings work, but there is no observable global
object or full property model yet. Built-in error categories are represented in
Rust. JavaScript Error objects and catchable exception handling are future work.

Recognized missing features return Unsupported. Because the grammar is incomplete,
a syntax diagnostic alone does not prove arbitrary input violates ECMA-262. The
current parser must not be used to score general Test262 negative tests.

The lexical Test262 smoke suite uses a reviewed manifest and matches the specific
rejection point for each negative fixture. It does not run the general harness,
parse arbitrary Test262 YAML, report a whole-suite pass rate, or cover all of the
implemented semantics. A complete runner is a separate roadmap item.

Number-to-string formatting uses Rust's shortest round-trip decimal conversion
with ECMAScript presentation rules. Primitive numeric operations have boundary
regressions. Exhaustive numerical and cross-platform conformance audits remain
part of the roadmap.

The resource limits bound specific work and value sizes. They do not provide a
complete memory budget or an operating-system sandbox. State is not rolled back
after runtime failure. Parsing and early errors occur before any execution.

## Intentionally outside scope

Node.js and browser APIs, CommonJS, ECMA-402, syntax extensions, and unstandardized
proposals are outside the target. Annex B's optional browser compatibility
extensions are not enabled for this non-browser host. They are standardized
optional behavior, not evidence that Node.js compatibility is required.
