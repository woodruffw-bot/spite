# Benchmarking

The runtime's [benchmark example](../crates/spite-runtime/examples/bench.rs)
measures parsing, realm initialization, evaluation, and garbage collection.
Run it with an optimized profile that retains debug assertions and overflow
checks:

```sh
cargo run -p spite-runtime --profile conformance --example bench --locked -- 7 10 > bench.csv
```

The optional positive arguments select the number of samples and iterations per
sample. Output is CSV; durations are microseconds, reported as medians of sample
averages.

## What is measured

Program cases reuse a realm and preparsed script, warm up before sampling, and
validate results outside the timed evaluation. After each evaluation, the host
clears kept objects and explicitly collects garbage. Evaluation and collection
times are separate, so include both when comparing a complete host turn.
Runtime resource quotas are disabled.

Parsing and initialization measurements include destruction costs. The example's
case definitions document the source programs and expected results.

## Compare changes

Use the same machine and toolchain, repeat runs to assess variation, and record
the compiler, profile, machine, and revision alongside results. Keep raw
measurements outside the repository.

Use profiling to identify the operation behind a timing change. Benchmarks have
no CI timing thresholds and do not replace semantic, error-order, or lifetime
tests.
