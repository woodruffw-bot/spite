# Specification review

The baseline is [published ECMA-262 edition 17](https://262.ecma-international.org/17.0/),
with versioned Unicode data and the [non-browser host policies](design.md#host-policies).
The [source identity](../tests/test262/audit/specification.json) pins the exact
published source. Test262 provenance and scope are recorded in the
[audit index](../tests/test262/audit/README.md).

## Implementation map

| Clauses | Implementation and regression coverage |
| --- | --- |
| 1–5: scope, conformance and references | Host policies, source pins and explicit limitations |
| 6: language values and specification records | Core primitives, Runtime values, property keys, references and completions |
| 7: abstract operations | Conversion, comparison, descriptors, iteration, calls and integrity operations |
| 8: syntax-directed operations | Parser scope/containment checks, bound names, naming and tail positions |
| 9: contexts and environments | Realm/environment graphs, declaration instantiation and host jobs |
| 10: internal methods and objects | Ordinary/exotic dispatch, arguments, proxies and module namespaces |
| 11–12: source and lexical grammar | Lossless source, lexical goals, Unicode identifiers and strict legacy-literal rules |
| 13–14: expressions and statements | References, operators, binding patterns, control flow and completion propagation |
| 15: functions and classes | Function kinds, parameters, eval context, class construction and private elements |
| 16: Scripts and Modules | Instantiation, linking, live bindings, namespaces and async graph evaluation |
| 17–18: extensions and built-in conventions | Forbidden extensions, intrinsic graphs and native function metadata |
| 19–20: globals and fundamental objects | Eval, URI operations, Object/Function/Symbol and Error behavior |
| 21: numeric APIs and Date | BigInt/Number conversion, Math special values, time values and time zones |
| 22: text processing | UTF-16 String operations, Unicode transforms and RegExp protocols |
| 23: indexed collections | Array/TypedArray factories, iteration, species, copying and sorting |
| 24: keyed collections | Map/Set ordering, set-like records and weak-key operations |
| 25: structured data | Buffers, views, Atomics and JSON |
| 26–27: memory/control abstractions | Weak references, finalization, iterators, Promise and suspension |
| 28–29: reflection and memory model | Reflect/Proxy, shared storage, ordering and agent boundaries |
| Annexes A–F | Grammar cross-checks, optional browser features, strict mode, host hooks and compatibility notes |

The implementation lives under [crates](../crates); unit and integration tests
sit beside the corresponding abstractions. The tests protect observable order,
error ownership, strictness, source goals, lifetimes and host-failure separation,
as well as returned values.

## Qualification of the reviewed target

The complete pinned Test262 inventory has reviewed dispositions. Retained
originals execute with upstream metadata, unchanged source and harness code.
Scope records explain optional features, facilities absent from the published
baseline, Unicode-version differences, permitted representation choices and
native resource constraints. A scope partition is not a conformance percentage.

Syntax-negative tests require an exact reviewed Syntax diagnostic. Unsupported
features, limits and unverified diagnostics remain nonpassing. Runtime and
resolution negatives preserve their original phase and exception type.

## Host choices and limits

Normative optional Object.prototype accessor/helpers and Annex B extensions are
disabled. The required function poison accessors remain present; optional caller
reflection is disabled. WeakRef.prototype.constructor is enabled. Legacy clauses
that are not normative optional remain mandatory.

The host supplies module sources, supported attribute keys, jobs, time zones,
collection checkpoints and optional synchronous waiting. Locale behavior without
ECMA-402 follows the choices in [design](design.md#host-policies).

Quotas are opt-in. Native stack and allocator/platform capacity remain finite;
collection occurs at explicit idle checkpoints. Sources that exhaust those
resources retain nonpassing results in the [resource records](../tests/test262/audit/resource-stress.json).
Those records describe diagnostic conditions rather than universal algorithm bounds.

## Published-text inconsistencies

Unicode RegExp execution beginning inside a surrogate pair uses its leading
boundary for the match and captures. This is the chosen resolution of the
RegExpBuiltinExec inconsistency; see [RegExp design](design.md#regular-expressions).

ClassFieldDefinitionEvaluation creates an ordinary initializer function, while
the published FieldDefinition grammar forwards enclosing Yield/Await parameters.
Initializers here establish ordinary-function contexts; computed names preserve
the enclosing context. Upstream [issue 2437](https://github.com/tc39/ecma262/issues/2437)
and [issue 3333](https://github.com/tc39/ecma262/issues/3333) document the conflict.
The [class-field design](design.md#public-class-fields) records the interpretation
and reference-engine differences.

Confidence in the reviewed implementation is conditional on these host policies,
interpretations and native constraints. Passing tests do not remove those qualifications.
