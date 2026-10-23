#!/usr/bin/env python3
"""Execute frozen WI-1107 with the separately pinned official Runtime.

The helper is not an authority: only the fixed Runtime records native evidence.
This file and its workflow stay outside the frozen source checkout.
"""

import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import stat
import subprocess
import tarfile
import tempfile
import time


def execution_environment(inherited):
    # Give the verification process one explicit, publicly recordable environment.
    # Host credentials remain available to the host's artifact action, never to
    # the Runtime or tests. Natural HOME/PATH/toolchain values are not rewritten.
    allowed = set(('HOME PATH USER LOGNAME SHELL LANG LC_ALL LC_CTYPE TZ TMPDIR TMP TEMP '
                   'CARGO_HOME RUSTUP_HOME RUSTUP_TOOLCHAIN RUSTUP_DIST_SERVER RUSTUP_UPDATE_ROOT '
                   'CARGO_TARGET_DIR CARGO_INCREMENTAL RUSTFLAGS RUSTDOCFLAGS CARGO_BUILD_TARGET '
                   'CARGO_BUILD_JOBS CC CXX AR CFLAGS CXXFLAGS LDFLAGS LIBRARY_PATH LD_LIBRARY_PATH '
                   'PKG_CONFIG_PATH CI GITHUB_ACTIONS GITHUB_WORKSPACE GITHUB_SHA GITHUB_REF '
                   'GITHUB_REF_NAME GITHUB_EVENT_NAME GITHUB_EVENT_PATH GITHUB_REPOSITORY '
                   'GITHUB_RUN_ID GITHUB_RUN_ATTEMPT RUNNER_OS RUNNER_ARCH RUNNER_TEMP').split())
    unsafe_value = re.compile(r'(?i)(?:https?://[^/\s]*@|[?&](?:token|key|secret|sig)=|'
                              r'-----BEGIN .*PRIVATE KEY-----|github_pat_|gh[pousr]_|sk-proj-)')
    values = {key: value for key, value in sorted(inherited.items())
              if key in allowed and not unsafe_value.search(value)}
    check('HOME' in values and 'PATH' in values, 'natural HOME/PATH unavailable or unsafe')
    for key, value in values.items():
        key.encode('utf-8')
        value.encode('utf-8')
    return values, sorted(set(inherited) - set(values))


def cargo_environment(values, source):
    effective = {k: v for k, v in values.items()
                 if k not in ['CARGO_INCREMENTAL', 'CARGO_TARGET_DIR']}
    effective['CARGO_INCREMENTAL'] = '0'
    target = values.get('CARGO_TARGET_DIR')
    if not target:
        target = str(Path(values['HOME']) / '.cache/ai-cockpit-verify-target' /
                     sha(os.fsencode(source)))
    effective['CARGO_TARGET_DIR'] = target
    return list(effective.items())


def check(condition, message):
    if not condition:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def read_json(path):
    return json.loads(Path(path).read_bytes())


def write_json(path, value):
    with Path(path).open('x', encoding='utf-8') as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write('\n')


def archive_payloads(path, records):
    expected = {item['path']: item for item in records}
    check(len(expected) == len(records), 'duplicate manifest path')
    payloads = {}
    with tarfile.open(path, 'r:gz') as archive:
        for member in archive:
            name = member.name
            parts = PurePosixPath(name)
            check(name in expected and name not in payloads, 'unexpected or duplicate archive path')
            check(not parts.is_absolute() and '..' not in parts.parts and '\\' not in name,
                  'unsafe archive path')
            check(member.isfile(), 'archive member must be a regular file')
            entry = expected[name]
            check(member.size == entry['bytes'], 'archive member size mismatch')
            data = archive.extractfile(member).read(entry['bytes'] + 1)
            check(len(data) == entry['bytes'] and sha(data) == entry['sha256'],
                  'native input bytes changed')
            payloads[name] = data
    check(set(payloads) == set(expected), 'missing native input')
    return payloads


def require_status(status, manifest, verified=False):
    check(status.get('workItemId') == manifest['workItemId'], 'status Work Item mismatch')
    check(status.get('repositoryId') == manifest['repositoryId'], 'status repository mismatch')
    binding = status.get('sourceDigests', {})
    check(binding.get('contract') == manifest['contractDigest'], 'Contract binding changed')
    check(binding.get('repositorySnapshot') == manifest['sourceSnapshotDigest'],
          'source snapshot binding changed')
    check(status.get('humanDecisionRequired') is False, 'Runtime requires a human decision')
    if verified:
        check(status.get('verification') == 'verified', 'Runtime did not accept verification')
        check(status.get('evidenceFreshness', {}).get('state') == 'fresh',
              'Runtime evidence is not fresh')


