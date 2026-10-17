from pathlib import Path
import json

repo = Path('/workspace/ai-cockpit-wi1088-runtime-continuity')
root = Path('/workspace/scratch/wi1088-runtime-continuity')
wi = 'WI-1088-runtime-activation-amendment-continuity-recovery'
result = json.loads((root / 'commands/n04-full-declared-verify.stdout').read_text())
assert result['nodesExecuted'] == 33 and result['nodesReused'] == 0
assert 'exitCode' in json.loads((root / 'commands/n04-full-declared-verify.json').read_text())
assert len(result['executionRecords']) == 33
source = json.loads((root / 'commands/n04-full-declared-verify.json').read_text())['sourceHead']
translations = {
    '': ('Runtime activation continuity and present-baseline recovery', 'Reviewed implementation and evidence for strict activation continuity and present-baseline recovery.', 'Actual human:Ray confirm_review was natively recorded at 08:45:53Z; checkpoint 1 is established. Implementation and actual verification evidence are preserved. This remains an in-progress Draft with no terminal lifecycle claim.', f'Production repair source 84b7dc49d56f4848ccd8d4616f91b1c5249b3f26 passed the complete 33-node declared native route at {source}, with one worker, an isolated target, and no node timeout. Exact receipts, all prior RED attempts and source bindings are retained in the execution ledger. Current-head CI and later evidence are reported in Draft PR1047; software verification does not mean Runtime delivery, actual WI-1087 recovery or lifecycle completion.'),
    '.zh-CN': ('Runtime activation 连续性与当下 baseline 恢复', '严格 activation 连续性及当下 baseline 恢复的已审实现与证据。', 'human:Ray 在 08:45:53Z 的真实 confirm_review 已原生记录，checkpoint 1 已建立。实现与真实验证证据已保留；当前仍为进行中的 Draft，不主张终端生命周期完成。', f'生产修复源码 84b7dc49d56f4848ccd8d4616f91b1c5249b3f26 在 {source} 通过完整 33 节点原生声明路线，使用单 worker、隔离 target，各节点无超时。执行台账保留精确收据、此前全部 RED 和源码绑定。当前 HEAD 的 CI 及后续证据见 Draft PR1047；软件验证不表示 Runtime 已交付、WI-1087 已实际恢复或生命周期已完成。'),
    '.ja': ('Runtime activation の連続性と現時点 baseline の復旧', '厳密な activation の連続性と現時点 baseline 復旧の承認済み実装と証拠。', 'human:Ray の実際の confirm_review は 08:45:53Z にネイティブ記録済みで、checkpoint 1 が成立しています。実装と実際の検証証拠を保持しています。現在も進行中の Draft であり、終端ライフサイクルの完了は主張しません。', f'実装修正ソース 84b7dc49d56f4848ccd8d4616f91b1c5249b3f26 は {source} で完全な 33 ノードのネイティブ宣言ルートに成功しました。単一 worker と分離 target を使用し、ノードのタイムアウトはありません。実行台帳は正確な receipt、過去の全 RED、ソース対応を保持します。現在 HEAD の CI と後続証拠は Draft PR1047 に記載します。ソフトウェア検証は Runtime 配布、WI-1087 の実復旧、ライフサイクル完了を意味しません。'),
}
statuses = {'': 'In progress → Implemented after verified close', '.zh-CN': '进行中 → 验证关闭后已实现', '.ja': '進行中 → 検証済みクローズ後に実装済み'}
endings = {
    '': 'The execution ledger retains the exact actual native33-node route, scoped results, prior RED attempts, source bindings and CI diagnoses. Current acceptance and exact-head CI are reported in Draft PR1047. Software evidence does not mean Runtime delivery, actual WI-1087 recovery or lifecycle completion.',
    '.zh-CN': '执行台账保留原生33节点路线、聚焦结果、此前全部 RED、源码绑定及 CI 诊断的精确实际记录。当前验收与精确 HEAD 的 CI 见 Draft PR1047。软件证据不表示 Runtime 已交付、WI-1087 已实际恢复或生命周期已完成。',
    '.ja': '実行台帳はネイティブ33ノードのルート、限定的な結果、過去の全 RED、ソース対応、CI 診断の正確な実記録を保持します。現在の受入れと正確な HEAD の CI は Draft PR1047 に記載します。ソフトウェア証拠は Runtime 配布、WI-1087 の実復旧、ライフサイクル完了を意味しません。',
}
for suffix, (title, description, opening, ending) in translations.items():
    ending = endings[suffix]
    path = repo / f'docs/work-items/{wi}{suffix}.md'
    text = path.read_text()
    assert text.startswith('---\nworkItemId:')
    text = text.replace('---\nworkItemId:', '---\nauthor: AI Cockpit maintainers\ntitle: ' + json.dumps('WI-1088 — ' + title, ensure_ascii=False) + '\ndescription: ' + json.dumps(description, ensure_ascii=False) + '\naudience:\n  - maintainer\n  - reviewer\nauthority: explicit-user-authorization\nworkItemId:', 1)
    lines = text.splitlines()
    heading = next(i for i, line in enumerate(lines) if line.startswith('# '))
    lines[heading + 2] = opening
    lines[-1] = ending
    path.write_text('\n'.join(lines) + '\n')
    path = repo / f'docs/reference/reference-parity{suffix}.md'
    lines = path.read_text().splitlines(keepends=True)
    matches = [i for i, line in enumerate(lines) if wi in line]
    assert len(matches) == 1
    index = matches[0]
    cells = lines[index].rstrip('\n').split('|')
    assert len(cells) == 5
    cells[2] = ' ' + statuses[suffix] + ' '
    cells[3] = f' [Work Item](../work-items/{wi}{suffix}.md); active Contract `.ai/work-items/active/{wi}.contract.json`; planned archive `.ai/work-items/archive/{wi}.contract.json`; planned verification `.ai/evidence/{wi}.verification.json`; planned finalization `.ai/decisions/{wi}.finalize.json`; planned close `.ai/decisions/{wi}.close.json`. Contract `sha256:6e93cf3ec7618a3b95cddc8796942c09e37ae43c3d9eb1dd454e7c99452013a7`; checkpointCount=1; actual human review recorded; native RED and actual formal attempts at `{source}` retained. Current CI/evidence: Draft PR1047. Planned terminal records are not asserted present. No actual WI-1087 recovery or terminal lifecycle. '
    lines[index] = '|'.join(cells) + '\n'
    path.write_text(''.join(lines))
print('Updated only own three briefs and three parity rows after actual terminal; no success inferred.')
