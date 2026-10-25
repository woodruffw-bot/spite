# Spite integration of regress

Upstream: https://github.com/ridiculousfish/regress

Pinned commit: `4f2bfeb56ff4374a64d89c709b5083b07c529e86`. The upstream crate
version is 0.12.0; this snapshot additionally includes upstream fixes for empty
UnicodeSets strings, class-set semantics and budgeted backtracking. Licenses
and upstream tests are retained.

Features: `std`, `utf16`, `prohibit-unsafe`, `index-positions`; defaults disabled.
The normalized 0.12.0 registry manifest is retained for workspace integration.

Local changes:

- Add a UTF-16/UCS-2 sticky/search API with optional instruction and backtracking
  work accounting and an explicit interruption outcome.
- Fold UnicodeSets scalar/string operands before set operations and complement.
- Expose the compiled capture count.
- Decode UnicodeSets `\b` class members as backspace.
- Balance flat disjunction IR trees while preserving branch order, preventing
  native-stack overflow during recursive optimization and destruction.

Rust 1.88 is required for the upstream let-chain syntax. Runtime uses this
adapter for all native matching. Backend representation limits remain explicit
implementation constraints.

The adapter compiles ordinary Pattern input as UTF-16 code units, Unicode
Pattern input as decoded code points. Group IdentifierNames retain Unicode
decoding in both modes, including surrogate pairs and braced escapes.

Unicode tables and their generated property tests were regenerated with the
upstream `gen-unicode` tool from Unicode 18.0.0. Exact source URLs and SHA-256
digests are in `UNICODE-SOURCES.json`. The development generator is run
separately; the engine build has no code-generation dependencies.

Repetition counters reject decimal overflow before execution rather than
saturating to the native integer maximum. Unrepresentable minimum/exact bounds
report backend capacity errors, separate from syntax errors and JavaScript
exceptions. An overflowing upper bound uses the VM unbounded form: it cannot
constrain any executable sequence of optional progress steps.

Legacy ignore-case uses ECMAScript's one-code-unit uppercase rule, including
full-uppercase expansion exclusions and the non-ASCII-to-ASCII barrier. Unicode
matching retains simple case folding. Both modes close class operands before
complementing word classes. The generated expansion exclusions are pinned to
Unicode 18 SpecialCasing.txt in UNICODE-SOURCES.json.

The Unicode generator retains the valid, empty Katakana_Or_Hiragana script
(and Hrkt alias), for Script and Script_Extensions. The upstream exclusion no
longer matches the project's edition-17 property-name validation.

Mixed reserved punctuation remains valid in UnicodeSets; only identical
neighbors form reserved double punctuators. Fixed-hex surrogate escape pairing
restores the complete input position when the second escape does not pair.
Legacy expansion exclusions are regenerated and checked by
`tools/generate-case-mappings.py` alongside the Core casing tables.
