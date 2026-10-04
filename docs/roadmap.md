# Roadmap

Track implementation of concrete specification behavior here. Each milestone
lands in small commits with tests. Record test execution and fixture inventories
in [coverage](coverage.md) and the [Test262 documentation](../tests/test262/README.md).
No milestone implies full conformance until the final audit passes.

## 0. Foundation

- [x] Specify scope, architecture, safety policy, and conformance criteria.
- [x] Create the workspace, shared lints, dependency checks, and GitHub Actions.
- [x] Add UTF-16 strings and source diagnostics.

## 1. First executable subset

- [x] Implement Number, String, Boolean, and null literals, comments, ASCII
  identifiers, core operators, and ASI for the supported statements.
- [x] Parse expression statements and lexical declarations with source spans.
- [x] Evaluate primitive expressions and lexical bindings with correct errors.
- [x] Add a command-line host.

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
- [x] Parse/evaluate synchronous for-of with assignment/var/lexical bindings, fresh iteration scopes, and iterator closing.
- [x] Parse/evaluate for-in over complete ordinary/exotic prototype chains with live descriptors and inherited-name suppression.
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
- [ ] Add remaining statements, catch patterns, and completions.
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
- [x] Add exact binary64-to-BigInt conversion for finite integral Numbers.
- [x] Integrate BigInt literals, values, coercions, and language operators.
- [x] Implement Unicode identifier properties and literal identifier names.
- [x] Implement identifier escapes and escape-aware reserved-word validation.

## 3. Objects and functions

