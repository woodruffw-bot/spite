# Core scope decisions

These decisions compare upstream originals with the published edition and
spite's [host choices](../../../docs/compatibility.md). Excluded originals remain
listed in [scope.tsv](scope.tsv).

## Unicode differences

ECMA-262 requires the latest Unicode standard. Some upstream originals embed
older exhaustive property sets. A single code point whose membership changed is
enough to show that the original's expected set conflicts with the current data;
[unicode-witnesses.tsv](unicode-witnesses.tsv) records such examples.

The runner checks the witnesses in Unicode matching modes. Their expected
memberships come from original UCD ranges, independently of the generated matcher
tables. Source identities are in
[UNICODE-SOURCES.json](../../../vendor/regress/UNICODE-SOURCES.json).

Separate literal tests require obsolete join-control identity escapes. The
runner checks that the current grammar rejects those escapes; they are not
property-table differences.

## Optional facilities

The host disables the normative optional `Object.prototype.__proto__` accessor
and legacy accessor definition/lookup helpers. This also excludes originals that
use those helpers while testing another feature. Required object-literal
prototype initializers remain supported.

Optional function `caller`/`arguments` reflection is disabled. Required
`Function.prototype` poison accessors retain their shared throwing identities.

Annex B RegExp escapes are disabled, including when an original uses one inside
an otherwise ordinary String operation.

## Iterator limits

The published `Iterator.prototype.take` and `drop` algorithms reject `NaN` and
negative limits. They do not reject a finite limit solely because it exceeds
the maximum safe integer. Originals requiring that later restriction are
identified in the scope manifest.

[Policy probes](published-policy-probes.js) exercise the published conversion
order, method lookup, and iterator closing behavior.

## Sparse allocation

Ordinary and shared Data Blocks can represent large logical zero regions
without a dense physical allocation. An original that assumes allocation must
fail at such a length therefore does not describe this storage model.

For TypedArray construction from an array-like value, `ToLength` clamps the
length before allocation and element reading. Getter sentinels check that element
access occurs in the prescribed order rather than being preempted by an assumed
`RangeError`. Sparse storage does not change primitive `ToIndex` rejection or
explicit host quotas.
