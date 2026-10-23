from pathlib import Path
from datetime import datetime,timezone
import json,hashlib,re,subprocess,shutil
root=Path('/workspace/ai-cockpit-wi1090-material-replacement')
item='WI-1090-authority-source-material-review-replacement'
out=Path('/workspace/scratch/wi1090-minimal-negative-matrix-20261019/final-evidence')
out.mkdir(exist_ok=True)
runtime=Path('/workspace/scratch/wi1087-fixed-runtime-b8ec269-20261018/target/release/ai-cockpit')
assert hashlib.sha256(runtime.read_bytes()).hexdigest()=='b5cbd8570df5d3cb2004ae517fe88df6136c66d92fc1913070d18f2c8f67f257'
r=subprocess.run([str(runtime),'work-item','material-review','plan','--repo',str(root),'--id',item],capture_output=True,timeout=90)
(out/'final-material-plan.stdout.actual.json').write_bytes(r.stdout)
(out/'final-material-plan.stderr.actual.txt').write_bytes(r.stderr)
assert r.returncode==0,r.stderr
plan=json.loads(r.stdout)
assert plan['reviewEnabled'] is False and plan['materialInspectionReviewProfileDigest'] is None
assert not plan['findingCodes'] and not plan['blockedByFinding']
assert len(plan['entries'])==8
unknown=[e for e in plan['entries'] if e['scannerAssessment']=='unknown']
assert len(unknown)==1 and unknown[0]['reviewable']
actual=[]
for index,e in enumerate(plan['entries']):
 path=e['path']; blob=subprocess.check_output(['git','show','HEAD:'+path],cwd=root)
 blob_digest='sha256:'+hashlib.sha256(blob).hexdigest();assert blob_digest==e['afterBlobDigest']
 patch=subprocess.check_output(['git','diff','--no-ext-diff','--no-textconv','--no-renames','--unified=0',plan['immutableContractBaseRevision'],'HEAD','--',path],cwd=root).decode()
 added=[];removed=[];origins=[];hunk=-1;after_line=None
 for line in patch.splitlines():
  if line.startswith('@@ '):
   m=re.match(r'@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@',line); assert m
   after_line=int(m[1]);hunk+=1
  elif after_line is not None:
   if line.startswith('+'):
    added.append(line[1:]);origins.append([after_line,hunk]);after_line+=1
   elif line.startswith('-'):removed.append(line[1:])
   elif line.startswith(' '):after_line+=1
 hunk_input=[added,origins,removed]
 hunk_bytes=json.dumps(hunk_input,ensure_ascii=False,separators=(',',':')).encode()
 hunk_digest='sha256:'+hashlib.sha256(hunk_bytes).hexdigest();assert hunk_digest==e['changedHunkDigest'],(path,hunk_digest,e['changedHunkDigest'])
 actual.append(dict(path=path,byteCount=len(blob),afterBlobDigest=blob_digest,changedHunkDigest=hunk_digest,addedLines=len(added),removedLines=len(removed),hunkCount=hunk+1,scannerAssessment=e['scannerAssessment'],unknownCause=e['unknownCause'],reviewable=e['reviewable']))
 if e in unknown:
  external=out.parent
  (external/'final-reviewable.blob.actual.txt').write_bytes(blob)
  (external/'final-reviewable.u0.patch.actual.txt').write_text(patch)
  (external/'final-reviewable.hunk-digest-input.actual.json').write_bytes(hunk_bytes)
