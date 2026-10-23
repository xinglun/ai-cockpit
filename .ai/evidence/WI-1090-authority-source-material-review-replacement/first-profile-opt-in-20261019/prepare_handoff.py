from pathlib import Path
import subprocess,json,hashlib,datetime,shutil
root=Path('/workspace/ai-cockpit-wi1090-material-replacement');item='WI-1090-authority-source-material-review-replacement';out=Path('/workspace/scratch/wi1090-first-profile-opt-in-20261019');r='/workspace/scratch/wi1087-fixed-runtime-b8ec269-20261018/target/release/ai-cockpit'
summary=json.loads((out/'after-preflight.summary.actual.json').read_text());status=json.loads((out/'after-preflight-status.stdout.actual.json').read_text());plan=json.loads((out/'after-amend-material-plan.stdout.actual.json').read_text());receipt=json.loads((out/'native-first-opt-in.attempt1.stdout.actual.json').read_text())
assert status['humanDecisionRequired'] and status['actionExplanation']['admissionState']=='needs_human_decision'
assert summary['decisionEvidence']['contractDigest']=='sha256:c9a3bda349078f1fc15891e9e789d100952cbadaea12eb464f2024c90617ce9c'
assert (root/f'.ai/evidence/{item}.verification.json').read_bytes()==(out/'before.verification.canonical.actual.json').read_bytes()
prior=json.loads((out/'before-governance-raw-inventory.private.actual.json').read_text());changes=[];old_own_preserved=0
for e in prior:
 p=root/e['path'];d=hashlib.sha256(p.read_bytes()).hexdigest() if p.is_file() else None
 if d!=e['sha256']:changes.append(e['path'])
 elif item in e['path']:old_own_preserved+=1
allowed={f'.ai/work-items/active/{item}.contract.json',f'.ai/work-items/active/{item}.summary.json'}
assert set(changes)<=allowed,changes
current={str(p.relative_to(root)) for p in (root/'.ai').rglob('*') if p.is_file() and not p.is_symlink()}
new=current-{e['path'] for e in prior}
assert all(item in p for p in new),new
approved=json.loads((out/'before-material-plan.stdout.actual.json').read_text())
for e in approved['entries']:
 assert hashlib.sha256((root/e['path']).read_bytes()).hexdigest()==e['afterBlobDigest'].removeprefix('sha256:')
