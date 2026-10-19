# Resume only current Native-admitted lifecycle actions; preserve refused preflight and do not repeat it.
from pathlib import Path
from datetime import datetime,timezone
import json,subprocess,hashlib,os,shutil
root=Path('/workspace/ai-cockpit-wi1090-material-replacement');item='WI-1090-authority-source-material-review-replacement';out=Path('/workspace/scratch/wi1090-own-lifecycle-completion-20261019');publish=root/f'.ai/evidence/{item}/own-lifecycle-completion-20261019';runtime=Path('/workspace/scratch/wi1087-fixed-runtime-b8ec269-20261018/target/release/ai-cockpit');formal=Path('/workspace/scratch/wi1090-v4-implementation-20261019')
assert not (out/'native-finish.attempt1.metadata.actual.json').exists();assert not (out/'native-archive.attempt1.metadata.actual.json').exists()
meta=json.loads((formal/'native-seven.attempt5.metadata.actual.json').read_text());assert meta['terminal']=='returned' and meta['exitCode']==0 and meta['contractDigest']=='sha256:1cc7ec218c2033ed48feb6b1ac298b392485dc898dcefac58f0fee14d06895c4'
assert meta['sourceHead']==meta['sourceHeadAfter']==subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip();assert hashlib.sha256(runtime.read_bytes()).hexdigest()=='b5cbd8570df5d3cb2004ae517fe88df6136c66d92fc1913070d18f2c8f67f257';assert not os.environ.get('AI_COCKPIT_OUTCOME_HOST_PROGRAM'),'Do not trigger an unreviewed external host adapter'
canonical=root/f'.ai/evidence/{item}.verification.json';raw=canonical.read_bytes();envelope=json.loads(raw);assert envelope['passed'] and envelope['contractDigest']==meta['contractDigest'];assert envelope['receipt']['nodesExecuted']==7 and envelope['receipt']['nodesReused']==0 and envelope['receipt']['maxConcurrentProcesses']==1
contract_path=root/f'.ai/work-items/active/{item}.contract.json';summary_path=root/f'.ai/work-items/active/{item}.summary.json';contract_raw=contract_path.read_bytes();contract=json.loads(contract_raw)
expected=json.loads((root/f'.ai/evidence/{item}/own-lifecycle-scope-20261019/after-scope.contract.actual.json').read_text());assert contract==expected
reference_plan=json.loads((out/'before-same-profile-renewal-plan.stdout.actual.json').read_text())
for e in reference_plan['entries']:assert 'sha256:'+hashlib.sha256((root/e['path']).read_bytes()).hexdigest()==e['afterBlobDigest']
protected=json.loads((out/'protected-prior-history.actual.agent-recorded.json').read_text())
def check_protected():
 for e in protected:assert hashlib.sha256((root/e['path']).read_bytes()).hexdigest()==e['rawSha256'],e['path']
check_protected()
def copy_new():
 for f in out.iterdir():
  if f.is_file() and not (publish/f.name).exists():shutil.copyfile(f,publish/f.name)
def call(label,args,mutating=False,timeout=90,human=False):
 stdout=out/(label+('.stdout.actual.txt' if human else '.stdout.actual.json'));assert not stdout.exists(),label
 marker=out/(label+'.metadata.actual.json');assert not marker.exists(),label
 argv=[str(runtime),*args,'--repo',str(root)];m={'recordedBy':'agent:Codex','startedAt':datetime.now(timezone.utc).isoformat(),'terminal':'running','argv':argv,'mutating':mutating,'singleAttempt':True,'timeoutSeconds':timeout};marker.write_text(json.dumps(m,ensure_ascii=False,indent=2)+'\n')
 try:
  p=subprocess.run(argv,capture_output=True,timeout=timeout)
 except subprocess.TimeoutExpired as error:
  stdout.write_bytes(error.stdout or b'');(out/(label+'.stderr.actual.txt')).write_bytes(error.stderr or b'');m.update(terminal='timeout_native_final_state_unconfirmed_no_replay',endedAt=datetime.now(timezone.utc).isoformat());marker.write_text(json.dumps(m,ensure_ascii=False,indent=2)+'\n');copy_new();print('NATIVE_ACTION_TIMEOUT_NO_REPLAY',json.dumps(m,ensure_ascii=False),flush=True);raise
 stdout.write_bytes(p.stdout);(out/(label+'.stderr.actual.txt')).write_bytes(p.stderr);m.update(terminal='returned',exitCode=p.returncode,endedAt=datetime.now(timezone.utc).isoformat());marker.write_text(json.dumps(m,ensure_ascii=False,indent=2)+'\n');copy_new()
 if p.returncode:
  print('NATIVE_ACTION_RETURNED_ERROR',json.dumps(m,ensure_ascii=False),p.stderr.decode(errors='replace'),flush=True);raise SystemExit(p.returncode)
 print(label+' exit0',flush=True)
 return p.stdout.decode() if human else json.loads(p.stdout)
