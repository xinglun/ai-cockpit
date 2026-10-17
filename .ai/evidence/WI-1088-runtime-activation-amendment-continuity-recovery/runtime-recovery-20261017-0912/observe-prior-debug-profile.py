import base64
import datetime
import hashlib
import json
import os
import pathlib
import subprocess
import sys
import time

ROOT = pathlib.Path('/workspace/scratch/wi1088-runtime-continuity')
REPO = pathlib.Path('/workspace/ai-cockpit-wi1088-runtime-continuity')
RUNTIME = pathlib.Path('/workspace/scratch/wi1087-rebuild-20261017/runtime-target/release/ai-cockpit')
name, *arguments = sys.argv[1:]
path = ROOT / 'commands' / (name + '.json')
if path.exists():
    raise SystemExit('Refusing to overwrite observation ' + name)
env = os.environ.copy()
env.update({
    'PATH': '/workspace/.cloud-tools/cargo/bin:' + env['PATH'],
    'CARGO_HOME': '/workspace/.cloud-tools/cargo',
    'RUSTUP_HOME': '/workspace/.cloud-tools/rustup',
    'CARGO_BUILD_JOBS': '1',
    'CARGO_INCREMENTAL': '0',
    'CARGO_TARGET_DIR': str(REPO / 'target/wi1088-runtime-continuity'),
    'RUST_TEST_THREADS': '1',
    'CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER': 'bash ' + str(REPO / 'tests/ci/run_process_observer_test_runner.sh'),
})
env.pop('CARGO_PROFILE_DEV_DEBUG', None)
env.pop('CARGO_PROFILE_TEST_DEBUG', None)
command = [str(RUNTIME), *arguments, '--repo', str(REPO)]
record = {
    'kind': 'actual_current_environment_native_command_observation',
    'command': command, 'cwd': str(REPO),
    'startedAt': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'runtimeRawSha256': hashlib.sha256(RUNTIME.read_bytes()).hexdigest(),
    'sourceHead': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=REPO, text=True).strip(),
    'processOnlyEnvironment': {k:env.get(k) for k in ['PATH','CARGO_HOME','RUSTUP_HOME','CARGO_TARGET_DIR','CARGO_BUILD_JOBS','CARGO_INCREMENTAL','CARGO_PROFILE_DEV_DEBUG','CARGO_PROFILE_TEST_DEBUG','RUST_TEST_THREADS','CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER']},
}
path.write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n')
started = time.monotonic()
with path.with_suffix('.stdout').open('wb') as out, path.with_suffix('.stderr').open('wb') as err:
    p = subprocess.run(command, cwd=REPO, env=env, stdout=out, stderr=err, timeout=900)
record.update({'exitCode':p.returncode, 'elapsedSeconds':round(time.monotonic()-started,3), 'endedAt':datetime.datetime.now(datetime.timezone.utc).isoformat()})
for stream in ['stdout','stderr']:
    data = path.with_suffix('.'+stream).read_bytes()
    record[stream] = {'bytes':len(data),'rawSha256':hashlib.sha256(data).hexdigest(),'encoding':'base64','data':base64.b64encode(data).decode('ascii')}
path.write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({'observation':name,'exitCode':p.returncode,'elapsedSeconds':record['elapsedSeconds'],'metadataPath':str(path)}, ensure_ascii=False))
if p.returncode:
    print(path.with_suffix('.stderr').read_text(errors='replace')[:5000])
sys.exit(p.returncode)
