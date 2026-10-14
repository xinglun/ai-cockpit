import ctypes
import hashlib
import json
import os
from pathlib import Path
import select
import struct
import subprocess
import time

repo = Path('/workspace/ai-cockpit-wi1081-append-only-evidence-observations')
temporary_root = Path('/workspace/.tmp-wi1081-tests-20261014')
output = repo / '.ai/evidence/WI-1081-append-only-evidence-observations.pause-observation-probe1-20261014'
output.mkdir(exist_ok=False)
libc = ctypes.CDLL(None, use_errno=True)
libc.inotify_init1.argtypes = [ctypes.c_int]
libc.inotify_add_watch.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_uint32]
notify_fd = libc.inotify_init1(os.O_NONBLOCK | os.O_CLOEXEC)
if notify_fd < 0:
    raise OSError(ctypes.get_errno(), 'inotify_init1')
mask = 0x100 | 0x8 | 0x80 | 0x400
watches = {}
held = {}
seen = set()
captures = []
events = []

def add_watch(path):
    wd = libc.inotify_add_watch(notify_fd, os.fsencode(path), mask)
    if wd >= 0:
        watches[wd] = path
    else:
        events.append({'kind': 'watch_error', 'path': str(path), 'errno': ctypes.get_errno()})
    return wd

root_watch = add_watch(temporary_root)
if root_watch < 0:
    raise RuntimeError('cannot watch diagnostic temporary root')

def open_record(path):
    if path.suffix not in ('.json', '.tmp') or str(path) in held:
        return
    try:
        held[str(path)] = os.open(path, os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW)
    except FileNotFoundError:
        pass

def capture_records():
    for source, fd in list(held.items()):
        raw = os.pread(fd, 1048577, 0)
        if not raw or len(raw) > 1048576:
            continue
        try:
            record = json.loads(raw)
        except (ValueError, UnicodeError):
            continue
        if not isinstance(record, dict) or 'attemptId' not in record:
            continue
        digest = hashlib.sha256(raw).hexdigest()
        if digest in seen:
            continue
        seen.add(digest)
        path = output / f'{len(captures):03d}-{digest}.captured.json'
        path.write_bytes(raw)
        captures.append({'file': path.name, 'sourcePath': source, 'sha256': digest,
                         'bytes': len(raw), 'observedUnixNs': time.time_ns(),
                         'attemptId': record['attemptId'], 'passed': record.get('passed'),
                         'failure': record.get('failure'),
                         'executionOutcome': record.get('executionOutcome'),
                         'cleanupDisposition': record.get('cleanupDisposition')})

environment = os.environ.copy()
environment.update({'PATH': '/workspace/.cargo/bin:' + environment['PATH'],
                    'HOME': '/workspace/.tmp-wi1081-runtime-home-20261014',
                    'RUSTUP_HOME': '/workspace/.rustup', 'CARGO_HOME': '/workspace/.cargo',
                    'CARGO_TARGET_DIR': '/tmp/wi1081-build-00461714-20261014',
                    'CARGO_BUILD_JOBS': '1', 'CARGO_INCREMENTAL': '0',
                    'CARGO_PROFILE_TEST_DEBUG': '0', 'TMPDIR': str(temporary_root),
                    'RUST_TEST_THREADS': '2', 'AI_COCKPIT_COMPOSITION_RECONCILE_TRACE': '1'})
command = ['/tmp/wi1079-toolchain/runtime/ai-cockpit', 'verify', '--repo', str(repo),
           '--work-item', 'WI-1081-append-only-evidence-observations', '--command', 'cargo',
           '--args=test,--package,cockpit-verification,--test,composition,--,--nocapture',
           '--workers', '1']
started = time.time_ns()
(output / 'invocation.json').write_text(json.dumps({'kind': 'agent_read_only_capture_probe',
    'startedUnixNs': started, 'command': command, 'testThreads': 2,
    'home': environment['HOME'], 'tmpdir': environment['TMPDIR'],
    'sourceHead': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=repo, text=True).strip()}, indent=2) + '\n')
with (output / 'runtime.log').open('wb') as log:
    process = subprocess.Popen(command, cwd=repo, env=environment, stdout=log, stderr=subprocess.STDOUT)
    try:
        while process.poll() is None:
            ready, _, _ = select.select([notify_fd], [], [], 0.05)
            if ready:
                data = os.read(notify_fd, 262144)
                offset = 0
                while offset < len(data):
                    wd, event_mask, cookie, length = struct.unpack_from('iIII', data, offset)
                    name = data[offset + 16:offset + 16 + length].split(b'\0', 1)[0]
                    offset += 16 + length
                    parent = watches.get(wd)
                    if event_mask & 0x4000:
                        events.append({'kind': 'inotify_queue_overflow'})
                    if parent is None:
                        continue
                    path = parent / os.fsdecode(name)
                    if wd == root_watch and name.startswith(b'cockpit-composition-reuse-pause-race-'):
                        if path.is_dir():
                            add_watch(path)
                            for item in path.iterdir():
                                open_record(item)
                    elif wd != root_watch:
                        open_record(path)
            capture_records()
        capture_records()
    finally:
        # Reap the owned Runtime; never leave it detached if capture fails.
        exit_code = process.wait()
        for fd in held.values():
            os.close(fd)
        os.close(notify_fd)
(output / 'exit-code').write_text(str(exit_code) + '\n')
(output / 'capture-manifest.json').write_text(json.dumps({'kind': 'agent_read_only_capture_manifest',
    'runtimeExitCode': exit_code, 'elapsedSeconds': (time.time_ns() - started) / 1e9,
    'events': events, 'captures': captures}, indent=2) + '\n')
print(json.dumps({'runtimeExitCode': exit_code, 'captures': len(captures), 'output': str(output)}))
raise SystemExit(exit_code)
