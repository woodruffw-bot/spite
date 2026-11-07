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
| 13: expressions | Reviewed | Reference/value evaluation and operator groups below |
| 14: statements and declarations | Reviewed | [Scope, control and completion ledger](coverage.md#published-declaration-and-statement-review) |
| 15: functions and classes | Reviewed | Function creation, parameters, bodies and class evaluation below |
| 16: Scripts and Modules | Reviewed | Instantiation, linking, evaluation and host requests below |
| 17–18: language extensions and standard built-in conventions | Reviewed | [Published host policies](design.md#published-host-policies); [intrinsic inventory](coverage.md#published-intrinsic-inventory-and-collection-roots) |
| 19: global object | Reviewed | Global bindings, eval, numeric prefix parsing and URI operations below |
| 20: fundamental objects | Reviewed | Object/Function/Boolean/Symbol/Error construction and methods below |
| 21: Numbers and Dates | Reviewed | Number/BigInt conversion, Math special values and Date normalization below |
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
