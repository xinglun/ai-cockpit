from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import re
import subprocess

root = Path('/workspace/ai-cockpit-wi1090-material-replacement')
out = Path('/workspace/scratch/wi1090-m1-m5-material-inventory-20261019')
item = 'WI-1090-authority-source-material-review-replacement'
tested = '5b6954671c991d57261b98f4f0c4bdd3bf1e377a'
current = 'c1affb57188b152cfe15f00b4328302ca775d5d6'
canonical_path = root / f'.ai/evidence/{item}.verification.json'
canonical_bytes = canonical_path.read_bytes()
envelope = json.loads(canonical_bytes)
assert envelope['passed'] and envelope['receipt']['nodesExecuted'] == 7
assert hashlib.sha256(canonical_bytes).hexdigest() == 'de695b545e6ebd6ca3e7437b1e91b52490a0449c4d09269da8c38655e73736d0'
metadata = json.loads((root / f'.ai/evidence/{item}/development-20261019/native-seven.attempt2.metadata.actual.json').read_text())
assert metadata['sourceHead'] == tested and metadata['exitCode'] == 0
contract = json.loads((root / f'.ai/work-items/active/{item}.contract.json').read_text())
paths = {
    'unit': ('crates/cockpit-repository/src/material_review.rs', 'project-command-2'),
    'integration': ('crates/cockpit-repository/tests/material_review.rs', 'project-command-3'),
    'status': ('crates/cockpit-repository/tests/status_projection.rs', 'project-command-4'),
    'cli': ('crates/cockpit-cli/tests/contract_amendment_processes.rs', 'project-command-5'),
    'mcp': ('crates/cockpit-mcp/tests/material_review.rs', 'project-command-6'),
}
logs = {}
for record in envelope['receipt']['executionRecords']:
    assert record['passed'] and record['exitCode'] == 0
    if record['nodeId'] in [value[1] for value in paths.values()]:
        raw = bytes.fromhex(record['stdout'])
        path = out / (record['nodeId'] + '.stdout.decoded-from-canonical.actual.txt')
        assert not path.exists(); path.write_bytes(raw)
        logs[record['nodeId']] = {'path': path.name, 'rawSha256': hashlib.sha256(raw).hexdigest(),
                                  'lines': raw.decode().splitlines()}

def test(kind, name, assertion):
    path, node = paths[kind]
    before = subprocess.check_output(['git', 'show', tested + ':' + path], cwd=root)
    now = (root / path).read_bytes()
    assert now == before
    lines = now.decode().splitlines()
    starts = [index + 1 for index, line in enumerate(lines)
              if re.match(r'\s*fn\s+' + re.escape(name) + r'\s*\(', line)]
    assert len(starts) == 1, name
    display = 'material_review::tests::' + name if kind == 'unit' else name
    matches = [index + 1 for index, line in enumerate(logs[node]['lines'])
               if line == 'test ' + display + ' ... ok']
    assert len(matches) == 1, (node, name)
    return {'test': name, 'sourcePath': path, 'sourceLine': starts[0], 'testedSourceHead': tested,
            'sourceRawSha256': hashlib.sha256(now).hexdigest(), 'sameBytesAtCurrentEvidenceHead': current,
            'formalNodeId': node, 'decodedStdoutPath': logs[node]['path'], 'passedStdoutLine': matches[0],
            'assertionActuallyInspected': assertion}

rows = []
def row(id, condition, state, evidence, gap=None, proof_level='actual_tests_and_source_link'):
    rows.append({'id': id, 'condition': condition, 'coverage': state, 'proofLevel': proof_level,
                 'evidence': evidence, 'gapOrLimit': gap})

row('M1.1', '完整已审R-to-A允许scope/acceptanceCriteria/scenarioCoverage/sources四类变化且profile不变', 'proven', [
    test('integration', 'segmented_authority_source_change_exposes_only_new_decision', 'Native fixture adds2scope/8acceptance/3scenario/7sources; invalid old receipt/no receipt digest/no discharge and finite new-decision action asserted.'),
    test('integration', 'segmented_new_decision_preserves_history_and_rejects_committed_replay', 'Runs both leaf and whole-profile replacement; current independent record succeeds and preserves old bytes/history once.')])
