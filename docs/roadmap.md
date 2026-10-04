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
- [x] Parse untagged template literals with cooked/raw text and nested substitutions.
- [x] Evaluate untagged template substitutions and string conversion.
- [x] Implement edition-17 legacy numeric/string literals and strict lexical early errors.
- [x] Add while and do-while loops, including completion values and ASI.
- [x] Add unlabelled break and continue with completion propagation and early errors.
- [x] Add labelled statements, labelled control transfers, and label early errors.
- [x] Add expression-header for loops and their completion semantics.
- [x] Add lexical for-loop declarations and per-iteration environments.
- [x] Parse switch clauses and validate their lexical scope and control flow.
- [x] Evaluate switch selection, fall-through, and completion values.
- [x] Parse try-finally blocks and validate their scopes and control targets.
- [x] Evaluate finalizers and their normal and abrupt completion overrides.
- [x] Implement debugger statements with no active debugging facility.
- [x] Handle optional-binding catch clauses and catch-finally completions.
- [x] Parse catch binding identifiers and validate catch scope conflicts.
- [x] Execute catch binding identifiers for implemented thrown values, including objects.
- [x] Bind built-in exceptions as JavaScript Error objects in catch clauses.
- [x] Add Error/NativeError constructors, causes, ErrorData identity, and standard methods.
- [ ] Add remaining statements, for-in/of loops, catch parameters, and completions.
- [ ] Complete strict mode, declarations, early errors, and reference semantics.
- [x] Add prefix and postfix updates with reference and line-terminator rules.
- [x] Add arithmetic, bitwise, shift, and logical compound assignments.
- [x] Implement delete for environment references and non-reference expressions.
- [x] Parse var declarations and validate lexical conflicts through nested statements.
- [x] Instantiate Script vars and execute variable statements and for headers.
- [x] Add arbitrary-precision BigInt and primitive numeric conversions.
- [x] Add normalized integer storage, radix conversion, signed addition, and work budgets.
- [x] Add integer multiplication and truncating division with signed remainders.
- [x] Add integer shifts, bitwise operations, and exponentiation.
- [x] Parse exact BigInt literals, separators, and numeric-token boundaries.
- [x] Compare arbitrary integers with binary64 values without rounding.
- [x] Add correctly rounded BigInt-to-binary64 conversion for the Number constructor.
- [x] Integrate BigInt literals, values, coercions, and language operators.
- [x] Implement Unicode identifier properties and literal identifier names.
- [x] Implement identifier escapes and escape-aware reserved-word validation.
- [x] Establish a pinned Test262 runner with strict modes and phase-aware results.
- [x] Parse Test262 execution metadata and plan modes and ordered harness includes.
- [x] Classify Test262 phases and keep unsupported, host-limit, and unverified results distinct.
- [x] Match runtime-negative constructor names for built-in exceptions, explicit Errors, and rethrows.
- [x] Run a reviewed pinned corpus through the runner in CI.

## 3. Objects and functions

