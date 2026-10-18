#!/usr/bin/env python3
"""Run Cargo's unchanged workspace invocations, optionally split by package."""

import argparse
from collections import Counter
import concurrent.futures
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time
import uuid


CORPUS_SKIP = ['--skip', 'pinned_corpus_runs_all_reviewed_variants']


def guard_windows_command(command, windows=None):
    """Check CreateProcessW's bound using the actual quoted command line."""
    if windows is None:
        windows = os.name == 'nt'
    if windows:
        units = len(subprocess.list2cmdline(command).encode('utf-16-le')) // 2 + 1
        if units > 32767:
            raise RuntimeError(f'Windows command line exceeds 32767 UTF-16 units: {units}')
        return units
    return None


def run_command(command, **kwargs):
    guard_windows_command(command)
    return subprocess.run(command, shell=False, **kwargs)


def names_hash(names):
    encoded = json.dumps(sorted(names), ensure_ascii=True, separators=(',', ':'))
    return hashlib.sha256(encoded.encode('utf-8')).hexdigest()


def parse_test_listing(output):
    """Accept only a complete ordinary libtest --list inventory."""
    lines = output.splitlines()
    while lines and not lines[-1]:
        lines.pop()
    if not lines:
        raise RuntimeError('Runtime test listing is empty')
    summary = re.fullmatch(r'(\d+) tests?, 0 benchmarks?', lines.pop())
    if not summary:
        raise RuntimeError('Runtime test listing has no complete test-only summary')
    names = []
    for line in lines:
        if not line:
            continue
        match = re.fullmatch(r'(.+): test', line)
        if not match:
            raise RuntimeError('Unexpected Runtime test listing line: ' + repr(line))
        names.append(match[1])
    if len(names) != int(summary[1]) or len(names) != len(set(names)):
        raise RuntimeError('Runtime test listing has duplicate names or a wrong count')
    return sorted(names)


def runtime_partition(names, count=2):
    """Keep every substring-colliding module prefix in the same shard."""
    if type(count) is not int or count not in (2, 3):
        raise RuntimeError('Runtime partition count must be two or three')
    names = sorted(names)
    if not names or len(names) != len(set(names)):
        raise RuntimeError('Runtime inventory must contain unique test names')
    owners = {name: name.split('::', 1)[0] + '::' if '::' in name else name
              for name in names}
    prefixes = sorted(set(owners.values()))
    parent = {prefix: prefix for prefix in prefixes}

    def find(prefix):
        while parent[prefix] != prefix:
            parent[prefix] = parent[parent[prefix]]
            prefix = parent[prefix]
        return prefix

    for name in names:
        for prefix in prefixes:
            if prefix in name:
                left, right = find(owners[name]), find(prefix)
                if left != right:
                    parent[max(left, right)] = min(left, right)
    groups = {}
    for prefix in prefixes:
        groups.setdefault(find(prefix), []).append(prefix)
    components = []
    for filters in groups.values():
        members = [name for name in names if owners[name] in filters]
        components.append((filters, members))
    components.sort(key=lambda row: (-len(row[1]), row[0][0]))
    shards = [[] for _ in range(count)]
    filters = [[] for _ in range(count)]
    for component_filters, members in components:
        index = min(range(count), key=lambda i: (len(shards[i]), i))
        shards[index].extend(members)
        filters[index].extend(component_filters)
    shards = [sorted(shard) for shard in shards]
    if not all(shards):
        required = 'two' if count == 2 else 'three'
        raise RuntimeError(f'Runtime partition requires {required} nonempty collision components')
    skips = [sorted(prefix for other, prefixes in enumerate(filters) if other != index
                    for prefix in prefixes) for index in range(count)]
    for index in range(count):
        selected = [name for name in names
                    if not any(prefix in name for prefix in skips[index])]
        if selected != shards[index]:
            raise RuntimeError('Runtime complement filters do not match the partition')
    return {'names': names, 'prefixes': prefixes,
            'components': [row[0] for row in components],
            'shards': shards, 'skips': skips}


