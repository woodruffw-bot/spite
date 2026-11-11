# Staging scope decisions

The upstream staging directory contains both baseline tests and tests for later
proposals. Each original is reviewed against the published edition.
[staging-reviewed.tsv](staging-reviewed.tsv) records the source-specific reason
and upstream Git identity for each decided exclusion.

## Required facilities

Some originals need disposal syntax or APIs, Temporal, auto-accessor class
elements, source-phase imports, or later restrictions on private-element
installation. The [later-proposal review](post-baseline.md) describes those
differences.

A tag is not sufficient evidence. For example, an ordinary identifier named
`accessor` can use baseline grammar even when its metadata names the decorators
proposal.

Other originals need disabled optional behavior: Object prototype helpers,
caller-chain reflection, or Annex B features such as IsHTMLDDA, block-function
var synthesis, legacy RegExp methods and escapes, `String.prototype.substr`, or
`Date.prototype.setYear`. Required literal prototype initializers and function
poison accessors remain enabled.

## Host choices and data versions

Some expectations assume choices this host does not make:

- Non-ISO `Date.parse` formats are implementation-specific. SpiderMonkey-specific
  forms can return `NaN` here.
- `EscapeRegExpPattern` permits equivalent source spellings, provided they
  preserve literal parsing and matching behavior.
- Generated casing expectations can use an older Unicode version.

Policy probes and Unicode witnesses check the selected behavior independently
of those originals. The [compatibility guide](../../../docs/compatibility.md)
documents the host's defaults.

## Native resource constraints

[resource-stress.json](resource-stress.json) records unchanged originals that
reach native limits, including deep constructor chains and intentional infinite
recursion. Other cases stress allocation through array copies, typed-array
conversion, repeated Date operations, large sorts, or very large replacement
strings.

Allocation diagnostics can use an external address-space ceiling, recorded
separately from runtime defaults. That ceiling is a condition of the
investigation, not a universal bound on the algorithm.

Collection currently occurs only at explicit idle checkpoints. An uninterrupted
script can retain substantial memory before such a checkpoint. These cases
remain nonpassing: a timeout, allocation failure, or host termination cannot
become a JavaScript exception or a passing negative test. Smaller local tests can
isolate the operations without replacing the upstream originals.
