# Test262 provenance and scope

The corpus selects tests for a particular published language edition and host.
These records make that selection inspectable and verify that retained files
still match upstream. Test outcomes do not determine scope.

## Records

| Record | Purpose |
| --- | --- |
| [specification.json](specification.json) | Exact published ECMA-262 source identity |
| [upstream-commit.txt](upstream-commit.txt) | Git commit object named by [REVISION](../REVISION) |
| [upstream-tree.tsv](upstream-tree.tsv) | Expanded test and harness trees, with the remaining root entries |
| [scope.tsv](scope.tsv) | Reason for excluding each unretained original |
| [staging-reviewed.tsv](staging-reviewed.tsv) | Source-specific rationale and Git blob for each decided staging exclusion |
| [unicode-witnesses.tsv](unicode-witnesses.tsv) | Code points whose property membership contradicts an older fixture |
| [published-policy-probes.js](published-policy-probes.js) | Bounded checks of published algorithms and host choices |
| [resource-stress.json](resource-stress.json) | Nonpassing native-limit outcomes and diagnostic conditions |

## What verification establishes

`cargo xtask verify` reconstructs the pinned Git trees and commit, checks retained
files against upstream blobs and the local SHA-256 manifest, and checks that the
runner and scope inventories account for every upstream original exactly once.
Files marked `_FIXTURE` remain support files rather than standalone tests.

A matching local SHA-256 alone is insufficient: editing a fixture and updating
its manifest must still fail the upstream Git identity check. Verification uses
the checked-in records, without an upstream checkout or network access.

Scope decisions have additional evidence requirements:

- Decided staging exclusions need a matching source-specific explanation and
  pinned Git blob. A feature tag alone is insufficient.
- Unicode property exclusions need a code-point witness with contradictory
  old/current membership.
- Native resource exclusions need a pinned, nonpassing limit result for each
  recorded execution mode. They cannot claim passing credit.

The [core review](core-review.md), [later-proposal review](post-baseline.md), and
[staging review](staging.md) explain the corresponding reasons.

## Commands

From the repository root:

```sh
cargo xtask verify
cargo test -p xtask --locked
```

The first checks repository records; the second tests the verifier itself.

To download and verify the exact published specification:

```sh
cargo xtask verify-spec
```

To verify an existing copy, pass its path as the final argument. An annotated or
edited copy does not substitute for the pinned source.
