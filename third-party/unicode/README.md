# Unicode identifier, case, normalization, and RegExp property data

`crates/spite-core/src/unicode_data.rs` contains ID_Start and ID_Continue ranges
from Unicode 18.0.0. The source URL and SHA-256 are recorded in that file and in
`tools/generate-unicode.py`. The Unicode license is included here.

ECMA-262 edition 17 refers to the latest Unicode standard in sections 2, 3, and
12.7. Unicode 18.0.0 is the current final UCD release used by this implementation.
The version is explicit so Rust toolchain updates cannot change identifier syntax
or String transforms.

`crates/spite-core/src/case_data.rs` contains full lower/uppercase mappings and
Cased/Case_Ignorable ranges used for contextual final sigma, plus simple/common
case folding for RegExp. Its generator,
`tools/generate-case-mappings.py`, pins UnicodeData.txt, SpecialCasing.txt,
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

`crates/spite-core/src/normalization_data.rs` contains decomposition mappings,
combining classes, and canonical compositions after full composition exclusions.
`tools/generate-normalization.py` pins UnicodeData.txt and
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

`crates/spite-parser/src/regexp/property_data.rs` contains exact Unicode 18
general-category and script value aliases for RegExp property early errors.
`tools/generate-regexp-properties.py` pins PropertyAliases.txt and
PropertyValueAliases.txt by SHA-256 and verifies the edition-17 binary property
whitelist against them. Script_Extensions shares Script's value aliases. The
whitelist excludes other UCD properties and aliases, including WSpace. Seven
edition-17 string properties are allowed only in UnicodeSetsMode. These tables
validate syntax; matching is implemented separately for the admitted property sets.

```sh
python3 tools/generate-unicode.py --check
python3 tools/generate-unicode.py /path/to/DerivedCoreProperties.txt --check
python3 tools/generate-case-mappings.py --check
python3 tools/generate-case-mappings.py /path/to/ucd-directory --check
python3 tools/generate-normalization.py --check
python3 tools/generate-normalization.py /path/to/ucd-directory --check
python3 tools/generate-regexp-properties.py --check
python3 tools/generate-regexp-properties.py /path/to/ucd-directory --check
```

Omit `--check` to regenerate. The first form downloads the pinned source. The
second uses a local copy. CI verifies the checked-in output against the pinned
source. Updating Unicode requires an explicit version, source digest, generated
table, and test update.

The remaining Script matcher registry is generated offline by
`tools/generate-regexp-scripts.py` from unchanged local Unicode 18
PropertyValueAliases, Scripts, and CaseFolding references under
`crates/spite-core/src/testdata/`. The alias reference has SHA-256
`06c4c8eaf7b0bf34abe73b113da1215bd784ac254d4c223600b90267caa4bbbd`.
The generator verifies all three pins and checks all 1,533 common/simple fold
records to prove invariance of the 167 newly generated Script sets. It emits 652
exact Script/sc keys and their positive/complement intervals. It includes
Unknown's default membership and the empty Katakana_Or_Hiragana value. Earlier
Script plans retain their existing IDs and fold handling. The local check needs
no network access or build-time dependency:

```sh
python3 tools/generate-regexp-scripts.py --check
python3 tools/generate-regexp-scripts.py /path/to/versioned-reference-directory --check
```
