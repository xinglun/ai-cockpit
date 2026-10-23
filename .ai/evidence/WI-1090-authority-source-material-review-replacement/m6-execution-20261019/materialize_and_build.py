from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import os
import signal
import subprocess
import time

base = Path('/workspace/scratch/wi1090-m6-execution-20261019')
plan = Path('/workspace/scratch/wi1090-m6-readonly-plan-20261019')
owner = Path('/workspace/ai-cockpit-wi1090-material-replacement')
consumer = Path('/workspace/ai-cockpit-wi1089-recovery')
gov = Path('/workspace/scratch/wi1087-fixed-runtime-b8ec269-20261018/target/release/ai-cockpit')
item = 'WI-1090-authority-source-material-review-replacement'
source = base / 'source'
evidence = base / 'evidence'
evidence.mkdir(exist_ok=True)

def sha(data):
    return hashlib.sha256(data).hexdigest()

def save(name, data):
    path = evidence / name
    assert not path.exists(), path
    path.write_bytes(data)

def git(*args):
    return subprocess.check_output(['git', *args], cwd=owner)

assert not (base / 'build-candidate.attempt1.metadata.actual.json').exists()
assert sha(gov.read_bytes()) == 'b5cbd8570df5d3cb2004ae517fe88df6136c66d92fc1913070d18f2c8f67f257'
assert git('rev-parse', 'HEAD').decode().strip() == '33d79c5b89334b2c5b57a2df9317b856e8467b08'
assert not git('status', '--porcelain')
manifest_bytes = (plan / 'prospective-non-ai-source-manifest.proposal-only.json').read_bytes()
assert sha(manifest_bytes) == '299d90d475d2905a43868e5dc995993f0b79fafc7b9f32c452114833d37c50b8'
assert sha((plan / 'wi1090-two-rust-files.proposal-only.patch').read_bytes()) == '7129aa3fefeb5cfda56b93c948b695a4815e3e1878b3d0a13d591ea405738541'
manifest = json.loads(manifest_bytes)
for label, args in [('owner-status', ['work-item', 'status', '--id', item, '--json']),
                    ('owner-validate', ['work-item', 'validate', '--id', item, '--json'])]:
    result = subprocess.run([str(gov), *args, '--repo', str(owner)], capture_output=True, timeout=90)
    save(label + '.actual.json', result.stdout)
    save(label + '.stderr.actual.txt', result.stderr)
    assert result.returncode == 0, result.stderr
    if label == 'owner-status':
        state = json.loads(result.stdout)
        assert not state['blocking'] and not state['humanDecisionRequired']
        assert 'run_verification' in state['safeActions']
        assert state['sourceDigests']['contract'] == 'sha256:c9a3bda349078f1fc15891e9e789d100952cbadaea12eb464f2024c90617ce9c'

ledger = base / 'progress.md'
assert not ledger.exists()
ledger.write_text('# SDD ledger — plan: /workspace/scratch/wi1090-m6-readonly-plan-20261019/M6-minimum-plan.proposal-only.zh-CN.md\n\n'
    'Task1: Route admitted for this exact serial single-WI artifact/fixture execution, not native cross-WI composition.\n'
    'Ruling: ordinary-work-item permits admitted serial fallback; agent-workflow lines278-298 explicitly distinguishes parallel coordination from admitted serial work, and c9-bound FIXTURES F3 explicitly uses approved external governance fixtures. Only WI1090 owns this test; ba1 is immutable source input. No live WI1089 action/registration/lease/provider integration is executed. Raydot explicitly approved the exact manifest and patch. Cost if wrong: stop rather than imitate or bypass a required composition action. No current Contract/capability declares that dedicated API mandatory for these synthetic single-owner tests.\n'
    'Ruling: Keep all plan/ledger/evidence in authorized external scratch and preserve them; user overrides skill-default worktree scratch and deletion. Raydot owns review; no subagents. No pending-negative source changes.\n'
    'Pre-flight: source manifest produces exact code consumed by one candidate build; both fixture cases must pin that same actual binary. Actual old canonical/Outcome in each case feed its own mixed history and current four-node cycle; no real history import.\n')
assert not source.exists()
source.mkdir()
unique = sorted({v['gitBlob'] for v in manifest.values() if v['originCommit'] != 'prospective_union_of_exact_two_rows'})
result = subprocess.run(['git', 'cat-file', '--batch'], input=('\n'.join(unique) + '\n').encode(),
                        cwd=owner, capture_output=True, timeout=90)
assert result.returncode == 0, result.stderr
objects = {}
offset = 0
for oid in unique:
    end = result.stdout.index(b'\n', offset)
    header = result.stdout[offset:end].decode().split()
    assert header[0] == oid and header[1] == 'blob', header
    size = int(header[2]); start = end + 1
    objects[oid] = result.stdout[start:start + size]
    assert result.stdout[start + size:start + size + 1] == b'\n'
    offset = start + size + 1
