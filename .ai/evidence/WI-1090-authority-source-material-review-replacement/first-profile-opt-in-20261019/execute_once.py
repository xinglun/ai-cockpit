from pathlib import Path
import subprocess,json,hashlib,datetime
root=Path('/workspace/ai-cockpit-wi1090-material-replacement');item='WI-1090-authority-source-material-review-replacement';out=Path('/workspace/scratch/wi1090-first-profile-opt-in-20261019');r=Path('/workspace/scratch/wi1087-fixed-runtime-b8ec269-20261018/target/release/ai-cockpit')
marker=out/'native-first-opt-in.attempt1.metadata.actual.json';assert not marker.exists(),'Inspect prior attempt; never replay blindly'
request=root/f'.ai/evidence/{item}/scenario-mapping-20261019/first-profile-opt-in.amend.request.proposal-only.json'
assert hashlib.sha256(request.read_bytes()).hexdigest()=='ff93edd6797c0c6e973e1e3355024d84e96d4ca8d616604b37eec859785f323f'
assert hashlib.sha256(r.read_bytes()).hexdigest()=='b5cbd8570df5d3cb2004ae517fe88df6136c66d92fc1913070d18f2c8f67f257'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()=='b1fecfc35e14f700f146a51d4cc494590b13919c'
assert subprocess.check_output(['git','status','--porcelain'],cwd=root)==b''
checked=json.loads((out/'before-conditions.actual.agent-recorded.json').read_text());assert checked['currentPlanSameExceptHead'] and len(checked['independentBlobHunkChecks'])==8
policy=json.loads(request.read_text());assert policy['expectedContractDigest']==checked['nativeStatus']['sourceDigests']['contract']
authorization={'recordedBy':'agent:Codex','recordedAt':datetime.datetime.now(datetime.timezone.utc).isoformat(),'recordType':'retained delegated transcript of conditional policy and material risk authorization; not native new Contract review','authorizer':'human:Ray','delegatingAgent':'agent:Raydot','requestMessageId':'Sentinel_6551594dd2d08191bf22c8b9ef127bac','requestMessageTime':'2026-10-19 21:57:59 UTC','replyMessageId':'Sentinel_89eec2e40c0c819181319fd54b8fa553','replyMessageTimeAsProvided':'2026-10-19 22:25 UTC','verbatimReply':'批准上述 WI-1090 最小材料审阅政策，并按上述条件接受固定材料的残余风险。','approvedHead':'b1fecfc35e14f700f146a51d4cc494590b13919c','approvedOldContractDigest':policy['expectedContractDigest'],'approvedOldRequestDigest':'sha256:23e04c983732d47799e1ec444849fd89e0c92f3a5b13742ac636be30c8b6e3d8','policyRequestRawSha256':'ff93edd6797c0c6e973e1e3355024d84e96d4ca8d616604b37eec859785f323f','conditions':['all8 material raw blobs and changed hunks unchanged after amendment','scanner semantic/implementation/target identities unchanged','no additional Finding or Unknown','policy strictly equals approved proposal','actual new Contract and request read natively before binding','native required actual human Contract review handled separately'],'newContractHumanReviewOccurred':False,'designatedConsentFileCreated':False,'assurance':'self_declared agent-retained conversation evidence, not authenticated human identity'}
(out/'ray-conditional-policy-authorization.transcript.agent-recorded.json').write_text(json.dumps(authorization,ensure_ascii=False,indent=2)+'\n')
argv=[str(r),'work-item','amend','--repo',str(root),'--id',item,'--request',str(request)]
meta={'kind':'actual native first profile opt-in attempt','argv':argv,'startedAt':datetime.datetime.now(datetime.timezone.utc).isoformat(),'terminal':'running','policyRawSha256':authorization['policyRequestRawSha256'],'sourceHead':authorization['approvedHead'],'runtimeSha256':hashlib.sha256(r.read_bytes()).hexdigest(),'timeoutSeconds':120,'newContractHumanReviewOccurred':False}
marker.write_text(json.dumps(meta,ensure_ascii=False,indent=2)+'\n')
try:
 p=subprocess.run(argv,capture_output=True,timeout=120)
 (out/'native-first-opt-in.attempt1.stdout.actual.json').write_bytes(p.stdout);(out/'native-first-opt-in.attempt1.stderr.actual.txt').write_bytes(p.stderr)
 meta.update(terminal='returned',exitCode=p.returncode,endedAt=datetime.datetime.now(datetime.timezone.utc).isoformat());marker.write_text(json.dumps(meta,ensure_ascii=False,indent=2)+'\n')
 print(json.dumps(meta,ensure_ascii=False));print(p.stdout.decode()[:7000]);print(p.stderr.decode()[:1200])
except subprocess.TimeoutExpired as e:
 meta.update(terminal='timeout_child_state_unknown_no_retry',endedAt=datetime.datetime.now(datetime.timezone.utc).isoformat());marker.write_text(json.dumps(meta,ensure_ascii=False,indent=2)+'\n');raise