def validate_runtime_invocation(invocations):
    matches = [row for row in invocations if row['package'] == 'spite-runtime'
               and re.fullmatch(r'integration-[0-9a-f]{16}(?:\.exe)?',
                                Path(row['command'][0]).name)]
    if len(matches) != 1:
        raise RuntimeError('Expected exactly one hashed Runtime integration invocation')
    if matches[0]['command'][1:] != CORPUS_SKIP:
        raise RuntimeError('Unexpected Runtime integration arguments')
    return matches[0]


def invocation_environment(invocation, baseline):
    environment = baseline.copy()
    environment.update(invocation['changed_environment'])
    for key in invocation['removed_environment']:
        environment.pop(key, None)
    return environment


def prepare_runtime_shard(invocations, index, baseline, count=2):
    if type(count) is not int or count not in (2, 3):
        raise RuntimeError('Runtime partition count must be two or three')
    if type(index) is not int or not 0 <= index < count:
        raise RuntimeError('Runtime shard index must be less than its partition count')
    invocation = validate_runtime_invocation(invocations)
    environment = invocation_environment(invocation, baseline)
    executable = invocation['command'][0]
    listings = []

    def listing(command):
        result = run_command(command, cwd=invocation['cwd'], env=environment,
                             stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                             text=True, encoding='utf-8', errors='replace')
        listings.append((command, result.stdout))
        try:
            if result.returncode:
                raise RuntimeError(f'Runtime listing exited with status {result.returncode}')
            return parse_test_listing(result.stdout)
        except RuntimeError:
            print(result.stdout, flush=True)
            raise

    def reject(message):
        for command, output in listings:
            print('Runtime listing command: ' + json.dumps(command), flush=True)
            print(output, flush=True)
        raise RuntimeError(message)

    names = listing([executable, '--list'])
    if listing(invocation['command'] + ['--list']) != names:
        reject('The original Runtime arguments filter the complete inventory')
    try:
        plan = runtime_partition(names, count)
    except RuntimeError as error:
        reject(str(error))
    commands = [invocation['command'] +
                [argument for prefix in skips for argument in ['--skip', prefix]]
                for skips in plan['skips']]
    observed = [listing(command + ['--list']) for command in commands]
    if observed != plan['shards'] or any(
            set(observed[left]) & set(observed[right])
            for left in range(count) for right in range(left + 1, count)) \
            or Counter(name for shard in observed for name in shard) != Counter(names):
        reject('Actual Runtime shard listings are not a disjoint complete partition')
    metadata = {'schema': 1, 'algorithm': 'substring-components-greedy-v1',
                'index': index, 'count': count, 'full_count': len(names),
                'full_sha256': names_hash(names),
                'partition_counts': [len(shard) for shard in observed],
                'partition_sha256': [names_hash(shard) for shard in observed],
                'prefix_count': len(plan['prefixes']),
                'component_count': len(plan['components']),
                'listing_counts': [len(names), len(names), *map(len, observed)],
                'listing_output_sha256': [hashlib.sha256(output.encode('utf-8')).hexdigest()
                                          for _, output in listings],
                'listing_command_utf16_units': [guard_windows_command(command)
                                                for command, _ in listings],
                'native_command_utf16_units': [guard_windows_command(command)
                                               for command in commands]}
    print('Runtime shard plan: ' + json.dumps(metadata, sort_keys=True), flush=True)
    selected = dict(invocation, command=commands[index])
    return invocation, selected, metadata, plan


