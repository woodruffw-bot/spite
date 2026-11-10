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
    "unicode-version-difference": ("test/built-ins/RegExp/", "test/language/literals/regexp/"),
    "optional-legacy-helpers": ("test/built-ins/Object/prototype/", "test/language/expressions/class/", "test/language/statements/class/"),
    "optional-annex-b-dependent": ("test/built-ins/", "test/language/", "test/staging/"),
    "optional-function-introspection": ("test/built-ins/Function/", "test/language/arguments-object/"),
    "post-publication-assertion": ("test/built-ins/Iterator/prototype/take/", "test/built-ins/Iterator/prototype/drop/"),
    "allocation-assumption": ("test/built-ins/ArrayBuffer/", "test/built-ins/SharedArrayBuffer/", "test/built-ins/TypedArrayConstructors/"),
}
POST_BASELINE_FEATURES = {
    "Temporal", "ShadowRealm", "explicit-resource-management",
    "source-phase-imports", "import-defer", "await-dictionary",
    "joint-iteration", "iterator-chunking", "iterator-includes",
    "Iterator.prototype.join", "immutable-arraybuffer", "error-stack-accessor",
    "decorators", "import-text", "nonextensible-applies-to-private",
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
        valid = disposition in DISPOSITIONS and path.startswith(DISPOSITIONS[disposition])
        if disposition.startswith("outside-edition17:"):
            feature = disposition.removeprefix("outside-edition17:")
            valid = feature in POST_BASELINE_FEATURES and path.startswith(("test/built-ins/", "test/language/", "test/staging/"))
        if not valid:
            raise ValueError(f"invalid scope disposition: {disposition}: {path}")
        if path in scoped:
            raise ValueError(f"duplicate scope disposition: {path}")
        scoped[path] = disposition
    missing = originals - retained
    if set(scoped) != missing:
        omitted = sorted(missing - set(scoped))[:3]
        extra = sorted(set(scoped) - missing)[:3]
        raise ValueError(f"scope must account for every unretained original: missing={omitted}, extra={extra}")
    # Staging metadata often names a proposal even when the source only tests
    # ordinary syntax. Require an explicit source review before closing an entry.
    reviewed_staging = {
        path: reason for path, reason in scoped.items()
        if path.startswith("test/staging/") and reason != "staging-review"
    }
    staging_reviews = {}
    review_file = audit / "staging-reviewed.tsv"
    if review_file.exists():
        for reason, path, oid, explanation in rows(review_file, 4):
            if (path in staging_reviews or reviewed_staging.get(path) != reason
                    or entries.get(path) != ("100644", oid) or not explanation.strip()):
                raise ValueError(f"invalid or unpinned staging review: {path}")
            staging_reviews[path] = reason
    if staging_reviews != reviewed_staging:
        raise ValueError("closed staging scope entries require matching pinned source reviews")
    unicode_paths = {path for path, reason in scoped.items() if reason == "unicode-version-difference"}
    # The two literal loops instead hard-code acceptance of join-control
    # IdentityEscapes, contrary to Unicode18's ID_Continue membership.
    literal_loops = {
        "test/language/literals/regexp/S7.8.5_A1.4_T2.js",
        "test/language/literals/regexp/S7.8.5_A2.4_T2.js",
    }
    witness_file = audit / "unicode-witnesses.tsv"
    witnesses = set()
    if witness_file.exists():
        for path, prop, point, old, current in rows(witness_file, 5):
            try:
                valid_point = 0 <= int(point, 16) <= 0x10ffff
            except ValueError:
                valid_point = False
            if (path in witnesses or not prop or not valid_point
                    or {old, current} != {"true", "false"}):
                raise ValueError(f"invalid or non-contradictory Unicode witness: {path}")
            witnesses.add(path)
    if witnesses != unicode_paths - literal_loops:
        raise ValueError("Unicode witnesses must cover every version-dependent property original exactly")
    return len(originals), len(retained), Counter(scoped.values())
