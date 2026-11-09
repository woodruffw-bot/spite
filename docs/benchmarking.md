# Manual benchmarks

Run the dependency-free benchmark example after the conformance gate passes:

```sh
cargo run -p spite-runtime --profile conformance --example bench --locked -- 7 10 > bench.csv
```

The optional positive arguments are sample count and iterations per sample;
defaults are seven and ten. The conformance profile optimizes execution while
keeping debug assertions and overflow checks enabled. Record the commit,
`rustc --version --verbose`, machine and profile alongside each result. Compare
revisions using the same toolchain/profile/machine and repeat runs to assess
variation.

The output is CSV. Durations are microseconds per operation, taking the median
of sample averages; even sample counts average the two middle values. Evaluation
and collection medians are computed independently.

| Row | One operation |
| --- | --- |
| parse_all | Parse and drop the ASTs for all 92 program workloads |
| initialize_and_drop | Create a fresh Realm, initialize it by evaluating an empty preparsed Script, then destroy it |
| bigint_bitnot_one | Complement -1n 2,048 times, checking canonical zero |
| bigint_bitnot_two | Repeat with -2n as a nonzero-result control, checking one |
| bigint_remainder_even | Divide signed even 4,097-bit values by ±2n across 2,048 quotient/remainder pairs, checking canonical zero |
| bigint_remainder_odd | Repeat with odd dividends, checking signed nonzero remainders |
| bigint_div_rem_one | Divide signed 4,097-bit values by ±1n across 2,048 quotient/remainder pairs, checking both results |
| bigint_div_rem_two | Repeat with ±2n as a nonunit control, checking halved magnitudes and signed remainders |
| json_reviver_whole | Parse a 4,098-unit quoted BMP/lone-surrogate primitive sixteen times, returning and checking its reviver source |
| json_reviver_padded | Repeat with outer whitespace so the reviver source remains a copied substring |
| bigint_multiply_one | Multiply signed 4,097-bit values by ±1n in both orders across 2,048 operations, checking the signed result |
| bigint_multiply_two | Repeat with ±2n as a nonunit control, checking doubled magnitudes |
| string_split_miss | Split a 16,385-unit BMP/lone-surrogate String sixteen times with an unmatched separator, checking the sole result |
| string_split_hit | Repeat with one leading separator, checking the 16,384-unit tail |
| bigint_or_xor_zero | Alternate OR/XOR with zero, both operand orders and positive/negative 4,097-bit values across 2,048 operations |
| bigint_or_xor_one | Repeat with 1n as a nonzero control, checking retained OR values and toggled XOR values |
| bigint_equal_and_or | Alternate equal positive AND and negative OR operands of 4,097 bits across 2,048 operations, checking each retained value |
| bigint_unequal_and_or | Repeat with unequal operands sharing their low words, checking the same results |
| bigint_xor_equal | XOR equal positive/negative 4,097-bit values 2,048 times, checking zero |
| bigint_xor_different | Repeat with unequal operands sharing their low words, checking the high-bit mask |
| array_locale_single | Stringify a one-element Array containing a 16,384-unit BMP/lone-surrogate String sixteen times with toLocaleString, checking length and final unit |
| array_locale_multiple | Repeat with two elements as a control |
| array_join_single | Join a one-element Array containing a 16,384-unit BMP/lone-surrogate String sixteen times, checking length and final unit |
| array_join_multiple | Repeat with two elements as a control |
| template_no_substitutions | Call a dynamically compiled function returning a 16,384-unit template without substitutions sixteen times, checking lengths |
| template_empty_substitution | Repeat with one empty String substitution as a control |
| tagged_template_creation | Compile and call sixteen fresh Function bodies with a 4,096-unit tagged template, checking cooked and raw lengths |
| sum_precise_integer | Sum four integers exactly 2,048 times with Math.sumPrecise, checking every result |
| arithmetic | Evaluate a 1,000-iteration numeric/bitwise loop |
| arguments_body_scan | Call default-parameter and arguments-named functions with two local lexical bindings 2,048 times each, checking the returned sum |
| lexical_bindings | Call a plain-parameter function with two local lexical bindings 2,048 times, checking the returned sum |
| var_bindings | Call an ordinary function with two local var bindings 2,048 times, checking the returned sum |
| radix_overflow | Convert an overflowing 8,192-digit hexadecimal string and its invalid-suffix variant sixteen times, checking Infinity and NaN |
| parse_int_overflow | Parse an overflowing 8,192-digit hexadecimal prefix and its non-digit suffix sixteen times, checking Infinity for both |
| parse_int_small | Parse the hexadecimal Strings for integers 0–1,023, checking their numeric sum |
| properties | Create, populate and read 128 computed object properties |
| unresolved_globals | Query typeof for an unresolved global 4,096 times, checking the count |
| string_repeat_single | Repeat a 16,384-unit BMP/lone-surrogate String once, sixteen times, checking length and final code unit |
| string_repeat_multiple | Repeat the same input twice as a copying control, checking each 32,768-unit result |
| string_raw_single | Call String.raw sixteen times over one 16,384-unit BMP/lone-surrogate literal, checking length and final code unit |
| string_raw_multiple | Repeat with two literals and an empty substitution, checking each 32,768-unit result as a concatenation control |
| string_search | Search eight times for a 129-unit needle near the end of a repeated-prefix String |
| string_box | Box a 2,048-unit BMP/lone-surrogate String eight times, checking lengths and outer indexed units |
| string_codes_single | Construct BMP and supplementary Strings from one numeric input 4,096 times each, checking decoded code points |
| string_codes_multiple | Repeat with multiple numeric inputs per construction as a control, checking lengths and decoded code points |
| string_character_single | Alternate at/charAt over 4,096 one-unit ASCII/BMP/lone-surrogate inputs, checking their numeric code units |
| string_character | Alternate at/charAt over 4,096 ASCII/BMP/lone-surrogate units, checking their numeric code units |
| string_slice | Alternate sixteen slice/substring copies from a 16,384-unit BMP/lone-surrogate input, checking lengths and edge units |
| string_slice_full | Alternate slice/substring of that complete input 64 times, checking length and final unit |
| string_normalize_ascii | Normalize a 16,384-unit ASCII String sixteen times across all four forms, checking lengths and final code units |
| string_iteration | Iterate a 768-unit ASCII/BMP/supplementary/lone-surrogate input, checking yielded lengths and first units |
| string_add | Append a four-unit BMP/lone-surrogate fragment 2,048 times with String addition, checking length and final unit |
| string_add_empty | Alternate empty left/right operands around a 16,384-unit String 64 times, checking length and final unit |
| string_concat_no_args | Call concat without arguments on a 16,384-unit BMP/lone-surrogate String 64 times, checking length and final unit |
| string_trim_noop | Alternate trim/trimStart/trimEnd on that already clean String 64 times, checking length and final unit |
| string_repeat | Repeat sixteen single-unit inputs 65,536 times each, checking lengths and final code units |
| string_repeat_wide | Repeat a four-unit BMP/lone-surrogate input 16,384 times, sixteen times, checking lengths and final code units |
| string_pad | Alternate padStart/padEnd to 65,536 units with default spaces sixteen times, checking lengths and outer code units |
| string_pad_wide | Repeat string_pad with a three-unit BMP/lone-surrogate filler and a partial final copy |
| string_wellformed | Run toWellFormed sixteen times over a valid 10,240-unit BMP/supplementary input, checking length and final unit |
| string_wellformed_mixed | Alternate an unpaired leading/trailing surrogate around that input, checking length and repaired unit |
| string_case_identity_ascii | Perform sixteen unchanged upper/lower conversions of 16,384-unit ASCII Strings, checking total length |
| string_case_ascii | Lowercase/uppercase a 12,288-unit ASCII input eight times each, checking lengths and selected units |
| string_case_context | Repeat the casing checks over 11,264 units of ASCII, Greek sigma, combining marks and lone surrogates |
| string_case_unicode | Repeat over 7,168 units of Greek, supplementary letters, expanding mappings and joiners as a Unicode control |
| object_group_by | Group 1,024 integers under distinct String keys, checking the key count and endpoint values |
| typed_array_indices | Construct and map 1,024-element Float64Arrays eight times, reduce each mapped result and check the total |
| object_group_by_few | Group 4,096 integers under four repeating String keys, checking group sizes and the final value |
| map_group_by | Repeat object_group_by with Map.groupBy as a control |
| map_iteration | Populate and iterate a 256-entry Map |
| function_source | Compile a function with a 10,240-unit BMP/supplementary comment, then stringify it sixteen times, checking lengths and final units |
| function_source_ascii | Repeat source stringification with a 16,384-byte ASCII comment |
| function_source_surrogates | Repeat function_source with a 12,288-unit comment containing 4,096 lone surrogates |
| bigint_to_number_integer | Convert a 513-bit BigInt to Number 2,048 times, checking the exact power of two |
| bigint | Perform 64 multiply/add/divide steps starting at 2^256 |
| bigint_and_zero | AND positive/negative 4,097-bit integers with 0n in 2,048 operations, checking zero |
| bigint_and_one | Repeat with 1n as a nonzero-mask control, checking one |
| bigint_bitwise | Perform 64 mixed-sign AND/XOR/OR and shift steps on 2,048-bit integers, checking a low-word checksum |
| bigint_divide_u64 | Perform 32 quotient/remainder steps by 2^63+17 from a changing 2,048-bit integer, checking a low-word checksum |
| bigint_radix | Format changing 2,048-bit integers in hexadecimal/base 32 across 32 steps, checking lengths and trailing-digit checksums |
| bigint_decimal | Format changing 2,048-bit integers in decimal across 32 steps, checking lengths and trailing-digit checksums |
| bigint_small_decimal | Format the integers 0–1,023 in decimal, checking lengths and parsed values |
| bigint_parse_decimal | Parse a 2,048-digit decimal String sixteen times, checking low-word checksums |
| bigint_parse_hex | Parse a 2,048-digit hexadecimal String sixteen times, checking low-word checksums |
| bigint_parse_small | Convert the decimal Strings for integers 0–1,023 to BigInt, checking their numeric sum |
| json_strings | Stringify a pair of 3,584-unit ordinary BMP strings sixteen times, checking lengths and closing quotes |
| json_escaped | Repeat json_strings with a newline, quote and lone surrogate appended to each input |
| uri_decode | Decode a long literal BMP prefix and trailing lone surrogate sixteen times, checking length and final unit |
| uri_decode_mixed | Alternate URI/component decoding of the same prefix ending in reserved and UTF-8 escapes, checking length and final unit |
| uri_encode | Encode a 12,309-unit unchanged URL/ASCII input sixteen times, checking length and final unit |
| uri_encode_mixed | Alternate URI/component encoding of a 12,288-unit ASCII prefix and a reserved/Unicode suffix sixteen times |
| regexp | Construct a RegExp literal and consume 128 named-capture matches |
| generator | Create, resume and consume a 128-value source generator |
| async_jobs | Execute 64 awaits, drain Promise jobs and read the completed sum |

