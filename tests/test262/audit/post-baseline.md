# Facilities absent from edition 17

This review uses the [published es2026 source](https://github.com/tc39/ecma262/blob/0248456c758431e4bb8e5d26333ff1865123c9cd/spec.html),
whose SHA-256 is
`ce7bc30174061fd8d212270b81cf6511661180c1e174f6911d10ced0581527b0`.
The [source identity](specification.json) also records its Git blob and byte count.
The official annotated es2026 tag resolves to that revision. It does not use the
living specification to decide which features are mandatory.

Scope entries identify APIs, productions or semantic changes absent from the
published baseline. A metadata `esid` can refer to an existing clause even when
the tested behavior belongs to a later proposal. Review the source prerequisites
rather than excluding tests solely by their feature tags.

| Scope feature | Published-baseline comparison |
| --- | --- |
| `Temporal` | No Temporal global, constructor/prototype API, or Date.prototype.toTemporalInstant. The TemporalDecimalFraction grammar used for UTC offsets is not the Temporal API. |
| `ShadowRealm` | No ShadowRealm global or associated evaluation/importValue methods. |
| `explicit-resource-management` | No UsingDeclaration/await-using production, DisposableStack/AsyncDisposableStack API, disposal symbols or SuppressedError API. Existing implementation extensions supply no edition-17 credit. |
| `source-phase-imports` | The published ImportDeclaration and ImportCall productions have no source phase; ModuleRequest and loading algorithms have no source-phase module-source objects. |
| `import-defer` | No deferred import productions, deferred namespace internal methods or defer import option. Existing module namespace/evaluation clauses do not specify these added modes. |
| `await-dictionary` | No Promise.allKeyed/allSettledKeyed APIs or their dictionary-based aggregation algorithms. |
| `joint-iteration` | No Iterator.zip/zipKeyed APIs, zip iterator prototype or longest/strict iteration modes. |
| `iterator-chunking` | No Iterator.prototype.chunks/windows methods or their grouped-yield semantics. |
| `iterator-includes` | No Iterator.prototype.includes method. |
| `Iterator.prototype.join` | No Iterator.prototype.join method. |
| `immutable-arraybuffer` | No immutable backing slot, immutable accessor or sliceToImmutable/transferToImmutable methods; existing view/atomic methods do not have the added immutable-buffer branches. Includes five tests also requiring byte-module imports. |
| `error-stack-accessor` | No standard Error.prototype.stack getter/setter or associated cross-Realm receiver and descriptor requirements. |
| `decorators` | No decorator productions or auto-accessor class element grammar. |
| `import-text` | No text-module record creation/loading contract; the host's supported module attribute types remain an explicit choice. |
| `nonextensible-applies-to-private` | These originals require TypeError when adding private fields/methods/accessors to non-extensible objects. Published PrivateFieldAdd and PrivateMethodOrAccessorAdd do not test [[Extensible]]; their distinct-private-name installations succeed. |

The import checks use the complete published [import grammar](https://262.ecma-international.org/17.0/#sec-imports)
and [dynamic import evaluation](https://262.ecma-international.org/17.0/#sec-import-call-runtime-semantics-evaluation),
not merely new method names. The private-element comparison uses
[PrivateFieldAdd](https://262.ecma-international.org/17.0/#sec-privatefieldadd)
and [PrivateMethodOrAccessorAdd](https://262.ecma-international.org/17.0/#sec-privatemethodoraccessoradd).
The two private-element tests assert a different algorithm, rather than uncovering
a missing edition-17 extensibility check.
