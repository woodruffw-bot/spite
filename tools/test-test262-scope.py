#!/usr/bin/env python3
"""Ensure missing tests and rewritten upstream bytes cannot evade scope accounting."""

from pathlib import Path
import tempfile
import unittest

from test262_scope import check, object_id, tree_id


class ScopeTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="spite-test262-scope-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        (self.root / "audit").mkdir()
        self.files = {
            "test/built-ins/retained.js": b"retained;\n",
            "test/built-ins/RegExp/property-escapes/pending.js": b"pending;\n",
            "test/built-ins/helper_FIXTURE.js": b"helper;\n",
            "test/annexB/optional.js": b"optional;\n",
            "harness/assert.js": b"assert;\n",
        }
        tree = {}
        rows = []
        for path, data in self.files.items():
            node = tree
            parts = path.split("/")
            for name in parts[:-1]:
                node = node.setdefault(name, {})
            oid = object_id("blob", data)
            node[parts[-1]] = ("100644", oid)
            rows.append(f"100644\t{oid}\t{path}\n")
            local = self.root / "upstream" / path
            local.parent.mkdir(parents=True, exist_ok=True)
            local.write_bytes(data)
        self.write("audit/upstream-tree.tsv", "".join(rows))
        commit = f"tree {tree_id(tree)}\nauthor Test <test@example.com> 0 +0000\ncommitter Test <test@example.com> 0 +0000\n\nTest\n".encode()
        (self.root / "audit/upstream-commit.txt").write_bytes(commit)
        self.write("REVISION", object_id("commit", commit) + "\n")
        self.write("manifest.tsv", "".join(
            f"script-pass\tunused-local-checksum\t{path}\n"
            for path in self.files if not path.endswith(("pending.js", "optional.js"))
        ))
        self.write("runner.tsv", "test/built-ins/retained.js\t-\t-\t-\n")
        self.write("audit/scope.tsv", "baseline-review\ttest/built-ins/RegExp/property-escapes/pending.js\noptional-annex-b\ttest/annexB/optional.js\n")

    def write(self, path, text):
        (self.root / path).write_text(text, encoding="utf-8")

    def test_complete_inventory_distinguishes_originals_from_support(self):
        originals, retained, dispositions = check(self.root)
        self.assertEqual((originals, retained), (3, 1))
        self.assertEqual(dispositions, {"baseline-review": 1, "optional-annex-b": 1})

    def test_commit_object_must_match_the_pin(self):
        self.write("REVISION", "0" * 40)
        with self.assertRaisesRegex(ValueError, "commit object"):
            check(self.root)

    def test_an_omitted_upstream_entry_cannot_change_the_denominator(self):
        path = self.root / "audit/upstream-tree.tsv"
        path.write_text("".join(line for line in path.read_text().splitlines(True) if "pending.js" not in line))
        with self.assertRaisesRegex(ValueError, "tree does not match"):
            check(self.root)

    def test_duplicate_tree_entries_fail(self):
        path = self.root / "audit/upstream-tree.tsv"
        path.write_text(path.read_text() + path.read_text().splitlines(True)[0])
        with self.assertRaisesRegex(ValueError, "duplicate upstream"):
            check(self.root)

    def test_retained_bytes_are_checked_independently_of_the_local_manifest_hash(self):
        self.write("upstream/test/built-ins/retained.js", "rewritten;\n")
        with self.assertRaisesRegex(ValueError, "bytes differ from upstream"):
            check(self.root)

    def test_every_unretained_original_needs_an_explicit_entry(self):
        self.write("audit/scope.tsv", "optional-annex-b\ttest/annexB/optional.js\n")
        with self.assertRaisesRegex(ValueError, "account for every unretained"):
            check(self.root)

    def test_retained_and_scoped_must_be_disjoint(self):
        self.write("manifest.tsv", (self.root / "manifest.tsv").read_text() + "script-pass\tunused\ttest/built-ins/RegExp/property-escapes/pending.js\n")
        self.write("runner.tsv", (self.root / "runner.tsv").read_text() + "test/built-ins/RegExp/property-escapes/pending.js\t-\t-\t-\n")
        with self.assertRaisesRegex(ValueError, "account for every unretained"):
            check(self.root)

    def test_scope_duplicates_and_inappropriate_categories_fail(self):
        original = (self.root / "audit/scope.tsv").read_text()
        for text, message in [
            (original + original.splitlines(True)[0], "duplicate scope"),
            (original.replace("baseline-review", "optional-annex-b"), "invalid scope"),
            (original.replace("baseline-review", "unknown"), "invalid scope"),
        ]:
            with self.subTest(text=text):
                self.write("audit/scope.tsv", text)
                with self.assertRaisesRegex(ValueError, message):
                    check(self.root)

    def test_only_reviewed_post_baseline_feature_names_are_accepted(self):
        path = self.root / "audit/scope.tsv"
        original = path.read_text()
        self.write("audit/scope.tsv", original.replace("baseline-review", "outside-edition17:Temporal"))
        self.assertEqual(check(self.root)[2]["outside-edition17:Temporal"], 1)
        self.write("audit/scope.tsv", original.replace("baseline-review", "outside-edition17:unknown"))
        with self.assertRaisesRegex(ValueError, "invalid scope"):
            check(self.root)

    def test_unicode_dispositions_need_complete_contradictory_witnesses(self):
        scope = (self.root / "audit/scope.tsv").read_text()
        self.write("audit/scope.tsv", scope.replace("baseline-review", "unicode-version-difference"))
        with self.assertRaisesRegex(ValueError, "Unicode witnesses must cover"):
            check(self.root)
        witness = "test/built-ins/RegExp/property-escapes/pending.js\tAlphabetic\t0558\tfalse\ttrue\n"
        self.write("audit/unicode-witnesses.tsv", witness)
        self.assertEqual(check(self.root)[2]["unicode-version-difference"], 1)
        self.write("audit/unicode-witnesses.tsv", witness.replace("false", "true"))
        with self.assertRaisesRegex(ValueError, "non-contradictory Unicode witness"):
            check(self.root)


if __name__ == "__main__":
    unittest.main()