proposal=root/f'.ai/evidence/{item}/scenario-mapping-20261019/first-profile-opt-in.amend.request.proposal-only.json'
raw=proposal.read_bytes();request=json.loads(raw)
assert request['expectedContractDigest']==plan['contractDigest'] and len(request['changes'])==1 and request['changes'][0]['path']=='/governanceProfile'
consent='.ai/evidence/'+item+'/material-risk-consent/current-risk-consent.agent-recorded.json'
assert not (root/consent).exists()
shutil.copyfile(proposal,out/'first-profile-opt-in.amend.request.STILL-PROPOSAL-ONLY.json')
report=dict(kind='final_source_material_binding_and_unexecuted_policy_proposal',observedAt=datetime.now(timezone.utc).isoformat(),reviewedSourceHead=plan['reviewedSourceHead'],contractDigest=plan['contractDigest'],requestDigest=plan['requestDigest'],sourceSnapshotDigest=plan['sourceSnapshotDigest'],materialManifestDigest=plan['materialManifestDigest'],runtimeDigest='sha256:'+hashlib.sha256(runtime.read_bytes()).hexdigest(),analysisImplementationDigest=plan['analysisImplementationDigest'],effectivePolicyDigest=plan['effectivePolicyDigest'],profile=None,reviewEnabled=False,entryCount=8,cleanCount=7,reviewableUnknownCount=1,entries=actual,independentHunkAndRawBlobRecalculation='all8 matched actual fixed Runtime plan',proposalRawSha256=hashlib.sha256(raw).hexdigest(),proposalOperationCount=1,proposalState='UNEXECUTED; no consent file, amendment, actual risk grant, material decision or WI1089 inheritance',proposedReviewerActor=request['changes'][0]['value']['materialInspectionReview']['reviewerActor'],proposedAuthoritySource=consent,futureContractDigest=None,futureRequestDigest=None,futureSequence=['Raydot technical review of final source and minimum policy proposal; no actual risk acceptance inferred.','After actual policy authorization only, natively amend; read resulting real Contract/preflight/snapshot/admission and obtain required bound Contract review if any.','Refresh post-policy/post-documentation material plan; actual future request differs and is not predicted here.','Obtain actual WI1090 residual-risk consent and reviewer decision bound to the real new material request. Native material record only if then admitted.','Use resulting native verification freshness to determine any required checks; do not repeat unchanged checks preemptively.'])
(out/'final-material-binding-and-policy-draft.actual.agent-recorded.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
lines=['# WI1090 最终源码材料绑定与首次 profile 草案','','当前 profile=null、reviewEnabled=false；7 clean + 1 readable syntax unknown，Finding=0。本材料只更新最终源码绑定，不执行 opt-in、不生成同意文件。','','- Source HEAD: `'+plan['reviewedSourceHead']+'`','- Contract: `'+plan['contractDigest']+'`','- Request: `'+plan['requestDigest']+'`','- Snapshot: `'+plan['sourceSnapshotDigest']+'`','- Manifest: `'+plan['materialManifestDigest']+'`','- 唯一最小政策提案：初次 `/governanceProfile` set 一项；SHA-256 `'+report['proposalRawSha256']+'`。`acceptResidualRisk=true` 仍仅拟议字段，不是已有风险同意。','','| 材料 | blob | hunk | 评估 |','|---|---|---|---|']
for e in actual: lines.append('| '+e['path']+' | '+e['afterBlobDigest']+' | '+e['changedHunkDigest']+' | '+e['scannerAssessment']+' |')
lines+=['','全部8项 raw blob 与实际 u0 hunk 独立重算匹配原生计划。完整reviewable blob/u0 patch在外部保存，可用GitHub exact HEAD同文件审阅。','', '后续授权顺序：']+['- '+s for s in report['futureSequence']]
lines+=['','M5.7 实现验收证明拒绝沿用旧批准/旧绑定；真实WI1090残余风险同意是独立未完成工作流。旧WI1089同意、技术审阅、合成fixture决定及 c9确认均不构成该同意。','', '未来 Contract/Request 必须取实际原生生成值；此处不预造绑定，不向当前材料批准推定未来材料批准。']
(out/'final-material-and-minimum-profile.STILL-PROPOSAL-ONLY.zh-CN.md').write_text('\n'.join(lines)+'\n')
print(json.dumps({k:report[k] for k in ['reviewedSourceHead','contractDigest','requestDigest','sourceSnapshotDigest','materialManifestDigest','proposalRawSha256','entryCount','reviewableUnknownCount']},ensure_ascii=False))
