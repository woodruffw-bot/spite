# Facilities outside ECMAScript 2026

These comparisons use the exact published ECMA-262 edition recorded in
[specification.json](specification.json). They concern requirements absent from
that edition, including later changes to an existing algorithm.

A metadata `esid` can name an existing clause while the tested behavior comes
from a later proposal. Review source requirements rather than relying on the
clause name or feature tag.

## Syntax and module loading

| Scope feature | Difference from the published edition |
| --- | --- |
| `decorators` | No decorator or auto-accessor class-element grammar |
| `explicit-resource-management` | No using/await-using declarations, disposal symbols, DisposableStack, AsyncDisposableStack, or SuppressedError |
| `source-phase-imports` | No source-phase import syntax, module-source objects, or source-phase loading algorithms |
| `import-defer` | No deferred import syntax, deferred namespace methods, or defer import option |
| `import-text` | No text-module creation/loading contract; supported module attribute types remain a host choice |

The import comparisons cover the full published
[import grammar](https://262.ecma-international.org/17.0/#sec-imports) and
[dynamic import evaluation](https://262.ecma-international.org/17.0/#sec-import-call-runtime-semantics-evaluation),
including their module records and loading algorithms.

## APIs and algorithm changes

| Scope feature | Difference from the published edition |
| --- | --- |
| `Temporal` | No Temporal API or `Date.prototype.toTemporalInstant`; the similarly named offset grammar does not introduce that API |
| `ShadowRealm` | No ShadowRealm constructor, evaluation, or `importValue` |
| `await-dictionary` | No `Promise.allKeyed` or `Promise.allSettledKeyed` |
| `joint-iteration` | No `Iterator.zip`, `Iterator.zipKeyed`, or zip iterator modes |
| `iterator-chunking` | No `Iterator.prototype.chunks` or `windows` |
| `iterator-includes` | No `Iterator.prototype.includes` |
| `Iterator.prototype.join` | No `Iterator.prototype.join` |
| `immutable-arraybuffer` | No immutable backing slot, accessor, `sliceToImmutable`, or `transferToImmutable` |
| `error-stack-accessor` | No standard `Error.prototype.stack` getter/setter or their receiver and descriptor requirements |
| `nonextensible-applies-to-private` | Published private-element installation does not reject an object solely because it is non-extensible |

The private-element comparison follows
[PrivateFieldAdd](https://262.ecma-international.org/17.0/#sec-privatefieldadd) and
[PrivateMethodOrAccessorAdd](https://262.ecma-international.org/17.0/#sec-privatemethodoraccessoradd).
Neither checks `[[Extensible]]`; installing a distinct private name succeeds.
Tests requiring a `TypeError` for non-extensibility assert a different algorithm.
