# Published specification review

Review against [ECMA-262 edition 17](https://262.ecma-international.org/17.0/)
and its [published es2026 source](https://github.com/tc39/ecma262/blob/0248456c758431e4bb8e5d26333ff1865123c9cd/spec.html).
This ledger connects the published clause groups to their implementation and
regression evidence. The completed review and final regression gate are recorded
in coverage. Exact fixture inventories and test receipts belong in
[coverage](coverage.md).
This clause review does not close the independent
[whole-upstream qualification](../tests/test262/audit/README.md). Its unresolved
scope entries must be reviewed before making a full-conformance claim.
The exact [published source identity](../tests/test262/audit/specification.json)
is verified by Git blob, SHA-256 and byte count in CI; an annotated local copy
cannot substitute for it.

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
| 13: expressions | Reviewed | Reference/value evaluation and operator groups below |
| 14: statements and declarations | Reviewed | [Scope, control and completion ledger](coverage.md#published-declaration-and-statement-review) |
| 15: functions and classes | Reviewed | Function creation, parameters, bodies and class evaluation below |
| 16: Scripts and Modules | Reviewed | Instantiation, linking, evaluation and host requests below |
| 17–18: language extensions and standard built-in conventions | Reviewed | [Published host policies](design.md#published-host-policies); [intrinsic inventory](coverage.md#published-intrinsic-inventory-and-collection-roots) |
| 19: global object | Reviewed | Global bindings, eval, numeric prefix parsing and URI operations below |
| 20: fundamental objects | Reviewed | Object/Function/Boolean/Symbol/Error construction and methods below |
| 21: Numbers and Dates | Reviewed | Number/BigInt conversion, Math special values and Date normalization below |
| 22: text processing | Reviewed | UTF-16 String operations, Unicode transforms and RegExp protocols below |
| 23: indexed collections | Reviewed | Array/TypedArray factories, sparse access, copying, sorting and byte codecs below |
| 24: keyed collections | Reviewed | Ordered Map/Set storage, set-like records and weak-key operations below |
| 25: structured data | Reviewed | Buffer/view witnesses, Atomics and JSON protocols below |
| 26: managing memory | Reviewed | WeakRef kept objects and FinalizationRegistry cleanup below |
| 27: control abstraction objects | Reviewed | Iterator, Promise and source-suspension operations below; [milestone evidence](coverage.md#generator-promise-and-suspension-review) |
| 28: reflection | Reviewed | Reflect, Proxy and Module namespace boundaries below |
| 29: memory model | Reviewed | Shared word storage, ordering and Agent boundaries below |
| Annex A: grammar summary | Reviewed | Production references cross-checked with the lexical, syntax and Pattern reviews below |
| Annex B: optional web features | Reviewed | [Published host policies](design.md#published-host-policies); disabled extensions remain excluded whole |
| Annex C: strict mode | Reviewed | [Declaration/reference review](coverage.md#published-declaration-and-statement-review); strictness remains independent of Script/Module goals |
| Annex D: host layering | Reviewed | [Published host-hook choices](design.md#published-host-policies) |
| Annexes E–F: historical compatibility notes | Reviewed | Referenced current algorithms and compatibility regressions below |

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

Array constructor reflection also traverses nested Proxies. The ordinary target
provides its complete own-key set for both forwarding and trap-invariant checks;
nonconfigurable keys and nonextensible targets retain the rules of 10.5.11.

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

## Expressions

| Group | Implementation | Regression evidence |
| --- | --- | --- |
| Literals, arrays, objects and templates | Shared literal creation, ordered property/spread operations, per-Realm template sites and retained raw/cooked source | Complete literal inventories, holes/spread, computed keys, shorthand/methods, duplicate prototype setters, tagged templates and lossless UTF-16 cases |
| References, member access, calls and new | Reference/value separation, cached property keys, captured receivers/callees, argument/spread accumulation and shared call/construct drivers | Primitive/super/private properties, key conversion, getter/setter order, eval, optional chains, constructor/new.target and cross-realm tests |
| Unary, update and numeric operators | Ordered GetValue/ToPrimitive/ToNumeric/PutValue, Number special cases and exact BigInt arithmetic | Delete/typeof reference boundaries, strict targets, prefix/postfix results, mixed numeric errors, exponent grammar, shifts and signed/nonfinite values |
| Equality, relational and conditional/logical operators | Primitive comparison plus ordered object conversion; lazy selected operands and branch completion | The 12,800-case comparison audit, exact BigInt/String conversion, custom hasInstance/in, nullish/logical grammar and side-effect order |
| Assignment and destructuring | Saved references, short-circuit writes, shared nested patterns, excluded property keys and iterator closing | Simple/compound/logical assignments, nested defaults/rest, elisions that skip value getters, target timing and abrupt cleanup |
| Await and yield in expressions | Explicit source-expression continuation tasks and nested-function boundaries | Suspended operands, calls/new, properties, optional chains, templates, assignments/patterns and partial class definitions; ordinary/async delegation |

The published GetValue/PutValue algorithms convert and cache a non-property-key
ReferencedName in the Reference Record. The implementation preserves this rule:
member evaluation retains the raw key, GetValue/PutValue perform the specified
conversion, and later writes use the cached key. Optional-chain short circuits
skip later operands; grouping retains its specified reference/value boundary.

## Functions and classes

| Group | Implementation | Regression evidence |
| --- | --- | --- |
| Ordinary, arrow, async and generator forms | Retained function kind/source, lexical versus ordinary contexts, shared closure/intrinsic builders and independent dynamic grammar goals | Declarations/expressions/methods, unique/default/rest/destructured parameters, inherited strictness, lexical this/super/new.target, name/length/toString and dynamic constructors |
| Parameter/body instantiation and completion | Separate parameter/body environments where required, mapped arguments, body declaration checks and shared invocation/tail transfer | Defaults and parameter aliases, eval in parameters/bodies, duplicate-name rules, hoisting, tail positions and error restoration |
| Class definition and element order | [Shared class builder](../crates/spite-runtime/src/function/class.rs), name TDZ, heritage/prototype checks, method installation and deferred static initialization | Heritage and constructor hooks, computed-name order, inferred/internal names, public/private fields/accessors/methods and static blocks |
| Construction and private elements | Base/derived this state, default constructors, dynamic super lookup, private identity/brand slots and scoped private environments | Primitive/object constructor returns, new.target/Proxy/bound forwarding, early super errors, private duplicate/accessor rules, shadowing and eval |
| Suspended function bodies | Shared source interpreter, serialized generator requests, intrinsic Promise reactions and protected completions | Lazy generator bodies, call-time parameter setup, next/return/throw, async rejection, await/yield delegation, scopes and finally/closing across suspension |

Class code is strict. Ordinary function bodies establish their own argument and
control contexts; arrows retain lexical context. Computed names use the enclosing
context, while class initialization enforces its own containment rules. Inferred
function/class names are available at the required point before static elements
run. Generator/async object algorithms also appear in clause 27's remaining
built-in review.

## Scripts and Modules

| Group | Implementation | Regression evidence |
| --- | --- | --- |
| Script/eval instantiation | Shared declaration analysis, persistent global environment and distinct direct/indirect eval contexts | Restricted global properties, var/function versus lexical declarations, deletable eval bindings, caller restoration and catchable syntax errors |
| Module records and linking | [Host graph linking](../crates/spite-runtime/src/module/link.rs), live import/export cells, namespace records and retained record identity | Duplicate/unresolved/ambiguous exports, star cycles/diamonds, function/var/lexical initialization and namespace TDZ/live aliases |
| Synchronous/asynchronous evaluation | [Persistent cycle roots](../crates/spite-runtime/src/module/evaluation.rs), async dependency/parent state and cached evaluation Promises | Dependency order, cycles with top-level await, repeated evaluation identity, language rejection, host abort and collection of retained state |
| import.meta and dynamic import | Per-record metadata plus [explicit import requests/completion](../crates/spite-runtime/src/module/dynamic.rs), fixed host attribute policy and intrinsic Promise continuation | Operand/ToString/options/attribute order, static versus dynamic errors, referrers, metadata identity, loaded-module errors and namespace resolution |

The host supplies graph sources and completes enabled dynamic-import requests.
An unconfigured host retains its documented unsupported boundary. Imported
syntax errors, link failures and language evaluation throws keep their specified
phase and completion identity. Original Module/helper graphs and execution flags
are retained in the baseline corpus.

## Global and fundamental built-ins

| Group | Implementation | Regression evidence |
| --- | --- | --- |
| Global values, constructors and namespaces | Complete intrinsic/global descriptors, property-backed bindings and retained Realm graphs | Global intrinsic metadata, mutation/deletion/shadowing, fixed NaN/Infinity/undefined and independent/child Realm identity cases |
| eval | Direct/indirect [eval execution](../crates/spite-runtime/src/eval.rs), independent Script parsing and caller/global declaration instantiation | Non-String identity, lexical/private/super/new.target contexts, deletable bindings, strictness and restored caller state |
| Numeric global functions | [UTF-16 prefix parsing](../crates/spite-runtime/src/function/number/parse.rs), shared Number aliases and numeric predicates | String-before-radix conversion, whitespace, signed zero, radix stripping, exponent rollback, rounding and surrogate/non-ASCII tails |
| URI functions | [URI/component codecs](../crates/spite-runtime/src/function/uri.rs), String-hint conversion, exact escape sets and strict UTF-8 decoding | Reserved-escape preservation, malformed/overlong/surrogate sequences, raw UTF-16, URIError identity and conversion order |
| Object construction/static/prototype methods | Intrinsic boxing/allocation, new.target fallback and shared property/integrity/copy/group operations | Generic receivers, descriptor/prototype audits, primitive Object APIs, assign/fromEntries/groupBy, own string/Symbol order and Proxy behavior |
| Function constructor/prototype methods | Independent dynamic source goals, captured call/apply/bind targets, strict poison accessors and ordinary hasInstance | Callable validation/argument order, array-like apply, bound construction/metadata, exact source retention, custom hasInstance and cross-realm fallback |
| Boolean construction and branding | [Boolean data slots](../crates/spite-runtime/src/function/boolean.rs), primitive versus boxed results and new.target prototype selection | Truthiness without coercion, borrowed methods, incompatible/Proxy receivers, subclassing and reflection |
| Symbol construction/prototype/registry | Primitive identities, description/value access, well-known Symbols and [append-only shared registry](../crates/spite-runtime/src/function/symbol/registry.rs) | Unique versus registered identity, for/keyFor conversion and type checks, realms/threads, wrapper brands and weak-key eligibility |
| Error and NativeError families | [ErrorData and construction](../crates/spite-runtime/src/function/error.rs), ordered message/cause installation and generic toString | Prototype/new.target behavior, inherited cause and Proxy options, conversion/throw order, branding, descriptors and intrinsic exception materialization |
| AggregateError | [IteratorToList and intrinsic result Array](../crates/spite-runtime/src/function/error/aggregate.rs), after prototype/message/cause setup | Iterable protocol, ordered hooks, step failures, exact element identities, subclassing and reflection |

Prototype hasOwnProperty/propertyIsEnumerable convert the key before boxing the
receiver; static Object.hasOwn uses its specified target-first order. Numeric
prefix parsers stop at non-ASCII/surrogate tails without discarding an earlier
valid prefix. Error construction checks prototype/message/cause/iteration in
the required order. Registered Symbols remain distinct from unique Symbols
with equal descriptions. Optional legacy properties follow the host policy.

## Numbers and Dates

| Group | Implementation | Regression evidence |
| --- | --- | --- |
| Number construction, predicates and branded methods | [Number intrinsics](../crates/spite-runtime/src/function/number.rs), exact decimal formatting and radix conversion | Number/number_formatting cases: omitted versus undefined arguments, BigInt construction, incompatible receivers, conversion order, signed zero, subnormals and 100-digit limits |
| BigInt construction and methods | [BigInt intrinsics](../crates/spite-runtime/src/function/bigint.rs), exact integer conversion and ordered width reduction | bigint_api/bigint_width cases: integral Number acceptance, String parsing, brands, radix bounds, ToIndex before ToBigInt and arbitrarily wide sign extension |
| Math constants and numeric methods | [Math dispatch](../crates/spite-runtime/src/function/math.rs), direct binary16 rounding, special exponent rules and compensated hypot | Math rounding/integer/extrema/power/root/log/trigonometric cases: signed zeros, infinities, NaNs, coercion order, exact powers of two and rounding boundaries |
| Math.sumPrecise and random | [Exact summation](../crates/spite-runtime/src/function/math/sum.rs), full iterator observation and Realm-specific random state | Exact cancellation/ties, noncoercing Number validation, continued iteration after nonfinite values, closing/error precedence, empty negative zero and random range/Realm independence |
| Date construction, UTC/parse and calendar operations | [Date dispatch](../crates/spite-runtime/src/function/date.rs), [Core calendar arithmetic](../crates/spite-core/src/date.rs) and exact MakeDay normalization | Date/UTC cases: ordered component conversion, copied Date slots, short-year adjustment, calendar rollover, enormous cancelling inputs and final TimeClip boundaries |
| Date methods and host policies | UTC/local getters and setters, branded time access, generic toJSON/toPrimitive and documented parsing/time-zone/locale policy | Date setter/getter/String cases: invalid-time recovery, conversion after NaN, prototype lookup order, DST boundaries, UTF-16 parse input and generic protocol hooks |

A supplemental 1,050-case decimal-formatting audit agrees with Node for results,
error kinds and argument-conversion observations. Fixed formatting checks its
precision range before returning a nonfinite spelling; exponential and
precision formatting retain their different specified ordering. Existing Date
regressions already protect mathematical month normalization and enormous
component cancellation before the final timestamp is clipped. Number-based
calendar and summation work does not acquire a JavaScript BigInt magnitude
quota merely because the implementation uses exact integer arithmetic.

Locale methods follow the documented ECMA-262 fallback without ECMA-402.
Implementation-approximated Math/radix results and host-dependent Date strings
are reviewed against their published requirements rather than requiring Node's
particular approximation or host configuration.

## Text processing

| Group | Implementation | Regression evidence |
| --- | --- | --- |
| String construction, wrappers and static methods | [String intrinsics](../crates/spite-runtime/src/function/string.rs), character encoding and array-like raw templates | String wrapper/character/raw cases: Symbol call special case, conversion before new.target prototype lookup, immutable UTF-16 indices and ordered literal/substitution reads |
| Character, sequence, search, repeat and padding methods | String modules operate on original UTF-16 units with explicit integer/length conversion and generic receivers | Character/sequence/search/predicate/repeat cases: lone surrogates, relative/clamped indices, empty searches, IsRegExp hooks, skipped fill conversion and invalid repetition counts |
| Unicode transforms and String iteration | Pinned Unicode 18 full case mappings, contextual Final_Sigma, iterative normalization, well-formedness and code-point iteration | Case/normalization/well-formedness/iterator cases: expansion, canonical ordering/blocking, Hangul, compatibility forms, surrogate preservation/replacement and paired iterator advancement |
| Pattern delegation and substitution | Ordered object-only Symbol hooks, intrinsic RegExpCreate fallback, UTF-16 split/replace and shared GetSubstitution scanner | String matching/split/replace/replaceAll cases: original receiver forwarding, global checks before delegation, conversion order, callback arguments and nonrecursive dollar expansion |
| RegExp syntax and matching | Parser validates published Pattern/flags; [shared regress adapter](../crates/spite-core/src/regexp_program.rs) preserves original UTF-16 ranges and captures | Retained whole Pattern originals and 2,617 Runtime RegExp groups, including UnicodeSets, lookaround, backreferences, modifiers, empty matches and undefined captures |
| RegExp construction and protocol methods | [RegExp intrinsics](../crates/spite-runtime/src/function/regexp.rs), native slots, live exec/species dispatch, lastIndex and match/iterator result construction | Construction/metadata/exec/match/search/replace/split/matchAll cases: identity shortcuts, ordered Gets/conversions, readonly state, custom results, captures/indices, species and iterator lifetime |

A supplemental 3,950-case String audit agrees with Node for UTF-16 character
access, forward/backward search, predicates, slicing, substring and splitting,
including result/error kinds and conversion-hook traces. It includes lone
surrogates, signed/nonfinite positions, BigInt/Symbol conversion failures and
empty strings/searches.

Published String pattern consumers consult Symbol hooks on object arguments.
Unicode normalization/casing uses pinned data; locale methods retain the
documented non-ECMA-402 policy and canonical-equivalence requirement. The sole
approved Unicode pair-start RegExpBuiltinExec resolution remains documented in
the design. Backend capacity/work interruption stays separate from Pattern
syntax errors and ordinary language exceptions.

## Indexed and keyed collections

| Group | Implementation | Regression evidence |
| --- | --- | --- |
| Array construction and factories | [Array factories](../crates/spite-runtime/src/function/array), distinct iterable/array-like paths and traced sequential fromAsync state | Array constructor/of/from/fromAsync cases: constructor arguments, captured iterator methods, mapper validation, awaits, closing and safe-integer limits |
| Array access, mutation and callbacks | Generic length snapshots, live HasProperty/Get/Set/delete, explicit callback order and shared species creation | Sparse/inherited elements, callback mutation, short circuits, search equality, reductions, front/range/splice and custom/cross-realm species cases |
| Array flattening, sorting, copies and iteration | Explicit flattening stack, stable fallible merge sort, intrinsic copies and live iterator lengths | Holes versus own undefined, stable ties, skipped replaced/discarded reads, nested depth, iterator reentry and TypedArray iterator witnesses |
| TypedArray construction and methods | [TypedArray overloads](../crates/spite-runtime/src/function/typed_array.rs), twelve element kinds, internal buffer witnesses and live indexed access | Primitive/iterable/buffer/typed-source order, content types, detached/resized views, species, callbacks, raw-byte copies, overlap, numeric sort and NaN/signed-zero cases |
| Uint8Array text codecs | [Hex](../crates/spite-runtime/src/function/uint8_hex.rs) and [Base64](../crates/spite-runtime/src/function/uint8_base64.rs), exact alphabets/chunks and partial-write records | Options/Get order, String brands, malformed input, strict overflow bits, capacity-limited prefixes, read/written counts and resized receivers |
| Map and Set constructors/keyed methods | Hash-indexed ordered storage, identity keys and canonical positive-zero insertion; cached constructor adders | Primitive/object/Symbol identity, NaN/zero, closing, overwritten entries, deletion/reinsertion, subclassing and complete reflection |
| Collection iteration and grouping/insertion | Live ordered cursors/forEach, intrinsic Map.groupBy and reentrant computed insertion | Mutation during traversal, permanently finished iterators, key order, callback throws/closing, intrinsic bypass and callback insertion/deletion/clear cases |
| Set-like combination and predicates | [Ordered Set records](../crates/spite-runtime/src/function/set/combine.rs), specified size branches, snapshot/live traversals and early closing | All seven operations, duplicate keys, size/has/keys lookup order, callback mutation, Infinity/fractional sizes and iterator close precedence |
| WeakMap and WeakSet | Checked object/non-registered-Symbol identities, non-retaining hash storage and ephemeron tracing | Key eligibility, incompatible receivers, constructor closing, weak computed insertion/reentry, collection fixed points and Symbol lifetime cases |

A supplemental 1,734-case Array/TypedArray audit agrees with Node on results,
errors, resulting source properties/values and observed Proxy/callback/coercion
traces. It exercises sparse/inherited/accessor/frozen sources, relative and
nonfinite indices, copying/mutation/search/callback methods and three numeric
TypedArray kinds. Sorting comparison order is implementation-defined; existing
regressions instead check stable ordering and the specified snapshot/write
boundaries.

Copying methods preserve their specified distinction between holes and own
undefined. TypedArray methods use internal witnesses rather than public length
properties, revalidate only at required steps and preserve raw representations
where required. Map/Set hashing does not call user coercion hooks; weak keys do
not become strong roots through the hash index.

## Structured data and memory management

| Group | Implementation | Regression evidence |
| --- | --- | --- |
| ArrayBuffer creation, resizing, slicing and transfer | [Buffer operations](../crates/spite-runtime/src/function/array_buffer.rs), checked sparse byte storage and detach state | Ordered length/options/prototype conversion, resize/detach during hooks, species-result validation, preserved copy ranges, transfer resizability and allocation-failure preservation |
| SharedArrayBuffer | [Shared construction/grow/slice](../crates/spite-runtime/src/function/shared_array_buffer.rs), shared native backing and monotonic published length | Shared brands/metadata, zero-filled growth, same-backing species rejection, receiving-Realm wrappers and concurrent native-thread visibility |
| DataView and binary encoding | [Live view witnesses](../crates/spite-runtime/src/function/data_view.rs) and element codecs for all eleven view types | All 22 get/set methods, unaligned endian access, integer/BigInt conversion, direct Float16 rounding, offset/value order and fixed/tracking resize/detach recovery |
| Atomics numeric operations | [Integer TypedArray validation](../crates/spite-runtime/src/function/atomics.rs), ordered index/operand conversion and atomic backing operations | All eight integer types, ordinary/shared buffers, modular operations, compareExchange, conversion errors, invalidated views and native-thread updates |
| Atomics waiting, notification and pause | Per-location FIFO native waiters, owning-Agent capabilities/jobs, condition variables and cooperative processor hints | Immediate/async/synchronous wait results, count/timeout conversion, race-safe timeout/notify, cross-agent delivery, termination and metadata-configured Test262 workers |
| JSON parsing and revivers | Independent ECMA-404 parser, intrinsic materialization and [iterative InternalizeJSONProperty](../crates/spite-runtime/src/function/json/reviver.rs) | Exact grammar/UTF-16, duplicate names, primitive source contexts, snapshotted children/live values, callback mutation, rejected updates and deep traversal |
| JSON stringify and raw values | Iterative serialization, ordered toJSON/replacer/wrapper processing, exact quoting and unforgeable frozen raw brand | Property lists, indentation, holes/omissions, BigInt hooks, raw spelling, cycles versus aliases, Proxy/errors and surrogate-safe quoting |
| WeakRef and kept objects | [Checked weak slots](../crates/spite-runtime/src/function/weak_ref.rs), Agent-wide kept roots and explicit idle checkpoints | Object/non-registered-Symbol targets, constructor/brand order, cleared targets, work-failure preservation and child-Realm lifetime |
| FinalizationRegistry | [Weak target/token cells](../crates/spite-runtime/src/function/finalization_registry.rs), strong held/callback captures and rooted cleanup jobs | Registration identity/order, unregister, target/held distinction, collection, reentry, queued callback lifetime and owning-Realm execution |

A supplemental 1,613-case JSON/DataView audit agrees with Node on results,
error kinds, callback/conversion traces and resulting bytes. It covers JSON
replacers/indentation, primitive reviver source contexts, callback mutation and
raw validation, plus unaligned integer/Float16/Float64 setters whose offset/value
hooks resize or detach fixed and tracking views.

Shared backing contains no JavaScript heap values. Wait notifications carry
native identities to the owning Agent, which retains and services its own
Promise capabilities. Weak cleanup remains host-scheduled and cannot run during
evaluation or collection. The production trace-inlining validation includes
all 302 unchanged admitted WeakRef/WeakMap/WeakSet/FinalizationRegistry sources,
executed in their 603 official runner modes with no failures or allowances.

## Control abstractions and reflection

| Group | Implementation | Regression evidence |
| --- | --- | --- |
| Iterator acquisition, wrappers and helpers | [Shared iterator records](../crates/spite-runtime/src/function/iterator/operations.rs), direct/flattenable acquisition, lazy helpers/concat and eager consumers | Cached next and receivers, validation/closing order, exact unbounded counters, skipped drop values, flatMap inner/outer closing, helper reentry and retained captures |
| Async iteration | [Async-from-sync adapter](../crates/spite-runtime/src/function/async_iterator/from_sync.rs), Promise continuation and source for-await-of | next/return/throw lookup, missing methods, value unwrapping, rejected sync values, closing precedence and job timing |
| Promise construction and reaction jobs | Capabilities, shared already-resolved state, thenable assimilation, species-aware then/catch/finally and FIFO Agent jobs | Executor/capability order, self-resolution, callback identity/Realm, reaction propagation, custom species and queued object/Symbol argument lifetimes |
| Promise static methods | Generic resolve/reject/try/withResolvers, ordered race/all/allSettled/any and per-input aggregation guards | Constructor resolve capture, iterable closing, synchronous custom thenables, repeated callbacks, ordered values/errors and empty inputs |
| Generator/AsyncGenerator/AsyncFunction objects | Intrinsic constructor/prototype graphs, shared traced source interpreter and serialized async-generator requests | Lazy generator bodies, call-time parameters, next/return/throw, yield delegation, protected finally, awaited return/yield, queue order and cross-realm resumption |
| Reflect | [All thirteen operations](../crates/spite-runtime/src/function/reflect.rs), explicit target/key/descriptor/list validation and receiver forwarding | Primitive rejection, absent versus explicit undefined, Boolean internal-method rejection, own String/Symbol keys, construction/new.target and Proxy consumers |
| Proxy and Module namespace objects | Proxy creation/revocation and cached capabilities; intrinsic per-Module namespace identity | Revocation without traps, all internal-method invariants, fixed namespace prototype/extensibility, sorted exports and live binding/TDZ behavior |

Suspended source frames retain references, partial results, environments and
pending completion targets until resumption or release. Dynamic constructors
parse independent goals. Host servicing advances Promise/async jobs explicitly;
language rejection and host abort remain separate outcomes. The ordinary/exotic
and Script/Module reviews cover the internal methods used by reflection.

## Memory model

| Requirement group | Implementation and evidence |
| --- | --- |
| Shared data-block identity and initialization | Shared backing identity survives native handle clones and receiving-Realm wrappers; sparse pages represent zero initialization, monotonic growth and stable owned pages |
| Unordered view access and no-tear configurations | Ordinary shared views and Atomics use the same native atomic words; aligned integer elements fit a word, masked updates preserve adjacent bytes, and permitted unaligned fragments retain original byte ranges |
| Sequentially consistent Atomics | Prepared Arc-owned accesses perform SeqCst loads/CAS after directory preparation; no storage lock/raw pointer is retained by the atomic memory step |
| Synchronization and Agent order | Wait registration compares under the waiter-list critical section; per-location FIFO notification, timeout/termination arbitration and owning-Agent job queues preserve the specified ordering |
| Candidate-execution constraints | Native atomic operations provide concrete executions subject to the specification's ordering/coherence/no-tear rules; abstract event/relation records need no JavaScript-visible object or stored execution graph |
| Regression evidence and platform boundary | Native store-buffering tests reject the forbidden both-zero result; first-touch/adjacent-update, raw/atomic visibility, copy destination isolation and isolated-Realm agent tests protect shared state. Eight-byte shared Atomics explicitly require a target with native 64-bit atomics |

The implementation may provide stronger ordering than an unordered access
requires. Tests sample concurrency outcomes; the ordering argument also depends
on the native atomics and ownership discipline, rather than treating sampled
executions as proof of every possible interleaving. Directory/growth/waiter
locks contain no user callbacks. Project code remains safe Rust with standard
atomic words, Arc and synchronization primitives.

## Grammar summary and historical compatibility notes

Annex A contains 381 production references to 376 distinct names. Its lexical,
numeric-String, RegExp and syntactic grammar groups refer back to the reviewed
Core/Parser implementations and clauses 12–16/22; it adds no independent
evaluation algorithm. Script/Module goals, parameterized await/yield/return/in
contexts, cover grammars and early errors retain their existing regressions.

Annex E's 12 and Annex F's 41 distinct clause links all resolve into the
reviewed current algorithms. Existing regressions cover own global-declaration
checks, Array length recapture, positive-zero TimeClip, Date parsing/invalid
formatting, RegExp source escaping and empty-match advancement, stable NaN
comparison, numeric-String radices, per-site templates, Unicode whitespace and
case mapping, lexical let/ASI, declaration restrictions and UpdateEmpty,
primitive Object APIs, bound/function/Error metadata and async job timing.
The historical notes do not reinstate superseded behavior or enable disabled
Annex B policies. Reference counts organize this cross-check; they do not
represent additional conformance tests.