Program rows reuse an initialized Realm and preparsed Script. One warm-up runs
before sampling. Every invocation checks its numeric result outside evaluation
timing and exits with an error on a wrong result or reported job failure.

After each invocation, the host clears kept objects and explicitly collects.
That time appears in collection_median_us and is excluded from median_us.
Both columns matter when assessing an embedding's complete host turn. Parse and
initialization rows report zero in the separate collection column; AST/Realm
destruction is already included in their main duration. All Realm quotas retain
their ordinary unlimited defaults.

These measurements guide profiling. They introduce no CI timing thresholds or
execution allowances and do not replace conformance or lifetime tests.

## Returned BigInt zero storage

Result validation now releases spare magnitude storage when an operation returns
zero. Internal normalization keeps scratch capacity available within arithmetic
loops. The native example uses public BigInt APIs and retains 512 results from
two 1,000,000-bit operands. AND produces zero; OR produces a nonzero control.

```sh
cargo build -p spite-bigint --profile conformance --example zero_storage --locked --offline
target/conformance/examples/zero_storage 1000000 512 and
target/conformance/examples/zero_storage 1000000 512 or
```

On Linux the example prints its own /proc VmHWM while the results are retained.
Matched Rust 1.99 executables use identical probe source and optimized
conformance libraries with debug/overflow checks. Baseline production is
ecfa84003. One fresh process per mode/revision produced:

| Native result mode | Peak RSS before (KiB) | Peak RSS after (KiB) |
| --- | ---: | ---: |
| AND, zero results | 64,012 | 1,492 |
| OR, nonzero results | 63,988 | 64,016 |

Peak RSS includes process setup and input construction. The deterministic unit
also checks that the returned general-AND zero owns no magnitude capacity and
retains its exact work debit. All 3,912 semantic cases and 12,576 budget
observations match before/after. These are native-component memory measurements,
with no Runtime throughput or wall-time claim.

## BigInt complement of minus one

After the existing word-work charge, complementing -1n returns canonical zero
through the result limit check. Baseline production is 7e0b4be8c. Matched Rust
1.99 executables use identical 92-workload sources and optimized conformance
libraries with debug/overflow checks enabled; both smokes pass all 94 rows.

Each instrumented process evaluates five times, complementing -1n 2,048 times
per evaluation and checking zero. The control complements -2n and checks one.
Counts include initialization, parsing/evaluation and collection.

| Workload | Instructions before | Instructions after | Rust deallocations before | Rust deallocations after |
| --- | ---: | ---: | ---: | ---: |
| Minus one | 269,824,001 | 267,671,579 | 54,666 | 44,426 |
| Minus-two control | 272,381,827 | 272,490,606 | 64,906 | 64,906 |

The target removes one temporary buffer per operation and uses 0.7977% fewer
instructions. The control uses 0.0399% more instructions with unchanged
deallocations. All 584 semantic cases and 360 budget observations retain their
before/after results. One instrumented process per revision/workload supports
these counts, with no wall-time or general speedup claim.

## Native BigInt zero construction

Converting native signed integer zero returns canonical empty BigInt storage
before creating a magnitude buffer. Baseline production is 0a966bd32. Matched
Rust 1.99 executables use identical 90-workload sources and optimized conformance
libraries with debug/overflow checks enabled; both smokes pass all 92 rows.

Each instrumented process evaluates five times, with 2,048 quotient/remainder
pairs on signed 4,097-bit dividends divided by ±2. Even dividends produce zero
remainders; odd control dividends produce ±1. These cases use the existing
word-division loop. Counts include initialization, parsing/evaluation and collection.

| Workload | Instructions before | Instructions after | Rust deallocations before | Rust deallocations after |
| --- | ---: | ---: | ---: | ---: |
| Zero remainder | 653,845,526 | 651,011,195 | 332,171 | 311,691 |
| Nonzero control | 660,898,481 | 660,918,947 | 357,776 | 357,776 |

The target removes one discarded zero buffer per division call and uses 0.4335%
fewer instructions. The control uses 0.0031% more instructions with unchanged
deallocations. All 3,840 semantic programs and 12,576 budget observations retain
their before/after results. One instrumented process per revision/workload
supports these counts, with no wall-time or general speedup claim.

## BigInt division by unit magnitudes

With work accounting disabled, division by ±1 copies the checked quotient and
returns canonical zero remainder. Accounted division retains the per-word loop
and partial refusals. Baseline production is 8a61c1f50. Matched Rust 1.99
executables use identical 88-workload sources and optimized conformance libraries
with debug/overflow checks enabled; both smokes pass all 90 rows.

Each instrumented process evaluates five times, with 2,048 quotient/remainder
pairs on signed 4,097-bit dividends and both divisor signs. The control divides
by ±2. Each evaluation makes 4,096 div_rem calls. Counts include initialization,
parsing/evaluation and collection.

| Workload | Instructions before | Instructions after | Rust deallocations before | Rust deallocations after |
| --- | ---: | ---: | ---: | ---: |
| Unit divisor | 653,026,658 | 606,236,813 | 332,141 | 311,661 |
| Nonunit control | 660,147,747 | 660,599,215 | 357,776 | 357,776 |

The target uses 7.1651% fewer instructions and removes one zero-remainder buffer
per division call. The control uses 0.0684% more instructions with unchanged
deallocations. All 3,840 semantic programs and 12,576 budget observations retain
their before/after results, including exact unspent work on refusal. One
instrumented process per revision/workload supports these counts, with no
wall-time or general speedup claim.

## Complete JSON reviver source

For a primitive spanning all input, context.source retains the converted String
after SameValue eligibility and with the existing output check. Partial spans
keep their checked copy. Baseline production is 7ced6d404. Matched Rust 1.99
executables use identical 86-workload sources and optimized conformance libraries
with debug/overflow checks enabled; both smokes pass all 88 rows.

Each instrumented process reads five copies of a Script, each in a fresh Realm
with the Test262 host. Each Script parses a 4,098-unit quoted BMP/lone-surrogate
primitive sixteen times and returns its source through the reviver. The control
adds outer whitespace. Counts include input/output, host initialization,
parsing/evaluation, job draining and explicit collection.

| Workload | Instructions before | Instructions after | Rust deallocations before | Rust deallocations after |
| --- | ---: | ---: | ---: | ---: |
| Whole input | 244,911,686 | 243,377,124 | 19,150 | 18,990 |
| Whitespace control | 245,149,985 | 245,150,046 | 19,280 | 19,280 |

The target removes two allocations per parse and uses 0.6266% fewer instructions.
The control adds 61 instructions with unchanged deallocations. All 762 distinct
semantic programs match before/after, ordinarily and with forced collection;
all 28 work thresholds are identical. One instrumented process per revision and
workload supports these counts, with no wall-time or general speedup claim.

## BigInt multiplication by unit magnitudes

After the existing width checks and work debit, multiplication by ±1 copies
the retained magnitude and applies the product sign. Baseline production is
81b56b271. Matched Rust 1.99 executables use identical 84-workload sources and
optimized conformance libraries with debug/overflow checks enabled.

Each instrumented process evaluates five times, multiplying signed 4,097-bit
values by both signs of one in either order across 2,048 operations. The control
uses both signs of two. Counts include initialization, parsing/evaluation and
collection.