assert offset == len(result.stdout)
actual = {}
for name, expected in manifest.items():
    relative = Path(name)
    assert not relative.is_absolute() and '..' not in relative.parts
    path = source / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    if expected['originCommit'] == 'prospective_union_of_exact_two_rows':
        original = git('show', 'ba1c6c90f4ac039521567c1365b4351fe7ce92ae:' + name)
        newer = git('show', '33d79c5b89334b2c5b57a2df9317b856e8467b08:' + name)
        row, = [x for x in newer.splitlines(keepends=True) if x.startswith(('| ' + item + ' ').encode())]
        lines = original.splitlines(keepends=True)
        anchor, = [i for i, x in enumerate(lines) if x.startswith(b'| WI-1089-blocked-outcome-material-review-recovery ')]
        lines.insert(anchor + 1, row)
        data = b''.join(lines)
    else:
        data = objects[expected['gitBlob']]
    oid = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
    assert oid == expected['gitBlob'], name
    if 'rawSha256' in expected:
        assert sha(data) == expected['rawSha256'], name
    if expected['mode'] == '120000':
        path.symlink_to(os.fsdecode(data))
        assert os.fsencode(os.readlink(path)) == data
    else:
        assert expected['mode'] in ['100644', '100755'], name
        path.write_bytes(data)
        path.chmod(0o755 if expected['mode'] == '100755' else 0o644)
        assert path.read_bytes() == data
    actual[name] = dict(expected)
assert not (source / '.ai').exists() and not (source / '.git').exists()
actual_bytes = (json.dumps(actual, sort_keys=True, ensure_ascii=False, separators=(',', ':')) + '\n').encode()
assert actual_bytes == manifest_bytes
save('actual-materialized-source-manifest.json', actual_bytes)
save('actual-materialization-result.agent-recorded.json', (json.dumps({'fileCount': len(actual), 'manifestRawSha256': sha(actual_bytes),
    'approvedManifestExactMatch': True, 'approvedRustPatchRawSha256': '7129aa3fefeb5cfda56b93c948b695a4815e3e1878b3d0a13d591ea405738541',
    'sourceArtifactOnlyNotNativeComposition': True}, indent=2) + '\n').encode())
print('Exact approved materialization verified: ' + str(len(actual)) + ' files, sha256:' + sha(actual_bytes), flush=True)
env = os.environ.copy()
env.update(PATH='/workspace/.cloud-tools/cargo/bin:' + env['PATH'], CARGO_HOME='/workspace/.cloud-tools/cargo',
           RUSTUP_HOME='/workspace/.cloud-tools/rustup', CARGO_TARGET_DIR=str(base / 'build-target'),
           CARGO_BUILD_JOBS='1', CARGO_INCREMENTAL='0', CARGO_PROFILE_DEV_DEBUG='0', CARGO_NET_OFFLINE='true')
argv = ['cargo', 'build', '--locked', '--offline', '--package', 'cockpit-cli', '--bin', 'ai-cockpit']
metadata = base / 'build-candidate.attempt1.metadata.actual.json'
meta = {'kind': 'actual_isolated_m6_build', 'argv': argv, 'sourceManifestRawSha256': sha(actual_bytes),
        'startedAt': datetime.now(timezone.utc).isoformat(), 'terminal': 'running', 'outerTimeoutSeconds': 850,
        'target': env['CARGO_TARGET_DIR'], 'workerCount': 1}
started = time.monotonic()
with (base / 'build-candidate.attempt1.stdout.actual.txt').open('wb') as stdout, (base / 'build-candidate.attempt1.stderr.actual.txt').open('wb') as stderr:
    process = subprocess.Popen(argv, cwd=source, env=env, stdout=stdout, stderr=stderr, start_new_session=True)
    meta.update(pid=process.pid, processGroup=process.pid)
    metadata.write_text(json.dumps(meta, indent=2) + '\n')
    try:
        code = process.wait(timeout=850)
        meta.update(exitCode=code, terminal='returned')
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGTERM)
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait(timeout=5)
        meta.update(exitCode=process.returncode, terminal='outer_timeout_children_state_not_inferred')
meta.update(endedAt=datetime.now(timezone.utc).isoformat(), elapsedSeconds=time.monotonic() - started)
metadata.write_text(json.dumps(meta, indent=2) + '\n')
print(json.dumps(meta), flush=True)
assert meta['terminal'] == 'returned' and meta['exitCode'] == 0, meta
binary = base / 'build-target/debug/ai-cockpit'
version = subprocess.check_output([str(binary), '--version'], text=True).strip()
identity = {'path': str(binary), 'rawSha256': sha(binary.read_bytes()), 'versionOutput': version,
            'sourceManifestRawSha256': sha(actual_bytes), 'installed': False, 'adoptedToRealWorkItem': False,
            'inspectRuntimeDigest': None, 'inspectState': 'to_be_read_in_first_new_isolated_fixture'}
save('candidate-binary.actual.agent-recorded.json', (json.dumps(identity, indent=2) + '\n').encode())
with ledger.open('a') as stream:
    stream.write('\nTask1: complete. Exact3176-file approved manifest materialized; build exit0. Candidate SHA' + identity['rawSha256'] + '; no install/adoption. Inspect identity will be read in first synthetic fixture.\n')
print(json.dumps(identity), flush=True)