- [x] Add the handle heap, explicit roots, tracing, and collection tests.
- [x] Add bounded generational storage with checked heap identity and slot reuse.
- [x] Add iterative tracing with explicit roots and failure-before-sweep guarantees.
- [ ] Root interpreter temporaries, environments, intrinsics, and host-held object values.
- [x] Add checked host-root tokens with clone/drop lifetimes and bounded registry reuse.
- [x] Add object identity values and trace object-valued data properties, including cycles.
- [ ] Implement ordinary objects, descriptors, prototypes, and property order.
- [x] Add ordinary data-property records, descriptor invariants, deletion, and string-key order.
- [x] Add accessor descriptors, kind transitions, callable validation/tracing, and receiver-aware calls.
- [x] Implement prototype traversal and mutation with bounded cycle checks.
- [x] Implement inherited data reads, presence checks, and receiver-sensitive writes.
- [x] Parse object literals, computed keys, shorthand, and duplicate prototype-setter early errors.
- [x] Evaluate object literals and integrate the object heap with realms.
- [x] Collect between evaluations with persistent bindings, intrinsic handles, and host roots.
- [x] Parse dotted/computed property references and preserve assignment targets in the AST.
- [x] Evaluate ordinary-object property reads, writes, updates, and deletion with ordered references.
- [x] Add the in operator and context-sensitive In grammar for for initializers.
- [x] Implement primitive property access, String indices/length, and strict write/delete behavior.
- [ ] Implement object coercion hooks and primitive wrapper constructors/prototypes.
- [x] Route coercion through realms and reject ordinary objects without callable conversion methods.
- [ ] Add functions, closures, this, arguments, constructors, and direct eval.
- [x] Store declarative environments in the traced heap with stable identity and outer links.
- [x] Parse expression-bodied arrows with simple parameters, early errors, and exact source text.
- [x] Execute arrow closures with shared bindings, names, metadata, and source stringification.
- [x] Add arrow block bodies, function-local declarations, strict directives, and return completions.
- [x] Add default arrow parameters, ordered initialization, TDZ, and separate body var environments.
- [x] Parse ordinary function expressions with shared parameters/bodies and function-specific early errors.
- [x] Parse ordinary function declarations and validate Script/function versus block scope.
- [x] Instantiate ordinary functions with metadata, prototype cycles, named-expression scopes, and declaration hoisting.
- [x] Execute strict ordinary calls with unmapped arguments, preserved receivers, and arrow this capture.
- [x] Execute non-strict calls with object receivers and mapped arguments, including alias detachment.
- [x] Add the ordinary global object, property-backed global bindings, globalThis, and nullish/global receivers.
- [x] Add Boolean calls/construction, prototype methods, wrappers, and Boolean boxed receivers.
- [x] Add Number calls/construction, wrappers, constants, predicates, and decimal formatting.
- [x] Add String calls/construction, UTF-16 indexed wrappers, branded methods, and boxed receivers.
- [x] Add String code-unit/code-point constructors and character access methods.
- [x] Add Unicode well-formedness checks and replacement of unpaired String surrogates.
- [x] Add generic String concatenation and UTF-16 slice/substring methods.
- [x] Add String trim/trimStart/trimEnd with exact ECMAScript whitespace membership.
- [x] Add String repeat/padStart/padEnd with ordered conversions and bounded output.
- [x] Add bounded UTF-16 String indexOf/lastIndexOf searches.
- [x] Add String includes/startsWith/endsWith for currently exposed values.
- [x] Connect String search predicates to IsRegExp's Symbol.match hook with native-injection tests.
- [ ] Add IsRegExp's internal-brand fallback when RegExp objects are implemented.
- [x] Add ordinary-call String.raw and complete String constructor static own properties.
- [x] Add exact Number.prototype.toFixed rounding, argument order, and special cases.
- [x] Add exact Number.prototype.toPrecision with significant zeros and exponent correction.
- [x] Add Number.prototype.toExponential with shortest and explicit-digit formatting.
- [x] Add the documented ECMA-262 Number.prototype.toLocaleString fallback.
- [x] Add global parseFloat/parseInt and their shared Number aliases with exact prefix parsing.
- [x] Add global isFinite/isNaN with ordered ToNumber coercion and abrupt completion propagation.
- [x] Complete non-decimal Number formatting with exact shortest-roundtrip intervals.
- [ ] Add remaining BigInt boxed receivers.
- [x] Add the intrinsic Array values iterator hook to mapped and unmapped arguments.
- [x] Parse new expressions with optional arguments and constructor/member/call precedence.
- [x] Execute ordinary and bound construction with prototype selection and ordered arguments.
- [x] Add new.target early errors, call/construction bindings, and lexical arrow capture.
- [x] Implement instanceof for ordinary/bound functions and inherited default hooks.
- [x] Materialize Function.prototype Symbol.hasInstance and test custom hooks with native-injected symbols.
- [x] Parse calls and evaluate callee/arguments in order, with correct non-callable TypeErrors.
- [x] Add builtin function objects and Object.prototype conversion methods.
- [x] Add Object construction and mandatory string-keyed prototype methods.
- [x] Add Object.defineProperty/getOwnPropertyDescriptor, hasOwn, and is.
- [x] Add Object prototype inspection/mutation and extensibility APIs.
- [x] Add Object.create and ordered two-phase Object.defineProperties.
- [x] Add Object.freeze/seal and frozen/sealed integrity predicates.
- [x] Add Object.assign and complete own-descriptor copying for supported objects.
- [x] Add mandatory Function.prototype restricted accessors and %ThrowTypeError%.
- [x] Add Function.prototype call and native function source representation.
- [x] Add Function.prototype apply, ordered array-like arguments, and argument limits.
- [x] Add bound callable objects, capture tracing, and Function.prototype bind.
- [x] Bound re-entrant calls and dispatch builtin/bound tail transfers iteratively.
- [ ] Add arrays, destructuring, iteration, classes, and private elements.
- [x] Add sparse Array exotic storage, indexed growth, and partial length truncation.
- [x] Add ordered Realm-level ArraySetLength coercions and deferred length assignments.
- [x] Add Array calls/new and Array.isArray.
- [x] Parse/evaluate array literals with holes and trailing commas; defer spread to iteration.
- [x] Add generic Array join and dynamic toString with ordered, bounded conversion.
- [x] Execute 65 reviewed Array construction, identity, literal, length, and string-conversion Test262 files.
- [x] Add generic Array at with relative indexing, ordered conversion, and six Test262 files.
- [x] Add generic Array push/pop with strict property operations, partial effects, and 14 Test262 files.
- [x] Add generic Array forEach/every/some with live presence checks, ordered callbacks, and 42 Test262 files.
- [x] Add Array find/findIndex/findLast/findLastIndex with ordered visits to holes and 46 Test262 files.
- [x] Add Array includes/indexOf/lastIndexOf with distinct equality and hole handling, plus 26 Test262 files.
- [x] Add Array reduce/reduceRight with optional accumulators, live sparse traversal, and 34 Test262 files.
- [x] Add generic Array reverse with sparse presence handling, ordered partial effects, and 11 Test262 files.
- [x] Add generic Array fill/copyWithin with ordered ranges, overlap handling, and 20 Test262 files.
- [x] Add generic Array shift/unshift with sparse movement, strict length writes, and 14 Test262 files.
- [x] Add Array toReversed/with copies, 19 Test262 files, and the pinned compareArray compatibility include.
- [x] Add Array toSpliced with optional deletion ranges, skipped discarded reads, and 19 Test262 files.
- [x] Add Array.of with constructor dispatch, ordered data definitions, strict length writes, and eight Test262 files.
- [x] Add Array toLocaleString using the non-ECMA-402 algorithm and four Test262 files.
- [x] Add stable Array sort/toSorted with bounded fallible merging, sparse writeback, and 33 Test262 files.
- [ ] Add remaining Array prototype methods and expand reviewed array/harness conformance coverage.
- [ ] Implement symbols, coercion hooks, proxies, and Reflect.
- [x] Add immutable symbol identities and distinct string/symbol property keys.
- [x] Integrate symbol keys into ordinary/exotic storage, ordering, and work budgets.
- [x] Add runtime Symbol primitives and shared well-known identities with native-injection tests.
- [x] Integrate Realm symbol keys, reflection/enumeration, function names, and ToPrimitive hooks.
- [x] Add Object.prototype.toString Symbol.toStringTag lookup and bounded UTF-16 formatting.
- [x] Add Symbol wrappers, branded prototype methods, fresh intrinsic calls, and fixed well-known properties.
- [x] Add bounded Symbol.for/keyFor interning shared across realms and host threads.
- [x] Materialize Array's Symbol.species getter and Symbol.unscopables table.
- [x] Add Array keys/values/entries iterators, live/reentrant state tests, source tracing, and nine Test262 files.
- [ ] Add String iteration and remaining shared Iterator prototype properties.
- [ ] Expose the Symbol global after completing remaining intrinsic symbol properties.
- [ ] Complete Array species-dependent methods and iterator integration.
- [x] Execute pinned assert.js/sta.js and positive Test262 function/capture regressions.
- [ ] Complete remaining harness paths/includes as their language and library dependencies arrive.

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
