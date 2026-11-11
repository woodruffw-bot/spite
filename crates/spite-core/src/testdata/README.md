# Unicode reference files

These are unchanged Unicode 18.0.0 files from the
[Unicode Character Database](https://www.unicode.org/Public/18.0.0/ucd/),
distributed under the [Unicode License v3](../../../../third-party/unicode/LICENSE).
Their source identities are recorded in
[UNICODE-SOURCES.json](../../../../vendor/regress/UNICODE-SOURCES.json).

| File | Reference data |
| --- | --- |
| [UnicodeData-18.0.0.txt](UnicodeData-18.0.0.txt) | Character categories, mappings, and other core properties |
| [CaseFolding-18.0.0.txt](CaseFolding-18.0.0.txt) | Case-fold mappings |
| [Scripts-18.0.0.txt](Scripts-18.0.0.txt) | Script membership, including the Unknown default |
| [ScriptExtensions-18.0.0.txt](ScriptExtensions-18.0.0.txt) | Explicit Script_Extensions sets |
| [PropertyValueAliases-18.0.0.txt](PropertyValueAliases-18.0.0.txt) | Canonical values and exact aliases |

Runtime tests parse the script, alias, and case-fold files directly to derive
expected RegExp memberships. Script_Extensions uses an explicit set where
present and otherwise inherits Script. Tests cover aliases, overlapping sets,
case-fold images, and the different complement orders in Unicode matching modes.

Production code uses generated tables and the regress backend rather than
parsing these files. The [Unicode data guide](../../../../third-party/unicode/README.md)
describes the consumers, generation commands, and update procedure. No download
is needed to run the reference tests.
