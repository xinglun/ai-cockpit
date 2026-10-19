from pathlib import Path
from datetime import datetime,timezone
import json,hashlib,re,subprocess,copy
root=Path('/workspace/ai-cockpit-wi1090-material-replacement')
item='WI-1090-authority-source-material-review-replacement'
out=Path('/workspace/scratch/wi1090-minimal-negative-matrix-20261019/final-evidence')
out.mkdir(exist_ok=True)
old=json.loads((root/f'.ai/evidence/{item}/scenario-mapping-20261019/M1-M5-clause-to-evidence-map.actual.agent-recorded.json').read_text())
canonical=root/f'.ai/evidence/{item}.verification.json'
raw=canonical.read_bytes(); envelope=json.loads(raw)
meta=json.loads(Path('/workspace/scratch/wi1090-v4-implementation-20261019/native-seven.attempt3.metadata.actual.json').read_text())
assert meta['exitCode']==0 and meta['terminal']=='returned'
assert envelope['passed'] and envelope['receipt']['nodesExecuted']==7 and envelope['receipt']['nodesReused']==0
assert envelope['contractDigest']==old['contractDigest']
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()
logs={}
for record in envelope['receipt']['executionRecords']:
 assert record['passed'] and record['exitCode']==0
 if 'stdout' in record:
  data=bytes.fromhex(record['stdout']); name=record['nodeId']+'.stdout.decoded-from-canonical.actual.txt'
  (out/name).write_bytes(data); logs[record['nodeId']]={'name':name,'lines':data.decode().splitlines()}
def update_test(e):
 path=e['sourcePath']; data=(root/path).read_bytes()
 assert data==subprocess.check_output(['git','show',meta['sourceHead']+':'+path],cwd=root)
 lines=data.decode().splitlines(); name=e['test']; node=e['formalNodeId']
 display='material_review::tests::'+name if node=='project-command-2' else name
 matches=[i+1 for i,line in enumerate(logs[node]['lines']) if line=='test '+display+' ... ok']
 assert len(matches)==1,(node,name)
 e.update(sourceLine=next(i+1 for i,line in enumerate(lines) if re.match(r'\s*fn\s+'+re.escape(name)+r'\s*\(',line)),testedSourceHead=meta['sourceHead'],sourceRawSha256=hashlib.sha256(data).hexdigest(),sameBytesAtCurrentEvidenceHead=head,decodedStdoutPath=logs[node]['name'],passedStdoutLine=matches[0])
 return e
for row in old['rows']:
 for e in row['evidence']: update_test(e)
base={'test':'segmented_minimal_independent_integrity_boundaries','sourcePath':'crates/cockpit-repository/tests/material_review.rs','formalNodeId':'project-command-3','assertionActuallyInspected':''}
def matrix(assertion):
 e=copy.deepcopy(base); e['assertionActuallyInspected']=assertion+' All ten cases first assert eligible baseline; after mutation projection offers no new decision or errors fail closed, record errors, complete .ai bytes/link targets/path set and old receipt stay unchanged; no prepare/sidecar publication.'
 return update_test(e)
rows={r['id']:r for r in old['rows']}
rows['M2.5'].update(coverage='proven_independent_boundaries',proofLevel='actual_native_finite_fixture_and_tamper_negatives',evidence=[matrix('Actual well-formed foreign native pair; disconnected predecessor whose individual native pairs pass read_work_item_contract_amendments; protected base native amendment refused without writes then resealed tamper rejected; actual later native sources amendment makes B nonterminal; direct Contract edit rejected.')],gapOrLimit='Base is natively protected: no successful native base amendment claimed. Attack resealing is adversarial input only. No Cartesian-product claim.')
rows['M5.3']['evidence'].append(matrix('Current decision artifact and Summary consistently changed to stale snapshot or foreign repository identity, independently of missing-review case.'))
rows['M5.3'].update(coverage='proven_independent_current_review_boundaries',gapOrLimit='Native self_declared files are checked for binding/integrity, not a new human authentication root.')
rows['M5.4']['evidence'].append(matrix('After P, actual committed src/material.rs change produces distinct blob, changed-hunk and request digests. After fresh exact Contract review both old input and new input reject; fresh input cannot repair stale P source binding.'))
rows['M5.4'].update(coverage='proven_shared_validator_and_finite_source_boundaries',gapOrLimit='Public record recomputes the request and accepts no request body; no nonexistent member-injection interface is tested. Existing validator tests cover analysis/scanner/policy identities. No field Cartesian product.')
rows['M5.5']['evidence'].append(matrix('A genuine natively produced, canonically valid foreign repository receipt is appended via a well-formed full pointer; original old pointer is retained. Byte-identical journal prepared file is replaced with a symlink (Unix); both reject.'))
rows['M5.5'].update(coverage='proven_independent_integrity_boundaries',gapOrLimit='Journal symlink executed on Linux; no Windows junction execution claimed. Deferred pending-after-historical-corruption combinations are not promoted to blockers.')
rows['M5.6']['evidence'].append(matrix('Dedicated actual Rust blob/hunk source change after P rejects even after fresh current Contract review.'))
rows['M5.7'].update(coverage='proven_old_binding_rejection_behavior',proofLevel='actual_service_input_and_finite_source_binding_tests',evidence=[update_test({'test':'segmented_wrong_current_inputs_publish_no_prepare_or_receipt','sourcePath':base['sourcePath'],'formalNodeId':'project-command-3','assertionActuallyInspected':'Wrong old request, reviewer, authoritySource, schema, empty evidence and blank rationale reject without prepare/sidecar/ Summary change.'}),matrix('Actual new Rust material refuses old decision input; refreshed input still cannot reuse historical source anchor. Existing positive fixture requires a separate new exact decision.')],gapOrLimit='Implementation acceptance is refusal to extend old bindings. Actual WI1090 residual-risk consent is a separate pending real-material workflow, not a missing implementation test or an inferred grant. No WI1089 consent inherited.')
for row in old['rows']:
 for e in row['evidence']:
  if e['sourcePath']==base['sourcePath'] and e['test'].startswith('segmented_') and 'Helper checks' in e['assertionActuallyInspected']:
   e['assertionActuallyInspected']+=' Current helper additionally compares complete .ai byte/link/path inventory.'
