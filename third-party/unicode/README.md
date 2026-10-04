# Unicode identifier and case data

`crates/spite-core/src/unicode_data.rs` contains ID_Start and ID_Continue ranges
from Unicode 18.0.0. The source URL and SHA-256 are recorded in that file and in
`tools/generate-unicode.py`. The Unicode license is included here.

ECMA-262 edition 17 refers to the latest Unicode standard in sections 2, 3, and
12.7. Unicode 18.0.0 is the current final UCD release used by this implementation.
The version is explicit so Rust toolchain updates cannot change identifier syntax
or case conversion.

`crates/spite-core/src/case_data.rs` contains full lower/uppercase mappings and
Cased/Case_Ignorable ranges used for contextual final sigma. Its generator,
`tools/generate-case-mappings.py`, pins UnicodeData.txt, SpecialCasing.txt, and
DerivedCoreProperties.txt by SHA-256. It incorporates all unconditional special
mappings and verifies that Final_Sigma is the only language-insensitive condition.
Language-specific tailoring is excluded from default conversion. The locale
methods use this host's fixed locale-neutral fallback.

The generator merges adjacent ranges without changing membership. It verifies the
source digest before reading properties. Regeneration requires Python's standard
library and is not part of a Rust build.

```sh
python3 tools/generate-unicode.py --check
python3 tools/generate-unicode.py /path/to/DerivedCoreProperties.txt --check
python3 tools/generate-case-mappings.py --check
python3 tools/generate-case-mappings.py /path/to/ucd-directory --check
```

Omit `--check` to regenerate. The first form downloads the pinned source. The
second uses a local copy. CI verifies the checked-in output against the pinned
source. Updating Unicode requires an explicit version, source digest, generated
table, and test update.
