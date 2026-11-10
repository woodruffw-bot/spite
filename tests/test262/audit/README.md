# Complete pinned upstream inventory

This audit covers all 53,597 non-`_FIXTURE` JavaScript originals in the pinned
Test262 `test/` tree. It is independent of which originals currently pass.
There are 41,573 retained originals and 12,024 explicit scope entries.
This partition is not a conformance percentage.

[specification.json](specification.json) identifies the exact published ECMA-262
source by commit, Git blob, size and SHA-256. Run
`python3 tools/check-specification.py /path/to/spec.html` to verify local bytes,
or `python3 tools/check-specification.py --download` to fetch and verify the pin.
CI runs the latter. Annotated copies are rejected even when their normative
clauses happen to be identical.

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
| `unicode-version-difference` | 92 | Independently verified Unicode18 changes; ninety property originals have retained contradictory witnesses. |
| `optional-legacy-helpers` | 58 | Disabled normative optional Object.prototype helpers, including their use inside four private-accessor originals. |
| `optional-function-introspection` | 23 | Caller-chain extensions are disabled; required Function.prototype poison accessors remain enabled. |
| `optional-annex-b-dependent` | 17 | Core-directory originals requiring disabled Annex B accessors or RegExp escapes. |
| `post-publication-assertion` | 6 | Later Iterator.take/drop finite-limit assertions contradict the published algorithms. |
| `allocation-assumption` | 5 | Allocation-failure assumptions in originals whose logical sizes this sparse host can represent. |
| `staging-review` | 1,474 | Supplementary staging tests awaiting semantic/host review. Nine originals covering shared views, eval bindings, Unicode backreferences, empty blocks and field initializer contexts have been admitted unchanged. |
| `harness-self-test` | 116 | Infrastructure validation, separately executed: all 232 original modes pass after the absent-$DONE correction. |
| `optional-annex-b` | 1,086 | Annex B browser-compatibility tests; this non-browser host disables Annex B. |
| `outside-ecma262` | 3,365 | ECMA-402 internationalization tests, outside the ECMA-262 target. |

The [core review](core-review.md) records these decisions and corrects one earlier
object-literal exclusion; that unchanged original is now retained. Supplementary
policy probes and all ninety Unicode witnesses execute in the runner controls.
The staging-review category remains unresolved work, not an accepted exclusion. A scope
entry proves accounting only; its semantic justification must be reviewed
against the published specification and documented host policies. Passing the
retained suite and the provenance checks does not close that qualification.

Run `python3 tools/check-test262.py` for fixture, review and complete-tree checks,
or `python3 tools/check-test262-scope.py` for the provenance/scope report alone.
Run `python3 tools/test-test262-scope.py` for the omission and tampering controls.
All tools use the Python standard library.
