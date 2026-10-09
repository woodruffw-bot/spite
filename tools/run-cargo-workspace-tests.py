#!/usr/bin/env python3
"""Run Cargo's unchanged workspace test invocations two binaries at a time."""

import argparse
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


def fingerprint(value):
    return hashlib.sha256(value.encode('utf-8', 'surrogateescape')).hexdigest()


def record(directory, command):
    directory = Path(directory)
    baseline = json.loads((directory / 'environment-fingerprints.json').read_text())
    changed = {key: value for key, value in os.environ.items()
               if baseline.get(key) != fingerprint(value)}
    removed = [key for key in baseline if key not in os.environ]
    row = {'command': command, 'cwd': str(Path.cwd()),
           'changed_environment': changed, 'removed_environment': removed}
    path = directory / f'invocation-{time.monotonic_ns()}-{uuid.uuid4().hex}.json'
    path.write_text(json.dumps(row), encoding='utf-8')
    print('Queued Cargo test invocation: ' + Path(command[0]).name, flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--toolchain', choices=['stable', '1.85.0'], required=True)
    parser.add_argument('--offline', action='store_true')
    parser.add_argument('--report', type=Path)
    args = parser.parse_args()
    version = subprocess.check_output(['rustc', '+' + args.toolchain, '-vV'], text=True)
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
        command = ['cargo', '+' + args.toolchain, 'test', '--workspace', '--all-targets',
                   '--locked', '--config', configuration]
        if args.offline:
            command.append('--offline')
        command += ['--', '--skip', 'pinned_corpus_runs_all_reviewed_variants']
        print('Collecting Cargo test invocations with their original arguments and environment', flush=True)
        status = subprocess.run(command).returncode
        if status:
            return status
        invocations = [json.loads(path.read_text(encoding='utf-8')) for path in
                       sorted(directory.glob('invocation-*.json'))]
        if not invocations:
            raise RuntimeError('Cargo recorded no test invocations; check the host runner configuration')
        collected = time.monotonic()

        def run(invocation):
            environment = baseline.copy()
            environment.update(invocation['changed_environment'])
            for key in invocation['removed_environment']:
                environment.pop(key, None)
            beginning = time.monotonic()
            result = subprocess.run(invocation['command'], cwd=invocation['cwd'],
                                    env=environment, stdout=subprocess.PIPE,
                                    stderr=subprocess.STDOUT, text=True,
                                    encoding='utf-8', errors='replace')
            # Exit status is authoritative, including for custom test harnesses.
            summaries = re.findall(
                r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; '
                r'(\d+) measured; (\d+) filtered out', result.stdout)
            row = {'command': invocation['command'], 'cwd': invocation['cwd'],
                   'status': result.returncode, 'seconds': time.monotonic() - beginning,
                   'summaries': summaries}
            return row, result.stdout

        results = []
        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as executor:
            futures = [executor.submit(run, invocation) for invocation in invocations]
            for future in concurrent.futures.as_completed(futures):
                row, output = future.result()
                results.append(row)
                print('::group::' + Path(row['command'][0]).name, flush=True)
                print(output, end='' if output.endswith('\n') else '\n', flush=True)
                print('::endgroup::', flush=True)
        finished = time.monotonic()
        report = {'collection_seconds': collected - started,
                  'test_seconds': finished - collected, 'total_seconds': finished - started,
                  'invocations': sorted(results, key=lambda row: (row['command'], row['cwd']))}
        if args.report:
            args.report.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
        failures = sum(row['status'] != 0 for row in results)
        print(f'Ran all {len(results)} Cargo test invocations; {failures} failed', flush=True)
        return int(failures != 0)


if __name__ == '__main__':
    if len(sys.argv) > 3 and sys.argv[1] == '--record':
        record(sys.argv[2], sys.argv[3:])
    else:
        sys.exit(main())
