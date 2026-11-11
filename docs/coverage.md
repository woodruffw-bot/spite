# Coverage and implementation constraints

spite targets published ECMA-262 edition 17 for a non-browser host. The
[clause review](spec-review.md) maps specification areas to implementation and
tests. Test totals are reported by the test tools rather than maintained here.

## Test layers

- Crate unit tests cover primitive algorithms, storage invariants and internal
  state transitions.
- Parser snapshots cover syntax trees and exact diagnostics across Script,
  Module, eval and dynamic-function contexts.
- Runtime regressions cover language behavior, conversion/error order, Realm
  ownership, suspension, collection and explicit resource budgets.
- The [Test262 runner](../tests/test262/README.md) executes unchanged upstream
  originals with their declared modes and harness files.
- [Scope and provenance records](../tests/test262/audit/README.md) account for
  unretained originals independently of whether they pass.

A passing corpus does not prove unqualified conformance. Unsupported syntax,
unverified negative diagnostics, metadata failures, host failures and resource
limits remain nonpassing outcomes. Optional features and post-baseline proposals
are identified explicitly in the scope records.

## Implemented areas

| Area | Coverage |
| --- | --- |
| Source and syntax | UTF-8/UTF-16, lexical goals, Unicode identifiers, strictness, early errors and source spans |
| Values and operators | Number, BigInt, strings, symbols, coercion, references and exact completion propagation |
| Functions and classes | Calls/construction, closures, eval, private elements, fields, static blocks and generator/async forms |
| Objects and collections | Descriptors, ordinary/exotic methods, Proxy/Reflect, arrays, Map/Set and weak collections |
| Text and numeric APIs | RegExp protocols, Unicode casing/normalization, numeric conversion, Math and Date |
| Structured data | JSON, buffers, DataView, typed arrays and byte encodings |
| Execution | Realms, agents, jobs, modules, async iteration and shared-memory operations |

The specification review and source tests provide the detailed evidence for each
area; this table is not a separate compatibility contract.

## Intentional host choices

Annex B and the normative optional Object.prototype accessor/helpers are disabled.
WeakRef.prototype.constructor is enabled. Optional function caller-chain reflection
is disabled, while required poison accessors remain present. ECMA-402, browser
APIs and Node.js compatibility are outside the target. Existing later-proposal
facilities provide no edition-17 conformance credit.

Module loading, supported import attributes, time-zone selection, job advancement,
collection and synchronous waiting are embedding responsibilities. The
[host policy table](design.md#host-policies) records defaults.

## Native constraints

Host quotas are opt-in; every `Limits` field defaults to None. Allocation,
platform capacity and native-stack checks still apply. Hosts must provide at
least a 2 MiB native thread stack. Calls/construction share a 32-level guard;
recursive parser and AST forms have separate stack guards. Flat expression
chains and selected graph/tree operations use iterative representations.

Collection and kept-object checkpoints occur at explicit idle boundaries.
Uninterrupted Scripts can retain large heaps until the host reaches a checkpoint.
Sparse buffers avoid materializing untouched zero regions, so dense-allocation
failure assumptions need not hold for their logical lengths.

[Resource-stress records](../tests/test262/audit/resource-stress.json) preserve
nonpassing outcomes for sources that exceed native stack or allocation capacity.
External diagnostic memory ceilings are separate from Runtime defaults. No
resource failure counts as a JavaScript exception or a passing negative test.

## Specification interpretations

- Unicode RegExp execution beginning inside a surrogate pair uses the leading
  boundary consistently for the match and captures.
- Class field initializers use an ordinary-function suspension context, while
  computed names retain the enclosing context.

The [RegExp](design.md#regular-expressions) and
[class-field](design.md#public-class-fields) design sections explain these
published-text inconsistencies and the selected behavior. Conformance confidence
is conditional on these interpretations and the native constraints above.
