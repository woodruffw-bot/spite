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
| 8: syntax-directed operations | Reviewed | Scope, containment, naming and tail-position operations below |
| 9: execution contexts and environments | Reviewed | Environment, Realm and host-job boundaries below |
| 10: ordinary and exotic objects | Reviewed | Internal-method and descriptor dispatch below |
| 11: source text | Reviewed | Lossless representation and independent grammar goals below |
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

## Syntax-directed operations

| Group | Implementation | Regression evidence |
| --- | --- | --- |
| Declared/bound names and scope traversal | [AST name traversal](../crates/spite-parser/src/ast.rs), shared Parser scope validators and Module import/export records; nested function/static-block boundaries start fresh scopes | var/lexical/catch/parameter patterns, block redeclarations, loop headers, switch scope and Module binding originals |
| Containment, assignment targets and private-name validity | Parser expression/cover validation, separate private-name scopes and class initialization context flags | Assignment/delete/update/for-head early errors; private-name shadowing/rollback; ContainsArguments crosses arrows and computed names but excludes ordinary function bodies and labels |
| Simple parameters, expected argument counts and strict directives | Parameter metadata and function validators; exact unescaped String directive recognition | Rest/default/destructuring length, unique/duplicate parameters, inherited strictness and independent dynamic parameter/body goals |
| Function definitions, naming and source identity | Shared named evaluation and retained FunctionSource; computed property/function names | Function name/length/toString, inferred names in defaults/assignments/fields, symbol property names and UTF-16 source originals |
| Label targets and tail positions | Active label/iteration validation and [strict tail-expression dispatch](../crates/spite-runtime/src/function/tail_call.rs) | Local control boundaries, logical/conditional/comma/optional/tagged tails, direct-eval handling and native/ordinary call transfer cases |

The class-initialization regression specifically permits literal and escaped
arguments labels and their break/continue targets in static blocks and field
initializer arrows. Published ContainsArguments tests IdentifierReferences,
which these label positions do not contain. The existing negative cases still
reject actual arguments references in both initialization forms.

## Execution contexts and environments

| Group | Implementation | Regression evidence |
| --- | --- | --- |
| Declarative, function, object and global environments | [Binding state](../crates/spite-runtime/src/environment.rs), reference resolution, declaration instantiation and [eval](../crates/spite-runtime/src/eval.rs) | TDZ/mutability/deletability, parameter aliases, non-strict with/unscopables, global descriptors, eval conflicts and closures |
| Module and private environments | Live import cells, Module records and distinct private-name identities | Read-only live imports, cycles, namespace TDZ, private brands, nested shadowing and eval access |
| This, super and new.target | Ordinary/arrow invocation contexts, home objects and derived-constructor state | Uninitialized this, super receivers, lexical arrows, default parameters, fields/static blocks and dynamic function independence |
| Realm/context transitions | [Checked Realm entry/restoration](../crates/spite-runtime/src/function/realm.rs), inactive records and retained intrinsic graphs | Caller restoration after throws/host aborts, cross-realm constructors/prototypes, revoked callbacks and inactive-Realm collection |
| Host jobs and agent checkpoints | Shared FIFO Promise/module/notification jobs, explicit host servicing and kept-object checkpoints | Reaction/thenable ordering, async generators, top-level await, module requests, cross-agent wait/notify and child-Realm WeakRef lifetimes |

Job execution establishes the captured Realm/module context and restores the
caller afterward. Preflight failures retain queued work; started-job failures
have explicit reported outcomes. Evaluation/allocation never collect, while
explicit collection traces suspended state and all retained context graphs.
Kept objects belong to the Agent, including its child Realms, and hosts choose
the documented idle checkpoints.

## Ordinary and exotic objects

| Group | Implementation | Regression evidence |
| --- | --- | --- |
| Ordinary internal methods and property compatibility | [Object storage](../crates/spite-runtime/src/object/store.rs), partial/complete descriptors and Realm get/set/define/prototype dispatch | Descriptor, integrity, prototype and property-order cases; inherited accessors, receiver-sensitive writes, immutable prototypes and cycle checks |
| ECMAScript and built-in function internal methods | [Ordinary calls/construction](../crates/spite-runtime/src/function/ordinary.rs), builtin dispatch and function-Realm metadata | Strict/nonstrict this, parameter/body instantiation, call/construct capability, derived returns, new.target and cross-realm fallback |
| Bound functions and arguments exotics | [Bound calls](../crates/spite-runtime/src/function/bound.rs) and mapped parameter cells | Bound argument/receiver/new.target forwarding, live parameter aliases, deletion/accessor unmapping and descriptor changes |
| Array, String and integer-indexed exotics | [Array storage](../crates/spite-runtime/src/object/array.rs), String virtual indices and [TypedArray dispatch](../crates/spite-runtime/src/object/typed_array.rs) | ArraySetLength conversion/truncation, UTF-16 fixed properties, canonical numeric keys, invalid-index coercion, resize/detachment and alternate receivers |
| Module namespace exotics | [Namespace slots](../crates/spite-runtime/src/object/module_namespace.rs) and [live bindings](../crates/spite-runtime/src/module/namespace.rs) | Sorted exports, symbols, live aliases/TDZ, fixed prototype/extensibility and descriptor/write/delete restrictions |
| Proxy internal methods | [Iterative Proxy dispatch](../crates/spite-runtime/src/realm_proxy.rs) | All thirteen methods, trap lookup/argument order, frozen/nonextensible invariants, revoked state, cached call/construct capability and dependent consumers |

An additional 7,168-case descriptor audit agrees with Node for all selected
cases. It checks Object.defineProperty and Reflect.defineProperty through
Proxy-observed own/inherited descriptor fields, partial/mixed descriptors,
nonextensible targets, frozen signed-zero/NaN data and fixed accessors. Both the
resulting descriptor and the complete has/get/key-conversion observation order
are compared. Existing prototype tests also protect the ordinary cycle-check
boundary: it stops at a Proxy without invoking its getPrototypeOf trap.

## Source text

[SourceText](../crates/spite-parser/src/source.rs) retains scalar UTF-8 unchanged
and records each unpaired UTF-16 surrogate separately at its scanner-byte
offset. Scanner placeholders remain internal; literal contents, comments,
RegExp body/flags, raw/cooked templates, import strings and FunctionSource use
the original code points. Spans never split a scanner code point. Joins check
capacity and relocate surrogate offsets, preserving exact dynamic source.

Script, Module, eval and [dynamic function](../crates/spite-parser/src/dynamic_function.rs)
entry points use their specified grammar/strictness contexts. Dynamic parameter
and body texts validate independently before combined early errors; the required
body line feeds also prevent hashbang acceptance inside FunctionBody. Initial
Script/Module hashbang goals remain distinct from later lexical goals.

Evidence includes scripts_utf16, dynamic_source_utf16, lexical_goals, streaming,
RegExp source/metadata, tagged-template and dynamic-constructor cases, together
with the retained whole hashbang/comment/identifier/literal inventories.
HostHasSourceTextAvailable follows the recorded host policy; native/bound/proxy
toString results use the required implementation-defined native form.
