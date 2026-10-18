#!/usr/bin/env python3
"""Exercise the CI runner using real, dependency-free Cargo test packages."""

import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
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

RUNTIME_SHARD_SOURCE = '''
fn check() {
    assert_eq!(std::env::current_dir().unwrap(), std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    assert_eq!(std::env::var("CARGO_MANIFEST_DIR").unwrap(), env!("CARGO_MANIFEST_DIR"));
    assert_eq!(std::env::var("CARGO_PKG_NAME").unwrap(), "spite-runtime");
    assert_eq!(std::env::var("SPITE_RUNNER_PROBE_INHERITED").unwrap(), "unchanged");
    assert!(spite_core::workspace_feature_is_unified());
    assert_eq!(env!("SPITE_RUNNER_PROBE_OPT_LEVEL"),
               std::env::var("SPITE_RUNNER_PROBE_RUNTIME_OPT").unwrap());
    assert!(cfg!(debug_assertions));
    assert!(std::panic::catch_unwind(|| {
        std::hint::black_box(u8::MAX) + std::hint::black_box(1u8)
    }).is_err(), "integer overflow checks must remain active");
    assert_ne!(std::env::var("SPITE_RUNNER_PROBE_FAIL_CASE").unwrap_or_default(),
               std::thread::current().name().unwrap());
}
#[test]
fn root_case() { check(); }
#[test]
fn root_profile() { check(); }
mod arrays {
    #[test]
    fn fixture_environment() { super::check(); }
}
mod typed_arrays {
    #[test]
    fn fixture_environment() { super::check(); }
}
mod nested {
    mod arrays {
        #[test]
        fn fixture_environment() { super::super::check(); }
    }
    #[test]
    fn root_case() { super::check(); }
}
mod independent_a {
    #[test]
    fn fixture_environment() { super::check(); }
}
mod independent_b {
    #[test]
    fn fixture_environment() { super::check(); }
}
'''
RUNTIME_SHARD_NAMES = sorted([
    'root_case', 'root_profile', 'arrays::fixture_environment',
    'typed_arrays::fixture_environment', 'nested::arrays::fixture_environment',
    'nested::root_case', 'independent_a::fixture_environment',
    'independent_b::fixture_environment',
])


def names_sha256(names):
    return hashlib.sha256(json.dumps(sorted(names), ensure_ascii=True,
                                     separators=(',', ':')).encode('utf-8')).hexdigest()


def make_runtime_shard_workspace(root):
    make_workspace(root, rest_name='spite-runtime', profile_probe=True)
    with (root / 'a/src/lib.rs').open('a', encoding='utf-8') as source:
        source.write('''
pub fn workspace_feature_is_unified() -> bool { cfg!(feature = "workspace-unified") }
''')
    (root / 'b/tests').mkdir()
    (root / 'b/tests/integration.rs').write_text(RUNTIME_SHARD_SOURCE, encoding='utf-8')
    (root / 'Cargo.toml').write_text(
        '[workspace]\nmembers=["a","b","c"]\nresolver="2"\n', encoding='utf-8')
    (root / 'c/src').mkdir(parents=True)
    (root / 'c/Cargo.toml').write_text(
        '[package]\nname="runner-probe-c"\nversion="0.1.0"\nedition="2021"\n'
        '[features]\nworkspace-unified=[]\n', encoding='utf-8')
    (root / 'c/src/lib.rs').write_text(SOURCE + PROFILE_SOURCE, encoding='utf-8')
    (root / 'c/build.rs').write_text(PROFILE_BUILD, encoding='utf-8')


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


def make_integration_only_workspace(root):
    """Exercise covered dev-feature context without workspace feature maps/scripts."""
    (root / 'Cargo.toml').write_text(
        '[workspace]\nmembers=["a","b","c"]\nresolver="2"\n', encoding='utf-8')
    shared = root.parent / 'shared'
    (shared / 'src').mkdir(parents=True)
    (shared / 'Cargo.toml').write_text(
        '[package]\nname="probe-shared"\nversion="0.1.0"\nedition="2021"\n'
        '[features]\nwide=[]\n', encoding='utf-8')
    (shared / 'src/lib.rs').write_text(
        'pub fn enabled() -> bool { cfg!(feature = "wide") }\n', encoding='utf-8')
    binding = ('[dev-dependencies]\nshared_alias={package="probe-shared",'
               'path="../../shared",default-features=false,features=["wide"]}\n')
    for directory, name in [('a', 'spite-core'), ('b', 'spite-runtime'),
                            ('c', 'runner-probe-c')]:
        package = root / directory
        (package / 'src').mkdir(parents=True)
        dependencies = ('[dependencies]\nspite-core={path="../a"}\n' if directory == 'b'
                        else '[dependencies]\nspite-runtime={path="../b"}\n'
                        if directory == 'c' else '')
        (package / 'Cargo.toml').write_text(
            f'[package]\nname="{name}"\nversion="0.1.0"\nedition="2021"\n'
            + dependencies + binding, encoding='utf-8')
        (package / 'src/lib.rs').write_text(
            'pub fn primary() -> bool { option_env!("CARGO_PRIMARY_PACKAGE").is_some() }\n',
            encoding='utf-8')
    source = RUNTIME_SHARD_SOURCE.replace(
        'assert!(spite_core::workspace_feature_is_unified());',
        'assert!(spite_core::primary());\n    assert!(shared_alias::enabled());')
    source = source.replace(
        '    assert_eq!(env!("SPITE_RUNNER_PROBE_OPT_LEVEL"),\n'
        '               std::env::var("SPITE_RUNNER_PROBE_RUNTIME_OPT").unwrap());\n', '')
    (root / 'b/tests').mkdir()
    (root / 'b/tests/integration.rs').write_text(source, encoding='utf-8')


