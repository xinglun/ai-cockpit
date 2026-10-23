from pathlib import Path
from datetime import datetime,timezone
import subprocess,json,hashlib,re
root=Path('/workspace/ai-cockpit-wi1090-material-replacement');item='WI-1090-authority-source-material-review-replacement';out=Path('/workspace/scratch/wi1090-confirm-material-verify-20261019');r=Path('/workspace/scratch/wi1087-fixed-runtime-b8ec269-20261018/target/release/ai-cockpit');evidence=root/f'.ai/evidence/{item}/new-contract-confirm-material-20261019'
assert not evidence.exists();evidence.mkdir()
assert hashlib.sha256(r.read_bytes()).hexdigest()=='b5cbd8570df5d3cb2004ae517fe88df6136c66d92fc1913070d18f2c8f67f257'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()=='8307af61e75e0898aa74fb6283a8e72891fc0f99'
assert subprocess.check_output(['git','diff','--name-only'],cwd=root)==b''
def capture(label,args):
 p=subprocess.run([str(r),*args,'--repo',str(root)],capture_output=True,timeout=90)
 for suffix,data in [('stdout.actual.json',p.stdout),('stderr.actual.txt',p.stderr)]:
  f=out/(label+'.'+suffix);assert not f.exists();f.write_bytes(data)
 assert p.returncode==0,(label,p.stderr.decode())
 return json.loads(p.stdout)
status=capture('before-confirm-status',['work-item','status','--id',item,'--json']);plan=capture('before-confirm-material-plan',['work-item','material-review','plan','--id',item]);inspect=capture('before-confirm-inspect',['inspect'])
assert inspect['runtimeDigest']=='sha256:'+hashlib.sha256(r.read_bytes()).hexdigest() and inspect['protocolVersion']==1
binding=json.loads((root/f'.ai/evidence/{item}/first-profile-opt-in-20261019/pending-new-contract-human-review.binding.actual.agent-recorded.json').read_text());summary_path=root/f'.ai/work-items/active/{item}.summary.json';summary=json.loads(summary_path.read_text())
for a,b in [(status['sourceDigests']['contract'],binding['contractDigest']),(summary['preflightDecisionDigest'],binding['preflightDecisionDigest']),(summary['preflightRepositorySnapshotDigest'],binding['repositorySnapshotDigest']),(plan['requestDigest'],binding['materialRequestDigest'])]:assert a==b,(a,b)
assert status['humanDecisionRequired'] and 'record_governance_controls' in status['safeActions']
original=json.loads((root/f'.ai/evidence/{item}/minimal-negative-matrix-20261019/final-material-plan.stdout.actual.json').read_text())
for k in ['entries','sourceSnapshotDigest','materialManifestDigest','scannerSemanticVersion','analysisImplementationDigest','analysisTargetSemanticProfile','rawUnknownCodes','findingCodes','blockedByFinding']:assert plan[k]==original[k],k
assert len(plan['entries'])==8 and len([e for e in plan['entries'] if e['scannerAssessment']=='unknown'])==1 and plan['reviewEnabled']
proposal=root/f'.ai/evidence/{item}/scenario-mapping-20261019/first-profile-opt-in.amend.request.proposal-only.json';assert hashlib.sha256(proposal.read_bytes()).hexdigest()=='ff93edd6797c0c6e973e1e3355024d84e96d4ca8d616604b37eec859785f323f'
contract=json.loads((root/f'.ai/work-items/active/{item}.contract.json').read_text());assert contract['governanceProfile']==json.loads(proposal.read_text())['changes'][0]['value']
for e in plan['entries']:
 path=e['path'];blob=subprocess.check_output(['git','show','HEAD:'+path],cwd=root);assert 'sha256:'+hashlib.sha256(blob).hexdigest()==e['afterBlobDigest'];assert (root/path).read_bytes()==blob
 patch=subprocess.check_output(['git','diff','--no-ext-diff','--no-textconv','--no-renames','--unified=0',plan['immutableContractBaseRevision'],'HEAD','--',path],cwd=root).decode();added=[];removed=[];origins=[];hunk=-1;line_number=None
 for line in patch.splitlines():
  if line.startswith('@@ '):
   m=re.match(r'@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@',line);assert m;line_number=int(m[1]);hunk+=1
  elif line_number is not None:
   if line.startswith('+'):added.append(line[1:]);origins.append([line_number,hunk]);line_number+=1
   elif line.startswith('-'):removed.append(line[1:])
   elif line.startswith(' '):line_number+=1
 digest='sha256:'+hashlib.sha256(json.dumps([added,origins,removed],ensure_ascii=False,separators=(',',':')).encode()).hexdigest();assert digest==e['changedHunkDigest'],path
