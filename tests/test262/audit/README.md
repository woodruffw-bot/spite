# Test262 provenance and scope

These records distinguish the complete pinned upstream inventory from the
originals selected for execution. They are independent of test results.

| Record | Purpose |
| --- | --- |
| `specification.json` | Exact published ECMA-262 source identity |
| `upstream-commit.txt` | Git commit object named by `../REVISION` |
| `upstream-tree.tsv` | Expanded test/harness leaves and other root entries |
| `scope.tsv` | Disposition of every unretained test original |
| `staging-reviewed.tsv` | Source-specific rationale and pinned Git blob for closed staging decisions |
| `unicode-witnesses.tsv` | Contradictory old/current property memberships |
| `published-policy-probes.js` | Bounded checks of published algorithms and host choices |
| `resource-stress.json` | Nonpassing native-limit outcomes and their diagnostic conditions |

The checker reconstructs the Git trees and commit, verifies retained bytes
against upstream blobs and the local manifest, and requires the runner/scope
inventories to partition upstream originals exactly. `_FIXTURE` support files
remain in the tree but are not test originals. Verification uses local records;
it does not need a network connection or an upstream checkout.

The [core review](core-review.md), [post-baseline review](post-baseline.md) and
[staging review](staging.md) explain the disposition categories. Feature tags
alone cannot close a staging entry. Closed entries require a matching rationale
and pinned source identity. Unicode property exclusions require contradictory
witnesses. Resource exclusions require complete nonpassing Limit records and
cannot claim passing credit.

For fixture, review and complete-tree checks:

```sh
python3 tools/check-test262.py
python3 tools/test-test262-scope.py
```

Verify a local published specification copy with
`python3 tools/check-specification.py /path/to/spec.html`, or fetch its exact pin
with `python3 tools/check-specification.py --download`. Annotated copies do not
substitute for the pinned source.
