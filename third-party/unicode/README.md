# Unicode identifier, case, normalization, and RegExp property data

`crates/spite-unicode/src/unicode_data.rs` contains ID_Start and ID_Continue ranges
from Unicode 18.0.0. The source URL and SHA-256 are recorded in that file and in
`cargo xtask generate unicode`. The Unicode license is included here.

ECMA-262 edition 17 refers to the latest Unicode standard in sections 2, 3, and
12.7. Unicode 18.0.0 is the current final UCD release used by this implementation.
The version is explicit so Rust toolchain updates cannot change identifier syntax
or String transforms.

`crates/spite-unicode/src/case_data.rs` contains full lower/uppercase mappings and
Cased/Case_Ignorable ranges used for contextual final sigma, plus simple/common
case folding for RegExp. Its generator,
`cargo xtask generate case-mappings`, pins UnicodeData.txt, SpecialCasing.txt,
DerivedCoreProperties.txt and CaseFolding.txt by SHA-256. It incorporates all
unconditional special mappings and verifies that Final_Sigma is the only
language-insensitive condition.
Language-specific tailoring is excluded from default conversion. The locale
methods use this host's fixed locale-neutral fallback.
RegExp Canonicalize uses C/S records from CaseFolding.txt with u/v ignore-case.
Its full and Turkic records are excluded. Ordinary ignore-case instead uses the
existing full uppercase mappings with single-unit and ASCII-boundary checks.
The CaseFolding.txt source is https://www.unicode.org/Public/18.0.0/ucd/CaseFolding.txt
and its SHA-256 is
`a004797658a457bec4dc11683e39f69249ea3b595b752dbea6721c4c9f587b0d`.

`crates/spite-unicode/src/normalization_data.rs` contains decomposition mappings,
combining classes, and canonical compositions after full composition exclusions.
`cargo xtask generate normalization` pins UnicodeData.txt and
DerivedNormalizationProps.txt and validates the decomposition graph before emitting
tables. Hangul decomposition and composition use UAX #15's algorithm.
The complete unchanged Unicode 18 NormalizationTest.txt is retained at
`crates/spite-runtime/tests/fixtures/NormalizationTest-18.0.0.txt` under the included
Unicode license. Its source is
https://www.unicode.org/Public/18.0.0/ucd/NormalizationTest.txt and its SHA-256 is
`25a50d816764b04abfb4a646d3eb2b2a803284c3873d9a06757b94fe4513dde3`.
The generator verifies those bytes; native tests check all specified column
invariants and scalar identity outside the Part 1 inventory.

The generator merges adjacent ranges without changing membership. It verifies the
source digest before reading properties. Regeneration requires Python's standard
library and is not part of a Rust build.

`crates/spite-regexp/src/pattern/property_data.rs` contains exact Unicode 18
general-category and script value aliases for RegExp property early errors.
`cargo xtask generate regexp-properties` pins PropertyAliases.txt and
PropertyValueAliases.txt by SHA-256 and verifies the edition-17 binary property
whitelist against them. Script_Extensions shares Script's value aliases. The
whitelist excludes other UCD properties and aliases, including WSpace. Seven
edition-17 string properties are allowed only in UnicodeSetsMode. These tables
validate syntax; matching is implemented separately for the admitted property sets.

```sh
cargo xtask generate unicode --check
cargo xtask generate unicode /path/to/DerivedCoreProperties.txt --check
cargo xtask generate case-mappings --check
cargo xtask generate case-mappings /path/to/ucd-directory --check
cargo xtask generate normalization --check
cargo xtask generate normalization /path/to/ucd-directory --check
cargo xtask generate regexp-properties --check
cargo xtask generate regexp-properties /path/to/ucd-directory --check
```

Omit `--check` to regenerate. The first form downloads the pinned source. The
second uses a local copy. CI verifies the checked-in output against the pinned
source. Updating Unicode requires an explicit version, source digest, generated
table, and test update.

RegExp matching now uses the pinned regress backend under `vendor/regress`.
Its Unicode data provenance and local integration changes are recorded in
`vendor/regress/SPITE-PATCHES.md`. The former specialized Script matcher tables
and their generator were removed with the custom matching engine. Unchanged
Unicode 18 reference files under `crates/spite-core/src/testdata/` remain available
for backend data verification; parser property-alias generation above remains
independent of the matching backend.
