import datetime
import ctypes
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

repo = Path('/workspace/ai-cockpit-wi1085-legacy-v1-closeout-source-registration')
evidence = repo / '.ai/evidence/WI-1085-legacy-v1-closeout-source-registration/implementation-20261015'
name, *command = sys.argv[1:]
if '/' in name or not command:
    raise SystemExit('Invalid bounded build invocation')
paths = {key: evidence / (name + '.' + key) for key in ['stdout', 'stderr', 'meta.json', 'resources.jsonl']}
if any(path.exists() for path in paths.values()):
    raise SystemExit('Refuse log overwrite')
env = os.environ.copy()
env.update({'PATH': '/workspace/.cargo/bin:' + env['PATH'],
            'RUSTUP_HOME': '/workspace/.rustup', 'CARGO_HOME': '/workspace/.cargo',
            'HOME': '/tmp/wi1084-runtime-home-20261015',
            'CARGO_TARGET_DIR': str(repo / 'target/wi1085-legacy-closeout-source'),
            'CARGO_BUILD_JOBS': '1', 'CARGO_INCREMENTAL': '0',
            'CARGO_PROFILE_DEV_DEBUG': '0', 'CARGO_PROFILE_TEST_DEBUG': '0',
            'RUST_TEST_THREADS': '1'})
subreaper = False
if sys.platform.startswith('linux'):
    libc = ctypes.CDLL(None, use_errno=True)
    if libc.prctl(36, 1, 0, 0, 0) != 0:
        raise SystemExit('Cannot enable owned-run descendant reaping: errno ' + str(ctypes.get_errno()))
    subreaper = True
started = datetime.datetime.now(datetime.timezone.utc).isoformat()
clock = time.monotonic()
stop = None
with paths['stdout'].open('wb') as stdout, paths['stderr'].open('wb') as stderr, paths['resources.jsonl'].open('w') as samples:
    process = subprocess.Popen(command, cwd=repo, env=env, stdout=stdout, stderr=stderr, start_new_session=True)
    while process.poll() is None:
        elapsed = time.monotonic() - clock
        free = os.statvfs('/workspace').f_bavail * os.statvfs('/workspace').f_frsize
        samples.write(json.dumps({'elapsedSeconds': round(elapsed, 3), 'workspaceAvailableBytes': free}) + '\n')
        samples.flush()
        if elapsed > 900 or free < 350_000_000:
            stop = 'timeout_900_seconds' if elapsed > 900 else 'disk_margin_below_350MB'
            os.killpg(process.pid, signal.SIGTERM)
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
            break
        time.sleep(2)
    code = process.wait()
adopted_reaped = []
if subreaper:
    deadline = time.monotonic() + 5
    while True:
        try:
            reaped_pid, reaped_status = os.waitpid(-1, os.WNOHANG)
        except ChildProcessError:
            break
        if reaped_pid:
            adopted_reaped.append({'pid': reaped_pid, 'waitStatus': reaped_status})
            continue
        if time.monotonic() >= deadline:
            break
        time.sleep(0.05)
metadata = {'subreaper': subreaper, 'adoptedDescendantsReaped': adopted_reaped, 'command': command, 'cwd': str(repo), 'startedAt': started,
            'elapsedSeconds': round(time.monotonic() - clock, 3), 'exitCode': code,
            'stopReason': stop, 'pid': process.pid, 'childReaped': True,
            'sourceHead': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=repo, text=True).strip(),
            'sourceDiffSha256': hashlib.sha256(subprocess.check_output(['git', 'diff', '--', 'crates', 'docs'], cwd=repo)).hexdigest(),
            'target': env['CARGO_TARGET_DIR'], 'jobs': 1}
paths['meta.json'].write_text(json.dumps(metadata, ensure_ascii=False, indent=2) + '\n')
print(json.dumps(metadata, ensure_ascii=False))
for key in ['stdout', 'stderr']:
    print(key + ' tail:\n' + paths[key].read_text(errors='replace')[-4500:])
raise SystemExit(code if code >= 0 else 1)
