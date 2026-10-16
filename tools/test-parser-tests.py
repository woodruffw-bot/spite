#!/usr/bin/env python3
"""Regressions for complete, unconditional Parser test discovery."""

import importlib.util
from pathlib import Path
import tempfile
import unittest


specification = importlib.util.spec_from_file_location(
    "parser_test_inventory", Path(__file__).with_name("check-parser-tests.py")
)
checker = importlib.util.module_from_spec(specification)
specification.loader.exec_module(checker)


class ParserTestInventoryTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="spite-parser-inventory-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.crate = self.root / "crates/spite-parser"
        self.tests = self.crate / "tests"
        self.tests.mkdir(parents=True)
        self.cargo()
        self.case("first.rs")
        self.manifest(('first.rs', 'first'))

    def cargo(self, package='autotests = false', targets=None):
        if targets is None:
            targets = '[[test]]\nname = "parser_integration"\npath = "tests/integration.rs"\n'
        (self.crate / "Cargo.toml").write_text(
            '[package]\nname = "spite-parser"\n' + package + '\n\n' + targets,
            encoding="utf-8",
        )

    def case(self, path):
        source = self.tests / path
        source.parent.mkdir(parents=True, exist_ok=True)
        source.write_text("#[test]\nfn example() {}\n", encoding="utf-8")

    def manifest(self, *declarations):
        source = "//! One integration target.\n"
        source += "".join(
            f'\n#[path = "{path}"]\nmod {module};\n'
            for path, module in declarations
        )
        (self.tests / "integration.rs").write_text(source, encoding="utf-8")

    def test_all_root_and_nested_cases_are_included(self):
        self.case("nested/second.rs")
        self.manifest(('first.rs', 'first'), ('nested/second.rs', 'second'))
        self.assertEqual(checker.check(self.root), 2)

    def test_a_new_undeclared_case_fails(self):
        self.case("new_case.rs")
        with self.assertRaisesRegex(ValueError, "missing from integration.rs.*new_case"):
            checker.check(self.root)

    def test_a_declared_missing_case_fails(self):
        self.manifest(('first.rs', 'first'), ('missing.rs', 'missing'))
        with self.assertRaisesRegex(ValueError, "cases are missing.*missing.rs"):
            checker.check(self.root)

    def test_a_case_cannot_be_declared_twice(self):
        self.manifest(('first.rs', 'first'), ('first.rs', 'first'))
        with self.assertRaisesRegex(ValueError, "duplicate Parser case path"):
            checker.check(self.root)

    def test_different_case_paths_cannot_share_a_module_name(self):
        self.case("nested/first.rs")
        self.manifest(('first.rs', 'first'), ('nested/first.rs', 'first'))
        with self.assertRaisesRegex(ValueError, "duplicate Parser module"):
            checker.check(self.root)

    def test_a_module_alias_fails(self):
        self.manifest(('first.rs', 'renamed'))
        with self.assertRaisesRegex(ValueError, "must match its case-file stem"):
            checker.check(self.root)

    def test_helper_directories_are_not_exempt(self):
        self.case("fixtures/reference.rs")
        (self.tests / "fixtures/reference.rs").write_text("const REFERENCE: usize = 1;\n")
        with self.assertRaisesRegex(ValueError, "missing from integration.rs.*fixtures/reference.rs"):
            checker.check(self.root)
        self.manifest(('first.rs', 'first'), ('fixtures/reference.rs', 'reference'))
        self.assertEqual(checker.check(self.root), 2)

    def test_unregistered_root_and_directory_cases_fail(self):
        for relative in ["extra.rs", "extra/main.rs"]:
            with self.subTest(relative=relative):
                self.case(relative)
                target = self.tests / relative
                try:
                    with self.assertRaisesRegex(ValueError, "missing from integration.rs"):
                        checker.check(self.root)
                finally:
                    target.unlink()

    def test_conditionals_other_attributes_and_extra_code_fail(self):
        for unexpected in ["#[cfg(any())]", "#[ignore]", "fn hidden() {}", "mod extra;"]:
            with self.subTest(unexpected=unexpected):
                self.manifest(('first.rs', 'first'))
                path = self.tests / "integration.rs"
                path.write_text(path.read_text().replace('#[path', unexpected + '\n#[path'))
                with self.assertRaisesRegex(ValueError, "only unconditional"):
                    checker.check(self.root)

    def test_noncanonical_and_outside_paths_fail(self):
        for path in [
            ".", "../first.rs", "./first.rs", "nested/../first.rs", "nested/./first.rs",
            "nested//first.rs", "/first.rs", "C:/first.rs", "nested\\first.rs",
            "integration.rs", "first.txt", "nested/\tfirst.rs",
        ]:
            with self.subTest(path=path):
                self.manifest((path, 'first'))
                with self.assertRaisesRegex(ValueError, "invalid Parser case path"):
                    checker.check(self.root)

    def test_an_empty_case_inventory_cannot_pass(self):
        (self.tests / "first.rs").unlink()
        self.manifest()
        with self.assertRaisesRegex(ValueError, "inventory is empty"):
            checker.check(self.root)

    def test_comments_are_allowed_between_unconditional_declarations(self):
        path = self.tests / "integration.rs"
        path.write_text(
            '//! Tests.\n/* outer /* nested */ comment */\n'
            '#[path = "first.rs"] // case\n/* explanation */ mod first; // end\n'
        )
        self.assertEqual(checker.check(self.root), 1)
        path.write_text(path.read_text() + '/* unfinished')
        with self.assertRaisesRegex(ValueError, "unterminated comment"):
            checker.check(self.root)

    def test_cargo_must_disable_automatic_integration_targets(self):
        for package in ['', 'autotests = true', 'autotests = 0', 'autotests = "false"']:
            with self.subTest(package=package):
                self.cargo(package=package)
                with self.assertRaisesRegex(ValueError, "package.autotests must be false"):
                    checker.check(self.root)

    def test_cargo_requires_exactly_one_explicit_target(self):
        correct = '[[test]]\nname = "parser_integration"\npath = "tests/integration.rs"\n'
        for targets in ['', correct + correct, '[test]\nname = "parser_integration"\n']:
            with self.subTest(targets=targets):
                self.cargo(targets=targets)
                with self.assertRaisesRegex(ValueError, "exactly one explicit"):
                    checker.check(self.root)

    def test_cargo_rejects_target_aliases_disabled_harnesses_and_feature_gates(self):
        for target in [
            'name = "other"\npath = "tests/integration.rs"',
            'name = "parser_integration"\npath = "tests/first.rs"',
            'name = "parser_integration"',
            'name = "parser_integration"\npath = "tests/integration.rs"\nharness = false',
            'name = "parser_integration"\npath = "tests/integration.rs"\ntest = false',
            'name = "parser_integration"\npath = "tests/integration.rs"\nrequired-features = []',
            'name = "parser_integration"\npath = "tests/integration.rs"\nbench = false',
        ]:
            with self.subTest(target=target):
                self.cargo(targets='[[test]]\n' + target + '\n')
                with self.assertRaisesRegex(ValueError, "target must be parser_integration"):
                    checker.check(self.root)

    def test_cargo_allows_explicit_true_default_test_and_harness_settings(self):
        self.cargo(targets='[[test]]\nname = "parser_integration"\npath = "tests/integration.rs"\nharness = true\ntest = true\n')
        self.assertEqual(checker.check(self.root), 1)

    def test_malformed_cargo_and_non_table_package_fail(self):
        path = self.crate / "Cargo.toml"
        for source in ['[package', 'package = false\n']:
            with self.subTest(source=source):
                path.write_text(source)
                with self.assertRaises(ValueError):
                    checker.check(self.root)

    def test_a_directory_cannot_be_a_rust_case_file(self):
        (self.tests / "not_a_file.rs").mkdir()
        with self.assertRaisesRegex(ValueError, "case path is not a file"):
            checker.check(self.root)

    def test_symlinked_cases_and_directories_fail(self):
        outside = self.root / "outside"
        outside.mkdir()
        (outside / "second.rs").write_text("#[test]\nfn other() {}\n")
        for relative, target in [('linked.rs', outside / 'second.rs'), ('nested', outside)]:
            with self.subTest(relative=relative):
                link = self.tests / relative
                try:
                    link.symlink_to(target, target_is_directory=target.is_dir())
                except (OSError, NotImplementedError) as error:
                    self.skipTest(str(error))
                try:
                    with self.assertRaisesRegex(ValueError, "cannot be symlinks"):
                        checker.check(self.root)
                finally:
                    link.unlink()


if __name__ == "__main__":
    unittest.main()