row('M1.2', '相邻A-to-B仅authoritySource；正确整对象governanceProfile replace正例', 'proven', [
    test('integration', 'segmented_whole_profile_replace_has_source_only_projection', 'Real native whole-profile replace has one changedValues entry; new-decision safeAction asserted.'),
    test('integration', 'segmented_new_decision_preserves_history_and_rejects_committed_replay', 'Both encodings record a new exact receipt and reject committed replay.')])
row('M1.3', 'H/P只允许NEW决定；旧收据仍invalid且不消除Unknown', 'proven', [
    test('integration', 'segmented_authority_source_change_exposes_only_new_decision', 'Both material_review_receipt_invalid and repository_material_inspection_unavailable remain; review_receipt_digest is None and old sidecar unchanged.')])

reject = 'Helper checks no new-decision action when status is readable, record Err, exact Summary bytes unchanged, material artifact-name set unchanged; it also permits a status read to fail closed.'
row('M2.1', '缺必要历史review，即使current B确认完备仍拒绝', 'proven', [
    test('integration', 'segmented_requires_each_historical_review_even_with_valid_current_confirmation', 'Deletes exactly1 R review, asserts current human_decision_required false, then read-only rejection helper. ' + reject)])
row('M2.2', '断/损坏journal、pending原生链拒绝', 'proven', [
    test('integration', 'segmented_native_history_integrity_and_false_anchor_matrix', 'Before every mutation, copied Native baseline must expose record action. broken-journal and pending-native-tail then reject without publishing. ' + reject)])
row('M2.3', '禁止profile/risk中间变化，含修改再还原', 'proven_representatives', [
    test('integration', 'segmented_rejects_forbidden_intermediate_history_and_mixed_adjacency', 'Actual cases: historical goal, historical risk, reviewerActor change then revert; adjacent extra scope and a goal no-op. ' + reject),
    test('integration', 'segmented_rejects_source_replace_that_adds_another_null_profile_field', 'Whole-profile replace with another explicit-null field is rejected; missing and null are distinct.')], 'Representative profile field is reviewerActor; no Cartesian claim over every profile field.')
row('M2.4', 'Summary自报锚点及mixed root-replace拒绝', 'proven', [
    test('integration', 'segmented_native_history_integrity_and_false_anchor_matrix', 'fake-summary-adjacency removes P native records and inserts an invented Summary anchor; baseline was proven eligible before mutation.'),
    test('integration', 'segmented_rejects_source_replace_that_adds_another_null_profile_field', reject)])
row('M2.5', 'foreign/forked native chain、历史base变更、独立非相邻锚点、直接未记账编辑在有限替换lane拒绝', 'not_fully_exercised', [
    test('cli', 'interlocked_request_rejects_invalid_provenance_without_writes', 'Generic preflight request boundary tests unjournaled_contract, summary_repo/wi and tampered/pending journal with unchanged Contract/Summary/history.'),
    test('unit', 'material_review_amendment_anchor_requires_adjacent_unique_receipt_edges', 'Uses fabricated digest graph to reject detached nonadjacent edge; this is the older ordinary receipt-chain helper, not a native finite H/P end-to-end fixture.')], 'No direct native finite-H/P negative run for well-formed foreign/fork chain, protected base transition or independent nonadjacent anchor/direct edit was located in the executed tests. The native loader/transition replay and four-root whitelist are source guards, not substitutes for those missing runs.', 'adjacent_supporting_tests_plus_static_guards_only')

row('M3.1', 'projection与持锁record共用私有typed H/P/C0，保持invalid诊断及blockers', 'proven', [
    test('integration', 'segmented_authority_source_change_exposes_only_new_decision', 'Finite action availability does not give a receipt or discharge.'),
    test('integration', 'segmented_new_decision_preserves_history_and_rejects_committed_replay', 'Existing service performs actual current record for both encodings.'),
    test('status', 'finish_and_archive_reject_canonical_material_unknowns', 'Existing lifecycle still rejects canonical Unknown at finish/archive.')], 'Source linkage: material_review.rs projection calls finite_authority_source_replacement_proof; record acquires lifecycle lock then native admission and the same proof; precommit revalidation recomputes it. No status_projection.rs edit in WI1090.')
row('M3.2', 'C1独立验证实际current request/reviewer/source/assurance input', 'proven', [
    test('integration', 'segmented_wrong_current_inputs_publish_no_prepare_or_receipt', 'Actual wrong cases request/reviewer/source/schema/empty evidence/rationale; each Err preserves Summary bytes and evidence-name set.'),
    test('integration', 'segmented_new_decision_preserves_history_and_rejects_committed_replay', 'New receipt.request_digest equals actual input; current status digest and discharge only follow that new receipt.')], 'Assurance positive path is the sole typed SelfDeclared enum and both actual M6 receipts; no claim that a wrong-assurance raw transport variant was separately run.')
