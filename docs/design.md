# Design

## Scope

spite implements the published ECMA-262 edition 17 language for a non-browser
host. Unicode data follows the latest supported Unicode standard; source pins
and generated tables make that version explicit. The living specification and
later proposals are separate from the published baseline.

Annex B browser extensions are disabled. The normative optional
Object.prototype `__proto__` accessor and legacy accessor helpers are disabled;
object-literal prototype initializers remain required. WeakRef.prototype.constructor
is enabled. Optional function `caller`/`arguments` reflection is disabled, while
the required Function.prototype poison accessors remain present. A `Legacy`
label alone does not make a clause optional: non-strict legacy numeric literals
and string escapes remain supported.

The [specification review](spec-review.md) records clause coverage and known
interpretations. [Test262 scope records](../tests/test262/audit/README.md) explain
which upstream tests apply to this host. Existing later-proposal facilities are
extensions and provide no evidence of edition-17 conformance.

## Crates

| Crate | Responsibility |
| --- | --- |
| `spite-bigint` | Arbitrary-precision integer arithmetic and numeric conversion |
| `spite-heap` | Generational handles, roots, weak edges and tracing |
| `spite-core` | Source locations, diagnostics, UTF-16 strings, symbols, Unicode and shared language algorithms |
| `spite-parser` | Lexical goals, syntax trees, early errors and JSON grammar |
| `spite-runtime` | Values, environments, objects, built-ins, evaluation and host jobs |
| `spite` | Embedding facade and command-line host |
| `spite-test262` | Test metadata, harness loading, execution modes and outcome reporting |

Use separate crates for independently reusable abstractions. Keep code that
shares object identity, environments and execution state together in Runtime.
Public facade types are reexports rather than parallel implementations.

All workspace crates forbid unsafe Rust. Production dependencies are workspace
crates plus the approved RegExp and time-zone libraries. The standard library's
internal implementation is outside this prohibition. Prefer clear ownership and
borrowing; shared syntax, immutable strings and cross-agent backing storage use
reference counting where their lifetimes require it.

## Source and parsing

JavaScript strings are UTF-16 code-unit sequences, including lone surrogates.
Rust string conversion is fallible. Source ranges are byte offsets for UTF-8
input; dynamically compiled UTF-16 source uses the equivalent encoded lengths,
with lone surrogates occupying three bytes.

The handwritten lexer has explicit lexical goals. The parser controls division,
RegExp and template-substitution scanning through checkpoints. Strictness,
Script/Module goals, function kinds, private names and direct-eval context are
tracked separately. Legacy lexical metadata survives until the directive
prologue determines strictness.

Expressions use binding powers. Flat binary/comma chains, graph traversals and
JSON trees use iterative representations where native recursion would be
unnecessary. Recursive grammar and AST shapes retain native-stack checks.
Early errors are validated before execution, including dynamically compiled
functions and eval source.

Syntax diagnostics, unsupported features and resource failures are distinct.
Unsupported syntax cannot satisfy a negative test expecting SyntaxError.

## Values, objects and environments

Runtime values represent ECMAScript primitives and checked object handles.
Property keys distinguish UTF-16 strings from symbol identities. Descriptors
preserve absent fields separately from explicit undefined. Ordinary and exotic
internal methods share descriptor validation and observable conversion order.

Objects and environments occupy the same generational heap. Handles carry heap
identity and generation, so foreign and stale references fail validation.
References retain their resolved environment, receiver and property key across
side effects; writes do not repeat name resolution.

The environment graph represents lexical/variable scopes, private names,
imports, this/new.target and method home objects. Direct eval inherits only the
specified caller context. Foreign-Realm calls preserve creation-Realm intrinsics
and restore the caller after language and host failures.

Completions distinguish empty values, undefined, normal results and abrupt
control transfers. Statement lists, loops and finally clauses apply the
specification's completion rules rather than using Rust errors for JavaScript
control flow. Host failures bypass JavaScript catch/finally handling.

## Memory and execution limits

The embedding host roots values that must survive collection. Collection and
kept-object checkpoints run at idle boundaries; native evaluation frames are
not implicitly scanned. Weak references, ephemerons, registry cleanup jobs,
module records and suspended continuations participate in tracing.

Collection is explicit. Long uninterrupted Scripts can retain substantial
storage until the next checkpoint. Unrooted handles must not be retained across
collection. Finalization callbacks run as host jobs, separately from tracing.

Every public `Limits` quota defaults to None. Work, source size, string length,
BigInt size, heap slots, object properties and argument counts can be bounded by
the embedding host. Exhaustion returns a host Limit, not a JavaScript exception.
Work accounting preserves conversion order and distinguishes interrupted searches
from completed negative results.

Supported threads provide at least a 2 MiB native stack. The call/construct guard
is 32 levels; the parser additionally guards recursive grammar, expression-tree
and template depth. Platform/allocator capacity checks remain active with quotas
disabled. These constraints and the nonpassing stress cases are documented in
[coverage](coverage.md) and the [Test262 audit](../tests/test262/audit/staging.md).

## Jobs, modules and agents

