# Staging scope decisions

Staging originals are reviewed against the exact published baseline,
independently of their outcomes. [staging-reviewed.tsv](staging-reviewed.tsv)
records a source-specific explanation and upstream Git identity for every
closed, unretained entry.

## Prerequisites

Some sources require facilities absent from the baseline: disposal syntax/APIs,
Temporal, auto-accessor class elements, source-phase imports or nonextensibility
restrictions on installing private fields. The [post-baseline review](post-baseline.md)
explains those differences. Ordinary identifiers named `accessor` still use
baseline grammar despite their proposal metadata tag.

Other sources require disabled Object.prototype helpers, caller-chain extensions
or Annex B features such as browser IsHTMLDDA, block-function var synthesis,
legacy RegExp methods/escapes, String.substr or Date.setYear. Required literal
prototype initializers and function poison accessors remain enabled.

## Permitted choices and data versions

Non-ISO Date.parse fallback formats are implementation-specific. This host can
return NaN for SpiderMonkey-specific forms. EscapeRegExpPattern permits equivalent,
deterministic source spellings that preserve literal parsing and matcher behavior.
Some generated Unicode casing expectations use an older data version. Policy
probes and Unicode witnesses check the selected current behavior independently.

## Native resource constraints

[resource-stress.json](resource-stress.json) records unchanged sources that reach
native limits. They receive no passing credit. The native call/construct guard
covers deep explicit constructor chains and intentional infinite recursion on
supported native stacks.

Exhaustive allocation stress includes array-copy matrices, repeated Date/cache
sequences, TypedArray conversions, overlap copies and large numeric sorts with
repeated assertions. Another source asks for a 2**36-code-unit replacement result.
Their allocation diagnostics use an external address-space ceiling, recorded
separately from Runtime defaults. The ceiling is an investigation condition,
not a universal algorithm bound.

Collection occurs at explicit idle checkpoints. Long uninterrupted Scripts can
retain substantial storage until that point. A timeout, allocator failure or
host termination cannot be converted into a JavaScript exception or a passing
negative test. Do not shorten originals or increase default quotas to satisfy
these stress cases; supplementary tests receive separate coverage.