(out/'before-confirm.summary.actual.json').write_bytes(summary_path.read_bytes())
now=datetime.now(timezone.utc).isoformat().replace('+00:00','Z')
transcript={'recordedBy':'agent:Codex','recordedAt':now,'recordType':'actual relayed human Contract confirmation, separate from earlier conditional material risk acceptance','actor':'human:Ray','recorder':'agent:Codex','relayedBy':'agent:Raydot','requestMessageId':'Sentinel_09302e9f41a88191bcd960ffe84f27fa','replyMessageId':'Sentinel_5e627daabe548191ac72167f9a8963c3','replyMessageTimeAsProvided':'2026-10-19 22:42 UTC','verbatimReply':'确认上述 WI-1090 修约后完整绑定，confirm_review。','confirmedBinding':binding,'fixedHeadConfirmed':'8307af61e75e0898aa74fb6283a8e72891fc0f99','all8BlobHunkAndScannerConditionsRechecked':True,'policyRequestRawSha256':'ff93edd6797c0c6e973e1e3355024d84e96d4ca8d616604b37eec859785f323f','provenanceLimit':'Relayed conversation recorded by agent; no new host-authenticated human identity claim. This does not invent another material risk decision, ordinary acceptance or Sentinel recovery.'}
tp=evidence/'ray-new-contract-confirmation.relayed.agent-recorded.json';tp.write_text(json.dumps(transcript,ensure_ascii=False,indent=2)+'\n')
decision={'schemaVersion':1,'decisionId':'contract-preflight-review','decision':'confirm_review','workItemId':item,'repositoryId':binding['repositoryId'],'contractDigest':binding['contractDigest'],'preflightDecisionDigest':binding['preflightDecisionDigest'],'repositorySnapshotDigest':binding['repositorySnapshotDigest'],'recordedAt':now,'recordedBy':'human:Ray','reason':'Ray replied literally 确认上述 WI-1090 修约后完整绑定，confirm_review。 in Sentinel_5e627daabe548191ac72167f9a8963c3 to exact request Sentinel_09302e9f41a88191bcd960ffe84f27fa; relayed by agent:Raydot, recorder agent:Codex. Actual full binding and provenance at '+str(tp.relative_to(root))+'. This is the new b6d2 Contract confirmation, not replay of c9, no invented new risk acceptance, ordinary acceptance, Sentinel recovery or release approval.'}
input_path=out/'actual-new-contract-confirm.controls.input.json';input_path.write_text(json.dumps({'decisionEvidence':decision},ensure_ascii=False,indent=2)+'\n')
marker=out/'native-confirm.attempt1.metadata.actual.json';assert not marker.exists();meta={'recordedBy':'agent:Codex','startedAt':now,'terminal':'running','kind':'actual identity-bound native human Contract review record','humanReplyMessageId':transcript['replyMessageId'],'contractDigest':binding['contractDigest']};marker.write_text(json.dumps(meta,indent=2)+'\n')
p=subprocess.run([str(r),'work-item','controls','--repo',str(root),'--id',item,'--input',str(input_path)],capture_output=True,timeout=90)
(out/'native-confirm.attempt1.stdout.actual.json').write_bytes(p.stdout);(out/'native-confirm.attempt1.stderr.actual.txt').write_bytes(p.stderr);meta.update(terminal='returned',exitCode=p.returncode,endedAt=datetime.now(timezone.utc).isoformat());marker.write_text(json.dumps(meta,indent=2)+'\n');assert p.returncode==0,p.stderr
current=json.loads(summary_path.read_text());assert current['decisionEvidence']==decision
matches=[]
for f in (root/'.ai/decisions').iterdir():
 if f.is_file():
  try:v=json.loads(f.read_text())
  except (ValueError,UnicodeError):continue
  if v==decision:matches.append({'path':str(f.relative_to(root)),'rawSha256':hashlib.sha256(f.read_bytes()).hexdigest()})
assert len(matches)==1,matches
(out/'native-human-review-readback.actual.agent-recorded.json').write_text(json.dumps({'recordedBy':'agent:Codex','actualStoredDecision':current['decisionEvidence'],'nativeDecisionFiles':matches,'oldC9ReviewNotReplayed':True},ensure_ascii=False,indent=2)+'\n')
after=capture('after-confirm-status',['work-item','status','--id',item,'--json']);assert not after['humanDecisionRequired'],after
print(json.dumps({'confirmExitCode':p.returncode,'newContractDigest':binding['contractDigest'],'storedActor':decision['recordedBy'],'recorder':'agent:Codex','nativeDecisionFile':matches[0],'humanDecisionRequired':after['humanDecisionRequired'],'blocking':after['blocking'],'safeActions':after['safeActions'],'verification':after['verification']},ensure_ascii=False))
