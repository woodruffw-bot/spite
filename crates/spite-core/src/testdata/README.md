These unchanged Unicode 18 files provide independent test references.

UnicodeData-18.0.0.txt: https://www.unicode.org/Public/18.0.0/ucd/UnicodeData.txt
SHA-256: 0736451de439ae7baf1425136617da495e09ee5afbe6e394374db7009ea08950

CaseFolding-18.0.0.txt: https://www.unicode.org/Public/18.0.0/ucd/CaseFolding.txt
SHA-256: a004797658a457bec4dc11683e39f69249ea3b595b752dbea6721c4c9f587b0d

License: https://www.unicode.org/license.txt (Unicode License v3).
Production uses generated static intervals and the existing pinned case data.
Tests parse these original files independently; no build-time download is needed.

Scripts-18.0.0.txt: https://www.unicode.org/Public/18.0.0/ucd/Scripts.txt
SHA-256: 0071fd81b6aeae25f6e8bce8efec3066a6476a91b49bdb2f52dc76e817862a6a

Greek Script tests independently check both complement orders and all common/simple fold preimages using these same unchanged references.

Cyrillic Script tests independently check raw membership and its invariant common/simple fold images using these same unchanged references.

Arabic Script tests independently check raw membership and its invariant common/simple fold images using these same unchanged references.

Hebrew Script tests independently check raw membership and its invariant common/simple fold images using these same unchanged references.

Common Script tests independently check raw membership and its common/simple fold images and distinct u+i/v+i complements using these same unchanged references.

Inherited Script tests independently check raw membership and its common/simple fold images and distinct u+i/v+i complements using these same unchanged references.

Devanagari Script tests independently check raw membership and its common/simple fold images and distinct u+i/v+i complements using these same unchanged references.

Bengali Script tests independently check raw membership and its invariant common/simple fold images using these same unchanged references.

Gurmukhi Script tests independently check raw membership and its invariant common/simple fold images using these same unchanged references.

Gujarati Script tests independently check raw membership and its invariant common/simple fold images using these same unchanged references.

PropertyValueAliases-18.0.0.txt: https://www.unicode.org/Public/18.0.0/ucd/PropertyValueAliases.txt
SHA-256: 06c4c8eaf7b0bf34abe73b113da1215bd784ac254d4c223600b90267caa4bbbd

The remaining Script registry uses this unchanged alias reference with Scripts
and CaseFolding. Independent tests parse all 179 canonical values and 351 value
spellings, honor Unknown's @missing default, and retain Katakana_Or_Hiragana as a
valid empty set. The generator emits 167 new Script sets while preserving the
earlier specialized plans.

ScriptExtensions-18.0.0.txt: https://www.unicode.org/Public/18.0.0/ucd/ScriptExtensions.txt
SHA-256: 5c9d34a922f687726f2a8bcf57d49f905987e51f1b21b58c95a00fbe255cec23

Script_Extensions uses each explicit nonempty set instead of its Script fallback;
unlisted points inherit Script, including Unknown's default. All 179 canonical
values and 351 exact spellings form 702 Script_Extensions=/scx= keys; Hrkt is a
valid empty set. Common and Greek retain distinct u+i/v+i complement plans.
Independent tests parse aliases, Scripts, ScriptExtensions and CaseFolding
separately from the generator, checking overlapping memberships, canonical
images and complete fold preimages. The offline generator verifies all four
source pins and reproduces both matcher modules.
