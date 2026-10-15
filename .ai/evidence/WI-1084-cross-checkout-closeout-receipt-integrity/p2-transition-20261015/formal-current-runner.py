import os, pathlib, subprocess, json, hashlib, time, signal, datetime
r=pathlib.Path('/workspace/ai-cockpit-wi1084-closeout-receipt-integrity')
d=r/'.ai/evidence/WI-1084-cross-checkout-closeout-receipt-integrity/p2-transition-20261015'
name='formal-current-full'
env=os.environ.copy()
env.update(PATH='/workspace/.cargo/bin:'+env['PATH'],RUSTUP_HOME='/workspace/.rustup',CARGO_HOME='/workspace/.cargo',HOME='/tmp/wi1084-runtime-home-20261015',CARGO_TARGET_DIR=str(r/'target/wi1084-closeout-receipt-integrity'),CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',CARGO_PROFILE_DEV_DEBUG='0',CARGO_PROFILE_TEST_DEBUG='0',RUST_TEST_THREADS='1',CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=str(r/'tests/ci/run_process_observer_test_runner.sh'))
b=r/'target/wi1084-closeout-receipt-integrity/debug/ai-cockpit'
cmd=[str(b),'verify','--repo',str(r),'--work-item','WI-1084-cross-checkout-closeout-receipt-integrity','--workers','1']
assert not (d/(name+'.execution.json')).exists(), 'Do not overwrite a formal run'
t=time.time()
m={'command':cmd,'sourceHead':subprocess.check_output(['git','rev-parse','HEAD'],cwd=r,text=True).strip(),'binarySha256':hashlib.sha256(b.read_bytes()).hexdigest(),'sourceDiffSha256':hashlib.sha256(subprocess.check_output(['git','diff','--binary'],cwd=r)).hexdigest(),'sourceFiles':json.loads((d/'07f-runtime-binding.json').read_bytes())['sourceFiles'],'environment':{k:env[k] for k in ['HOME','RUSTUP_HOME','CARGO_HOME','CARGO_TARGET_DIR','CARGO_BUILD_JOBS','CARGO_INCREMENTAL','CARGO_PROFILE_DEV_DEBUG','CARGO_PROFILE_TEST_DEBUG','RUST_TEST_THREADS','CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER']},'startedAtUTC':datetime.datetime.now(datetime.timezone.utc).isoformat(),'startedAtEpoch':t,'outerTimeoutSeconds':900,'perNodeTimeoutOverride':None}
def save(): (d/(name+'.execution.json')).write_text(json.dumps(m,indent=2)+'\n')
save()
with (d/(name+'.stdout')).open('xb') as out,(d/(name+'.stderr')).open('xb') as err:
    p=subprocess.Popen(cmd,cwd=r,env=env,stdout=out,stderr=err,start_new_session=True)
    m['ownedProcessGroup']=p.pid; save()
    try: m['exitCode']=p.wait(timeout=900)
    except subprocess.TimeoutExpired:
        m['timedOut']=True; m['exitCode']=124
        # Only the process group created above is subject to this timeout.
        os.killpg(p.pid,signal.SIGTERM)
        try: p.wait(timeout=5)
        except subprocess.TimeoutExpired: os.killpg(p.pid,signal.SIGKILL); p.wait()
        m['scopedTimeoutCleanup']='owned process group only'
m['elapsedSeconds']=time.time()-t; m['finishedAtUTC']=datetime.datetime.now(datetime.timezone.utc).isoformat(); save()
print(json.dumps({k:m[k] for k in ['sourceHead','binarySha256','exitCode','elapsedSeconds','finishedAtUTC']},indent=2))
raise SystemExit(m['exitCode'])