row('M3.3', '当前精确材料和真实记录的批准/review来源，self_declared不升级真人认证', 'proven_with_explicit_assurance_limit', [
    test('integration', 'material_review_decision_validator_rejects_unapproved_profile_actor', 'Wrong actor is rejected by the shared typed validator.'),
    test('integration', 'material_review_decision_validator_rejects_wrong_authority_source', 'Wrong authoritySource is AuthorityMismatch.')], 'Actual M6 records include exact synthetic current plan/consent and explicitly synthetic actors; real WI1090 remains profile=null with no material grant. evidenceRefs is structural and not human authentication. This scenario verification is implementation/fixture evidence, not actual WI1090 residual-risk acceptance.')

row('M4.1', 'prepare/sidecar/Summary六个边界实际故障注入；未提交残余无discharge', 'proven', [
    test('unit', 'replacement_transaction_failure_matrix', 'Iterates before/after prepare, before/after sidecar, before/after Summary. Precommit errors preserve Summary/old sidecar and no receipt discharge; after_summary is resolved from actual canonical committed state.')])
row('M4.2', '精确pending续接复用原immutable receipt/time，history最多一次', 'proven', [
    test('unit', 'replacement_pending_retry_reuses_immutable_receipt', 'Injected after_sidecar interruption then exact retry; serialized returned receipt equals staged bytes, old sidecar unchanged and committed replay rejected.'),
    test('unit', 'replacement_transaction_failure_matrix', 'Each path ends with history exactly one old pointer; replay preserves artifacts and Summary.')])
row('M4.3', '错/冲突pending、错误input/runtime/rehashed target及committed replay拒绝', 'proven', [
    test('unit', 'replacement_pending_rejects_wrong_input_runtime_and_rehashed_target', 'Actual pending wrong rationale, different Runtime and resealed injected target Summary all reject without advancing Summary/artifacts.'),
    test('unit', 'replacement_orphan_sidecar_without_prepare_is_not_adopted', 'Actual staged sidecar with removed prepare rejects without adoption or Summary advance.')])
row('M4.4', 'CAS及source/review/目录身份变化重校验', 'proven', [
    test('unit', 'replacement_pending_summary_conflict_preserves_concurrent_state', 'Actual pending plus concurrently changed Summary rejects and preserves concurrent bytes and artifacts.'),
    test('unit', 'replacement_rechecks_source_review_and_summary_before_commit', 'Precommit callback mutates Summary/source/current review/active directory/evidence directory. Every mutation rejects; old pointer/history and concurrent state preserved.')], 'Cooperative lifecycle-lock CAS only; no power-loss durability, parent-directory fsync or arbitrary final-gap uncooperative-writer guarantee.')

row('M5.1', 'Finding、其他Unknown拒绝', 'proven_shared_validator_and_planner', [
    test('integration', 'material_review_decision_validator_rejects_findings_and_extra_unknowns', 'Finding booleans/categories -> FindingPresent; additional Unknown -> UnexpectedUnknown.'),
    test('integration', 'finding_is_never_reviewable', 'Committed Finding never becomes reviewable.')])
row('M5.2', 'dirty/unavailable source拒绝', 'proven_shared_source_boundary', [
    test('integration', 'dirty_and_untracked_source_refuse_request', 'Dirty and untracked source refuse canonical request.'),
    test('integration', 'unreadable_committed_source_refuses_request', 'Unreadable current source is rejected.'),
    test('status', 'unavailable_material_projection_blocks_verification_even_with_pending_retry', 'Existing status retains source-unavailable verification gate despite pending retry.')])
row('M5.3', '缺current review、当前review竞态/foreign通用审阅绑定拒绝', 'partly_proven', [
    test('integration', 'segmented_native_history_integrity_and_false_anchor_matrix', 'missing-current-review deletes exact matching current decision; initial finite baseline positive then rejection.'),
    test('unit', 'replacement_rechecks_source_review_and_summary_before_commit', 'Current review bytes change before commit and record rejects.'),
    test('cli', 'foreign_recorded_preflight_contract_rejects_review_without_writes', 'Generic Contract review boundary rejects foreign recorded preflight Contract.')], 'Dedicated stale/foreign current-review variants on an otherwise eligible native finite-H/P fixture were not run; generic review-boundary evidence is labelled separately.')