refusal=json.loads((out/'final-fresh-preflight.attempt1.metadata.actual.json').read_text())
assert refusal['terminal']=='returned' and refusal['exitCode']==1
assert 'requested action "run_preflight"' in (out/'final-fresh-preflight.attempt1.stderr.actual.txt').read_text()
status=call('resume-current-finish-admission',['work-item','status','--id',item,'--json'])
current_summary=json.loads(summary_path.read_text())
current_preflight_binding={k:current_summary[k] for k in ['preflightState','preflightAt','preflightContractDigest','preflightRepositorySnapshotDigest','preflightDecisionDigest']}
assert current_preflight_binding['preflightState']=='green'
assert current_preflight_binding['preflightContractDigest']==meta['contractDigest']
assert current_preflight_binding['preflightRepositorySnapshotDigest']==status['snapshotDigest']
assert status['verification']=='verified' and status['evidenceFreshness']['state']=='fresh'
assert status['reviewReceiptDigest']=='sha256:b09f40987129791c62f86a9a73c0d7b5aaf7a0421bf0eca364121e1796357449'
assert 'finish' in status['safeActions'] and status['actionExplanation']['admissionState']=='allowed'
assert not status['blocking'] and not status['humanDecisionRequired']
assert status['effectiveUnknowns']==['user_visible_benefit_not_declared']
(out/'resume-current-preflight-binding.actual.agent-recorded.json').write_text(json.dumps({'recordedBy':'agent:Codex','actualPersistedNativeBinding':current_preflight_binding,'freshness':'matches current Contract/snapshot and fresh Native evidence; Native explicitly admits finish','independentPreflightCallWasRefused':True,'independentPreflightCallNotRetried':True},ensure_ascii=False,indent=2)+'\n')
material=call('fresh-before-finish-material-plan',['work-item','material-review','plan','--id',item]);assert material['requestDigest']==reference_plan['requestDigest']
for key in ['entries','sourceSnapshotDigest','materialManifestDigest','scannerSemanticVersion','analysisImplementationDigest','analysisTargetSemanticProfile','materialInspectionReviewProfileDigest','effectivePolicyDigest']:assert material[key]==reference_plan[key],key
(out/'before-finish.contract.actual.json').write_bytes(contract_path.read_bytes());(out/'before-finish.summary.actual.json').write_bytes(summary_path.read_bytes());(out/'current-1cc7-verification.before-finish.actual.json').write_bytes(raw)
finish=call('native-finish.attempt1',['finish','--id',item,'--language','zh-CN'],mutating=True,timeout=120)
after_finish=call('after-finish-before-archive-status',['work-item','status','--id',item,'--json'])
assert after_finish['lifecyclePhase']=='finish_ready';assert 'archive_when_reviewed' in after_finish['safeActions'] and not after_finish['blocking'] and not after_finish['humanDecisionRequired'];assert after_finish['verification']=='verified' and after_finish['evidenceFreshness']['state']=='fresh'
(out/'actual-finish-ready.summary.actual.json').write_bytes(summary_path.read_bytes());(out/'actual-finish-ready.outcome.actual.json').write_bytes((root/f'.ai/work-items/active/{item}.outcome.json').read_bytes());assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()==meta['sourceHead'];check_protected()
# No commit, test rerun, source edit, Runtime switch or close between these
# two individually admitted lifecycle actions.
archive=call('native-archive.attempt1',['archive','--id',item,'--json','--language','zh-CN'],mutating=True,timeout=180)
assert 'outcomeDelivery' in archive and 'humanHandoff' in archive and 'Outcome:' in json.dumps(archive,ensure_ascii=False)
after_archive=call('after-archive-status',['work-item','status','--id',item,'--json'])
assert after_archive['lifecyclePhase']=='archived',after_archive
assert not contract_path.exists() and not summary_path.exists();archived=root/'.ai/work-items/archive';assert (archived/f'{item}.archive.json').is_file();assert (archived/f'{item}.contract.json').is_file()
for suffix in ['archive','contract','summary','outcome','task-report']:
 p=archived/f'{item}.{suffix}.json'
 if p.is_file():(out/f'archived-{suffix}.native.actual.json').write_bytes(p.read_bytes())
