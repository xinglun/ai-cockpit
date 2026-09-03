---
workItemId: WI-531-historical-direct-merge-apply
title: "WI-531 — bundled historical direct-merge の適用"
status: in_progress
mode: code
author: AI Cockpit maintainers
description: "実際の bundled merge parent と不変な Contract base を別々の監査事実として束縛する。"
audience:
  - maintainer
  - adopter
authority: canonical
lastVerifiedBy: documentation-acceptance
---

# WI-531 — bundled historical direct-merge の適用

## Intent と境界

Pull Request なしで複数 Work Item が一つの merge にまとめられた adopter を、公開
Runtime で正直に収束できるようにする。optional な `historical.contractBaseRevision` を
追加し、`pullRequest.baseRevision` は実際の merge commit の第一 parent に bind する。
resource-context の不一致カテゴリも示し、歴史 bytes を編集せず plan を完成できることを
証明する。object repository の変更と PR の捏造は行わない。

## Acceptance

- bundled merge parent と Contract base が異なっても、完全な plan receipt を最初の
  canonical direct-merge record として受理できる。
- Contract base、context、repository、Work Item、Git parent、Runtime の欠落/foreign facts
  は具体的な field を示して fail-closed する。
- 英語・中国語・日本語の command docs が deterministic facts と human-owned fields を説明する。
- preserved/generated context、bundled base drift、malformed input、拒否時 no-write を
  protocol/repository tests で検証する。

## Compatibility

新 field は optional で、merge base が Contract base と同じ既存 receipt はそのまま読める。
不一致を許すのは、正確な archived Contract digest と Contract base を明示的に束縛する
`direct_merge_no_pr`/`historical_low` receipt だけである。

## Object repository handoff

`/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator` は read-only。Release 後、
team は `finalize-recovery-plan --merge-commit <sha>` を再実行し、二つの base field を保持して
生成された receipt だけを適用する。`resourceContext.<field>` が残れば field 名を報告し、`.ai/`
を手編集しない。