row('M5.4', 'request/blob/hunk/profile/scanner/policy变化拒绝旧决定', 'partly_proven', [
    test('integration', 'material_review_decision_validator_rejects_stale_input_and_tampered_request', 'Old requestDigest rejected; changed entry path without resealing rejected as RequestDigestMismatch.'),
    test('integration', 'old_material_decision_is_stale_after_analysis_or_policy_identity_changes', 'Rebinds actual typed request after analysis implementation/target semantic profile/policy changes: new input accepted and old input rejected.'),
    test('integration', 'adding_a_syntax_unknown_adds_its_plan_member_without_fixed_cardinality', 'New committed Rust member joins exact request and changes digest.'),
    test('integration', 'stale_receipt_with_changed_contract_profile_is_not_renewable', 'Generic changed-profile receipt is not an ordinary renewal.')], 'No dedicated after-P Rust blob/hunk member mutation or per-field forged canonical member-digest finite record case was found. Public record accepts only decision input and recomputes request, so direct request injection is not its API; no exploit is asserted.')
row('M5.5', '腐败/重复JSON/symlink receipt/pointer/history/chain拒绝', 'partly_proven', [
    test('integration', 'segmented_native_history_integrity_and_false_anchor_matrix', 'Duplicate prepared JSON, duplicate sidecar JSON with resealed pointer digest and broken journal all reject from a proven positive baseline.'),
    test('integration', 'segmented_old_sidecar_symlink_does_not_gain_replacement_eligibility', 'Actual old sidecar symlink is rejected without new publication.'),
    test('integration', 'stale_material_review_receipt_requires_valid_prior_sidecar', 'Corrupt, resealed foreign-repository identity and missing old sidecar are invalid and do not publish a new sidecar.'),
    test('integration', 'material_review_rejects_a_summary_pointer_to_a_noncanonical_sidecar_path', 'Copied valid receipt under a noncanonical pointer path cannot admit renewal.'),
    test('unit', 'replacement_pending_symlinks_reject_without_following_or_overwriting', 'Pending prepare and pending new-sidecar symlinks reject; old bytes/current Summary/artifacts remain.')], 'No direct symlinked native journal or well-formed foreign pointer-history sidecar end-to-end variant was found; the existing foreign-pointer-history test uses a malformed string member. Bounded nofollow/history identity source guards are present; no Cartesian-product execution claim.')
row('M5.6', '相邻换绑后来源变化拒绝', 'proven', [
    test('integration', 'segmented_rejects_committed_source_changed_after_adjacent_anchor', 'Commits README change after B, refreshes review and still rejects finite proof; not merely a stale-review failure.'),
    test('unit', 'replacement_rechecks_source_review_and_summary_before_commit', 'Actual source write after sidecar publication rejects before Summary commit.')])
row('M5.7', '工作流拒绝把旧批准扩展到新binding', 'current_workflow_stopped_no_grant', [], 'Actual current plan is reviewEnabled=false/profile=null and only one structurally reviewable Unknown. New WI1090 risk decision has not occurred. Technical review, synthetic fixture decisions and old WI1089 consent are not recorded as real material approval. A minimal policy proposal is prepared only. This is an explicit uncompleted real-material workflow, not a Rust human-authentication test.', 'workflow_scope_and_current_native_facts')

decisions = {
    'M1 authoritySource-only proof': {'nativeRecord': 'verified', 'reason': 'All original positive H/P/new-decision/no-discharge clauses map to passing native fixture integration tests and both actual M6 encodings.'},
    'M2 forbidden amendments': {'nativeRecord': 'unverified', 'reason': 'Keep partial: dedicated finite-native foreign/fork/base/direct/nonadjacent cases not all exercised; generic preflight/ordinary-chain guards are not relabelled as that proof.'},
    'M3 shared admission and service proof': {'nativeRecord': 'verified', 'reason': 'Shared private proof and native admission under lifecycle lock plus actual independent C1 fixtures/wrong-input rejection; assurance remains self_declared, not a real WI1090 risk grant.'},
    'M4 logical atomicity and bounded resume': {'nativeRecord': 'verified', 'reason': 'Actual six publication boundaries, exact original receipt retry, wrong/conflicting pending, replay, CAS/current-source/review/namespace races all passed; cooperative-lock logical atomicity only.'},
    'M5 integrity and source rejection': {'nativeRecord': 'unverified', 'reason': 'Keep partial: named finite-current authorization/member/symlink-history variants and actual real-material risk workflow remain incomplete; no full-matrix green is inferred.'},
}
source_guards = [{'path': 'crates/cockpit-repository/src/material_review.rs', 'function': name,
                  'line': next(i + 1 for i, line in enumerate((root/'crates/cockpit-repository/src/material_review.rs').read_text().splitlines()) if line.startswith('fn ' + name + '('))}
                 for name in ['finite_authority_source_replacement_proof', 'read_material_review_receipt_binding',
                              'validate_material_review_decision_with_contract_digest', 'revalidate_material_replacement']]
