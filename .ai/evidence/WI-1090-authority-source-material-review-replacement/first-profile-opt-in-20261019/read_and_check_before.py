from pathlib import Path
import subprocess,json,hashlib,re,datetime
root=Path('/workspace/ai-cockpit-wi1090-material-replacement');item='WI-1090-authority-source-material-review-replacement';out=Path('/workspace/scratch/wi1090-first-profile-opt-in-20261019');r=Path('/workspace/scratch/wi1087-fixed-runtime-b8ec269-20261018/target/release/ai-cockpit')
assert hashlib.sha256(r.read_bytes()).hexdigest()=='b5cbd8570df5d3cb2004ae517fe88df6136c66d92fc1913070d18f2c8f67f257'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()=='b1fecfc35e14f700f146a51d4cc494590b13919c'
assert subprocess.check_output(['git','status','--porcelain'],cwd=root)==b''
def capture(label,args):
 p=subprocess.run([str(r),*args,'--repo',str(root)],capture_output=True,timeout=90)
 for suffix,data in [('stdout.actual.json',p.stdout),('stderr.actual.txt',p.stderr)]:
  d=out/(label+'.'+suffix);assert not d.exists();d.write_bytes(data)
 assert p.returncode==0,(label,p.stderr.decode())
 return json.loads(p.stdout)
inspect=capture('before-inspect',['inspect'])
status=capture('before-status',['work-item','status','--id',item,'--json'])
agent=capture('before-agent-doctor',['agent','doctor','--json'])
doctor=capture('before-doctor',['doctor'])
coord=capture('before-coordination-inspect',['work-item','coordination','inspect'])
history=capture('before-amendments',['work-item','amendments','--id',item])
plan=capture('before-material-plan',['work-item','material-review','plan','--id',item])
assert not status['blocking'] and not status['humanDecisionRequired']
assert status['sourceDigests']['contract']=='sha256:c9a3bda349078f1fc15891e9e789d100952cbadaea12eb464f2024c90617ce9c'
assert plan['requestDigest']=='sha256:23e04c983732d47799e1ec444849fd89e0c92f3a5b13742ac636be30c8b6e3d8'
prior=json.loads((root/f'.ai/evidence/{item}/minimal-negative-matrix-20261019/final-material-plan.stdout.actual.json').read_text())
for v in [prior,plan]:v.pop('reviewedSourceHead')
assert plan==prior
proposal=root/f'.ai/evidence/{item}/scenario-mapping-20261019/first-profile-opt-in.amend.request.proposal-only.json'
assert hashlib.sha256(proposal.read_bytes()).hexdigest()=='ff93edd6797c0c6e973e1e3355024d84e96d4ca8d616604b37eec859785f323f'
request=json.loads(proposal.read_text());assert len(request['changes'])==1 and request['changes'][0]['path']=='/governanceProfile'
assert not (root/f'.ai/evidence/{item}/material-risk-consent/current-risk-consent.agent-recorded.json').exists()
assert len(plan['entries'])==8 and not plan['findingCodes'] and plan['rawUnknownCodes']==['repository_material_inspection_unavailable']
checks=[]
for e in plan['entries']:
 path=e['path'];blob=subprocess.check_output(['git','show','HEAD:'+path],cwd=root)
 blob_digest='sha256:'+hashlib.sha256(blob).hexdigest();assert blob_digest==e['afterBlobDigest']
 patch=subprocess.check_output(['git','diff','--no-ext-diff','--no-textconv','--no-renames','--unified=0',plan['immutableContractBaseRevision'],'HEAD','--',path],cwd=root).decode()
 added=[];removed=[];origins=[];hunk=-1;after_line=None
 for line in patch.splitlines():
  if line.startswith('@@ '):
   m=re.match(r'@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@',line);assert m
   after_line=int(m[1]);hunk+=1
  elif after_line is not None:
   if line.startswith('+'):added.append(line[1:]);origins.append([after_line,hunk]);after_line+=1
   elif line.startswith('-'):removed.append(line[1:])
   elif line.startswith(' '):after_line+=1
 d='sha256:'+hashlib.sha256(json.dumps([added,origins,removed],ensure_ascii=False,separators=(',',':')).encode()).hexdigest();assert d==e['changedHunkDigest'],path
 checks.append({'path':path,'afterBlobDigest':blob_digest,'changedHunkDigest':d,'scannerAssessment':e['scannerAssessment'],'reviewable':e['reviewable'],'unknownCause':e['unknownCause']})
for suffix in ['contract','summary']:
 (out/f'before.{suffix}.actual.json').write_bytes((root/f'.ai/work-items/active/{item}.{suffix}.json').read_bytes())
canonical=root/f'.ai/evidence/{item}.verification.json';(out/'before.verification.canonical.actual.json').write_bytes(canonical.read_bytes())
manifest=[]
for p in sorted((root/'.ai').rglob('*')):
 if p.is_file() and not p.is_symlink():manifest.append({'path':str(p.relative_to(root)),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()})
(out/'before-governance-raw-inventory.private.actual.json').write_text(json.dumps(manifest,indent=2)+'\n')
report={'recordedBy':'agent:Codex','recordedAt':datetime.datetime.now(datetime.timezone.utc).isoformat(),'HEAD':'b1fecfc35e14f700f146a51d4cc494590b13919c','proposalRawSha256':hashlib.sha256(proposal.read_bytes()).hexdigest(),'nativeStatus':{k:status[k] for k in ['lifecyclePhase','governanceState','blocking','humanDecisionRequired','safeActions','sourceDigests']},'currentPlanSameExceptHead':True,'independentBlobHunkChecks':checks,'scannerIdentities':{k:plan[k] for k in ['scannerSemanticVersion','analysisImplementationDigest','analysisTargetSemanticProfile']},'findingCodes':plan['findingCodes'],'rawUnknownCodes':plan['rawUnknownCodes'],'coordinationKeys':list(coord),'inspectKeys':list(inspect),'agentDoctor':agent,'doctorState':doctor.get('state'),'amendmentsType':str(type(history)),'approvedNewContractReview':False}
(out/'before-conditions.actual.agent-recorded.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k not in ['independentBlobHunkChecks','agentDoctor']},ensure_ascii=False))
print('agentDoctor',json.dumps(agent,ensure_ascii=False)[:1800]);print('coordination',json.dumps(coord,ensure_ascii=False)[:4500]);print('amendment_count',len(history) if isinstance(history,list) else list(history))