def require_plan(plan, manifest):
    for key in ['workItemId', 'repositoryId', 'runtimeDigest', 'runtimeVersion']:
        check(plan.get(key) == manifest[key], 'plan identity mismatch: ' + key)
    check(plan.get('repositorySnapshotDigest') == manifest['sourceSnapshotDigest'],
          'plan snapshot mismatch')
    requests = manifest['expectedRequests']
    count = len(requests)
    check(plan.get('requests') == requests, 'original expanded verification commands changed')
    check(plan.get('nodesPlanned') == count and plan.get('nodesToExecute') == count
          and plan.get('nodesReused') == 0 and plan.get('processesSpawned') == 0,
          'plan must select all original nodes without spawning or reuse')
    check(plan.get('timeoutSeconds') == 900, 'verification timeout changed')
    nodes = plan.get('plannedNodes', [])
    check(len(nodes) == count and all(node.get('action') == 'execute' for node in nodes),
          'plan contains a non-execution node')


def require_success(evidence, status, manifest, attempts):
    check(evidence.get('passed') is True, 'native canonical evidence did not pass')
    require_status(status, manifest, verified=True)
    for key in ['workItemId', 'repositoryId', 'contractDigest', 'runtimeDigest', 'runtimeVersion']:
        check(evidence.get(key) == manifest[key], 'evidence identity mismatch: ' + key)
    check(evidence.get('repositorySnapshotDigest') == manifest['sourceSnapshotDigest'],
          'evidence source mismatch')
    receipt = evidence.get('receipt', {})
    count = len(manifest['expectedRequests'])
    ids = {item['nodeId'] for item in manifest['expectedRequests']}
    check(receipt.get('passed') is True and receipt.get('nodesPlanned') == count
          and receipt.get('nodesExecuted') == count and receipt.get('nodesReused') == 0,
          'all original nodes must execute and pass without reuse')
    results = receipt.get('results', [])
    records = receipt.get('executionRecords', [])
    check(len(results) == count and {r.get('nodeId') for r in results} == ids
          and all(r.get('passed') is True and not r.get('reused')
                  and not r.get('timedOut') for r in results), 'incomplete node results')
    check(len(records) == count and {r.get('nodeId') for r in records} == ids
          and all(r.get('passed') is True and r.get('spawned') is True
                  and r.get('exitCode') == 0 and r.get('timedOut') is False
                  and r.get('timeoutSeconds') == 900 for r in records),
          'incomplete actual execution records')
    expected = {r['nodeId']: r for r in manifest['expectedRequests']}
    matching = []
    for attempt in attempts:
        if attempt.get('state') != 'execution_completed' or attempt.get('passed') is not True:
            continue
        if any(attempt.get(key) != manifest[key] for key in
               ['workItemId', 'repositoryId', 'contractDigest', 'runtimeDigest', 'runtimeVersion']):
            continue
        if attempt.get('repositorySnapshotDigest') != manifest['sourceSnapshotDigest']:
            continue
        if attempt.get('receipt') != receipt or attempt.get('executionRecords') != records:
            continue
        commands = attempt.get('commands', [])
        if len(commands) != count or {c.get('nodeId') for c in commands} != ids:
            continue
        if any(c.get('program') != expected[c['nodeId']]['program']
               or c.get('args') != expected[c['nodeId']]['args']
               or c.get('timeoutSeconds') != 900 for c in commands):
            continue
        by_id = {r['nodeId']: r for r in records}
        if any(c.get('commandDigest') != by_id[c['nodeId']].get('commandDigest') for c in commands):
            continue
        matching.append(attempt)
    check(len(matching) == 1, 'no unique new successful native attempt binds the canonical receipt')


def verify_source(source, manifest):
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=source, text=True).strip()
    check(head == manifest['sourceHead'], 'not the frozen source HEAD')
    subprocess.run(['git', 'diff', '--exit-code', 'HEAD', '--'], cwd=source, check=True,
                   stdout=subprocess.DEVNULL)
    for item in manifest['sourceFiles']:
        path = source / item['path']
        check(not path.is_symlink() and sha(path.read_bytes()) == item['sha256'],
              'frozen source bytes changed: ' + item['path'])


