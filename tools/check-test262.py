"""Verify exact bytes and inventory of the pinned upstream regression fixtures."""

import hashlib
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent.parent / "tests/test262"
revision = (root / "REVISION").read_text().strip()
if len(revision) != 40 or any(c not in "0123456789abcdef" for c in revision):
    raise SystemExit("Test262 must be pinned to a full commit SHA")
paths = set()
for line in (root / "manifest.tsv").read_text().splitlines():
    if not line or line.startswith("#"):
        continue
    expectation, checksum, path = line.split("\t")
    if expectation not in {"raw-pass", "raw-syntax-error", "identifier-tokens", "identifier-error"}:
        raise SystemExit(f"unsupported fixture mode: {expectation}")
    relative = PurePosixPath(path)
    if relative.is_absolute() or ".." in relative.parts or relative.suffix != ".js":
        raise SystemExit(f"invalid fixture path: {path}")
    if path in paths:
        raise SystemExit(f"duplicate fixture: {path}")
    paths.add(path)
    actual = hashlib.sha256((root / "upstream" / path).read_bytes()).hexdigest()
    if actual != checksum:
        raise SystemExit(f"fixture bytes changed: {path}")
actual_paths = {p.relative_to(root / "upstream").as_posix() for p in (root / "upstream").rglob("*.js")}
if paths != actual_paths or not paths:
    raise SystemExit("fixture inventory does not match the manifest")
print(f"Verified {len(paths)} Test262 fixtures from {revision}.")
