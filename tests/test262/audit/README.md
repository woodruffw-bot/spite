# Complete pinned upstream inventory

This audit covers all 53,597 non-`_FIXTURE` JavaScript originals in the pinned
Test262 `test/` tree. It is independent of which originals currently pass.
There are 41,567 retained originals and 12,030 explicit scope entries.
This partition is not a conformance percentage.

`upstream-commit.txt` is the exact Git commit object named by `../REVISION`.
`upstream-tree.tsv` records each leaf's mode, Git object SHA-1 and upstream path
under `test/` and `harness/`, plus the other root entries. The checker rebuilds
the Git trees, checks the root against the commit object, checks the commit
against the revision, and checks every retained file against its upstream blob.
Removing an unretained test from this inventory changes the root hash and fails
CI. No network connection or upstream checkout is needed for verification.

`scope.tsv` assigns every unretained original exactly one disposition. A fixture
admission must remove its scope entry; the runner and scope inventories must
partition the upstream originals exactly. `_FIXTURE` JavaScript files and other
support files remain in the verified tree but are not test originals.

| Disposition | Originals | Meaning |
| --- | ---: | --- |
| `outside-edition17:<feature>` | 5,782 | Tests requiring facilities or changes absent from the published baseline; see the [feature review](post-baseline.md). |
| `baseline-review` | 202 | Remaining core built-ins/language originals awaiting independent review of optional behavior, changed assertions, Unicode data and allocation assumptions. |
| `staging-review` | 1,479 | Supplementary staging tests awaiting review of standard semantics, engine-specific helpers and host requirements. Four originals that exposed fixed shared-view extensibility and eval-binding bugs have been admitted unchanged. |
| `harness-self-test` | 116 | Infrastructure validation, separately executed: all 232 original modes pass after the absent-$DONE correction. |
| `optional-annex-b` | 1,086 | Annex B browser-compatibility tests; this non-browser host disables Annex B. |
| `outside-ecma262` | 3,365 | ECMA-402 internationalization tests, outside the ECMA-262 target. |

The two review categories are unresolved work, not accepted exclusions. A scope
entry proves accounting only; its semantic justification must be reviewed
against the published specification and documented host policies. Passing the
retained suite and the provenance checks does not close that qualification.

Run `python3 tools/check-test262.py` for fixture, review and complete-tree checks,
or `python3 tools/check-test262-scope.py` for the provenance/scope report alone.
Run `python3 tools/test-test262-scope.py` for the omission and tampering controls.
All tools use the Python standard library.
