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
| Expressions | Primitive literals, untagged templates with substitutions, identifiers, parentheses, simple and compound assignment, prefix/postfix updates, conditional and comma expressions |
| Operators | Arithmetic, exponentiation, bitwise, shifts, primitive equality and comparison, logical and nullish operators, typeof, void, and delete |
| Statements | Empty and expression statements, let, const, and var, blocks, if/else, while, do-while, for with expression, lexical, or var headers, switch, labels, break/continue with optional targets, throw, try with catch without a parameter and/or finally |
| Static semantics | Implemented ASI rules, strict directives, duplicate lexical bindings, strict binding and assignment restrictions, escaped reserved-word checks, enclosing-loop/switch checks, duplicate labels, control-target validation, duplicate defaults and case-block lexical names |
| Runtime | Persistent realm state, lexical scope, declaration instantiation, per-iteration let scopes, temporal dead zones, immutable bindings, ordered evaluation |
| Completions | Empty versus undefined, statement-list values, if-statement UpdateEmpty, loop body values, break/continue propagation through blocks, conditionals, nested loops, and switch fall-through, primitive throws, finalizer preservation and overrides of language completions |
| Global values | undefined, NaN, Infinity, and simple sloppy-mode global assignment |
| Limits | 1 MiB source, depth 64, configurable evaluation steps and string code-unit limits |
| Tests | Algorithm and integration tests, AST and diagnostic snapshots, 11 pinned Test262 hashbang fixtures, 13 identifier lexer fixtures, and 6 statement parser fixtures |

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

The `spite-bigint` crate provides integer storage, radix conversion, and signed
addition, subtraction, multiplication, division, shifts, bitwise operations, and
exponentiation. The parser preserves exact BigInt digits and radices; evaluation
still reports them as unsupported. JavaScript BigInt values
are not integrated yet.

BigInt, Symbols, objects, properties,
arrays, functions, calls, closures, constructors, classes, destructuring, regular
expressions, tagged templates, for-in/of, catch parameters, generators,
async functions, promises, modules, standard library objects, eval, agents, shared
memory, and garbage collection remain open. See the roadmap for their order.

Global lexical bindings and Script var declarations persist between evaluations.
New global vars are non-deletable. A var declaration for an existing global value
preserves its value and deletability. There is no observable global object or
full property model yet. Built-in error categories are represented in
Rust. Catch clauses without a parameter handle language throws and built-in
exceptions. JavaScript Error objects and catch parameters are future work.

Recognized missing features return Unsupported. Because the grammar is incomplete,
a syntax diagnostic alone does not prove arbitrary input violates ECMA-262. The
current parser must not be used to score general Test262 negative tests.

The Test262 regression fixtures use a reviewed manifest and match the specific
rejection point for each negative fixture. Identifier component tests compare
literal and escaped spellings at the lexer boundary. Statement parser fixtures
check contextual `let` lookahead and ASI. Neither component group is counted as
Script execution passes. The fixture suite does not run the general harness,
parse arbitrary Test262 YAML, report a whole-suite pass rate, or cover all of the
implemented semantics. A complete runner is a separate roadmap item.

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