mapping = {'kind': 'inspected_acceptance_clause_mapping_not_new_test_execution', 'observedAt': datetime.now(timezone.utc).isoformat(),
    'contractDigest': envelope['contractDigest'], 'originalAcceptanceCriteria': contract['acceptanceCriteria'],
    'originalScenarios': contract['scenarioCoverage'][:5], 'canonicalVerificationPath': str(canonical_path.relative_to(root)),
    'canonicalRawSha256': hashlib.sha256(canonical_bytes).hexdigest(), 'canonicalReceiptDigest': envelope['receiptDigest'],
    'actualFormalSourceHead': tested, 'currentEvidenceOnlyHead': current, 'formalStartedAt': metadata['startedAt'],
    'formalEndedAt': metadata['endedAt'], 'runtimeDigest': envelope['runtimeDigest'], 'noChecksRerun': True,
    'rows': rows, 'scenarioRecordingDecisions': decisions, 'sourceGuardsLabelledStatic': source_guards,
    'nonblockingDeferred': 'Parent explicitly deferred pending-after-old-sidecar/historical-review/journal deletion/tamper source additions. Shared proof revalidation is statically reviewed; no such additional test is claimed run.',
    'assurance': 'Verified scenario labels mean the stated implementation/test behavior under retained evidence, never actual real-person authentication, WI1090 material residual-risk consent, Sentinel recovery or terminal lifecycle authority.'}
path = out / 'M1-M5-clause-to-evidence-map.actual.agent-recorded.json'
assert not path.exists(); path.write_text(json.dumps(mapping, ensure_ascii=False, indent=2) + '\n')
lines = ['# WI1090 M1–M5 原始条件与实际证据映射', '',
         '仅重读已保留证据；未运行 Cargo/verify。当前正式执行源码为 `' + tested + '`；c1affb57与其相关Rust测试/实现bytes相同。', '',
         '正式7项 canonical raw SHA：`' + mapping['canonicalRawSha256'] + '`。通过标记、日志行与函数源码逐项核对；不把源码守卫称作缺失的运行结果。', '',
         '| 原始条件 | 判定 | 精确测试/断言/日志 | 缺项或保证限制 |', '|---|---|---|---|']
for row in rows:
    refs = '; '.join(x['test'] + ' (`' + x['sourcePath'] + ':' + str(x['sourceLine']) + '`, ' + x['formalNodeId'] + ' stdout:' + str(x['passedStdoutLine']) + ')' for x in row['evidence'])
    lines.append('| ' + row['id'] + ' ' + row['condition'] + ' | ' + row['coverage'] + ' | ' + refs + ' | ' + str(row['gapOrLimit'] or '') + ' |')
lines += ['', '## 本次原生记录选择', '']
for name, value in decisions.items(): lines.append('- ' + name + ': `' + value['nativeRecord'] + '` — ' + value['reason'])
lines += ['', '## 单独保留的边界', '', mapping['nonblockingDeferred'], '', mapping['assurance'],
          '', 'M6：实际4执行/0复用来自 reuse_not_configured，仅证明执行；没有证明已配置缓存的失效或Runtime变化阻止复用。',
          '', '完整3176项清单未公开；原始prospective与actual文件均在外部，SHA均299d90d4…，字节相同；重算现场blob/mode结果missing0/extra0/mismatch0。详见补充JSON。']
path = out / 'M1-M5-evidence-map.actual.zh-CN.md'; assert not path.exists(); path.write_text('\n'.join(lines) + '\n')
print(json.dumps({'rows': len(rows), 'decisions': decisions, 'mappingRawSha256': hashlib.sha256((out/'M1-M5-clause-to-evidence-map.actual.agent-recorded.json').read_bytes()).hexdigest()},ensure_ascii=False,indent=2))