def restore_native(source, payloads):
    for name, data in payloads.items():
        target = source / name
        parent = source
        for part in PurePosixPath(name).parts[:-1]:
            parent /= part
            if parent.exists() or parent.is_symlink():
                check(not parent.is_symlink() and parent.is_dir(), 'unsafe native parent')
            else:
                parent.mkdir()
        if target.exists() or target.is_symlink():
            check(not target.is_symlink() and target.is_file() and target.read_bytes() == data,
                  'refusing to overwrite existing native bytes')
        else:
            with target.open('xb') as stream:
                stream.write(data)


def save_command(label, argv, source, artifacts, environment):
    started = now()
    print(label + ': started', flush=True)
    with (artifacts / (label + '.stdout')).open('xb') as out, \
            (artifacts / (label + '.stderr')).open('xb') as err:
        process = subprocess.Popen(argv, cwd=source, stdout=out, stderr=err, env=environment)
        while True:
            try:
                code = process.wait(timeout=30)
                break
            except subprocess.TimeoutExpired:
                print(label + ': running; output is being preserved', flush=True)
    write_json(artifacts / (label + '.execution.json'),
               {'argv': list(map(str, argv)), 'startedAt': started, 'endedAt': now(),
                'exitCode': code, 'cwd': str(source)})
    print(label + ': exit ' + str(code), flush=True)
    return code


def preserve_native(source, artifacts, work_item):
    paths = set()
    for folder in ['.ai/work-items/active', '.ai/evidence', '.ai/decisions']:
        for path in (source / folder).glob(work_item + '.*'):
            if path.is_dir() and not path.is_symlink():
                paths.update(p for p in path.rglob('*') if p.is_file() and not p.is_symlink())
            elif path.is_file() and not path.is_symlink():
                paths.add(path)
    with tarfile.open(artifacts / 'native-after.tar.gz', 'w:gz') as archive:
        for path in sorted(paths):
            archive.add(path, arcname=path.relative_to(source), recursive=False)


