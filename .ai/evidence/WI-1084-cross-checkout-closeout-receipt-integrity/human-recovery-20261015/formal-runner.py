import os, pathlib, subprocess, json, hashlib, time, signal, datetime
r = pathlib.Path('/workspace/ai-cockpit-wi1084-closeout-receipt-integrity')
w = 'WI-1084-cross-checkout-closeout-receipt-integrity'
d = r / '.ai/evidence' / w / 'human-recovery-20261015'
name = 'new-contract-formal-full'
env = os.environ.copy()
env.update(PATH='/workspace/.cargo/bin:' + env['PATH'], RUSTUP_HOME='/workspace/.rustup', CARGO_HOME='/workspace/.cargo', HOME='/tmp/wi1084-runtime-home-20261015', CARGO_TARGET_DIR=str(r / 'target/wi1084-closeout-receipt-integrity'), CARGO_BUILD_JOBS='1', CARGO_INCREMENTAL='0', CARGO_PROFILE_DEV_DEBUG='0', CARGO_PROFILE_TEST_DEBUG='0', RUST_TEST_THREADS='1', CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=str(r / 'tests/ci/run_process_observer_test_runner.sh'))
def now(): return datetime.datetime.now(datetime.timezone.utc).isoformat()
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def storage():
    rows = []
    for p in [r, pathlib.Path('/tmp'), r / 'target/wi1084-closeout-receipt-integrity', pathlib.Path('/workspace/.cargo'), pathlib.Path('/workspace/.rustup'), pathlib.Path(env['HOME'])]:
        v = os.statvfs(p)
        rows.append(dict(path=str(p), device=p.stat().st_dev, availableBytes=v.f_bavail*v.f_frsize, availableInodes=v.f_favail))
    return dict(recordedAtUTC=now(), filesystems=rows)
def save(): (d / (name + '.execution.json')).write_text(json.dumps(m, indent=2) + '\n')
b = r / 'target/wi1084-closeout-receipt-integrity/debug/ai-cockpit'
assert digest(b) == '5307ba5324d397e0fb971aa8b926ba8176336a0a6e76dfae7bd3a6e5cd11fa38'
assert not (d / (name + '.execution.json')).exists(), 'Do not overwrite a formal run'
status = json.loads((d / 'after-confirm-status.stdout').read_text())
assert 'run_verification' in status['safeActions'] and not status['humanDecisionRequired']
assert status['sourceDigests']['contract'] == 'sha256:6cdd832a40672ea8b73aad731b22a5ec1a6dc6e3737c7046b442695274253159'
source_files = json.loads((d.parent / 'p2-transition-20261015/07f-runtime-binding.json').read_text())['sourceFiles']
for path, expected in source_files.items(): assert digest(r / path) == expected, path
receipt = r / '.ai/evidence' / (w + '.verification.json')
assert digest(receipt) == '3e50cfff0224aaf40694b75be5d315364dcdf37ee1a0ba15913cd05ca9a19554'
(d / 'historical-original-contract.verification.json').write_bytes(receipt.read_bytes())
before = storage()
assert before['filesystems'][0]['availableBytes'] > 1024**3
assert before['filesystems'][1]['availableBytes'] > 2*1024**3
assert min(x['availableInodes'] for x in before['filesystems']) > 100000
cmd = [str(b), 'verify', '--repo', str(r), '--work-item', w, '--workers', '1']
t = time.time()
m = dict(command=cmd, sourceHead=subprocess.check_output(['git','rev-parse','HEAD'], cwd=r, text=True).strip(), binarySha256=digest(b), sourceFiles=source_files, contractDigest=status['sourceDigests']['contract'], repositorySnapshotDigest=status['snapshotDigest'], environment={k:env[k] for k in ['HOME','RUSTUP_HOME','CARGO_HOME','CARGO_TARGET_DIR','CARGO_BUILD_JOBS','CARGO_INCREMENTAL','CARGO_PROFILE_DEV_DEBUG','CARGO_PROFILE_TEST_DEBUG','RUST_TEST_THREADS','CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER']}, startedAtUTC=now(), startedAtEpoch=t, outerTimeoutSeconds=900, perNodeTimeoutOverride=None, storageBefore=before, storageSamples=[], storageEstimate='Warm target; prior run sampled roughly 1 GiB temporary growth. Require 2 GiB temporary headroom and 1 GiB workspace headroom before execution. Stop owned process group if available bytes fall below 512 MiB or inodes below 10000. No extra deletion authorized.', historicalReceiptCopy=str((d/'historical-original-contract.verification.json').relative_to(r)))
save()
def terminate_owned(p):
    os.killpg(p.pid, signal.SIGTERM)
    try: p.wait(timeout=5)
    except subprocess.TimeoutExpired: os.killpg(p.pid, signal.SIGKILL); p.wait()
    m['scopedTimeoutCleanup'] = 'owned process group only; waited for Runtime process'
with (d / (name + '.stdout')).open('xb') as out, (d / (name + '.stderr')).open('xb') as err:
    p = subprocess.Popen(cmd, cwd=r, env=env, stdout=out, stderr=err, start_new_session=True)
    m['ownedProcessGroup'] = p.pid
    save()
    while True:
        try:
            m['exitCode'] = p.wait(timeout=min(30, max(0.1, 900 - (time.time()-t))))
            break
        except subprocess.TimeoutExpired:
            sample = storage(); m['storageSamples'].append(sample); save()
            if any(x['availableBytes'] < 512*1024**2 or x['availableInodes'] < 10000 for x in sample['filesystems']):
                m['stoppedForResourceMargin'] = True; m['exitCode'] = 125; terminate_owned(p); break
            if time.time()-t >= 900:
                m['timedOut'] = True; m['exitCode'] = 124; terminate_owned(p); break
m.update(elapsedSeconds=time.time()-t, finishedAtUTC=now(), storageAfter=storage())
for path, expected in source_files.items(): assert digest(r/path) == expected, path
m['sourceFilesUnchangedAfter'] = True
save()
print(json.dumps({k:m[k] for k in ['sourceHead','binarySha256','contractDigest','exitCode','elapsedSeconds','finishedAtUTC']}, indent=2))
raise SystemExit(m['exitCode'])