| Workload | Instructions before | Instructions after | Rust deallocations before | Rust deallocations after |
| --- | ---: | ---: | ---: | ---: |
| Unit magnitude | 591,388,675 | 563,599,598 | 290,971 | 290,971 |
| Nonunit control | 595,846,442 | 596,872,721 | 291,156 | 291,156 |

The target uses 4.6990% fewer instructions; the control uses 0.1722% more.
Deallocation counts are unchanged. Both benchmark smokes pass all 86 rows;
2,688 semantic programs and 96 work probes remain exact before and after.
One instrumented process per revision/workload supports these counts, with no
wall-time or general speedup claim.

## String split without a match

For an unmatched nonempty separator, split retains the complete converted
String after the same output-length check and copy-work charge. Baseline
production is 15a685c21. Matched Rust 1.99 executables use identical 82-workload
sources and optimized conformance libraries with debug/overflow checks enabled.

Each instrumented process evaluates five times, splitting a 16,385-unit String
with BMP text and lone surrogates sixteen times per evaluation. The control
finds one leading separator and copies the remaining 16,384-unit tail. Counts
include initialization, parsing/evaluation and collection.

| Workload | Instructions before | Instructions after | Rust deallocations before | Rust deallocations after |
| --- | ---: | ---: | ---: | ---: |
| Unmatched separator | 229,832,340 | 227,022,745 | 5,411 | 5,331 |
| Leading-separator control | 230,415,048 | 230,277,911 | 6,051 | 6,051 |

The target removes one String allocation per split and uses 1.2225% fewer
instructions. The control retains its deallocation count with 0.0595% fewer
instructions. Both benchmark smokes pass all 84 rows. All 1,024 supplemental
programs agree with Python and Node, before/after and with forced collection;
all 21 work thresholds are identical. One instrumented process per revision
and workload supports these counts, with no wall-time or general speed claim.

## BigInt OR/XOR with zero

After the existing width-based work debit, OR/XOR with zero copies the retained
magnitude directly and validates the result limit. Baseline production is
3f6081fd6. Matched Rust 1.99 executables use identical 80-workload sources and
optimized conformance libraries with debug assertions and overflow checks.

Each instrumented process evaluates five times, alternating OR/XOR, operand
order and positive/negative 4,097-bit values across 2,048 loop operations. Two
additional XORs prepare expected values per evaluation. The control uses 1n.
Counts include initialization, parsing/evaluation and collection.

| Workload | Instructions before | Instructions after | Rust deallocations before | Rust deallocations after |
| --- | ---: | ---: | ---: | ---: |
| Zero operand | 535,750,843 | 502,542,859 | 239,961 | 229,711 |
| Nonzero control | 537,571,962 | 537,592,457 | 250,216 | 250,216 |

The target uses 6.1984% fewer instructions and removes one buffer per operation.
The control uses 0.0038% more instructions with unchanged deallocations. Both
benchmark smokes pass all 82 rows; 3,072 semantic programs and 72 work probes
remain exact before and after. One instrumented process per workload/revision
supports these counts, with no elapsed-time or general speedup claim.

## Equal BigInt AND/OR operands

The equal-operand branch now handles AND/OR with a checked operand copy,
retaining the existing work debit. Baseline production is 5e6966f89. Matched
executables use identical 78-workload sources, Rust 1.99, optimized conformance
libraries, debug assertions and overflow checks. Parser source is unchanged;
its binary rebuilds with its BigInt dependency.

Each Callgrind program evaluates five times, alternating positive AND and
negative OR across 2,048 operations on 4,097-bit magnitudes. Unequal control
operands share their low words and differ in a high-bit mask, producing the
same checked results. Counts include initialization, parsing/evaluation and
collection.

| Workload | Instructions before | Instructions after | Rust deallocations before | Rust deallocations after |
| --- | ---: | ---: | ---: | ---: |
| Equal operands | 464,314,973 | 428,757,009 | 198,451 | 188,211 |
| Unequal control | 469,922,293 | 471,024,087 | 198,746 | 198,746 |

The target uses 7.6582% fewer instructions and removes one buffer per operation.
The control uses 0.2345% more instructions and retains its deallocation count.
Both benchmark smokes pass all 80 rows; 3,072 semantic programs and all 72 work
allowances remain exact before and after. These counts support no elapsed-time
or general workload speedup claim and add no CI timing threshold.

## Equal BigInt XOR operands

After prepaying the existing width-based work, the integer backend returns
canonical zero for equal XOR operands. Baseline production is c59cf28f8.
Matched executables use identical 76-workload sources, Rust 1.99, optimized
conformance libraries, debug assertions and overflow checks. Parser source is
unchanged; its binary rebuilds with its BigInt dependency.

Each Callgrind program evaluates five times, with 2,048 XOR operations per
evaluation on positive/negative 4,097-bit magnitudes. Unequal control operands
share their low words and differ in a high-bit mask. Counts include
initialization, parsing/evaluation and collection.

| Workload | Instructions before | Instructions after | Rust deallocations before | Rust deallocations after |
| --- | ---: | ---: | ---: | ---: |
| Equal operands | 400,744,872 | 355,636,103 | 126,676 | 106,196 |
| Unequal control | 409,213,628 | 410,409,712 | 147,451 | 147,451 |

The target uses 11.2562% fewer instructions and removes two buffers per XOR.
The control uses 0.2923% more instructions and retains its deallocation count.
Both benchmark smokes pass all 78 rows; 3,072 semantic programs and all 24
measured work allowances remain exact before and after. These counts support
no elapsed-time or general workload speedup claim and add no CI timing threshold.

## Singleton Array.toLocaleString

The non-nullish singleton path reuses the method's converted result after the
existing observations and checks. Baseline production is 6298b20ef. Matched
executables use identical 74-workload sources, Rust 1.99, optimized conformance
libraries, debug assertions and overflow checks.

Each Callgrind program evaluates five times, with sixteen calls per evaluation
over a 16,384-unit BMP/lone-surrogate String. The control uses two elements.
Counts include initialization, parsing/evaluation and collection.

| Workload | Instructions before | Instructions after | Rust deallocations before | Rust deallocations after |
| --- | ---: | ---: | ---: | ---: |
| Singleton | 184,816,867 | 179,461,588 | 5,061 | 4,901 |
| Two-element control | 192,543,264 | 192,543,662 | 5,326 | 5,326 |

The target removes two buffers per call and uses 2.8976% fewer instructions.
The control changes by +0.0002%, with identical deallocations. All 540 semantic
programs meet published expectations before and after, including collection
repeats; only the documented ECMA-262/ECMA-402 argument count differs from Node.
Work allowances remain identical and both benchmark smokes pass all 76 rows.
These counts support no elapsed-time or general workload speedup claim.

## BigInt AND with zero

After prepaying the existing width-based work, the integer backend returns
canonical zero when either AND operand is zero. Baseline production is e8443d76a.
Matched executables use identical 67-workload sources, Rust 1.99, optimized
conformance libraries, debug assertions and overflow checks. Parser source is
unchanged; its binary rebuilds with its BigInt dependency.

Each dedicated Callgrind program evaluates five times, with 2,048 AND operations
per evaluation on positive/negative 4,097-bit magnitudes. The target uses 0n and
the control uses 1n. Counts include initialization, parsing/evaluation and collection.

| Workload | Instructions before | Instructions after | Rust deallocations before | Rust deallocations after |
| --- | ---: | ---: | ---: | ---: |
| Zero mask | 384,487,261 | 343,633,019 | 116,421 | 95,941 |
| One-mask control | 391,189,649 | 391,187,257 | 147,141 | 147,141 |

The target uses 10.6256% fewer instructions and 20,480 fewer Rust deallocations,
two per AND operation. The control instruction count differs by -0.0006% and
retains its deallocation count. Both benchmark smokes pass all 69 CSV rows;
1,536 coercion/error programs and the 1,024-program wide-integer oracle remain
exact before and after. These counts support no elapsed-time or general workload
speedup claim and add no CI timing threshold.

## One-unit character access

Successful at/charAt calls on a one-unit String retain the converted input after
index conversion and bounds checks (22.1.3.1–2). Baseline production is 4e41cf9c8.
Matched executables use identical 65-workload sources, Rust 1.99, optimized
conformance libraries, debug assertions and overflow checks.

Each dedicated Callgrind program evaluates five times, with 4,096 character
calls per evaluation. The target selects one-unit ASCII/BMP/lone-surrogate
inputs; the control indexes the existing four-unit String. Counts include
initialization, parsing/evaluation and collection.

| Workload | Instructions before | Instructions after | Rust deallocations before | Rust deallocations after |
| --- | ---: | ---: | ---: | ---: |
| One-unit inputs | 765,212,294 | 756,764,387 | 382,691 | 362,211 |
| Four-unit control | 639,028,393 | 638,913,389 | 228,936 | 228,936 |

The target uses 1.104% fewer instructions and 20,480 fewer Rust deallocations,
one per character call. The control instruction count differs by -0.018% and
retains its deallocation count. Both benchmark smokes pass all 67 CSV rows.
All 65,536 UTF-16 units and 1,428 coercion/error programs match independent
expectations before and after. These counts support no elapsed-time or general
workload speedup claim and add no CI timing threshold.