assert subprocess.check_output(['git','diff','b1fecfc35e14f700f146a51d4cc494590b13919c','--',*[e['path'] for e in approved['entries']]],cwd=root)==b''
binding={'recordType':'actual pending native Contract review binding; not a recorded decision','recordedBy':'agent:Codex','observedAt':datetime.datetime.now(datetime.timezone.utc).isoformat(),'schemaVersion':1,'decisionId':'contract-preflight-review','requestedDecision':'confirm_review','repositoryId':status['repositoryId'],'workItemId':item,'contractDigest':status['sourceDigests']['contract'],'preflightDecisionDigest':summary['preflightDecisionDigest'],'repositorySnapshotDigest':summary['preflightRepositorySnapshotDigest'],'runtimeVersion':receipt['runtimeVersion'],'runtimeDigest':receipt['runtimeDigest'],'protocolVersion':1,'environmentObservationDigest':receipt['environmentObservationDigest'],'materialRequestDigest':plan['requestDigest'],'materialSourceSnapshotDigest':plan['sourceSnapshotDigest'],'materialManifestDigest':plan['materialManifestDigest'],'materialInspectionReviewProfileDigest':plan['materialInspectionReviewProfileDigest'],'effectivePolicyDigest':plan['effectivePolicyDigest'],'policyRequestRawSha256':'ff93edd6797c0c6e973e1e3355024d84e96d4ca8d616604b37eec859785f323f','nativeAmendmentRequestDigest':receipt['requestDigest'],'nativeAmendmentSequence':3,'nativeAmendmentJournalDigest':receipt['journalDigest'],'preflightAt':summary['preflightAt'],'admissionDigest':status['actionExplanation']['admissionDigest'],'nativeHumanDecisionRequired':True,'proposedActorOnly':'human:Ray','proposedRecorderOnly':'agent:Codex','proposedActorHasNotConfirmedThisNewBinding':True,'oldC9ReviewNotReplayed':True,'materialConsentFileCreated':False,'materialRecordInvoked':False,'newVerificationInvoked':False,'approvalRequiredBy':'Native preflight contract_amendment_policy_review_required; prior Contract confirmation cannot authorize changed Contract','sourceHeadBeforeGovernanceCommit':'b1fecfc35e14f700f146a51d4cc494590b13919c','eightSourceFilesUnchanged':True}
(out/'pending-new-contract-human-review.binding.actual.agent-recorded.json').write_text(json.dumps(binding,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(binding,ensure_ascii=False))
p=subprocess.run([r,'work-item','outcome','--repo',str(root),'--id',item,'--view','full','--language','zh-CN'],capture_output=True,timeout=90);assert p.returncode==0,p.stderr
(out/'native-pending-outcome-full.zh-CN.actual.txt').write_bytes(p.stdout)
print(p.stdout.decode()[:900])
(out/'preservation-check.actual.agent-recorded.json').write_text(json.dumps({'recordedBy':'agent:Codex','actualBeforeFilesChanged':changes,'actualNewNativePaths':sorted(new),'priorOwnFilesPreserved':old_own_preserved,'priorOtherWorkItemsChanged':False,'canonicalPriorVerificationRawPreserved':True,'oldDecisionEvidencePreserved':True,'allEightSourceFilesByteIdentical':True,'noVerificationRerun':True},ensure_ascii=False,indent=2)+'\n')
lines=['Outcome: 🟡 首次材料政策修约成功；等待实际新 Contract 真人确认','','原始政策请求ff93…323f执行恰一次，exit0，native seq3。所有8项材料的blob/hunk、manifest、scanner身份、Finding0与唯一raw syntax Unknown均匹配已批准固定材料。Contract只改变指定profile与Runtime自动推导capability marker。','', 'Native preflight要求新的confirm_review。已有c9确认仅保留历史，未重放；Ray的有条件材料风险接受不被记录成新Contract已经审阅。consent文件未创建，材料record未调用，未重跑测试；源码和三语文档保持b1字节。','', '| 绑定字段 | 实际值 |','|---|---|']
for k in ['repositoryId','workItemId','contractDigest','preflightDecisionDigest','repositorySnapshotDigest','runtimeVersion','runtimeDigest','protocolVersion','environmentObservationDigest','materialRequestDigest','materialSourceSnapshotDigest','materialManifestDigest','materialInspectionReviewProfileDigest','policyRequestRawSha256','nativeAmendmentRequestDigest','nativeAmendmentJournalDigest','admissionDigest','preflightAt']:
 lines.append('| '+k+' | `'+str(binding[k])+'` |')
lines+=['','待Ray确认此新完整绑定后，再由agent:Codex按实际授权原生记录human:Ray的confirm_review并重新查询准入。材料风险条件继续有效，但不得预造新绑定审阅或继承旧决定。','', '当前verification因合法政策修约为stale/not_ready，旧七项真实通过收据原样保留，不称新Contract已验证。Native status blocking=false、humanDecisionRequired=true、admissionState=needs_human_decision；safeActions中出现材料record不替代真人门槛。','', '环境：固定Runtime b5cbd…，agent doctor VERIFIED，doctor red仅已知self-host runtimeCodeInRepository=true；兼容性COMPATIBLE。WI1090无coordination registration，不伪造generation或绑定，也未修改任何协调记录。','', '其他WI、Sentinel、发布、采用、真实终态均未操作。收益仍未声明；本次仅建立获批政策及可复核新绑定。原生Outcome已返回；宿主显示确认未知。']
(out/'NEW-CONTRACT-REVIEW-HANDOFF.actual.zh-CN.md').write_text('\n'.join(lines)+'\n')
publish=root/f'.ai/evidence/{item}/first-profile-opt-in-20261019';publish.mkdir()
excluded={'before-governance-raw-inventory.private.actual.json','before-coordination-inspect.stdout.actual.json','before-coordination-inspect.stderr.actual.txt'}
for p in out.iterdir():
 if p.is_file() and p.name not in excluded:shutil.copyfile(p,publish/p.name)
print('published',len(list(publish.iterdir())),'artifacts; private full governance inventory and other-WI coordination dump not published')
