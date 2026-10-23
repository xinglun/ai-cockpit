from pathlib import Path
from datetime import datetime,timezone
import subprocess,json,hashlib
root=Path('/workspace/ai-cockpit-wi1090-material-replacement');item='WI-1090-authority-source-material-review-replacement';out=Path('/workspace/scratch/wi1090-intent-admission-20261019');r=Path('/workspace/scratch/wi1087-fixed-runtime-b8ec269-20261018/target/release/ai-cockpit')
assert hashlib.sha256(r.read_bytes()).hexdigest()=='b5cbd8570df5d3cb2004ae517fe88df6136c66d92fc1913070d18f2c8f67f257'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()=='78ffc397a1d2de5907e93fddafecbad22290474b';assert subprocess.check_output(['git','status','--porcelain'],cwd=root)==b''
payload=root/f'.ai/evidence/{item}/new-contract-confirm-material-20261019/intent-alignment.controls.PROPOSAL-ONLY.json';assert hashlib.sha256(payload.read_bytes()).hexdigest()=='5c30d3b5083c6e3d5e8a40658dd5e1082307ad2e097d124628bfd621ff087b8d'
def capture(label,args):
 p=subprocess.run([str(r),*args,'--repo',str(root)],capture_output=True,timeout=90)
 for suffix,data in [('stdout.actual.json',p.stdout),('stderr.actual.txt',p.stderr)]:
  f=out/(label+'.'+suffix);assert not f.exists();f.write_bytes(data)
 assert p.returncode==0,(label,p.stderr.decode())
 return json.loads(p.stdout)
s=capture('before-intent-status',['work-item','status','--id',item,'--json']);plan=capture('before-intent-material-plan',['work-item','material-review','plan','--id',item])
assert not s['blocking'] and not s['humanDecisionRequired'] and 'record_governance_controls' in s['safeActions'];assert s['verification']=='verified' and s['evidenceFreshness']['state']=='fresh'
assert s['sourceDigests']['contract']=='sha256:b6d2f736ba2c9872294578dea6e6c7dfae528f0c46c33ce603599d31b73a5214' and plan['requestDigest']=='sha256:c5ddefb60c57f35b2f1da6cbc4a4268cbf1039aa9bfe15b961a764d9dd23dae7'
old=json.loads((root/f'.ai/evidence/{item}/new-contract-confirm-material-20261019/final-new-contract-material-plan.stdout.actual.json').read_text())
for p in [plan,old]:p.pop('reviewedSourceHead')
assert plan==old
for e in plan['entries']:assert 'sha256:'+hashlib.sha256((root/e['path']).read_bytes()).hexdigest()==e['afterBlobDigest']
summary_path=root/f'.ai/work-items/active/{item}.summary.json';(out/'before-intent.summary.actual.json').write_bytes(summary_path.read_bytes())
canonical=root/f'.ai/evidence/{item}.verification.json';canonical_hash=hashlib.sha256(canonical.read_bytes()).hexdigest();assert canonical_hash=='f5d328a96bb623aa675547fe2a6943a47217215d1a7c9394c7e784f9ce595821'
meta={'recordedBy':'agent:Codex','startedAt':datetime.now(timezone.utc).isoformat(),'terminal':'running','inputRawSha256':'5c30d3b5083c6e3d5e8a40658dd5e1082307ad2e097d124628bfd621ff087b8d','authorization':'Raydot independently accepted exact78ffc397 technical evidence and bounded intent; expressly authorized only this exact intentAlignment control. No terminal call authorized by this instruction.'}
marker=out/'intent-controls.attempt1.metadata.actual.json';assert not marker.exists();marker.write_text(json.dumps(meta,ensure_ascii=False,indent=2)+'\n')
p=subprocess.run([str(r),'work-item','controls','--repo',str(root),'--id',item,'--input',str(payload)],capture_output=True,timeout=90)
(out/'intent-controls.attempt1.stdout.actual.json').write_bytes(p.stdout);(out/'intent-controls.attempt1.stderr.actual.txt').write_bytes(p.stderr);meta.update(terminal='returned',exitCode=p.returncode,endedAt=datetime.now(timezone.utc).isoformat());marker.write_text(json.dumps(meta,ensure_ascii=False,indent=2)+'\n');assert p.returncode==0,p.stderr
actual=json.loads(summary_path.read_text());assert actual['intentAlignment']==json.loads(payload.read_text())['intentAlignment'] and actual['intentAlignment']['state']=='resolved';assert hashlib.sha256(canonical.read_bytes()).hexdigest()==canonical_hash
(out/'after-intent.summary.actual.json').write_bytes(summary_path.read_bytes());after=capture('after-intent-status',['work-item','status','--id',item,'--json']);assert after['effectiveUnknowns']==['user_visible_benefit_not_declared'];assert after['verification']=='verified' and after['evidenceFreshness']['state']=='fresh'
print(json.dumps({'intentAlignment':actual['intentAlignment'],'nativeStatus':after},ensure_ascii=False)[:11000])
