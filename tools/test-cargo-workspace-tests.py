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
    if env!("CARGO_PKG_NAME") == "spite-core" {
        assert!(cfg!(feature = "workspace-unified"));
    }
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


PROFILE_BUILD = '''
fn main() {
    println!("cargo:rustc-env=SPITE_RUNNER_PROBE_OPT_LEVEL={}",
             std::env::var("OPT_LEVEL").unwrap());
}
'''
PROFILE_SOURCE = '''
#[test]
fn compiler_profile_preserves_checks() {
    let expected = if env!("CARGO_PKG_NAME") == "spite-runtime" {
        std::env::var("SPITE_RUNNER_PROBE_RUNTIME_OPT").unwrap()
    } else {
        "0".to_owned()
    };
    assert_eq!(env!("SPITE_RUNNER_PROBE_OPT_LEVEL"), expected);
    assert!(cfg!(debug_assertions));
    assert!(std::panic::catch_unwind(|| {
        std::hint::black_box(u8::MAX) + std::hint::black_box(1u8)
    }).is_err(), "integer overflow checks must remain active");
}
'''


def make_workspace(root, core_name='spite-core', include_rest=True,
                   rest_name='runner-probe-b', profile_probe=False):
    names = ['a', 'b'] if include_rest else ['a']
    (root / 'Cargo.toml').write_text(
        '[workspace]\nmembers=' + json.dumps(names) + '\nresolver="2"\n', encoding='utf-8')
    for name in names:
        package = root / name
        (package / 'src').mkdir(parents=True)
        package_name = core_name if name == 'a' else rest_name
        (package / 'Cargo.toml').write_text(
            f'[package]\nname="{package_name}"\nversion="0.1.0"\nedition="2021"\n'
            '[features]\nworkspace-unified=[]\n'
            + (f'[dependencies]\n{core_name}={{path="../a",features=["workspace-unified"]}}\n'
               if name == 'b' else ''),
            encoding='utf-8')
        (package / 'src/lib.rs').write_text(
            SOURCE + (PROFILE_SOURCE if profile_probe else ''), encoding='utf-8')
        if profile_probe:
            (package / 'build.rs').write_text(PROFILE_BUILD, encoding='utf-8')


class CargoRunnerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory(prefix='spite-ci-runner-probe-')
        cls.root = Path(cls.temporary.name)
        make_workspace(cls.root)
        cls.environment = dict(os.environ)
        cls.environment['SPITE_RUNNER_PROBE_INHERITED'] = 'unchanged'
        cls.environment.pop('SPITE_RUNNER_PROBE_FAIL', None)
        cls.environment['CARGO_TARGET_DIR'] = str(cls.root / 'target')
        subprocess.run(['cargo', '+stable', 'generate-lockfile', '--offline'],
                       cwd=cls.root, env=cls.environment, check=True)

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()

    def run_suite(self, fail=None, suite='all', root=None,
                  extra_arguments=(), extra_environment=None):
        root = root or self.root
        report = root / 'report.json'
        report.unlink(missing_ok=True)
        environment = self.environment.copy()
        if extra_environment:
            environment.update(extra_environment)
        if fail:
            environment['SPITE_RUNNER_PROBE_FAIL'] = fail
        result = subprocess.run(
            [sys.executable, str(RUNNER), '--toolchain', 'stable', '--offline',
             '--suite', suite, '--report', str(report), *extra_arguments],
            cwd=root, env=environment,
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
        result, report = self.run_suite('spite-core')
        self.assertEqual(result.returncode, 1, result.stdout)
        self.assertEqual(len(report['invocations']), 2)
        self.assertEqual(sorted(row['status'] for row in report['invocations']), [0, 101])

    def test_split_suites_preserve_exact_invocations_and_cover_the_workspace(self):
        complete, all_report = self.run_suite()
        core, core_report = self.run_suite(suite='core')
        rest, rest_report = self.run_suite(suite='rest')
        for result in [complete, core, rest]:
            self.assertEqual(result.returncode, 0, result.stdout)
        self.assertEqual([row['package'] for row in core_report['invocations']], ['spite-core'])
        self.assertEqual([row['package'] for row in rest_report['invocations']], ['runner-probe-b'])
        combined = core_report['invocations'] + rest_report['invocations']
        fields = ['command', 'cwd', 'package', 'status', 'summaries']
        project = lambda rows: sorted([{key: row[key] for key in fields} for row in rows],
                                      key=lambda row: row['package'])
        self.assertEqual(project(combined), project(all_report['invocations']),
                         'Commands, working directories, results, and filters must agree')

    def test_failure_stays_in_its_suite(self):
        core, core_report = self.run_suite('spite-core', suite='core')
        rest, rest_report = self.run_suite('spite-core', suite='rest')
        self.assertEqual(core.returncode, 1, core.stdout)
        self.assertEqual(rest.returncode, 0, rest.stdout)
        self.assertEqual(core_report['invocations'][0]['status'], 101)
        self.assertEqual(rest_report['invocations'][0]['status'], 0)

    def test_split_suite_requires_the_core_package(self):
        with tempfile.TemporaryDirectory(prefix='spite-ci-runner-no-core-') as temporary:
            root = Path(temporary)
            make_workspace(root, core_name='runner-probe-a')
            subprocess.run(['cargo', '+stable', 'generate-lockfile', '--offline'],
                           cwd=root, env=self.environment, check=True)
            result, report = self.run_suite(suite='core', root=root)
            self.assertNotEqual(result.returncode, 0, result.stdout)
            self.assertIn('split workspace suites require a spite-core package', result.stdout)
            self.assertIsNone(report)

    def test_empty_remaining_suite_cannot_succeed(self):
        with tempfile.TemporaryDirectory(prefix='spite-ci-runner-only-core-') as temporary:
            root = Path(temporary)
            make_workspace(root, include_rest=False)
            subprocess.run(['cargo', '+stable', 'generate-lockfile', '--offline'],
                           cwd=root, env=self.environment, check=True)
            result, report = self.run_suite(suite='rest', root=root)
            self.assertNotEqual(result.returncode, 0, result.stdout)
            self.assertIn('selected workspace suite recorded no test invocations', result.stdout)
            self.assertIsNone(report)

    def test_compiler_failure_does_not_report_success(self):
        source = self.root / 'a/src/lib.rs'
        source.write_text('not valid Rust source', encoding='utf-8')
        try:
            result, report = self.run_suite()
            self.assertNotEqual(result.returncode, 0, result.stdout)
            self.assertIsNone(report)
        finally:
            source.write_text(SOURCE, encoding='utf-8')

    def test_optional_runtime_optimization_preserves_workspace_invocations(self):
        with tempfile.TemporaryDirectory(prefix='spite-ci-runner-profile-') as temporary:
            root = Path(temporary)
            make_workspace(root, rest_name='spite-runtime', profile_probe=True)
            environment = self.environment.copy()
            environment['CARGO_TARGET_DIR'] = str(root / 'target')
            subprocess.run(['cargo', '+stable', 'generate-lockfile', '--offline'],
                           cwd=root, env=environment, check=True)
            reports = {}
            for optimize in [False, True]:
                for suite in ['all', 'rest']:
                    with self.subTest(optimize=optimize, suite=suite):
                        result, report = self.run_suite(
                            suite=suite, root=root,
                            extra_arguments=['--optimize-runtime'] if optimize else [],
                            extra_environment={
                                'CARGO_TARGET_DIR': str(root / 'target'),
                                'SPITE_RUNNER_PROBE_RUNTIME_OPT': '1' if optimize else '0',
                            })
                        self.assertEqual(result.returncode, 0, result.stdout)
                        self.assertEqual(report['recorded_invocations'], 2)
                        self.assertEqual(report['suite'], suite)
                        self.assertEqual(
                            {row['package'] for row in report['invocations']},
                            {'spite-core', 'spite-runtime'} if suite == 'all'
                            else {'spite-runtime'})
                        for row in report['invocations']:
                            self.assertEqual(row['status'], 0)
                            self.assertEqual(row['command'][1:],
                                             ['--skip', 'pinned_corpus_runs_all_reviewed_variants'])
                            self.assertEqual(row['cwd'], str(root / (
                                'a' if row['package'] == 'spite-core' else 'b')))
                            self.assertEqual(row['summaries'],
                                             [['ok', '3', '0', '1', '0', '1']])
                            self.assertNotIn('environment', json.dumps(row))
                        reports[optimize, suite] = report

            # Executable names can change when the runtime profile changes;
            # every recorded argument, directory and test outcome must agree.
            def project(rows):
                return sorted([{
                    'arguments': row['command'][1:], 'cwd': row['cwd'],
                    'package': row['package'], 'status': row['status'],
                    'summaries': row['summaries'],
                } for row in rows], key=lambda row: row['package'])

            for suite in ['all', 'rest']:
                self.assertEqual(project(reports[False, suite]['invocations']),
                                 project(reports[True, suite]['invocations']))
            for optimize in [False, True]:
                runtime = [row for row in reports[optimize, 'all']['invocations']
                           if row['package'] == 'spite-runtime']
                self.assertEqual(project(runtime),
                                 project(reports[optimize, 'rest']['invocations']))
                self.assertEqual(runtime[0]['command'],
                                 reports[optimize, 'rest']['invocations'][0]['command'],
                                 'The rest suite must execute the recorded runtime artifact')
            core_commands = [[row['command'] for row in reports[optimize, 'all']['invocations']
                              if row['package'] == 'spite-core'] for optimize in [False, True]]
            self.assertEqual(core_commands[0], core_commands[1],
                             'The Core test artifact must stay unchanged')

            # The override must not swallow failures or select the Core package
            # into the rest suite. Both cases execute real compiled test code.
            for fail, suite, expected_statuses in [
                    ('spite-runtime', 'all', [0, 101]),
                    ('spite-core', 'rest', [0])]:
                with self.subTest(fail=fail, suite=suite):
                    result, report = self.run_suite(
                        fail=fail, suite=suite, root=root,
                        extra_arguments=['--optimize-runtime'],
                        extra_environment={
                            'CARGO_TARGET_DIR': str(root / 'target'),
                            'SPITE_RUNNER_PROBE_RUNTIME_OPT': '1',
                        })
                    self.assertEqual(result.returncode, int(101 in expected_statuses), result.stdout)
                    self.assertEqual(report['recorded_invocations'], 2)
                    self.assertEqual(sorted(row['status'] for row in report['invocations']),
                                     expected_statuses)
                    self.assertEqual({row['package'] for row in report['invocations']},
                                     {'spite-core', 'spite-runtime'} if suite == 'all'
                                     else {'spite-runtime'})

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