old.update(kind='final_original_acceptance_mapping_from_new_actual_formal_verification',observedAt=datetime.now(timezone.utc).isoformat(),canonicalRawSha256=hashlib.sha256(raw).hexdigest(),canonicalReceiptDigest=envelope['receiptDigest'],actualFormalSourceHead=meta['sourceHead'],currentEvidenceOnlyHead=head,formalStartedAt=meta['startedAt'],formalEndedAt=meta['endedAt'],noChecksRerun=False)
old['formalExecution']='One new final formal seven-node run after test source stabilized; all7 executed, zero reused. Earlier same-source checks not repeated.'
old['nonblockingDeferred']='Pending-after-historical-corruption combinations remain deferred as Raydot directed; no power-loss/parent-fsync/uncooperative final-gap writer or human-authentication guarantee. M6 actual execution4/reuse0 reason reuse_not_configured; no cache invalidation or Runtime-upgrade proof.'
for name,decision in old['scenarioRecordingDecisions'].items():
 decision['nativeRecord']='verified'
 if name.startswith('M2 '): decision['reason']='Original clauses now have actual native finite-fixture negative coverage including independent foreign/fork/base/nonterminal/direct boundaries; all reject without governance writes. Protected base native refusal is not a successful amendment.'
 if name.startswith('M5 '): decision['reason']='Existing validator/planner/transaction cases plus independent stale/foreign current review, actual Rust blob/hunk change, genuine foreign history receipt and Unix journal symlink passed. M5.7 proves refusal of old bindings; actual WI1090 risk consent remains a separate pending workflow.'
(out/'M1-M5-clause-to-evidence-map.final.actual.agent-recorded.json').write_text(json.dumps(old,ensure_ascii=False,indent=2)+'\n')
lines=['# WI1090 最终最小矩阵与原始条件映射','','正式执行HEAD `'+meta['sourceHead']+'`；七项真实执行、零复用；原始22条条件逐条回读新正式日志。仅新增独立边界测试，生产代码未改。','','| 条件 | 实际判定 | 测试 | 保证限制 |','|---|---|---|---|']
for row in old['rows']:
 lines.append('| '+row['id']+' '+row['condition']+' | '+row['coverage']+' | '+'; '.join(e['test']+' ('+e['formalNodeId']+':'+str(e['passedStdoutLine'])+')' for e in row['evidence'])+' | '+str(row['gapOrLimit'] or '')+' |')
lines+=['','M2/M5 实现行为验收可原生记录 verified；真实材料风险同意仍未发生，profile opt-in 仍是草案。不得将此实现验收当作材料批准。','',old['nonblockingDeferred']]
(out/'M1-M5-evidence-map.final.actual.zh-CN.md').write_text('\n'.join(lines)+'\n')
contract=json.loads((root/f'.ai/work-items/active/{item}.contract.json').read_text())
controls=[]
for scenario in contract['scenarioCoverage'][:5]:
 decision=old['scenarioRecordingDecisions'][scenario['scenario']]
 s=copy.deepcopy(scenario); s.update(status=decision['nativeRecord'],reason=decision['reason'],evidence=[f'.ai/evidence/{item}.verification.json',f'.ai/evidence/{item}/minimal-negative-matrix-20261019/native-seven.attempt3.metadata.actual.json',f'.ai/evidence/{item}/minimal-negative-matrix-20261019/M1-M5-clause-to-evidence-map.final.actual.agent-recorded.json'])
 controls.append(s)
(out/'M1-M5-final.controls.input.json').write_text(json.dumps({'scenarioCoverage':controls},ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'rows':len(old['rows']),'testedHead':meta['sourceHead'],'canonicalRawSha256':old['canonicalRawSha256'],'canonicalReceiptDigest':old['canonicalReceiptDigest']},ensure_ascii=False))