def run(manifest_path, source, runtime, artifacts, validate_only=False):
    manifest = read_json(manifest_path)
    source, runtime = source.resolve(), runtime.resolve()
    archive = manifest_path.parent / manifest['nativeArchive']['path']
    check(sha(runtime.read_bytes()) == manifest['runtimeDigest'].removeprefix('sha256:'),
          'official Runtime binary digest mismatch')
    check(sha(archive.read_bytes()) == manifest['nativeArchive']['sha256'],
          'native archive digest mismatch')
    verify_source(source, manifest)
    payloads = archive_payloads(archive, manifest['nativeFiles'])
    if validate_only:
        print('Exact frozen source, fixed Runtime and ten native input bytes checked; no verification executed.')
        return 0
    artifacts.mkdir(parents=True, exist_ok=False)
    shutil.copyfile(archive, artifacts / 'native-before.tar.gz')
    shutil.copyfile(manifest_path, artifacts / 'binding-manifest.json')
    started = now()
    outcome = {'state': 'not_verified', 'startedAt': started,
               'sourceHead': manifest['sourceHead'], 'runtimeDigest': manifest['runtimeDigest'],
               'contractDigest': manifest['contractDigest'],
               'sourceSnapshotDigest': manifest['sourceSnapshotDigest'],
               'workflowCommit': os.environ.get('GITHUB_SHA'),
               'workflowRunId': os.environ.get('GITHUB_RUN_ID'),
               'workflowRunAttempt': os.environ.get('GITHUB_RUN_ATTEMPT')}
    try:
        restore_native(source, payloads)
        environment, omitted = execution_environment(os.environ)
        home = Path(environment['HOME'])
        check(home.is_absolute(), 'runner natural HOME must be absolute')
        cache = home / '.cache' / 'ai-cockpit-verify-target'
        cache.mkdir(parents=True, exist_ok=True)
        check(not cache.is_symlink(), 'default cache must not be a symlink')
        with tempfile.TemporaryDirectory(prefix='wi1107-write-probe-', dir=cache) as probe:
            (Path(probe) / 'probe').write_bytes(b'WI-1107 default cache capability probe\n')
        write_json(artifacts / 'execution-environment.json', {
            'observedAt': now(), 'naturalHome': str(home), 'defaultImplicitCache': str(cache),
            'cacheWriteProbe': 'passed', 'workspaceFreeBytes': shutil.disk_usage(source).free,
            'runnerImage': os.environ.get('ImageVersion'),
            'canonicalCwd': str(source),
            'orderedRuntimeEnvironment': list(environment.items()),
            'effectiveCargoEnvironment': cargo_environment(environment, source),
            'omittedHostEnvironmentKeysWithoutValues': omitted,
            'fixedSourceCommit': 'a1dbec70c8fc050b95165163da0da03cd90b79d1',
            'commandIdentitySerialization': 'serde_json::to_vec((logical_program, args, Some(cwd_os_bytes), 1, 900, false, ordered_environment_os_byte_pairs)); sha256. Non-Cargo uses orderedRuntimeEnvironment; Cargo uses effectiveCargoEnvironment. Linux pinned executable adds no loader variables.'})
        for label, argv in [('rust-toolchain', ['rustc', '-Vv']), ('cargo-version', ['cargo', '-V'])]:
            check(save_command(label, argv, source, artifacts, environment) == 0, label + ' failed')
        prefix = [str(runtime)]
        status_args = prefix + ['work-item', 'status', '--repo', str(source), '--id',
                                manifest['workItemId'], '--json']
        check(save_command('status-before', status_args, source, artifacts, environment) == 0,
              'status failed')
        status = read_json(artifacts / 'status-before.stdout')
        require_status(status, manifest)
        if 'run_verification' not in status.get('safeActions', []):
            check('run_preflight' in status.get('safeActions', []),
                  'Runtime admits neither preflight nor verification')
            args = prefix + ['preflight', '--repo', str(source), '--contract',
                             str(source / manifest['contractPath'])]
            check(save_command('preflight', args, source, artifacts, environment) == 0,
                  'preflight failed')
            check(save_command('status-refreshed', status_args, source, artifacts, environment) == 0,
                  'refreshed status failed')
            status = read_json(artifacts / 'status-refreshed.stdout')
            require_status(status, manifest)
        check('run_verification' in status.get('safeActions', []), 'verification not admitted')
        verify_args = prefix + ['verify', '--repo', str(source), '--work-item',
                                manifest['workItemId'], '--timeout-seconds', '900', '--workers', '1']
        check(save_command('formal-plan', verify_args + ['--plan-only'], source, artifacts,
                           environment) == 0,
              'formal plan rejected')
        require_plan(read_json(artifacts / 'formal-plan.stdout'), manifest)
        before = set((source / '.ai/evidence').glob(manifest['workItemId'] + '.verification-attempt.*.json'))
        code = save_command('formal-verification', verify_args, source, artifacts, environment)
        outcome['formalVerificationExitCode'] = code
        check(save_command('status-after', status_args, source, artifacts, environment) == 0,
              'post-verification status failed')
        check(code == 0, 'formal Runtime verification failed; original failure retained')
        evidence_path = source / '.ai/evidence' / (manifest['workItemId'] + '.verification.json')
        evidence = read_json(evidence_path)
        attempts = [read_json(p) for p in sorted(set((source / '.ai/evidence').glob(
            manifest['workItemId'] + '.verification-attempt.*.json')) - before)]
        require_success(evidence, read_json(artifacts / 'status-after.stdout'), manifest, attempts)
        verify_source(source, manifest)
        check(sha(runtime.read_bytes()) == manifest['runtimeDigest'].removeprefix('sha256:'),
              'fixed Runtime changed during verification')
        for item in manifest['nativeFiles']:
            if item['path'] in [manifest['summaryPath'],
                                f".ai/evidence/{manifest['workItemId']}.verification.json"]:
                continue
            check(sha((source / item['path']).read_bytes()) == item['sha256'],
                  'original immutable native input changed')
        outcome.update(state='runtime_verified', nodesExecuted=len(manifest['expectedRequests']),
                       nodesReused=0, newAttemptIds=[a['attemptId'] for a in attempts],
                       note='No finish/archive/publication/activation or human decision recorded by this job.')
        return 0
    except Exception as error:
        outcome['error'] = str(error)
        print('STOP: ' + str(error), flush=True)
        return 1
    finally:
        outcome['endedAt'] = now()
        if (source / '.ai/work-items/active').exists():
            preserve_native(source, artifacts, manifest['workItemId'])
        write_json(artifacts / 'hosted-outcome.json', outcome)
        files = {str(p.relative_to(artifacts)): {'bytes': p.stat().st_size,
                 'sha256': sha(p.read_bytes())} for p in artifacts.rglob('*') if p.is_file()}
        write_json(artifacts / 'artifact-digests.json', files)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest', type=Path, required=True)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--artifacts', type=Path, required=True)
    parser.add_argument('--validate-only', action='store_true')
    args = parser.parse_args()
    raise SystemExit(run(args.manifest.resolve(), args.source, args.runtime,
                         args.artifacts, args.validate_only))
