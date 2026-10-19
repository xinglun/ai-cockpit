from pathlib import Path
import subprocess,json,datetime,hashlib
root=Path('/workspace/ai-cockpit-wi1090-material-replacement');item='WI-1090-authority-source-material-review-replacement';out=Path('/workspace/scratch/wi1090-first-profile-opt-in-20261019');r='/workspace/scratch/wi1087-fixed-runtime-b8ec269-20261018/target/release/ai-cockpit'
def capture(label,args):
 p=subprocess.run([r,*args,'--repo',str(root)],capture_output=True,timeout=90)
 for suffix,data in [('stdout.actual.json',p.stdout),('stderr.actual.txt',p.stderr)]:
  d=out/(label+'.'+suffix);assert not d.exists();d.write_bytes(data)
 assert p.returncode==0,(label,p.stderr.decode())
 return json.loads(p.stdout)
status=capture('after-amend-before-preflight-status',['work-item','status','--id',item,'--json'])
plan=capture('after-amend-material-plan',['work-item','material-review','plan','--id',item])
before=json.loads((out/'before.contract.actual.json').read_text());actual=json.loads((root/f'.ai/work-items/active/{item}.contract.json').read_text());proposal=json.loads((root/f'.ai/evidence/{item}/scenario-mapping-20261019/first-profile-opt-in.amend.request.proposal-only.json').read_text())
assert actual['governanceProfile']==proposal['changes'][0]['value']
assert sorted(actual['requiredRuntimeCapabilities'])==sorted(before['requiredRuntimeCapabilities']+['material-inspection-review'])
diff=[k for k in set(before)|set(actual) if before.get(k)!=actual.get(k)];assert set(diff)=={'governanceProfile','requiredRuntimeCapabilities'},diff
old=json.loads((out/'before-material-plan.stdout.actual.json').read_text())
for k in ['entries','sourceSnapshotDigest','materialManifestDigest','immutableContractBaseRevision','scannerSemanticVersion','analysisImplementationDigest','analysisTargetSemanticProfile','rawUnknownCodes','findingCodes','blockedByFinding']:
 assert plan[k]==old[k],k
assert plan['reviewEnabled'] and len(plan['entries'])==8 and len([e for e in plan['entries'] if e['scannerAssessment']=='unknown'])==1
assert status['sourceDigests']['contract']=='sha256:b6d2f736ba2c9872294578dea6e6c7dfae528f0c46c33ce603599d31b73a5214'
assert 'run_preflight' in status['safeActions'],status
(out/'after-amend.contract.actual.json').write_bytes((root/f'.ai/work-items/active/{item}.contract.json').read_bytes())
(out/'after-amend.before-preflight.summary.actual.json').write_bytes((root/f'.ai/work-items/active/{item}.summary.json').read_bytes())
print('preflight admission',json.dumps({k:status[k] for k in ['blocking','humanDecisionRequired','safeActions','actionExplanation']},ensure_ascii=False))
preflight=capture('new-contract-preflight',['preflight','--contract',str(root/f'.ai/work-items/active/{item}.contract.json')])
status=capture('after-preflight-status',['work-item','status','--id',item,'--json'])
summary=json.loads((root/f'.ai/work-items/active/{item}.summary.json').read_text());(out/'after-preflight.summary.actual.json').write_bytes((root/f'.ai/work-items/active/{item}.summary.json').read_bytes())
report={'recordedBy':'agent:Codex','recordedAt':datetime.datetime.now(datetime.timezone.utc).isoformat(),'conditionResult':'all approved source/scanner/policy conditions matched','changedContractRoots':diff,'onlyRequestedProfilePlusNativeDerivedCapability':True,'eightPlanEntriesIdentical':True,'sourceSnapshotUnchanged':True,'manifestUnchanged':True,'scannerIdentitiesUnchanged':True,'findingCodes':plan['findingCodes'],'rawUnknownCodes':plan['rawUnknownCodes'],'actualContractDigest':status['sourceDigests']['contract'],'actualMaterialRequestDigest':plan['requestDigest'],'actualMaterialProfileDigest':plan['materialInspectionReviewProfileDigest'],'actualEffectivePolicyDigest':plan['effectivePolicyDigest'],'preflightStatus':{k:status[k] for k in ['lifecyclePhase','governanceState','blocking','humanDecisionRequired','safeActions','humanDecisions','actionExplanation','sourceDigests']},'consentFileCreated':False,'newContractHumanReviewRecorded':False,'materialRecordInvoked':False}
(out/'post-amend-conditions-and-review-boundary.actual.agent-recorded.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
print('preflight result',json.dumps(preflight,ensure_ascii=False)[:7000]);print('post status',json.dumps(report,ensure_ascii=False)[:7500]);print('summary decision fields',json.dumps({k:v for k,v in summary.items() if 'decision' in k.lower() or 'preflight' in k.lower()},ensure_ascii=False)[:7000])