An agent owns a heap, Realm records, kept objects and FIFO jobs. Hosts explicitly
advance Promise, timeout and cleanup jobs. Promise rejection tracking defaults
to a no-op; a rejected Promise retains its ordinary state. Suspended async
functions, generators and module evaluations keep traced continuation state.

Module loading, supported import-attribute keys and source identities are host
choices. Linking and evaluation retain live bindings, namespaces, graph cycles
and cached evaluation Promises. Dynamic import uses the same graph machinery;
loader failures remain separate from JavaScript rejection values. import.meta
starts as an empty null-prototype object unless the host supplies properties.

Independently constructed Realms are separate execution domains. Test262 workers
create isolated agents and share only the supported host messages and shared
backing buffers. Shared buffers use safe Rust storage and native atomic memory
steps. Shared growth is monotonic; ordinary buffer resize/detachment invalidates
view witnesses according to the relevant internal method.

## Regular expressions

The regress backend matches the complete original UTF-16 input. Ordinary mode
uses code units; Unicode modes decode valid pairs and preserve lone surrogates.
The adapter retains original source, flags, capture ranges and undefined captures.
It never slices input to simulate anchors, boundaries or lookaround.

Runtime owns RegExp objects, lastIndex, result construction and the live
RegExpExec boundary used by String methods. The backend reports optional work
exhaustion separately from match/no-match results. Unicode tables are pinned
independently of the Rust compiler's Unicode version.

Unicode execution starting inside a surrogate pair uses the pair's leading
boundary for the whole match and captures. This is the explicitly chosen
resolution of the edition-17 RegExpBuiltinExec inconsistency. See
[vendor patches](../vendor/regress/SPITE-PATCHES.md) for the adapter's backend
requirements and maintained changes.

## Public class fields

Computed names run in the enclosing context before static initialization.
Field initializers run in their own strict ordinary-function environments with
initialized this, undefined new.target, a method home object and the field name
for NamedEvaluation. Instance fields initialize in construction order; static
fields and blocks run in source order after class-name initialization.

The published FieldDefinition grammar forwards Yield/Await to Initializer,
although ClassFieldDefinitionEvaluation creates an ordinary function. The
implementation resets initializer suspension contexts while preserving them
for computed names. Script initializers can read an identifier named `await`;
Module code still reserves it. Nested async/generator functions establish their
own contexts. Direct suspension expressions are rejected before evaluation.

This grammar/runtime conflict is documented in upstream
[issue 2437](https://github.com/tc39/ecma262/issues/2437) and
[issue 3333](https://github.com/tc39/ecma262/issues/3333). Reference engines differ
on direct static-field `await` identifiers, including field-order-dependent
behavior. The same ordinary-function interpretation applies to public/private
and instance/static fields here.

## Host policies

| Facility | Policy |
| --- | --- |
| String compilation and source availability | Enabled, subject to explicit quotas and native capacity |
| Private element installation | Default host hook; nonextensibility does not prevent new private elements |
| Job callbacks and Promise reactions | Preserve callback identity, creation Realm and FIFO order |
| Finalization | Explicit host collection/checkpoints and queued cleanup jobs |
| Module loading and import attributes | Explicit embedding loader; supported attribute keys fixed at Realm creation |
| Buffer resizing and shared growth | Checked sparse backing storage; serialized monotonic shared growth |
| Synchronous waits | Ordinary agents cannot suspend; an explicit suspending host enables waits |
| DebuggerStatement | Normal empty completion without an active debugger |
| Math.random | Distinct per-Realm pseudo-random sequences seeded by the process |
| Time zones | Host/system selection with pinned IANA fallback data |
| Locale behavior without ECMA-402 | Ordinary numeric formatting; local Date formats; NFD UTF-16 ordering for locale comparison |
| Non-standard date parsing | Documented supported formats; other non-ISO fallback forms can return NaN |

## Command-line host

The CLI accepts UTF-8 Script or Module input. Script mode prints the completion
and drains queued jobs; an idle pending Promise does not extend the turn. Module
mode waits for top-level evaluation and reports rejection or an idle unsettled
result as a failure. `--check` validates only the supplied source and early errors.

Static and dynamic imports resolve literal filesystem paths relative to their
referrer. Canonical path/type pairs identify cached modules. The `type` import
attribute accepts JSON explicitly; file suffixes do not select the parser.
Filesystem, encoding and unsupported-host failures remain host errors.

Source and work limits are opt-in CLI options. Bounded readers consume at most
one byte beyond an explicit source limit. Error reporting inspects stored data
properties without invoking accessors, Proxy traps or coercion hooks.

## Development

Follow [Google's Rust style guide](https://google.github.io/styleguide/rustguide.html)
and the Rust API Guidelines. This Cargo repository uses conventional Cargo
layout; Chromium-specific flat GN layouts and test frameworks do not apply.
Use rustfmt's stable formatting, explicit imports, leaf modules with focused
responsibilities, descriptive names and short control flow. Use context-bearing
assertions and reserve panics for documented invariants.

Tests should state behavior directly, share helpers where useful and cover
observable semantics, error order and lifetimes. Keep generated datasets and
unchanged upstream tests distinct from hand-written regressions. Development
commands belong in the [README](../README.md); benchmark methodology is in
[benchmarking](benchmarking.md). Historical measurements and run logs do not
belong in design documentation.
