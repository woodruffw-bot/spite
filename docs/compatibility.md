# Compatibility and constraints

spite targets [ECMA-262, 17th edition (ECMAScript 2026)](https://262.ecma-international.org/17.0/)
in a non-browser host. The published edition defines the target; the living
specification and later proposals do not silently change it. The exact source
identity is recorded in [specification.json](../tests/test262/audit/specification.json).

Browser APIs, Node.js compatibility, and ECMA-402 internationalization are outside
this target. This page describes the host choices and implementation constraints
that matter when running JavaScript or assessing conformance.

## Optional language features

| Feature | Choice |
| --- | --- |
| Annex B browser extensions | Disabled |
| `Object.prototype.__proto__` accessor and legacy accessor helpers | Disabled |
| Optional function `caller` and `arguments` reflection | Disabled; required throwing accessors remain present |
| `WeakRef.prototype.constructor` | Enabled |

Object-literal prototype initializers remain supported. A specification section
labelled “Legacy” is not necessarily optional: non-strict legacy numeric literals
and string escapes remain part of the language.

Some implemented facilities come from later proposals. Their presence does not
establish conformance to the published edition or expand the target to include
the rest of a proposal.

## Host behavior

### Modules and jobs

The embedding API leaves module loading and supported import-attribute keys to
the host. Attribute keys are fixed when a realm is created; registering a key
does not install a loader or module format. The default `import.meta` object has
a null prototype and no properties.

The library requires the host to advance Promise jobs and arrange collection and
finalization checkpoints. Unhandled Promise rejection tracking defaults to a
no-op. Ordinary realms cannot suspend in `Atomics.wait`; a host can explicitly
create a suspending realm.

The [embedding guide](embedding.md) explains these responsibilities. The
[CLI guide](cli.md) describes the file loader and job handling supplied by the
command-line host.

### Dates, locales, and randomness

A realm uses the system time zone unless the host selects another `TimeZone`.
Named-zone lookup uses host data with a pinned IANA database as a fallback.
`TimeZone::bundled` selects the pinned database directly for reproducible
embeddings. Invalid or unreadable host configuration reports a host error rather
than silently selecting UTC.

Without ECMA-402, locale methods use fixed fallbacks:

- Number and BigInt locale formatting use ordinary numeric formatting.
- Date locale formatting uses the local date/time formats with English names.
- String locale comparison orders canonically decomposed (NFD) UTF-16 sequences.
- Locale case conversion uses Unicode default casing without language-specific
  tailoring.

`Date.parse` accepts the standard date-time format and the runtime's UTC and local
date-string formats. Other non-ISO forms can return `NaN`; browser-specific
fallback parsing is not a compatibility promise.

`Math.random` uses distinct per-realm pseudo-random sequences seeded by the
process. A `debugger` statement completes normally without an active debugger.

### Compilation, private elements, and buffers

String compilation and function source availability are enabled, subject to
explicit host quotas and native capacity. The default private-element host hook
allows installation on non-extensible objects, as the published algorithms
require.

Buffers use sparse backing storage: untouched zero regions need not allocate
physical memory. A large logical length can therefore succeed where a dense
allocation would fail. This does not change the specified numeric conversions
or their error order.

## Native constraints

Every field of `Limits` defaults to `None`. Hosts can opt into work, source,
string, BigInt, argument, heap-slot, and property quotas. A quota failure is a host
failure, not a catchable JavaScript exception.

Native resources remain finite even with no quotas selected:

- Hosts must provide a native thread stack of at least 2 MiB.
- Calls and construction share a guard of 32 active frames. Deep recursion can
  reach this guard with all quotas disabled.
- Recursive grammar, expression trees, and templates have separate parser stack
  guards. Flat expression chains and selected graph operations are iterative.
- Platform capacity checks and allocation failures can stop execution.
- Garbage collection runs only at explicit idle checkpoints. An uninterrupted
  script can retain a large heap before the host regains control.

The [resource-stress records](../tests/test262/audit/resource-stress.json) preserve
nonpassing outcomes for upstream tests that exceed native capacity. Some records
use an external diagnostic memory ceiling; that ceiling is separate from the
runtime's default configuration. These records describe observed resource
failures, not portable algorithm bounds.

## Specification interpretations

Two published-text inconsistencies require explicit choices:

- **Unicode RegExp starts inside a surrogate pair.** The whole match and captures
  use the pair's leading boundary. See [regular expressions](design.md#regular-expressions)
  for the conflict in `RegExpBuiltinExec`.
- **Class field initializers.** Initializers establish an ordinary-function
  suspension context; computed names retain their enclosing context. See
  [public class fields](design.md#public-class-fields) for the grammar/runtime
  conflict and upstream discussions.

Conformance claims must retain these qualifications along with the host choices
and native constraints above.

## Conformance evidence

Native tests cover primitive algorithms, parser snapshots, observable runtime
behavior, conversion and error order, object lifetimes, and host boundaries.
The [Test262 runner](../tests/test262/README.md) executes selected, unchanged
upstream originals with their declared modes and harness files.

[Scope records](../tests/test262/audit/README.md) account for every original
outside the executable selection. They explain optional features, later
proposals, incompatible Unicode data, permitted host choices, and resource
constraints. Excluded tests are not counted as passes.

Syntax-negative tests require an exact reviewed syntax diagnostic. Unsupported
syntax, limits, metadata errors, and host failures cannot satisfy an expected
JavaScript exception. Passing the selected corpus is useful evidence; it is not
a proof of complete conformance.
