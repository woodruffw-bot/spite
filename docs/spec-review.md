# Published specification review

Review against [ECMA-262 edition 17](https://262.ecma-international.org/17.0/)
and its [published es2026 source](https://github.com/tc39/ecma262/blob/0248456c758431e4bb8e5d26333ff1865123c9cd/spec.html).
The roadmap's whole-specification review remains open until every pending row
has implementation and regression evidence. Pending rows require further review
documentation. Exact fixture inventories and test receipts belong in
[coverage](coverage.md).

| Clauses | Review | Evidence |
| --- | --- | --- |
| 1–5: scope, conformance, references, overview and notation | Reviewed | [Scope and architecture](design.md#scope); implementation separates language completions, unsupported features and host limits |
| 6: types and values | Reviewed | Value and specification-record representations below |
| 7: abstract operations | Reviewed | Operation groups below |
| 8: syntax-directed operations | Pending | |
| 9: execution contexts and environments | Pending | |
| 10: ordinary and exotic objects | Pending | |
| 11: source text | Pending | |
| 12: lexical grammar | Reviewed | [Production/implementation ledger](coverage.md#published-lexical-grammar-review), including all five lexical goals |
| 13: expressions | Pending | [Comparison review](coverage.md#primitive-and-mixed-comparison-review) records one completed part |
| 14: statements and declarations | Reviewed | [Scope, control and completion ledger](coverage.md#published-declaration-and-statement-review) |
| 15: functions and classes | Pending | |
| 16: Scripts and Modules | Pending | |
| 17–18: language extensions and standard built-in conventions | Reviewed | [Published host policies](design.md#published-host-policies); [intrinsic inventory](coverage.md#published-intrinsic-inventory-and-collection-roots) |
| 19: global object | Pending | |
| 20: fundamental objects | Pending | |
| 21: Numbers and Dates | Pending | |
| 22: text processing | Pending | |
| 23: indexed collections | Pending | |
| 24: keyed collections | Pending | |
| 25: structured data | Pending | |
| 26: managing memory | Pending | |
| 27: control abstraction objects | Pending | [Generator/Promise milestone review](coverage.md#generator-promise-and-suspension-review) records the completed implementation components |
| 28: reflection | Pending | |
| 29: memory model | Pending | |
| Annex A: grammar summary | Pending | |
| Annex B: optional web features | Reviewed | [Published host policies](design.md#published-host-policies); disabled extensions remain excluded whole |
| Annex C: strict mode | Reviewed | [Declaration/reference review](coverage.md#published-declaration-and-statement-review); strictness remains independent of Script/Module goals |
| Annex D: host layering | Reviewed | [Published host-hook choices](design.md#published-host-policies) |
| Annexes E–F: historical compatibility notes | Pending | |

## Types, values and specification records

| Group | Implementation | Regression evidence |
| --- | --- | --- |
| 6.1: language values and identity | Exhaustive [Value](../crates/spite-runtime/src/value.rs) variants; lossless Core Strings; [Symbol identities](../crates/spite-core/src/symbol.rs); checked heap handles | Primitive identity, UTF-16/lone-surrogate, Symbol registry/weak lifetime and foreign/stale-handle cases |
| 6.1.6: Number and BigInt operations | Binary64 arithmetic with explicit special exponent cases, exact mixed comparison, [BigInt arithmetic](../crates/spite-bigint/src) and checked native capacity/work accounting | Arithmetic/equality/relational originals; BigInt division/bitwise/shift tests; signed zero, NaN, infinity, subnormal and rounding boundaries |
| 6.1.7: properties and internal methods | String/Symbol property keys, complete data/accessor properties and partial [descriptors](../crates/spite-runtime/src/object/descriptor.rs), ordinary/exotic dispatch | Descriptor compatibility, reflection, integer key order, Proxy invariants, String/Array/TypedArray/Module exotic cases |
| 6.2: internal records, Lists, references and completions | Native records/Vecs, [environments](../crates/spite-runtime/src/environment.rs), [references](../crates/spite-runtime/src/reference.rs) and source continuation/completion state | TDZ versus initialized undefined, missing descriptor fields versus explicit undefined, cached references across effects/suspension, Empty versus undefined completions and targeted abrupt control |
| 6.2: private names and Data Blocks | Distinct private identities/slots; ordinary and safely shared byte storage | Private brands and shadowing, nonextensible objects, byte codecs, shared backing identity, bounds/detachment/resize and native-thread atomic tests |

String equality/ordering uses code units without normalization. Number SameValue
equates NaNs while distinguishing signed zeros; Symbol identity is independent
of its description. Reference state preserves the original receiver and deferred
property-key conversion, including across suspension. Internal empty, absent and
uninitialized states remain separate from JavaScript undefined. Host capacity or
work exhaustion cannot become an ordinary language throw or a negative pass.

## Abstract operations

The review follows observable conversion/property/iterator order and abrupt
completion rules, including operations shared by several syntax or built-in
entry points. Internal records and Lists use native representations; an
operation does not require a separate JavaScript-visible helper.

| Group | Implementation | Regression evidence |
| --- | --- | --- |
| 7.1: primitive conversion, numeric/String conversions, indices and lengths | [Primitive values](../crates/spite-runtime/src/value.rs), [object conversion/property keys](../crates/spite-runtime/src/realm_object.rs), [binary element conversion](../crates/spite-runtime/src/binary_data.rs), BigInt arithmetic and caller length/index checks | object_coercion, numeric_parsing, bigint_api, number, primitive_properties and TypedArray/DataView cases; explicit rounding and signed-zero boundaries |
| 7.2: type tests, identity, equality and ordering | Value equality/comparison plus ordered Realm conversion and Proxy-aware brand/callability/constructor checks | [Mixed comparison review](coverage.md#primitive-and-mixed-comparison-review), instanceof/Proxy cases, intrinsic callable metadata and well-formed String/Module-name cases |
| 7.3: property, call/construct, integrity and copying operations | [Realm property dispatch](../crates/spite-runtime/src/realm_object.rs), [Proxy dispatch](../crates/spite-runtime/src/realm_proxy.rs), [Object operations](../crates/spite-runtime/src/function/object), invocation/instanceof/species paths | object_descriptors, object_integrity, object_assign, rest/spread, proxy, Reflect and cross-realm construction cases |
| 7.3: private elements, fields and grouped collections | [Private slots](../crates/spite-runtime/src/object/private.rs), shared class builder, [Object grouping](../crates/spite-runtime/src/function/object/group_by.rs) and [Map grouping](../crates/spite-runtime/src/function/map/group_by.rs) | Private field/method/accessor cases; object_group_by and Map grouping; safe-integer guard and iterator-closing units |
| 7.3: options objects and protected prototype setters | [Base64 options](../crates/spite-runtime/src/function/uint8_base64.rs), [Iterator setters](../crates/spite-runtime/src/function/iterator/tag.rs) | Undefined versus null/primitive options; inherited options; inherited descriptor suppression, own setters, Proxy failures and strict failed writes |
| 7.4: iterator acquisition, stepping, closing and result/List records | [Shared iterator operations](../crates/spite-runtime/src/function/iterator/operations.rs), helper/concat closing, [async-from-sync](../crates/spite-runtime/src/function/async_iterator/from_sync.rs) and suspended async closing | Iterator helper, destructuring, loop, collection constructor, Promise aggregation, yield delegation and for-await-of cases |

Conversion hooks preserve the requested hint and original receiver. Object
conversion precedes BigInt/String ordering. Iterator records capture next once;
stepping failures update done state, and a throw completion retains precedence
over language failures from return. Host aborts remain distinct. Normal closing
validates the return result, and async closing awaits it. Copy/group operations
retain key/value observation order and close only active iterators on the
specified failure paths.

Validation includes the unchanged whole-source baseline, exact parse-negative
reviews, native conversion/descriptor/iterator cases and the targeted comparison
and queued-argument regressions recorded in coverage. Esid references help locate
tests; a matching reference alone does not establish algorithm coverage.