def prepare_runtime_full(invocations, baseline):
    """Validate the complete original Runtime invocation without partition filters."""
    invocation = validate_runtime_invocation(invocations)
    environment = invocation_environment(invocation, baseline)
    commands = [[invocation['command'][0], '--list'], invocation['command'] + ['--list']]
    listings, inventories = [], []
    for command in commands:
        result = run_command(command, cwd=invocation['cwd'], env=environment,
                             stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                             text=True, encoding='utf-8', errors='replace')
        listings.append(result.stdout)
        try:
            if result.returncode:
                raise RuntimeError(f'Runtime listing exited with status {result.returncode}')
            inventories.append(parse_test_listing(result.stdout))
        except RuntimeError:
            for output in listings:
                print(output, flush=True)
            raise
    if inventories[0] != inventories[1]:
        for output in listings:
            print(output, flush=True)
        raise RuntimeError('The original Runtime arguments filter the complete inventory')
    names = inventories[0]
    metadata = {'schema': 1, 'algorithm': 'original-integration-full-v1',
                'full_count': len(names), 'full_sha256': names_hash(names),
                'listing_counts': list(map(len, inventories)),
                'listing_output_sha256': [hashlib.sha256(output.encode('utf-8')).hexdigest()
                                          for output in listings],
                'listing_command_utf16_units': [guard_windows_command(command)
                                                for command in commands],
                'native_command_utf16_units': guard_windows_command(invocation['command'])}
    print('Runtime full plan: ' + json.dumps(metadata, sort_keys=True), flush=True)
    return invocation, metadata, names


def runtime_execution(output, expected_names, full_count):
    rows = re.findall(r'^test (.+) \.\.\. ([^\r\n]+)$',
                      output, re.MULTILINE)
    summaries = re.findall(
        r'^test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; '
        r'(\d+) measured; (\d+) filtered out(?:; finished in [^\r\n]+)?$',
        output, re.MULTILINE)
    running = re.findall(r'^running (\d+) tests?$', output, re.MULTILINE)
    names = [name for name, _ in rows]
    expected = ['ok', str(len(expected_names)), '0', '0', '0',
                str(full_count - len(expected_names))]
    if Counter(names) != Counter(expected_names) or any(verdict != 'ok' for _, verdict in rows) \
            or summaries != [tuple(expected)] or running != [str(len(expected_names))]:
        raise RuntimeError('Runtime execution names or verdicts differ from the selected inventory')
    return {'names': sorted(names), 'executed_name_count': len(names),
            'executed_names_sha256': names_hash(names),
            'passed': len(names), 'failed': 0, 'ignored': 0, 'measured': 0,
            'filtered': full_count - len(names)}


def fingerprint(value):
    return hashlib.sha256(value.encode('utf-8', 'surrogateescape')).hexdigest()


def record(directory, command):
    directory = Path(directory)
    baseline = json.loads((directory / 'environment-fingerprints.json').read_text())
    changed = {key: value for key, value in os.environ.items()
               if baseline.get(key) != fingerprint(value)}
    removed = [key for key in baseline if key not in os.environ]
    package = os.environ.get('CARGO_PKG_NAME')
    if not package:
        raise RuntimeError('Cargo did not provide the test package name')
    row = {'command': command, 'cwd': str(Path.cwd()), 'package': package,
           'changed_environment': changed, 'removed_environment': removed}
    path = directory / f'invocation-{time.monotonic_ns()}-{uuid.uuid4().hex}.json'
    path.write_text(json.dumps(row), encoding='utf-8')
    print('Queued Cargo test invocation: ' + Path(command[0]).name, flush=True)


def can_collect_core_libs(metadata):
    """Keep every workspace library test root and every Core test target."""
    members = metadata['workspace_members']
    packages = {package['id']: package for package in metadata['packages']}
    if not members or len(set(members)) != len(members):
        raise RuntimeError('Cargo metadata has no unique workspace members')
    core = []
    for member in members:
        package = packages[member]
        targets = package['targets']
        libraries = [target for target in targets
                     if target['kind'] == ['lib'] and target['crate_types'] == ['lib']
                     and target['test'] is True and not target.get('required-features', [])]
        if len(libraries) != 1:
            return False
        if package['name'] == 'spite-core':
            if len(targets) != 1:
                return False
            core.append(package)
    return len(core) == 1


