# Facilities absent from edition 17

This review uses the [published es2026 source](https://github.com/tc39/ecma262/blob/0248456c758431e4bb8e5d26333ff1865123c9cd/spec.html),
whose SHA-256 is
`032ecc740356b38f6846338d8c44536291a134a6ce60e651f935befb239cfc1a`.
The official annotated es2026 tag resolves to that revision. It does not use the
living specification to decide which features are mandatory.

The 5,782 core originals below require APIs, productions or semantic changes
absent from that published baseline. Their scope entries name the relevant
feature explicitly. Many cite an existing clause's `esid`; absence of a metadata
clause identifier alone was not used to exclude a test. Features are assigned in
the table's order so tests with several proposal prerequisites appear once.

| Scope feature | Originals | Published-baseline comparison |
| --- | ---: | --- |
| `Temporal` | 4,613 | No Temporal global, constructor/prototype API, or Date.prototype.toTemporalInstant. The TemporalDecimalFraction grammar used for UTC offsets is not the Temporal API. |
| `ShadowRealm` | 64 | No ShadowRealm global or associated evaluation/importValue methods. |
| `explicit-resource-management` | 160 | No UsingDeclaration/await-using production, DisposableStack/AsyncDisposableStack API, disposal symbols or SuppressedError API. Existing implementation extensions supply no edition-17 credit. |
| `source-phase-imports` | 251 | The published ImportDeclaration and ImportCall productions have no source phase; ModuleRequest and loading algorithms have no source-phase module-source objects. |
| `import-defer` | 252 | No deferred import productions, deferred namespace internal methods or defer import option. Existing module namespace/evaluation clauses do not specify these added modes. |
| `await-dictionary` | 89 | No Promise.allKeyed/allSettledKeyed APIs or their dictionary-based aggregation algorithms. |
| `joint-iteration` | 82 | No Iterator.zip/zipKeyed APIs, zip iterator prototype or longest/strict iteration modes. |
| `iterator-chunking` | 78 | No Iterator.prototype.chunks/windows methods or their grouped-yield semantics. |
| `iterator-includes` | 44 | No Iterator.prototype.includes method. |
| `Iterator.prototype.join` | 18 | No Iterator.prototype.join method. |
| `immutable-arraybuffer` | 66 | No immutable backing slot, immutable accessor or sliceToImmutable/transferToImmutable methods; existing view/atomic methods do not have the added immutable-buffer branches. Includes five tests also requiring byte-module imports. |
| `error-stack-accessor` | 35 | No standard Error.prototype.stack getter/setter or associated cross-Realm receiver and descriptor requirements. |
| `decorators` | 22 | No decorator productions or auto-accessor class element grammar. |
| `import-text` | 6 | No text-module record creation/loading contract; the host's supported module attribute types remain an explicit choice. |
| `nonextensible-applies-to-private` | 2 | These originals require TypeError when adding private fields/methods/accessors to non-extensible objects. Published PrivateFieldAdd and PrivateMethodOrAccessorAdd do not test [[Extensible]]; their distinct-private-name installations succeed. |
| **Total** | **5,782** | **No edition-17 passing-test credit is claimed.** |

The import checks use the complete published [import grammar](https://262.ecma-international.org/17.0/#sec-imports)
and [dynamic import evaluation](https://262.ecma-international.org/17.0/#sec-import-call-runtime-semantics-evaluation),
not merely new method names. The private-element comparison uses
[PrivateFieldAdd](https://262.ecma-international.org/17.0/#sec-privatefieldadd)
and [PrivateMethodOrAccessorAdd](https://262.ecma-international.org/17.0/#sec-privatemethodoraccessoradd).
The two private-element tests assert a different algorithm, rather than uncovering
a missing edition-17 extensibility check.

The conformance-post-baseline-review artifact under `/workspace/work` records
every reviewed original's upstream blob, SHA-256, full metadata and assigned
feature, plus source identity and facility/production checks. The remaining 202
core scope entries and the staging inventory still require separate review.
