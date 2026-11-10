"""Verify Test262 provenance and complete scope accounting without a Git checkout."""

from collections import Counter
import hashlib
from pathlib import PurePosixPath
import re


SHA1 = re.compile(r"[0-9a-f]{40}\Z")
DISPOSITIONS = {
    "optional-annex-b": "test/annexB/",
    "outside-ecma262": "test/intl402/",
    "harness-self-test": "test/harness/",
    "staging-review": "test/staging/",
    "baseline-review": ("test/built-ins/", "test/language/"),
}


def object_id(kind, data):
    return hashlib.sha1(f"{kind} {len(data)}\0".encode() + data).hexdigest()


def rows(path, fields):
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if not line or line.startswith("#"):
            continue
        values = line.split("\t")
        if len(values) != fields:
            raise ValueError(f"{path.name}:{number}: expected {fields} fields")
        yield values


def canonical_path(path):
    relative = PurePosixPath(path)
    if (relative.is_absolute() or not relative.parts or ".." in relative.parts
            or relative.as_posix() != path or "\\" in path or "\0" in path):
        raise ValueError(f"invalid upstream path: {path}")
    return relative.parts


def read_tree(path):
    entries = {}
    tree = {}
    for mode, oid, name in rows(path, 3):
        if mode not in {"040000", "100644", "100755", "120000"} or not SHA1.fullmatch(oid):
            raise ValueError(f"invalid upstream object: {name}")
        if name in entries:
            raise ValueError(f"duplicate upstream object: {name}")
        parts = canonical_path(name)
        if mode == "040000" and (len(parts) != 1 or name in {"test", "harness"}):
            raise ValueError(f"test/harness trees must be fully expanded: {name}")
        node = tree
        for part in parts[:-1]:
            node = node.setdefault(part, {})
            if not isinstance(node, dict):
                raise ValueError(f"upstream file/directory collision: {name}")
        if parts[-1] in node:
            raise ValueError(f"upstream file/directory collision: {name}")
        node[parts[-1]] = (mode, oid)
        entries[name] = (mode, oid)
    if not all(isinstance(tree.get(name), dict) for name in ("test", "harness")):
        raise ValueError("upstream test and harness trees must be fully expanded")
    return entries, tree


def tree_id(tree):
    entries = []
    for name, value in tree.items():
        mode, oid = ("040000", tree_id(value)) if isinstance(value, dict) else value
        # Git sorts directory names as though followed by '/', files by NUL.
        key = name.encode("utf-8") + (b"/" if mode == "040000" else b"\0")
        data = mode.lstrip("0").encode() + b" " + name.encode("utf-8") + b"\0" + bytes.fromhex(oid)
        entries.append((key, data))
    return object_id("tree", b"".join(data for _, data in sorted(entries)))


def original(path):
    return path.startswith("test/") and path.endswith(".js") and "_FIXTURE" not in path


def check(root):
    audit = root / "audit"
    revision = (root / "REVISION").read_text().strip()
    commit = (audit / "upstream-commit.txt").read_bytes()
    if not SHA1.fullmatch(revision) or object_id("commit", commit) != revision:
        raise ValueError("upstream commit object does not match REVISION")
    entries, tree = read_tree(audit / "upstream-tree.tsv")
    if commit.split(b"\n", 1)[0] != b"tree " + tree_id(tree).encode():
        raise ValueError("upstream tree does not match the pinned commit")

    # Verify the retained bytes against upstream Git identities, independently
    # of the local SHA-256 manifest. Include support files and harness helpers.
    manifest = set()
    for _, _, path in rows(root / "manifest.tsv", 3):
        if path in manifest:
            raise ValueError(f"duplicate retained file: {path}")
        manifest.add(path)
        if path not in entries or entries[path][0] == "040000":
            raise ValueError(f"retained file absent from upstream: {path}")
        if object_id("blob", (root / "upstream" / path).read_bytes()) != entries[path][1]:
            raise ValueError(f"retained bytes differ from upstream: {path}")
    retained = set()
    for path, _, _, _ in rows(root / "runner.tsv", 4):
        if path in retained or path not in manifest:
            raise ValueError(f"duplicate or unmanifested runner path: {path}")
        retained.add(path)
    originals = {path for path in entries if original(path)}
    if not retained <= originals:
        raise ValueError("runner contains a non-original upstream path")
    scoped = {}
    for disposition, path in rows(audit / "scope.tsv", 2):
        if disposition not in DISPOSITIONS or not path.startswith(DISPOSITIONS[disposition]):
            raise ValueError(f"invalid scope disposition: {disposition}: {path}")
        if path in scoped:
            raise ValueError(f"duplicate scope disposition: {path}")
        scoped[path] = disposition
    missing = originals - retained
    if set(scoped) != missing:
        omitted = sorted(missing - set(scoped))[:3]
        extra = sorted(set(scoped) - missing)[:3]
        raise ValueError(f"scope must account for every unretained original: missing={omitted}, extra={extra}")
    return len(originals), len(retained), Counter(scoped.values())
