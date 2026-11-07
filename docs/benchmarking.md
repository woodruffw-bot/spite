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
| parse_all | Parse and drop the ASTs for all nine program workloads |
| initialize_and_drop | Create a fresh Realm, initialize it by evaluating an empty preparsed Script, then destroy it |
| arithmetic | Evaluate a 1,000-iteration numeric/bitwise loop |
| properties | Create, populate and read 128 computed object properties |
| string_search | Search eight times for a 129-unit needle near the end of a repeated-prefix String |
| map_iteration | Populate and iterate a 256-entry Map |
| bigint | Perform 64 multiply/add/divide steps starting at 2^256 |
| bigint_radix | Format changing 2,048-bit integers in hexadecimal/base 32 across 32 steps, checking lengths and trailing-digit checksums |
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
