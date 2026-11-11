# Embedding guide

The `spite` crate re-exports the parser, runtime, and their public types. Use it
when embedding the interpreter; the [architecture guide](design.md#crates)
explains the lower-level workspace crates.

## Create a realm

The crates are not published to crates.io. For an application beside a local
checkout of this repository, add a path dependency:

```toml
[dependencies]
spite = { path = "../spite/crates/spite" }
```

A realm holds JavaScript state between evaluations. Provide a native thread stack
of at least 2 MiB:

```rust
use spite::{Realm, Value};

fn main() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let mut realm = Realm::default();
            realm.eval("let answer = 40").expect("initialize answer");
            let value = realm.eval("answer += 2").expect("evaluate expression");
            assert_eq!(value, Value::Number(42.0));
        })
        .expect("create interpreter thread")
        .join()
        .expect("interpreter thread panicked");
}
```

Independent realms have separate heaps. Object handles from one cannot be used
in another. The examples below assume a mutable `realm` inside an interpreter
thread.

Generate the API reference from the repository root with
`cargo doc --workspace --no-deps --open`.

## Errors

Evaluation returns a `Result<Value, Error>`. The error variants distinguish
language behavior from host failures:

| Variant | Meaning |
| --- | --- |
| `Parse` | A parser diagnostic, including syntax, unsupported syntax, or a parsing limit |
| `Exception` | An interpreter-created JavaScript exception |
| `Thrown` | A value thrown by JavaScript |
| `Unsupported` | An operation the interpreter does not implement |
| `Host` | A host operation failed |
| `Limit` | An execution or platform capacity limit was reached |
| `InvalidObject` | A foreign, stale, or otherwise invalid object handle |

JavaScript `try`/`catch` can handle language exceptions. Host failures,
unsupported operations, and limits leave the evaluation directly, bypassing
JavaScript `catch` and `finally` blocks. A parse diagnostic is distinct from a
language exception raised while evaluating valid source.

## Promise jobs

The library does not drain Promise jobs automatically. After evaluation, the host
can run them one at a time:

```rust
use spite::PromiseJobOutcome;

realm
    .eval("Promise.resolve(42).then(value => { globalThis.answer = value; })")
    .expect("queue reaction");

loop {
    match realm.run_next_promise_job().expect("run Promise job") {
        PromiseJobOutcome::NoJob => break,
        PromiseJobOutcome::Completed => {}
        PromiseJobOutcome::Reported(error) => panic!("Promise job failed: {error}"),
    }
}

assert_eq!(realm.eval("answer").unwrap(), spite::Value::Number(42.0));
```

`NoJob` means the queue is empty, not that every Promise has settled. The host
also decides how to report unhandled rejections; the default rejection tracker
does nothing. Atomics timeouts and finalization cleanup have separate host entry
points. Drive these only when JavaScript execution is idle.

## Rooting values and collecting garbage

The interpreter does not collect automatically during evaluation or allocation.
A long-running script can therefore use substantial memory before returning
control to the host.

Before collecting, root any object-valued results the host needs to retain,
including values carried by `Error::Thrown`. Bindings, intrinsics, and queued jobs
are traced automatically.

```rust
use spite::{JsString, Value};

let object = realm.eval("({ answer: 42 })").expect("create object");
let saved = realm.root_value(object, usize::MAX).expect("root result");

realm.clear_kept_objects().expect("finish host sequence");
realm.collect(usize::MAX).expect("collect garbage");

assert_eq!(
    realm
        .read_property(saved.value(), &JsString::from("answer"))
        .unwrap(),
    Value::Number(42.0),
);
```

A `RootedValue` keeps its value alive until the last copy of the root token is
dropped. Cloning the inner `Value` alone does not retain that root. The
`max_work` arguments to rooting and collection are explicit host work budgets;
`usize::MAX` supplies an unrestricted budget.

Call `clear_kept_objects` only after a whole synchronous host sequence, including
its callbacks, has finished. This releases temporary roots created by
`WeakRef.deref`; it does not release explicit root tokens. Collection can queue
finalization work, which the host may later run with
`run_finalization_cleanup_job`.

## Resource limits

Every field of `Limits` defaults to `None`. Opt into quotas appropriate to the
application:

```rust
use spite::{Limits, Realm};

let mut realm = Realm::new(Limits {
    max_steps: Some(100_000),
    max_source_bytes: Some(1_000_000),
    ..Limits::default()
});

assert_eq!(realm.eval("6 * 7").unwrap(), spite::Value::Number(42.0));
```

Work limits apply separately to each evaluation or job. Other options limit
strings, BigInts, argument lists, heap slots, and object properties. For source
parsed separately before evaluation, use the parser's source-limit API as well.

Native stack guards and platform capacity checks always apply. The runtime
allows up to 32 active call/construct frames and requires at least a 2 MiB native
stack. See [native constraints](coverage.md#native-constraints) for the parser
guards and collection constraints.

## Modules

The library supplies module records and evaluation, but leaves loading policy to
the host. A typical loader:

1. Creates records with `create_module` or `create_module_from_ast`.
2. Calls `load_module_graph` with a resolver that returns reusable module records.
3. Calls `evaluate_module_record`, which links the graph and returns an evaluation
   Promise.
4. Runs Promise jobs and inspects that Promise's state.

Reuse records for repeated requests so shared dependencies and cycles refer to
the same module instance. Evaluating the same record again returns its cached
Promise without executing the body again. `ModuleHandle` keeps its record rooted.

Supported import-attribute keys are fixed when the realm is created, using
`new_with_supported_import_attributes`. The CLI is an example of a filesystem
loader; see [modules and imports](cli.md#modules-and-imports) for its policy.

## Time zones and waiting

A realm uses the host's system time zone by default. Use `set_time_zone` with a
`TimeZone` to select UTC, a fixed offset, or a named zone explicitly.

Ordinary realms cannot suspend in `Atomics.wait`. Hosts that support blocking can
create a realm with `Realm::new_suspending`.
