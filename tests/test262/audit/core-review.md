# Core scope decisions

The published algorithms and documented non-browser host policies determine
scope. Every unretained original remains in the pinned tree and
[scope manifest](scope.tsv), without passing credit.

## Unicode data

ECMA-262 requires the latest Unicode standard. Some upstream originals contain
older exhaustive property datasets or obsolete assumptions about join-control
IdentityEscapes. [unicode-witnesses.tsv](unicode-witnesses.tsv) records contradictory
memberships against the current pinned data. Runner controls exercise those
witnesses in Unicode matching modes and reject the obsolete literal escapes.

The Unicode input identities are recorded in
[UNICODE-SOURCES.json](../../../vendor/regress/UNICODE-SOURCES.json). Witnesses
come from the original UCD ranges rather than from the generated matcher tables.

## Optional facilities

The normative optional Object.prototype `__proto__` accessor and legacy
lookup/definition helpers are disabled. This also affects originals that use
those helpers while testing another feature. Required object-literal prototype
initializers remain supported.

Optional function caller/arguments reflection is disabled. Required
Function.prototype poison accessors retain their shared throwing identities.
Annex B RegExp escapes remain disabled, including originals that use those forms
inside an otherwise ordinary String operation.

## Published algorithms and allocation

Published Iterator.take/drop reject NaN and negative limits, without a finite
maximum-safe-integer rejection. Originals requiring that later restriction are
identified explicitly. Policy probes protect conversion/lookup order and closing
behavior for the published algorithms.

Sparse ordinary/shared Data Blocks can represent large logical zero regions.
A dense-allocation failure assumption is not valid for those lengths. For a
TypedArray array-like length, ToLength clamps before allocation and element
reading. Getter sentinels distinguish that ordering from a premature RangeError.
These choices do not change primitive ToIndex rejection or explicit host quotas.