- [x] Add the handle heap, explicit roots, tracing, and collection.
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
- [x] Parse/evaluate object spread with ordered CopyDataProperties and own data definitions.
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
- [x] Add identifier rest parameters with dense intrinsic Arrays, non-simple early errors, and unmapped arguments.
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
- [x] Connect String search predicates to IsRegExp's Symbol.match hook.
- [ ] Add IsRegExp's internal-brand fallback when RegExp objects are implemented.
- [x] Add ordinary-call String.raw and complete String constructor static own properties.
- [x] Add String split with UTF-16 boundaries, ToUint32 limits, and object Symbol.split delegation.
- [x] Add String replace with first-match searches, callbacks, object Symbol.replace hooks, and uncaptured substitution.
- [x] Add String replaceAll with non-overlapping matches, callbacks, uncaptured substitution, and global-flag checks before object hooks.
- [x] Add exact Number.prototype.toFixed rounding, argument order, and special cases.
- [x] Add exact Number.prototype.toPrecision with significant zeros and exponent correction.
- [x] Add Number.prototype.toExponential with shortest and explicit-digit formatting.
- [x] Add the documented ECMA-262 Number.prototype.toLocaleString fallback.
- [x] Add global parseFloat/parseInt and their shared Number aliases with exact prefix parsing.
- [x] Add global isFinite/isNaN with ordered ToNumber coercion and abrupt completion propagation.
- [x] Complete non-decimal Number formatting with exact shortest-roundtrip intervals.
- [x] Add BigInt calls, integral Number/string conversion, wrappers, and boxed receivers.
- [x] Add branded BigInt valueOf/toString/toLocaleString methods and observable tags.
- [x] Add BigInt.asIntN/asUintN width conversions.
- [x] Add the intrinsic Array values iterator hook to mapped and unmapped arguments.
- [x] Parse new expressions with optional arguments and constructor/member/call precedence.
- [x] Execute ordinary and bound construction with prototype selection and ordered arguments.
- [x] Add new.target early errors, call/construction bindings, and lexical arrow capture.
- [x] Implement instanceof for ordinary/bound functions and inherited default hooks.
- [x] Materialize Function.prototype Symbol.hasInstance and dispatch custom hooks.
- [x] Parse calls and evaluate callee/arguments in order, with correct non-callable TypeErrors.
- [x] Parse/evaluate iterable spread arguments for calls and construction with ordered argument accumulation.
- [x] Add builtin function objects and Object.prototype conversion methods.
- [x] Add Object construction and mandatory string-keyed prototype methods.
- [x] Add Object.defineProperty/getOwnPropertyDescriptor, hasOwn, and is.
- [x] Add Object prototype inspection/mutation and extensibility APIs.
- [x] Add Object.create and ordered two-phase Object.defineProperties.
- [x] Add Object.freeze/seal and frozen/sealed integrity predicates.
- [x] Add Object.assign and complete own-descriptor copying for supported objects.
- [x] Add Object own-name/symbol reflection and enumerable keys/values/entries.
- [x] Add Object.fromEntries with ordered entry reads, own data definitions, and iterator closing.
- [x] Add Object.groupBy with property-key groups, ordered callbacks, and iterator closing.
- [x] Complete required Object constructor properties and enable full own-key reflection/integrity operations.
- [x] Parse ordinary object methods/getters/setters with scoped early errors.
- [x] Execute object methods/accessors with non-constructible closures, names, and home-object tracing.
- [x] Add mandatory Function.prototype restricted accessors and %ThrowTypeError%.
- [x] Add Function.prototype call and native function source representation.
- [x] Add Function.prototype apply, ordered array-like arguments, and argument limits.
- [x] Add bound callable objects, capture tracing, and Function.prototype bind.
- [x] Bound re-entrant calls and dispatch builtin/bound tail transfers iteratively.
- [ ] Add arrays, destructuring, iteration, classes, and private elements.
- [x] Add sparse Array exotic storage, indexed growth, and partial length truncation.
- [x] Add ordered Realm-level ArraySetLength coercions and deferred length assignments.
- [x] Add Array calls/new and Array.isArray.
- [x] Parse/evaluate array literals with holes and trailing commas.
- [x] Add generic Array join and dynamic toString with ordered, bounded conversion.
- [x] Add generic Array at with relative indexing and ordered conversion.
- [x] Add generic Array push/pop with strict property operations and partial effects.
- [x] Add generic Array forEach/every/some with live presence checks and ordered callbacks.
- [x] Add Array find/findIndex/findLast/findLastIndex with ordered visits to holes.
- [x] Add Array includes/indexOf/lastIndexOf with distinct equality and hole handling.
- [x] Add Array reduce/reduceRight with optional accumulators and live sparse traversal.
- [x] Add generic Array reverse with sparse presence handling and ordered partial effects.
- [x] Add generic Array fill/copyWithin with ordered ranges and overlap handling.
- [x] Add generic Array shift/unshift with sparse movement and strict length writes.
- [x] Add Array toReversed/with copies with dense own elements and skipped replacement reads.
- [x] Add Array toSpliced with optional deletion ranges and skipped discarded reads.
- [x] Add Array.of with constructor dispatch, ordered data definitions, and strict length writes.
- [x] Add Array.from with iterable/array-like traversal, ordered mapping/construction, and iterator closing.
- [x] Parse/evaluate iterable Array literal spread with ordered consumption and length writes.
- [x] Add Array toLocaleString using the non-ECMA-402 algorithm.
- [x] Add stable Array sort/toSorted with bounded fallible merging and sparse writeback.
- [x] Add ArraySpeciesCreate and map/filter with ordered constructors, live sparse visits, and partial definitions.
- [x] Add sparse Array slice with ordered range conversion, species results, and final strict length writes.
- [x] Add sparse Array concat with species results, ordered spreadability hooks, and strict length writes.
- [x] Add Array flat/flatMap with species results, iterative sparse flattening, and ordered mapper calls.
- [x] Add Array splice with sparse species results, ordered moves/deletes, and strict length writes.
- [x] Complete Array prototype method materialization and enable full reflection/integrity operations.
- [ ] Implement symbols, coercion hooks, proxies, and Reflect.
- [x] Add the Reflect object, array-like apply calls, and construction with explicit newTarget and bound forwarding.
- [x] Add immutable symbol identities and distinct string/symbol property keys.
- [x] Integrate symbol keys into ordinary/exotic storage, ordering, and work budgets.
- [x] Add runtime Symbol primitives and shared well-known identities.
- [x] Integrate Realm symbol keys, reflection/enumeration, function names, and ToPrimitive hooks.
- [x] Add Object.prototype.toString Symbol.toStringTag lookup and bounded UTF-16 formatting.
- [x] Add Symbol wrappers, branded prototype methods, fresh intrinsic calls, and fixed well-known properties.
- [x] Add bounded Symbol.for/keyFor interning shared across realms and host threads.
- [x] Materialize Array's Symbol.species getter and Symbol.unscopables table.
- [x] Add Array keys/values/entries iterators with live state, reentrant next calls, and source tracing.
- [x] Add String iteration with immediate coercion, exact code-point boundaries, and bounded next steps.
- [x] Add synchronous iterator acquisition, cached next calls, stepping, and language-completion closing.
- [x] Add the shared Iterator tag getter and setter with receiver checks and strict own-property updates.
- [ ] Add remaining shared Iterator prototype properties.
- [x] Expose the Symbol global with standard attributes.
- [ ] Complete Array species-dependent methods and iterator integration.

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
- [ ] Review every specification section and record implementation evidence.
- [ ] Benchmark and optimize only after the conformance gate is stable.

Full conformance requires more than a passing subset or a test count. Every
mandatory baseline feature must have implementation and test evidence, with all
host choices and legitimate optional exclusions documented.
