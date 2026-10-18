import base64,datetime,hashlib,json,os,pathlib,subprocess,sys,threading,time
root=pathlib.Path('/workspace/scratch/wi1088-runtime-continuity');repo=pathlib.Path('/workspace/ai-cockpit-wi1088-runtime-continuity');runtime=pathlib.Path('/workspace/scratch/wi1087-rebuild-20261017/runtime-target/release/ai-cockpit')
name=sys.argv[1];plan_name=sys.argv[2];path=root/'commands'/(name+'.json')
if path.exists():raise SystemExit('Refusing to overwrite '+name)
plan=json.loads((root/'commands'/(plan_name+'.stdout')).read_text())
assert plan['processesSpawned']==0 and plan['nodesToExecute']==len(plan['plannedNodes'])
assert all(0<n['timeoutSeconds']<=900 for n in plan['plannedNodes'])
env=os.environ.copy();env.update({'PATH':'/workspace/.cloud-tools/cargo/bin:'+env['PATH'],'CARGO_HOME':'/workspace/.cloud-tools/cargo','RUSTUP_HOME':'/workspace/.cloud-tools/rustup','CARGO_BUILD_JOBS':'1','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_TEST_DEBUG':'0','CARGO_TARGET_DIR':str(repo/'target/wi1088-runtime-continuity'),'RUST_TEST_THREADS':'1','CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER':'bash '+str(repo/'tests/ci/run_process_observer_test_runner.sh')})
command=[str(runtime),'verify','--work-item','WI-1088-runtime-activation-amendment-continuity-recovery','--workers','1','--repo',str(repo)]
record={'kind':'actual_native_full_declared_verification_observation_not_receipt','command':command,'cwd':str(repo),'sourceHead':subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),'runtimeRawSha256':hashlib.sha256(runtime.read_bytes()).hexdigest(),'startedAt':datetime.datetime.now(datetime.timezone.utc).isoformat(),'nativePlanObservation':plan_name+'.json','nativePlanRawSha256':hashlib.sha256((root/'commands'/(plan_name+'.stdout')).read_bytes()).hexdigest(),'plannedNodeCount':len(plan['plannedNodes']),'coordinatorOuterTimeoutSeconds':None,'timeoutPolicy':'Native individual node budgets govern; coordinator has no blanket 900-second cutoff.','processOnlyEnvironment':{k:env[k] for k in ['PATH','CARGO_HOME','RUSTUP_HOME','CARGO_BUILD_JOBS','CARGO_INCREMENTAL','CARGO_PROFILE_DEV_DEBUG','CARGO_PROFILE_TEST_DEBUG','CARGO_TARGET_DIR','RUST_TEST_THREADS','CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER']}}
path.write_text(json.dumps(record,indent=2)+'\n');started=time.monotonic()
print(json.dumps({'observation':name,'sourceHead':record['sourceHead'],'startedAt':record['startedAt'],'plannedNodes':record['plannedNodeCount'],'nodeBudgetMaxSeconds':max(n['timeoutSeconds'] for n in plan['plannedNodes']),'outerTimeoutSeconds':None}),flush=True)
with path.with_suffix('.stdout').open('wb') as out,path.with_suffix('.stderr').open('wb') as err:
 child=subprocess.Popen(command,cwd=repo,env=env,stdout=out,stderr=subprocess.PIPE)
 def stream():
  for line in iter(child.stderr.readline,b''):
   err.write(line);err.flush();print('native: '+line.decode(errors='replace').rstrip(),flush=True)
 observer=threading.Thread(target=stream);observer.start();code=child.wait();observer.join()
record.update({'exitCode':code,'elapsedSeconds':round(time.monotonic()-started,3),'endedAt':datetime.datetime.now(datetime.timezone.utc).isoformat()})
for key in ['stdout','stderr']:
 data=path.with_suffix('.'+key).read_bytes();record[key]={'encoding':'base64','bytes':len(data),'rawSha256':hashlib.sha256(data).hexdigest(),'data':base64.b64encode(data).decode()}
path.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'observation':name,'exitCode':code,'elapsedSeconds':record['elapsedSeconds'],'metadataPath':str(path)}),flush=True);raise SystemExit(code)
