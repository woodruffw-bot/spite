# Test262 fixtures

The files under `upstream/` are unchanged originals from the revision in
[REVISION](REVISION). The upstream license is in [LICENSE](LICENSE). Local
manifests select executable originals, support modules and harness helpers;
[audit records](audit/README.md) account for the rest of the pinned inventory.

## Running the corpus

```sh
cargo xtask test262
```

The conformance profile optimizes exhaustive tests while retaining debug and
overflow checks. Corpus workers use fresh Realms and ordinary native stacks.
Runtime quotas retain their defaults. The library and CLI also support serial
execution; parallel reports preserve manifest order and the same outcomes.

The runner interprets each original's frontmatter. Script strictness, raw source,
Module mode, asynchronous completion, includes and negative-test phase are not
inferred from file names or feature tags. It loads the upstream assertion harness
and explicitly installs Test262 host facilities.

## Outcomes

Only successful executions and verified expected exceptions pass. The report
keeps these nonpassing categories distinct:

- Unexpected JavaScript results or exceptions.
- Unsupported features and unavailable host facilities.
- Opted-in or native capacity limits.
- Unverified negative diagnostics.
- Metadata and harness/setup failures.

A parse-negative test needs a reviewed Syntax diagnostic with the exact source
span and message. Unsupported syntax cannot stand in for SyntaxError. Runtime
and resolution negatives retain their declared phase and error constructor.
Missing or repeated async completion fails; host failures cannot satisfy a
JavaScript negative test.

## Manifests and provenance

`manifest.tsv` records a fixture kind, SHA-256 and upstream-relative path.
`runner.tsv` records each selected original and either its exact reviewed parse
diagnostic or dashes for an executed test. `audit/upstream-tree.tsv` and
`audit/upstream-commit.txt` reproduce the pinned Git identities independently of
the local SHA-256 manifest. Original bytes, support modules and harness files
are checked before changes are accepted.

A new admission must include its required unchanged support/harness files and
remove its old scope entry. Do not shorten an original, change its metadata,
increase default quotas or convert a host failure into a pass. Supplementary
local tests can isolate behavior, but receive no credit as whole originals.

## Scope

The [scope manifest](audit/scope.tsv) and [source reviews](audit/README.md) cover
optional browser behavior, later proposals, version-dependent data, host choices
and resource constraints. Feature metadata alone is not a scope decision: an
original tagged with a proposal can still test ordinary baseline syntax.

Policy probes and Unicode witnesses test the selected published algorithms
independently of incompatible originals. Resource-stress records retain their
nonpassing outcomes and external diagnostic conditions. A scope partition is
not a conformance percentage.

See [compatibility](../../docs/compatibility.md) for host choices, implementation
constraints, and specification interpretations.
