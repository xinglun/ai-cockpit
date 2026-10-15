import pathlib, os, json, hashlib, subprocess, time, datetime, sys
r=pathlib.Path('/workspace/ai-cockpit-wi1084-closeout-receipt-integrity')
w='WI-1084-cross-checkout-closeout-receipt-integrity'
d=r/'.ai/evidence'/w/'post-close-20261015'
d.mkdir(exist_ok=True)
env=os.environ.copy()
env.update(PATH='/workspace/.cargo/bin:'+env['PATH'],RUSTUP_HOME='/workspace/.rustup',CARGO_HOME='/workspace/.cargo',HOME='/tmp/wi1084-runtime-home-20261015',CARGO_TARGET_DIR=str(r/'target/wi1084-closeout-receipt-integrity'),CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',CARGO_PROFILE_DEV_DEBUG='0',CARGO_PROFILE_TEST_DEBUG='0',RUST_TEST_THREADS='1',CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=str(r/'tests/ci/run_process_observer_test_runner.sh'))
def storage():
 rows=[]
 for p in [r,pathlib.Path('/tmp'),r/'target/wi1084-closeout-receipt-integrity',pathlib.Path('/workspace/.cargo'),pathlib.Path('/workspace/.rustup'),pathlib.Path(env['HOME'])]:
  v=os.statvfs(p);rows.append({'path':str(p),'device':p.stat().st_dev,'availableBytes':v.f_bavail*v.f_frsize,'availableInodes':v.f_favail})
 return rows
def save(p,x):p.write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
name=sys.argv[1]
b=r/'target/wi1084-closeout-receipt-integrity/debug/ai-cockpit'
assert hashlib.sha256(b.read_bytes()).hexdigest()=='5307ba5324d397e0fb971aa8b926ba8176336a0a6e76dfae7bd3a6e5cd11fa38'
cmd=[str(b),*sys.argv[2:]]
assert not (d/(name+'.execution.json')).exists(), 'Do not overwrite previous command evidence'
meta={'command':cmd,'recordedAtUTC':datetime.datetime.now(datetime.timezone.utc).isoformat(),'sourceHead':subprocess.check_output(['git','rev-parse','HEAD'],cwd=r,text=True).strip(),'runtimeSha256':hashlib.sha256(b.read_bytes()).hexdigest(),'storageBefore':storage(),'environment':{k:env[k] for k in ['HOME','RUSTUP_HOME','CARGO_HOME','CARGO_TARGET_DIR','CARGO_BUILD_JOBS','CARGO_INCREMENTAL','RUST_TEST_THREADS']},'timeoutSeconds':900}
save(d/(name+'.execution.json'),meta);t=time.time()
q=subprocess.run(cmd,cwd=r,env=env,capture_output=True,timeout=900)
(d/(name+'.stdout')).write_bytes(q.stdout);(d/(name+'.stderr')).write_bytes(q.stderr)
meta.update(exitCode=q.returncode,elapsedSeconds=time.time()-t,storageAfter=storage());save(d/(name+'.execution.json'),meta)
print(name,'exit',q.returncode,'elapsed',round(meta['elapsedSeconds'],2));print(q.stderr.decode(errors='replace')[-1800:]);raise SystemExit(q.returncode)