## Single-count String repetition

Nonempty input with a converted repetition count of one retains its immutable
String after the same length and work checks (22.1.3.18). Baseline production is
5122dd4ee. Matched executables use identical 64-workload sources, Rust 1.99,
optimized conformance libraries, debug assertions and overflow checks.

The dedicated Callgrind programs each evaluate five times, with sixteen calls
per evaluation on a 16,384-unit BMP/lone-surrogate input. Counts include Realm
initialization, parsing/evaluation and collection.

| Workload | Instructions before | Instructions after | Rust deallocations before | Rust deallocations after |
| --- | ---: | ---: | ---: | ---: |
| One repetition | 184,017,560 | 178,565,168 | 4,641 | 4,481 |
| Two-repetition control | 189,274,242 | 189,179,969 | 4,641 | 4,641 |

The target uses 2.963% fewer instructions and 160 fewer Rust deallocations.
The control instruction count differs by -0.050%. Both complete benchmark
smokes pass all 66 CSV rows, and all 288 coercion/error/UTF-16 cases match Node
before and after. Loaded smoke durations support no elapsed-time claim, and
these counts establish no general workload speedup or CI threshold.

## TypedArray observer reads

A candidate routed unaccounted at/reduce/reduceRight/join/toLocaleString reads
through the existing live element reader, retaining normal property dispatch
for work quotas and agent control. Compared Runtime revisions are 7fdb8d6cb and
a26ae4a0d, with identical 49-workload sources. The candidate's full 80,060-mode
gate compiles on 3126d87ce with identical production sources and passes before
timing.

Rust 1.99/conformance-profile measurements use Linux x86_64 on an Intel Xeon
Platinum 8573C, one CPU, no active builds/tests, twelve ABBA process runs, seven
samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| typed_array_indices | 35827.9 | 34082.3 | 7308.9 | 6926.5 |
| properties | 527.0 | 505.8 | 189.0 | 191.2 |
| map_iteration | 720.6 | 705.1 | 233.2 | 235.6 |
| string_slice_full | 220.3 | 220.0 | 170.8 | 172.4 |
| parse_all | 1241.3 | 1313.1 | 0 | 0 |
| initialize_and_drop | 639.4 | 644.0 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
Candidate evaluation is 4.87% lower and collection 5.23% lower. Other program
evaluation changes range from -10.53% to +7.21%, and collection changes from
-7.98% to +5.29%. Parsing and initialization differ by +5.78% and +0.72%.
The target changes fall within the control variation. These measurements do not
establish a speedup attributable to direct reads, so the extra dispatch branch
was removed. Production retains the common property path. The measurements add
no CI timing threshold or execution allowance.

## Whole-range String slicing

slice/substring retain their converted receiver when normalized endpoints select
the complete String, preserving endpoint coercion and output/work checks. The
added workload alternates the two methods 64 times on a 16,384-unit BMP/lone-surrogate
String. Baseline production is 824cfbf93 and candidate production is 7fdb8d6cb.
Both saved binaries use identical 49-workload sources. The candidate's full
80,060-mode gate passes before timing.

Rust 1.99/conformance-profile measurements use Linux x86_64 on an Intel Xeon
Platinum 8573C, one CPU, no active builds/tests, twelve ABBA process runs, seven
samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| string_slice_full | 332.6 | 218.7 | 195.2 | 175.7 |
| string_slice | 92.3 | 90.9 | 157.8 | 158.4 |
| string_concat_no_args | 196.9 | 196.6 | 163.4 | 164.5 |
| string_trim_noop | 228.2 | 235.0 | 164.7 | 169.0 |
| typed_array_indices | 34950.2 | 35104.5 | 6430.6 | 6637.8 |
| parse_all | 1235.9 | 1216.2 | 0 | 0 |
| initialize_and_drop | 638.0 | 665.6 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
Whole-range slicing evaluation is 34.25% lower and collection 9.96% lower in this
batch. Other program evaluation changes range from -8.03% to +7.34%, and collection
changes from -6.28% to +6.81%. Parsing and initialization differ by -1.59% and
+4.34%. These observations add no CI timing threshold or execution allowance.

## Unchanged TrimString storage

When trim/trimStart/trimEnd removes no units, TrimString returns its converted
receiver after the existing work and output checks, reusing immutable UTF-16
storage. The added workload alternates the three methods 64 times on a 16,384-unit
BMP/lone-surrogate String. Baseline production is e9b27f0b3 and candidate production
is 28ddd189f. Both saved binaries use identical 48-workload sources. The
candidate's full 80,060-mode gate passes before timing.

Rust 1.99/conformance-profile measurements use Linux x86_64 on an Intel Xeon
Platinum 8573C, one CPU, no active builds/tests, twelve ABBA process runs, seven
samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| string_trim_noop | 359.1 | 237.2 | 208.4 | 172.8 |
| string_concat_no_args | 194.5 | 201.8 | 166.0 | 171.4 |
| string_add_empty | 192.8 | 187.2 | 171.9 | 166.2 |
| string_slice | 100.5 | 96.6 | 168.3 | 159.5 |
| typed_array_indices | 35370.5 | 34599.5 | 7309.7 | 6855.7 |
| parse_all | 1242.1 | 1302.0 | 0 | 0 |
| initialize_and_drop | 645.8 | 673.6 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
Unchanged trimming evaluation is 33.95% lower and collection 17.07% lower in this
batch. Other program evaluation changes range from -5.02% to +5.94%, and collection
changes from -7.15% to +6.27%. Parsing and initialization differ by +4.82% and
+4.31%. These observations add no CI timing threshold or execution allowance.

## Argument-free String.concat

With no arguments, concat now returns its converted receiver after the existing
work/output checks, reusing immutable UTF-16 storage. The added workload calls
concat 64 times on a 16,384-unit BMP/lone-surrogate String and checks the result.
Baseline production is 469be7e01 and candidate production is 5996c936f. Both saved
binaries use identical 47-workload sources. The candidate's full 80,028-mode gate
passes before timing.

Rust 1.99/conformance-profile measurements use Linux x86_64 on an Intel Xeon
Platinum 8573C, one CPU, no active builds/tests, twelve ABBA process runs, seven
samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| string_concat_no_args | 364.4 | 191.3 | 184.4 | 158.9 |
| string_add_empty | 186.8 | 187.8 | 162.0 | 162.5 |
| string_slice | 91.6 | 92.7 | 153.1 | 154.5 |
| typed_array_indices | 35329.4 | 35781.9 | 7848.6 | 7826.0 |
| parse_all | 1170.9 | 1182.6 | 0 | 0 |
| initialize_and_drop | 618.8 | 622.2 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
Argument-free concat evaluation is 47.50% lower and collection 13.84% lower in
this batch. Other program evaluation changes range from -4.46% to +8.05%, and
collection changes from -7.67% to +3.11%. Parsing and initialization differ by
+1.00% and +0.54%. These observations add no CI timing threshold or execution
allowance.

## Direct TypedArray numeric-index classification

Canonical Array-index keys now use the existing checked UTF-16 parser before
general String-to-Number/Number-to-String conversion. Other numeric spellings
keep the existing conversion path. Baseline production is 6924e5b0d and candidate
production is e10e6a2ad; both saved binaries contain identical 46-workload sources.
The candidate passes the full 80,028-mode gate before timing.

Rust 1.99/conformance-profile measurements use Linux x86_64 on an Intel Xeon
Platinum 8573C, one CPU, no active builds/tests, twelve ABBA process runs, seven
samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| typed_array_indices | 45816.6 | 36814.1 | 9948.9 | 8193.2 |
| arithmetic | 1010.1 | 1002.5 | 275.0 | 269.7 |
| properties | 502.6 | 488.3 | 180.3 | 178.4 |
| object_group_by | 7707.0 | 7780.4 | 920.4 | 929.4 |
| parse_all | 1136.5 | 1110.6 | 0 | 0 |
| initialize_and_drop | 621.9 | 608.4 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
TypedArray evaluation is 19.65% lower and collection 17.65% lower in this batch.
Other program evaluation changes range from -9.88% to +2.43%, and collection
changes from -2.85% to +5.81%. Parsing and initialization differ by -2.28% and
-2.16%. These observations apply to this workload and host; they add no CI
timing threshold or execution allowance.

## Shared TypedArray index keys

Twenty-one TypedArray key conversions reuse the exact unsigned UTF-16 formatter.
The added typed_array_indices workload constructs and maps 1,024-element
Float64Arrays eight times, then reduces the mapped values to a checked total.
Baseline production is 2bb90c1d3 and candidate production is 4e5fea077. Both saved
binaries contain the same 46 program workloads. The candidate's full 80,028-mode
gate passed before timing.

