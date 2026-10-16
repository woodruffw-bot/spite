#!/usr/bin/env python3
"""Regressions for complete, unconditional runtime test discovery."""

import importlib.util
from pathlib import Path
import tempfile
import unittest


specification = importlib.util.spec_from_file_location(
    "runtime_test_inventory", Path(__file__).with_name("check-runtime-tests.py")
)
checker = importlib.util.module_from_spec(specification)
specification.loader.exec_module(checker)


class RuntimeTestInventoryTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="spite-runtime-inventory-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.tests = self.root / "crates/spite-runtime/tests"
        (self.tests / "common").mkdir(parents=True)
        (self.tests / "common/mod.rs").write_text("pub const COUNT: usize = 1;\n")
        self.case("first.rs")
        self.manifest(('cases/first.rs', 'first'))

    def case(self, path):
        source = self.tests / "cases" / path
        source.parent.mkdir(parents=True, exist_ok=True)
        source.write_text("#[test]\nfn example() {}\n", encoding="utf-8")

    def manifest(self, *declarations):
        source = "//! One integration target.\n\nmod common;\n"
        source += "".join(
            f'\n#[path = "{path}"]\nmod {module};\n'
            for path, module in declarations
        )
        (self.tests / "integration.rs").write_text(source, encoding="utf-8")

    def test_all_cases_including_nested_cases_are_included_without_fixture_helpers(self):
        self.case("nested/second.rs")
        self.case("fixtures/reference.rs")
        (self.tests / "cases/fixtures/reference.rs").write_text("const REFERENCE: usize = 1;\n")
        self.manifest(('cases/first.rs', 'first'), ('cases/nested/second.rs', 'second'))
        self.assertEqual(checker.check(self.root), 2)

    def test_a_new_undeclared_case_fails(self):
        self.case("new_case.rs")
        with self.assertRaisesRegex(ValueError, "missing from integration.rs.*new_case"):
            checker.check(self.root)

    def test_a_declared_missing_case_fails(self):
        self.manifest(('cases/first.rs', 'first'), ('cases/missing.rs', 'missing'))
        with self.assertRaisesRegex(ValueError, "cases are missing.*missing.rs"):
            checker.check(self.root)

    def test_a_case_cannot_be_declared_twice(self):
        self.manifest(('cases/first.rs', 'first'), ('cases/first.rs', 'first'))
        with self.assertRaisesRegex(ValueError, "duplicate runtime case path"):
            checker.check(self.root)

    def test_different_case_paths_cannot_share_a_module_name(self):
        self.case("nested/first.rs")
        self.manifest(('cases/first.rs', 'first'), ('cases/nested/first.rs', 'first'))
        with self.assertRaisesRegex(ValueError, "duplicate runtime module"):
            checker.check(self.root)

    def test_a_module_alias_fails(self):
        self.manifest(('cases/first.rs', 'renamed'))
        with self.assertRaisesRegex(ValueError, "must match its case-file stem"):
            checker.check(self.root)

    def test_a_fixture_helper_cannot_be_declared_as_a_case(self):
        self.case("fixtures/reference.rs")
        (self.tests / "cases/fixtures/reference.rs").write_text("const REFERENCE: usize = 1;\n")
        self.manifest(('cases/first.rs', 'first'), ('cases/fixtures/reference.rs', 'reference'))
        with self.assertRaisesRegex(ValueError, "cases are missing or are helpers"):
            checker.check(self.root)

    def test_unexpected_root_and_directory_targets_fail(self):
        for relative in ["extra.rs", "extra/main.rs"]:
            with self.subTest(relative=relative):
                target = self.tests / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text("#[test]\nfn extra() {}\n")
                try:
                    with self.assertRaisesRegex(ValueError, "root target|directory test target"):
                        checker.check(self.root)
                finally:
                    target.unlink()

    def test_conditionals_other_attributes_and_extra_code_fail(self):
        for unexpected in ["#[cfg(any())]", "#[ignore]", "fn hidden() {}", "mod extra;"]:
            with self.subTest(unexpected=unexpected):
                self.manifest(('cases/first.rs', 'first'))
                path = self.tests / "integration.rs"
                path.write_text(path.read_text().replace('#[path', unexpected + '\n#[path'))
                with self.assertRaisesRegex(ValueError, "only unconditional"):
                    checker.check(self.root)

    def test_noncanonical_and_outside_paths_fail(self):
        for path in [".", "cases/../first.rs", "cases/./first.rs", "cases//first.rs", "other/first.rs"]:
            with self.subTest(path=path):
                self.manifest((path, 'first'))
                with self.assertRaisesRegex(ValueError, "invalid runtime case path"):
                    checker.check(self.root)

    def test_an_empty_case_inventory_cannot_pass(self):
        (self.tests / "cases/first.rs").unlink()
        self.manifest()
        with self.assertRaisesRegex(ValueError, "inventory is empty"):
            checker.check(self.root)


if __name__ == "__main__":
    unittest.main()