# This test-only entry retains the real runner's __file__, recorder and main.
# Only hash observations are written; the returned child environment is unchanged.
OBSERVE_RUNTIME_ENVIRONMENT = '''
import hashlib, json, runpy, sys
from pathlib import Path
runner, observation, *arguments = sys.argv[1:]
namespace = runpy.run_path(runner, run_name="runtime_environment_probe")
main = namespace["main"]
original = main.__globals__["invocation_environment"]
rows = []
def observe(invocation, baseline):
    environment = original(invocation, baseline)
    rows.append({"command": invocation["command"], "cwd": invocation["cwd"],
                 "package": invocation["package"],
                 "environment": {key: hashlib.sha256(value.encode("utf-8")).hexdigest()
                                 for key, value in environment.items()}})
    Path(observation).write_text(json.dumps(rows), encoding="utf-8")
    return environment
main.__globals__["invocation_environment"] = observe
sys.argv = [runner, *arguments]
sys.exit(main())
'''


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
                  extra_arguments=(), extra_environment=None, toolchain='stable'):
        root = root or self.root
        report = root / 'report.json'
        report.unlink(missing_ok=True)
        environment = self.environment.copy()
        if extra_environment:
            environment.update(extra_environment)
        if fail:
            environment['SPITE_RUNNER_PROBE_FAIL'] = fail
        result = subprocess.run(
            [sys.executable, str(RUNNER), '--toolchain', toolchain, '--offline',
             '--suite', suite, '--report', str(report), *extra_arguments],
            cwd=root, env=environment,
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        return result, json.loads(report.read_text()) if report.exists() else None

    def integration_collection_context(self, root, toolchain, environment):
        """Collect the actual old library+integration graph without executing Rust."""
        records = root / ('records-' + toolchain)
        records.mkdir()
        fingerprint = lambda value: hashlib.sha256(value.encode('utf-8')).hexdigest()
        (records / 'environment-fingerprints.json').write_text(
            json.dumps({key: fingerprint(value) for key, value in environment.items()}),
            encoding='utf-8')
        version = subprocess.check_output(['rustc', '+' + toolchain, '-vV'],
                                          text=True, env=environment)
        host = re.search(r'^host: ([A-Za-z0-9_-]+)$', version, re.MULTILINE)[1]
        runner = json.dumps([sys.executable, str(RUNNER.resolve()), '--record', str(records)])
        result = subprocess.run(
            ['cargo', '+' + toolchain, 'test', '--workspace', '--lib', '--test', 'integration',
             '--locked', '--offline', '--config', 'target.' + host + '.runner=' + runner,
             '--', '--skip', 'pinned_corpus_runs_all_reviewed_variants'],
            cwd=root, env=environment, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        self.assertEqual(result.returncode, 0, result.stdout)
        rows = [json.loads(path.read_text(encoding='utf-8'))
                for path in records.glob('invocation-*.json')]
        self.assertEqual(len(rows), 4)
        runtime = next(row for row in rows
                       if re.fullmatch(r'integration-[0-9a-f]{16}(?:\.exe)?',
                                       Path(row['command'][0]).name))
        effective = environment.copy()
        effective.update(runtime['changed_environment'])
        for key in runtime['removed_environment']:
            effective.pop(key, None)
        listing = subprocess.run(runtime['command'] + ['--list'], cwd=runtime['cwd'],
                                 env=effective, stdout=subprocess.PIPE,
                                 stderr=subprocess.STDOUT, text=True)
        self.assertEqual(listing.returncode, 0, listing.stdout)
        self.assertEqual(sorted(re.findall(r'^(.+): test$', listing.stdout, re.MULTILINE)),
                         RUNTIME_SHARD_NAMES)
        return ({'command': runtime['command'], 'cwd': runtime['cwd'],
                 'package': runtime['package'],
                 'environment': {key: fingerprint(value) for key, value in effective.items()}},
                hashlib.sha256(Path(runtime['command'][0]).read_bytes()).hexdigest())

    def observed_integration_shard(self, root, toolchain, environment, index, full=False):
        selection = 'full' if full else index
        observation = root / f'environment-{toolchain}-{selection}.json'
        report = root / f'report-{toolchain}-{selection}.json'
        observer = root / 'observe.py'
        observer.write_text(OBSERVE_RUNTIME_ENVIRONMENT, encoding='utf-8')
        arguments = (['--runtime-integration', 'only'] if full else
                     ['--runtime-shard', str(index), '--rest-shard', str(1 - index)])
        result = subprocess.run(
            [sys.executable, str(observer), str(RUNNER.resolve()), str(observation),
             '--toolchain', toolchain, '--offline', '--suite', 'rest',
             *arguments,
             '--report', str(report)], cwd=root, env=environment,
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        self.assertEqual(result.returncode, 0, result.stdout)
        return (result, json.loads(report.read_text(encoding='utf-8')),
                json.loads(observation.read_text(encoding='utf-8')))

    def test_runtime_target_only_preserves_library_context_binary_and_environment(self):
        for toolchain in ['stable', '1.85.0']:
            with self.subTest(toolchain=toolchain), tempfile.TemporaryDirectory(
                    prefix='spite-ci-integration-only-') as temporary:
                root = Path(temporary) / 'workspace'
                root.mkdir()
                make_integration_only_workspace(root)
                environment = dict(self.environment, CARGO_TARGET_DIR=str(root / 'target'))
                subprocess.run(['cargo', '+' + toolchain, 'generate-lockfile', '--offline'],
                               cwd=root, env=environment, check=True)
                # Run before the reference collection: Cargo may reuse a library
                # compiled with CARGO_PRIMARY_PACKAGE from that earlier build.
                cold_result, cold_report, cold_observations = self.observed_integration_shard(
                    root, toolchain, environment, 1)
                self.assertIn('Runtime integration library collection: eligible; '
                              'using --workspace --lib --test integration', cold_result.stdout)
                self.assertEqual(cold_report['recorded_invocations'], 4)
                original, binary_hash = self.integration_collection_context(root, toolchain, environment)
                for observed in cold_observations:
                    self.assertEqual(observed['environment'], original['environment'])
                    self.assertEqual(observed['cwd'], original['cwd'])
                    self.assertEqual(observed['command'][:len(original['command'])],
                                     original['command'])
                executed = []
                for index in [0, 1]:
                    result, report, observations = self.observed_integration_shard(
                        root, toolchain, environment, index)
                    self.assertIn('Runtime integration library collection: eligible; '
                                  'using --workspace --lib --test integration', result.stdout)
                    self.assertNotIn('Runtime integration target collection:', result.stdout)
                    self.assertEqual(report['recorded_invocations'], 4)
                    self.assertEqual(len(report['invocations']), 1)
                    self.assertEqual(len(observations), 2)
                    for observed in observations:
                        self.assertEqual(observed['package'], original['package'])
                        self.assertEqual(observed['cwd'], original['cwd'])
                        self.assertEqual(observed['command'][:len(original['command'])],
                                         original['command'])
                        self.assertEqual(observed['environment'], original['environment'])
                    row = report['invocations'][0]
                    self.assertEqual(row['status'], 0)
                    self.assertEqual(row['child_status'], 0)
                    self.assertEqual(row['runtime_shard_execution']['names'],
                                     report['runtime_shard']['planned_shards'][index])
                    self.assertEqual(report['runtime_shard']['full_names'], RUNTIME_SHARD_NAMES)
                    self.assertEqual(hashlib.sha256(Path(row['command'][0]).read_bytes()).hexdigest(),
                                     binary_hash)
                    executed.extend(row['runtime_shard_execution']['names'])
                self.assertEqual(sorted(executed), RUNTIME_SHARD_NAMES)

    def test_runtime_target_only_falls_back_for_unproved_workspace_contexts(self):
        for toolchain in ['stable', '1.85.0']:
            for shape in ['default-feature', 'custom-build', 'build-dependency',
                          'renamed-dev-binding']:
                with self.subTest(toolchain=toolchain, shape=shape), tempfile.TemporaryDirectory(
                        prefix='spite-ci-integration-context-') as temporary:
                    root = Path(temporary) / 'workspace'
                    root.mkdir()
                    make_integration_only_workspace(root)
                    if shape == 'default-feature':
                        with (root / 'a/Cargo.toml').open('a', encoding='utf-8') as manifest:
                            manifest.write('[features]\ndefault=["wide"]\nwide=[]\n')
                        manifest = root / 'b/Cargo.toml'
                        manifest.write_text(manifest.read_text().replace(
                            'spite-core={path="../a"}',
                            'spite-core={path="../a",default-features=false}'), encoding='utf-8')
                        with (root / 'a/src/lib.rs').open('a', encoding='utf-8') as source:
                            source.write('pub fn wide() -> bool { cfg!(feature = "wide") }\n')
                        source = root / 'b/tests/integration.rs'
                        source.write_text(source.read_text().replace(
                            'assert!(spite_core::primary());',
                            'assert!(spite_core::primary());\n    assert!(spite_core::wide());'),
                            encoding='utf-8')
                    elif shape == 'custom-build':
                        (root / 'c/build.rs').write_text('fn main() {}\n', encoding='utf-8')
                    elif shape == 'build-dependency':
                        with (root / 'c/Cargo.toml').open('a', encoding='utf-8') as manifest:
                            manifest.write('[build-dependencies]\nshared_alias={package="probe-shared",'
                                           'path="../../shared",default-features=false,features=["wide"]}\n')
                    else:
                        manifest = root / 'c/Cargo.toml'
                        manifest.write_text(manifest.read_text().replace(
                            'shared_alias={', 'different_alias={'), encoding='utf-8')
                    environment = dict(self.environment, CARGO_TARGET_DIR=str(root / 'target'))
                    subprocess.run(['cargo', '+' + toolchain, 'generate-lockfile', '--offline'],
                                   cwd=root, env=environment, check=True)
                    original, binary_hash = self.integration_collection_context(
                        root, toolchain, environment)
                    result, report, observations = self.observed_integration_shard(
                        root, toolchain, environment, 1)
                    self.assertIn('Runtime integration library collection: eligible; '
                                  'using --workspace --lib --test integration', result.stdout)
                    self.assertNotIn('Runtime integration target collection:', result.stdout)
                    self.assertEqual(report['recorded_invocations'], 4)
                    self.assertEqual(len(report['invocations']), 1)
                    self.assertEqual(len(observations), 2)
                    for observed in observations:
                        self.assertEqual(observed['environment'], original['environment'])
                        self.assertEqual(observed['cwd'], original['cwd'])
                        self.assertEqual(observed['command'][:len(original['command'])],
                                         original['command'])
                    self.assertEqual(hashlib.sha256(Path(original['command'][0]).read_bytes()).hexdigest(),
                                     binary_hash)

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

    def test_core_library_collection_retains_sibling_and_dev_features(self):
        with tempfile.TemporaryDirectory(prefix='spite-ci-core-libs-features-') as temporary:
            root = Path(temporary)
            make_workspace(root)
            manifest = root / 'a/Cargo.toml'
            manifest.write_text(manifest.read_text().replace(
                'workspace-unified=[]\n',
                'workspace-unified=[]\nsibling-dev-unified=[]\n'), encoding='utf-8')
            with (root / 'a/src/lib.rs').open('a', encoding='utf-8') as source:
                source.write('''
#[test]
fn sibling_dev_feature_and_primary_package_remain_active() {
    assert!(cfg!(feature = "sibling-dev-unified"));
    assert!(option_env!("CARGO_PRIMARY_PACKAGE").is_some());
}
''')
            with (root / 'b/Cargo.toml').open('a', encoding='utf-8') as manifest:
                manifest.write('[dev-dependencies]\n'
                               'spite-core={path="../a",features=["sibling-dev-unified"]}\n')
            (root / 'b/tests').mkdir()
            (root / 'b/tests/extra.rs').write_text(
                '#[test]\nfn sibling_integration_target() {}\n', encoding='utf-8')
            environment = {'CARGO_TARGET_DIR': str(root / 'target')}
            subprocess.run(['cargo', '+stable', 'generate-lockfile', '--offline'],
                           cwd=root, env={**self.environment, **environment}, check=True)
            fields = ['command', 'cwd', 'package', 'status', 'summaries']
            for toolchain in ['stable', '1.85.0']:
                with self.subTest(toolchain=toolchain):
                    reference, original = self.run_suite(
                        suite='core', root=root, toolchain=toolchain,
                        extra_environment=environment)
                    result, report = self.run_suite(
                        suite='core', root=root, toolchain=toolchain,
                        extra_arguments=['--collect-core-libs'],
                        extra_environment=environment)
                    self.assertEqual(reference.returncode, 0, reference.stdout)
                    self.assertEqual(result.returncode, 0, result.stdout)
                    self.assertEqual(original['recorded_invocations'], 3)
                    self.assertEqual(report['recorded_invocations'], 2)
                    self.assertEqual(len(original['invocations']), 1)
                    self.assertEqual(len(report['invocations']), 1)
                    self.assertEqual(
                        {key: original['invocations'][0][key] for key in fields},
                        {key: report['invocations'][0][key] for key in fields})
                    self.assertEqual(report['invocations'][0]['summaries'],
                                     [['ok', '3', '0', '1', '0', '1']])
                    decision = 'Core library collection: eligible; using --workspace --lib'
                    self.assertEqual(result.stdout.count(decision), 1)
                    self.assertLess(result.stdout.index(decision),
                                    result.stdout.index('Queued Cargo test invocation:'))
                    self.assertEqual(result.stdout.count('Queued Cargo test invocation:'), 2)
                    self.assertNotIn('Core library collection:', reference.stdout)
                    failed, failure = self.run_suite(
                        fail='spite-core', suite='core', root=root, toolchain=toolchain,
                        extra_arguments=['--collect-core-libs'],
                        extra_environment=environment)
                    self.assertEqual(failed.returncode, 1, failed.stdout)
                    self.assertEqual(failure['recorded_invocations'], 2)
                    self.assertEqual([row['status'] for row in failure['invocations']], [101])
                    self.assertEqual(failed.stdout.count(decision), 1)
                    self.assertEqual(failed.stdout.count('Queued Cargo test invocation:'), 2)
                    self.assertNotIn('ineligible; using', failed.stdout)

    def test_core_library_collection_falls_back_before_compiling_new_target_shapes(self):
        for shape, recorded, retained in [
                ('core-integration', 3, 2), ('core-bin', 3, 2), ('sibling-bin-only', 2, 1)]:
            with self.subTest(shape=shape), tempfile.TemporaryDirectory(
                    prefix='spite-ci-core-libs-fallback-') as temporary:
                root = Path(temporary)
                make_workspace(root)
                if shape == 'core-integration':
                    (root / 'a/tests').mkdir()
                    (root / 'a/tests/extra.rs').write_text(
                        '#[test]\nfn core_integration_target_runs() {}\n', encoding='utf-8')
                elif shape == 'core-bin':
                    (root / 'a/src/main.rs').write_text(
                        'fn main() {}\n#[test]\nfn core_binary_target_runs() {}\n',
                        encoding='utf-8')
                else:
                    (root / 'b/src/lib.rs').unlink()
                    (root / 'b/src/main.rs').write_text(
                        'fn main() {}\n' + SOURCE, encoding='utf-8')
                environment = {'CARGO_TARGET_DIR': str(root / 'target')}
                subprocess.run(['cargo', '+stable', 'generate-lockfile', '--offline'],
                               cwd=root, env={**self.environment, **environment}, check=True)
                reference, original = self.run_suite(
                    suite='core', root=root, extra_environment=environment)
                result, report = self.run_suite(
                    suite='core', root=root, extra_arguments=['--collect-core-libs'],
                    extra_environment=environment)
                self.assertEqual(reference.returncode, 0, reference.stdout)
                self.assertEqual(result.returncode, 0, result.stdout)
                self.assertEqual(original['recorded_invocations'], recorded)
                self.assertEqual(report['recorded_invocations'], recorded)
                self.assertEqual(len(report['invocations']), retained)
                fields = ['command', 'cwd', 'package', 'status', 'summaries']
                project = lambda rows: sorted(
                    [{key: row[key] for key in fields} for row in rows],
                    key=lambda row: row['command'][0])
                self.assertEqual(project(original['invocations']), project(report['invocations']))
                decision = 'Core library collection: ineligible; using --workspace --all-targets'
                self.assertEqual(result.stdout.count(decision), 1)
                self.assertLess(result.stdout.index(decision),
                                result.stdout.index('Queued Cargo test invocation:'))
                self.assertEqual(result.stdout.count('Queued Cargo test invocation:'), recorded)
                self.assertNotIn('Core library collection:', reference.stdout)

    def test_core_library_collection_rejects_other_suites_and_metadata_errors(self):
        for suite in ['all', 'rest']:
            with self.subTest(suite=suite):
                result, report = self.run_suite(
                    suite=suite, extra_arguments=['--collect-core-libs'])
                self.assertNotEqual(result.returncode, 0, result.stdout)
                self.assertIsNone(report)
                self.assertIn('--collect-core-libs requires --suite core', result.stdout)
                self.assertNotIn('Queued Cargo test invocation:', result.stdout)
        with tempfile.TemporaryDirectory(prefix='spite-ci-core-libs-metadata-error-') as temporary:
            root = Path(temporary)
            make_workspace(root)
            (root / 'Cargo.toml').write_text('[workspace\n', encoding='utf-8')
            result, report = self.run_suite(
                suite='core', root=root, extra_arguments=['--collect-core-libs'],
                extra_environment={'CARGO_TARGET_DIR': str(root / 'target')})
            self.assertNotEqual(result.returncode, 0, result.stdout)
            self.assertIsNone(report)
            self.assertNotIn('Core library collection:', result.stdout)
            self.assertNotIn('Collecting Cargo test invocations', result.stdout)
            self.assertNotIn('Queued Cargo test invocation:', result.stdout)

    def test_runtime_library_collection_preserves_actual_binary_and_dev_roots(self):
        with tempfile.TemporaryDirectory(prefix='spite-ci-runtime-libs-features-') as temporary:
            root = Path(temporary)
            make_runtime_shard_workspace(root)
            (root / 'a/build.rs').unlink()
            core_source = root / 'a/src/lib.rs'
            core_source.write_text(core_source.read_text().replace(PROFILE_SOURCE, '', 1),
                                   encoding='utf-8')
            manifest = root / 'a/Cargo.toml'
            manifest.write_text(manifest.read_text().replace(
                'workspace-unified=[]\n',
                'workspace-unified=[]\nsibling-dev-unified=[]\n'), encoding='utf-8')
            with (root / 'a/src/lib.rs').open('a', encoding='utf-8') as source:
                source.write('''
pub fn sibling_dev_and_primary_roots() -> (bool, bool) {
    (cfg!(feature = "sibling-dev-unified"), option_env!("CARGO_PRIMARY_PACKAGE").is_some())
}
''')
            with (root / 'c/Cargo.toml').open('a', encoding='utf-8') as manifest:
                manifest.write('[dev-dependencies]\n'
                               'spite-core={path="../a",features=["sibling-dev-unified"]}\n')
            (root / 'c/tests').mkdir()
            (root / 'c/tests/extra.rs').write_text('''
#[test]
fn sibling_dev_context_is_selected() {
    assert_eq!(spite_core::sibling_dev_and_primary_roots(), (true, true));
}
''', encoding='utf-8')
            (root / 'b/tests/integration.rs').write_text(RUNTIME_SHARD_SOURCE.replace(
                '    assert!(spite_core::workspace_feature_is_unified());',
                '    assert!(spite_core::workspace_feature_is_unified());\n'
                '    assert_eq!(spite_core::sibling_dev_and_primary_roots(), (true, true));'),
                encoding='utf-8')
            environment = {'CARGO_TARGET_DIR': str(root / 'target'),
                           'SPITE_RUNNER_PROBE_RUNTIME_OPT': '0',
                           'SPITE_RUNNER_PROBE_FAIL_CASE': ''}
            subprocess.run(['cargo', '+stable', 'generate-lockfile', '--offline'],
                           cwd=root, env={**self.environment, **environment}, check=True)
            for toolchain in ['stable', '1.85.0']:
                with self.subTest(toolchain=toolchain):
                    reference, complete = self.run_suite(
                        suite='all', root=root, toolchain=toolchain,
                        extra_environment=environment)
                    self.assertEqual(reference.returncode, 0, reference.stdout)
                    self.assertEqual(complete['recorded_invocations'], 5)
                    original = self.runtime_row(complete)
                    original_hash = hashlib.sha256(Path(original['command'][0]).read_bytes()).hexdigest()
                    for index in [1, 0]:
                        result, report = self.run_suite(
                            suite='rest', root=root, toolchain=toolchain,
                            extra_arguments=['--runtime-shard', str(index)] +
                                (['--rest-shard', '1'] if index == 0 else []),
                                extra_environment=environment)
                        runtime = self.assert_runtime_shard(
                            result, report, index, RUNTIME_SHARD_NAMES, rest_owner=1 - index)
                        self.assertEqual(runtime['command'][0], original['command'][0])
                        self.assertEqual(runtime['cwd'], original['cwd'])
                        self.assertEqual(hashlib.sha256(Path(runtime['command'][0]).read_bytes()).hexdigest(),
                                         original_hash)
                        decision = ('Runtime integration library collection: eligible; '
                                    'using --workspace --lib --test integration')
                        self.assertEqual(result.stdout.count(decision), 1)
                        self.assertLess(result.stdout.index(decision),
                                        result.stdout.index('Queued Cargo test invocation:'))
                        self.assertNotIn('Runtime integration library collection:', reference.stdout)

    def test_runtime_library_collection_falls_back_before_ambiguous_or_disabled_targets(self):
        for shape, recorded in [('second-integration', 5), ('integration-bin', 5),
                                ('sibling-bin-only', 4), ('required-integration-feature', 4)]:
            with self.subTest(shape=shape), tempfile.TemporaryDirectory(
                    prefix='spite-ci-runtime-libs-fallback-') as temporary:
                root = Path(temporary)
                make_runtime_shard_workspace(root)
                (root / 'a/build.rs').unlink()
                core_source = root / 'a/src/lib.rs'
                core_source.write_text(core_source.read_text().replace(PROFILE_SOURCE, '', 1),
                                       encoding='utf-8')
                if shape == 'second-integration':
                    (root / 'c/tests').mkdir()
                    (root / 'c/tests/integration.rs').write_text(
                        '#[test]\nfn another_integration_target() {}\n', encoding='utf-8')
                elif shape == 'integration-bin':
                    (root / 'c/src/bin').mkdir()
                    (root / 'c/src/bin/integration.rs').write_text(
                        'fn main() {}\n#[test]\nfn binary_named_integration() {}\n', encoding='utf-8')
                elif shape == 'sibling-bin-only':
                    (root / 'c/src/lib.rs').unlink()
                    (root / 'c/src/main.rs').write_text(
                        'fn main() {}\n#[test]\nfn binary_only_workspace_member() {}\n', encoding='utf-8')
                else:
                    manifest = root / 'b/Cargo.toml'
                    manifest.write_text(manifest.read_text().replace(
                        'workspace-unified=[]\n',
                        'workspace-unified=[]\ndefault=["gated"]\ngated=[]\n') +
                        '\n[[test]]\nname="integration"\npath="tests/integration.rs"\n'
                        'required-features=["gated"]\n', encoding='utf-8')
                environment = {'CARGO_TARGET_DIR': str(root / 'target'),
                               'SPITE_RUNNER_PROBE_RUNTIME_OPT': '0',
                               'SPITE_RUNNER_PROBE_FAIL_CASE': ''}
                subprocess.run(['cargo', '+stable', 'generate-lockfile', '--offline'],
                               cwd=root, env={**self.environment, **environment}, check=True)
                for toolchain in ['stable', '1.85.0']:
                    with self.subTest(toolchain=toolchain):
                        reference, complete = self.run_suite(
                            suite='rest', root=root, toolchain=toolchain,
                            extra_environment=environment)
                        self.assertEqual(reference.returncode, 0, reference.stdout)
                        original = self.runtime_row(complete)
                        original_hash = hashlib.sha256(Path(original['command'][0]).read_bytes()).hexdigest()
                        for index in [1, 0]:
                            result, report = self.run_suite(
                                suite='rest', root=root, toolchain=toolchain,
                                extra_arguments=['--runtime-shard', str(index)] +
                                    (['--rest-shard', '1'] if index == 0 else []),
                                    extra_environment=environment)
                            self.assertEqual(result.returncode, 0, result.stdout)
                            self.assertEqual(complete['recorded_invocations'], recorded)
                            self.assertEqual(report['recorded_invocations'], recorded)
                            self.assertEqual(len(report['invocations']), 1)
                            runtime = self.runtime_row(report)
                            self.assertEqual(runtime['command'][0], original['command'][0])
                            self.assertEqual(runtime['cwd'], original['cwd'])
                            self.assertEqual(hashlib.sha256(Path(runtime['command'][0]).read_bytes()).hexdigest(),
                                             original_hash)
                            plan = report['runtime_shard']
                            self.assertEqual(plan['full_names'], RUNTIME_SHARD_NAMES)
                            self.assertEqual(runtime['runtime_shard_execution']['names'],
                                             plan['planned_shards'][index])
                            self.assertEqual(runtime['child_status'], 0)
                            decision = ('Runtime integration library collection: ineligible; '
                                        'using --workspace --all-targets')
                            self.assertEqual(result.stdout.count(decision), 1)
                            self.assertLess(result.stdout.index(decision),
                                            result.stdout.index('Queued Cargo test invocation:'))
                            self.assertEqual(result.stdout.count('Queued Cargo test invocation:'), recorded)
                            self.assertNotIn('Runtime integration library collection:', reference.stdout)

    def test_runtime_library_collection_metadata_error_never_retries_collection(self):
        with tempfile.TemporaryDirectory(prefix='spite-ci-runtime-libs-metadata-error-') as temporary:
            root = Path(temporary)
            make_runtime_shard_workspace(root)
            (root / 'Cargo.toml').write_text('[workspace\n', encoding='utf-8')
            for index in [1, 0]:
                result, report = self.run_suite(
                    suite='rest', root=root, extra_arguments=['--runtime-shard', str(index)] +
                        (['--rest-shard', '1'] if index == 0 else []),
                    extra_environment={'CARGO_TARGET_DIR': str(root / 'target')})
                self.assertNotEqual(result.returncode, 0, result.stdout)
                self.assertIsNone(report)
                self.assertNotIn('Runtime integration library collection:', result.stdout)
                self.assertNotIn('Collecting Cargo test invocations', result.stdout)
                self.assertNotIn('Queued Cargo test invocation:', result.stdout)

    def runtime_row(self, report):
        rows = [row for row in report['invocations']
                if row['package'] == 'spite-runtime'
                and re.fullmatch(r'integration-[0-9a-f]{16}(?:\.exe)?',
                                 Path(row['command'][0]).name)]
        self.assertEqual(len(rows), 1, report)
        return rows[0]

    def assert_runtime_shard(self, result, report, index, expected_names, rest_owner=0):
        self.assertEqual(result.returncode, 0, result.stdout)
        self.assertIsNotNone(report, result.stdout)
        self.assertEqual(report['suite'], 'rest')
        self.assertEqual(report['recorded_invocations'], 4)
        self.assertEqual(len(report['invocations']), 3 if index == rest_owner else 1)
        self.assertNotIn('spite-core', {row['package'] for row in report['invocations']})
        plan = report['runtime_shard']
        self.assertEqual(plan['schema'], 1)
        self.assertEqual(plan['algorithm'], 'substring-components-greedy-v1')
        self.assertEqual((plan['index'], plan['count']), (index, 2))
        self.assertEqual(plan['full_names'], sorted(expected_names))
        self.assertEqual(plan['full_count'], len(expected_names))
        self.assertEqual(plan['full_sha256'], names_sha256(expected_names))
        shards = plan['planned_shards']
        self.assertEqual(len(shards), 2)
        self.assertTrue(all(shards))
        self.assertFalse(set(shards[0]) & set(shards[1]))
        self.assertEqual(sorted(shards[0] + shards[1]), sorted(expected_names))
        self.assertEqual(plan['partition_counts'], [len(names) for names in shards])
        self.assertEqual(plan['partition_sha256'], [names_sha256(names) for names in shards])
        row = self.runtime_row(report)
        execution = row['runtime_shard_execution']
        self.assertEqual(execution['names'], shards[index])
        self.assertEqual(execution['executed_name_count'], len(shards[index]))
        self.assertEqual(execution['executed_names_sha256'], names_sha256(shards[index]))
        self.assertEqual(execution['passed'], len(shards[index]))
        self.assertEqual([execution[key] for key in ['failed', 'ignored', 'measured']], [0, 0, 0])
        self.assertEqual(execution['filtered'], len(expected_names) - len(shards[index]))
        self.assertEqual(row['child_status'], 0)
        self.assertEqual(row['status'], 0)
        self.assertNotIn('shard_validation_error', row)
        self.assertEqual(row['summaries'], [['ok', str(len(shards[index])), '0', '0', '0',
                                             str(len(expected_names) - len(shards[index]))]])
        # Read actual pretty libtest verdicts independently of runner metadata.
        verdicts = re.findall(r'^test (\S+) \.\.\. (ok|FAILED|ignored)\s*$',
                              result.stdout, re.MULTILINE)
        actual = [(name, verdict) for name, verdict in verdicts if name in expected_names]
        self.assertEqual(sorted(name for name, _ in actual), shards[index], result.stdout)
        self.assertTrue(all(verdict == 'ok' for _, verdict in actual), result.stdout)
        self.assertEqual(row['command'][1:3],
                         ['--skip', 'pinned_corpus_runs_all_reviewed_variants'])
        for invocation in report['invocations']:
            self.assertFalse(set(invocation) & {
                'environment', 'changed_environment', 'removed_environment'})
        return row

    def test_runtime_shards_preserve_real_coverage_profiles_and_failures(self):
        with tempfile.TemporaryDirectory(prefix='spite-ci-runtime-shard-') as temporary:
            root = Path(temporary)
            make_runtime_shard_workspace(root)
            environment = {
                'CARGO_TARGET_DIR': str(root / 'target'),
                'SPITE_RUNNER_PROBE_FAIL_CASE': '',
            }
            lock_environment = self.environment.copy()
            lock_environment.update(environment)
            subprocess.run(['cargo', '+stable', 'generate-lockfile', '--offline'],
                           cwd=root, env=lock_environment, check=True)
            reports = {}
            for toolchain in ['stable', '1.85.0']:
                for optimize in [False, True]:
                    arguments = ['--optimize-runtime'] if optimize else []
                    environment['SPITE_RUNNER_PROBE_RUNTIME_OPT'] = '1' if optimize else '0'
                    baseline, complete = self.run_suite(
                        suite='rest', root=root, toolchain=toolchain,
                        extra_arguments=arguments, extra_environment=environment)
                    self.assertEqual(baseline.returncode, 0, baseline.stdout)
                    self.assertEqual(complete['recorded_invocations'], 4)
                    self.assertEqual(len(complete['invocations']), 3)
                    self.assertNotIn('runtime_shard', complete)
                    complete_runtime = self.runtime_row(complete)
                    other_rows = [row for row in complete['invocations']
                                  if row is not complete_runtime]
                    self.assertEqual({row['package'] for row in other_rows},
                                     {'spite-runtime', 'runner-probe-c'})
                    for index in [0, 1]:
                        with self.subTest(toolchain=toolchain, optimize=optimize, shard=index):
                            result, report = self.run_suite(
                                suite='rest', root=root, toolchain=toolchain,
                                extra_arguments=arguments + ['--runtime-shard', str(index)],
                                extra_environment=environment)
                            runtime = self.assert_runtime_shard(
                                result, report, index, RUNTIME_SHARD_NAMES)
                            actual_others = [row for row in report['invocations'] if row is not runtime]
                            fields = ['command', 'cwd', 'package', 'status', 'summaries']
                            project = lambda rows: sorted(
                                [{key: row[key] for key in fields} for row in rows],
                                key=lambda row: row['package'])
                            self.assertEqual(project(actual_others),
                                             project(other_rows) if index == 0 else [])
                            self.assertEqual(runtime['cwd'], str(root / 'b'))
                            reports[toolchain, optimize, index] = report
                            selected = report['runtime_shard']['planned_shards'][index]
                            failed, failure_report = self.run_suite(
                                suite='rest', root=root, toolchain=toolchain,
                                extra_arguments=arguments + ['--runtime-shard', str(index)],
                                extra_environment={**environment,
                                                   'SPITE_RUNNER_PROBE_FAIL_CASE': selected[0]})
                            self.assertNotEqual(failed.returncode, 0, failed.stdout)
                            failed_runtime = self.runtime_row(failure_report)
                            self.assertEqual(failed_runtime['child_status'], 101)
                            self.assertNotEqual(failed_runtime['status'], 0)
                            self.assertIn('shard_validation_error', failed_runtime)
                            self.assertTrue(all(row['status'] == 0
                                                for row in failure_report['invocations']
                                                if row is not failed_runtime), failed.stdout)
                    first, second = [reports[toolchain, optimize, index]['runtime_shard']
                                     for index in [0, 1]]
                    for field in ['full_sha256', 'partition_sha256', 'planned_shards']:
                        self.assertEqual(first[field], second[field])

            # New source is compiled and discovered, rather than relying on a fixed inventory.
            source = root / 'b/tests/integration.rs'
            added = '''
mod newly_added {
    #[test]
    fn discovered_after_first_run() { super::check(); }
}
'''
            source.write_text(RUNTIME_SHARD_SOURCE + added, encoding='utf-8')
            expected = sorted(RUNTIME_SHARD_NAMES + ['newly_added::discovered_after_first_run'])
            environment['SPITE_RUNNER_PROBE_RUNTIME_OPT'] = '0'
            for index in [0, 1]:
                result, report = self.run_suite(
                    suite='rest', root=root, toolchain='stable',
                    extra_arguments=['--runtime-shard', str(index)], extra_environment=environment)
                self.assert_runtime_shard(result, report, index, expected)
                self.assertNotEqual(report['runtime_shard']['full_sha256'],
                                    reports['stable', False, index]['runtime_shard']['full_sha256'])

            # libtest exits zero for an ignored test; a shard must still reject omitted execution.
            source.write_text(RUNTIME_SHARD_SOURCE + '''
#[test]
#[ignore]
fn ignored_shard_case() { panic!("must not be credited as executed"); }
''', encoding='utf-8')
            rejected = []
            for index in [0, 1]:
                result, report = self.run_suite(
                    suite='rest', root=root, extra_arguments=['--runtime-shard', str(index)],
                    extra_environment=environment)
                if result.returncode:
                    runtime = self.runtime_row(report)
                    self.assertEqual(runtime['child_status'], 0)
                    self.assertIn('shard_validation_error', runtime)
                    rejected.append(index)
            self.assertEqual(len(rejected), 1)

            # An added name matching the existing global skip must fail the listing proof.
            source.write_text(RUNTIME_SHARD_SOURCE + '''
#[test]
fn new_pinned_corpus_runs_all_reviewed_variants_case() { check(); }
''', encoding='utf-8')
            for index in [0, 1]:
                result, _ = self.run_suite(
                    suite='rest', root=root, extra_arguments=['--runtime-shard', str(index)],
                    extra_environment=environment)
                self.assertNotEqual(result.returncode, 0, result.stdout)


    def test_rest_owner_preserves_paired_coverage_and_moved_failures(self):
        with tempfile.TemporaryDirectory(prefix='spite-ci-rest-owner-') as temporary:
            root = Path(temporary)
            make_runtime_shard_workspace(root)
            (root / 'a/build.rs').unlink()
            core = root / 'a/src/lib.rs'
            core.write_text(core.read_text().replace(PROFILE_SOURCE, '', 1), encoding='utf-8')
            environment = {'CARGO_TARGET_DIR': str(root / 'target'),
                           'SPITE_RUNNER_PROBE_RUNTIME_OPT': '0',
                           'SPITE_RUNNER_PROBE_FAIL_CASE': ''}
            subprocess.run(['cargo', '+stable', 'generate-lockfile', '--offline'],
                           cwd=root, env={**self.environment, **environment}, check=True)
            fields = ['command', 'cwd', 'package', 'status', 'summaries']
            project = lambda rows: sorted(
                [{key: row[key] for key in fields} for row in rows],
                key=lambda row: (row['command'], row['cwd']))
            for toolchain in ['stable', '1.85.0']:
                with self.subTest(toolchain=toolchain):
                    reference, complete = self.run_suite(
                        suite='rest', root=root, toolchain=toolchain,
                        extra_environment=environment)
                    self.assertEqual(reference.returncode, 0, reference.stdout)
                    original = self.runtime_row(complete)
                    original_hash = hashlib.sha256(Path(original['command'][0]).read_bytes()).hexdigest()
                    other_rows = [row for row in complete['invocations'] if row is not original]
                    self.assertEqual({row['package'] for row in other_rows},
                                     {'spite-runtime', 'runner-probe-c'})
                    reports = {}
                    for owner in [0, 1]:
                        combined = []
                        executed = []
                        for index in [0, 1]:
                            arguments = ['--runtime-shard', str(index)]
                            if owner == 1:
                                arguments += ['--rest-shard', '1']
                            result, report = self.run_suite(
                                suite='rest', root=root, toolchain=toolchain,
                                extra_arguments=arguments, extra_environment=environment)
                            runtime = self.assert_runtime_shard(
                                result, report, index, RUNTIME_SHARD_NAMES, rest_owner=owner)
                            self.assertEqual(report.get('rest_shard', 0), owner)
                            self.assertEqual(runtime['command'][0], original['command'][0])
                            self.assertEqual(runtime['cwd'], original['cwd'])
                            self.assertEqual(hashlib.sha256(Path(runtime['command'][0]).read_bytes()).hexdigest(),
                                             original_hash)
                            others = [row for row in report['invocations'] if row is not runtime]
                            self.assertEqual(project(others), project(other_rows) if index == owner else [])
                            combined += others
                            executed += runtime['runtime_shard_execution']['names']
                            reports[owner, index] = report
                        self.assertEqual(project(combined), project(other_rows))
                        self.assertEqual(sorted(executed), RUNTIME_SHARD_NAMES)
                    for index in [0, 1]:
                        first, second = [reports[owner, index] for owner in [0, 1]]
                        self.assertEqual(first['runtime_shard'], second['runtime_shard'])
                        self.assertEqual(project([self.runtime_row(first)]),
                                         project([self.runtime_row(second)]))
                    # Explicit owner zero retains the omitted-option behavior.
                    result, explicit = self.run_suite(
                        suite='rest', root=root, toolchain=toolchain,
                        extra_arguments=['--runtime-shard', '0', '--rest-shard', '0'],
                        extra_environment=environment)
                    self.assertEqual(result.returncode, 0, result.stdout)
                    self.assertEqual(explicit['rest_shard'], 0)
                    self.assertEqual(project(explicit['invocations']),
                                     project(reports[0, 0]['invocations']))
                    # Only the new owner executes a failing nonRuntime package;
                    # its failure must not prevent the Runtime half from running.
                    for index in [0, 1]:
                        result, failed = self.run_suite(
                            fail='runner-probe-c', suite='rest', root=root, toolchain=toolchain,
                            extra_arguments=['--runtime-shard', str(index), '--rest-shard', '1'],
                            extra_environment=environment)
                        self.assertEqual(result.returncode, int(index == 1), result.stdout)
                        runtime = self.runtime_row(failed)
                        self.assertEqual(runtime['child_status'], 0)
                        self.assertEqual(runtime['runtime_shard_execution']['names'],
                                         failed['runtime_shard']['planned_shards'][index])
                        self.assertEqual(sorted(row['status'] for row in failed['invocations']),
                                         [0, 0, 101] if index == 1 else [0])

    def test_rest_owner_requires_an_explicit_runtime_shard_and_rest_suite(self):
        for suite, runtime, owner in [('all', None, 1), ('core', None, 1), ('rest', None, 1),
                                      ('rest', None, 0), ('all', 0, 1), ('core', 1, 1)]:
            with self.subTest(suite=suite, runtime=runtime, owner=owner):
                arguments = ['--rest-shard', str(owner)]
                if runtime is not None:
                    arguments += ['--runtime-shard', str(runtime)]
                result, report = self.run_suite(suite=suite, extra_arguments=arguments)
                self.assertNotEqual(result.returncode, 0, result.stdout)
                self.assertIsNone(report)
                self.assertIn('--rest-shard requires --suite rest and --runtime-shard', result.stdout)
                self.assertNotIn('Collecting Cargo test invocations', result.stdout)
                self.assertNotIn('Queued Cargo test invocation:', result.stdout)
        result, report = self.run_suite(
            suite='rest', extra_arguments=['--runtime-shard', '0', '--rest-shard', '2'])
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIsNone(report)
        self.assertIn('invalid choice', result.stdout)


    def test_full_runtime_and_nonruntime_replay_preserve_context_union_and_failures(self):
        for toolchain in ['stable', '1.85.0']:
            with self.subTest(toolchain=toolchain), tempfile.TemporaryDirectory(
                    prefix='spite-ci-runtime-replay-') as temporary:
                root = Path(temporary) / 'workspace'
                root.mkdir()
                make_integration_only_workspace(root)
                with (root / 'c/src/lib.rs').open('a', encoding='utf-8') as source:
                    source.write('''
#[test]
fn sibling_context_and_failure() {
    assert!(option_env!("CARGO_PRIMARY_PACKAGE").is_some());
    assert_eq!(std::env::current_dir().unwrap(), std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    assert_eq!(std::env::var("CARGO_MANIFEST_DIR").unwrap(), env!("CARGO_MANIFEST_DIR"));
    assert_eq!(std::env::var("SPITE_RUNNER_PROBE_INHERITED").unwrap(), "unchanged");
    assert_ne!(std::env::var("SPITE_RUNNER_PROBE_FAIL").unwrap_or_default(), env!("CARGO_PKG_NAME"));
}
''')
                environment = dict(self.environment, CARGO_TARGET_DIR=str(root / 'target'),
                                   SPITE_RUNNER_PROBE_FAIL_CASE='')
                subprocess.run(['cargo', '+' + toolchain, 'generate-lockfile', '--offline'],
                               cwd=root, env=environment, check=True)
                original, binary_hash = self.integration_collection_context(root, toolchain, environment)
                result, complete = self.run_suite(
                    suite='rest', root=root, toolchain=toolchain, extra_environment=environment)
                self.assertEqual(result.returncode, 0, result.stdout)
                self.assertEqual(complete['recorded_invocations'], 4)
                original_row = self.runtime_row(complete)
                others = [row for row in complete['invocations'] if row is not original_row]
                fields = ['command', 'cwd', 'package', 'status', 'summaries']
                project = lambda rows: sorted(
                    [{key: row[key] for key in fields} for row in rows],
                    key=lambda row: (row['command'], row['cwd']))

                result, full, observations = self.observed_integration_shard(
                    root, toolchain, environment, 0, full=True)
                self.assertIn('Runtime integration library collection: eligible; '
                              'using --workspace --lib --test integration', result.stdout)
                self.assertEqual(full['recorded_invocations'], 4)
                self.assertEqual(len(full['invocations']), 1)
                self.assertEqual(full['runtime_integration'], 'only')
                self.assertNotIn('runtime_shard', full)
                self.assertEqual(full['runtime_full']['full_names'], RUNTIME_SHARD_NAMES)
                self.assertEqual(full['runtime_full']['full_sha256'], names_sha256(RUNTIME_SHARD_NAMES))
                self.assertEqual(full['runtime_full']['listing_counts'], [8, 8])
                runtime = self.runtime_row(full)
                self.assertEqual(runtime['child_status'], 0)
                self.assertEqual(runtime['runtime_full_execution']['names'], RUNTIME_SHARD_NAMES)
                self.assertEqual(runtime['runtime_full_execution']['filtered'], 0)
                self.assertEqual(project([runtime]), project([original_row]))
                self.assertEqual(len(observations), 2)
                for observed in observations:
                    self.assertEqual(observed, original)
                self.assertEqual(hashlib.sha256(Path(runtime['command'][0]).read_bytes()).hexdigest(),
                                 binary_hash)

                result, excluded = self.run_suite(
                    suite='rest', root=root, toolchain=toolchain,
                    extra_arguments=['--runtime-integration', 'exclude'], extra_environment=environment)
                self.assertEqual(result.returncode, 0, result.stdout)
                self.assertEqual(excluded['recorded_invocations'], 4)
                self.assertEqual(excluded['runtime_integration'], 'exclude')
                self.assertEqual(project(excluded['invocations']), project(others))
                self.assertEqual(project(excluded['invocations'] + full['invocations']),
                                 project(complete['invocations']))
                self.assertTrue(all(row['package'] != 'spite-core' for row in excluded['invocations']))
                for mode in ['only', 'exclude']:
                    result, failed = self.run_suite(
                        fail='runner-probe-c', suite='rest', root=root, toolchain=toolchain,
                        extra_arguments=['--runtime-integration', mode], extra_environment=environment)
                    self.assertEqual(result.returncode, int(mode == 'exclude'), result.stdout)
                    self.assertEqual(sorted(row['status'] for row in failed['invocations']),
                                     [0, 101] if mode == 'exclude' else [0])
                result, failed = self.run_suite(
                    suite='rest', root=root, toolchain=toolchain,
                    extra_arguments=['--runtime-integration', 'only'],
                    extra_environment={**environment, 'SPITE_RUNNER_PROBE_FAIL_CASE': 'root_case'})
                self.assertEqual(result.returncode, 1, result.stdout)
                runtime = self.runtime_row(failed)
                self.assertEqual(runtime['child_status'], 101)
                self.assertIn('full_validation_error', runtime)

                # Keep the same guarded library and all-target fallbacks for full replay.
                (root / 'c/build.rs').write_text('fn main() {}\n', encoding='utf-8')
                result, fallback, observations = self.observed_integration_shard(
                    root, toolchain, environment, 0, full=True)
                self.assertIn('using --workspace --lib --test integration', result.stdout)
                self.assertEqual(fallback['recorded_invocations'], 4)
                self.assertEqual(len(fallback['invocations']), 1)
                self.assertEqual(self.runtime_row(fallback)['runtime_full_execution']['names'],
                                 RUNTIME_SHARD_NAMES)
                self.assertTrue(all(observed == original for observed in observations))
                self.assertEqual(hashlib.sha256(Path(original['command'][0]).read_bytes()).hexdigest(),
                                 binary_hash)
                (root / 'a/tests').mkdir()
                (root / 'a/tests/extra.rs').write_text(
                    '#[test]\nfn core_must_not_run_in_rest() { panic!("Core replayed"); }\n', encoding='utf-8')
                result, fallback, observations = self.observed_integration_shard(
                    root, toolchain, environment, 0, full=True)
                self.assertIn('ineligible; using --workspace --all-targets', result.stdout)
                self.assertEqual(fallback['recorded_invocations'], 5)
                self.assertEqual(len(fallback['invocations']), 1)
                self.assertEqual(self.runtime_row(fallback)['runtime_full_execution']['names'],
                                 RUNTIME_SHARD_NAMES)
                self.assertTrue(all(observed == original for observed in observations))

    def test_full_runtime_replay_discovers_names_and_rejects_ignored_or_globally_skipped_cases(self):
        for toolchain in ['stable', '1.85.0']:
            with self.subTest(toolchain=toolchain), tempfile.TemporaryDirectory(
                    prefix='spite-ci-runtime-full-inventory-') as temporary:
                root = Path(temporary) / 'workspace'
                root.mkdir()
                make_integration_only_workspace(root)
                environment = {'CARGO_TARGET_DIR': str(root / 'target'),
                               'SPITE_RUNNER_PROBE_FAIL_CASE': ''}
                subprocess.run(['cargo', '+' + toolchain, 'generate-lockfile', '--offline'],
                               cwd=root, env={**self.environment, **environment}, check=True)
                source = root / 'b/tests/integration.rs'
                original = source.read_text(encoding='utf-8')
                arguments = ['--runtime-integration', 'only']
                source.write_text(original + '\n#[test]\nfn newly_discovered() { check(); }\n', encoding='utf-8')
                result, discovered = self.run_suite(
                    suite='rest', root=root, toolchain=toolchain,
                    extra_arguments=arguments, extra_environment=environment)
                self.assertEqual(result.returncode, 0, result.stdout)
                expected = sorted(RUNTIME_SHARD_NAMES + ['newly_discovered'])
                self.assertEqual(discovered['runtime_full']['full_names'], expected)
                self.assertEqual(self.runtime_row(discovered)['runtime_full_execution']['names'], expected)
                source.write_text(original + '\n#[test]\n#[ignore]\nfn ignored_full_case() {}\n', encoding='utf-8')
                result, ignored = self.run_suite(
                    suite='rest', root=root, toolchain=toolchain,
                    extra_arguments=arguments, extra_environment=environment)
                self.assertEqual(result.returncode, 1, result.stdout)
                runtime = self.runtime_row(ignored)
                self.assertEqual(runtime['child_status'], 0)
                self.assertIn('full_validation_error', runtime)
                source.write_text(original + '\n#[test]\nfn new_pinned_corpus_runs_all_reviewed_variants_case() { check(); }\n',
                                  encoding='utf-8')
                result, missing = self.run_suite(
                    suite='rest', root=root, toolchain=toolchain,
                    extra_arguments=arguments, extra_environment=environment)
                self.assertNotEqual(result.returncode, 0, result.stdout)
                self.assertIsNone(missing)
                self.assertIn('original Runtime arguments filter the complete inventory', result.stdout)
                self.assertNotIn('Cargo invocation elapsed:', result.stdout)

    def test_runtime_replay_modes_require_rest_without_shard_options(self):
        for mode in ['only', 'exclude']:
            for suite, extra in [('all', []), ('core', []),
                                 ('rest', ['--runtime-shard', '0']),
                                 ('rest', ['--rest-shard', '1'])]:
                with self.subTest(mode=mode, suite=suite, extra=extra):
                    result, report = self.run_suite(
                        suite=suite, extra_arguments=['--runtime-integration', mode, *extra])
                    self.assertNotEqual(result.returncode, 0, result.stdout)
                    self.assertIsNone(report)
                    self.assertIn('--runtime-integration requires --suite rest without Runtime shard options', result.stdout)
                    self.assertNotIn('Collecting Cargo test invocations', result.stdout)
            result, report = self.run_suite(
                suite='rest', extra_arguments=['--runtime-integration', mode, '--collect-core-libs'])
            self.assertNotEqual(result.returncode, 0, result.stdout)
            self.assertIsNone(report)
            self.assertNotIn('Collecting Cargo test invocations', result.stdout)
        with tempfile.TemporaryDirectory(prefix='spite-ci-runtime-full-metadata-error-') as temporary:
            root = Path(temporary)
            (root / 'Cargo.toml').write_text('[workspace\n', encoding='utf-8')
            result, report = self.run_suite(
                suite='rest', root=root, extra_arguments=['--runtime-integration', 'only'])
            self.assertNotEqual(result.returncode, 0, result.stdout)
            self.assertIsNone(report)
            self.assertNotIn('Collecting Cargo test invocations', result.stdout)
            self.assertNotIn('Queued Cargo test invocation:', result.stdout)


class RuntimeShardHelperTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        specification = importlib.util.spec_from_file_location('workspace_shard_runner', RUNNER)
        cls.runner = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(cls.runner)

    def test_listing_requires_complete_unique_test_inventory(self):
        listing = 'arrays::one: test\nroot_case: test\n\n2 tests, 0 benchmarks\n'
        self.assertEqual(self.runner.parse_test_listing(listing), ['arrays::one', 'root_case'])
        self.assertEqual(self.runner.parse_test_listing('root_case: test\n\n1 test, 0 benchmarks\n'),
                         ['root_case'])
        for malformed in [
                '', 'root_case: test\n', 'root_case: test\n\n2 tests, 0 benchmarks\n',
                'root_case: test\nroot_case: test\n\n2 tests, 0 benchmarks\n',
                'root_case: benchmark\n\n0 tests, 1 benchmark\n',
                listing + 'unexpected output\n', listing.replace(': test', ': unknown', 1)]:
            with self.subTest(output=malformed), self.assertRaises(RuntimeError):
                self.runner.parse_test_listing(malformed)

    def test_partition_proves_substring_collisions_and_determinism(self):
        names = RUNTIME_SHARD_NAMES
        plan = self.runner.runtime_partition(names)
        for discovery in [list(reversed(names)), names[3:] + names[:3]]:
            self.assertEqual(self.runner.runtime_partition(discovery), plan)
        self.assertEqual(plan['names'], names)
        self.assertTrue(all(plan['shards']))
        self.assertFalse(set(plan['shards'][0]) & set(plan['shards'][1]))
        self.assertEqual(sorted(plan['shards'][0] + plan['shards'][1]), names)
        collided = {'arrays::fixture_environment', 'typed_arrays::fixture_environment',
                    'nested::arrays::fixture_environment', 'nested::root_case', 'root_case'}
        self.assertEqual(sum(collided <= set(shard) for shard in plan['shards']), 1)
        for index in [0, 1]:
            selected = [name for name in names
                        if not any(prefix in name for prefix in plan['skips'][index])]
            self.assertEqual(selected, plan['shards'][index])
        changed = self.runner.runtime_partition(names + ['new_module::new_case'])
        self.assertEqual(sorted(changed['shards'][0] + changed['shards'][1]),
                         sorted(names + ['new_module::new_case']))
        for unsplittable in [[], ['single_root'], ['arrays::one', 'typed_arrays::two']]:
            with self.subTest(names=unsplittable), self.assertRaises(RuntimeError):
                self.runner.runtime_partition(unsplittable)

    def test_runtime_invocation_requires_exact_package_target_and_arguments(self):
        def invocation(target='integration-0123456789abcdef', package='spite-runtime', tail=None):
            return {'package': package, 'cwd': '/fixture/b',
                    'command': ['/fixture/target/' + target] + (
                        ['--skip', 'pinned_corpus_runs_all_reviewed_variants']
                        if tail is None else tail),
                    'changed_environment': {}, 'removed_environment': []}
        valid = invocation()
        self.assertEqual(self.runner.validate_runtime_invocation([valid]), valid)
        windows = invocation('integration-0123456789abcdef.exe')
        self.assertEqual(self.runner.validate_runtime_invocation([windows]), windows)
        for rows in [[], [valid, dict(valid)], [invocation(package='other')],
                     [invocation('integration-0123456789abcde')],
                     [invocation('integration-0123456789abcdef-extra')],
                     [invocation('integration_helpers-0123456789abcdef')],
                     [invocation(tail=[])], [invocation(tail=['--ignored'])],
                     [invocation(tail=['--skip', 'pinned_corpus_runs_all_reviewed_variants', 'filter'])]]:
            with self.subTest(rows=rows), self.assertRaises(RuntimeError):
                self.runner.validate_runtime_invocation(rows)

    def test_runtime_execution_rejects_missing_duplicate_ignored_or_failed_cases(self):
        expected = ['arrays::one', 'root_case']
        output = ('running 2 tests\ntest arrays::one ... ok\ntest root_case ... ok\n\n'
                  'test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; '
                  '4 filtered out; finished in 0.00s\n')
        result = self.runner.runtime_execution(output, expected, 6)
        self.assertEqual(result['names'], expected)
        self.assertEqual(result['executed_name_count'], 2)
        self.assertEqual(result['executed_names_sha256'], names_sha256(expected))
        for malformed in [
                output.replace('running 2 tests', 'running 3 tests'),
                output.replace('test root_case ... ok\n', ''),
                output.replace('test root_case ... ok', 'test arrays::one ... ok'),
                output.replace('test root_case ... ok', 'test unexpected ... ok'),
                output.replace('test root_case ... ok', 'test root_case ... ignored'),
                output.replace('test root_case ... ok', 'test root_case ... FAILED'),
                output.replace('\n\ntest result:',
                               '\ntest unexpected ... unknown-verdict\n\ntest result:'),
                output.replace('0 ignored', '1 ignored'),
                output.replace('0 measured', '1 measured'),
                output.replace('4 filtered out', '3 filtered out'),
                output + output,
                output.replace('test result: ok.', 'test result: FAILED.')]:
            with self.subTest(output=malformed), self.assertRaises(RuntimeError):
                self.runner.runtime_execution(malformed, expected, 6)

    def test_shard_preparation_rejects_failed_or_incomplete_actual_listings(self):
        invocation = {
            'package': 'spite-runtime', 'cwd': '/fixture/b',
            'command': ['/fixture/integration-0123456789abcdef',
                        '--skip', 'pinned_corpus_runs_all_reviewed_variants'],
            'changed_environment': {'changed': 'after'},
            'removed_environment': ['removed'],
        }
        baseline = {'kept': 'same', 'changed': 'before', 'removed': 'gone'}
        full = 'first::one: test\nsecond::two: test\n\n2 tests, 0 benchmarks\n'
        first = 'first::one: test\n\n1 test, 0 benchmarks\n'
        second = 'second::two: test\n\n1 test, 0 benchmarks\n'
        empty = '0 tests, 0 benchmarks\n'
        cases = {
            'failed initial listing': [(101, full)],
            'malformed initial listing': [(0, 'unrecognized listing\n')],
            'original arguments omit a name': [(0, full), (0, first)],
            'filtered listings overlap': [(0, full), (0, full), (0, first), (0, first)],
            'first filtered selection empty': [(0, full), (0, full), (0, empty), (0, second)],
            'second filtered selection missing': [(0, full), (0, full), (0, first), (0, empty)],
            'filtered listing missing entirely': [(0, full), (0, full), (0, first), (0, '')],
        }
        for index in [0, 1]:
            for label, responses in cases.items():
                with self.subTest(shard=index, case=label):
                    remaining = iter(responses)

                    def listed(command, **kwargs):
                        self.assertEqual(command[-1], '--list',
                                         'No native execution may precede a complete listing proof')
                        self.assertEqual(kwargs['cwd'], '/fixture/b')
                        self.assertEqual(kwargs['env'], {'kept': 'same', 'changed': 'after'})
                        status, output = next(remaining)
                        return subprocess.CompletedProcess(command, status, stdout=output)

                    with mock.patch.object(self.runner, 'run_command', side_effect=listed) as child, \
                         mock.patch('builtins.print'), self.assertRaises(RuntimeError):
                        self.runner.prepare_runtime_shard([invocation], index, baseline)
                    self.assertEqual(child.call_count, len(responses))

    def test_full_preparation_preserves_original_arguments_and_requires_complete_listings(self):
        invocation = {
            'package': 'spite-runtime', 'cwd': '/fixture/b',
            'command': ['/fixture/integration-0123456789abcdef',
                        '--skip', 'pinned_corpus_runs_all_reviewed_variants'],
            'changed_environment': {'changed': 'after'}, 'removed_environment': ['removed'],
        }
        baseline = {'kept': 'same', 'changed': 'before', 'removed': 'gone'}
        full = 'arrays::one: test\ntyped_arrays::two: test\n\n2 tests, 0 benchmarks\n'
        # One collision component cannot be split, but needs no partition for full replay.
        with mock.patch.object(self.runner, 'run_command', return_value=
                subprocess.CompletedProcess([], 0, stdout=full)) as child, mock.patch('builtins.print'):
            original, metadata, names = self.runner.prepare_runtime_full([invocation], baseline)
        self.assertIs(original, invocation)
        self.assertEqual(names, ['arrays::one', 'typed_arrays::two'])
        self.assertEqual(metadata['full_count'], 2)
        self.assertEqual(metadata['full_sha256'], names_sha256(names))
        self.assertEqual(metadata['listing_counts'], [2, 2])
        self.assertEqual([call.args[0] for call in child.call_args_list],
                         [[invocation['command'][0], '--list'], invocation['command'] + ['--list']])
        for call in child.call_args_list:
            self.assertEqual(call.kwargs['cwd'], '/fixture/b')
            self.assertEqual(call.kwargs['env'], {'kept': 'same', 'changed': 'after'})
        cases = [[(101, full)], [(0, 'unrecognized listing\n')],
                 [(0, full), (101, full)], [(0, full), (0, '')],
                 [(0, full), (0, 'arrays::one: test\n\n1 test, 0 benchmarks\n')]]
        for responses in cases:
            with self.subTest(responses=responses):
                remaining = iter(responses)

                def listed(command, **kwargs):
                    self.assertEqual(command[-1], '--list', 'No Native replay before both lists pass')
                    status, output = next(remaining)
                    return subprocess.CompletedProcess(command, status, stdout=output)

                with mock.patch.object(self.runner, 'run_command', side_effect=listed) as child, \
                     mock.patch('builtins.print'), self.assertRaises(RuntimeError):
                    self.runner.prepare_runtime_full([invocation], baseline)
                self.assertEqual(child.call_count, len(responses))

    def test_windows_command_guard_uses_full_quoted_utf16_boundary(self):
        command = ['C:\\Program Files\\spite\\integration-0123456789abcdef.exe',
                   'space here', 'trailing space\\', 'quote"inside', '\U0001f600',
                   '--skip', 'pinned_corpus_runs_all_reviewed_variants']
        expected = len(subprocess.list2cmdline(command).encode('utf-16-le')) // 2 + 1
        self.assertEqual(self.runner.guard_windows_command(command, windows=True), expected)
        self.assertIsNone(self.runner.guard_windows_command(command, windows=False))
        self.assertEqual(self.runner.guard_windows_command(['x', 'a' * 32764], windows=True),
                         32767)
        with self.assertRaises(RuntimeError):
            self.runner.guard_windows_command(['x', 'a' * 32765], windows=True)
        self.assertEqual(self.runner.guard_windows_command(['x' * 32766], windows=True),
                         32767)
        with self.assertRaises(RuntimeError):
            self.runner.guard_windows_command(['x' * 32767], windows=True)
        # An astral character consumes two UTF-16 units, despite one Python code point.
        with self.assertRaises(RuntimeError):
            self.runner.guard_windows_command(['x', 'a' * 32763 + '\U0001f600'], windows=True)

    def test_runtime_shard_cli_requires_rest_suite(self):
        for suite in ['all', 'core']:
            with self.subTest(suite=suite):
                result = subprocess.run(
                    [sys.executable, str(RUNNER), '--toolchain', 'stable', '--suite', suite,
                     '--runtime-shard', '0'], stdout=subprocess.PIPE,
                    stderr=subprocess.STDOUT, text=True)
                self.assertNotEqual(result.returncode, 0, result.stdout)
                self.assertIn('--suite rest', result.stdout)


if __name__ == '__main__':
    unittest.main()