Rust 1.99/conformance-profile measurements use Linux x86_64 on an Intel Xeon
Platinum 8573C, one CPU, no active builds/tests, twelve ABBA process runs, seven
samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| typed_array_indices | 46563.9 | 45655.8 | 10028.1 | 10180.5 |
| arithmetic | 978.1 | 968.1 | 273.1 | 262.8 |
| properties | 478.8 | 476.6 | 176.9 | 179.1 |
| object_group_by | 7348.6 | 7403.3 | 928.9 | 908.3 |
| parse_all | 1138.5 | 1143.2 | 0 | 0 |
| initialize_and_drop | 613.4 | 614.2 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
TypedArray evaluation is 1.95% lower and collection 1.52% higher. Other program
evaluation changes range from -12.89% to +12.50%, and collection changes from
-6.28% to +3.02%. Parsing and initialization differ by +0.41% and +0.13%.
The target difference lies within variation among unchanged workloads, so this
batch does not establish a speedup. It adds no CI timing threshold or execution
allowance.

## Object.groupBy result construction

The fresh ordinary result now appends GroupBy's already distinct keys without
another property scan when work accounting and agent control are absent.
Baseline production is 6eb513777. Both binaries use the same 45 program workloads,
Rust 1.99 and the conformance profile. Measurements use one CPU, no active
builds/tests, twelve ABBA process runs, seven samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| object_group_by | 9903.3 | 8317.1 | 1028.9 | 1049.5 |
| object_group_by_few | 43034.5 | 42467.0 | 4605.6 | 4756.0 |
| map_group_by | 6522.0 | 6449.3 | 972.7 | 978.1 |
| radix_overflow | 761.4 | 733.3 | 158.9 | 158.2 |
| parse_all | 1185.1 | 1139.1 | 0 | 0 |
| initialize_and_drop | 646.9 | 686.7 | 0 | 0 |

Values are microseconds, using medians of six process medians per revision.
Distinct-key evaluation decreases 16.02%; its collection time increases 2.00%.
The few-key workload decreases 1.32% and Map control decreases 1.12%, with
collection increasing 3.27% and 0.56%. Other program evaluation medians range
from -4.43% to +10.01%, and collection from -6.35% to +6.69%. These observations
support the distinct-key improvement without claiming a general speedup.

## Shared radix rounding for parseInt

Without work/magnitude quotas or agent control, parseInt in bases 2/8/16 shares
Core's exact radix-to-binary64 conversion. It avoids constructing a BigInt
magnitude after selecting the fully validated ASCII prefix. Prefix scanning,
coercions, negative zero and the other radix/quota paths retain their behavior.

The two added workloads measure long overflowing prefixes and small hexadecimal
values. Baseline production is 35b6fee20; both compared binaries contain identical
workloads. Rust 1.99/conformance-profile Linux x86_64 measurements use one CPU,
no active builds/tests, twelve ABBA process runs, seven samples and ten
iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| parse_int_overflow | 1178.5 | 778.7 | 154.5 | 152.1 |
| parse_int_small | 4134.7 | 4118.7 | 284.8 | 289.0 |
| radix_overflow | 686.9 | 702.4 | 149.1 | 150.7 |
| parse_all | 1067.0 | 1114.4 | 0 | 0 |
| initialize_and_drop | 615.9 | 602.0 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
Overflowing-prefix evaluation is 33.9% lower; small hexadecimal parsing differs
by -0.4%. Their collection differs by -1.6% and +1.5%. The unrelated
radix-overflow row differs by +2.3%; remaining program evaluation changes range
from -5.4% to +5.0%, and collection changes from -3.6% to +4.1%. Parsing and
initialization differ by +4.4% and -2.3%. These observations add no CI timing
threshold or execution allowance.

## Intrinsic Array index keys

CreateArrayFromList reuses exact decimal UTF-16 index construction instead of
formatting a temporary Rust String and encoding it into another temporary buffer.
Element definition and Array length handling continue through the same runtime
operations. The production change replaces one conversion call.

Baseline production is c42845b0d. Both compared binaries contain identical
forty-three program workloads. Rust 1.99/conformance-profile Linux x86_64
measurements use one CPU, no active builds/tests, twelve ABBA process runs,
seven samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| object_group_by | 9593.2 | 9787.2 | 969.9 | 938.6 |
| object_group_by_few | 40414.6 | 40906.1 | 4098.9 | 3770.5 |
| map_group_by | 6175.4 | 6129.9 | 923.8 | 913.6 |
| radix_overflow | 781.7 | 694.1 | 150.9 | 151.1 |
| parse_all | 1100.9 | 1072.4 | 0 | 0 |
| initialize_and_drop | 636.2 | 634.6 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
Distinct Object, four-group Object and Map evaluation differ by +2.0%, +1.2%
and -0.7%; their collection differs by -3.2%, -8.0% and -1.1%. The unrelated
radix-overflow row differs by -11.2%. Remaining program evaluation changes
range from -4.8% to +7.6%, and collection changes from -5.5% to +10.6%, with
RegExp collection at the upper end. Parsing and initialization differ by -2.6%
and -0.2%. These are mixed end-to-end timings for a shared-construction cleanup
with fewer temporary buffers. They add no CI timing threshold or execution allowance.

## Object.groupBy group lookup

A standard HashMap indexes already-converted temporary keys when work is
unaccounted and agent control is absent. The existing group vector preserves
order. Accounted/agent paths retain their charged scan, including when a
callback enables agent control. Result materialization still uses ordinary
property definition and can retain its own linear scans.

The three added workloads measure distinct Object keys, four repeating keys and
distinct Map keys. Baseline production is fd6e97d95. Both compared binaries
contain identical workloads. Rust 1.99/conformance-profile Linux x86_64
measurements use one CPU, no active builds/tests, twelve ABBA process runs,
seven samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| object_group_by | 12055.5 | 9386.2 | 947.3 | 952.5 |
| object_group_by_few | 38740.4 | 39504.7 | 3612.1 | 3902.1 |
| map_group_by | 6060.2 | 6094.0 | 903.3 | 922.4 |
| radix_overflow | 713.3 | 753.9 | 148.4 | 149.5 |
| parse_all | 1068.9 | 1081.1 | 0 | 0 |
| initialize_and_drop | 612.0 | 633.6 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
Distinct Object grouping evaluation is 22.1% lower; its collection differs by
+0.6%. Four-group evaluation differs by +2.0% and collection by +8.0%. The
Map control differs by +0.6% and +2.1%. The unrelated radix-overflow row differs
by +5.7%; remaining program evaluation changes range from -6.5% to +5.2%,
and collection changes from -2.9% to +5.0%. Parsing and initialization differ
by +1.1% and +3.5%. These observations add no CI timing threshold or execution
allowance.

## Stack-formatted String wrapper index keys

StringCreate formats index keys into a fixed twenty-unit UTF-16 stack buffer,
then copies the digits into standard Arc storage. It avoids temporary UTF-8
Strings and UTF-16 Vecs. The existing string_box workload covers 2,048 indices
per wrapper; every other workload is retained as a control.

Baseline production is 0f0857860. Both binaries contain identical workloads.
Rust 1.99/conformance-profile Linux x86_64 measurements use one CPU, no active
builds/tests, twelve ABBA process runs, seven samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| string_box | 1471.9 | 747.6 | 1143.2 | 1070.1 |
| string_character | 11940.5 | 11913.9 | 1600.3 | 1607.7 |
| radix_overflow | 690.4 | 773.7 | 153.6 | 153.9 |
| parse_all | 986.6 | 990.7 | 0 | 0 |
| initialize_and_drop | 626.7 | 620.2 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
Boxing evaluation is 49.2% lower and collection is 6.4% lower. The unrelated
radix-overflow row differs by +12.1%; remaining program evaluation changes range
from -3.1% to +10.0%, with mixed String repair at the upper end. Collection
changes range from -4.9% to +4.9%. Parsing and initialization differ by +0.4%
and -1.0%. These observations add no CI timing threshold or execution allowance.

## Checked String addition preallocation

Nonempty concatenation checks and reserves its complete output length before
the two bulk copies. Empty operands reuse the other immutable input. Runtime
coercions and output-length checks retain their existing order. Repeated
nonempty additions still copy eagerly and retain their quadratic total cost.

The two added workloads measure growing additions and empty operands.
Both compared binaries contain identical workloads. Baseline production is
f51c25bd9 (the following 854346fb8 changes documentation only); the candidate
changes only the Core concatenation helper. Rust 1.99/conformance-profile Linux
x86_64 measurements use one CPU, no active builds/tests, twelve ABBA process
runs, seven samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| string_add | 1948.5 | 1734.2 | 156.6 | 150.3 |
| string_add_empty | 357.7 | 186.2 | 186.6 | 161.9 |
| radix_overflow | 779.5 | 682.2 | 151.2 | 151.0 |
| parse_all | 994.8 | 998.4 | 0 | 0 |
| initialize_and_drop | 622.1 | 606.0 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
Growing-addition evaluation is 11.0% lower and empty-operand evaluation is 48.0%
lower. Their collection times differ by -4.0% and -13.2%. The unrelated
radix-overflow row differs by -12.5%; remaining program evaluation changes range
from -6.2% to +2.6%, and collection changes from -3.7% to +4.0%. Parsing and
initialization differ by +0.4% and -2.6%. These observations add no CI timing
threshold or execution allowance.

## Direct borrowed-slice UTF-16 String construction

