#!/usr/bin/env python3
"""Actual native six-node run; preserves each input and a bounded terminal record."""
from pathlib import Path
import os,json,subprocess,time,signal,hashlib
from datetime import datetime,timezone
base=Path(__file__).parent.parent;root=Path('/workspace/ai-cockpit-wi1089-recovery');wid='WI-1089-blocked-outcome-material-review-recovery';binary=Path('/workspace/scratch/wi1087-fixed-runtime-b8ec269-20261018/target/release/ai-cockpit')
label='native-six-node.attempt1';assert not (base/(label+'.metadata.actual.json')).exists()
env=os.environ.copy();env.update(PATH='/workspace/.cloud-tools/cargo/bin:'+env['PATH'],CARGO_HOME='/workspace/.cloud-tools/cargo',RUSTUP_HOME='/workspace/.cloud-tools/rustup',CARGO_TARGET_DIR='/workspace/scratch/wi1089-preserved-history-verify-target-20261019',CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',CARGO_NET_OFFLINE='true',RUST_TEST_THREADS='1')
for suffix in ['contract','summary']:
 (base/('before-native-six-node.'+suffix+'.actual.json')).write_bytes((root/f'.ai/work-items/active/{wid}.{suffix}.json').read_bytes())
(base/'before-native-six-node.canonical.actual.json').write_bytes((root/f'.ai/evidence/{wid}.verification.json').read_bytes())
for name,args in [('status',['work-item','status','--id',wid,'--json']),('validate',['work-item','validate','--id',wid,'--json']),('plan',['verify','--work-item',wid,'--workers','1','--plan-only'])]:
 r=subprocess.run([str(binary),*args,'--repo',str(root)],env=env,cwd=root,capture_output=True,timeout=60);(base/('before-native-six-node.'+name+'.actual.json')).write_bytes(r.stdout);(base/('before-native-six-node.'+name+'.actual.stderr')).write_bytes(r.stderr);print(name,r.returncode,flush=True)
 if name=='status':s=json.loads(r.stdout);assert not s['humanDecisionRequired'] and 'run_verification' in s['safeActions']
 if name=='plan':s=json.loads(r.stdout);assert r.returncode==0 and s['processesSpawned']==0 and s['nodesPlanned']==6 and s['nodesToExecute']==6 and s['nodesReused']==0,s
args=[str(binary),'verify','--repo',str(root),'--work-item',wid,'--workers','1'];head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip();at=datetime.now(timezone.utc).isoformat();start=time.monotonic()
meta={'kind':'actual_native_six_node_verification_run','argv':args,'sourceHead':head,'runtimeSha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'startedAt':at,'terminalState':'running','workerCount':1,'targetDirectory':env['CARGO_TARGET_DIR']};mp=base/(label+'.metadata.actual.json');mp.write_text(json.dumps(meta,indent=2)+'\n')
with (base/(label+'.stdout.actual.json')).open('wb') as out,(base/(label+'.stderr.actual.log')).open('wb') as err:
 p=subprocess.Popen(args,env=env,cwd=root,stdout=out,stderr=err,start_new_session=True)
 try:code=p.wait(timeout=870)
 except subprocess.TimeoutExpired:
  os.killpg(p.pid,signal.SIGTERM)
  try:p.wait(timeout=5)
  except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait()
  code=124
meta.update(exitCode=code,endedAt=datetime.now(timezone.utc).isoformat(),elapsedSeconds=time.monotonic()-start,terminalState='completed',sourceHeadAfter=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip());mp.write_text(json.dumps(meta,indent=2)+'\n');print(json.dumps(meta));print((base/(label+'.stderr.actual.log')).read_text(errors='replace')[-1500:]);raise SystemExit(code)
