# Architecture

This guide describes how the interpreter is organized. For supported host
behavior and native resource constraints, see [compatibility](compatibility.md).
For the public API and host lifecycle, see [embedding](embedding.md).

## Crates

All workspace members live under `crates/`.

| Crate | Responsibility |
| --- | --- |
| `spite-bigint` | Arbitrary-precision integer arithmetic and numeric conversion |
| `spite-heap` | Generational handles, roots, weak edges, and tracing |
| `spite-core` | Source locations, diagnostics, UTF-16 strings, symbols, and shared language algorithms |
| `spite-unicode` | Pinned character properties, casing, normalization, and case folding |
| `spite-regexp` | Pattern validation, metadata, UTF-16 matching, escaping, and replacement syntax |
| `spite-parser` | Lexical goals, syntax trees, early errors, and JSON grammar |
| `spite-runtime` | Values, environments, objects, built-ins, evaluation, and host jobs |
| `spite` | Embedding facade and command-line host |
| `spite-test262` | Test metadata, harness loading, execution modes, and outcome reporting |
| `xtask` | Development-only integrity checks and data maintenance |

Reusable storage, numeric, Unicode, and pattern abstractions have independent
crates. Code that shares object identity, environments, and execution state stays
in `spite-runtime`. The facade re-exports public types.

The parser separates grammar state, source entry points, directives, operators,
and early-error validation. The runtime separates realm storage and host entry
points from conversions, declarations, statements, expressions, references, and
completion records. Crate roots declare modules and expose their APIs.

All workspace crates forbid unsafe Rust. Shared syntax, immutable strings, and
cross-agent backing storage use reference counting where their lifetimes require
it. Development conventions and dependency rules are in the
[development guide](development.md) and [AGENTS.md](../AGENTS.md).

## Source and parsing

JavaScript strings are UTF-16 code-unit sequences, including lone surrogates.
Converting one to a Rust string is fallible. Source ranges are byte offsets for
UTF-8 input; dynamically compiled UTF-16 source uses equivalent encoded lengths,
with lone surrogates occupying three bytes.

The handwritten lexer has explicit lexical goals. The parser controls division,
RegExp, and template-substitution scanning through checkpoints. Strictness,
script/module goals, function kinds, private names, and direct-eval context are
tracked separately. Legacy lexical metadata survives until the directive
prologue determines strictness.

Expression parsing uses binding powers. Flat binary and comma chains, selected
graph traversals, and JSON trees use iterative representations to avoid
unnecessary native recursion. Recursive grammar and AST shapes retain stack
guards. Early errors are validated before execution, including for dynamically
compiled functions and eval source.

Syntax diagnostics, unsupported features, and resource failures are distinct.
Unsupported syntax cannot satisfy a negative test expecting a `SyntaxError`.

## Values, objects, and environments

Runtime values represent ECMAScript primitives and checked object handles.
Property keys distinguish UTF-16 strings from symbol identities. Descriptors
preserve absent fields separately from explicit `undefined`. Ordinary and exotic
internal methods share descriptor validation while preserving observable
conversion order.

Objects and environments occupy the same generational heap. Handles carry heap
identity and generation, so foreign and stale references fail validation.
References retain their resolved environment, receiver, and property key across
side effects; writes do not repeat name resolution.

The environment graph represents lexical and variable scopes, private names,
imports, `this`, `new.target`, and method home objects. Direct eval inherits only
the specified caller context. Cross-realm calls use the function's creation-realm
intrinsics and restore the caller after language exceptions or host failures.

Completion records distinguish empty values, `undefined`, normal results, and
abrupt control transfers. Statement lists, loops, and `finally` clauses apply the
specification's completion rules. Host failures leave evaluation directly rather
than following JavaScript `catch` or `finally` control flow.

## Memory and resource accounting

The heap traces roots, strong edges, weak edges, and ephemerons. Module records,
suspended continuations, and queued jobs retain their reachable values. The
embedding host registers roots for values it needs to keep across collection;
native evaluation frames are not implicitly scanned.

Collection and kept-object checkpoints run only while execution is idle.
Finalization callbacks run as separate host jobs after tracing has queued them.
The [embedding guide](embedding.md#rooting-values-and-collecting-garbage)
describes the lifecycle and root-token API.

Optional work accounting includes interpreter and built-in operations. It must
preserve conversion order and distinguish an interrupted search from a completed
negative result. Exhaustion returns a host limit error rather than a JavaScript
exception. Stack and platform checks remain active independently of host quotas;
their current bounds are documented under
[native constraints](compatibility.md#native-constraints).

## Jobs, modules, and agents

An agent owns a heap, realm records, kept objects, and FIFO jobs. Hosts explicitly
advance Promise, timeout, and cleanup jobs. Suspended async functions, generators,
and module evaluations retain traced continuation state.

The host supplies module sources and their identities. Linking and evaluation
preserve live bindings, namespaces, graph cycles, and cached evaluation Promises.
Dynamic import uses the same graph machinery. A JavaScript rejection remains
distinct from a loader's host failure.

Independently constructed realms have separate heaps and execution state.
Test262 agents exchange host messages and shared backing buffers rather than
object handles. Shared buffers use safe Rust storage and native atomic memory
operations. Shared growth is monotonic; ordinary buffer resize and detachment
invalidate view witnesses according to the relevant internal method.

## Regular expressions

`spite-regexp` owns pattern validation, capture metadata, and the regress adapter.
The backend matches the complete original UTF-16 input. Ordinary mode uses code
units; Unicode modes decode valid surrogate pairs and preserve lone surrogates.
The adapter retains original source, flags, capture ranges, and unmatched
captures. It does not slice input to simulate anchors, boundaries, or lookaround.

The runtime owns RegExp objects, `lastIndex`, result construction, and the live
`RegExpExec` boundary used by String methods. The backend reports optional work
exhaustion separately from match and no-match results. Unicode tables are pinned
independently of the Rust compiler's Unicode version.

### Starts inside a surrogate pair

The published [RegExpBuiltinExec](https://262.ecma-international.org/17.0/#sec-regexpbuiltinexec)
algorithm is inconsistent about the start of a Unicode match when `lastIndex`
points inside a valid surrogate pair: matching uses the decoded code point, but
the result construction can retain the interior code-unit index.

spite uses the pair's leading boundary for the whole match and captures, for
both empty and consuming matches. This follows Node/V8 and keeps the result
consistent with the matcher's boundaries. The
[regress integration notes](../vendor/regress/SPITE-PATCHES.md) describe the
backend requirements and maintained patches.

## Public class fields

Computed names run in the enclosing context before static initialization.
Field initializers run in their own strict ordinary-function environments with
initialized `this`, undefined `new.target`, a method home object, and the field
name for `NamedEvaluation`. Instance fields initialize in construction order;
static fields and blocks run in source order after class-name initialization.

The published `FieldDefinition` grammar forwards `Yield` and `Await` to
`Initializer`, although `ClassFieldDefinitionEvaluation` creates an ordinary
function. The parser establishes an ordinary-function suspension context for
initializers while preserving the enclosing context for computed names.

In scripts, an initializer can read an identifier named `await`; module code
still reserves it. Direct suspension expressions are rejected before evaluation.
Nested async and generator functions establish their own contexts. This
interpretation applies to public and private fields, both instance and static.

The grammar/runtime conflict is discussed in upstream
[issue 2437](https://github.com/tc39/ecma262/issues/2437) and
[issue 3333](https://github.com/tc39/ecma262/issues/3333). Reference engines differ
on direct static-field `await` identifiers, including field-order-dependent
behavior.
