#!/usr/bin/env python3
"""Exercise the CI runner using real, dependency-free Cargo test packages."""

import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock


RUNNER = Path(__file__).with_name('run-cargo-workspace-tests.py')
SOURCE = '''
#[test]
fn package_environment_and_directory() {
    assert_eq!(std::env::current_dir().unwrap(), std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    assert_eq!(std::env::var("CARGO_MANIFEST_DIR").unwrap(), env!("CARGO_MANIFEST_DIR"));
    assert_eq!(std::env::var("CARGO_PKG_NAME").unwrap(), env!("CARGO_PKG_NAME"));
    assert_eq!(std::env::var("SPITE_RUNNER_PROBE_INHERITED").unwrap(), "unchanged");
}
#[test]
fn requested_failure() {
    assert_ne!(std::env::var("SPITE_RUNNER_PROBE_FAIL").unwrap_or_default(), env!("CARGO_PKG_NAME"));
}
#[test]
#[ignore]
fn ignored_test_remains_ignored() { panic!("ignored test executed"); }
#[test]
fn pinned_corpus_runs_all_reviewed_variants() { panic!("corpus must run separately"); }
'''


class CargoRunnerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory(prefix='spite-ci-runner-probe-')
        cls.root = Path(cls.temporary.name)
        (cls.root / 'Cargo.toml').write_text(
            '[workspace]\nmembers=["a","b"]\nresolver="2"\n', encoding='utf-8')
        for name in ['a', 'b']:
            package = cls.root / name
            (package / 'src').mkdir(parents=True)
            (package / 'Cargo.toml').write_text(
                f'[package]\nname="runner-probe-{name}"\nversion="0.1.0"\nedition="2021"\n',
                encoding='utf-8')
            (package / 'src/lib.rs').write_text(SOURCE, encoding='utf-8')
        cls.environment = dict(os.environ)
        cls.environment['SPITE_RUNNER_PROBE_INHERITED'] = 'unchanged'
        cls.environment.pop('SPITE_RUNNER_PROBE_FAIL', None)
        cls.environment['CARGO_TARGET_DIR'] = str(cls.root / 'target')
        subprocess.run(['cargo', '+stable', 'generate-lockfile', '--offline'],
                       cwd=cls.root, env=cls.environment, check=True)

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()

    def run_suite(self, fail=None):
        report = self.root / 'report.json'
        report.unlink(missing_ok=True)
        environment = self.environment.copy()
        if fail:
            environment['SPITE_RUNNER_PROBE_FAIL'] = fail
        result = subprocess.run(
            [sys.executable, str(RUNNER), '--toolchain', 'stable', '--offline',
             '--report', str(report)], cwd=self.root, env=environment,
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        return result, json.loads(report.read_text()) if report.exists() else None

    def test_package_environment_filter_and_ignored_tests(self):
        result, report = self.run_suite()
        self.assertEqual(result.returncode, 0, result.stdout)
        self.assertEqual(len(report['invocations']), 2)
        self.assertEqual({row['cwd'] for row in report['invocations']},
                         {str(self.root / 'a'), str(self.root / 'b')})
        for row in report['invocations']:
            self.assertEqual(row['status'], 0)
            self.assertEqual(row['command'][1:],
                             ['--skip', 'pinned_corpus_runs_all_reviewed_variants'])
            self.assertEqual(row['summaries'], [['ok', '2', '0', '1', '0', '1']])
            self.assertNotIn('environment', json.dumps(row))

    def test_failure_propagates_and_other_package_still_runs(self):
        result, report = self.run_suite('runner-probe-a')
        self.assertEqual(result.returncode, 1, result.stdout)
        self.assertEqual(len(report['invocations']), 2)
        self.assertEqual(sorted(row['status'] for row in report['invocations']), [0, 101])

    def test_compiler_failure_does_not_report_success(self):
        source = self.root / 'a/src/lib.rs'
        source.write_text('not valid Rust source', encoding='utf-8')
        try:
            result, report = self.run_suite()
            self.assertNotEqual(result.returncode, 0, result.stdout)
            self.assertIsNone(report)
        finally:
            source.write_text(SOURCE, encoding='utf-8')

    def test_missing_records_cannot_succeed(self):
        specification = importlib.util.spec_from_file_location('workspace_runner', RUNNER)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        with mock.patch.object(sys, 'argv', [str(RUNNER), '--toolchain', 'stable']), \
             mock.patch.object(module.subprocess, 'check_output', return_value='host: test-host\n'), \
             mock.patch.object(module.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0)):
            with self.assertRaisesRegex(RuntimeError, 'no test invocations'):
                module.main()


if __name__ == '__main__':
    unittest.main()