Borrowed UTF-16 slices copy directly into owned Arc storage. The shared
substring/trim/split helper and String iterator yields avoid first creating
a Vec. Existing output/work checks and iterator boundaries are unchanged.

The two added workloads measure substring copying and String iteration.
Both compared binaries contain identical workloads. Baseline production is
8933cca32; the candidate adds slice conversion and updates the two construction
sites. Rust 1.99/conformance-profile Linux x86_64 measurements use one CPU,
no active builds/tests, twelve ABBA process runs, seven samples and ten
iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| string_slice | 105.6 | 91.5 | 154.0 | 153.4 |
| string_iteration | 1397.2 | 1406.3 | 353.4 | 339.8 |
| radix_overflow | 768.9 | 757.8 | 149.9 | 153.5 |
| parse_all | 912.7 | 916.0 | 0 | 0 |
| initialize_and_drop | 611.3 | 624.4 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
Slice/substring evaluation is 13.4% lower. String iteration differs by +0.7%
in evaluation and -3.9% in collection. The unrelated radix-overflow row differs
by -1.4%. Remaining program evaluation changes range from -1.3% to +9.9%, with
mixed String repair at the upper end; collection changes range from -4.3%
to +3.5%. Parsing and initialization differ by +0.4% and +2.1%. These
observations add no CI timing threshold or execution allowance.

## Direct fixed-array UTF-16 String construction

Fixed UTF-16 arrays use standard Arc conversion. Single-unit values produced
by String boxing, at/charAt and primitive indexed reads avoid first constructing
a Vec. Their immutable storage, contents and work/quota checks retain the
existing behavior.

The two added workloads measure boxing and character access. Both compared
binaries contain identical workloads. Baseline production is b53051bfe; the
candidate adds array conversion and updates the three construction sites.
Rust 1.99/conformance-profile Linux x86_64 measurements use one CPU, no active
builds/tests, twelve ABBA process runs, seven samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| string_box | 1702.3 | 1456.5 | 1201.1 | 1142.7 |
| string_character | 11640.3 | 11739.1 | 1442.7 | 1568.3 |
| radix_overflow | 693.5 | 683.5 | 149.3 | 150.5 |
| parse_all | 878.8 | 870.8 | 0 | 0 |
| initialize_and_drop | 599.3 | 618.4 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
Boxing evaluation is 14.4% lower and collection is 4.9% lower. Character-access
evaluation differs by +0.9% and collection is 8.7% higher. The unrelated
radix-overflow row differs by -1.4%. Remaining program evaluation changes range
from -2.1% to +3.6%, and collection changes from -1.0% to +6.1%. Parsing and
initialization differ by -0.9% and +3.2%. These observations add no CI timing
threshold or execution allowance.

## Word division for 64-bit BigInt magnitudes

Unaccounted division by two-word magnitudes carries the remainder in u128 and
processes one u32 magnitude word at a time. Each combined dividend fits 96 bits;
the quotient digit fits u32. Quotient storage and remainder construction remain
fallible. Explicit quotas keep their existing paths.

The added workload performs quotient and remainder operations across 32 changing
large dividends. Both compared binaries contain identical workloads. Baseline
production is ba1dc92b7 (the following c0f3e1ea9 changes documentation only);
the candidate adds only the two-word division path. Rust 1.99/conformance-profile
Linux x86_64 measurements use one CPU, no active builds/tests, twelve ABBA process
runs, seven samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| bigint_divide_u64 | 2710.5 | 182.4 | 156.4 | 150.7 |
| bigint | 132.1 | 132.9 | 154.6 | 152.6 |
| radix_overflow | 760.8 | 692.5 | 150.6 | 148.8 |
| parse_all | 801.9 | 842.8 | 0 | 0 |
| initialize_and_drop | 606.8 | 606.1 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
The 64-bit-divisor workload's evaluation is 93.3% lower. The small-divisor
control differs by +0.6%, and the unrelated radix-overflow row by -9.0%.
Remaining program evaluation changes range from -7.9% to +2.3%, and collection
changes from -6.2% to +6.3%. Parsing is 5.1% higher and initialization differs
by -0.1%. These observations add no CI timing threshold or execution allowance.

## Linear BigInt parsing in power-of-two radices

Larger unaccounted inputs in radices 2/4/8/16/32 validate their digits and pack
them directly into checked, fallibly reserved magnitude words. Leading zeroes
need no magnitude storage. Other radices retain grouped multiply/add, while
short and explicitly limited inputs keep the per-digit path.

This comparison reuses all existing workloads; no benchmark program changes.
Baseline production is 3f54035af; the candidate changes only power-of-two
parsing. Rust 1.99/conformance-profile Linux x86_64 measurements use one CPU,
no active builds/tests, twelve ABBA process runs, seven samples and ten
iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| bigint_parse_hex | 590.0 | 210.4 | 149.8 | 147.4 |
| bigint_parse_decimal | 446.8 | 441.7 | 151.1 | 149.0 |
| bigint_parse_small | 3071.9 | 3090.9 | 285.6 | 275.7 |
| radix_overflow | 752.0 | 717.1 | 150.7 | 150.4 |
| parse_all | 778.4 | 771.8 | 0 | 0 |
| initialize_and_drop | 611.8 | 610.7 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
Hexadecimal parsing evaluation is 64.3% lower. Decimal and small-input controls
differ by -1.1% and +0.6%; the unrelated radix-overflow row differs by -4.6%.
Remaining program evaluation changes range from -2.8% to +3.5%, and collection
changes from -5.3% to +1.6%. Parsing and initialization differ by -0.9% and
-0.2%. These observations add no CI timing threshold or execution allowance.

## Unchanged ASCII URI encoding prefixes

Without work accounting or agent control, URI encoding copies the initial
unchanged ASCII span in one checked append. A shared predicate preserves both
escape sets; remaining points retain Unicode validation and UTF-8 conversion.
Accounted and controlled calls keep their per-point scan/append checks.

The two added workloads encode unchanged and mixed inputs sixteen times.
Both compared binaries contain identical workloads. Baseline production is
4d03027de; the candidate adds prefix copying and a shared byte escape-set
predicate. Rust 1.99/conformance-profile Linux x86_64 measurements use one CPU,
no active builds/tests, twelve ABBA process runs, seven samples and ten
iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| uri_encode | 2967.9 | 390.2 | 163.7 | 159.0 |
| uri_encode_mixed | 2926.0 | 380.3 | 163.7 | 161.8 |
| radix_overflow | 707.8 | 758.3 | 155.7 | 151.5 |
| parse_all | 801.5 | 798.4 | 0 | 0 |
| initialize_and_drop | 628.2 | 601.5 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
Encoding evaluation is 86.9% and 87.0% lower. The unrelated radix-overflow
row is 7.1% higher in these executables; no cause is established. Remaining
program evaluation changes range from -5.4% to +2.8%, and collection changes
from -7.5% to +4.4%. Parsing and initialization differ by -0.4% and -4.2%.
These observations add no CI timing threshold or execution allowance.

## ASCII String case conversion

ASCII case mappings and their Cased/Case_Ignorable properties use standard
ASCII operations and the fixed ASCII property values from the pinned Unicode
data. Other scalars retain their generated tables. Work, termination and
output checks still run per point, including during sigma-context lookahead.

The three added workloads compare ASCII, mixed contextual text and a Unicode
control. Both compared binaries contain identical workloads. Baseline
production is 45e966271 (the following 5b8f5b35b changes documentation only);
the candidate changes only casing. Rust 1.99/conformance-profile Linux x86_64
measurements use one CPU, no active builds/tests, twelve ABBA process runs,
seven samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| string_case_ascii | 8136.4 | 3144.9 | 165.3 | 162.1 |
| string_case_context | 7604.0 | 4030.6 | 166.8 | 160.9 |
| string_case_unicode | 3891.6 | 3914.3 | 161.6 | 165.1 |
| radix_overflow | 713.6 | 785.0 | 153.0 | 159.6 |
| parse_all | 758.8 | 753.8 | 0 | 0 |
| initialize_and_drop | 613.4 | 614.4 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
ASCII and mixed-context evaluation is 61.3% and 47.0% lower. The Unicode
control differs by +0.6%. The unrelated radix-overflow row is 10.0% higher
in these executables; no cause is established. Remaining program evaluation
changes range from -8.8% to +4.9%, and collection changes from -4.0% to +3.1%.
Parsing and initialization differ by -0.7% and +0.2%. These observations add
no CI timing threshold or execution allowance.

## Grouped BigInt input parsing

Unsigned parsing groups inputs longer than 32 bytes into u32-sized values when
both work and magnitude quotas are disabled. Each group multiplies the current
magnitude by its radix power and adds the group's value, reserving possible
carry capacity fallibly. Short and explicitly limited inputs retain per-digit
validation and accounting.

