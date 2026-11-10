# Supplementary staging review

The pinned staging inventory is reviewed against the exact published source in
[specification.json](specification.json), independently of whether a test passes.
Each closed, unretained entry has a source-specific explanation and its upstream
Git blob in [staging-reviewed.tsv](staging-reviewed.tsv). The checker requires
that ledger and the scope decisions to match exactly; pending entries remain
`staging-review`.

## Proposal and browser prerequisites

| Prerequisite | Originals | Source review |
| --- | ---: | --- |
| Explicit resource management | 53 | Each body requires disposal stacks, disposal symbols, SuppressedError or using declarations. Those facilities are absent from edition 17. Existing partial extensions supply no passing credit. |
| Temporal | 3 | Two Temporal API originals and the Date.prototype.toTemporalInstant original require the absent Temporal API. |
| Auto-accessors | 2 | These class elements require the absent auto-accessor production. The separate accessor-as-identifier original uses ordinary identifier/property grammar and stays in the applicable review despite its decorators feature tag. |
| Source-phase imports | 2 | Both require import.source and associated source-phase module records or objects, absent from the published ImportCall and module-loading algorithms. |
| Nonextensibility applying to private fields | 1 | The original requires TypeError when stamping a new private field onto a frozen object. Published PrivateFieldAdd has no extensibility check; the host uses the default private-element hook. |
| Browser IsHTMLDDA | 7 | Four Iterator coercion tests, Reflect.defineProperty, nullish coalescing and delegated generator throwing require $262.IsHTMLDDA. This host disables Annex B.3.6 and that browser object. |
| **Total** | **68** | **Explicit prerequisites, without conformance passing credit.** |

These decisions account for every Unsupported result in the initial staging
sweep: 24 disposal originals produce 47 Unsupported modes, and seven IsHTMLDDA
originals produce 14. Unsupported is never accepted as a negative-test pass.
The other reviewed proposal originals may fail differently or pass existing
extensions; the published algorithms determine their scope.

## Remaining qualification

There are 1,406 pending staging originals. Nine other originals have already
been admitted unchanged, covering fixed shared views, deleted eval bindings,
Unicode backreferences, empty blocks and class-field initializer contexts.

The initial sweep's 20-second process deadline was an investigation safeguard,
not a language limit or an accepted exclusion. Longer runs retain Runner defaults
and report native-capacity failures and external termination separately. Finite
stress, native-stack constraints, implementation-defined expectations, remaining
negative diagnostics and optional legacy prerequisites still require review.
Passing a preliminary diagnostic run does not close the final applicable gate.