md=archived/f'{item}.task-report.md'
if md.is_file():(out/'archived-task-report.native.actual.md').write_bytes(md.read_bytes())
events=archived/f'{item}.events.jsonl'
if events.is_file():(out/'archived-events.native.actual.jsonl').write_bytes(events.read_bytes())
check_protected();assert canonical.read_bytes()==raw
for e in reference_plan['entries']:assert 'sha256:'+hashlib.sha256((root/e['path']).read_bytes()).hexdigest()==e['afterBlobDigest']
full=call('archived-human-outcome-full.zh-CN',['work-item','outcome','--id',item,'--view','full','--language','zh-CN'],human=True)
delivery=call('archived-outcome-delivery',['work-item','outcome','--id',item,'--delivery','--json','--language','zh-CN'])
assert full.startswith('Outcome:') and 'Outcome:' in json.dumps(delivery,ensure_ascii=False)
o=json.loads((out/'archived-outcome.native.actual.json').read_text());assert 'user_visible_benefit_not_declared' in o['humanBenefitReport']['unknowns']
report={'recordedBy':'agent:Codex','recordedAt':datetime.now(timezone.utc).isoformat(),'kind':'actual own WI1090 finish/archive under separately rechecked native admissions','sourceHeadAtVerificationAndLifecycle':meta['sourceHead'],'contractDigest':meta['contractDigest'],'materialRequestDigest':material['requestDigest'],'materialReceiptDigest':status['reviewReceiptDigest'],'newVerificationCanonicalRawSha256':hashlib.sha256(raw).hexdigest(),'newVerificationReceiptDigest':envelope['receiptDigest'],'nodesExecuted':7,'nodesReused':0,'maxConcurrentProcesses':1,'newVerificationElapsedSeconds':meta['elapsedSeconds'],'currentNativePreflightBinding':current_preflight_binding,'independentPreflightCallExitCode':refusal['exitCode'],'independentPreflightCallNotRetried':True,'finishExitCode':0,'finishAdmissionDigest':status['actionExplanation']['admissionDigest'],'afterFinishPhase':after_finish['lifecyclePhase'],'archiveExitCode':0,'archiveAdmissionDigest':after_finish['actionExplanation']['admissionDigest'],'actualFinalPhase':after_archive['lifecyclePhase'],'finalNativeStatus':{k:after_archive[k] for k in ['lifecyclePhase','governanceState','verification','blocking','humanDecisionRequired','effectiveUnknowns','safeActions','completionDomains','evidenceFreshness']},'benefitUnknownPreserved':True,'all8MaterialAndSourceFilesFrozen':True,'oldHistoryConsentAndMaterialReceiptsUnchanged':True,'noInterveningCommitBetweenVerificationFinishArchive':True,'archivePath':str(archived/f'{item}.archive.json'),'archiveOutcomeDeliveryReturned':True,'nativeHostDeliveryMode':archive.get('hostDeliveryMode'),'nativeHostDisplayConfirmation':archive.get('hostDisplayConfirmation'),'nativeDeliveryReport':archive.get('deliveryReport'),'externalHostProgramConfigured':False,'hostAccepted':'unknown','hostDisplayed':'unknown','closeExecuted':False,'branchOrWorktreeCleanupExecuted':False,'otherWorkItemOrSentinelOperationExecuted':False,'mergeOrReleaseExecuted':False}
(out/'actual-own-lifecycle-completion.actual.agent-recorded.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n');copy_new();print(json.dumps(report,ensure_ascii=False),flush=True);print('HUMAN_OUTCOME_FIRST_LINE '+full.splitlines()[0],flush=True)
