import datetime
import ctypes
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path('/workspace/wi1087-planning-20261016')
REPO = '/workspace/ai-cockpit-wi1087-contract-authority-scope-and-expiry'
BINARY = '/tmp/wi1084-closeout-execution-retained-b1487517/ai-cockpit'
name, *arguments = sys.argv[1:]
if not name or '/' in name or any((ROOT / (name + suffix)).exists() for suffix in ['.stdout', '.stderr', '.meta.json']):
    raise SystemExit('Refuse invalid or reused observation name')
environment = os.environ.copy()
environment.update({
    'HOME': '/tmp/wi1084-runtime-home-20261015',
    'RUSTUP_HOME': '/workspace/.rustup',
    'CARGO_HOME': '/workspace/.cargo',
    'PATH': '/workspace/.cargo/bin:' + environment['PATH'],
    'CARGO_BUILD_JOBS': '1',
    'CARGO_INCREMENTAL': '0',
    'RUST_TEST_THREADS': '1',
})
command = [BINARY, *arguments]
subreaper = False
if sys.platform.startswith('linux'):
    if ctypes.CDLL(None, use_errno=True).prctl(36, 1, 0, 0, 0) != 0:
        raise SystemExit('Cannot enable owned-query descendant reaping')
    subreaper = True
started = datetime.datetime.now(datetime.timezone.utc).isoformat()
clock = time.monotonic()
result = subprocess.run(command, cwd=REPO, env=environment, capture_output=True, timeout=120)
reaped = []
end = time.monotonic() + 5
if subreaper:
    while True:
        try:
            pid, status = os.waitpid(-1, os.WNOHANG)
        except ChildProcessError:
            break
        if pid:
            reaped.append({'pid': pid, 'waitStatus': status})
        else:
            if time.monotonic() >= end:
                break
            time.sleep(.05)
(ROOT / (name + '.stdout')).write_bytes(result.stdout)
(ROOT / (name + '.stderr')).write_bytes(result.stderr)
metadata = {'command': command, 'cwd': REPO, 'startedAt': started,
            'elapsedSeconds': round(time.monotonic() - clock, 3),
            'exitCode': result.returncode, 'stdoutBytes': len(result.stdout),
            'stderrBytes': len(result.stderr), 'subreaper': subreaper, 'adoptedDescendantsReaped': reaped}
(ROOT / (name + '.meta.json')).write_text(json.dumps(metadata, ensure_ascii=False, indent=2) + '\n')
print(json.dumps(metadata, ensure_ascii=False))
raise SystemExit(result.returncode)