def can_collect_runtime_integration_libs(metadata):
    """Retain all workspace dev roots and one unambiguous Runtime test target."""
    if not can_collect_core_libs(metadata):
        return False
    members = set(metadata['workspace_members'])
    matches = [(package, target) for package in metadata['packages']
               if package['id'] in members for target in package['targets']
               if target['name'] == 'integration']
    if len(matches) != 1:
        return False
    package, target = matches[0]
    return (package['name'] == 'spite-runtime' and target['kind'] == ['test']
            and target['crate_types'] == ['bin'] and target['test'] is True
            and not target.get('required-features', []))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--toolchain', choices=['stable', '1.85.0'], required=True)
    parser.add_argument('--offline', action='store_true')
    parser.add_argument('--suite', choices=['all', 'core', 'rest'], default='all',
                        help='Select recorded packages after compiling the complete workspace')
    parser.add_argument('--collect-core-libs', action='store_true',
                        help='Collect workspace library tests when they cover the complete Core suite')
    parser.add_argument('--optimize-runtime', action='store_true',
                        help='Compile spite-runtime at opt-level 1 for this run')
    parser.add_argument('--runtime-shard', type=int, choices=[0, 1, 2],
                        help='Opt into one complete Runtime integration partition')
    parser.add_argument('--runtime-shard-count', type=int, choices=[2, 3],
                        help='Select two or three Runtime partitions (default: 2)')
    parser.add_argument('--rest-shard', type=int, choices=[0, 1],
                        help='Select the Runtime shard owning other REST tests (default: 0)')
    parser.add_argument('--runtime-integration', choices=['only', 'exclude'],
                        help='Replay the complete Runtime integration invocation or other REST invocations')
    parser.add_argument('--report', type=Path)
    args = parser.parse_args()
    if args.runtime_integration is not None and (args.suite != 'rest' or
            args.runtime_shard is not None or args.rest_shard is not None or
            args.runtime_shard_count is not None):
        parser.error('--runtime-integration requires --suite rest without Runtime shard options')
    if args.rest_shard is not None and (args.suite != 'rest' or args.runtime_shard is None):
        parser.error('--rest-shard requires --suite rest and --runtime-shard')
    if args.runtime_shard is not None and args.suite != 'rest':
        parser.error('--runtime-shard requires --suite rest')
    if args.runtime_shard_count is not None and (
            args.suite != 'rest' or args.runtime_shard is None):
        parser.error('--runtime-shard-count requires --suite rest and --runtime-shard')
    shard_count = 2 if args.runtime_shard_count is None else args.runtime_shard_count
    if args.runtime_shard is not None and args.runtime_shard >= shard_count:
        parser.error('--runtime-shard must be less than --runtime-shard-count (default: 2)')
    if args.collect_core_libs and args.suite != 'core':
        parser.error('--collect-core-libs requires --suite core')
    rest_shard = 0 if args.rest_shard is None else args.rest_shard
    runtime_only = args.runtime_integration == 'only' or (
        args.runtime_shard is not None and args.runtime_shard != rest_shard)
    if args.rest_shard is not None:
        print(f'NonRuntime REST owner: runtime shard {rest_shard}', flush=True)
    version_command = ['rustc', '+' + args.toolchain, '-vV']
    guard_windows_command(version_command)
    version = subprocess.check_output(version_command, text=True, shell=False)
    host = re.search(r'^host: ([A-Za-z0-9_-]+)$', version, re.MULTILINE)
    if not host:
        raise RuntimeError('Rust did not report a host target')
    started = time.monotonic()
    baseline = dict(os.environ)
    # Private temporary records contain only Cargo's environment changes. Reports
    # never include those values; records are removed on success and failure.
    with tempfile.TemporaryDirectory(prefix='spite-ci-tests-') as temporary:
        directory = Path(temporary)
        (directory / 'environment-fingerprints.json').write_text(
            json.dumps({key: fingerprint(value) for key, value in baseline.items()}),
            encoding='utf-8')
        runner = [sys.executable, str(Path(__file__).resolve()), '--record', temporary]
        configuration = 'target.' + host[1] + '.runner=' + json.dumps(runner)
        target_selection = ['--all-targets']
        if args.collect_core_libs or runtime_only:
            metadata_command = ['cargo', '+' + args.toolchain, 'metadata', '--no-deps',
                                '--format-version', '1', '--locked']
            if args.offline:
                metadata_command.append('--offline')
            metadata = run_command(metadata_command, stdout=subprocess.PIPE,
                                   text=True, encoding='utf-8')
            if metadata.returncode:
                return metadata.returncode
            metadata = json.loads(metadata.stdout)
            if args.collect_core_libs:
                eligible = can_collect_core_libs(metadata)
                if eligible:
                    target_selection = ['--lib']
                print('Core library collection: ' +
                      ('eligible; using --workspace --lib' if eligible else
                       'ineligible; using --workspace --all-targets'), flush=True)
            else:
                eligible = can_collect_runtime_integration_libs(metadata)
                if eligible:
                    # Keep library roots even when replaying only Runtime:
                    # --test alone changes CARGO_PRIMARY_PACKAGE in dependencies.
                    target_selection = ['--lib', '--test', 'integration']
                print('Runtime integration library collection: ' +
                      ('eligible; using --workspace --lib --test integration' if eligible else
                       'ineligible; using --workspace --all-targets'), flush=True)
        command = ['cargo', '+' + args.toolchain, 'test', '--workspace', *target_selection,
                   '--locked', '--config', configuration]
        if args.optimize_runtime:
            command += ['--config', 'profile.test.package.spite-runtime.opt-level=1']
        if args.offline:
            command.append('--offline')
        command += ['--', '--skip', 'pinned_corpus_runs_all_reviewed_variants']
        print('Collecting Cargo test invocations with their original arguments and environment', flush=True)
        status = run_command(command).returncode
        if status:
            return status
        invocations = [json.loads(path.read_text(encoding='utf-8')) for path in
                       sorted(directory.glob('invocation-*.json'))]
        if not invocations:
            raise RuntimeError('Cargo recorded no test invocations; check the host runner configuration')
        # All collection paths keep workspace package roots. Package suites
        # filter only the recorded invocations.
        if args.suite != 'all':
            if not any(row['package'] == 'spite-core' for row in invocations):
                raise RuntimeError('The split workspace suites require a spite-core package')
            selected = [row for row in invocations
                        if (row['package'] == 'spite-core') == (args.suite == 'core')]
        else:
            selected = invocations
        if not selected:
            raise RuntimeError('The selected workspace suite recorded no test invocations')
        runtime = shard_metadata = shard_plan = None
        full_metadata = full_names = None
        batches = [selected]
        if args.runtime_integration == 'only':
            runtime, full_metadata, full_names = prepare_runtime_full(invocations, baseline)
            batches = [[runtime]]
        elif args.runtime_integration == 'exclude':
            original = validate_runtime_invocation(invocations)
            others = [row for row in selected if row is not original]
            if not others:
                raise RuntimeError('Runtime exclusion recorded no other REST invocations')
            batches = [others]
            print(f'Runtime integration replay: excluded; {len(others)} other REST invocations', flush=True)
        elif args.runtime_shard is not None:
            original, runtime, shard_metadata, shard_plan = prepare_runtime_shard(
                invocations, args.runtime_shard, baseline, shard_count)
            if args.runtime_shard == rest_shard:
                others = [row for row in selected if row is not original]
                batches = [others, [runtime]]
            else:
                batches = [[runtime]]
        collected = time.monotonic()

        def run(invocation):
            environment = invocation_environment(invocation, baseline)
            beginning = time.monotonic()
            result = run_command(invocation['command'], cwd=invocation['cwd'],
                                 env=environment, stdout=subprocess.PIPE,
                                 stderr=subprocess.STDOUT, text=True,
                                 encoding='utf-8', errors='replace')
            # Exit status is authoritative, including for custom test harnesses.
            summaries = re.findall(
                r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; '
                r'(\d+) measured; (\d+) filtered out', result.stdout)
            row = {'command': invocation['command'], 'cwd': invocation['cwd'],
                   'package': invocation['package'],
                   'status': result.returncode, 'seconds': time.monotonic() - beginning,
                   'summaries': summaries}
            if invocation is runtime:
                row['child_status'] = result.returncode
                try:
                    if full_metadata is not None:
                        row['runtime_full_execution'] = runtime_execution(
                            result.stdout, full_names, len(full_names))
                    else:
                        row['runtime_shard_execution'] = runtime_execution(
                            result.stdout, shard_plan['shards'][args.runtime_shard],
                            len(shard_plan['names']))
                except RuntimeError as error:
                    key = 'full_validation_error' if full_metadata is not None else 'shard_validation_error'
                    row[key] = str(error)
                    row['status'] = result.returncode or 1
            return row, result.stdout

        results = []
        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as executor:
            for batch in batches:
                futures = [executor.submit(run, invocation) for invocation in batch]
                for future in concurrent.futures.as_completed(futures):
                    row, output = future.result()
                    results.append(row)
                    print('::group::' + Path(row['command'][0]).name, flush=True)
                    print(output, end='' if output.endswith('\n') else '\n', flush=True)
                    print('::endgroup::', flush=True)
                    print(f'Cargo invocation elapsed: {Path(row["command"][0]).name} {row["seconds"]:.3f}s', flush=True)
                    if 'child_status' in row:
                        full = full_metadata is not None
                        execution = row.get('runtime_full_execution' if full else 'runtime_shard_execution', {})
                        public = {key: value for key, value in execution.items() if key != 'names'}
                        public.update(full_metadata if full else shard_metadata, child_status=row['child_status'],
                                      status=row['status'])
                        error_key = 'full_validation_error' if full else 'shard_validation_error'
                        if error_key in row:
                            public['validation_error'] = row[error_key]
                        prefix = 'Runtime full result: ' if full else 'Runtime shard result: '
                        print(prefix + json.dumps(public, sort_keys=True), flush=True)
        finished = time.monotonic()
        report = {'collection_seconds': collected - started,
                  'test_seconds': finished - collected, 'total_seconds': finished - started,
                  'suite': args.suite, 'recorded_invocations': len(invocations),
                  'invocations': sorted(results, key=lambda row: (row['command'], row['cwd']))}
        if shard_metadata is not None:
            report['runtime_shard'] = dict(shard_metadata, full_names=shard_plan['names'],
                                           planned_shards=shard_plan['shards'])
        if args.runtime_integration is not None:
            report['runtime_integration'] = args.runtime_integration
        if full_metadata is not None:
            report['runtime_full'] = dict(full_metadata, full_names=full_names)
        if args.rest_shard is not None:
            report['rest_shard'] = rest_shard
        if args.report:
            args.report.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
        failures = sum(row['status'] != 0 for row in results)
        print(f'Workspace phase elapsed: collection {collected - started:.3f}s; '
              f'tests {finished - collected:.3f}s; total {finished - started:.3f}s', flush=True)
        print(f'Ran all {len(results)} Cargo test invocations; {failures} failed', flush=True)
        return int(failures != 0)


if __name__ == '__main__':
    if len(sys.argv) > 3 and sys.argv[1] == '--record':
        record(sys.argv[2], sys.argv[3:])
    else:
        sys.exit(main())
