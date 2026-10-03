# Roadmap

Each milestone lands in small commits with tests. Update this file with actual
coverage. No milestone implies full conformance until the final audit passes.

## 0. Foundation

- [x] Specify scope, architecture, safety policy, and conformance criteria.
- [x] Create the workspace, shared lints, dependency checks, and GitHub Actions.
- [x] Add UTF-16 strings and source diagnostics.

## 1. First executable subset

- [x] Implement Number, String, Boolean, and null literals, comments, ASCII
  identifiers, core operators, and ASI for the supported statements.
- [x] Parse expression statements and lexical declarations with source spans.
- [x] Evaluate primitive expressions and lexical bindings with correct errors.
- [x] Add parser snapshots, runtime integration tests, and a command-line host.
- [x] Bound parser nesting, evaluation steps, and individual string lengths.
- [x] Add a pinned Test262 lexical smoke suite with reviewed expectations.

Milestones 0 and 1 are implemented. See [coverage](coverage.md) for exact limits.

This slice is deliberately incomplete. Unsupported syntax and operations must not
silently acquire substitute semantics. The coverage document must list gaps.

## 2. Grammar and execution core

- [ ] Complete lexical goals, templates, RegExp literals, and numeric literals.
- [x] Add while and do-while loops, including completion values and ASI.
- [x] Add unlabelled break and continue with completion propagation and early errors.
- [x] Add labelled statements, labelled control transfers, and label early errors.
- [x] Add expression-header for loops and their completion semantics.
- [x] Add lexical for-loop declarations and per-iteration environments.
- [x] Parse switch clauses and validate their lexical scope and control flow.
- [x] Evaluate switch selection, fall-through, and completion values.
- [ ] Add remaining statements, for-in/of loops, try, and completions.
- [ ] Complete strict mode, declarations, early errors, and reference semantics.
- [ ] Add arbitrary-precision BigInt and spec-correct numeric conversions.
- [x] Implement Unicode identifier properties and literal identifier names.
- [x] Implement identifier escapes and escape-aware reserved-word validation.
- [ ] Establish a pinned Test262 runner with strict modes and phase-aware results.

## 3. Objects and functions

- [ ] Add the handle heap, explicit roots, tracing, and collection tests.
- [ ] Implement ordinary objects, descriptors, prototypes, and property order.
- [ ] Add functions, closures, this, arguments, constructors, and direct eval.
- [ ] Add arrays, destructuring, iteration, classes, and private elements.
- [ ] Implement symbols, coercion hooks, proxies, and Reflect.
- [ ] Run the upstream Test262 harness and grow a supported regression set.

## 4. Standard library

- [ ] Complete Object, Function, Boolean, Number, BigInt, String, Symbol, and Math.
- [ ] Add collections, JSON, errors, and iterator helpers.
- [ ] Implement the ECMAScript RegExp grammar and matching semantics.
- [ ] Add Date, binary buffers, typed arrays, and DataView.
- [ ] Add weak collections, WeakRef, and FinalizationRegistry with valid GC behavior.
- [ ] Audit all edition-17 intrinsics against the specification inventory.

## 5. Modules and suspended execution

- [ ] Add module parsing, linking, namespace objects, live bindings, and cycles.
- [ ] Add generators, promises, async functions, async iteration, and job handling.
- [ ] Implement dynamic import, import.meta, and top-level await through host hooks.
- [ ] Implement explicit resource management where required by the baseline.

## 6. Completion and hardening

- [ ] Implement agents, SharedArrayBuffer, Atomics, and memory-model requirements.
- [ ] Audit normative optional features and host-defined behavior.
- [ ] Run the complete in-scope pinned Test262 suite without hidden exclusions.
- [ ] Add grammar fuzzing, resource-limit tests, and cross-platform validation.
- [ ] Review every specification section and record implementation evidence.
- [ ] Benchmark and optimize only after the conformance gate is stable.

Full conformance requires more than a passing subset or a test count. Every
mandatory baseline feature must have implementation and test evidence, with all
host choices and legitimate optional exclusions documented.
