# Roadmap

The language target remains [published ECMAScript 2026](compatibility.md).
The main remaining implementation work is reducing native resource constraints
and strengthening the evidence for observable language behavior.

## Reduce native execution constraints

Calls and construction currently share a 32-frame native-stack guard. Moving
more execution state into explicit interpreter frames would allow deeper
JavaScript execution without depending on the host thread's stack size. Such a
change must preserve completions, exception ownership, suspension, tail calls,
and cross-realm restoration.

Collection currently requires an idle host checkpoint. Safe collection points
during long executions would reduce retained memory, but require explicit roots
for every live evaluation value. WeakRef kept-object semantics and finalization
scheduling must remain tied to the host lifecycle.

Until those designs are implemented and validated, the
[current constraints](compatibility.md#native-constraints) remain part of the
implementation's contract.

## Strengthen conformance coverage

Prioritize tests that distinguish plausible implementations: conversion and
error order, effects of getters and proxies, realm ownership, module cycles,
suspension, shared memory, and collection lifetimes. Independent numeric and
Unicode references should continue to check generated tables and algorithms.

Review additional upstream originals against the published edition and host
choices before admitting them. Preserve original source and execution modes.
Resource failures remain nonpassing results; smaller local regressions can
explain behavior but do not replace an upstream original.

## Maintain the language and data baseline

Track new Unicode releases and upstream conformance tests, with explicit source
pins and reproducible generation. Review incompatible fixture expectations
rather than treating another engine's output as the specification.

Adopting a later ECMAScript edition requires a deliberate baseline review:
grammar, algorithms, host hooks, fixtures, and documentation must agree on the
same edition. Browser APIs, Node.js compatibility, and unadopted proposals remain
outside this roadmap.
