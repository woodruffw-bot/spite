"""Verify exact bytes and inventory of the pinned upstream regression fixtures."""

import hashlib
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent.parent / "tests/test262"
revision = (root / "REVISION").read_text().strip()
if len(revision) != 40 or any(c not in "0123456789abcdef" for c in revision):
    raise SystemExit("Test262 must be pinned to a full commit SHA")
paths = set()
modes = {}
for line in (root / "manifest.tsv").read_text().splitlines():
    if not line or line.startswith("#"):
        continue
    expectation, checksum, path = line.split("\t")
    if expectation not in {
        "raw-pass", "raw-syntax-error", "parse-syntax-error", "identifier-tokens", "identifier-error", "parser-pass", "script-pass", "script-runtime-error", "harness"
    }:
        raise SystemExit(f"unsupported fixture mode: {expectation}")
    relative = PurePosixPath(path)
    if relative.is_absolute() or ".." in relative.parts or relative.suffix != ".js":
        raise SystemExit(f"invalid fixture path: {path}")
    if (expectation == "harness") != (relative.parts[0] == "harness"):
        raise SystemExit(f"harness mode/path mismatch: {path}")
    if expectation != "harness" and relative.parts[0] != "test":
        raise SystemExit(f"test path must start with test/: {path}")
    if path in paths:
        raise SystemExit(f"duplicate fixture: {path}")
    paths.add(path)
    modes[path] = expectation
    actual = hashlib.sha256((root / "upstream" / path).read_bytes()).hexdigest()
    if actual != checksum:
        raise SystemExit(f"fixture bytes changed: {path}")
actual_paths = {p.relative_to(root / "upstream").as_posix() for p in (root / "upstream").rglob("*.js")}
if paths != actual_paths or not paths:
    raise SystemExit("fixture inventory does not match the manifest")
reviewed = set()
for line in (root / "runner.tsv").read_text().splitlines():
    if not line or line.startswith("#"):
        continue
    fields = line.split("\t")
    if len(fields) != 4:
        raise SystemExit("invalid runner review row")
    path, start, end, message = fields
    if path not in paths or path in reviewed:
        raise SystemExit(f"unknown or duplicate runner path: {path}")
    reviewed.add(path)
    if modes[path] in {"raw-pass", "script-pass", "script-runtime-error"}:
        if fields[1:] != ["-", "-", "-"]:
            raise SystemExit(f"executed runner entry has a parse review: {path}")
    elif modes[path] in {"raw-syntax-error", "parse-syntax-error"}:
        data = (root / "upstream" / path).read_bytes()
        try:
            start, end = int(start), int(end)
            valid = 0 <= start < end <= len(data) and message not in {"", "-"}
            data[:start].decode("utf-8")
            data[:end].decode("utf-8")
        except (ValueError, UnicodeDecodeError):
            valid = False
        if not valid:
            raise SystemExit(f"invalid reviewed diagnostic: {path}")
    else:
        raise SystemExit(f"non-executable fixture cannot enter the execution corpus: {path}")
expected = {path for path, mode in modes.items() if mode in {"raw-pass", "script-pass", "script-runtime-error", "raw-syntax-error", "parse-syntax-error"}}
if reviewed != expected:
    raise SystemExit("runner inventory does not match the selected fixture modes")
harness_count = sum(mode == "harness" for mode in modes.values())
print(f"Verified {len(paths) - harness_count} Test262 fixtures and {harness_count} harness files from {revision}.")
print(f"Verified {len(reviewed)} reviewed runner entries.")
