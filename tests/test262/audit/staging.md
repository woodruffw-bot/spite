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

Seventy-two more originals require disabled legacy facilities: twenty-nine use
the normative optional Object.prototype accessor/helpers, thirteen use function
caller/arguments reflection extensions, and thirty require Annex B facilities.
The latter include block-function var synthesis, labelled/if function forms,
for-in initializers, catch-var redeclarations, RegExp.compile/static accessors,
non-Unicode RegExp escapes, String.substr and Date.setYear. Required literal
prototype initializers and Function.prototype poison accessors stay enabled.

Three other originals assume a narrower result than the published algorithms:
the Date non-ISO fallback parser is implementation-specific (21.4.3.2), the exact
RegExp source spelling can vary while preserving deterministic literal parsing
and matcher behavior (EscapeRegExpPattern), and the generated Unicode17 casing
table differs from current Unicode18 at eight BMP mappings. All 65,536 mapping
pairs were independently compared with the original pinned UnicodeData and
SpecialCasing inputs; the differences are the upper/lower mappings of
0277/A7DD, 027C/A7E2, AB4B/AB6C and AB4C/AB6D. Bounded runner probes exercise
the Date policy, RegExp literal equivalence and every changed case mapping.

The fresh run of 1,310 applicable
originals passes all 2,476 modes with default Runner settings, two ordinary
workers and no external quotas, in 106.292 seconds. They are now admitted along
with the five exactly reviewed Module negatives and the longer-running
String.fromCodePoint original. All 1,316 added originals / 2,483 modes pass again
using only retained sources and helpers, Runner defaults and no external quotas
in 32.968 seconds. Six supporting modules and nine required harness helpers are
retained unchanged as well.

Together with the nine earlier semantic-bug originals, this retains 1,325
applicable staging originals. Fifteen resource/native-stack originals remain
pending. The enlarged full gate and final host-constraint review stay open.

The initial sweep's 20-second process deadline was an investigation safeguard,
not a language limit or an accepted exclusion. Longer runs retain Runner defaults
and report native-capacity failures and external termination separately. Finite
stress, native-stack constraints, implementation-defined expectations, remaining
negative diagnostics and optional legacy prerequisites still require review.
Passing a preliminary diagnostic run does not close the final applicable gate.