The three added workloads parse decimal/hexadecimal inputs or small decimal
controls. Both compared binaries contain identical workloads. Baseline
production is 2b860ee04; the candidate changes only digit parsing. Rust
1.99/conformance-profile Linux x86_64 measurements use one CPU, no active
builds/tests, twelve ABBA process runs, seven samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| bigint_parse_decimal | 2568.2 | 448.3 | 162.0 | 158.4 |
| bigint_parse_hex | 3129.7 | 609.0 | 164.8 | 154.9 |
| bigint_parse_small | 3208.6 | 3203.4 | 299.3 | 294.4 |
| radix_overflow | 707.9 | 761.7 | 156.3 | 157.2 |
| parse_all | 685.4 | 690.5 | 0 | 0 |
| initialize_and_drop | 629.1 | 619.8 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
Large-input evaluation is 82.5% and 80.5% lower, and the small-input control
differs by -0.2%. The unrelated radix-overflow row is 7.6% higher in these
executables; no cause is established. Remaining program evaluation changes
range from -4.4% to +4.1%, and collection changes from -6.0% to +3.9%.
Parsing and initialization differ by +0.7% and -1.5%. These observations add
no CI timing threshold or execution allowance.

## Sequential lossless source conversion

Retained source conversion encodes scalar spans with the standard UTF-16
iterator and restores recorded lone surrogates in order. It performs one
metadata search for the selected range rather than one search per character.
Actual U+FFFD characters and supplementary pairs remain ordinary scalar text.

The two added workloads compile one function and read its exact source sixteen
times. Both compared binaries contain identical workloads. Baseline production
is 57f59da12; the candidate changes only source conversion. Rust 1.99/conformance
profile Linux x86_64 measurements use one CPU, no active builds/tests, twelve
ABBA process runs, seven samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| function_source | 797.7 | 620.8 | 164.3 | 159.3 |
| function_source_surrogates | 4793.1 | 879.8 | 174.2 | 161.4 |
| radix_overflow | 688.6 | 793.0 | 153.9 | 154.4 |
| parse_all | 639.1 | 623.1 | 0 | 0 |
| initialize_and_drop | 639.4 | 639.7 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
Source-workload evaluation is 22.2% and 81.6% lower. The unrelated radix-overflow
row is 15.2% higher in these executables; no cause is established. Remaining
program evaluation changes range from -2.8% to +3.8%, and collection changes
from -2.7% to +3.3%. Parsing and initialization differ by -2.5% and +0.04%.
These observations add no CI timing threshold or execution allowance.

## Trace constructor inlining

Three inline hints on Entry, ordinary-object and environment trace constructors
let optimized collection simplify the composed iterator layers across codegen
units. The traversal expressions and collection algorithm are identical.

The comparison below uses Rust 1.99, the conformance profile and Linux x86_64,
pinned to one allowed CPU. Twelve alternating process runs provide six runs per
revision, each with seven samples and ten iterations. Values are medians of the
per-process medians, in microseconds. The baseline library is 7fbb5a97c; the
candidate adds only those three compiler hints.

| Workload | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| arithmetic | 1032.9 | 1005.8 | 1079.6 | 280.7 |
| properties | 514.5 | 520.8 | 978.3 | 184.2 |
| map_iteration | 700.2 | 683.0 | 1032.3 | 214.9 |
| bigint | 262.6 | 251.7 | 970.1 | 155.0 |
| regexp | 606.4 | 614.5 | 1035.3 | 218.9 |
| generator | 425.6 | 414.8 | 1018.1 | 203.9 |
| async_jobs | 605.0 | 588.8 | 1066.3 | 244.0 |

Collection time falls 74–84% in these workloads. Evaluation changes range from
about -4% to +1.4%; the measurements establish a collection improvement on this
configuration. Parsing and initialization medians differ by -1.2% and +2.8%.

## String substring scanning

Unaccounted searches without an agent control use standard slice iterators over
the selected UTF-16 range. Searches with a work allowance or agent control keep
their existing per-candidate checks. The algorithm still compares candidate
slices and retains its quadratic worst case; it adds no allocation or dependency.

The added string_search workload checks eight searches of a 4,097-unit input for
a 129-unit needle at index 3,968. Both binaries include that same benchmark row.
The baseline production code is 8a691c5b1; the candidate adds the slice scan.
Rust 1.99/conformance-profile Linux x86_64 runs use one CPU, no active builds or
tests, and twelve ABBA process runs with seven samples and ten iterations each.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| string_search | 316.5 | 205.3 | 147.0 | 151.3 |
| parse_all | 150.2 | 160.6 | 0 | 0 |
| initialize_and_drop | 626.6 | 621.0 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
The search workload's evaluation is 35.1% lower; its collection is 3.0% higher.
Other program evaluation changes range from -1.9% to +1.3%. The parse row is
6.9% higher, although parser code is unchanged; initialization is 0.9% lower.
These observations describe this workload/configuration and add no CI threshold.

## BigInt division by a word

When a divisor's magnitude fits u32, the integer kernel divides one 32-bit
magnitude word at a time with a u64 carry. Larger divisors retain binary long
division. The bigint workload performs 64 multiply/add/divide steps with a
divisor of two, so it exercises the new path directly.

Rust 1.99/conformance-profile Linux x86_64 measurements use the same eight
program workloads, pinned to one CPU with no active builds or tests. Two batches
of twelve ABBA process runs provide twelve runs per revision, each with seven
samples and ten iterations. The table pools all 24 runs, retaining initialization
outliers. The baseline is e4bdb4fb2 production; the candidate changes only integer
division.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| bigint | 246.5 | 131.4 | 153.2 | 152.5 |
| parse_all | 161.8 | 162.1 | 0 | 0 |
| initialize_and_drop | 628.4 | 629.3 | 0 | 0 |

Values are microseconds, using medians of per-process medians. BigInt evaluation
is 46.7% lower and collection 0.4% lower. Other program evaluation changes range
from -1.2% to +2.5%; parsing and initialization differ by about +0.2% and +0.1%.
Work and result limits remain opt-in; this adds no timing gate or allowance.

## BigInt formatting in power-of-two radices

Radices 2, 4, 8, 16 and 32 now extract fixed-width digits from adjacent magnitude
words, including digits that cross word boundaries. This makes their formatting
linear in output length. The formatter reserves checked output capacity, charges
work before extraction and preserves sign, lowercase digits and minimal spelling.

The added bigint_radix workload formats hexadecimal/base-32 strings from an
integer starting at 2^2048 - 1, then multiplies/adds across 32 steps. Its checksum
checks both string lengths and trailing digits. Both compared binaries include
the same workload; baseline production is c564b9e72, and the candidate changes
the formatter. Rust 1.99/conformance-profile Linux x86_64 measurements use one
CPU, no active builds/tests, twelve ABBA process runs, seven samples and ten
iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| bigint_radix | 5167.5 | 467.8 | 172.9 | 161.4 |
| parse_all | 196.2 | 190.7 | 0 | 0 |
| initialize_and_drop | 638.8 | 632.0 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
The radix workload's evaluation is about 91% lower. Other program evaluation
changes range from -3.0% to +4.9%; parsing and initialization differ by -2.8% and
-1.1%. These observations add no CI timing threshold or execution allowance.

## Grouped BigInt formatting in other radices

Non-power-of-two radices divide larger magnitudes by the largest radix power
that fits u32, emitting one group per division. Lower groups retain leading
zeroes; the highest group has minimal spelling. One-word inputs retain their
single-digit path. Capacity and work checks remain active.

The added bigint_decimal workload formats a changing integer starting at
2^2048 - 1 across 32 steps; bigint_small_decimal checks 0–1,023. Both compared
binaries contain identical workloads. Baseline production is 455e965bf; the
candidate changes only non-power-of-two formatting. Rust 1.99/conformance-profile
Linux x86_64 measurements use one CPU, no active builds/tests, twelve ABBA
process runs, seven samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| bigint_decimal | 4849.5 | 871.9 | 178.5 | 161.7 |
| bigint_small_decimal | 3842.5 | 3855.8 | 512.9 | 510.6 |
| parse_all | 260.9 | 246.4 | 0 | 0 |
| initialize_and_drop | 680.7 | 649.9 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
Large-decimal evaluation is 82.0% lower; small-decimal evaluation differs by
+0.3%. Other program evaluation changes range from -7.6% to +1.9%; parsing and
initialization differ by -5.6% and -4.5%. These observations add no CI timing
threshold or execution allowance.

## JSON quoting of ordinary prefixes

QuoteJSONString copies its initial span of ordinary BMP code units with one
append. Escape and surrogate handling continue in the shared loop. The prefix
scan runs only without work accounting or agent control; their per-unit checks
retain the existing path. Output length and capacity checks remain active.

The added json_strings workload stringifies two 3,584-unit strings sixteen
times. json_escaped appends a newline, quote and lone surrogate to each input.
Both compared binaries include identical workloads. Baseline production is
555b72194; the candidate adds only prefix copying. Rust 1.99/conformance-profile
Linux x86_64 measurements use one CPU, no active builds/tests, twelve ABBA
process runs, seven samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| json_strings | 1250.8 | 242.8 | 165.4 | 164.2 |
| json_escaped | 1286.7 | 238.3 | 171.3 | 159.9 |
| parse_all | 324.1 | 314.9 | 0 | 0 |
| initialize_and_drop | 644.8 | 626.0 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
Evaluation is 80.6% and 81.5% lower for the two JSON workloads. Other program
evaluation changes range from -5.5% to +5.0%; parsing and initialization differ
by -2.8% and -2.9%. These observations add no CI timing threshold or execution
allowance.

