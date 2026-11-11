# Unicode data

spite uses Unicode 18.0.0 for identifiers, case conversion, normalization, and
RegExp properties. ECMA-262 requires the latest Unicode standard. The data
version is pinned independently of Rust, so a compiler update does not change
language behavior.

The data is distributed under the [Unicode License v3](LICENSE).

## Generated tables

| Data | Sources | Consumer |
| --- | --- | --- |
| Identifier start/continue ranges | DerivedCoreProperties.txt | [unicode_data.rs](../../crates/spite-unicode/src/unicode_data.rs) |
| Case mappings and folding | UnicodeData.txt, SpecialCasing.txt, DerivedCoreProperties.txt, CaseFolding.txt | [case_data.rs](../../crates/spite-unicode/src/case_data.rs) |
| Normalization | UnicodeData.txt, DerivedNormalizationProps.txt | [normalization_data.rs](../../crates/spite-unicode/src/normalization_data.rs) |
| RegExp property aliases | PropertyAliases.txt, PropertyValueAliases.txt | [property_data.rs](../../crates/spite-regexp/src/pattern/property_data.rs) |

Source URLs and SHA-256 identities are kept in the generators and generated-file
headers. RegExp matching uses regress's tables; its source inventory is
[UNICODE-SOURCES.json](../../vendor/regress/UNICODE-SOURCES.json), with integration
details in [SPITE-PATCHES.md](../../vendor/regress/SPITE-PATCHES.md).

### Case conversion

Default casing includes unconditional special mappings and contextual final
sigma. Language-specific tailoring is excluded. Unicode RegExp ignore-case uses
common/simple folding; full and Turkic folding records are excluded. Ordinary
RegExp ignore-case instead applies the specified uppercase and ASCII-boundary
rules.

### Normalization

Tables contain decompositions, combining classes, and canonical compositions
after full composition exclusions. Hangul decomposition and composition use the
algorithm from UAX #15.

The complete, unchanged
[NormalizationTest.txt](../../crates/spite-runtime/tests/fixtures/NormalizationTest-18.0.0.txt)
is checked against its pinned source during generation. Native tests exercise its
column invariants and scalar identity outside its Part 1 inventory.

### RegExp properties

Alias tables validate the exact spellings allowed by the published edition,
including its binary-property whitelist. Script_Extensions uses Script's value
aliases. Properties of strings are admitted only in UnicodeSetsMode. Syntax
validation is independent of the matching backend's property tables.

Unchanged [UCD reference files](../../crates/spite-core/src/testdata/README.md)
let runtime tests derive Script and Script_Extensions expectations independently
of generated matcher data.

## Check or regenerate

Run commands from the repository root. Generation requires Python 3; ordinary
Cargo builds and tests use checked-in data.

```sh
cargo xtask generate unicode --check
cargo xtask generate case-mappings --check
cargo xtask generate normalization --check
cargo xtask generate regexp-properties --check
```

Omit `--check` to write generated output. With no source argument, the generators
download their checksum-pinned inputs. To use local data, pass a
DerivedCoreProperties.txt path to `unicode`, or a UCD directory to the other
generators. Each generator accepts `--help`.

A Unicode update must change the explicit version and source pins, regenerate
affected tables and fixtures, update the backend data, and review tests whose
expectations depend on changed memberships. See the
[generator guide](../../crates/xtask/codegen/README.md) for commands.
