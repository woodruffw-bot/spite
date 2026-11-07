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
| parse_all | Parse and drop the ASTs for all seven program workloads |
| initialize_and_drop | Create a fresh Realm, initialize it by evaluating an empty preparsed Script, then destroy it |
| arithmetic | Evaluate a 1,000-iteration numeric/bitwise loop |
| properties | Create, populate and read 128 computed object properties |
| map_iteration | Populate and iterate a 256-entry Map |
| bigint | Perform 64 multiply/add/divide steps starting at 2^256 |
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
