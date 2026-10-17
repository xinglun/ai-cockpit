import ctypes
import datetime
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time
import threading

REPO = Path('/workspace/ai-cockpit-wi1086-sensitive-amendment-preflight-request-interlock')
ROOT = REPO / '.ai/evidence/WI-1086-sensitive-amendment-preflight-request-interlock/full-verification-20261016'
ROOT.mkdir(exist_ok=True)
TARGET = REPO / 'target/wi1086-sensitive-amendment-preflight-request-interlock'
label, *command = sys.argv[1:]
if not label or '/' in label or not command:
    raise SystemExit('Invalid label or empty command')
if any((ROOT / (label + ext)).exists() for ext in ['.stdout', '.stderr', '.meta.json']):
    raise SystemExit('Refuse to overwrite an observation')
if ctypes.CDLL(None, use_errno=True).prctl(36, 1, 0, 0, 0) != 0:
    raise SystemExit('Cannot enable descendant reaping')
environment = os.environ.copy()
changes = {
    'HOME': '/tmp/wi1084-runtime-home-20261015',
    'RUSTUP_HOME': '/workspace/.rustup',
    'CARGO_HOME': '/workspace/.cargo',
    'PATH': '/workspace/.cargo/bin:' + environment['PATH'],
    'CARGO_BUILD_JOBS': '1',
    'CARGO_INCREMENTAL': '0',
    'RUST_TEST_THREADS': '1',
    'CARGO_PROFILE_DEV_DEBUG': '0',
    'CARGO_PROFILE_TEST_DEBUG': '0',
    'CARGO_TARGET_DIR': str(TARGET),
}
environment.update(changes)
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=REPO, text=True).strip()
source = REPO / 'crates/cockpit-cli/tests/contract_amendment_processes.rs'
started = datetime.datetime.now(datetime.timezone.utc).isoformat()
clock = time.monotonic()
timed_out = False
owned_reaped = []
reaper_stop = threading.Event()
def reap_owned_orphans(main_pid):
    children = Path(f'/proc/{os.getpid()}/task/{os.getpid()}/children')
    while not reaper_stop.is_set():
        try:
            candidates = sorted({int(value) for task in Path(f'/proc/{os.getpid()}/task').iterdir() for value in (task / 'children').read_text().split()})
        except OSError:
            candidates = []
        for candidate in candidates:
            if candidate == main_pid:
                continue
            try:
                pid, status = os.waitpid(candidate, os.WNOHANG)
                if pid:
                    owned_reaped.append({'pid': pid, 'waitStatus': status})
            except ChildProcessError:
                pass
        reaper_stop.wait(.1)
with (ROOT / (label + '.stdout')).open('xb') as stdout, (ROOT / (label + '.stderr')).open('xb') as stderr:
    process = subprocess.Popen(command, cwd=REPO, env=environment, stdout=stdout, stderr=stderr,
                               start_new_session=True)
    reaper_thread = threading.Thread(target=reap_owned_orphans, args=(process.pid,), daemon=True)
    reaper_thread.start()
    try:
        code = process.wait(timeout=900)
    except subprocess.TimeoutExpired:
        timed_out = True
        os.killpg(process.pid, signal.SIGTERM)
        try:
            code = process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            code = process.wait()
reaper_stop.set()
reaper_thread.join(timeout=1)
reaped = owned_reaped
deadline = time.monotonic() + 5
while True:
    try:
        pid, status = os.waitpid(-1, os.WNOHANG)
    except ChildProcessError:
        break
    if pid:
        reaped.append({'pid': pid, 'waitStatus': status})
    elif time.monotonic() >= deadline:
        break
    else:
        time.sleep(.05)
binary = TARGET / 'debug/ai-cockpit'
metadata = {
    'command': command, 'cwd': str(REPO), 'startedAt': started,
    'elapsedSeconds': round(time.monotonic() - clock, 3),
    'timeoutSeconds': 900, 'timedOut': timed_out, 'exitCode': code,
    'sourceHead': head, 'testFileRawSha256': hashlib.sha256(source.read_bytes()).hexdigest(),
    'toolchain': 'rustc 1.98.1 (48a229cea 2026-09-01); cargo 1.98.1 (797e8a9bc 2026-08-05)',
    'environmentOverrides': {k: v for k, v in changes.items() if k != 'PATH'},
    'binaryPath': str(binary),
    'binaryRawSha256': hashlib.sha256(binary.read_bytes()).hexdigest() if binary.is_file() else None,
    'ownedDescendantsReaped': reaped,
}
(ROOT / (label + '.meta.json')).write_text(json.dumps(metadata, indent=2) + '\n')
print(json.dumps({k:v for k,v in metadata.items() if k != 'ownedDescendantsReaped'} | {'ownedDescendantsReapedCount':len(reaped)}))
raise SystemExit(124 if timed_out else code)
