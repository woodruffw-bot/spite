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
| parse_all | Parse and drop the ASTs for all twenty-three program workloads |
| initialize_and_drop | Create a fresh Realm, initialize it by evaluating an empty preparsed Script, then destroy it |
| arithmetic | Evaluate a 1,000-iteration numeric/bitwise loop |
| radix_overflow | Convert an overflowing 8,192-digit hexadecimal string and its invalid-suffix variant sixteen times, checking Infinity and NaN |
| properties | Create, populate and read 128 computed object properties |
| string_search | Search eight times for a 129-unit needle near the end of a repeated-prefix String |
| string_repeat | Repeat sixteen single-unit inputs 65,536 times each, checking lengths and final code units |
| string_repeat_wide | Repeat a four-unit BMP/lone-surrogate input 16,384 times, sixteen times, checking lengths and final code units |
| string_pad | Alternate padStart/padEnd to 65,536 units with default spaces sixteen times, checking lengths and outer code units |
| string_pad_wide | Repeat string_pad with a three-unit BMP/lone-surrogate filler and a partial final copy |
| string_wellformed | Run toWellFormed sixteen times over a valid 10,240-unit BMP/supplementary input, checking length and final unit |
| string_wellformed_mixed | Alternate an unpaired leading/trailing surrogate around that input, checking length and repaired unit |
| map_iteration | Populate and iterate a 256-entry Map |
| bigint | Perform 64 multiply/add/divide steps starting at 2^256 |
| bigint_bitwise | Perform 64 mixed-sign AND/XOR/OR and shift steps on 2,048-bit integers, checking a low-word checksum |
| bigint_radix | Format changing 2,048-bit integers in hexadecimal/base 32 across 32 steps, checking lengths and trailing-digit checksums |
| bigint_decimal | Format changing 2,048-bit integers in decimal across 32 steps, checking lengths and trailing-digit checksums |
| bigint_small_decimal | Format the integers 0–1,023 in decimal, checking lengths and parsed values |
| json_strings | Stringify a pair of 3,584-unit ordinary BMP strings sixteen times, checking lengths and closing quotes |
| json_escaped | Repeat json_strings with a newline, quote and lone surrogate appended to each input |
| uri_decode | Decode a long literal BMP prefix and trailing lone surrogate sixteen times, checking length and final unit |
| uri_decode_mixed | Alternate URI/component decoding of the same prefix ending in reserved and UTF-8 escapes, checking length and final unit |
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