## Batched String repetition

String.prototype.repeat copies its input once, then extends from the growing
output prefix using Vec::extend_from_within. The existing checked buffer covers
the complete result before copying starts. This keeps output-linear copying and
prepaid work accounting while reducing the number of append calls to logarithmic
in the repeat count. Arbitrary UTF-16 units retain their order.

The two added repeat workloads each create sixteen 65,536-unit results, using
single-unit inputs and a four-unit BMP/lone-surrogate input respectively. Both
compared binaries include identical workloads. Baseline production is 7b793527f;
the candidate changes only repeated copying. Rust 1.99/conformance-profile Linux
x86_64 measurements use one CPU, no active builds/tests, twelve ABBA process
runs, seven samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| string_repeat | 4437.3 | 217.4 | 179.9 | 170.5 |
| string_repeat_wide | 1234.4 | 208.2 | 172.3 | 170.3 |
| parse_all | 345.1 | 355.2 | 0 | 0 |
| initialize_and_drop | 635.7 | 631.8 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
Repeat evaluation is 95.1% and 83.1% lower. Other program evaluation changes range
from -9.2% to +4.6%; parsing and initialization differ by +2.9% and -0.6%.
These observations add no CI timing threshold or execution allowance.

## String well-formedness prefix handling

toWellFormed decodes until the first unpaired unit. Valid input returns the
existing String after its output-limit check. Repair reserves the unchanged
output length fallibly, copies the valid prefix, emits the first replacement
and continues the decoder with its retained lookahead. Input work charging is
unchanged.

Both compared binaries include identical valid and mixed-repair workloads over
a 10,240-unit BMP/supplementary base. Baseline production is 0e286d58b; the
candidate changes this scan/copy path. Rust 1.99/conformance-profile Linux x86_64
measurements use one CPU, no active builds/tests, twelve ABBA process runs,
seven samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| string_wellformed | 852.6 | 431.2 | 165.2 | 156.9 |
| string_wellformed_mixed | 887.7 | 742.8 | 165.8 | 170.7 |
| parse_all | 600.1 | 595.9 | 0 | 0 |
| initialize_and_drop | 622.1 | 626.3 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
Evaluation is 49.4% lower for valid input and 16.3% lower for mixed repairs.
Other program evaluation changes range from -3.8% to +6.7%, and collection
changes from -3.4% to +6.0%. Parsing and initialization differ by -0.7% and +0.7%.
These observations add no CI timing threshold or execution allowance.

## BigInt bitwise temporary allocation

Each two's-complement operand reserves its final width with try_reserve_exact,
then copies and extends the magnitude. This avoids growing a freshly cloned
vector, reports allocation refusal through the existing host-limit result and
retains the same word accounting and sign conversion.

The added bigint_bitwise workload performs 64 mixed-sign AND/XOR/OR and shift
steps from a 2,048-bit input, checking its low-word checksum. Both compared
binaries contain identical workloads. Baseline production is 6c03c45ad; the
candidate changes bitwise temporary allocation and checks width addition.
Rust 1.99/conformance-profile Linux x86_64 measurements use one CPU and no active
builds/tests. Two batches of twelve ABBA process runs provide twelve runs per
revision, each with seven samples and ten iterations; the table pools both batches.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| bigint_bitwise | 348.7 | 298.7 | 168.4 | 161.4 |
| radix_overflow | 980.0 | 1129.5 | 156.7 | 159.5 |
| parse_all | 425.6 | 408.2 | 0 | 0 |
| initialize_and_drop | 639.9 | 637.5 | 0 | 0 |

Values are microseconds, using medians of per-process medians. Bitwise evaluation
is 14.3% lower. The radix-overflow row is 15.3% higher in this executable, confirmed
in both batches. Its Core parser instructions match apart from address relocations;
the cause of the timing difference is not established. Other program evaluation
changes range from -2.8% to +5.7%, and collection changes from -4.8% to +5.7%.
Parsing and initialization differ by -4.1% and -0.4%. These observations add no
CI timing threshold or execution allowance.

## ASCII radix digit scanning

Core's radix integer parser iterates bytes during validation and accumulation;
accepted binary/octal/hexadecimal digits are ASCII. Complete validation still
precedes bounded accumulation and exact binary64 rounding.

Both compared binaries retain all seventeen program workloads, including the
checked radix_overflow row. Baseline production is 8e2091265; the candidate
changes only these iterators. Rust 1.99/conformance-profile Linux x86_64
measurements use one CPU, no active builds/tests, twelve ABBA process runs,
seven samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| radix_overflow | 1072.9 | 694.8 | 153.5 | 151.7 |
| parse_all | 396.5 | 405.5 | 0 | 0 |
| initialize_and_drop | 620.8 | 614.5 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
Radix conversion evaluation is 35.3% lower. Other program evaluation changes
range from -1.5% to +2.9%, and collection changes from -2.5% to +2.9%.
Parsing and initialization differ by +2.3% and -1.0%. These observations add
no CI timing threshold or execution allowance.

## Shared String padding and repetition copies

Padding reuses the growing-prefix copy operation used by repeat. Each append
records its start in the checked output buffer and copies only the appended
pattern. This preserves padEnd's receiver prefix and permits a partial final
copy of arbitrary UTF-16 units. Length, work and reservation checks precede copying.

Both compared binaries include identical default-space and mixed-filler padding
workloads and the existing repeat controls. Baseline production is 46b1685a3;
the candidate shares the copy helper and routes padding through it. Rust
1.99/conformance-profile Linux x86_64 measurements use one CPU, no active
builds/tests, twelve ABBA process runs, seven samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| string_pad | 2110.1 | 204.4 | 172.6 | 168.4 |
| string_pad_wide | 1870.8 | 199.1 | 175.9 | 168.1 |
| string_repeat | 216.3 | 213.6 | 166.1 | 168.9 |
| string_repeat_wide | 203.9 | 205.3 | 165.2 | 172.2 |
| radix_overflow | 706.6 | 781.8 | 155.3 | 151.6 |
| parse_all | 510.7 | 500.1 | 0 | 0 |
| initialize_and_drop | 625.0 | 626.2 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
Padding evaluation is 90.3% and 89.4% lower; repeat controls differ by -1.3% and
+0.7%. The radix-overflow row is 10.6% higher in this executable. Remaining
program evaluation changes range from -6.8% to +3.6%, and collection changes from
-5.0% to +2.8%. Parsing and initialization differ by -2.1% and +0.2%.
These observations add no CI timing threshold or execution allowance.

## URI decoding of literal prefixes

Without work accounting or agent control, decodeURI/decodeURIComponent copy the
literal UTF-16 prefix before the first percent sign in one checked append. Raw
lone surrogates remain literal units. Escaped UTF-8 validation and reserved-escape
handling retain their existing loop; accounted and agent-controlled calls keep
their per-unit checks.

The two added workloads decode sixteen strings with an 8,213-unit literal
prefix, followed by a lone surrogate or reserved/UTF-8 escapes. Both compared
binaries contain identical workloads. Baseline production is a738c8fa5; the
candidate adds only prefix copying. Rust 1.99/conformance-profile Linux x86_64
measurements use one CPU, no active builds/tests, twelve ABBA process runs,
seven samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| uri_decode | 1589.3 | 125.3 | 162.0 | 161.8 |
| uri_decode_mixed | 1598.7 | 130.9 | 159.5 | 161.0 |
| parse_all | 456.7 | 474.6 | 0 | 0 |
| initialize_and_drop | 634.4 | 631.8 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
URI evaluation is 92.1% and 91.8% lower. Other program evaluation changes range
from -2.9% to +7.9%, and collection changes from -6.2% to +3.7%.
Parsing and initialization differ by +3.9% and -0.4%. These observations add
no CI timing threshold or execution allowance.

## Radix Number overflow

Core's radix integer parser validates the entire input, then accumulates at most
1,025 significant bits before returning Infinity. Finite results and the overflow
midpoint retain the existing exact guard/sticky-bit rounding. Leading zeroes do
not count toward the significant-bit threshold.

The added radix_overflow workload converts an 8,192-digit hexadecimal string
and the same text with an invalid suffix sixteen times. It checks Infinity and
NaN separately. Both compared binaries contain identical workloads. Baseline
production is 64cbff78e; the candidate adds only the overflow return. Rust
1.99/conformance-profile Linux x86_64 measurements use one CPU, no active
builds/tests, twelve ABBA process runs, seven samples and ten iterations per sample.

| Row | Eval before | Eval after | Collection before | Collection after |
| --- | ---: | ---: | ---: | ---: |
| radix_overflow | 2162.3 | 1155.9 | 161.4 | 157.6 |
| parse_all | 396.2 | 401.7 | 0 | 0 |
| initialize_and_drop | 656.3 | 627.9 | 0 | 0 |

Values are microseconds, using medians of six per-process medians per revision.
The radix workload's evaluation is 46.5% lower. Other program evaluation changes
range from -8.2% to +2.6%; parsing and initialization differ by +1.4% and -4.3%.
These observations add no CI timing threshold or execution allowance.
