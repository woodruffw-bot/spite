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
| parse_all | Parse and drop the ASTs for all forty-seven program workloads |
| initialize_and_drop | Create a fresh Realm, initialize it by evaluating an empty preparsed Script, then destroy it |
| arithmetic | Evaluate a 1,000-iteration numeric/bitwise loop |
| radix_overflow | Convert an overflowing 8,192-digit hexadecimal string and its invalid-suffix variant sixteen times, checking Infinity and NaN |
| parse_int_overflow | Parse an overflowing 8,192-digit hexadecimal prefix and its non-digit suffix sixteen times, checking Infinity for both |
| parse_int_small | Parse the hexadecimal Strings for integers 0–1,023, checking their numeric sum |
| properties | Create, populate and read 128 computed object properties |
| string_search | Search eight times for a 129-unit needle near the end of a repeated-prefix String |
| string_box | Box a 2,048-unit BMP/lone-surrogate String eight times, checking lengths and outer indexed units |
| string_character | Alternate at/charAt over 4,096 ASCII/BMP/lone-surrogate units, checking their numeric code units |
| string_slice | Alternate sixteen slice/substring copies from a 16,384-unit BMP/lone-surrogate input, checking lengths and edge units |
| string_iteration | Iterate a 768-unit ASCII/BMP/supplementary/lone-surrogate input, checking yielded lengths and first units |
| string_add | Append a four-unit BMP/lone-surrogate fragment 2,048 times with String addition, checking length and final unit |
| string_add_empty | Alternate empty left/right operands around a 16,384-unit String 64 times, checking length and final unit |
| string_concat_no_args | Call concat without arguments on a 16,384-unit BMP/lone-surrogate String 64 times, checking length and final unit |
| string_repeat | Repeat sixteen single-unit inputs 65,536 times each, checking lengths and final code units |
| string_repeat_wide | Repeat a four-unit BMP/lone-surrogate input 16,384 times, sixteen times, checking lengths and final code units |
| string_pad | Alternate padStart/padEnd to 65,536 units with default spaces sixteen times, checking lengths and outer code units |
| string_pad_wide | Repeat string_pad with a three-unit BMP/lone-surrogate filler and a partial final copy |
| string_wellformed | Run toWellFormed sixteen times over a valid 10,240-unit BMP/supplementary input, checking length and final unit |
| string_wellformed_mixed | Alternate an unpaired leading/trailing surrogate around that input, checking length and repaired unit |
| string_case_ascii | Lowercase/uppercase a 12,288-unit ASCII input eight times each, checking lengths and selected units |
| string_case_context | Repeat the casing checks over 11,264 units of ASCII, Greek sigma, combining marks and lone surrogates |
| string_case_unicode | Repeat over 7,168 units of Greek, supplementary letters, expanding mappings and joiners as a Unicode control |
| object_group_by | Group 1,024 integers under distinct String keys, checking the key count and endpoint values |
| typed_array_indices | Construct and map 1,024-element Float64Arrays eight times, reduce each mapped result and check the total |
| object_group_by_few | Group 4,096 integers under four repeating String keys, checking group sizes and the final value |
| map_group_by | Repeat object_group_by with Map.groupBy as a control |
| map_iteration | Populate and iterate a 256-entry Map |
| function_source | Compile a function with a 10,240-unit BMP/supplementary comment, then stringify it sixteen times, checking lengths and final units |
| function_source_surrogates | Repeat function_source with a 12,288-unit comment containing 4,096 lone surrogates |
| bigint | Perform 64 multiply/add/divide steps starting at 2^256 |
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
