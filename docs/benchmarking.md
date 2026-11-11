# Benchmarking

Run the benchmark example with an optimized build that retains debug assertions
and overflow checks:

```sh
cargo run -p spite-runtime --profile conformance --example bench --locked -- 7 10 > bench.csv
```

The optional positive arguments select samples and iterations per sample.
Record the compiler, profile, machine and revision with results. Compare changes
on the same machine/toolchain and repeat runs to assess variation. Keep raw
measurements outside the repository; they are not project documentation.

Output is CSV. Evaluation and collection durations are reported separately in
microseconds, using medians of sample averages. Program cases reuse an initialized
Realm and preparsed Script, warm up before sampling, and validate results outside
the timed evaluation. Parse and initialization cases include destruction costs.

After each evaluation, the host clears kept objects and explicitly collects.
Collection time matters when comparing complete host turns, even though it is
excluded from evaluation time. Quotas retain their ordinary defaults.

Use profiling to justify optimizations. Benchmarks have no CI timing thresholds
and do not replace semantic, error-order or lifetime tests. The case definitions
and measured operations are documented in the benchmark example itself.
